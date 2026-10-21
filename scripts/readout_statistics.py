"""Paired uncertainty from recorded frozen actions; never an actor or trainer."""
import gzip,json
from pathlib import Path
import numpy as np
from run_external_continual5 import motors

def recorded_outcomes(folder):
    result={};context=None
    for name in ('life-before-resume.jsonl.gz','life-after-resume.jsonl.gz'):
        with gzip.open(Path(folder)/name,'rt') as rows:
            for line in rows:
                r=json.loads(line)
                if r['kind']=='restart':context=r['metadata']
                if r['kind']=='actual_post' and r['action'] in motors(context['phase'])[1]:
                    key=(context['phase'],context['index'])
                    assert key not in result and r['reward'] in (0.0,1.0)
                    result[key]=r['reward']
    return result

def bootstrap_difference(difference,rng,groups=None,repeats=20000):
    d=np.asarray(difference,dtype=float)
    if groups is None:
        counts=np.array([(d==-1).sum(),(d==0).sum(),(d==1).sum()])
        assert int(counts.sum())==len(d)
        sampled=rng.multinomial(len(d),counts/len(d),size=repeats)
        return (sampled[:,2]-sampled[:,0])/len(d)
    groups=np.asarray(groups);assert len(groups)==len(d)
    ids=np.unique(groups);sizes=np.array([(groups==g).sum() for g in ids])
    sums=np.array([d[groups==g].sum() for g in ids])
    selected=rng.integers(0,len(ids),size=(repeats,len(ids)))
    return sums[selected].sum(axis=1)/sizes[selected].sum(axis=1)

def compare_frozen_actions(root,variants):
    root=Path(root);protocol=json.loads((root/'registered-protocol.json').read_text())
    grouping=json.loads((root/'probe/provenance.json').read_text())['test_grouping']['har_subjects']
    truth={v:recorded_outcomes(root/'probe'/v) for v in variants}
    target=variants[0];result={};rng=np.random.default_rng(2026102124)
    for control in variants[1:]:
        assert truth[target].keys()==truth[control].keys()
        boot=[];current={}
        for kind,spec in protocol['probe_phases'].items():
            d=np.array([truth[target][kind,i]-truth[control][kind,i] for i in range(spec['count'])])
            samples=bootstrap_difference(d,rng,grouping if kind=='har' else None)
            low,high=np.quantile(samples,[.025,.975])
            current[kind]={'gain':float(d.mean()),'lower95':float(low),'upper95':float(high),
                'paired_candidate_only_correct':int((d==1).sum()),'paired_control_only_correct':int((d==-1).sum()),
                'grouping':'HAR subject clusters' if kind=='har' else 'corpus images',
                'groups':len(set(grouping)) if kind=='har' else len(d)}
            boot.append(samples)
        low,high=np.quantile(np.mean(boot,axis=0),[.025,.975])
        current['pooled']={'gain':sum(current[k]['gain'] for k in protocol['probe_phases'])/3,
            'lower95':float(low),'upper95':float(high),'grouping':'equal corpus weights; HAR clustered by subject'}
        result[control]=current
    return {'purpose':'Conditional paired bootstrap of frozen whole-agent outcomes; HAR resampled by subject, not overlapping windows',
        'repeats':20000,'seed':2026102124,'controls':result}

def candidate_exploit_tasks(folder):
    assignment=None;actions=0;result=0
    for name in ('life-before-resume.jsonl.gz','life-after-resume.jsonl.gz'):
        with gzip.open(Path(folder)/name,'rt') as rows:
            for line in rows:
                r=json.loads(line)
                if r['kind']=='restart' and r['metadata'].get('task_start'):
                    assignment=r['metadata']['policy_assignment'];actions=0
                if r['kind']=='actual_post' and assignment is not None:actions+=1
                if r['kind']=='task_end':
                    if assignment and actions and assignment['candidate'] and not assignment['experimental']:result+=1
                    assignment=None;actions=0
    return result
