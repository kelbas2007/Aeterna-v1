"""Execute a separately spawned binary and check exact learned-state roundtrip."""
import json,subprocess,sys,hashlib
from pathlib import Path
binary,trained=sys.argv[1:3]
state=Path(trained).read_text()
messages=[{'op':'load','checkpoint':state},{'op':'report'},{'op':'checkpoint'},{'op':'quit'}]
p=subprocess.run([binary],input=''.join(json.dumps(v)+'\n' for v in messages),capture_output=True,text=True,timeout=30,check=True)
replies=[json.loads(line) for line in p.stdout.splitlines()]
assert [r['type'] for r in replies]==['loaded','report','checkpoint','bye']
assert replies[0]['report']==replies[1]['report'], 'reading a report changed cognition'
assert replies[2]['checkpoint']==state
assert replies[1]['report']['learner']['events']==256
print(json.dumps({'status':'PASS','separate_process':True,'checkpoint_bit_exact':True,'report_read_inert':True,'events':256,'checkpoint_sha256':hashlib.sha256(state.encode()).hexdigest()}))
