#!/usr/bin/env python3
"""Public SyGuS CLIA -> numerical examples -> frozen AETERNA -> independent SMT check.

The teacher/checker sees public formal constraints. The Rust searcher only sees
numerical examples. No reference solution or counterexample feedback is supplied.
This is a bounded-integer PBE adaptation, not an official SyGuS grammar score.
"""
from __future__ import annotations
import argparse, hashlib, json, random, re, subprocess, time
from pathlib import Path
from typing import Any
import z3

BOUND=1_000_000
LIMITS=dict(work=2_000_000,evaluations=8192,max_nodes=15,frontier=2048,edits_per_expansion=24)
EVOLUTION=dict(object=dict(work=100_000,evaluations=384,max_nodes=11,frontier=512,edits_per_expansion=24),meta=dict(work=100_000_000,evaluations=120,max_nodes=23,frontier=256,edits_per_expansion=12),max_lifetime_work=5_000_000_000)
class Unsupported(ValueError): pass

def parse(text:str)->list[Any]:
    if len(text)>262144: raise Unsupported('oversized benchmark')
    tokens=re.findall(r'\(|\)|[^\s()]+',re.sub(r';[^\n]*','',text)); stack=[[]]
    for t in tokens:
        if t=='(':
            if len(stack)>64: raise Unsupported('expression too deep')
            stack.append([])
        elif t==')':
            if len(stack)==1: raise Unsupported('unbalanced close')
            item=stack.pop();stack[-1].append(item)
        else:stack[-1].append(t)
    if len(stack)!=1:raise Unsupported('unbalanced open')
    return stack[0]
def render(e:Any)->str:return '('+' '.join(map(render,e))+')' if isinstance(e,list) else str(e)
def walk(e:Any):
    yield e
    if isinstance(e,list):
        for a in e:yield from walk(a)

def eligible(ast:list[Any])->bool:
    fs=[x for x in ast if isinstance(x,list) and x and x[0]=='synth-fun']
    return len(fs)==1 and len(fs[0])>=4 and fs[0][3]=='Int' and isinstance(fs[0][2],list) and 1<=len(fs[0][2])<=3 and all(len(a)==2 and a[1]=='Int' for a in fs[0][2])

