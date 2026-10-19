#!/usr/bin/env python3
"""Regression: always use Cargo's actual executable path, including target triples."""
import io,json,os,subprocess,tempfile
from pathlib import Path
from unittest.mock import patch

from check_project import build_and_locate_binary, canonical_binary_from_artifacts

with tempfile.TemporaryDirectory(prefix='aeterna-cargo-build-artifacts-') as folder:
 root=Path(folder)
 name='aeterna-v1.exe' if os.name=='nt' else 'aeterna-v1'
 for triple,profile in [('', 'debug'),('x86_64-unknown-linux-gnu','debug'),
                        ('x86_64-pc-windows-msvc','release')]:
  executable=root / (triple or '.') / profile / name
  executable.parent.mkdir(parents=True,exist_ok=True)
  executable.write_bytes(b'fixture')
  records=[
   {'reason':'compiler-artifact', 'target':{'name':'aeterna-unified','kind':['bin']},'executable':str(executable.with_name('aeterna-unified'))},
   {'reason':'compiler-artifact', 'target':{'name':'some-dependency','kind':['lib']},'executable':None},
   {'reason':'compiler-artifact', 'target':{'name':'aeterna-v1','kind':['bin']},'executable':str(executable)},
   {'reason':'build-finished','success':True},
  ]
  assert canonical_binary_from_artifacts(records)==executable
  with patch('check_project.subprocess.Popen') as popen:
   instance=popen.return_value.__enter__.return_value
   instance.stdout=io.StringIO('\n'.join(map(json.dumps,records))+'\n')
   instance.wait.return_value=0
   env={'CARGO_TARGET_DIR':str(root),'CARGO_BUILD_TARGET':triple}
   selected=build_and_locate_binary(env,['--locked','--release' if profile=='release' else '--offline'])
   assert selected==executable,(selected,executable)
   argv=popen.call_args.args[0]
   assert '--message-format=json' in argv and '--bin' in argv
 try:
  canonical_binary_from_artifacts([])
  assert False,'Missing artifacts must fail closed'
 except RuntimeError: pass
 with patch('check_project.subprocess.Popen') as popen:
  instance=popen.return_value.__enter__.return_value
  instance.stdout=io.StringIO('')
  instance.wait.return_value=101
  try:
   build_and_locate_binary({},['--locked'])
   assert False,'Cargo build failure must propagate'
  except subprocess.CalledProcessError: pass
print('PASS: Cargo artifact selects canonical binary under custom target directory, explicit target triples and both profiles')
