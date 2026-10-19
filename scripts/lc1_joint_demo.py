#!/usr/bin/env python3
"""Repeatable joint-state LC1/RSP2 demo. No external qualification claim."""
import argparse,json,subprocess,hashlib
from pathlib import Path

def task(n,start):
    rows=[]
    for i in range(n):
        t=i+start
        a,b,c,d=(t*11%29)-8,(t*7%31)-12,(t*13%23)-6,(t*19%17)-5
        rows.append({'input':[a,b,c,d],'output':(a+b)-(c*d)})
    return {'examples':rows,'library':{'functions':[]}}

class Worker:
    def __init__(self,binary):
        self.p=subprocess.Popen([str(binary),'--jsonl'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True,encoding='utf-8')
    def ask(self,msg):
        self.p.stdin.write(json.dumps(msg,ensure_ascii=False)+'\n');self.p.stdin.flush()
        line=self.p.stdout.readline();assert line,('early exit',self.p.poll(),msg['op'])
        answer=json.loads(line)
        assert answer.get('type')!='error',answer
        return answer
    def close(self):
        self.ask({'op':'quit'});self.p.stdin.close();self.p.wait(timeout=5);assert self.p.returncode==0

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--binary',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);a=ap.parse_args()
    a.output.parent.mkdir(parents=True,exist_ok=True)
    c=Worker(a.binary.resolve());c.ask({'op':'init'})
    lim={'work':100000,'evaluations':30,'max_nodes':7,'frontier':64,'edits_per_expansion':8}
    c.ask({'op':'enable_recursive_search','config':{'object':lim,'meta':lim,'max_lifetime_work':1000000}})
    c.ask({'op':'enable_missing_computation_learning'})
    obs=[{'op':'Sub','left':(i*11%83)-37,'right':(i*17%59)-24} for i in range(35)]
    for r in obs:r['output']=r['left']-r['right']
    learned=c.ask({'op':'observe_executed_computations','observations':obs})
    assert learned['report']['learned_rules']==1
    limits={'work':3000000,'nodes':7,'terms':4096}
    original=c.ask({'op':'complete_missing_computation','task':task(22,5),'limits':limits})['result']
    assert original['forward_verified'] and original['used_acquired_inverse']
    cp=c.ask({'op':'checkpoint'})['checkpoint'];c.close()
    r=Worker(a.binary.resolve());r.ask({'op':'load','checkpoint':cp})
    assert r.ask({'op':'missing_computation_report'})['report']['learned_rules']==1
    r2=r.ask({'op':'complete_missing_computation','task':task(22,5),'limits':limits})['result']
    assert r2['program']==original['program']
    # Held-out rows never reach the completion call.
    ps=task(18,100)['examples']
    # The program is independently executed by a separate Rust test; the JSONL
    # demonstration explicitly records only observed training fit and replay.
    r.ask({'op':'freeze'})
    frozen=r.ask({'op':'checkpoint'})['checkpoint'];r.close()
    summary={'kind':'LC1_UNIFIED_JOINT_EXAMPLE_NOT_EXTERNAL_EXAM','acquired_rules':learned['report']['learned_rules'],
             'training_observations':len(obs),'target_support':22,'heldout_probe_not_passed_to_solver':len(ps),
             'learning_work':learned['report']['work'],'completion_work':original['work'],
             'automatic_hypotheses_tried':original['tried_rules'],'forward_verified_support':original['forward_verified'],
             'restored_program_byte_identical':r2['program']==original['program'],
             'joint_checkpoint_fingerprint':hashlib.sha256(cp.encode()).hexdigest(),
             'frozen_joint_checkpoint_fingerprint':hashlib.sha256(frozen.encode()).hexdigest(),
             'rsp2_policy_generation':0,'rsp2_admitted_new_policy':False,'human_level_intelligence':False}
    a.output.write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(summary,ensure_ascii=False,indent=2))

if __name__=='__main__':main()
