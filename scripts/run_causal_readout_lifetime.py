"""Registered whole-organism causal readout study; no external action policy."""
from __future__ import annotations
import argparse,gzip,hashlib,importlib.metadata,importlib.util,json,shutil,subprocess
from pathlib import Path
from external5_transport import Native
from external5_sensors import load_fashion,load_har
from run_external_continual5 import motors,verify_data_files
from run_organism_lifetime import sensory_phase,navigation_phase
from readout_statistics import compare_frozen_actions,candidate_exploit_tasks

ROOT=Path(__file__).resolve().parents[1]
HELPER=ROOT/'docs/evidence/first-readout/run_frozen_probe.py'
spec=importlib.util.spec_from_file_location('registered_frozen_probe',HELPER)
frozen=importlib.util.module_from_spec(spec);spec.loader.exec_module(frozen)
VARIANTS=('organism_causal','organism_consolidated','organism_fifo')
TRAIN=[{'name':'har_warmup','kind':'har','offset':3000,'count':256},
       {'name':'fashion_warmup','kind':'fashion','offset':55000,'count':256},
       {'name':'mnist','kind':'mnist','offset':10000,'count':512},
       {'name':'har_return','kind':'har','offset':4240,'count':128},
       {'name':'fashion_return','kind':'fashion','offset':55256,'count':128}]
PARTITION_SIZES={'mnist':10000,'har':2947,'fashion':10000}
def probe_indices(kind):
    import random
    excludes={'mnist':[(0,1000),(1000,1256),(4000,4256)],
        'har':[(0,1000),(2000,2256),(2650,2906)],
        'fashion':[(0,1000),(9000,9256),(9600,9856)]}
    previous=json.loads((ROOT/'docs/evidence/consolidated-memory/protocol.json').read_text())['probe_phases'][kind]['indices']
    available=[i for i in range(PARTITION_SIZES[kind]) if i not in previous and not any(a<=i<b for a,b in excludes[kind])]
    seed={'mnist':2026102121,'har':2026102122,'fashion':2026102123}[kind]
    return random.Random(seed).sample(available,512)
PROBE={k:{'offset':0,'count':512,'actions':5 if k=='har' else 2,'indices':probe_indices(k)} for k in ('mnist','har','fashion')}
CRITERIA={'har_gain_min':0.05,'other_drop_max':0.02,'pooled_gain_min':0.02,
          'navigation_drop_max':0,'training_action_ratio_max':1.25,'training_work_ratio_max':1.25,
          'entry_drop_max':0.02}

def sha(raw):return hashlib.sha256(raw).hexdigest()
def save(path,value):Path(path).write_text(json.dumps(value,indent=2,allow_nan=False)+'\n')
def raw_state(path):return gzip.decompress(Path(path).read_bytes()).decode()
def pack(path,raw):Path(path).write_bytes(gzip.compress(raw.encode(),mtime=0))
def revision():return subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
def source_binding():
    paths=[p for folder,suffix in [('src','*.rs'),('scripts','*.py')] for p in sorted((ROOT/folder).rglob(suffix))]
    paths += [ROOT/'docs/PROJECT_STATE.json',ROOT/'docs/CAUSAL_READOUT_PROTOCOL.md',HELPER]
    return {p.relative_to(ROOT).as_posix():sha(p.read_bytes()) for p in paths}
def mnist_files(data):
    originals={'train-images-idx3-ubyte.gz':'f68b3c2dcbeaaa9fbdd348bbdeb94873',
        'train-labels-idx1-ubyte.gz':'d53e105ee54ea40749a09fcbcd1e9432',
        't10k-images-idx3-ubyte.gz':'9fb629c4189551a2d022fa330f9573f3',
        't10k-labels-idx1-ubyte.gz':'ec29112dd5afa0611ce80d1b7f02629c'}
    result={}
    for name,md5 in originals.items():
        raw=(Path(data)/'mnist'/name).read_bytes();assert hashlib.md5(raw).hexdigest()==md5
        result[name]={'md5':md5,'sha256':sha(raw),'bytes':len(raw)}
    return result
def load(data,partition,phase):
    kind=phase['kind'];loader=load_har if kind=='har' else load_fashion
    root=Path(data)/'mnist' if kind=='mnist' else data
    if 'indices' in phase:
        import numpy as np
        indices=phase['indices'];low=min(indices);high=max(indices)
        arrays=loader(root,partition,low,high-low+1);take=np.array(indices)-low
        return tuple(None if x is None else x[take] for x in arrays)
    return loader(root,partition,phase['offset'],phase['count'])
