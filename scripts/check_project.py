#!/usr/bin/env python3
"""Repeatable integrated regression suite; no external or one-use exams."""
import argparse,json,os,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]


def canonical_binary_from_artifacts(messages):
 """Use Cargo's reported executable, not a guessed target/<triple>/<profile>.

 An explicit --target, CARGO_BUILD_TARGET, .cargo/config.toml build.target,
 custom CARGO_TARGET_DIR, and profile changes all affect the filename Cargo
 actually emits. Every successful binary build reports its absolute path in
 a compiler-artifact event; cargo metadata's target_directory alone does not.
 """
 found=set()
 for message in messages:
  if message.get('reason')!='compiler-artifact':
   continue
  target=message.get('target') or {}
  if target.get('name')=='aeterna-v1' and 'bin' in target.get('kind',[]):
   executable=message.get('executable')
   if executable:
    found.add(Path(executable))
 if len(found)!=1:
  raise RuntimeError(f'Cargo did not report exactly one aeterna-v1 executable (found {len(found)})')
 return found.pop()


def build_and_locate_binary(env,flags):
 """Build and select the canonical binary from *that* build's JSON events."""
 cmd=['cargo','build',*flags,'--bin','aeterna-v1','--bin','aeterna-unified','--message-format=json']
 print('+',' '.join(cmd),flush=True)
 messages=[]
 with subprocess.Popen(cmd,cwd=ROOT,env=env,stdout=subprocess.PIPE,
                       text=True,bufsize=1) as process:
  for line in process.stdout:
   try:
    record=json.loads(line)
   except json.JSONDecodeError:
    print(line,end='',flush=True)
    continue
   if not isinstance(record,dict):
    continue
   if record.get('reason')=='compiler-artifact':
    messages.append(record)
   elif record.get('reason')=='compiler-message':
    diagnostic=record.get('message') or {}
    print(diagnostic.get('rendered') or diagnostic.get('message',''),file=sys.stderr,flush=True)
  returncode=process.wait()
 if returncode:
  raise subprocess.CalledProcessError(returncode,cmd)
 binary=canonical_binary_from_artifacts(messages)
 if not binary.is_file():
  raise FileNotFoundError(f'Cargo reported executable that does not exist: {binary}')
 return binary

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--release',action='store_true');p.add_argument('--offline',action='store_true');a=p.parse_args()
 manifest=json.loads((ROOT/'docs/PROJECT_STATE.json').read_text(encoding='utf-8'))
 assert manifest['canonical_binary']=='aeterna-v1' and manifest['recursive_admission']['new_states']==2
 for path in manifest['guides']:assert (ROOT/path).is_file(),path
 env=os.environ.copy();env.setdefault('CARGO_BUILD_JOBS','2');env['PYTHONUTF8']='1'
 temp=ROOT/'.test-temp';temp.mkdir(exist_ok=True)
 env['TEMP']=env['TMP']=env['TMPDIR']=str(temp)
 def run(*cmd):
  print('+',' '.join(cmd),flush=True);subprocess.run(cmd,cwd=ROOT,env=env,check=True)
 common=['--locked']+(['--offline'] if a.offline else [])
 run('python','scripts/test_check_project_target.py')
 run('cargo','check',*common,'--all-targets')
 flags=common+(['--release'] if a.release else [])
 run('cargo','test',*flags,'--lib','construction::')
 run('cargo','test',*flags,'--lib','learning_world::')
 run('cargo','test',*flags,'--lib','causal_continuity_credit_tests')
 # Automatically synthesized trajectory evaluator, including prospective gates.
 run('cargo','test',*flags,'--lib','autonomous_temporal_revision_tests')
 run('cargo','test',*flags,'--lib','organism_development_tests')
 run('cargo','test',*flags,'--lib','contextual_meta_tests')
 names=['causal_continual_autonomy','causal_jsonl_contract','project_entrypoint','recursive_search_policy','recursive_search_repair','experiential','learned_requirements','lc1_execution_contract','quantity_relations','word_problems','education_runtime','semantic_grounding','unified_runtime','unified_restore_guards','unified_self_directed_lifetime','trajectory_self_revision_protected','trajectory_self_revision_transfer','trajectory_behavior_gate','agency_choice_credit_development','cold_episodic_autonomy_development','instrumental_credit_development','agency_credit_manual_equivalence_development','cold_pair_probe_development','cold_autobiographical_option_development','cold_variable_depth_option_development','complexity_necessity_development','adaptive_history_suffix_development','predictive_history_action_development','predictive_history_state_development','certified_chain_protected_development','factual_transfer_development','factual_selective_holdout','human_protection','u1_meta_control']+[p.stem for p in sorted((ROOT/'tests').glob('directed_construction*.rs'))]
 assert len(names)==len(set(names))
 names.append('external_continual')
 names.append('organism_lifetime')
 names.append('learning_organism')
 cmd=['cargo','test',*flags]
 for name in names:cmd+=['--test',name]
 run(*cmd);run('cargo','test',*flags,'--example','rsp3_experience')
 # Consume the executable from the exact Cargo invocation, independent of
 # CARGO_TARGET_DIR, CARGO_BUILD_TARGET and .cargo/config build.target.
 binary=build_and_locate_binary(env,flags)
 run('python','scripts/check_saved_organism.py','--binary',str(binary))
 run('python','scripts/check_acquired_organism.py','--binary',str(binary))
 run('python','scripts/check_first_readout_organism.py','--binary',str(binary))
 run('python','scripts/lc1_joint_demo.py','--binary',str(binary),'--output',str(temp/'lc1_joint_demo.json'))
 run('python','scripts/unified_factual_autonomy_demo.py','--binary',str(binary),'--output',str(temp/'unscheduled_u1_loop.json'))
 print(json.dumps({'status':'PASS','suites':names,'fresh_intelligence_claim':False}))
if __name__=='__main__':main()
