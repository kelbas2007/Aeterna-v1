"""Read-only corpus audit, parent binding and frozen-core verification."""
import argparse,gzip,json
from pathlib import Path
from audit_organism_lifetime import audit
from run_causal_readout_lifetime import VARIANTS,compare,sha,raw_state,frozen,learned_core
from run_external_continual5 import motors
from external5_sensors import features

def task_boundaries(folder,variant,model):
    frames={};context=None;pending=None;stats={};count=0;actions=0;candidate_exploits=0
    for name in ('life-before-resume.jsonl.gz','life-after-resume.jsonl.gz'):
        with gzip.open(folder/name,'rt') as lines:
            for line in lines:
                row=json.loads(line)
                if row['kind']=='sensor_frame':frames[row['id']]=row['frame']
                elif row['kind']=='restart':
                    context=row['metadata']
                    if context.get('task_start'):
                        assert pending is None,'overlapping external tasks'
                        receipt=context['policy_assignment'];namespace=features(frames[row['frame_ref']])[0]
                        if variant=='organism_causal':
                            assert receipt and receipt['namespace']==namespace
                            assert type(receipt['candidate']) is bool and type(receipt['experimental']) is bool
                        else:assert receipt is None
                        pending={'namespace':namespace,'receipt':receipt,'cost':0,'steps':0,'first':None}
                elif row['kind']=='actual_post' and pending is not None:
                    pending['cost']+=row['cost'];pending['steps']+=1;actions+=1
                    if row['action'] in motors(context['phase'])[1] and pending['first'] is None:pending['first']=row['reward']
                elif row['kind']=='task_end':
                    assert pending is not None and pending['steps']>0
                    count+=1;receipt=pending['receipt']
                    if receipt:
                        ns=pending['namespace'];d=stats.setdefault(str(ns),{'started':0,'trials_started':0,'exploited':0,
                            'baseline':{'tasks':0,'reward':0.,'cost':0},'candidate':{'tasks':0,'reward':0.,'cost':0},'parts':{}})
                        d['started']+=1
                        if receipt['experimental']:
                            d['trials_started']+=1;arm='candidate' if receipt['candidate'] else 'baseline'
                            d[arm]['tasks']+=1;d[arm]['reward']+=pending['first'] if pending['first'] is not None else 0.;d[arm]['cost']+=pending['cost']
                            pair=d['parts'].setdefault(receipt['block'],{});assert arm not in pair
                            pair[arm]={'reward':pending['first'] if pending['first'] is not None else 0.,'cost':pending['cost']}
                        else:
                            d['exploited']+=1;candidate_exploits+=int(receipt['candidate'])
                    pending=None
    assert pending is None and count==1280
    if variant=='organism_causal':
        policy=model['consolidated_memory']['causal_readout'];assert not policy['frozen'] and policy['task'] is None
        assert stats.keys()==policy['domains'].keys()
        for ns,d in stats.items():
            actual=policy['domains'][ns]
            for key in ('started','trials_started','exploited','baseline','candidate'):assert actual[key]==d[key],(ns,key)
            pairs=[{'baseline_reward':p['baseline']['reward'],'candidate_reward':p['candidate']['reward'],
                'baseline_cost':p['baseline']['cost'],'candidate_cost':p['candidate']['cost']}
                for block,p in sorted(d['parts'].items()) if len(p)==2]
            assert actual['completed_pairs']==len(pairs) and actual['pairs']==pairs[-128:]
            assert actual['abandoned']==0
    return {'status':'PASS','complete_tasks':count,'factual_actions_inside_tasks':actions,
        'assignment_receipts_before_actions':True,'first_rewards_and_full_costs_match_actual_POSTs':True,
        'candidate_exploit_tasks_after_admission':candidate_exploits}


