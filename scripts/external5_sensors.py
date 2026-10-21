"""Recorded real sensors, factual response feedback, and a matched 3NN control."""
from __future__ import annotations
import gzip, json, struct, zipfile
from collections import Counter
from pathlib import Path
import numpy as np

SENSOR_NAMES=[f'{sensor}_{axis}' for sensor in ['body_acc','body_gyro','total_acc'] for axis in ['x','y','z']]

def load_fashion(root,part,offset,count):
    prefix='train' if part=='train' else 't10k'
    images=gzip.decompress((Path(root)/f'{prefix}-images-idx3-ubyte.gz').read_bytes())
    labels=gzip.decompress((Path(root)/f'{prefix}-labels-idx1-ubyte.gz').read_bytes())
    magic,n,h,w=struct.unpack('>IIII',images[:16]);label_magic,m=struct.unpack('>II',labels[:8])
    if (magic,label_magic,h,w)!=(2051,2049,28,28) or n!=m or len(images)!=16+n*h*w or len(labels)!=8+n:
        raise ValueError('invalid official Fashion-MNIST IDX files')
    if offset+count>n:raise ValueError('Fashion range outside partition')
    x=np.frombuffer(images,dtype=np.uint8,offset=16).reshape(n,h,w)[offset:offset+count].copy()
    y=np.frombuffer(labels,dtype=np.uint8,offset=8)[offset:offset+count].astype(int)
    if np.any((y<0)|(y>=10)):raise ValueError('invalid Fashion class')
    return x,y,None

def load_har(root,part,offset,count):
    prefix=f'UCI HAR Dataset/{part}/'
    with zipfile.ZipFile(Path(root)/'uci-har.zip') as archive:
        channels=[]
        for sensor in SENSOR_NAMES:
            with archive.open(prefix+f'Inertial Signals/{sensor}_{part}.txt') as f:
                channels.append(np.loadtxt(f,max_rows=offset+count,ndmin=2)[offset:offset+count])
        with archive.open(prefix+f'y_{part}.txt') as f:y=np.loadtxt(f,dtype=int,converters=int,max_rows=offset+count,ndmin=1)[offset:offset+count]-1
        with archive.open(prefix+f'subject_{part}.txt') as f:subjects=np.loadtxt(f,dtype=int,converters=int,max_rows=offset+count,ndmin=1)[offset:offset+count]
    x=np.stack(channels,axis=1)
    if (x.shape!=(count,9,128) or y.shape!=(count,) or subjects.shape!=(count,)
        or not np.isfinite(x).all() or np.any((y<0)|(y>=6)) or np.any((subjects<1)|(subjects>30))):
        raise ValueError('invalid UCI HAR raw partition')
    return x,y,subjects

def blank(kind):
    if kind=='fashion':return {'kind':'image','namespace':1,'pixels':None}
    return {'kind':'signals','namespace':2,'channels':[None]*9}

def acquire(frame,index,sample):
    if frame['kind']=='image':return {'kind':'image','namespace':1,'pixels':sample.tolist()}
    channels=list(frame['channels']);channels[index]=sample[index].tolist()
    return {'kind':'signals','namespace':2,'channels':channels}