class Case:
    def __init__(self,text:str):
        ast=parse(text)
        if not eligible(ast):raise Unsupported('outside preregistered one-to-three-Int-argument cohort')
        f=next(x for x in ast if x[0]=='synth-fun');self.name=f[1];self.args=f[2]
        self.constraints=[x[1] for x in ast if x[0]=='constraint' and len(x)==2]
        self.variables=[x[1:] for x in ast if x[0]=='declare-var']
        self.helpers=[x for x in ast if x[0]=='define-fun']
        if not self.constraints or any(len(v)!=2 or v[1]!='Int' for v in self.variables):raise Unsupported('unsupported variable or absent constraint')
        calls={tuple(e[1:]) for c in self.constraints for e in walk(c) if isinstance(e,list) and e and e[0]==self.name and all(isinstance(a,str) for a in e[1:])}
        all_calls=[e for c in self.constraints for e in walk(c) if isinstance(e,list) and e and e[0]==self.name]
        if len(calls)!=1 or any(not all(isinstance(a,str) for a in e[1:]) for e in all_calls):raise Unsupported('requires relational or nonuniform function applications')
        self.call=next(iter(calls))
        if len(self.call)!=len(self.args) or len(set(self.call))!=len(self.call) or set(self.call)!={v[0] for v in self.variables}:raise Unsupported('extra globals or non-argument constraints require a richer adapter')
        self.f=z3.Function(self.name,*([z3.IntSort()]*(len(self.args)+1)))
        self.v=[z3.Int(n) for n in self.call]
        self.original=self.asts(None)
    def asts(self,definition:str|None):
        declarations='\n'.join('(declare-const '+v[0]+' '+v[1]+')' for v in self.variables)
        function='(declare-fun '+self.name+' ('+' '.join('Int' for _ in self.args)+') Int)' if definition is None else '(define-fun '+self.name+' '+render(self.args)+' Int '+definition+')'
        text=function+'\n'+declarations+'\n'+'\n'.join(render(x) for x in self.helpers)+'\n'+'\n'.join('(assert '+render(c)+')' for c in self.constraints)
        return list(z3.parse_smt2_string(text))
    def examples(self)->dict[str,Any]:
        # Same sample procedure for every file of the same arity, no filename/ID seed.
        rng=random.Random(0xC11A+len(self.args));points=[tuple([i]*len(self.args)) for i in [0,1,-1]]
        while len(points)<16:
            row=tuple(rng.randint(-15,15) for _ in self.args)
            if row not in points:points.append(row)
        examples=[]
        for row in points:
            s=z3.Solver();s.set(timeout=1000);s.add(*self.original,*[v==a for v,a in zip(self.v,row)])
            status=s.check()
            if status!=z3.sat:raise Unsupported('teacher SMT '+str(status))
            y=s.model().eval(self.f(*[z3.IntVal(v) for v in row]),model_completion=True)
            if not z3.is_int_value(y) or abs(y.as_long())>BOUND:raise Unsupported('teacher output outside bounded scalar interface')
            examples.append(dict(input=list(row),output=y.as_long()))
        return dict(examples=examples,library=dict(functions=[]))
    def check(self,program:dict)->dict:
        # Original constraints checked on all bounded input integers, not only
        # teacher examples. Intermediate overflow and division validity included.
        formal=[z3.Int(a[0]) for a in self.args]
        expression,_=translate(program,formal)
        actual,safe=translate(program,self.v)
        spec=self.asts(expression.sexpr())
        s=z3.Solver();s.set(timeout=3000)
        s.add(*[z3.And(v>=-BOUND,v<=BOUND) for v in self.v])
        s.add(z3.Or(z3.Not(z3.And(*spec)),z3.Not(safe)))
        status=s.check()
        result=dict(status=str(status),verified=status==z3.unsat,input_bound=BOUND,original_constraints=True,overflow_checked=True,official_grammar_checked=False)
        if status==z3.sat:result['counterexample']={str(v):str(s.model().eval(v,model_completion=True)) for v in self.v}
        if status==z3.unknown:result['reason']=s.reason_unknown()
        return result

def translate(e:dict,variables:list,depth:int=0):
    if depth>32 or not isinstance(e,dict) or len(e)!=1:raise Unsupported('invalid candidate AST')
    kind,value=next(iter(e.items()))
    if kind=='Input':return variables[value],z3.BoolVal(True)
    if kind=='Lit':
        if 'Int' in value:return z3.IntVal(value['Int']),z3.BoolVal(True)
        if 'Bool' in value:return z3.BoolVal(value['Bool']),z3.BoolVal(True)
    if kind in ['Not','AsInt']:
        a,sa=translate(value,variables,depth+1)
        return (z3.Not(a) if kind=='Not' else z3.If(a,1,0)),sa
    if kind=='If':
        a,sa=translate(value[0],variables,depth+1);b,sb=translate(value[1],variables,depth+1);c,sc=translate(value[2],variables,depth+1)
        return z3.If(a,b,c),z3.And(sa,z3.If(a,sb,sc))
    if kind=='Bin':
        op,x,y=value;a,sa=translate(x,variables,depth+1);b,sb=translate(y,variables,depth+1);safe=z3.And(sa,sb)
        if op=='Add':out=a+b
        elif op=='Sub':out=a-b
        elif op=='Mul':out=a*b
        elif op=='Xor':out=z3.BV2Int(z3.Int2BV(a,64)^z3.Int2BV(b,64),is_signed=True)
        elif op in ['Rem','DivExact']:
            aa=z3.If(a>=0,a,-a);bb=z3.If(b>=0,b,-b)
            q=z3.If(z3.Xor(a<0,b<0),-(aa/bb),aa/bb)
            safe=z3.And(safe,b!=0,z3.Not(z3.And(a==-(1<<63),b==-1)))
            if op=='DivExact':safe=z3.And(safe,a-q*b==0);out=q
            else:out=a-q*b
        elif op=='Lt':return a<b,safe
        elif op=='Eq':return a==b,safe
        elif op=='And':return z3.And(a,b),safe
        elif op=='Or':return z3.Or(a,b),safe
        else:raise Unsupported('operator '+op)
        return out,z3.And(safe,out>=-(1<<63),out<=(1<<63)-1)
    raise Unsupported('candidate outside pure supported expressions: '+kind)

