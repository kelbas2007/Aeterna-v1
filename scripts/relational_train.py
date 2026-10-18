"""REL-1 training-derivation supervision. Never reads external test data.

Primitive arithmetic and the pair-factor decomposition are authored. Train
Equation is now an explicit teacher derivation: this is NOT weak supervision.
Model selection uses a grouped split inside the original train.csv only.
"""
import os
os.environ['OMP_NUM_THREADS']='2';os.environ['OPENBLAS_NUM_THREADS']='2'
import sys,math,re,csv,json,itertools,time,pickle,collections,hashlib
from pathlib import Path
import numpy as np
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.svm import LinearSVC
from sklearn.model_selection import GroupShuffleSplit
import word_problem_train as prev
OPS=['+','-','r-','*','/','r/','none']
REV={'+':'+','-':'r-','r-':'-','*':'*','/':'r/','r/':'/','none':'none'}

def read_prefix(text):
    it=iter(text.split())
    def get():
        t=next(it)
        if t in '+-*/':return (t,get(),get())
        if t.startswith('number'):return int(t[6:])
        raise ValueError('literal')
    tree=get()
    if next(it,None) is not None:raise ValueError('trailing')
    return tree

def norm(e):
    if isinstance(e,int):return e
    op,a,b=e;a,b=norm(a),norm(b)
    additive=op in '+-';symbols='+-' if additive else '*/';items=[]
    def gather(x,sign):
        if isinstance(x,tuple) and x[0] in symbols:
            gather(x[1],sign);gather(x[2],sign if x[0] in '+*' else -sign)
        else:items.append((x,sign))
    gather((op,a,b),1)
    pos=sorted([x for x,s in items if s==1],key=repr);neg=sorted([x for x,s in items if s==-1],key=repr)
    if not pos:raise ValueError('no positive')
    def fold(seq):
        v=seq[0]
        for x in seq[1:]:v=('+' if additive else '*',v,x)
        return v
    p=fold(pos)
    return ('-' if additive else '/',p,fold(neg)) if neg else p

def leaves(e):return {e} if isinstance(e,int) else leaves(e[1])|leaves(e[2])
def numeric(e,v):
    if isinstance(e,int):return v[e]
    a=numeric(e[1],v);b=numeric(e[2],v)
    if e[0]=='+':return a+b
    if e[0]=='-':return a-b
    if e[0]=='*':return a*b
    return a/b if b else float('nan')
def relation(e,i,j):
    s=leaves(e)
    if i not in s or j not in s:return 'none'
    left=leaves(e[1]);right=leaves(e[2])
    if i in left and j in left:return relation(e[1],i,j)
    if i in right and j in right:return relation(e[2],i,j)
    return e[0] if i in left else REV[e[0]]

CACHE={}
def pool(n):
    if n in CACHE:return CACHE[n]
    out=set(range(n));two=[]
    for a,b in itertools.combinations(range(n),2):
        for op in '+-*/':
            two.append(norm((op,a,b)))
            if op in '-/':two.append(norm((op,b,a)))
    out.update(two)
    for e in two:
        for j in set(range(n))-leaves(e):
            for op in '+-*/':
                out.add(norm((op,e,j)))
                if op in '-/':out.add(norm((op,j,e)))
    out=sorted(out,key=lambda x:(len(leaves(x)),repr(x)))
    use=np.array([[int(i in leaves(e)) for i in range(n)] for e in out])
    rel=np.array([[OPS.index(relation(e,i,j)) for i,j in itertools.combinations(range(n),2)] for e in out])
    CACHE[n]=(out,use,rel);return CACHE[n]

STOP={'a','an','the','is','are','was','were','of','at','to','for','and','in','on','by','his','her','their','has','had','have','he','she','it','they','if','then','there','that','this','with','each','how','many','much','what','does','do','did','be','all','will','can','from','as','or','so','now','some'}
def stem(w):
    if len(w)>4 and w.endswith('ies'):return w[:-3]+'y'
    if len(w)>4 and w.endswith('ing'):return w[:-3]
    if len(w)>3 and w.endswith('s') and not w.endswith('ss'):return w[:-1]
    return w