def verify(root,data):
    root=Path(root);raw=(root/'registered-protocol.json').read_bytes();protocol=json.loads(raw)
    life=root/'lives';probe=root/'probe'
    parent=raw_state(life/'parent-organism.json.gz');assert sha(parent.encode())==protocol['parent_sha256']
    parent_state=json.loads(parent);memory=parent_state['cognition']['current']['learner']['external_memory']
    training=json.loads((life/'summary.json').read_text());tests=json.loads((probe/'summary.json').read_text())
    assert list(training)==list(VARIANTS) and list(tests)==list(VARIANTS)+['organism_entry']
    for directory in (life,probe):
        provenance=json.loads((directory/'provenance.json').read_text())
        assert (directory/'registered-protocol.json').read_bytes()==raw
        assert provenance['registered_protocol_sha256']==sha(raw)
        assert provenance['binary_sha256']==protocol['binary_sha256']
        assert provenance['source_sha256']==protocol['source_sha256']
        if directory==probe:assert provenance['source_indices']=={k:p['indices'] for k,p in protocol['probe_phases'].items()}
        assert provenance['data']==protocol['data'] and provenance['mnist_data']==protocol['mnist_data']
    for variant in VARIANTS:
        entry=json.loads(raw_state(life/variant/'starting-organism.json.gz'))
        em=entry['cognition']['current']['learner']['external_memory']
        expected=json.loads(parent)
        if variant!='organism_fifo':
            policy=em.pop('consolidated_memory')
            assert policy['enabled_at']==memory['events'] and policy['captured']==0 and policy['training_work']==0
            expected['cognition']['computation_work']+=policy['bootstrap_work']
        assert {k:v for k,v in entry.items() if k!='hash'}=={k:v for k,v in expected.items() if k!='hash'},'more than policy/checksum changed before learning'
        final=json.loads(raw_state(life/variant/'one-organism.json.gz'))['cognition']
        assert not final['freeze_controller'] and not final['archived']
        end=final['current']['learner']['external_memory'];assert len(end['rows'])<=2048 and end['bytes']<=4*1024*1024
        assert all(a<=b for a,b in zip(memory['retired_failures']['buckets'],end['retired_failures']['buckets'],strict=True))
        if variant!='organism_fifo':
            assert end['consolidated_memory']['captured']>0 and end['consolidated_memory']['resolved']>=64
            m=end['consolidated_memory'];extra=m.get('causal_readout',{}).get('work',0)
            assert m['bootstrap_work']+m['training_work']+extra<=final['computation_work']-parent_state['cognition']['computation_work']
        assert raw_state(probe/variant/'source-organism.json.gz')==raw_state(life/variant/'one-organism.json.gz')
    assert raw_state(probe/'organism_entry/source-organism.json.gz')==parent
    for variant in tests:
        source=raw_state(probe/variant/'source-organism.json.gz')
        entry=raw_state(probe/variant/'starting-organism.json.gz')
        final=raw_state(probe/variant/'one-organism.json.gz')
        assert json.loads(final)['cognition']['freeze_controller']
        a=learned_core(source);b=learned_core(entry);c=learned_core(final)
        policy=a['external_memory'].get('consolidated_memory',{}).get('causal_readout')
        if policy is not None:policy['frozen']=True
        assert a['external_memory']==b['external_memory'] and a['synapses']==b['synapses']
        # freeze closes working reward cost and freezes registry mutability;
        # compare all learned cores from AFTER that transition to final output.
        assert b==c,'frozen knowledge changed'
    result={'lives':audit(life,data),'probe':audit(probe,data),
        'same_parent_except_model_and_charged_seed_work':True,'one_learner_and_capacity_unchanged':True,
        'legacy_failure_sketch_never_reset':True,'all_frozen_learned_cores_unchanged':True}
    result['task_boundaries']={}
    for variant in VARIANTS:
        model=json.loads(raw_state(life/variant/'one-organism.json.gz'))['cognition']['current']['learner']['external_memory']
        result['task_boundaries'][variant]=task_boundaries(life/variant,variant,model)
    from run_causal_readout_lifetime import load
    spec=dict(protocol['probe_phases']['har'],kind='har')
    subjects=load(data,'test',spec)[2].tolist()
    assert subjects==json.loads((probe/'provenance.json').read_text())['test_grouping']['har_subjects']
    result['uncertainty_uses_actual_HAR_subject_groups']=True
    assert compare(protocol,training,tests,root)==json.loads((root/'comparison.json').read_text())
    return result

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for key in ('results','data','output'):p.add_argument('--'+key,required=True)
    args=p.parse_args();result=verify(args.results,args.data)
    Path(args.output).write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
