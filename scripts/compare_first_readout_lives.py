"""Score predeclared full-organism first-readout comparisons; never fit a model."""
from __future__ import annotations
import argparse,hashlib,json
from pathlib import Path
from compare_organism_lives import responses


def compare(root,protocol):
    root=Path(root);protocol_raw=Path(protocol).read_bytes();protocol=json.loads(protocol_raw)
    provenance=json.loads((root/'provenance.json').read_text())
    assert hashlib.sha256(protocol_raw).hexdigest()==provenance['registered_protocol_sha256']
    summaries=json.loads((root/'summary.json').read_text())
    variants=protocol['variants'];treatment=variants[1]
    assert provenance['variants']==variants
    assert provenance['binary_sha256']==protocol['binary_sha256']
    runs={name:responses(root/name) for name in variants}
    first=runs[treatment]
    assert all(r.keys()==first.keys() for r in runs.values())
    criterion=protocol['advance_criterion'];comparisons={}
    start_computation=None
    import gzip
    raw=gzip.decompress((root/'starting-organism.json.gz').read_bytes())
    assert hashlib.sha256(raw).hexdigest()==protocol['starting_checkpoint_sha256']
    start=json.loads(raw)['cognition'];start_computation=start['computation_work']
    for control in (variants[0],variants[2]):
        a=summaries[treatment];b=summaries[control];other=runs[control]
        wins=sum(first[k][1] and not other[k][1] for k in first)
        losses=sum(other[k][1] and not first[k][1] for k in first)
        phases={};max_drop=0;nav_a=nav_b=0
        for x,y in zip(a['phases'],b['phases'],strict=True):
            assert x['phase']==y['phase'];name=x['phase'];x=x['result'];y=y['result']
            if 'first_answer_prequential' in x:
                gain=x['first_answer_prequential']['accuracy']-y['first_answer_prequential']['accuracy']
                max_drop=max(max_drop,-gain)
                phases[name]={'treatment':x['first_answer_prequential'],'control':y['first_answer_prequential'],
                    'accuracy_gain':gain,'actions':{'treatment':x['actual_actions'],'control':y['actual_actions']}}
            else:nav_a+=x['successes'];nav_b+=y['successes']
        work_a=a['final_report']['organism']['computation_work']-start_computation
        work_b=b['final_report']['organism']['computation_work']-start_computation
        actions_a=sum(x['result']['actual_actions'] for x in a['phases'])
        actions_b=sum(x['result']['actual_actions'] for x in b['phases'])
        changed=sum(first[k][0]!=other[k][0] for k in first)
        checks={'first_accuracy_gain':(wins-losses)/len(first)+1e-12>=criterion['pooled_sensor_accuracy_gain_min'],
            'no_excess_phase_regression':max_drop<=criterion['per_sensor_phase_accuracy_drop_max']+1e-12,
            'no_navigation_regression':nav_b-nav_a<=criterion['navigation_success_drop_max'],
            'external_action_cost':actions_a/max(actions_b,1)<=criterion['actual_action_ratio_max'],
            'computation_cost':work_a/max(work_b,1)<=criterion['computation_ratio_max'],
            'actual_changed_first_actions':changed>0}
        comparisons[control]={'criterion_passed':all(checks.values()),'checks':checks,
            'sensor_tasks':len(first),'treatment_correct':sum(v[1] for v in first.values()),
            'control_correct':sum(v[1] for v in other.values()),'paired_wins':wins,'paired_losses':losses,
            'accuracy_gain':(wins-losses)/len(first),'changed_first_actions':changed,
            'navigation_successes':{'treatment':nav_a,'control':nav_b},
            'external_actions':{'treatment':actions_a,'control':actions_b},
            'incremental_computation_work':{'treatment':work_a,'control':work_b},'phases':phases}
    learning={}
    for variant,run in runs.items():
        rows=[value[1] for key,value in sorted(run.items(),key=lambda x:x[0][2]) if key[0]=='mnist']
        window=protocol['new_task_acquisition']['window'];assert len(rows)>=2*window
        early=sum(rows[:window]);late=sum(rows[-window:])
        learning[variant]={'early_correct':early,'late_correct':late,'window':window,
            'accuracy_gain':(late-early)/window,
            'criterion_passed':(late-early)/window>=protocol['new_task_acquisition']['accuracy_gain_min'],
            'interpretation':'new class annotations for this continuing organism; same authored learning machinery; not invention or transfer of a new algorithm'}
    evidence=summaries[treatment]['final_report']['reward_memory']['acquired_rules']['first_readouts']
    assert evidence and not evidence['fixed_readout_control']
    frozen=variants[3]
    assert summaries[treatment]['mnist_entry_checkpoint_sha256']==summaries[frozen]['mnist_entry_checkpoint_sha256'], 'learning control did not share the exact prefix'
    keys=[k for k in first if k[0]=='mnist']
    learned_correct=sum(first[k][1] for k in keys);frozen_correct=sum(runs[frozen][k][1] for k in keys)
    acquisition={'tasks':len(keys),'learned_correct':learned_correct,'frozen_correct':frozen_correct,
        'accuracy_gain':(learned_correct-frozen_correct)/len(keys),
        'criterion_passed':(learned_correct-frozen_correct)/len(keys)>=protocol['new_task_acquisition']['frozen_control_accuracy_gain_min'],
        'identical_complete_prefix':True,'prefix_checkpoint_sha256':summaries[treatment]['mnist_entry_checkpoint_sha256'],
        'interpretation':'effect of continued learning on actual new MNIST tasks; does not demonstrate invention of a new learning algorithm'}
    return {'treatment':treatment,'comparisons':comparisons,'new_task_acquisition':learning,
        'matched_learning_control':acquisition,
        'first_readout_evidence':evidence,
        'summary_sha256':hashlib.sha256((root/'summary.json').read_bytes()).hexdigest(),
        'useful_new_method_transfer_established':False,
        'interpretation':protocol['interpretation']}


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--results',required=True);p.add_argument('--protocol',required=True);p.add_argument('--output',required=True)
    args=p.parse_args();result=compare(args.results,args.protocol)
    Path(args.output).write_text(json.dumps(result,indent=2,allow_nan=False)+'\n')
    print(json.dumps({k:v for k,v in result.items() if k!='comparisons'},indent=2))
