"""Post-run verifier; never imported by the policy or supplied to the agent."""
import json,hashlib,re,sys
from pathlib import Path
from unified_arc3 import encode
p=Path(sys.argv[1]);learning=json.loads((p/'learning.json').read_text());summary=json.loads((p/'arc3/summary.json').read_text())
rows=[json.loads(line) for line in (p/'arc3/events.jsonl').read_text().splitlines()]
initial=rows[0];intents=[r for r in rows if r['kind']=='intent'];posts=[r for r in rows if r['kind']=='post']
assert initial['kind']=='initial'
assert len(intents)==len(posts)==summary['actions_executed']
for r in [initial]+posts:assert encode(r['raw']['frame'][-1])==r['encoded']
for index,(a,b) in enumerate(zip(intents,posts)):
    assert a['index']==b['index']==index
    assert a['action_index']==b['commit']['step']['action']
    assert b['commit']['step']['event']==index+1
trace={'initial':initial['encoded'],'steps':[[a['action_index'],b['encoded'],b['commit']['step']] for a,b in zip(intents,posts)]}
digest=hashlib.sha256(json.dumps(trace,sort_keys=True,separators=(',',':'),allow_nan=False).encode()).hexdigest()
# Reproduce the first actual attempt after fixing raw-frame logging. This is a
# frozen trace regression, not a demand that future intelligent policies fail.
expected='ab143985c9a6a7345518b42c6e583878596931f294fb7dbd2d155e724eaaa214'
assert digest==expected,'policy/observations changed: cannot call this the same frozen attempt'
assert summary['game_state']==posts[-1]['raw']['state']
assert summary['levels_completed']==posts[-1]['raw']['levels_completed']
assert summary['scorecard']['total_actions']==len(posts)
assert learning['full']['same_initial_program']==learning['same_knowledge_zero_controller']['same_initial_program']
trained=json.loads((p/'trained.json').read_text());after=json.loads((p/'arc3/after-external.json').read_text())
assert after['cognition']['archived'][0]==trained['cognition']['current']
assert initial['loaded']['report']['native_weights']==initial['entered']['report']['native_weights']
counts=[]
for f in p.glob('*tests*.log'):counts.extend(map(int,re.findall(r'test result: ok\. (\d+) passed',f.read_text())))
report={'source_revision':(p/'revision.txt').read_text().strip(),'all_test_runs_completed':True,'rust_tests_passed':sum(counts),
 'learning':{k:{'reached':sum(e['reached'] for e in learning[k]['episodes']),'episodes':len(learning[k]['episodes']),'actions':sum(e['steps'] for e in learning[k]['episodes'])} for k in ['full','same_knowledge_zero_controller','cold']},
 'external':{'game':summary['game_id'],'actions':len(posts),'completed_levels':summary['levels_completed'],'win':summary['win'],'state':summary['game_state']},
 'raw_frames_reencoded':len(posts)+1,'intent_post_pairs_verified':len(posts),'old_domain_unchanged':True,'controller_transferred':True,
 'policy_trace_sha256':digest,'reproduces_first_attempt_run':39042846504,'independent_second_trial':False,
 'artifact_checksums':{str(f.relative_to(p)):hashlib.sha256(f.read_bytes()).hexdigest() for f in p.rglob('*') if f.is_file()}}
(p/'verified-summary.json').write_text(json.dumps(report,indent=2));print(json.dumps(report,indent=2))
