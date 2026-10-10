#!/usr/bin/env python3
"""One generic native rule learner on recorded images/signals.

Labels stay in this evaluator. Rust receives binary-serialized measurements,
selects an opaque motor, then receives the actual success reward. This is open
recorded-data validation, not evidence of image-to-signal knowledge transfer.
"""
import argparse
import csv
import gzip
import hashlib
import json
import random
import subprocess
from collections import Counter
from pathlib import Path

from external_minigrid_benchmark import Agent
from external_relational_hypothesis_pairs import checked_command

ROOT = Path(__file__).resolve().parents[1]


class RecordedAgent(Agent):
    def __init__(self, binary, dimension, motors):
        self.proc = subprocess.Popen([str(binary)], stdin=subprocess.PIPE,
            stdout=subprocess.PIPE, text=True, bufsize=1)
        self.write({"cmd":"init", "dimension":dimension, "motors":motors})
        if self.read().get("type") != "ready":
            raise RuntimeError("native init failed")

    def write(self, message):
        fields={"init":{"cmd","dimension","motors"},
                "reset":{"cmd","observation","learning"},
                "post":{"cmd","observation","reward"}}
        commands={"advance","restart","status","quit","general_policy",
                  "context_value_learning","value_abstraction",
                  "value_abstraction_lesion","value_predicates"}
        if set(message)!=fields.get(message.get("cmd"),
                {"cmd"} if message.get("cmd") in commands else set()):
            raise ValueError("nonpublic controller field")
        Agent.write(self,message)

    def observation(self, bits, learning):
        self.write({"cmd":"reset","observation":bits,"learning":learning})
        if self.read().get("type")!="reset_ack":raise RuntimeError("reset refused")

    def trial(self, bits, correct):
        self.write({"cmd":"advance"})
        choice=self.read()
        if choice.get("type")!="action":raise RuntimeError(f"protected motor failed: {choice}")
        action=choice["action"]
        reward=int(action==correct)
        self.write({"cmd":"post","observation":bits,"reward":reward})
        ack=self.read()
        if ack.get("type")!="advance_ack" or ack.get("executed")!=action:
            raise RuntimeError("factual trial not confirmed")
        return action,reward


def encode(values, bits):
    return [(v>>bit)&1 for v in values for bit in range(bits)]


def records(corpus):
    if corpus=="digits":
        counts=Counter();train=[];test=[]
        for index,row in enumerate(csv.reader((ROOT/"tests/data/digits.csv").open())):
            values=list(map(int,row));label=values.pop()
            target=test if counts[label]%5==0 else train
            target.append({"id":index,"numeric":values,"label":label,"bits":encode(values,5)})
            counts[label]+=1
        random.Random(198000).shuffle(train)
        counts=Counter();chosen=[]
        for r in train:
            if counts[r["label"]]<32:chosen.append(r);counts[r["label"]]+=1
        return chosen,test,10,12,20
    def read(path):
        out=[];data=False
        for line in path.read_text().splitlines():
            if line.lower()=="@data":data=True;continue
            if not data or not line or line.startswith('#'):continue
            text,label=line.rsplit(':',1);values=list(map(float,text.split(',')))
            # Fixed measurement serialization, not extracted signal features.
            quantized=[round(max(0.0,min(1.0,(v+4.0)/8.0))*63) for v in values]
            out.append({"id":len(out),"numeric":values,"label":int(label)-1,
                        "bits":encode(quantized,6)})
        return out
    return read(ROOT/"tests/data/GunPoint_TRAIN.ts"),read(ROOT/"tests/data/GunPoint_TEST.ts"),2,16,8


