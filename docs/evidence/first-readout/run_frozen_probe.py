"""Registered frozen evaluation of complete organisms on reserved test rows.

Both agents disable model updates AND random exploration. Each remains one
checkpoint through MNIST -> exact resume -> HAR -> Fashion. No corrective retry.
"""
import argparse,gzip,hashlib,json,shutil,subprocess,sys
from pathlib import Path

ROOT=Path(__file__).resolve().parents[3]
sys.path.insert(0,str(ROOT/'scripts'))
from external5_transport import Native
from external5_sensors import load_fashion,load_har,blank,acquire,metrics
from run_external_continual5 import motors,verify_data_files

def sha(raw):return hashlib.sha256(raw).hexdigest()
def save(path,value):Path(path).write_text(json.dumps(value,indent=2,allow_nan=False)+'\n')


def learned_signature(raw):
    state=json.loads(raw);carrier=json.loads(bytes(state['carrier']))
    def synapses(rows):return [(r['weight'],r['phase_offset'],r['plastic']) for r in rows]
    weights={'physical':synapses(carrier['synapses']),
        'prior':synapses(carrier['contextual']['prior']),
        'rows':{k:synapses(v['synapses']) for k,v in carrier['contextual']['rows'].items()}}
    return sha(json.dumps(weights,sort_keys=True,separators=(',',':')).encode())


def phase(agent,kind,data,offset,limit):
    x,y,_=data;queries,answers=motors(kind);predictions=[];actions=0
    for index,(sample,target) in enumerate(zip(x,y)):
        frame=blank(kind);measured=set();prediction=None
        agent.reset(frame,phase=kind,index=index,source_offset=offset,partition='test',frozen_probe=True)
        for left in range(limit,0,-1):
            available=[False]*16
            for axis,motor in enumerate(queries):available[motor]=axis not in measured and left>1
            for motor in answers:available[motor]=kind=='har' or bool(measured)
            def execute(action):
                nonlocal frame,prediction
                if action in queries:
                    axis=queries.index(action);measured.add(axis);frame=acquire(frame,axis,sample);reward=0.0
                else:
                    prediction=answers.index(action);reward=float(prediction==int(target))
                return {'frame':frame,'reward':reward,'cost':1}
            agent.step(available,left,execute);actions+=1
            if prediction is not None:break
        agent.finish()
        if prediction is None:raise AssertionError('no first response within registered action limit')
        predictions.append(prediction)
    return {'first_answer_prequential':metrics(y,predictions,6 if kind=='har' else 10),
        'actual_actions':actions,'learning':False,'random_exploration':False}


