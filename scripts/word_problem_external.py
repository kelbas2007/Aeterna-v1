#!/usr/bin/env python3
"""Frozen external examination. Child AETERNA receives QUESTIONS ONLY.

First collect predictions for every arm. Only then inspect labels to score.
Test URLs are fetched only AFTER saving the code/model freeze manifest.
"""
from __future__ import annotations
import argparse,ast,csv,hashlib,json,math,re,subprocess,sys,time,urllib.request
from fractions import Fraction
from pathlib import Path
from collections import defaultdict
REF='78e727689e1c1bebfc4be39c446898e8e10b0518'
URL='https://raw.githubusercontent.com/arkilpatel/SVAMP/'+REF+'/SVAMP.json'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def dump(path,value):path.write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
class Client:
    def __init__(self,binary,log):
        self.err=log.open('w',encoding='utf-8');self.p=subprocess.Popen([str(binary)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=self.err,text=True,encoding='utf-8',bufsize=1)
    def request(self,obj):
        self.p.stdin.write(json.dumps(obj,ensure_ascii=False,separators=(',',':'))+'\n');self.p.stdin.flush()
        line=self.p.stdout.readline()
        if not line:raise RuntimeError('native process closed unexpectedly')
        r=json.loads(line)
        if r.get('type')=='error':raise RuntimeError(r)
        return r
    def close(self):
        self.request({'op':'quit'});self.p.wait(timeout=10);self.err.close()
        if self.p.returncode:raise RuntimeError('native nonzero exit')
def install(binary,model,prior,path,log):
    c=Client(binary,log)
    if prior:
        c.request({'op':'load','checkpoint':prior.read_text(encoding='utf-8')})
        before=c.request({'op':'checkpoint'})['checkpoint'];before=json.loads(before)
        c.request({'op':'begin_education_extension'})
    else:
        c.request({'op':'init'});before=json.loads(c.request({'op':'checkpoint'})['checkpoint'])
    c.request({'op':'install_word_problems','model':json.loads(model.read_text())});c.request({'op':'freeze'})
    checkpoint=c.request({'op':'checkpoint'})['checkpoint'];after=json.loads(checkpoint)
    # Appending an acquired interpretation model must not reset existing knowledge.
    for name in ['library','skills','facts','lessons']:
        old=(before['cognition'].get('education') or {}).get(name)
        new=(after['cognition'].get('education') or {}).get(name)
        if old is not None and old!=new:raise AssertionError('old school knowledge mutated: '+name)
    if prior and before['cognition']['current']['learner']!=after['cognition']['current']['learner']:raise AssertionError('world cognition changed')
    bcar=json.loads(bytes(before['carrier']));acar=json.loads(bytes(after['carrier']))
    if bcar['synapses']!=acar['synapses']:raise AssertionError('physical weights changed')
    path.write_text(checkpoint,encoding='utf-8');c.close()
    return {'checkpoint_sha256':sha(path),'bytes':path.stat().st_size,'old_knowledge_preserved':True,'world_cognition_preserved':True,'physical_weights_preserved':True}
def parity(binary,model_path,checkpoint,fixtures_path,log):
    c=Client(binary,log);c.request({'op':'load','checkpoint':checkpoint.read_text()})
    m=json.loads(model_path.read_text());fixtures=json.loads(fixtures_path.read_text());checks=0
    index={json.dumps(p['expression'],sort_keys=True,separators=(',',':')):i for i,p in enumerate(m['programs'])}
    for f in fixtures:
        i=c.request({'op':'inspect_word_problem','question':f['question'],'kind':m['feature_kind']})
        assert i['features']==f['counts'],('feature mismatch',f['question'])
        assert i['ground']['tokens']==f['tokens'],('token mismatch',f['question'])
        a=c.request({'op':'ask_word_problem','question':f['question']})['result']
        for p in a['interpretations']:
            k=index[json.dumps(p['program'],sort_keys=True,separators=(',',':'))]
            assert abs(p['score']-f['scores'][k])<1e-8,(p['score'],f['scores'][k])
            checks+=1
    c.close();return {'training_fixture_questions':len(fixtures),'matched_native_candidate_scores':checks}
def numberless(s):
    s=re.sub(r'number\d+',' @n@ ',s.lower())
    s=re.sub(r'\d+(?:,\d{3})*(?:\.\d+)?',' @n@ ',s)
    return ' '.join(re.findall(r'[a-z]+',s))
def collect(binary,checkpoint,rows,path,log,method='full'):
    c=Client(binary,log);c.request({'op':'load','checkpoint':checkpoint.read_text()})
    before=hashlib.sha256(c.request({'op':'checkpoint'})['checkpoint'].encode()).hexdigest()
    start=time.perf_counter()
    with path.open('w',encoding='utf-8') as f:
        for i,row in enumerate(rows):
            # The only per-case data passed across the process boundary:
            q=row['Body'] if method=='body_only' else row['Body'].strip()+' '+row['Question'].strip()
            obj={'op':'ask_word_problem','question':q,'prior_only':method=='prior'}
            if method=='previous':obj={'op':'ask','question':q,'mode':'ComputeOnly'}
            a=c.request(obj)['result']
            f.write(json.dumps({'index':i,'question':q,'prediction':a},ensure_ascii=False)+'\n')
    after=hashlib.sha256(c.request({'op':'checkpoint'})['checkpoint'].encode()).hexdigest();c.close()
    assert before==after,'examination mutated acquired state'
    return {'records':len(rows),'checkpoint_unchanged':True,'seconds':time.perf_counter()-start,'predictions_sha256':sha(path)}
def correct(answer,gold):
    if answer is None:return False
    try:return abs(float(Fraction(answer))-float(gold))<=1e-6*max(1,abs(float(gold)))
    except (ValueError,ZeroDivisionError,TypeError):return False

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--binary',type=Path,required=True);ap.add_argument('--learned',type=Path,required=True)
    ap.add_argument('--prior',type=Path);ap.add_argument('--output',type=Path,required=True);ap.add_argument('--test',type=Path)
    ap.add_argument('--prepare-only',action='store_true');a=ap.parse_args();a.binary=a.binary.resolve();o=a.output;o.mkdir(parents=True,exist_ok=True)
    preserved={};parities={}
    for kind in ['role','bag']:
        cp=o/(kind+'-checkpoint.json')
        preserved[kind]=install(a.binary,a.learned/(kind+'-model.json'),a.prior,cp,o/(kind+'-install.log'))
        parities[kind]=parity(a.binary,a.learned/(kind+'-model.json'),cp,a.learned/(kind+'-parity.json'),o/(kind+'-parity.log'))
    freeze={'stage':'FROZEN_BEFORE_TEST_READ','binary_sha256':sha(a.binary),'learned_models':{k:sha(a.learned/(k+'-model.json')) for k in ['role','bag']},
            'preservation':preserved,'native_parity':parities,'code_revision':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),
            'test_url':URL,'test_fetched_by_this_process':False,'selection':'role C=1, bag C=3; training-grouped dev only; no changes after external outcomes'}
    dump(o/'freeze.json',freeze)
    if a.prepare_only:return
    if a.test:data=a.test.read_bytes()
    else:
        with urllib.request.urlopen(URL,timeout=60) as r:data=r.read()
    rows=json.loads(data);assert len(rows)==1000
    (o/'SVAMP-original.json').write_bytes(data)
    traces={}
    for method,kind in [('full','role'),('bag','bag'),('body_only','role'),('prior','role'),('previous','role')]:
        traces[method]=collect(a.binary,o/(kind+'-checkpoint.json'),rows,o/(method+'-predictions.jsonl'),o/(method+'-process.log'),method)
        print('collected',method,traces[method]['records'],flush=True)
    # Label access starts here, AFTER all predictions and checkpoint hashes exist.
    train=json.loads((a.learned/'teacher.json').read_text());seen={numberless(r['text']) for r in train}
    overlap=[numberless(r['Body']+' '+r['Question']) in seen for r in rows]
    result={'status':'EXTERNAL_SVAMP_COMPLETED','test_records':len(rows),'source_revision':REF,'test_sha256':hashlib.sha256(data).hexdigest(),
            'test_answers_sent_to_solver':False,'exact_normalized_training_overlaps':sum(overlap),'arms':{},'traces':traces,
            'counts_are_numeric_answer_match_not_proof_of_semantic_correctness':True}
    predictions={}
    for method in traces:
        ps=[json.loads(s)['prediction'] for s in (o/(method+'-predictions.jsonl')).read_text().splitlines()]
        predictions[method]=ps;ok=[correct(p.get('answer'),r['Answer']) for p,r in zip(ps,rows)]
        high=[(p.get('margin') or -1e10)>=1 for p in ps]
        result['arms'][method]={'correct':sum(ok),'total':len(rows),'abstentions':sum(p.get('answer') is None for p in ps),
            'nonduplicate_correct':sum(b and not d for b,d in zip(ok,overlap)),'nonduplicate_total':sum(not d for d in overlap),
            'high_margin_correct':sum(b and h for b,h in zip(ok,high)),'high_margin_count':sum(high)}
    groups=defaultdict(list)
    for i,r in enumerate(rows):groups[r['Body'].strip().lower()].append(i)
    pairs=[]
    for ids in groups.values():
        for j,i in enumerate(ids):
            for k in ids[j+1:]:
                if float(rows[i]['Answer'])!=float(rows[k]['Answer']):pairs.append((i,k))
    result['different_question_same_body_pairs']={m:{'both_correct':sum(correct(ps[i].get('answer'),rows[i]['Answer']) and correct(ps[k].get('answer'),rows[k]['Answer']) for i,k in pairs),'total_pairs':len(pairs)} for m,ps in predictions.items()}
    examples=[]
    # Transparent deterministic example selection: first 6 full-correct cases where
    # bag is wrong, plus first 6 full errors; no re-training after this selection.
    for flag in [True,False]:
        n=0
        for i,r in enumerate(rows):
            good=correct(predictions['full'][i].get('answer'),r['Answer'])
            if good!=flag or (flag and correct(predictions['bag'][i].get('answer'),r['Answer'])):continue
            examples.append({'index':i,'body':r['Body'],'question':r['Question'],'expected':r['Answer'],'correct':good,'native':predictions['full'][i]});n+=1
            if n==6:break
    dump(o/'examples.json',examples);dump(o/'external-summary.json',result)
    print(json.dumps(result['arms'],indent=2))
if __name__=='__main__':main()