def shared_provenance(protocol,binary):
    return {'purpose':protocol['interpretation'],'source_commit':revision(),
        'source_sha256':protocol['source_sha256'],'binary_sha256':sha(binary.read_bytes()),
        'frozen_binary':binary.name,'registered_protocol_sha256':sha((json.dumps(protocol,indent=2,allow_nan=False)+'\n').encode()),
        'data':protocol['data'],'mnist_data':protocol['mnist_data'],
        'dependencies':{n:importlib.metadata.version(n) for n in ('numpy','gymnasium','minigrid')},
        'starting_checkpoints':{},'starting_checkpoint_sha256':None}
def check_model(raw):
    state=json.loads(raw)['cognition'];assert not state['archived']
    memory=state['current']['learner']['external_memory'];assert len(memory['rows'])<=2048
    assert memory['bytes']<=4*1024*1024

def register(args):
    parent=raw_state(args.parent);check_model(parent)
    assert not json.loads(parent)['cognition']['freeze_controller']
    save(args.register,{'schema':'aeterna.causal-readout.v2','status':'registered before scoring',
        'interpretation':'paired development of one continuing organism; public reserved test rows; not invention of a learning algorithm',
        'parent_sha256':sha(parent.encode()),'binary_sha256':sha(Path(args.binary).read_bytes()),
        'source_sha256':source_binding(),'data':verify_data_files(args.data),'mnist_data':mnist_files(args.data),
        'training_phases':TRAIN,'probe_phases':PROBE,'variants':list(VARIANTS),'criteria':CRITERIA,
        'grid_seed':851300,'grid_episodes':4,'capacity':2048,
        'build':{'rust':'1.99.0','profile':'release','lto':False,'codegen_units':16}})

def verify(args,protocol):
    assert protocol['status']=='registered before scoring'
    assert not subprocess.check_output(['git','status','--porcelain'],cwd=ROOT,text=True).strip(),'clean committed source required'
    assert source_binding()==protocol['source_sha256'],'registered source changed'
    assert sha(Path(args.binary).read_bytes())==protocol['binary_sha256']
    assert sha(raw_state(args.parent).encode())==protocol['parent_sha256']
    assert verify_data_files(args.data)==protocol['data'] and mnist_files(args.data)==protocol['mnist_data']
    assert protocol['training_phases']==TRAIN and protocol['probe_phases']==PROBE and protocol['criteria']==CRITERIA

