#!/usr/bin/env python3
"""Train answer-supervised text-to-program grounding. NEVER reads SVAMP test data.

Dependencies: numpy 2.3.5, scipy 1.17.0, scikit-learn 1.8.0.
Teacher equations, gold number groups and problem IDs are NOT learner inputs.
The arithmetic search grammar and statistical learner are developer supplied.
"""
from __future__ import annotations
import argparse,collections,csv,hashlib,itertools,json,math,os,re,time
from pathlib import Path
os.environ.setdefault('OMP_NUM_THREADS','2')
os.environ.setdefault('OPENBLAS_NUM_THREADS','2')
import numpy as np
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.svm import LinearSVC
from sklearn.model_selection import GroupShuffleSplit

OPS=['+','-','*','/']
CACHE={}
def candidates(n):
    if n in CACHE:return CACHE[n]
    leaves=[(f'n{i}',) for i in range(n)]
    def join(op,a,b):
        if op in ['+','*'] and a>b:a,b=b,a
        return (op,)+a+b
    two=[]
    for i,j in itertools.combinations(range(n),2):
        for op in OPS:
            two.append(join(op,leaves[i],leaves[j]))
            if op in ['-','/']:two.append(join(op,leaves[j],leaves[i]))
    three=[]
    for a in two:
        used={int(x[1:]) for x in a if x.startswith('n')}
        for i in range(n):
            if i not in used:
                for op in OPS:
                    three.append(join(op,a,leaves[i]))
                    if op in ['-','/']:three.append(join(op,leaves[i],a))
    CACHE[n]=sorted(set(leaves+two+three),key=lambda p:(len(p),p))
    return CACHE[n]
def evaluate(p,values):
    stack=[]
    for t in reversed(p):
        if t.startswith('n'):stack.append(values[int(t[1:])])
        else:
            a,b=stack.pop(),stack.pop()
            if t=='+':v=a+b
            elif t=='-':v=a-b
            elif t=='*':v=a*b
            elif b==0:return math.nan
            else:v=a/b
            stack.append(v)
    return stack[0]
def as_expr(p):
    it=iter(p)
    def get():
        s=next(it)
        if s.startswith('n'):return {'Input':int(s[1:])}
        return {'Bin':[{'+':'Add','-':'Sub','*':'Mul','/':'DivExact'}[s],get(),get()]}
    return get()
def tokens(s):return re.findall(r'number\d+|[a-z][a-z0-9]*|[.,?!;]',s.lower())
def features(text,n,kind='role'):
    ts=tokens(text)
    qpos=next((i for i in range(len(ts)-1,-1,-1) if ts[i] in ['how','what','find','calculate','determine']),len(ts))
    norm=['num' if re.fullmatch(r'number\d+',x) else x for x in ts]
    parts=[('all',norm)] if kind=='bag' else [('all',norm),('q',norm[qpos:]),('b',norm[:qpos])]
    f=[]
    for part,tt in parts:
        for k in range(1,2 if kind=='bag' else 4):
            for i in range(len(tt)-k+1):f.append(part+':'+ '_'.join(tt[i:i+k]))
    if kind=='role':
        for i,x in enumerate(ts):
            if re.fullmatch(r'number\d+',x):
                for j in range(max(0,i-5),min(len(ts),i+6)):
                    if i!=j:f.append(x+':'+str(j-i)+':'+norm[j])
                for j in range(i+1,min(len(ts),i+6)):
                    if norm[j] in norm[qpos:]:f.append(x+':qoverlap:'+norm[j])
    f.append('count:'+str(n));return ' '.join(f)
