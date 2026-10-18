#!/usr/bin/env python3
"""Read-only post-exam reference diagnostics. Never imported by the learner."""
import argparse,collections,json,sys
from pathlib import Path
from fractions import Fraction
from word_problem_verify import quantities,gold_tree,equal_program
from relational_external import scalar_answer
from word_problem_external import correct,numberless,sha,dump

def main():
    p=argparse.ArgumentParser();p.add_argument('folder',type=Path);a=p.parse_args();root=a.folder
    summary=json.loads((root/'summary.json').read_text());result={}
    for dataset,name in [('SVAMP-regression','SVAMP-original.json'),('ASDiv-complement','ASDiv-complement.json')]:
        rows=json.loads((root/name).read_text());arms={};golds=[];errors=[]
        for i,r in enumerate(rows):
            text=r['Body'].strip()+' '+r['Question'].strip();values=quantities(text)
            expression=r['Equation'].replace('×','*').replace('÷','/')
            try:
                if dataset=='ASDiv-complement':
                    # A quotient with remainder, a time, or several equations
                    # is not equivalent to its first arithmetic subexpression.
                    parts=expression.split('=')
                    if len(parts)!=2:raise ValueError('not one scalar equation')
                    rhs=Fraction(parts[1].strip());answer=scalar_answer(r['Answer_raw'])
                    if answer is None or rhs!=Fraction(answer):raise ValueError('reference requires extra semantics')
                    expression=parts[0]
                golds.append(gold_tree(expression,values))
            except (ValueError,KeyError,SyntaxError,TypeError):golds.append(None);errors.append(i)
        for arm in ['relations','whole_derivations','previous_ewp1']:
            path=root/dataset/(arm+'.jsonl');assert sha(path)==summary[dataset]['arms'][arm]['predictions_sha256']
            entries=[json.loads(s) for s in path.read_text().splitlines()];same=[];value_only=[]
            for i,(r,g,entry) in enumerate(zip(rows,golds,entries)):
                assert entry['index']==i
                if g is None:continue
                n=len(quantities(entry['question']));candidates=entry['prediction'].get('interpretations',[])
                equal=bool(candidates) and equal_program(candidates[0]['program'],g,n)
                if equal:same.append(i)
                expected=str(r['Answer']) if dataset=='SVAMP-regression' else scalar_answer(r['Answer_raw'])
                if expected is not None and correct(entry['prediction'].get('answer'),expected) and not equal:value_only.append(i)
            arms[arm]={'symbolic_correct':len(same),'bindable_reference_count':len(rows)-len(errors),'numeric_matches_without_equivalence':len(value_only),'symbolically_correct_indices':same,'numeric_only_indices':value_only}
        groups=collections.defaultdict(list)
        for i,r in enumerate(rows):groups[numberless(r['Body'])].append(i)
        controls=[]
        for ids in groups.values():
            if len(ids)<2 or any(golds[i] is None for i in ids):continue
            if len({numberless(rows[i]['Question']) for i in ids})<2:continue
            # Distinct structural expressions, not merely distinct numeric answers.
            if len({json.dumps(golds[i],sort_keys=True) for i in ids})<2:continue
            controls.append({'indices':ids,'all_relational_correct':all(i in arms['relations']['symbolically_correct_indices'] for i in ids),'all_previous_correct':all(i in arms['previous_ewp1']['symbolically_correct_indices'] for i in ids)})
        result[dataset]={'arms':arms,'unbound_reference_indices':errors,'question_dependent_groups':controls,'all_group_relational_correct':sum(g['all_relational_correct'] for g in controls),'all_group_previous_correct':sum(g['all_previous_correct'] for g in controls)}
    result['status']='POSTHOC_REFERENCE_DIAGNOSTIC_NOT_SOLVER_INPUT'
    result['caveat']='Numeric reference literals are associated with unique equal-valued inputs; ambiguous bindings excluded only from this symbolic diagnostic, never primary accuracy. No result establishes general language understanding.'
    dump(root/'posthoc-symbolic.json',result)
    print(json.dumps({k:{'arms':{n:{z:v for z,v in t.items() if not z.endswith('indices')} for n,t in d['arms'].items()},'question_groups':len(d['question_dependent_groups']),'all_group_relational_correct':d['all_group_relational_correct'],'all_group_previous_correct':d['all_group_previous_correct']} for k,d in result.items() if isinstance(d,dict)},indent=2))
if __name__=='__main__':main()