def lives(args,protocol,out):
    out.mkdir();binary=out/'frozen-aeterna-v1';shutil.copy2(args.binary,binary)
    parent=raw_state(args.parent);pack(out/'parent-organism.json.gz',parent)
    provenance=shared_provenance(protocol,binary)
    provenance.update({'variants':list(VARIANTS),'contextual_u1_requested':True,
        'training_ranges':{p['kind']:[p['offset'],p['offset']+p['count']] for p in TRAIN},
        'sensory_sources':{k:{str(p['offset']):p['count'] for p in TRAIN if p['kind']==k} for k in PROBE},
        'starting_events':json.loads(parent)['cognition']['current']['learner']['external_memory']['events']})
    save(out/'registered-protocol.json',protocol);results={}
    datasets={p['name']:load(args.data,'train',p) for p in TRAIN}
    for variant in VARIANTS:
        folder=out/variant;folder.mkdir()
        setup=Native(binary,folder/'setup.jsonl.gz',variant,checkpoint=parent)
        try:
            if variant!='organism_fifo':setup.command({'op':'organism_enable_consolidated_memory'})
            if variant!='organism_fifo':setup.command({'op':'organism_enable_partial_posterior'})
            if variant=='organism_causal':setup.command({'op':'organism_enable_causal_readout'})
            entry=setup.checkpoint()
        finally:setup.close()
        before=json.loads(parent);after=json.loads(entry)
        if variant!='organism_fifo':
            book=after['cognition']['current']['learner']['external_memory'].pop('consolidated_memory')
            assert book['enabled_at']==provenance['starting_events'] and book['captured']==0
            before['cognition']['computation_work']+=book['bootstrap_work']
        assert {k:v for k,v in before.items() if k!='hash'}=={k:v for k,v in after.items() if k!='hash'},'intervention changed more than memory policy/checksum'
        pack(folder/'starting-organism.json.gz',entry)
        provenance['starting_checkpoints'][variant]={'sha256':sha(entry.encode()),
            'events':provenance['starting_events'],'cost':before['cognition']['total_external_cost'],
            'file':variant+'/starting-organism.json.gz'}
        save(out/'provenance.json',provenance)
        agent=Native(binary,folder/'life-before-resume.jsonl.gz',variant,checkpoint=entry)
        phases=[];previous=provenance['starting_events'];resume_sha=None
        def record(name,result):
            nonlocal previous
            report=agent.command({'op':'organism_report'})
            assert report['reward_memory']['factual_events']==previous+result['actual_actions']
            assert report['organism']['total_external_cost']==previous+result['actual_actions']
            assert report['reward_memory']['retained_cases']<=2048
            assert sha(binary.read_bytes())==protocol['binary_sha256']
            previous=report['reward_memory']['factual_events']
            phases.append({'phase':name,'result':result,'report':report});save(folder/'phases.json',phases)
            print(json.dumps({'variant':variant,'phase':name,'actions':result['actual_actions'],
                'correct':result.get('first_answer_prequential',{}).get('correct'),
                'consolidation':report['reward_memory'].get('consolidated_memory')}),flush=True)
        try:
            for number,p in enumerate(TRAIN):
                record(p['name'],sensory_phase(agent,p['kind'],datasets[p['name']],source_offset=p['offset'],task_boundaries=True))
                if number==1:
                    saved=agent.checkpoint();resume_sha=sha(saved.encode())
                    pack(folder/'resume.json.gz',saved);pack(folder/'before-mnist.json.gz',saved)
                    agent.close();agent=Native(binary,folder/'life-after-resume.jsonl.gz',variant,checkpoint=saved)
                    assert agent.checkpoint()==saved
                if p['kind']=='mnist':
                    for index,game in enumerate(('MiniGrid-Empty-5x5-v0','MiniGrid-DoorKey-5x5-v0')):
                        record(game,navigation_phase(agent,game,20+index,range(protocol['grid_seed'],protocol['grid_seed']+protocol['grid_episodes'])))
            continuing=agent.checkpoint();check_model(continuing);pack(folder/'one-organism.json.gz',continuing)
            assert not json.loads(continuing)['cognition']['freeze_controller']
            continuing_report=agent.command({'op':'organism_report'})
            agent.freeze();final=agent.checkpoint();pack(folder/'frozen-organism.json.gz',final)
            report=agent.command({'op':'organism_report'})
            assert report['development']['namespaces']==[4,9,12,82,86]
            results[variant]={'phases':phases,'resume_checkpoint_sha256':resume_sha,
                'mnist_entry_checkpoint_sha256':resume_sha,'continuing_checkpoint_sha256':sha(continuing.encode()),
                'final_checkpoint_sha256':sha(final.encode()),'continuing_report':continuing_report,'final_report':report,'frozen_after_matched_prefix':False}
            save(folder/'summary.json',results[variant]);save(out/'summary.json',results)
        finally:agent.close()
    return results

