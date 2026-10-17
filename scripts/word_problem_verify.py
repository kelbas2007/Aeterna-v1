#!/usr/bin/env python3
"""Independent recount and exact rational-program replay. No model fitting."""
import argparse,ast,hashlib,json,re
from pathlib import Path
from fractions import Fraction as F

def run(e,x):
    if 'Input' in e:return x[e['Input']]
    op,a,b=e['Bin'];a,b=run(a,x),run(b,x)
    if op=='Add':return a+b
    if op=='Sub':return a-b
    if op=='Mul':return a*b
    if op=='DivExact':return a/b
    raise ValueError('unapproved arithmetic')
def ok(a,g):
    if a is None:return False
    return abs(float(F(a))-float(g))<=1e-6*max(1,abs(float(g)))
def quantities(s):
    return [F(m.group(0).replace(',','')) for m in re.finditer(r'(?<![A-Za-z0-9])[-+]?(?:\d{1,3}(?:,\d{3})+|\d+)(?:\.\d+)?',s)]

def polyadd(a,b,sign=1):
    out=dict(a)
    for k,v in b.items():out[k]=out.get(k,F(0))+sign*v
    return {k:v for k,v in out.items() if v}
def polymul(a,b):
    out={}
    for k,v in a.items():
        for j,w in b.items():
            t=tuple(x+y for x,y in zip(k,j));out[t]=out.get(t,F(0))+v*w
    return {k:v for k,v in out.items() if v}
def symbolic(e,n):
    one={(0,)*n:F(1)}
    if 'Input' in e:
        t=[0]*n;t[e['Input']]=1;return {tuple(t):F(1)},one
    if 'Constant' in e:return {(0,)*n:F(e['Constant'])},one
    op,a,b=e['Bin'];an,ad=symbolic(a,n);bn,bd=symbolic(b,n)
    if op in ['Add','Sub']:return polyadd(polymul(an,bd),polymul(bn,ad),1 if op=='Add' else -1),polymul(ad,bd)
    if op=='Mul':return polymul(an,bn),polymul(ad,bd)
    if op=='DivExact':return polymul(an,bd),polymul(ad,bn)
    raise ValueError(op)
def equal_program(a,b,n):
    an,ad=symbolic(a,n);bn,bd=symbolic(b,n)
    return bool(ad) and bool(bd) and not polyadd(polymul(an,bd),polymul(bn,ad),-1)
def gold_tree(text,values):
    def read(node):
        if isinstance(node,ast.Constant):
            v=F(str(node.value));ids=[i for i,x in enumerate(values) if x==v]
            if len(ids)>1:raise ValueError('ambiguous repeated numeric value')
            return {'Input':ids[0]} if ids else {'Constant':str(v)}
        if isinstance(node,ast.BinOp):
            op={ast.Add:'Add',ast.Sub:'Sub',ast.Mult:'Mul',ast.Div:'DivExact'}[type(node.op)]
            return {'Bin':[op,read(node.left),read(node.right)]}
        if isinstance(node,ast.UnaryOp) and isinstance(node.op,ast.USub):
            return {'Bin':['Sub',{'Constant':'0'},read(node.operand)]}
        raise ValueError('unsupported reference equation')
    return read(ast.parse(text.strip(),mode='eval').body)

def main():
    ap=argparse.ArgumentParser();ap.add_argument('folder',type=Path);a=ap.parse_args();o=a.folder
    rows=json.loads((o/'SVAMP-original.json').read_text());summary=json.loads((o/'external-summary.json').read_text())
    assert len(rows)==summary['test_records']==1000
    assert hashlib.sha256((o/'SVAMP-original.json').read_bytes()).hexdigest()==summary['test_sha256']
    arms={};replayed=0;symbolic_total=0;symbolic_correct=0;value_only=0
    for arm,expected in summary['arms'].items():
        path=o/(arm+'-predictions.jsonl');entries=[json.loads(s) for s in path.read_text().splitlines()]
        assert len(entries)==1000
        assert hashlib.sha256(path.read_bytes()).hexdigest()==summary['traces'][arm]['predictions_sha256']
        correct=0;errors=[]
        for i,(r,p) in enumerate(zip(rows,entries)):
            assert p['index']==i
            q=r['Body'] if arm=='body_only' else r['Body'].strip()+' '+r['Question'].strip()
            assert p['question']==q
            pred=p['prediction'];good=ok(pred.get('answer'),r['Answer']);correct+=good
            if not good:errors.append(i)
            if arm!='previous' and pred.get('answer') is not None:
                nums=list(map(F,pred['numbers']));assert nums==quantities(q)
                assert pred['answer']==pred['interpretations'][0]['answer']
                for hypothesis in pred['interpretations']:
                    assert run(hypothesis['program'],nums)==F(hypothesis['answer']);replayed+=1
                if arm=='full':
                    try:g=gold_tree(r['Equation'],nums)
                    except (ValueError,KeyError,SyntaxError):continue
                    symbolic_total+=1
                    same=equal_program(pred['interpretations'][0]['program'],g,len(nums));symbolic_correct+=same
                    value_only+=good and not same
        assert correct==expected['correct']
        arms[arm]={'correct':correct,'total':1000,'wrong_indices':errors}
    result={'verified':True,'native_candidate_programs_replayed':replayed,'arms':arms,
        'full_symbolic_equivalence':{'correct':symbolic_correct,'bindable_reference_cases':symbolic_total,'numeric_matches_without_equivalence':value_only,
        'caveat':'reference numeric constants mapped only to uniquely matching input values; symbolic equality does not establish unrestricted language understanding'}}
    (o/'independent-recount.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({**result,'arms':{k:{kk:vv for kk,vv in v.items() if kk!='wrong_indices'} for k,v in arms.items()}},indent=2))
if __name__=='__main__':main()
