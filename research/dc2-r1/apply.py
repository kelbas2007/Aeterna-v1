#!/usr/bin/env python3
"""Install the explicit R1 code on the measured DC-2 source; fail on drift."""
import hashlib,json,shutil,subprocess
from pathlib import Path
root=Path(__file__).resolve().parents[2]
subprocess.run(['git','diff','--exit-code'],cwd=root,check=True)
p=root/'src/construction/synthesis.rs'
s=p.read_text()
assert hashlib.sha256(p.read_bytes()).hexdigest()=='35c9cd62991ded318f4a4ee9f86d9283892188d8c211bd6b75d42ada9f2b6a0b', 'source drift'
old='    let mut enumeration = Enumerator::new(grammar,root);'
assert s.count(old)==1
s=s.replace(old,'    let mut enumeration = Enumerator::new(grammar,root.clone());',1)
a='    loop {\n        let expression = if let Some(e) = preferred.pop_front()'
b='''    if focused {
        let mut fast = Meter::new(meter.remaining()/2);
        let result = super::exact_completion::complete(&root,data,&states,job.target,parent.outputs.len(),cfg.max_expr_nodes,lib,&mut fast);
        meter.charge(fast.spent)?;
        match result {
            Ok(Some(e)) => preferred.push_front(e),
            Ok(None) | Err(Error::Budget) => {},
            Err(e) => return Err(e),
        }
    }
    loop {
        let expression = if let Some(e) = preferred.pop_front()'''
assert s.count(a)==1
p.write_text(s.replace(a,b))
p=root/'src/construction/mod.rs';s=p.read_text();assert 'mod exact_completion;' not in s;p.write_text(s+'mod exact_completion;\n')
shutil.copy(root/'research/dc2-r1/exact_completion.rs',root/'src/construction/exact_completion.rs')
shutil.copy(root/'research/dc2-r1/completion_tests.rs',root/'tests/directed_construction_completion.rs')
files=sorted(list(root.glob('src/construction/*.rs'))+list(root.glob('tests/directed_construction*.rs'))+list(root.glob('examples/directed_construction*.rs'))+[root/'Cargo.toml',root/'Cargo.lock'])
manifest={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in files}
(root/'dc2r1-source-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