def ground_raw(text):
    # Mirrors the PUBLIC raw-text tokenizer, not corpus-supplied numeric groups.
    text=text.lower();i=0;ts=[];values=[]
    while i<len(text):
        c=text[i]
        sign=c in '+-' and i+1<len(text) and text[i+1].isascii() and text[i+1].isdigit() and (i==0 or not text[i-1].isalnum())
        if (c.isascii() and c.isdigit()) or sign:
            start=i
            if sign:i+=1
            while i<len(text) and text[i].isascii() and text[i].isdigit():i+=1
            while i+3<len(text) and text[i]==',' and all(x.isascii() and x.isdigit() for x in text[i+1:i+4]) and (i+4==len(text) or not text[i+4].isdigit()):i+=4
            if i+1<len(text) and text[i]=='.' and text[i+1].isdigit():
                i+=1
                while i<len(text) and text[i].isdigit():i+=1
            values.append(text[start:i].replace(',',''));ts.append('number'+str(len(values)-1))
        elif 'a'<=c<='z':
            start=i
            while i<len(text) and text[i].isascii() and text[i].isalnum():i+=1
            word=text[start:i]
            if re.fullmatch(r'number\d+',word):raise ValueError('reserved slot literal')
            ts.append(word)
        else:
            if c.isalpha():raise ValueError('non-ASCII training language')
            if c in '.,?!;':ts.append(c)
            i+=1
    return ts,values
def raw_question(row):
    return re.sub(r'number(\d+)',lambda m:row['values'][int(m[1])],row['text'])
def load_training(path):
    result=[]
    for row in csv.DictReader(path.open()):
        # This whitelist is the full teacher interface. In particular no Equation,
        # Body, Ques, group_nums, or task metadata reaches the learner.
        text=row['Question'];values=row['Numbers'].split();answer=float(row['Answer'])
        slots=[int(x) for x in re.findall(r'number(\d+)',text)]
        if slots!=list(range(len(values))):raise ValueError('non-monotonic source slot contract')
        if not math.isfinite(answer):raise ValueError('non-finite teacher')
        raw=re.sub(r'number(\d+)',lambda m:values[int(m[1])],text)
        tt,actual_values=ground_raw(raw)
        result.append({'text':' '.join(tt),'values':actual_values,'answer':answer})
    return result

def induce(rows):
    compatible=[]
    for r in rows:
        vals=list(map(float,r['values']));positive=[]
        if len(vals)<=6:
            for p in candidates(len(vals)):
                v=evaluate(p,vals)
                if math.isfinite(v) and abs(v-r['answer'])<=1e-7*max(1,abs(r['answer'])):positive.append(p)
        if positive:
            smallest=min(map(len,positive));positive=[p for p in positive if len(p)==smallest]
        compatible.append(positive)
    return compatible

def fit(rows,compatible,indices,kind,C):
    usable=[i for i in indices if compatible[i]]
    pool=sorted({p for i in usable for p in compatible[i]})
    ids={p:i for i,p in enumerate(pool)};freq=np.zeros(len(pool))
    for i in usable:
        for p in compatible[i]:freq[ids[p]]+=1/len(compatible[i])
    labels=np.array([max((ids[p] for p in compatible[i]),key=lambda j:(freq[j],-j)) for i in usable])
    vec=TfidfVectorizer(token_pattern=r'\S+',lowercase=False,min_df=2,sublinear_tf=True,max_features=40000)
    X=vec.fit_transform([features(rows[i]['text'],len(rows[i]['values']),kind) for i in usable])
    model=LinearSVC(C=C,dual='auto',random_state=913,max_iter=3000)
    model.fit(X,labels)
    return vec,model,pool,usable,labels

