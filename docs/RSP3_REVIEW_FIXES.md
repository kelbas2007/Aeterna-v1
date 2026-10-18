# RSP3 PR #7: reviewed corrections, October 18 2026

Status: a candidate-only search policy, **not** an RSP2 admission or general intelligence qualification.

Three code-review findings were corrected:
1. The protocol now freezes complete source text for learner, task generator, search policy, VM and examination harness. `examine`, `confirm`, `producer` reject any change to these five sources before evaluation. Five deliberate protocol mutations were rejected without producing exam results.
2. The authored candidate grammar interleaves residual features 14, 16 and 17 before the 128-candidate cutoff. The previous order favored feature 14. A new test checks coverage of each feature.
3. All train/development cases validate before the first support-row access. A new test proves malformed first-row input returns `Error::InvalidData`, not panic.

**Local reproducibility after correction:** Rust 1.99.0 offline, all-target compilation passed; 320 selected Rust tests passed. On repeated **already-exposed synthetic cohorts** with the corrected grammar, the learned candidate solved 42/192 versus cold 26/192, loss+size 25/192 and scrambled feedback 25/192. Follow-up 576: 119, 75, 88 and 43 respectively. New training/search work counter: 178323997; 128 trained policy candidates. Producer comparisons remain mixed (2 vs 1 and 0 vs 1), hence no stable recursive self-improvement is established. This is a rerun, **not new blind independent validation**.

The old RSP2 engine and admission gates are byte-for-byte unchanged. Do not merge unless PR Linux and Windows checks pass; original research and frozen evidence remain historical records.