def probes(args,protocol,root,out):
    out.mkdir();binary=out/'frozen-aeterna-v1';shutil.copy2(args.binary,binary)
    provenance=shared_provenance(protocol,binary)
    sources={v:root/v/'one-organism.json.gz' for v in VARIANTS};sources['organism_entry']=Path(args.parent)
    provenance.update({'frozen_probe':True,'source_partition':'test','variants':list(sources),
        'training_ranges':{k:[0,PARTITION_SIZES[k]] for k in PROBE},
        'source_indices':{k:p['indices'] for k,p in PROBE.items()},
        'sensory_sources':{k:{str(p['offset']):p['count']} for k,p in PROBE.items()},
        'probe_action_limits':{k:p['actions'] for k,p in PROBE.items()}})
    save(out/'registered-protocol.json',protocol)
    datasets={k:load(args.data,'test',dict(p,kind=k)) for k,p in PROBE.items()};results={}
    provenance['test_grouping']={'har_subjects':datasets['har'][2].tolist()}
    for variant,path in sources.items():
        folder=out/variant;folder.mkdir();source=raw_state(path)
        pack(folder/'source-organism.json.gz',source)
        setup=Native(binary,folder/'setup.jsonl.gz',variant,checkpoint=source)
        try:setup.freeze();entry=setup.checkpoint();initial=setup.command({'op':'organism_report'})
        finally:setup.close()
        pack(folder/'starting-organism.json.gz',entry);pack(folder/'before-mnist.json.gz',entry)
        provenance['starting_checkpoints'][variant]={'sha256':sha(entry.encode()),
            'events':initial['reward_memory']['factual_events'],'cost':initial['organism']['total_external_cost'],
            'file':variant+'/starting-organism.json.gz'}
        save(out/'provenance.json',provenance)
        fingerprint=initial['reward_memory']['learned_fingerprint'];weights=frozen.learned_signature(entry)
        agent=Native(binary,folder/'life-before-resume.jsonl.gz',variant,checkpoint=entry)
        phases=[];resume_sha=None;previous=initial['reward_memory']['factual_events']
        try:
            for kind,p in PROBE.items():
                result=frozen.phase(agent,kind,datasets[kind],p['offset'],p['actions'])
                report=agent.command({'op':'organism_report'})
                assert report['organism']['controller_frozen']
                assert report['reward_memory']['learned_fingerprint']==fingerprint
                assert report['reward_memory']['factual_events']==previous+result['actual_actions']
                previous=report['reward_memory']['factual_events']
                phases.append({'phase':kind,'result':result,'report':report});save(folder/'phases.json',phases)
                print(json.dumps({'probe':variant,'phase':kind,'correct':result['first_answer_prequential']['correct'],'actions':result['actual_actions']}),flush=True)
                if kind=='mnist':
                    saved=agent.checkpoint();resume_sha=sha(saved.encode());pack(folder/'resume.json.gz',saved)
                    agent.close();agent=Native(binary,folder/'life-after-resume.jsonl.gz',variant,checkpoint=saved)
                    assert agent.checkpoint()==saved
            final=agent.checkpoint();assert frozen.learned_signature(final)==weights;check_model(final)
            pack(folder/'one-organism.json.gz',final);pack(folder/'frozen-organism.json.gz',final)
            results[variant]={'phases':phases,'final_report':agent.command({'op':'organism_report'}),
                'resume_checkpoint_sha256':resume_sha,'mnist_entry_checkpoint_sha256':sha(entry.encode()),
                'final_checkpoint_sha256':sha(final.encode()),'continuing_checkpoint_sha256':sha(final.encode()),
                'frozen_after_matched_prefix':True,'learned_fingerprint_unchanged':True,'learned_synapses_unchanged':True}
            save(folder/'summary.json',results[variant]);save(out/'summary.json',results)
        finally:agent.close()
    return results

def first_answers(folder):
    result={};context=None
    for name in ('life-before-resume.jsonl.gz','life-after-resume.jsonl.gz'):
        with gzip.open(folder/name,'rt') as rows:
            for line in rows:
                row=json.loads(line)
                if row['kind']=='restart':context=row['metadata']
                if row['kind']=='intent' and row['action'] in motors(context['phase'])[1]:
                    key=(context['phase'],context['index']);assert key not in result
                    result[key]=row['action']
    return result
