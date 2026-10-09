#!/usr/bin/env python3
"""Exact integration in the CI checkout; publish only after passing tests.

Historical open failures are preserved: 37966660011 admitted no calls;
37967069025 admitted one rather than both outcome calls. The mechanism
scored 96/96 in both. Revised binding inference uses only prefix evidence,
revokes contradicted mappings, and independently validates later outcomes.
The test's tasks, training budget and expected scores remain unchanged.
"""
from pathlib import Path
import runpy
import shutil

ROOT = Path(__file__).resolve().parents[1]


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if text.count(old) != 1:
        raise RuntimeError(f"Unexpected source at {label}: {text.count(old)} matches")
    return text.replace(old, new, 1)


def main() -> None:
    primitive_path = ROOT / 'src/phase_primitives.rs'
    induction_path = ROOT / 'src/phase_induction.rs'
    primitives = primitive_path.read_text()
    induction = induction_path.read_text()
    if 'include!("phase_primitive_arguments.rs");' in primitives:
        raise RuntimeError('Integration already applied; do not reapply')
    primitives = replace_once(primitives,
        '    policy_updates: u64,\n}',
        '    policy_updates: u64,\n    #[serde(default)]\n    argument_transfer_enabled: bool,\n}',
        'primitive state')
    primitives = replace_once(primitives,
        '        feed(self.policy_updates);',
        '        feed(self.policy_updates);\n        if self.argument_transfer_enabled { feed(0x4152475452414E53); }',
        'primitive fingerprint')
    primitives = replace_once(primitives,
        '            policy_updates: 0,\n        });',
        '            policy_updates: 0,\n            argument_transfer_enabled: false,\n        });',
        'primitive initial state')
    primitives += '\n// Parameterized views of the same acquired physical definitions.\ninclude!("phase_primitive_arguments.rs");\n'
    induction = replace_once(induction,
        '    pub fn phase_induction_predict(\n        &self,\n        frame: &[Option<f32>],\n    ) -> Option<PhaseInductionPrediction> {\n',
        '    pub fn phase_induction_predict(\n        &self,\n        frame: &[Option<f32>],\n    ) -> Option<PhaseInductionPrediction> {\n        if let Some(call) = self.phase_primitive_argument_prediction(frame) {\n            return Some(call);\n        }\n',
        'normal prediction dispatch')
    induction = replace_once(induction,
        '        if rebuild && p.facts.len() >= state.config.min_child_support {',
        '        // Reuse a factually validated call before rebuilding its definition.\n'
        '        if state.primitives.as_ref().is_some_and(|library| {\n'
        '            primitive_argument_match(&p.facts, library, &state.config,\n'
        '                &self.cells, &self.synapses).is_some()\n'
        '        }) {\n'
        '            return;\n'
        '        }\n'
        '        if rebuild && p.facts.len() >= state.config.min_child_support {',
        'reuse before definition rebuild')
    primitive_path.write_text(primitives)
    induction_path.write_text(induction)
    runpy.run_path(str(ROOT / 'scripts/refine_primitive_binding_evidence.py'),
                   run_name='__main__')
    target = ROOT / 'tests/primitive_argument_transfer.rs'
    shutil.copyfile(ROOT / 'scripts/probes/primitive_argument_transfer.rs', target)
    text = target.read_text()
    text = replace_once(text,
        '        let bindings=rt.organism().phase_primitive_argument_bindings();',
        '        let bindings=rt.organism().phase_primitive_argument_bindings();\n'
        '        println!("PRIMITIVE_ARGUMENT_BINDING_DIAGNOSTIC task={} calls={:?}",task,bindings);',
        'read-only diagnostic')
    target.write_text(text)
    print('Guarded integration ready for unchanged mechanism/runtime tests')


if __name__ == '__main__':
    main()
