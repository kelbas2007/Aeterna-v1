#!/usr/bin/env python3
"""Regression: honor Cargo's effective build directory for the joint demo."""
import os, tempfile
from pathlib import Path

from check_project import cargo_binary_path
with tempfile.TemporaryDirectory(prefix="lc1-custom-cargo-target-") as folder:
    env=os.environ.copy()
    env['CARGO_TARGET_DIR']=folder
    env['CARGO_BUILD_JOBS']='2'
    debug=cargo_binary_path(env,False)
    release=cargo_binary_path(env,True)
    assert debug.parent.parent.resolve()==Path(folder).resolve()
    assert release.parent.parent.resolve()==Path(folder).resolve()
    assert debug.parent.name=='debug' and release.parent.name=='release'
    assert debug.name=='aeterna-v1.exe' if os.name=='nt' else debug.name=='aeterna-v1'
print('PASS: custom CARGO_TARGET_DIR resolves to Cargo metadata output')