def run(args):
    protocol_raw=Path(args.protocol).read_bytes();protocol=json.loads(protocol_raw)
    if protocol['status']!='registered before probe':raise ValueError('unregistered probe')
    if subprocess.check_output(['git','status','--porcelain'],cwd=ROOT,text=True).strip():raise ValueError('clean committed source required')
    for name,expected in protocol['source_sha256'].items():
        if sha((ROOT/name).read_bytes())!=expected:raise ValueError('unregistered source: '+name)
    if sha(Path(args.binary).read_bytes())!=protocol['binary_sha256']:raise ValueError('unregistered binary')
    if verify_data_files(args.data)!=protocol['data']:raise ValueError('unregistered source data')
    for name,expected in protocol['mnist_data'].items():
        raw=(Path(args.data)/'mnist'/name).read_bytes()
        if sha(raw)!=expected['sha256'] or hashlib.md5(raw).hexdigest()!=expected['md5']:raise ValueError('MNIST checksum mismatch')
    sources={'organism_after_learning':Path(args.learned),'organism_before_learning':Path(args.control)}
    for name,path in sources.items():
        if sha(gzip.decompress(path.read_bytes()))!=protocol['source_checkpoints'][name]:raise ValueError('unregistered whole-organism checkpoint')
    out=Path(args.output);out.mkdir(parents=True,exist_ok=False)
    binary=out/'frozen-aeterna-v1';shutil.copy2(args.binary,binary)
    (out/'registered-protocol.json').write_bytes(protocol_raw)
    datasets={}
    for kind,spec in protocol['phase_specs'].items():
        loader=load_har if kind=='har' else load_fashion
        datasets[kind]=loader(Path(args.data)/'mnist' if kind=='mnist' else args.data,'test',spec['offset'],spec['count'])
    provenance={'purpose':protocol['interpretation'],'frozen_probe':True,'source_partition':'test',
        'source_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
        'binary_sha256':sha(binary.read_bytes()),'frozen_binary':binary.name,
        'registered_protocol_sha256':sha(protocol_raw),'starting_checkpoint_sha256':None,
        'starting_checkpoints':{},'variants':list(sources),'data':protocol['data'],'mnist_data':protocol['mnist_data'],
        'training_ranges':{k:[v['offset'],v['offset']+v['count']] for k,v in protocol['phase_specs'].items()},
        'sensory_sources':{k:{str(v['offset']):v['count']} for k,v in protocol['phase_specs'].items()},
        'probe_action_limits':{k:v['actions'] for k,v in protocol['phase_specs'].items()}}
    results={}
    for variant,path in sources.items():
        folder=out/variant;folder.mkdir();source=gzip.decompress(path.read_bytes()).decode()
        gzip.open(folder/'source-organism.json.gz','wt').write(source)
        setup=Native(binary,folder/'setup.jsonl.gz',variant,checkpoint=source)
        try:
            setup.freeze();entry=setup.checkpoint();initial=setup.command({'op':'organism_report'})
        finally:setup.close()
        gzip.open(folder/'starting-organism.json.gz','wt').write(entry)
        gzip.open(folder/'before-mnist.json.gz','wt').write(entry)
        provenance['starting_checkpoints'][variant]={'sha256':sha(entry.encode()),'events':initial['reward_memory']['factual_events'],
            'cost':initial['organism']['total_external_cost'],'file':variant+'/starting-organism.json.gz'}
        save(out/'provenance.json',provenance)
        fingerprint=initial['reward_memory']['learned_fingerprint'];weights=learned_signature(entry);phases=[]
        agent=Native(binary,folder/'life-before-resume.jsonl.gz',variant,checkpoint=entry)
        try:
            previous=initial['reward_memory']['factual_events'];resume_sha=None
            for kind in protocol['phase_order']:
                spec=protocol['phase_specs'][kind];result=phase(agent,kind,datasets[kind],spec['offset'],spec['actions'])
                report=agent.command({'op':'organism_report'})
                assert report['organism']['controller_frozen']
                assert report['reward_memory']['learned_fingerprint']==fingerprint
                assert report['reward_memory']['factual_events']==previous+result['actual_actions'];previous=report['reward_memory']['factual_events']
                phases.append({'phase':kind,'result':result,'report':report});save(folder/'phases.json',phases)
                print(json.dumps({'variant':variant,'phase':kind,'correct':result['first_answer_prequential']['correct'],'actions':result['actual_actions']}),flush=True)
                if kind=='mnist':
                    saved=agent.checkpoint();resume_sha=sha(saved.encode());gzip.open(folder/'resume.json.gz','wt').write(saved)
                    agent.close();agent=Native(binary,folder/'life-after-resume.jsonl.gz',variant,checkpoint=saved)
                    assert agent.checkpoint()==saved
            final=agent.checkpoint();assert learned_signature(final)==weights
            gzip.open(folder/'one-organism.json.gz','wt').write(final);gzip.open(folder/'frozen-organism.json.gz','wt').write(final)
            results[variant]={'phases':phases,'final_report':agent.command({'op':'organism_report'}),
                'resume_checkpoint_sha256':resume_sha,'mnist_entry_checkpoint_sha256':sha(entry.encode()),
                'final_checkpoint_sha256':sha(final.encode()),'continuing_checkpoint_sha256':sha(final.encode()),
                'frozen_after_matched_prefix':True,'learned_fingerprint_unchanged':True,'learned_synapses_unchanged':True}
            save(folder/'summary.json',results[variant]);save(out/'summary.json',results)
        finally:agent.close()
    treatment={r['phase']:r['result'] for r in results['organism_after_learning']['phases']}
    control={r['phase']:r['result'] for r in results['organism_before_learning']['phases']}
    gains={k:treatment[k]['first_answer_prequential']['accuracy']-control[k]['first_answer_prequential']['accuracy'] for k in treatment}
    checks={'new_mnist_skill':gains['mnist']>=protocol['criterion']['mnist_gain_min'],
        'fashion_retention':gains['fashion']>=-protocol['criterion']['old_skill_drop_max'],
        'har_retention':gains['har']>=-protocol['criterion']['old_skill_drop_max']}
    verdict={'purpose':protocol['interpretation'],'accuracy_gains':gains,'checks':checks,'criterion_passed':all(checks.values()),
        'same_frozen_exploration_policy':True,'new_learning_during_probe':False,'new_algorithm_invention_established':False}
    save(out/'comparison.json',verdict);print(json.dumps(verdict),flush=True)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for key in ('binary','data','output','protocol','learned','control'):p.add_argument('--'+key,required=True)
    run(p.parse_args())