def features(frame):
    x=np.zeros(30);mask=0
    if frame['kind']=='image':
        namespace=frame['namespace']*4
        if frame['pixels'] is not None:
            p=np.asarray(frame['pixels'],dtype=np.int64);h,w=p.shape
            for by in range(5):
                for bx in range(6):
                    patch=p[by*h//5:(by+1)*h//5,bx*w//6:(bx+1)*w//6]
                    x[by*6+bx]=int(patch.sum())*1024//(255*patch.size)
            mask=(1<<30)-1
    else:
        namespace=frame['namespace']*4+1
        for i,channel in enumerate(frame['channels']):
            if channel is None:continue
            values=np.asarray(channel,dtype=float)
            for j,value in enumerate([values.mean(),values.std(),np.abs(np.diff(values)).mean() if len(values)>1 else 0.0]):
                scaled=float(value)*1024
                x[3*i+j]=np.floor(scaled+0.5) if scaled>=0 else np.ceil(scaled-0.5)
                mask|=1<<(3*i+j)
    return namespace,mask,x

class MatchedKNN:
    """Learns only successful factual answer motors and actually measured axes."""
    def __init__(self):self.rows={};self.metrics={}
    def observe(self,frame):
        namespace,mask,x=features(frame)
        n,mean,m2=self.metrics.setdefault(namespace,(np.zeros(30),np.zeros(30),np.zeros(30)))
        for j in range(30):
            if mask&(1<<j):
                n[j]+=1;d=x[j]-mean[j];mean[j]+=d/n[j];m2[j]+=d*(x[j]-mean[j])
    def remember(self,frame,motor):
        namespace,mask,x=features(frame);self.rows.setdefault(namespace,[]).append((mask,x,motor))
    def predict(self,frame,answers):
        namespace,mask,x=features(frame);rows=self.rows.get(namespace,[])
        if not rows:return min(answers)
        eligible=[row for row in rows if row[0]&mask==mask and row[2] in answers]
        if not eligible or not mask:return Counter(r[2] for r in (eligible or rows)).most_common(1)[0][0]
        n,mean,m2=self.metrics[namespace];scale=np.maximum(np.sqrt(np.maximum(m2/np.maximum(n,1),0.0)),8.0)
        axes=np.array([bool(mask&(1<<j)) for j in range(30)])
        matrix=np.stack([r[1] for r in eligible]);distance=(((matrix[:,axes]-x[axes])/scale[axes])**2).mean(axis=1)
        nearest=np.argsort(distance,kind='stable')[:3];votes=Counter(eligible[i][2] for i in nearest)
        return max(votes,key=lambda a:(votes[a],-min(distance[i] for i in nearest if eligible[i][2]==a),-a))
    def report(self):
        return {'training_examples':sum(map(len,self.rows.values())),
                'feature_array_bytes':sum(row[1].nbytes+16 for rows in self.rows.values() for row in rows)}

def metrics(targets,predictions,classes):
    confusion=np.zeros((classes,classes),dtype=int)
    for true,pred in zip(targets,predictions):confusion[int(true),int(pred)]+=1
    support=confusion.sum(axis=1);chosen=confusion.sum(axis=0);diag=confusion.diagonal()
    recall=np.divide(diag,support,out=np.zeros(classes,dtype=float),where=support!=0)
    f1=np.divide(2*diag,support+chosen,out=np.zeros(classes,dtype=float),where=support+chosen!=0)
    return {'samples':len(targets),'correct':int(diag.sum()),'accuracy':float(diag.sum()/max(len(targets),1)),
            'balanced_accuracy':float(recall[support>0].mean()),'macro_f1':float(f1.mean()),
            'class_support':support.tolist(),'confusion_matrix':confusion.tolist()}

def run_windows(agent,kind,x,y,subjects,motors,training,paired=None):
    queries,answers=motors;query_lookup={m:i for i,m in enumerate(queries)};answer_lookup={m:i for i,m in enumerate(answers)}
    limit=(11 if kind=='fashion' else 15) if training else (2 if kind=='fashion' else 5)
    predictions=[];paired_predictions=[];measurement_counts=[];action_count=0;train_solved=0
    for index,(sample,target) in enumerate(zip(x,y)):
        frame=blank(kind);agent.reset(frame,dataset=kind,index=index,training=training)
        measured=set();failed=set();prediction=None
        for left in range(limit,0,-1):
            available=[False]*16
            for motor,axis in query_lookup.items():available[motor]=axis not in measured and left>1
            for motor in answers:available[motor]=motor not in failed and (kind!='fashion' or bool(measured))
            before=frame
            def execute(action):
                nonlocal frame,prediction
                if action in query_lookup:
                    axis=query_lookup[action];measured.add(axis);frame=acquire(frame,axis,sample);reward=0.0
                else:
                    prediction=answer_lookup[action]
                    if paired is not None and not training:paired_predictions.append(answer_lookup[paired.predict(frame,answers)])
                    reward=float(prediction==int(target))
                    if training and not reward:failed.add(action)
                return {'frame':frame,'cost':1,'reward':reward}
            action,result=agent.step(available,left,execute);action_count+=1
            if paired is not None and training:
                paired.observe(before);paired.observe(frame)
                if result['reward']>0:paired.remember(before,action)
            if action in answer_lookup:
                # Every answer closes its reward episode. Corrective training
                # retries retain readings, without retroactive choice credit.
                agent.finish()
                if training and not result['reward'] and left>1:
                    agent.reset(frame,dataset=kind,index=index,corrective_retry=True)
            if action in answer_lookup and (not training or result['reward']>0):break
        agent.finish();measurement_counts.append(len(measured))
        if prediction is None:raise RuntimeError('episode ended without a response')
        predictions.append(prediction);train_solved+=int(prediction==int(target))
        agent.trace.write({'kind':'graded_episode','dataset':kind,'index':index,'training':training,
            'prediction':prediction,'target_after_response':int(target),'measurements':len(measured)})
        if (index+1)%100==0:print(json.dumps({'phase':'train' if training else 'score','variant':agent.variant,
            'dataset':kind,'windows':index+1,'correct':train_solved,'actions':action_count}),flush=True)
    result=metrics(y,predictions,10 if kind=='fashion' else 6)
    result.update({'actions':action_count,'measurement_actions':sum(measurement_counts),
        'mean_measured_channels':float(np.mean(measurement_counts)),
        'subjects':sorted(set(map(int,subjects))) if subjects is not None else None,'maximum_actions_per_window':limit})
    if paired is not None and not training:result['matched_3nn']=metrics(y,paired_predictions,10 if kind=='fashion' else 6)
    return result
