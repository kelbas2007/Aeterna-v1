#!/usr/bin/env python3
"""Independent arithmetic/recount audit; no training and no model selection."""
from __future__ import annotations
import argparse,hashlib,json,math,re
from fractions import Fraction
from pathlib import Path
from word_problem_verify import run,quantities
from word_problem_external import correct,dump,sha
from relational_external import scalar_answer


def used(e):
    if list(e)==['Input']:return [int(e['Input'])]
    if list(e)==['Bin']:
        op,a,b=e['Bin'];assert op in ['Add','Sub','Mul','DivExact'];return used(a)+used(b)
    raise ValueError('Non-arithmetic executable in evidence')


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('folder',type=Path);args=p.parse_args();root=args.folder
    summary=json.loads((root/'summary.json').read_text(encoding='utf-8'));report={};total_programs=0
    for dataset,name in [('SVAMP-regression','SVAMP-original.json'),('ASDiv-complement','ASDiv-complement.json')]:
        rows=json.loads((root/name).read_text(encoding='utf-8'));expected=[str(r['Answer']) if dataset=='SVAMP-regression' else scalar_answer(r['Answer_raw']) for r in rows]
        checks={}
        for arm,stats in summary[dataset]['arms'].items():
            path=root/dataset/(arm+'.jsonl');assert sha(path)==stats['predictions_sha256']
            records=[json.loads(x) for x in path.read_text(encoding='utf-8').splitlines()];assert len(records)==len(rows)
            good=[];refusals=0
            for i,(record,row,gold) in enumerate(zip(records,rows,expected)):
                text=row['Body'] if arm=='body_only' else row['Body'].strip()+' '+row['Question'].strip()
                assert record['index']==i and record['question']==text
                answer=record['prediction'].get('answer');refusals+=answer is None
                good.append(correct(answer,gold) if gold is not None else False)
                values=quantities(text)
                for candidate in record['prediction'].get('interpretations',[]):
                    actual=run(candidate['program'],values)
                    assert actual==Fraction(candidate['answer']),('arithmetic replay mismatch',i,arm)
                    indices=used(candidate['program']);assert len(indices)==len(set(indices));assert all(0<=j<len(values) for j in indices)
                    total_programs+=1
            assert sum(good)==stats['correct'] and refusals==stats['refusals']
            assert [i for i,v in enumerate(good) if v]==stats['correct_indices']
            checks[arm]={'correct':sum(good),'total':len(rows),'refusals':refusals}
        report[dataset]=checks
    report['programs_replayed']=total_programs;report['program_replay_includes_wrong_semantic_answers']=True
    dump(root/'independent-recount.json',report);print(json.dumps(report,indent=2))

if __name__=='__main__':main()
