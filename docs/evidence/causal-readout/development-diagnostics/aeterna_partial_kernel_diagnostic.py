"""One pre-specified kernel diagnostic on already consumed V2 rows; no new world actions."""
import contextlib,io,runpy,math,json
import numpy as np
with contextlib.redirect_stdout(io.StringIO()): old=runpy.run_path('/tmp/aeterna_readout_diagnostic.py')
memory=old['memory'];frames=old['rows'];protocol=old['protocol'];load=old['load'];features=old['features'];motors=old['motors']
report={'purpose':__doc__,'new_external_actions':0,'fresh_behavioral_evidence':False,'phases':{}}
def failures(key):
    b=memory.get('retired_failures',{}).get('buckets');values=[]
    if b is None:return 0
    for row in range(4):
        h=0xcbf29ce484222325 ^ ((row*0x9e3779b97f4a7c15)&((1<<64)-1))
        for byte in key.encode():h=((h^byte)*0x100000001b3)&((1<<64)-1)
        values.append(b[row*2048+h%2048])
    return min(values)
for kind,p in protocol['probe_phases'].items():
    _,labels,_=load('/workspace/.aeterna-env/external5-data','test',dict(p,kind=kind))
    namespace={'mnist':12,'har':9,'fashion':4}[kind];answers=motors(kind)[1]
    domain=memory['consolidated_memory']['domains'][str(namespace)];g=domain['global']['features']
    rows=[r for r in memory['rows'] if r['pre'][0]==namespace and r['reward']>0]
    actions=np.array([r['action'] for r in rows]);x=np.array([r['pre'][2:] for r in rows],float)
    masks=np.array([[bool(r['pre'][1]&(1<<j)) for j in range(30)] for r in rows])
    means=np.zeros((16,30));variances=means.copy();known_var=means.copy()
    for a in range(16):
        f=domain['outcomes'][a]['features'];n=max(1,int((actions==a).sum()))
        for j in range(30):
            gv=g['m2'][j]/max(1,g['count'][j])
            means[a,j]=f['mean'][j] if f['count'][j] else g['mean'][j]
            cv=f['m2'][j]/f['count'][j] if f['count'][j] else gv
            variances[a,j]=max(cv,gv*.05,64.)
            known_var[a,j]=max(64.,variances[a,j]/math.sqrt(n))
    centers=np.where(masks,x,means[actions]);var=np.where(masks,known_var[actions],variances[actions]);norm=np.log(var)
    correct=guarded_correct=changed=eligible=0
    for i,label in enumerate(labels):
        frame,actual=frames[kind,i];_,mask,q=features(frame);j=np.array([bool(mask&(1<<k)) for k in range(30)])
        if not j.any():correct+=actual==answers[int(label)];guarded_correct+=actual==answers[int(label)];continue
        eligible+=1;dist=(q[j]-centers[:,j])**2/var[:,j];logs=-.5*(norm[:,j]+dist).sum(axis=1)
        scores=[]
        for a in answers:
            z=logs[actions==a];top=z.max();scores.append(top+np.log(np.exp(z-top).sum())-np.log(len(z)))
        scores=np.array(scores);best=int(np.argmax(scores));correct+=best==int(label);changed+=answers[best]!=actual
        probs=np.exp(scores-scores.max());probs/=probs.sum()
        cov=math.exp(-math.sqrt(float(dist[actions==answers[best]].sum(axis=1).min())/j.sum())/4);probs*=cov
        for k,a in enumerate(answers):
            key=json.dumps([[int(namespace),int(mask),*[int(v) for v in q]],a],separators=(',',':'))
            rate=memory['reward_rates'].get(key)
            if rate:probs[k]=rate['sum']/(rate['count']+rate.get('retired_failures',0))
            else:probs[k]/=1+failures(key)
        guarded_correct+=int(np.argmax(probs))==int(label)
    report['phases'][kind]={'samples':len(labels),'eligible_frames':eligible,'recorded_whole_agent_correct':sum(frames[kind,i][1]==answers[int(y)] for i,y in enumerate(labels)),
        'prototype_posterior_raw_correct':int(correct),'prototype_posterior_with_factual_guards_correct':int(guarded_correct),'changed_raw_answers':int(changed),
        'sensor_changes_and_U1_not_simulated':True}
path=old['ROOT']/'../.aeterna-env/partial-kernel-retrospective.json';path.write_text(json.dumps(report,indent=2)+'\n');print(path.read_text())
