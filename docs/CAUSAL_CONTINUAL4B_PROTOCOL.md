# CAUSAL-CONTINUAL-4B — categorical goals and delayed decision utility

2026-10-20. Freeze this protocol and implementation before any B scores.
The [interrupted first cohort](CAUSAL_CONTINUAL4_NEGATIVE.md) is negative and
remains preserved; it is not a successful one-use evaluation. Its full seed
range is withdrawn. Seeds 230001 and 230002 are now disclosed development
regressions, not evidence for this comparison.

Use **240000..240031**, 32 new opaque motor and sensor assignments. Preserve
the original [four families, training/action limits, budgets, arms and gates](CAUSAL_CONTINUAL4_PROTOCOL.md).
Every arm receives the actual remaining per-episode action limit through the
same bounded step API; it is resource metadata, not a motor/hidden-state label.
The causal search truncates its horizon at this real deadline. Full legacy
and full causal arms use maximum horizon 12 and 120,000 planning-work
units per actual action. The new-mode horizon-one ablation uses horizon one
and the same work limit. Every motor is available and costs one. The legacy
control means the selector with the new causal mode disabled in this runtime,
not a separately compiled historical release that forbade 12 sensors.

Before each held phase, freeze and restore the trained lifetime; neither
causal counts nor model weights may change. U1 remains the physical motor
selector. Reward may come from exact goal membership or actual environment
reward; numerical similarity of opaque state codes is not a reward. Delayed
information and route decisions are credited once against the real eventual
goal and accumulated actual cost, without an earlier immediate-zero penalty.

Run isolated seed processes, at most four concurrently by default, to avoid
serial evaluation becoming the dominant runtime. No acquired knowledge crosses
arms, seeds, processes or world families. The posterior-information prior for previously unknown goal-outcome laws
is Beta(1,1). Its expected information decreases on actual outcomes, including
random noise. It is kept separate from predicted goal value. Goal-conditioned
confidence and immediate forecasts use the same acquired contextual evidence
for every competing motor proposal; irrelevant numeric novelty is zero.

Process concurrency changes no action
or planning budget. Record the source revision, executable SHA256 and process
configuration **before** spawning any evaluator. Retain every completed life,
real action, POST, cost, all three arms and all noise controls. Report failed
gates, too; do not tune against B outcomes and call them fresh again.

The fixed engineering gates remain: causal route success at least 24/32 and
above horizon-one; causal noisy-sensor success at least 75% and at least ten
percentage points above legacy; direct-goal success no more than two points
below legacy. Report the uninformative-sensor success and action overhead
without presenting chance fluctuations as learned information. There is no
independent third-party or real-world AGI qualification from these authored
controls.

Build `cargo build --locked --release --example causal_continual_eval`, then
run `python scripts/run_causal_continual4.py --binary
target/release/examples/causal_continual_eval --output OUTPUT --workers 4`.
The evaluator and driver permit no automatic post-score cognitive mutations.
