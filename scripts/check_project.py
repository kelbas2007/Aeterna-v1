#!/usr/bin/env python3
"""Repeatable integrated regression suite; no external or one-use exams."""
import argparse,json,os,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def cargo_binary_path(env,release):
 """Read Cargo's actual target dir; never assume ROOT/target."""
 target=json.loads(subprocess.run(
  ['cargo','metadata','--locked','--no-deps','--format-version','1'],
  cwd=ROOT,env=env,check=True,capture_output=True,text=True
 ).stdout)['target_directory']
 return Path(target)/('release' if release else 'debug')/('aeterna-v1.exe' if os.name=='nt' else 'aeterna-v1')

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
 names=['project_entrypoint','recursive_search_policy','recursive_search_repair','experiential','learned_requirements','lc1_execution_contract','quantity_relations','word_problems','education_runtime','semantic_grounding','unified_runtime','unified_restore_guards','human_protection','u1_meta_control']+[p.stem for p in sorted((ROOT/'tests').glob('directed_construction*.rs'))]
 assert len(names)==len(set(names))
 cmd=['cargo','test',*flags]
 for name in names:cmd+=['--test',name]
 run(*cmd);run('cargo','test',*flags,'--example','rsp3_experience');run('cargo','build',*flags,'--bin','aeterna-v1','--bin','aeterna-unified')
 # Use the same Cargo configuration as the build, including CARGO_TARGET_DIR.
 binary=cargo_binary_path(env,a.release)
 run('python','scripts/lc1_joint_demo.py','--binary',str(binary),'--output',str(temp/'lc1_joint_demo.json'))
 print(json.dumps({'status':'PASS','suites':names,'fresh_intelligence_claim':False}))
if __name__=='__main__':main()
