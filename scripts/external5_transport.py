"""Factual JSONL transport. Policy remains inside the canonical Rust executable."""
from __future__ import annotations
import gzip, hashlib, json, os, queue, random, subprocess, threading
from pathlib import Path

def compact(value):return json.dumps(value,separators=(',',':'),allow_nan=False)

class Trace:
    def __init__(self,path):
        self.path=Path(path);self.path.parent.mkdir(parents=True,exist_ok=True)
        if self.path.exists():raise FileExistsError(self.path)
        self.file=gzip.open(self.path,'xt',encoding='utf-8');self.frames=set();self.events=0
    def write(self,value,flush=False):
        self.file.write(compact(value)+'\n');self.events+=1
        if flush:self.file.flush();os.fsync(self.file.fileno())
    def frame(self,value):
        encoded=compact(value).encode();key=hashlib.sha256(encoded).hexdigest()
        if key not in self.frames:
            self.write({'kind':'sensor_frame','id':key,'frame':value});self.frames.add(key)
        return key
    def close(self):self.file.close()

class Native:
    def __init__(self,binary,path,variant='full',frame=None,checkpoint=None):
        self.variant=variant;self.trace=Trace(path);self.actions=0;self.frame=None
        self.stderr=Path(str(path)+'.stderr').open('w')
        self.child=subprocess.Popen([str(Path(binary).resolve()),'--jsonl'],stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,stderr=self.stderr,text=True,bufsize=1)
        self.mail=queue.Queue()
        def reader():
            try:
                while True:
                    line=self.child.stdout.readline(66*1024*1024)
                    self.mail.put(line)
                    if not line:return
            except Exception as error:self.mail.put(error)
        threading.Thread(target=reader,daemon=True).start()
        try:
            if checkpoint is not None:
                self.command({'op':'load','checkpoint':checkpoint})
                if variant.startswith('organism'):
                    self.trace.write({'kind':'loaded','checkpoint_sha256':hashlib.sha256(checkpoint.encode()).hexdigest(),'variant':variant})
            else:
                self.command({'op':'organism_init' if variant.startswith('organism') else 'external_init',
                    'frame':frame,'reward_memory':variant!='legacy','recursive':variant!='organism_fixed',
                    'acquired_rules':variant not in ('organism_rules_fixed','organism_previous'),
                    'contextual_u1':variant!='organism_previous',
                    'adaptive':variant=='full','planning_depth':1 if variant=='one_step' else 32})
                self.frame=frame;self.trace.write({'kind':'initial','frame_ref':self.trace.frame(frame),'variant':variant})
        except BaseException:
            if self.child.poll() is None:self.child.kill()
            self.child.wait();self.stderr.close();self.trace.close();raise
    def send(self,value):self.child.stdin.write(compact(value)+'\n');self.child.stdin.flush()
    def receive(self):
        try:line=self.mail.get(timeout=120)
        except queue.Empty:raise TimeoutError('native transport stalled; never retry an unknown action')
        if isinstance(line,Exception):raise line
        if not line or not line.endswith('\n'):raise RuntimeError('native transport closed or exceeded line limit')
        value=json.loads(line)
        if value.get('type')=='error':raise RuntimeError('native: '+value.get('message','unknown'))
        return value
    def command(self,value):self.send(value);return self.receive()
    def reset(self,frame,**metadata):
        result=self.command({'op':'external_reset','frame':frame});self.frame=frame
        self.trace.write({'kind':'restart','frame_ref':self.trace.frame(frame),'metadata':metadata});return result
    def step(self,available,left,execute):
        offers=[{'available':bool(v),'max_cost':1} for v in available]
        self.send({'op':'external_step','offers':offers,'remaining_actions':int(left)})
        intent=self.receive()
        if intent.get('type')!='action':raise RuntimeError('no protected motor intention')
        action=int(intent['action'])
        if action not in range(16) or not available[action]:raise RuntimeError('unavailable motor selected')
        self.trace.write({'kind':'intent','index':self.actions,'action':action,'remaining_actions':left,
            'offers':offers,'pre_ref':self.trace.frame(self.frame)},flush=True)
        # The actual action happens exactly once after the durable intention.
        result=execute(action);self.actions+=1
        self.trace.write({'kind':'actual_post','index':self.actions-1,'action':action,
            'frame_ref':self.trace.frame(result['frame']),'cost':result['cost'],'reward':result['reward']},flush=True)
        self.send({'op':'post','result':result});commit=self.receive()
        if commit.get('type')!='committed':raise RuntimeError('actual result not committed')
        self.frame=result['frame'];self.trace.write({'kind':'commit','index':self.actions-1,'step':commit['step']})
        return action,result
    def finish(self):return self.command({'op':'external_episode_end'})
    def checkpoint(self):return self.command({'op':'checkpoint'})['checkpoint']
    def freeze(self):return self.command({'op':'freeze'})
    def report(self):return self.command({'op':'external_reward_report'})['report']
    def close(self):
        try:
            if self.child.poll() is None:self.command({'op':'quit'});self.child.wait(timeout=10)
        except Exception:
            # Closing an already faulted transport must not replace the
            # original unknown-outcome exception or issue another action.
            pass
        finally:
            if self.child.poll() is None:self.child.kill();self.child.wait()
            self.child.stdin.close();self.child.stdout.close();self.trace.close();self.stderr.close()

class RandomReference:
    """Separate uniform control for virtual/recorded tasks, never an AETERNA policy."""
    variant='random'
    def __init__(self,path,seed=90261020):self.trace=Trace(path);self.rng=random.Random(seed);self.actions=0;self.frame=None
    def reset(self,frame,**metadata):
        self.frame=frame;self.trace.write({'kind':'restart','frame_ref':self.trace.frame(frame),'metadata':metadata})
    def step(self,available,left,execute):
        action=self.rng.choice([a for a,v in enumerate(available) if v])
        self.trace.write({'kind':'intent','index':self.actions,'action':action,'remaining_actions':left,
            'offers':[{'available':bool(v),'max_cost':1} for v in available],
            'pre_ref':self.trace.frame(self.frame),'controller':'uniform_reference'},flush=True)
        result=execute(action);self.actions+=1;self.frame=result['frame']
        self.trace.write({'kind':'actual_post','index':self.actions-1,'action':action,'frame_ref':self.trace.frame(self.frame),
            'reward':result['reward'],'cost':result['cost']},flush=True)
        return action,result
    def finish(self):pass
    def report(self):return {'enabled':False,'controller':'uniform_reference','actions':self.actions}
    def close(self):self.trace.close()

def new_agent(binary,path,variant,frame,checkpoint=None):
    if variant=='random':
        agent=RandomReference(path);agent.reset(frame);return agent
    return Native(binary,path,variant,frame,checkpoint)