def context(ts):
    q=next((i for i in range(len(ts)-1,-1,-1) if ts[i] in ['how','what','find','calculate','determine']),len(ts))
    positions=[i for i,w in enumerate(ts) if re.fullmatch('number[0-9]+',w)]
    ss=[stem(w) for w in ts]; return ss,positions,q

def ff(ts,n,a,b=None,cross=True):
    ts,pos,q=context(ts)
    if len(pos)!=n:raise ValueError('pos')
    ti=pos[a];tj=pos[b] if b is not None else None
    # No corpus index or numeric value is a feature. A/B mark ordered arguments.
    words=['A' if k==ti else 'B' if k==tj else 'N' if re.fullmatch('number[0-9]+',w) else w for k,w in enumerate(ts)]
    out=[]
    def add(p,ww,maxn=2):
        for k in range(1,maxn+1):
            for i in range(len(ww)-k+1):out.append(p+':'+ '_'.join(ww[i:i+k]))
    add('all',words,2);add('q',words[q:],3)
    qa=[w for w in words[q:] if w not in STOP and w.isalpha() and w not in ['A','B','N']]
    local=[]
    for p,k in [('a',ti)]+([('b',tj)] if tj is not None else []):
        lo=k
        while lo>0 and words[lo-1] not in '.?!;':lo-=1
        hi=k+1
        while hi<len(words) and words[hi] not in '.?!;':hi+=1
        add(p+'s',words[lo:hi],2)
        add(p+'l',words[max(lo,k-5):k],2);add(p+'r',words[k+1:min(hi,k+6)],2)
        for j in range(max(0,k-5),min(len(ts),k+6)):
            if j!=k:out.append(p+':'+str(j-k)+':'+words[j])
        near=[w for w in words[max(lo,k-5):min(hi,k+6)] if w not in STOP and w.isalpha() and w not in ['A','B','N']]
        local.append(near)
        out.append(p+':in_question:'+str(int(k>=q)))
        for j in range(k+1,min(k+5,hi)):
            if words[j] in qa:out.append(p+':qmatch:'+str(j-k))
        if cross:
            for w in near:
                for v in qa:out.append(p+'xq:'+w+'|'+v)
    if b is not None:
        out.append('distance:'+str(min(5,abs(ti-tj)//4)))
        out.append('order:'+str(int(ti<tj)))
        out.append('same_clause:'+str(int(not any(w in '.?!;' for w in words[min(ti,tj)+1:max(ti,tj)]))))
        out.append('same_unit:'+str(int(ti+1<len(ts) and tj+1<len(ts) and words[ti+1]==words[tj+1])))
        if cross:
            for x in local[0]:
                for y in local[1]:out.append('axb:'+x+'|'+y)
    out.append('count:'+str(n))
    return ' '.join(out)

def load(path):
    result=[]
    for index,r in enumerate(csv.DictReader(Path(path).open(encoding='utf-8'))):
        # Teacher-only interface. IDs and group_nums are not used even here.
        raw=re.sub(r'number(\d+)',lambda m:r['Numbers'].split()[int(m[1])],r['Question'])
        ts,values=prev.ground_raw(raw)
        answer=float(r['Answer']);e=None;reason=None
        try:
            if not 1<=len(values)<=6:raise ValueError('number_scope')
            e=norm(read_prefix(r['Equation']))
            if len(values)!=len(r['Numbers'].split()):raise ValueError('raw_number_mismatch')
            def uses(t):return [t] if isinstance(t,int) else uses(t[1])+uses(t[2])
            us=uses(e)
            if len(us)>4 or len(set(us))!=len(us):raise ValueError('expression_scope')
            if max(us)>=len(values):raise ValueError('missing_quantity')
            if abs(numeric(e,list(map(float,values)))-answer)>1e-5:raise ValueError('teacher_disagreement')
        except (ValueError,StopIteration,ZeroDivisionError,IndexError) as error:
            reason=str(error) or type(error).__name__;e=None
        result.append({'text':raw,'ts':ts,'values':list(map(float,values)),
                       'answer':answer,'tree':e,'unsupported_reason':reason,'index':index})
    return result

def fit(rows,indices,C=1.0,cross=True):
    result={}
    for typ in ['unary','pair']:
        fs=[];ys=[]
        for i in indices:
            r=rows[i];n=len(r['values']);e=r['tree']
            if e is None:continue
            seq=[(a,None) for a in range(n)] if typ=='unary' else [(a,b) for a,b in itertools.combinations(range(n),2)]
            for a,b in seq:
                fs.append(ff(r['ts'],n,a,b,cross))
                ys.append(int(a in leaves(e)) if b is None else OPS.index(relation(e,a,b)))
        v=TfidfVectorizer(token_pattern=r'\S+',lowercase=False,sublinear_tf=True,min_df=2,max_features=60000)
        x=v.fit_transform(fs)
        m=LinearSVC(C=C,dual='auto',random_state=1907,max_iter=3000);m.fit(x,ys)
        result[typ]=(v,m)
        print('fit',typ,x.shape,'classes',m.classes_,flush=True)
    return result

def scores(model,r,cross=True):
    n=len(r['values']);out=[]
    for typ in ['unary','pair']:
        v,m=model[typ]
        seq=[(a,None) for a in range(n)] if typ=='unary' else [(a,b) for a,b in itertools.combinations(range(n),2)]
        if not seq:out.append(np.zeros((0,7)));continue
        x=v.transform([ff(r['ts'],n,a,b,cross) for a,b in seq]);s=m.decision_function(x)
        if len(s.shape)==1:s=np.stack([-s,s],axis=1)
        z=np.full((len(seq),2 if typ=='unary' else len(OPS)),-50.0)
        z[:,m.classes_]=s
        out.append(z)
    return out

def decode(r,ss,w=1.):
    n=len(r['values']);p,used,rel=pool(n);a,b=ss
    score=w*a[np.arange(n)[None,:],used].sum(axis=1)
    if b.size:score+=b[np.arange(b.shape[0])[None,:],rel].sum(axis=1)
    for k in np.argsort(-score,kind='stable'):
        value=numeric(p[k],r['values'])
        if math.isfinite(value):return value,p[k]
    return math.nan,None

def chart(r,ss,w=1.,kbest=3,max_used=4):
    n=len(r['values']);u,pair=ss;sub={};rels={}
    for j,(a,b) in enumerate(itertools.combinations(range(n),2)):rels[a,b]=pair[j]
    for mask in range(1,1<<n):
        count=mask.bit_count()
        if count>max_used:continue
        if count==1:
            i=mask.bit_length()-1;sub[mask]={'leaf':[(0.,i)]};continue
        buckets={}
        a=(mask-1)&mask
        while a:
            b=mask^a
            if b and a in sub and b in sub:
                for op in '+-*/':
                    if op in '+*' and a>b:continue
                    cc=0.
                    for i in range(n):
                        if not (a>>i&1):continue
                        for j in range(n):
                            if b>>j&1:
                                label=op if i<j else REV[op]
                                cc+=rels[min(i,j),max(i,j)][OPS.index(label)]
                    banned='-' if op in '+-' else '/'
                    for ra,items_a in sub[a].items():
                        if ra==banned:continue
                        for rb,items_b in sub[b].items():
                            if rb==banned:continue
                            for sa,ea in items_a:
                                for sb,eb in items_b:
                                    e=norm((op,ea,eb))
                                    val=sa+sb+cc
                                    buckets.setdefault(op,{})[e]=val
            a=(a-1)&mask
        sub[mask]={op:sorted([(v,e) for e,v in d.items()],key=lambda x:(-x[0],repr(x[1])))[:kbest] for op,d in buckets.items()}
    result=[]
    for mask,groups in sub.items():
        side=w*sum(u[i,int(mask>>i&1)] for i in range(n))
        for (a,b),q in rels.items():
            if not (mask>>a&1 and mask>>b&1):side+=q[6]
        for vv in groups.values():
            for score,e in vv:
                val=numeric(e,r['values'])
                if math.isfinite(val):result.append((side+score,e,val))
    result.sort(key=lambda x:(-x[0],repr(x[1])))
    return result


def expr(e):
    return {'Input':e} if isinstance(e,int) else {'Bin':[{'+':'Add','-':'Sub','*':'Mul','/':'DivExact'}[e[0]],expr(e[1]),expr(e[2])]}


def export_factor(v,m,k):
    names=sorted(v.vocabulary_);order=[v.vocabulary_[w] for w in names]
    coef=np.zeros((k,len(names)));intercept=np.full(k,-50.0)
    raw=m.coef_[:,order]
    if k==2 and raw.shape[0]==1:
        coef[0]=-raw[0];coef[1]=raw[0];intercept[:]=[-m.intercept_[0],m.intercept_[0]]
    else:
        coef[m.classes_]=raw;intercept[m.classes_]=m.intercept_
    scale=np.maximum(np.max(np.abs(coef),axis=1)/32760.0,1e-12)
    quant=np.clip(np.rint(coef/scale[:,None]),-32760,32760).astype(np.int16)
    return {'vocabulary':names,'idf':v.idf_[order].tolist(),'intercept':intercept.tolist(),
            'scales':scale.tolist(),'weights':quant.T.astype('>i2').tobytes().hex()}


def quantized_score(factor,counts):
    # Same feature ordering and f64 arithmetic order as the Rust loop.
    ids={w:i for i,w in enumerate(factor['vocabulary'])};k=len(factor['intercept'])
    weights=np.frombuffer(bytes.fromhex(factor['weights']),dtype='>i2').reshape(-1,k)
    x=[];ss=0.0
    for word,count in sorted(counts.items()):
        i=ids.get(word)
        if i is not None:
            value=(1.0+math.log(count))*factor['idf'][i];ss+=value*value;x.append((i,value))
    out=list(factor['intercept'])
    if ss:
        den=math.sqrt(ss)
        for i,value in x:
            for j in range(k):out[j]+=(value/den)*int(weights[i,j])*factor['scales'][j]
    return out


def whole_fit(rows,indices,C):
    usable=[i for i in indices if rows[i]['tree'] is not None]
    bank=sorted(set(rows[i]['tree'] for i in usable),key=repr);ids={e:i for i,e in enumerate(bank)}
    y=np.array([ids[rows[i]['tree']] for i in usable])
    v=TfidfVectorizer(token_pattern=r'\S+',lowercase=False,sublinear_tf=True,min_df=2,max_features=40000)
    x=v.fit_transform([prev.features(' '.join(rows[i]['ts']),len(rows[i]['values']),'role') for i in usable])
    m=LinearSVC(C=C,dual='auto',random_state=1907,max_iter=3000);m.fit(x,y)
    return v,m,bank,usable,y


def whole_answers(model,rows,ids):
    v,m,bank,_,_=model
    x=v.transform([prev.features(' '.join(rows[i]['ts']),len(rows[i]['values']),'role') for i in ids])
    ss=m.decision_function(x);out=[]
    for s,i in zip(ss,ids):
        value=math.nan
        for j in np.argsort(-s,kind='stable'):
            e=bank[m.classes_[j]]
            if max(leaves(e))>=len(rows[i]['values']):continue
            value=numeric(e,rows[i]['values'])
            if math.isfinite(value):break
        out.append(value)
    return out


def main():
    import argparse
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--train',type=Path,required=True);ap.add_argument('--output',type=Path,required=True)
    args=ap.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    rows=load(args.train);groups=[' '.join(r['ts']) for r in rows]
    tr,dv=next(GroupShuffleSplit(n_splits=1,test_size=.2,random_state=2701).split(rows,groups=groups))
    source=hashlib.sha256(args.train.read_bytes()).hexdigest()
    report={'stage':'TRAINING_ONLY','source_sha256':source,'records':len(rows),'supported':sum(r['tree'] is not None for r in rows),
            'supervision':'training_equations_not_answer_only','test_read':False,'llm_calls':0,'pretrained_weights':False,
            'unsupported':dict(collections.Counter(r['unsupported_reason'] for r in rows if r['tree'] is None)),
            'split':{'seed':2701,'train':tr.tolist(),'dev':dv.tolist(),'dev_total':len(dv)},'trials':[],'whole_trials':[]}
    best=None
    for C in [.2,1.,4.]:
        m=fit(rows,tr,C);cached=[]
        for i in dv:
            r=rows[i]
            cached.append(scores(m,r) if 1<=len(r['values'])<=6 else None)
        for weight in [.5,1.,2.]:
            good=0
            for i,s in zip(dv,cached):
                if s is None:continue
                result=chart(rows[i],s,w=weight)
                if result:good+=abs(result[0][2]-rows[i]['answer'])<1e-5
            row={'C':C,'relevance_weight':weight,'correct':int(good),'total':len(dv)};report['trials'].append(row);print('development',row,flush=True)
            key=(good,-C,-weight)
            if best is None or key>best[0]:best=(key,C,weight)
    best_whole=None
    for C in [.2,1.,4.]:
        fitted=whole_fit(rows,tr,C)
        values=whole_answers(fitted,rows,dv)
        good=sum(abs(value-rows[i]['answer'])<1e-5 for value,i in zip(values,dv))
        report['whole_trials'].append({'C':C,'correct':good,'total':len(dv)})
        if best_whole is None or (good,-C)>best_whole[0]:best_whole=((good,-C),C)
    _,C,weight=best;m=fit(rows,range(len(rows)),C)
    model={'version':1,'source_sha256':source,'supervision':'training_derivations','training_examples':len(rows),
           'supported_examples':report['supported'],'relevance_weight':weight,'max_used':4,'beam_per_root':3,
           'unary':export_factor(*m['unary'],2),'pair':export_factor(*m['pair'],7)}
    mp=args.output/'relational-model.json';mp.write_text(json.dumps(model,separators=(',',':'))+'\n',encoding='utf-8')
    v,clf,bank,usable,y=whole_fit(rows,range(len(rows)),best_whole[1])
    factor=export_factor(v,clf,len(clf.classes_));counts=collections.Counter(map(int,y))
    whole={'version':2,'feature_kind':'role','source_sha256':source,'training_examples':len(rows),
           'supported_training_examples':len(usable),'programs':[{'expression':expr(bank[i]),'support':counts[int(i)]} for i in clf.classes_],
           'developer_margin':1.0,**factor}
    wp=args.output/'whole-derivation-model.json';wp.write_text(json.dumps(whole,separators=(',',':'))+'\n',encoding='utf-8')
    report['chosen']={'C':C,'relevance_weight':weight,'beam_per_root':3,'max_used':4}
    report['whole_chosen']={'C':best_whole[1],'programs':len(whole['programs'])}
    report['model_sha256']=hashlib.sha256(mp.read_bytes()).hexdigest();report['whole_model_sha256']=hashlib.sha256(wp.read_bytes()).hexdigest()
    (args.output/'training-report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    # Fixtures selected from training only; exact feature/quantized-score parity.
    fixtures=[]
    for r in rows[::79]:
        n=len(r['values'])
        if not 1<=n<=6:continue
        for typ,seq in [('unary',[(a,None) for a in range(n)]),('pair',list(itertools.combinations(range(n),2)))]:
            for a,b in seq:
                f=dict(collections.Counter(ff(r['ts'],n,a,b).split()))
                fixtures.append({'question':r['text'],'a':a,'b':b,'features':f,'scores':quantized_score(model[typ],f)})
    (args.output/'parity.json').write_text(json.dumps(fixtures)+'\n',encoding='utf-8')
    # Training teacher preserved as provenance; never an inference lookup table.
    teacher=[{'question':r['text'],'answer':r['answer'],'tree':expr(r['tree']) if r['tree'] is not None else None,'unsupported':r['unsupported_reason']} for r in rows]
    (args.output/'teacher.json').write_text(json.dumps(teacher)+'\n',encoding='utf-8')
    print(json.dumps({k:v for k,v in report.items() if k!='split'},indent=2),flush=True)

if __name__=='__main__':main()