def run(binary, corpus, train_only=False):
    train,test,classes,passes,cap=records(corpus)
    roles=list(range(classes));random.Random(198001).shuffle(roles)
    agent=RecordedAgent(binary,len(train[0]["bits"]),classes)
    report={"corpus":corpus,"training_records":len(train),"training_passes":passes,
            "attempt_cap":cap,"roles_evaluator_only":roles,"training":[],"evaluations":{}}
    prototypes={}
    try:
        for command in ["general_policy","context_value_learning","value_abstraction"]:
            checked_command(agent,command,command+"_ack")
        for epoch in range(passes):
            for r in train:
                agent.observation(r["bits"],True)
                actions=[];success=0
                for _ in range(cap):
                    action,success=agent.trial(r["bits"],roles[r["label"]]);actions.append(action)
                    if success:prototypes[r["id"]]=(r["numeric"],action);break
                report["training"].append({"id":r["id"],"actions":actions,"success":bool(success)})
            print(json.dumps({"corpus":corpus,"epoch":epoch+1,"measured_labels":len(prototypes),
                              "rules":agent.status()["value_abstraction"]}),flush=True)
        if train_only:
            report["status"]="training-only diagnostic; no test scored"
            return report
        for arm in ["rules","lesioned"]:
            if arm=="lesioned":checked_command(agent,"value_abstraction_lesion","value_abstraction_lesion_ack")
            before=agent.status();out=[]
            for r in test:
                checked_command(agent,"restart","restart_ack")
                agent.observation(r["bits"],False)
                action,success=agent.trial(r["bits"],roles[r["label"]])
                distances=sorted((sum((x-y)**2 for x,y in zip(r["numeric"],p)),a)
                                 for p,a in prototypes.values())
                nearest=Counter(a for _,a in distances[:3]).most_common(1)[0][0]
                out.append({"id":r["id"],"input_sha256":hashlib.sha256(bytes(r["bits"])).hexdigest(),
                            "action":action,"success":bool(success),"knn_success":nearest==roles[r["label"]]})
            after=agent.status()
            for field in ["general_updates","general_rewards","value_states","value_abstraction"]:
                if before[field]!=after[field]:raise ValueError("frozen learned metadata changed")
            report["evaluations"][arm]={"before":before,"after":after,"records":out,
                "successes":sum(x["success"] for x in out),"knn_successes":sum(x["knn_success"] for x in out)}
            print(json.dumps({"corpus":corpus,"arm":arm,"successes":sum(x["success"] for x in out),
                              "records":len(out),"knn":sum(x["knn_success"] for x in out)}),flush=True)
        agent.write({"cmd":"value_predicates"});report["predicates"]=agent.read()
        return report
    finally:agent.close()


def main():
    parser=argparse.ArgumentParser();parser.add_argument("--agent",type=Path,required=True)
    parser.add_argument("--corpus",choices=["digits","gunpoint"],required=True)
    parser.add_argument("--output",type=Path,required=True);parser.add_argument("--train-only",action="store_true")
    args=parser.parse_args()
    if not args.train_only and subprocess.check_output(["git","status","--porcelain","--","src","Cargo.toml","Cargo.lock"],cwd=ROOT,text=True):
        raise SystemExit("Freeze cognitive source before recorded evaluation")
    report={"source":subprocess.check_output(["git","rev-parse","HEAD"],cwd=ROOT,text=True).strip(),
            "binary_sha256":hashlib.sha256(args.agent.read_bytes()).hexdigest(),
            "fixture_sha256":{name:hashlib.sha256((ROOT/"tests/data"/name).read_bytes()).hexdigest()
                for name in (["digits.csv"] if args.corpus=="digits" else ["GunPoint_TRAIN.ts","GunPoint_TEST.ts"])},
            "status":"open recorded-data validation, not independent qualification",**run(args.agent,args.corpus,args.train_only)}
    if not args.train_only:
        learned=report['evaluations']['rules'];control=report['evaluations']['lesioned']
        n=len(learned['records']);required=0.75 if args.corpus=='digits' else 0.80
        report['verdict']='RECORDED_RULE_DEVELOPMENT_PASS' if learned['successes']/n>=required and (
            learned['successes']-control['successes'])/n>=0.20 else 'RECORDED_RULE_DEVELOPMENT_FAIL'
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_bytes(gzip.compress((json.dumps(report,sort_keys=True)+"\n").encode(),mtime=0))


if __name__=="__main__":main()