class Client:
    def __init__(self,binary:Path,log:Path):
        self.log=log.open('w',encoding='utf-8');self.p=subprocess.Popen([str(binary.resolve())],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=self.log,text=True,encoding='utf-8',bufsize=1)
    def request(self,value:dict)->dict:
        self.p.stdin.write(json.dumps(value,ensure_ascii=False)+'\n');self.p.stdin.flush()
        line=self.p.stdout.readline()
        if not line:raise RuntimeError('native process closed')
        result=json.loads(line)
        if result.get('type')=='error':raise RuntimeError(result)
        return result
    def close(self):
        try:self.request(dict(op='quit'))
        except (RuntimeError,BrokenPipeError):pass
        if self.p.stdin:self.p.stdin.close()
        self.p.wait(timeout=5);self.log.close()

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--binary',type=Path,required=True);parser.add_argument('--trained',type=Path,required=True);parser.add_argument('--sources',type=Path,required=True);parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();args.output.mkdir(parents=True,exist_ok=False)
    learned=Client(args.binary,args.output/'learned.native.log');cold=Client(args.binary,args.output/'cold.native.log')
    state=args.trained.read_bytes();learned.request(dict(op='load',checkpoint=state.decode()));cold.request(dict(op='init'));cold.request(dict(op='enable_recursive_search',config=EVOLUTION));cold.request(dict(op='freeze_recursive_search'))
    prior=learned.request(dict(op='recursive_search_report'))['state'];assert prior['frozen']
    counts={a:dict(attempted=0,example_fits=0,verified=0,work=0) for a in ['learned','cold']};rows=[];excluded=[]
    for path in sorted(args.sources.rglob('*.sl')):
        text=path.read_text(encoding='utf-8');digest=hashlib.sha256(text.encode()).hexdigest()
        try:ast=parse(text)
        except Unsupported as e:excluded.append(dict(path=path.name,reason=str(e)));continue
        if not eligible(ast):excluded.append(dict(path=path.name,reason='outside syntax-selected arity/type cohort'));continue
        row=dict(path=path.name,sha256=digest,arms={})
        for c in counts.values():c['attempted']+=1
        try:
            case=Case(text);task=case.examples();row['task']=task
            for arm,client in [('learned',learned),('cold',cold)]:
                try:
                    result=client.request(dict(op='solve_synthesis',task=task,limits=LIMITS,seed=0xC11A))['result']
                    checked=case.check(result['program']);row['arms'][arm]=dict(search=result,check=checked)
                    counts[arm]['example_fits']+=result['solved'];counts[arm]['verified']+=checked['verified'];counts[arm]['work']+=result['work']
                except Exception as e:row['arms'][arm]=dict(error=str(e))
        except Exception as e:row['unsupported']=str(e)
        rows.append(row);print(json.dumps({k:row[k] for k in ['path']})+' '+str({a:counts[a]['verified'] for a in counts}),flush=True)
        (args.output/'cases.json').write_text(json.dumps(rows,indent=2)+'\n',encoding='utf-8')
    after=learned.request(dict(op='recursive_search_report'))['state'];assert prior==after
    native=learned.request(dict(op='report'));learned.close();cold.close()
    report=dict(kind='FROZEN_PUBLIC_SYGUS_PBE_ADAPTATION',official_sygus_score=False,independent_general_intelligence=False,source_tree='b7d6bffdda8b0a00ea841a1c7d0171b4cfcbe465',checkpoint_sha256=hashlib.sha256(state).hexdigest(),z3_version=z3.get_version_string(),limits=LIMITS,syntax_selected=len(rows),outside_cohort=excluded,unsupported_in_main_denominator=sum('unsupported' in r for r in rows),counts=counts,search_policy_unchanged=True,final_runtime=native,domain_bound=BOUND,grammar_conformance_not_claimed=True)
    (args.output/'summary.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(report,indent=2))
if __name__=='__main__':main()
