import sys,json,gzip,math
from pathlib import Path
import numpy as np
ROOT=Path('/workspace/Aeterna-v1');sys.path.insert(0,str(ROOT/'scripts'))
from external5_sensors import features
from run_external_continual5 import motors
from run_consolidated_memory_lifetime import load
root=ROOT/'docs/evidence/consolidated-memory'
protocol=json.loads((root/'protocol.json').read_text())
memory=json.loads(gzip.decompress((root/'lives/organism_consolidated/one-organism.json.gz').read_bytes()))['cognition']['current']['learner']['external_memory']
frames={};context=None;rows={}
for name in ('life-before-resume.jsonl.gz','life-after-resume.jsonl.gz'):
    for line in gzip.open(root/'probe/organism_consolidated'/name,'rt'):
        r=json.loads(line)
        if r['kind']=='sensor_frame':frames[r['id']]=r['frame']
        if r['kind']=='restart':context=r['metadata']
        if r['kind']=='intent' and r['action'] in motors(context['phase'])[1]:
            rows[context['phase'],context['index']]=(frames[r['pre_ref']],r['action'])
result={'purpose':'Retrospective readout diagnosis on already consumed V2 rows; no fitting to test outcomes; not a new behavioral experiment','new_external_actions':0,'phases':{}}
for kind,p in protocol['probe_phases'].items():
    _,labels,_=load('/workspace/.aeterna-env/external5-data','test',dict(p,kind=kind))
    ns={'mnist':12,'har':9,'fashion':4}[kind]
    domain=memory['consolidated_memory']['domains'][str(ns)];g=domain['global']['features']
    answers=motors(kind)[1];correct=0;actual_correct=0;diff=0
    for index,label in enumerate(labels):
        frame,actual=rows[kind,index];_,mask,x=features(frame);logits=[];distances=[]
        known=[j for j in range(30) if mask&(1<<j)]
        for action in answers:
            c=domain['outcomes'][action];f=c['features'];score=0.;distance=0.
            for j in known:
                gv=g['m2'][j]/max(1,g['count'][j]);mean=f['mean'][j] if f['count'][j] else g['mean'][j]
                var=f['m2'][j]/f['count'][j] if f['count'][j] else gv
                var=max(var,gv*.05,64.);d=(float(x[j])-mean)**2/var
                score-=.5*(math.log(var)+d);distance+=d
            logits.append(score if c['examples'] and known else -math.inf)
        pred=answers[int(np.argmax(logits))]
        correct+=pred==answers[int(label)];actual_correct+=actual==answers[int(label)];diff+=pred!=actual
    result['phases'][kind]={'samples':len(labels),'recorded_whole_agent_correct':actual_correct,'raw_gaussian_argmax_correct_same_frames':correct,'changed_predictions':diff,'factual_guards_U1_and_sensor_changes_included':False}
out=Path('/workspace/.aeterna-env/readout-v2-retrospective.json');out.write_text(json.dumps(result,indent=2)+'\n');print(out.read_text())
