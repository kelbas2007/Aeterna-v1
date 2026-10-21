"""Retrospective diagnostic on consumed trial rows, never fresh qualification.

Fit a simple factorized Gaussian readout only to already retained successful
facts. Compare its coverage on the FIFO agent's recorded answer frames and
counterfactual fully measured copies of the SAME consumed examples. No U1,
factual frequency guards, tree program or new agent behavior is simulated.
"""
import argparse,gzip,json,sys
from pathlib import Path
import numpy as np
ROOT=Path(__file__).resolve().parents[3];sys.path.insert(0,str(ROOT/'scripts'))
from external5_sensors import features,blank,acquire
from run_external_continual5 import motors
from run_coverage_memory_lifetime import load,raw_state,sha

def frames(folder):
    result={};context=None
    for name in ('life-before-resume.jsonl.gz','life-after-resume.jsonl.gz'):
        seen={}
        with gzip.open(folder/name,'rt') as rows:
            for line in rows:
                r=json.loads(line)
                if r['kind']=='sensor_frame':seen[r['id']]=r['frame']
                elif r['kind']=='restart':context=r['metadata']
                elif r['kind']=='intent' and r['action'] in motors(context['phase'])[1]:
                    result[(context['phase'],context['index'])]=seen[r['pre_ref']]
    assert len(result)==768;return result

def fit(rows,namespace):
    n=np.zeros((16,30));s=n.copy();q=n.copy();classes=np.zeros(16)
    for r in rows:
        if r['reward']<=0 or r['pre'][0]!=namespace:continue
        a=r['action'];mask=r['pre'][1];x=np.array(r['pre'][2:]);known=np.array([bool(mask&(1<<j)) for j in range(30)])
        classes[a]+=1;n[a,known]+=1;s[a,known]+=x[known];q[a,known]+=x[known]**2
    total=n.sum(axis=0);mean=np.divide(s.sum(axis=0),total,out=np.zeros(30),where=total>0)
    var=np.maximum(np.divide(q.sum(axis=0),total,out=np.zeros(30),where=total>0)-mean**2,0)
    means=np.divide(s,n,out=np.broadcast_to(mean,(16,30)).copy(),where=n>0)
    variances=np.divide(q,n,out=np.broadcast_to(var+mean**2,(16,30)).copy(),where=n>0)-means**2
    variances=np.maximum(variances,np.maximum(64,var*.05))
    return means,variances,classes

def predict(model,frame,answers):
    _,mask,x=features(frame);known=np.array([bool(mask&(1<<j)) for j in range(30)])
    means,var,classes=model
    if not known.any():scores=np.zeros(len(answers))
    else:scores=-.5*(np.log(var[answers][:,known])+(x[known]-means[answers][:,known])**2/var[answers][:,known]).mean(axis=1)
    scores=np.where(classes[answers]>0,scores,-np.inf)
    return int(np.argmax(scores))

def main():
    p=argparse.ArgumentParser(description=__doc__)
    for key in ('results','data','output'):p.add_argument('--'+key,required=True)
    a=p.parse_args();root=Path(a.results);protocol=json.loads((root/'registered-protocol.json').read_text())
    observed=frames(root/'probe/organism_fifo');models={'parent':root/'lives/parent-organism.json.gz',
        'fifo':root/'lives/organism_fifo/one-organism.json.gz','coverage':root/'lives/organism_coverage/one-organism.json.gz'}
    report={'purpose':__doc__,'new_external_actions':0,'fresh_behavioral_evidence':False,'models':{}}
    for name,path in models.items():
        raw=raw_state(path);rows=json.loads(raw)['cognition']['current']['learner']['external_memory']['rows'];current={}
        for kind,spec in protocol['probe_phases'].items():
            x,y,_=load(a.data,'test',dict(spec,kind=kind));answers=motors(kind)[1];namespace=9 if kind=='har' else 4 if kind=='fashion' else 12
            model=fit(rows,namespace);positive=[r for r in rows if r['reward']>0 and r['pre'][0]==namespace]
            measured_correct=full_correct=missing=full_missing=0;counts=[]
            for index,(sample,target) in enumerate(zip(x,y)):
                frame=observed[(kind,index)];_,mask,_=features(frame)
                eligible=[r for r in positive if r['pre'][1]&mask==mask];counts.append(len(eligible))
                missing+=not any(r['action']==answers[int(target)] for r in eligible)
                measured_correct+=predict(model,frame,answers)==int(target)
                full=blank(kind)
                for axis in range(9 if kind=='har' else 1):full=acquire(full,axis,sample)
                _,fm,_=features(full);full_missing+=not any(r['action']==answers[int(target)] and r['pre'][1]&fm==fm for r in positive)
                full_correct+=predict(model,full,answers)==int(target)
            current[kind]={'samples':len(y),'positive_prototypes':len(positive),
                'true_motor_without_eligible_prototype_at_recorded_mask':int(missing),
                'true_motor_without_eligible_prototype_at_full_mask':int(full_missing),
                'eligible_prototypes_at_recorded_mask_mean':float(np.mean(counts)),
                'gaussian_correct_on_recorded_masks':int(measured_correct),'gaussian_correct_full_information_diagnostic':int(full_correct)}
        report['models'][name]={'checkpoint_sha256':sha(raw.encode()),'phases':current}
    Path(a.output).write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))

if __name__=='__main__':main()
