#!/usr/bin/env python3
"""Independent recount and VM replay. No teaching or selection of test items."""
from __future__ import annotations
import argparse, json, hashlib
from pathlib import Path

def verify(root:Path):
    summary=json.loads((root/'summary.json').read_text(encoding='utf-8'))
    raw=(root/'learned-semantic.json').read_bytes();frozen=hashlib.sha256(raw).hexdigest()
    assert frozen==summary['checkpoint_sha256']
    state=json.loads(raw);school=state['cognition']['education'];lib={f['id']:f for f in school['library']['functions']}
    assert school['frozen'] and school['composition']
    # Independent exact integer evaluator, including truncating Rust remainder.
    def evaluate(e,inputs,args=(),depth=0):
        assert depth<128
        tag,value=next(iter(e.items()))
        if tag=='Input':return inputs[value]
        if tag=='Arg':return args[value]
        if tag=='Lit':return next(iter(value.values()))
        if tag=='Call':
            ident,terms=value;f=lib[ident];assert len(terms)==f['arity']
            vals=[evaluate(x,inputs,args,depth+1) for x in terms]
            return evaluate(f['body'],inputs,vals,depth+1)
        if tag=='AsInt':return int(evaluate(value,inputs,args,depth+1))
        if tag=='Not':return not evaluate(value,inputs,args,depth+1)
        if tag=='If':return evaluate(value[1] if evaluate(value[0],inputs,args,depth+1) else value[2],inputs,args,depth+1)
        assert tag=='Bin',tag
        op,a,b=value;a=evaluate(a,inputs,args,depth+1);b=evaluate(b,inputs,args,depth+1)
        if op=='Add':result=a+b
        elif op=='Sub':result=a-b
        elif op=='Mul':result=a*b
        elif op=='DivExact':assert b and a%b==0;result=a//b
        elif op=='Rem':
            assert b;quot=(abs(a)//abs(b))*(-1 if (a<0)!=(b<0) else 1);result=a-quot*b
        elif op=='Xor':result=a^b
        elif op=='Eq':return a==b
        elif op=='Lt':return a<b
        elif op=='And':return bool(a and b)
        elif op=='Or':return bool(a or b)
        else:raise AssertionError(op)
        assert -(1<<63)<=result<(1<<63)
        return result
    counts={};replayed=0;seen=set();training={r['question'].lower() for group in json.loads((root/'teacher.json').read_text(encoding='utf-8')) for r in group}
    for line in (root/'exam.jsonl').read_text(encoding='utf-8').splitlines():
        row=json.loads(line);q=row['question'].lower();assert q not in training and q not in seen;seen.add(q)
        for name in ['full','same_knowledge_surface','cold']:
            answer=row[name];value=answer['answer'];expected=row['expected']
            c=counts.setdefault(row['category'],{}).setdefault(name,dict(total=0,correct=0,answered=0,wrong_answered=0))
            c['total']+=1;c['correct']+=value==expected;c['answered']+=value is not None;c['wrong_answered']+=value is not None and value!=expected
            if answer['derivation'] is not None and value is not None:
                assert str(evaluate(answer['derivation'],answer['bound_numbers']))==value;replayed+=1
    assert counts==summary['counts']
    clarification=json.loads((root/'clarification.json').read_text(encoding='utf-8'));resolved=0
    for row in clarification:
        a=row['result'];assert str(evaluate(a['derivation'],a['bound_numbers']))==a['answer'];resolved+=a['answer']==row['expected']
        assert row['teacher_reply'] in row['probe']['predicted_answers']
    return dict(records=len(seen),program_results_independently_replayed=replayed,clarifications_correct=resolved,
        clarifications_total=len(clarification),checkpoint_sha256=frozen,counts_match=True,counts=counts)

def main():
    ap=argparse.ArgumentParser();ap.add_argument('root',type=Path);ap.add_argument('--binary',type=Path);a=ap.parse_args();report=verify(a.root)
    if a.binary:
        from education_experiment import Client
        client=Client(a.binary,a.root/'verify-process.log')
        checkpoint=(a.root/'learned-semantic.json').read_text(encoding='utf-8')
        try:
            client.send('load',checkpoint=checkpoint)
            for row in json.loads((a.root/'examples.json').read_text(encoding='utf-8')):
                got=client.send('ask',question=row['question'])['result']
                assert got['answer']==row['answer']['answer'] and got['reason']==row['answer']['reason']
            assert checkpoint==client.send('checkpoint')['checkpoint'];report['separate_process_restored']=True
        finally:client.close()
    print(json.dumps(report,ensure_ascii=False,indent=2))
if __name__=='__main__':main()
