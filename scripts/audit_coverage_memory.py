"""Read-only corpus audit, parent binding and frozen-core verification."""
import argparse,gzip,json
from pathlib import Path
from audit_organism_lifetime import audit
from run_coverage_memory_lifetime import VARIANTS,compare,sha,raw_state,frozen

def learned_core(raw):
    state=json.loads(raw);learner=state['cognition']['current']['learner']
    memory={k:v for k,v in learner['external_memory'].items() if k not in ('events','recent','encoder')}
    causal={k:v for k,v in learner['causal_memory'].items()
        if k not in ('compacted_cost','compactions','context','events','journal_digest','offset')}
    return {'external_memory':memory,'causal_model':causal,
        'programs':{k:learner[k] for k in ('active','library','registry','promotions','shadow')},
        'synapses':frozen.learned_signature(raw)}

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
        assert provenance['data']==protocol['data'] and provenance['mnist_data']==protocol['mnist_data']
    for variant in VARIANTS:
        entry=json.loads(raw_state(life/variant/'starting-organism.json.gz'))
        em=entry['cognition']['current']['learner']['external_memory']
        if variant==VARIANTS[0]:
            policy=em.pop('coverage_memory')
            assert policy['enabled_at']==memory['events'] and policy['decisions']==0 and policy['work']==0
        assert {k:v for k,v in entry.items() if k!='hash'}=={k:v for k,v in parent_state.items() if k!='hash'},'more than policy/checksum changed before learning'
        final=json.loads(raw_state(life/variant/'one-organism.json.gz'))['cognition']
        assert not final['freeze_controller'] and not final['archived']
        end=final['current']['learner']['external_memory'];assert len(end['rows'])<=2048 and end['bytes']<=4*1024*1024
        assert all(a<=b for a,b in zip(memory['retired_failures']['buckets'],end['retired_failures']['buckets'],strict=True))
        if variant==VARIANTS[0]:
            assert end['coverage_memory']['decisions']>0 and end['coverage_memory']['non_fifo_evictions']>0
            assert end['coverage_memory']['work']<=final['computation_work']-parent_state['cognition']['computation_work']
        assert raw_state(probe/variant/'source-organism.json.gz')==raw_state(life/variant/'one-organism.json.gz')
    assert raw_state(probe/'organism_entry/source-organism.json.gz')==parent
    for variant in tests:
        source=raw_state(probe/variant/'source-organism.json.gz')
        entry=raw_state(probe/variant/'starting-organism.json.gz')
        final=raw_state(probe/variant/'one-organism.json.gz')
        assert json.loads(final)['cognition']['freeze_controller']
        a=learned_core(source);b=learned_core(entry);c=learned_core(final)
        assert a['external_memory']==b['external_memory'] and a['synapses']==b['synapses']
        # freeze closes working reward cost and freezes registry mutability;
        # compare all learned cores from AFTER that transition to final output.
        assert b==c,'frozen knowledge changed'
    result={'lives':audit(life,data),'probe':audit(probe,data),
        'same_parent_except_memory_policy':True,'one_learner_and_capacity_unchanged':True,
        'legacy_failure_sketch_never_reset':True,'all_frozen_learned_cores_unchanged':True}
    assert compare(protocol,training,tests,root)==json.loads((root/'comparison.json').read_text())
    return result

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for key in ('results','data','output'):p.add_argument('--'+key,required=True)
    args=p.parse_args();result=verify(args.results,args.data)
    Path(args.output).write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