def compare(protocol,training,test,out):
    def scores(v):return {p['phase']:p['result']['first_answer_prequential'] for p in test[v]['phases']}
    a=scores(VARIANTS[0]);b=scores('organism_fifo');entry=scores('organism_entry')
    gains={k:a[k]['accuracy']-b[k]['accuracy'] for k in PROBE}
    retention={k:a[k]['accuracy']-entry[k]['accuracy'] for k in PROBE}
    baseline=json.loads(raw_state(out/'lives/parent-organism.json.gz'))['cognition']['computation_work']
    costs={v:{'actions':sum(p['result']['actual_actions'] for p in training[v]['phases']),
        'work':training[v]['final_report']['organism']['computation_work']-baseline,
        'navigation_successes':sum(p['result'].get('successes',0) for p in training[v]['phases'])} for v in VARIANTS}
    answers={v:first_answers(out/'probe'/v) for v in VARIANTS}
    total_samples=sum(p['count'] for p in PROBE.values())
    assert set(answers[VARIANTS[0]])==set(answers['organism_fifo']) and len(answers[VARIANTS[0]])==total_samples
    changed=sum(answers[VARIANTS[0]][k]!=answers['organism_fifo'][k] for k in answers[VARIANTS[0]])
    pooled=(sum(a[k]['correct'] for k in PROBE)-sum(b[k]['correct'] for k in PROBE))/total_samples
    c=protocol['criteria'];t,u=VARIANTS[0],VARIANTS[-1]
    uncertainty=compare_frozen_actions(out,VARIANTS)
    brier=scores('organism_consolidated')
    brier_gains={k:a[k]['accuracy']-brier[k]['accuracy'] for k in PROBE}
    brier_pooled=sum(a[k]['correct']-brier[k]['correct'] for k in PROBE)/total_samples
    exploited=candidate_exploit_tasks(out/'lives'/t)
    checks={'har_improvement':gains['har']+1e-12>=c['har_gain_min'],
        'mnist_no_regression':gains['mnist']+1e-12>=-c['other_drop_max'],
        'fashion_no_regression':gains['fashion']+1e-12>=-c['other_drop_max'],
        'pooled_improvement':pooled+1e-12>=c['pooled_gain_min'],
        'navigation_no_regression':costs[t]['navigation_successes']>=costs[u]['navigation_successes'],
        'action_cost':costs[t]['actions']/costs[u]['actions']<=c['training_action_ratio_max'],
        'computation_cost':costs[t]['work']/costs[u]['work']<=c['training_work_ratio_max'],
        'actual_changed_answers':changed>0,'bounded_one_learner':True,'frozen_models_unchanged':True,
        'model_budget':all(p['report']['reward_memory']['model_bytes']<=4*1024*1024 for v in VARIANTS for p in training[v]['phases']),
        'statistical_readout_selected':any(d['consolidated_active'] for k,d in training[t]['final_report']['reward_memory']['consolidated_memory']['causal_readout']['domains'].items() if k in ('4','9','12')),
        'live_use_after_admission':exploited>0,
        'har_improvement_vs_brier':brier_gains['har']+1e-12>=c['har_gain_min'],
        'images_no_regression_vs_brier':all(brier_gains[k]+1e-12>=-c['other_drop_max'] for k in ('mnist','fashion')),
        'pooled_improvement_vs_brier':brier_pooled+1e-12>=c['pooled_gain_min'],
        'action_cost_vs_brier':costs[t]['actions']/costs['organism_consolidated']['actions']<=c['training_action_ratio_max'],
        'computation_cost_vs_brier':costs[t]['work']/costs['organism_consolidated']['work']<=c['training_work_ratio_max'],
        'positive_har_lower95':all(s['har']['lower95']>0 for s in uncertainty['controls'].values()),
        'positive_pooled_lower95':all(s['pooled']['lower95']>0 for s in uncertainty['controls'].values())}
    entry_checks={k:retention[k]+1e-12>=-c['entry_drop_max'] for k in PROBE}
    passed=all(checks.values()) and all(entry_checks.values())
    result={'purpose':protocol['interpretation'],'scores':{v:scores(v) for v in test},
        'accuracy_gains_vs_fifo':gains,'accuracy_gains_vs_parent':retention,'pooled_gain':pooled,
        'accuracy_gains_vs_brier':brier_gains,'pooled_gain_vs_brier':brier_pooled,
        'paired_uncertainty':uncertainty,'live_candidate_exploit_tasks':exploited,
        'training_costs':costs,'changed_first_test_answers':changed,'improvement_checks':checks,
        'improvement_criterion_passed':all(checks.values()),'retention_checks':entry_checks,
        'retention_criterion_passed':all(entry_checks.values()),'combined_criterion_passed':passed,
        'selected_continuation':t if passed else 'published_PR18_parent',
        'new_learning_algorithm_invention_established':False}
    return result

def run(args):
    if args.register:register(args);return
    assert args.protocol and args.output
    protocol=json.loads(Path(args.protocol).read_text());verify(args,protocol)
    out=Path(args.output)
    if args.probe_existing:
        assert (out/'registered-protocol.json').read_bytes()==Path(args.protocol).read_bytes()
        training=json.loads((out/'lives/summary.json').read_text())
    else:
        out.mkdir(parents=True,exist_ok=False)
        (out/'registered-protocol.json').write_bytes(Path(args.protocol).read_bytes())
        training=lives(args,protocol,out/'lives')
    if args.train_only:
        save(out/'stage.json',{'status':'training completed; new test rows not decoded','held_out_evaluation_started':False})
        return
    verify(args,protocol)  # No retuning before opening the new test rows.
    test=probes(args,protocol,out/'lives',out/'probe')
    result=compare(protocol,training,test,out);save(out/'comparison.json',result)
    print(json.dumps(result,indent=2),flush=True)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for key in ('binary','data','parent'):p.add_argument('--'+key,required=True)
    p.add_argument('--register');p.add_argument('--protocol');p.add_argument('--output')
    p.add_argument('--train-only',action='store_true');p.add_argument('--probe-existing',action='store_true')
    run(p.parse_args())
