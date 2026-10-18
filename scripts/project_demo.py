#!/usr/bin/env python3
"""Small acquired state through one process. Engineering example, not a benchmark."""
import argparse,hashlib,json,subprocess
from pathlib import Path

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
 a.output.mkdir(parents=True,exist_ok=False)
 log=open(a.output/'process.log','w',encoding='utf-8')
 proc=subprocess.Popen([str(a.binary.resolve()),'--jsonl'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=log,text=True,encoding='utf-8',bufsize=1)
 trace=[]
 def request(message):
  proc.stdin.write(json.dumps(message,ensure_ascii=False)+'\n');proc.stdin.flush()
  line=proc.stdout.readline()
  if not line:raise RuntimeError('native process closed before response')
  v=json.loads(line)
  if v.get('type')=='error':raise RuntimeError(v.get('message'))
  return v
 try:
  request({'op':'init'})
  # The host supplies factual consequences of virtual commands, not instructions
  # selecting the commands. This uses the existing public protected step path.
  x=0
  for _ in range(16):
   act=request({'op':'step','available':[True]*4})
   assert act['type']=='action'
   x+=act['action']
   commit=request({'op':'post','result':{'observation':[x]+[0]*31,'cost':1,'reward':0.0}})
   assert commit['type']=='committed';trace.append(commit)
  examples=[]
  for i in range(32):
   x,y=3+i,2+(i*7)%29
   examples.append({'question':f'Найди сумму {x} и {y}?','answer':x+y,'source':f'project-teacher:{i}'})
  lesson=request({'op':'teach','examples':examples});assert lesson['result']['accepted']
  limits={'work':30000,'evaluations':64,'max_nodes':11,'frontier':64,'edits_per_expansion':12}
  request({'op':'enable_recursive_search','config':{'object':limits,'meta':limits,'max_lifetime_work':1000000}})
  request({'op':'freeze_recursive_search'})
  request({'op':'freeze'})
  before=request({'op':'checkpoint'})['checkpoint']
  answer=request({'op':'ask','question':'Найди сумму 137 и 58?','mode':'ComputeOnly'})
  assert answer['result']['answer']=='195'
  assert request({'op':'checkpoint'})['checkpoint']==before
  report=request({'op':'report'});rsp=request({'op':'recursive_search_report'})
  request({'op':'quit'});proc.wait(timeout=30);assert proc.returncode==0
 finally:
  if proc.poll() is None:proc.kill();proc.wait()
  log.close()
 state=a.output/'example-state.json';state.write_text(before,encoding='utf-8')
 inspection=subprocess.run([str(a.binary.resolve()),'--inspect',str(state)],capture_output=True,text=True,encoding='utf-8',check=True)
 inspected=json.loads(inspection.stdout)
 assert inspected['recursive_search']['generation']==0 and inspected['organism']['learner']['events']==16
 assert state.read_text(encoding='utf-8')==before
 summary={'status':'PASS','kind':'one_process_engineering_example','virtual_actions':16,'teacher_examples':32,'answer':answer['result'],
          'organism':report['report'],'recursive_generation':rsp['state']['generation'],
          'rsp2_initialized_not_self_improved':True,'checkpoint_sha256':hashlib.sha256(state.read_bytes()).hexdigest(),
          'separate_process_inspection':True,'fresh_external_intelligence_result':False}
 (a.output/'summary.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
 (a.output/'events.json').write_text(json.dumps(trace,indent=2)+'\n',encoding='utf-8')
 print(json.dumps(summary,ensure_ascii=False))
if __name__=='__main__':main()