def export(vec,m,pool,usable,labels,kind,n,source_sha):
    names=sorted(vec.vocabulary_);order=[vec.vocabulary_[w] for w in names]
    co=m.coef_[:,order]
    scales=np.maximum(np.max(np.abs(co),axis=1)/32760,1e-12)
    quant=np.clip(np.rint(co/scales[:,None]),-32760,32760).astype(np.int16)
    counts=collections.Counter(map(int,labels))
    payload={'version':1,'feature_kind':kind,'source_sha256':source_sha,'training_examples':n,
             'supported_training_examples':len(usable),'vocabulary':names,
             'idf':vec.idf_[order].tolist(),'programs':[{'expression':as_expr(pool[i]),'support':counts[int(i)]} for i in m.classes_],
             'intercept':m.intercept_.tolist(),'scales':scales.tolist(),
             'weights':quant.T.astype('>i2').tobytes().hex(),
             'developer_margin':1.0}
    return payload,quant

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--train',type=Path,required=True);ap.add_argument('--output',type=Path,required=True)
    ap.add_argument('--development',action='store_true');args=ap.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    start=time.perf_counter();rows=load_training(args.train);compatible=induce(rows)
    report={'teacher_records':len(rows),'answer_explainable_records':sum(bool(c) for c in compatible),
            'unsupported_training_records_counted':sum(not c for c in compatible),'teacher_equations_used':False,
            'grammar':'Input(0..5), Add/Sub/Mul/exact rational Div; no repeated input, <=2 operations; minimal explanatory programs',
            'program_selection':'minimum size then corpus compatible frequency; weak supervision can be spurious',
            'source_sha256':hashlib.sha256(args.train.read_bytes()).hexdigest(),'test_access':False,
            'pretrained_weights':False,'llm_calls':0,'models':{}}
    # The 650 held-out DEVELOPMENT records are groups of normalized training text,
    # not the 1000 SVAMP challenge. These settings were selected only on this split.
    groups=[' '.join(tokens(r['text'])) for r in rows]
    train,dev=next(GroupShuffleSplit(n_splits=1,test_size=.2,random_state=2701).split(rows,groups=groups))
    report['development_split']={'seed':2701,'grouped':True,'dev_records':len(dev),'training_indices':train.tolist(),'dev_indices':dev.tolist()}
    for kind,C in [('role',1.0),('bag',3.0)]:
        if args.development:
            v,m,pool,usable,y=fit(rows,compatible,train,kind,C)
            sc=m.decision_function(v.transform([features(rows[i]['text'],len(rows[i]['values']),kind) for i in dev]))
            good=0
            for t,i in enumerate(dev):
                vals=list(map(float,rows[i]['values']))
                eligible=[j for j,c in enumerate(m.classes_) if max(int(z[1:]) for z in pool[c] if z.startswith('n'))<len(vals)]
                ranked=sorted(eligible,key=lambda j:-sc[t,j])
                prediction=next((evaluate(pool[m.classes_[j]],vals) for j in ranked if math.isfinite(evaluate(pool[m.classes_[j]],vals))),math.nan)
                good+=abs(prediction-rows[i]['answer'])<1e-5
            report['models'][kind]={'development_correct':good,'development_total':len(dev),'C':C}
            print('development',kind,good,len(dev),flush=True)
        v,m,pool,usable,y=fit(rows,compatible,range(len(rows)),kind,C)
        payload,q=export(v,m,pool,usable,y,kind,len(rows),report['source_sha256'])
        out=args.output/(kind+'-model.json');out.write_text(json.dumps(payload,separators=(',',':'))+'\n')
        report['models'].setdefault(kind,{'C':C}).update({'features':len(payload['vocabulary']),'programs':len(payload['programs']),
            'model_sha256':hashlib.sha256(out.read_bytes()).hexdigest(),'model_bytes':out.stat().st_size,
            'quantization':'per-class int16, 32760 levels; f64 accumulation'})
        # Fixtures use TRAINING input only. Native parity is verified before test.
        fixtures=[]
        order=[v.vocabulary_[w] for w in payload['vocabulary']]
        for i in range(0,len(rows),31):
            if len(rows[i]['values'])>6:continue
            xx=v.transform([features(rows[i]['text'],len(rows[i]['values']),kind)])[:,order]
            scores=np.asarray(xx.dot((q*np.array(payload['scales'])[:,None]).T)).ravel()+np.array(payload['intercept'])
            fixtures.append({'question':raw_question(rows[i]),'tokens':tokens(rows[i]['text']),
                'counts':dict(collections.Counter(features(rows[i]['text'],len(rows[i]['values']),kind).split())),
                'scores':scores.tolist()})
        (args.output/(kind+'-parity.json')).write_text(json.dumps(fixtures)+'\n')
        print('exported',kind,out.stat().st_size,flush=True)
    report['elapsed_seconds']=time.perf_counter()-start
    (args.output/'training-report.json').write_text(json.dumps(report,indent=2)+'\n')
    # Store only training questions/answers for retrieval controls, never test labels.
    (args.output/'teacher.json').write_text(json.dumps(rows)+'\n')
if __name__=='__main__':main()
