"""Compare actual first responses of two complete continuing organisms.

No model is trained or selected here. This reads completed immutable traces and
the advance criterion registered before the continuation experiment.
"""
from __future__ import annotations
import argparse,gzip,hashlib,json
from pathlib import Path
from run_external_continual5 import motors


def responses(folder):
    result={};context=None
    for name in ('life-before-resume.jsonl.gz','life-after-resume.jsonl.gz'):
        with gzip.open(folder/name,'rt') as stream:
            for line in stream:
                row=json.loads(line)
                if row['kind']=='restart':
                    meta=row['metadata']
                    context=None if 'game' in meta else (meta['phase'],meta.get('source_offset'),meta['index'])
                elif row['kind']=='actual_post' and context is not None and context not in result:
                    _,answers=motors(context[0])
                    if row['action'] in answers:
                        result[context]=(row['action'],row['reward']>0)
    return result


def compare(root,protocol):
    root=Path(root);protocol=json.loads(Path(protocol).read_text())
    summaries=json.loads((root/'summary.json').read_text())
    learned=summaries['organism'];fixed=summaries['organism_rules_fixed']
    a=responses(root/'organism');b=responses(root/'organism_rules_fixed')
    assert a.keys()==b.keys(),'different sensory tasks or incomplete lives'
    changed=sum(a[k][0]!=b[k][0] for k in a)
    wins=sum(a[k][1] and not b[k][1] for k in a)
    losses=sum(b[k][1] and not a[k][1] for k in a)
    sensor_gain=(wins-losses)/len(a)
    phases={};max_drop=0;navigation_a=navigation_b=0
    assert len(learned['phases'])==len(fixed['phases'])
    for x,y in zip(learned['phases'],fixed['phases']):
        assert x['phase']==y['phase']
        key=x['phase']
        x=x['result'];y=y['result']
        if 'first_answer_prequential' in x:
            gain=x['first_answer_prequential']['accuracy']-y['first_answer_prequential']['accuracy']
            max_drop=max(max_drop,-gain)
            phases[key]={'organism':x['first_answer_prequential'],'fixed':y['first_answer_prequential'],'accuracy_gain':gain}
        else:
            navigation_a+=x['successes'];navigation_b+=y['successes']
    actions_a=sum(p['result']['actual_actions'] for p in learned['phases'])
    actions_b=sum(p['result']['actual_actions'] for p in fixed['phases'])
    report=learned['final_report'];rules=report['reward_memory']['acquired_rules']
    assert fixed['final_report']['reward_memory']['acquired_rules'] is None
    criterion=protocol['advance_criterion']
    checks={
        'at_least_one_program_admitted':rules['admitted_programs']>0,
        'changed_first_answers':changed>0,
        'pooled_sensor_accuracy_gain':sensor_gain+1e-12>=criterion['pooled_sensor_accuracy_gain_min'],
        'no_excess_per_phase_drop':max_drop<=criterion['per_sensor_phase_accuracy_drop_max']+1e-12,
        'no_navigation_regression':navigation_b-navigation_a<=criterion['navigation_success_drop_max'],
        'actual_action_budget':actions_a/actions_b<=criterion['actual_action_ratio_max'],
    }
    result={
        'purpose':'whole-lifetime behavioral comparison, distinct from internal predictive rule admission',
        'advance_criterion_passed':all(checks.values()),'criterion_checks':checks,
        'sensor_tasks':len(a),'organism_correct':sum(v[1] for v in a.values()),
        'fixed_correct':sum(v[1] for v in b.values()),'accuracy_gain':sensor_gain,
        'changed_first_answer_actions':changed,'paired_wins':wins,'paired_losses':losses,
        'navigation_successes':{'organism':navigation_a,'fixed':navigation_b},
        'actual_actions':{'organism':actions_a,'fixed':actions_b},
        'final_factual_events':{'organism':report['reward_memory']['factual_events'],
            'fixed':fixed['final_report']['reward_memory']['factual_events']},
        'final_computation_work':{'organism':report['organism']['computation_work'],
            'fixed':fixed['final_report']['organism']['computation_work']},
        'acquired_programs':rules,'sensor_phases':phases,
        'summary_sha256':hashlib.sha256((root/'summary.json').read_bytes()).hexdigest(),
        'interpretation':protocol['interpretation'],
    }
    return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--results',required=True);parser.add_argument('--protocol',required=True)
    parser.add_argument('--output',required=True);args=parser.parse_args()
    result=compare(args.results,args.protocol)
    Path(args.output).write_text(json.dumps(result,indent=2,allow_nan=False)+'\n')
    print(json.dumps({k:v for k,v in result.items() if k not in ('acquired_programs','sensor_phases')},indent=2))
