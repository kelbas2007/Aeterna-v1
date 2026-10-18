# DC-1: prospective recursive construction

Implementation on `research/beyond-intel4`; not a change to the frozen INTEL-4 qualification or EvoPhase's physical permission boundary.

## Operational meaning

A constructor is an executable typed search schema. Two prospectively accepted solutions produced by the same constructor can induce a strict refinement of its code. The stored `Refinement` is a capture-free substitution program over the parent's holes. `materialize_constructor` actually executes the ancestry of rewrites before proposing computations; the stored child code is checked as a cache. That child is used by the same search and learning cycle. There is no generation-two special case or authored final generation.

This implements recursion in bounded schema metaprogramming, not unrestricted native-code rewriting. Type rules, evidence acceptance, execution fuel, maximum memory and external authority remain immutable. Generation numbers alone do not establish improvement.

## One loop, including the existing runtime

`Learner::construct` invokes `Registry::begin_trial`, before any shadow POST is available. It freezes the candidate, ordinary same-library search, and (for a refined winner) search with the winning subtree removed. All arms have the same parent, acquired function library, observations, grammar and work limit.

`Learner::finish_shadow` supplies only subsequent factual records to `complete_trial`. The NLL model-installation rule may veto the candidate. Recursive acquisition additionally requires held-forward absolute prediction improvement and training-record retention. Only then can accepted search traces induce another constructor.

Search starts with smallest schema completions, giving every schema an opportunity before increasing the number of expression nodes. Schemas requiring fewer independent bindings are tried before those requiring more; generation is a tie-breaker, not a quality score. At least a quarter of search work remains available for broad fallback on failure. Failed hints, repeated setup, constructor materialization and ordering are charged. Constructor credit requires actual instantiation of that constructor, never the generic inverse solver. These generic scheduling rules are authored infrastructure, not claimed as a learned discovery.

## Evidence and accounting

Trial commitments are single-use, remain consumed on failure/cancellation, and persist in checkpoints. Changing row IDs cannot reopen the same acquired problem. Future labels are absent from `begin_trial`; changing them can change acceptance, not the frozen searches.

A proposed constructor is not automatically useful. Search improvement requires at least two prospective uses, retained quality, and accumulated savings over the same-library control after charging the entire registry's measured construction work. A recursive improvement additionally beats the parent-containing control. Usefulness is revoked when later measured costs remove the advantage. Counterfactual search and validation CPU work are recorded separately in the lifetime ledger: this verification is not free deployment work. Meter units are instrumented operations, not wall-clock seconds.

The fixed development exercise starts with zero constructors and no injected solutions. The environment supplies hidden-state dynamics; the registry sees only recorded PRE/action/POST. All cases remain in the report, including unprofitable transfers. Its gate requires retained final benefit, actual useful later-generation execution, and exact future predictions throughout the sequence; an isolated earlier success is insufficient.

The exercise intentionally supplies a sequence of related environments. Each local model search begins from an identity program while the constructor registry, traces, costs and trial consumption persist. This is not evidence of independently discovered curriculum, a single unreset world model across unrelated domains, or general intelligence. The pure-function library is empty in this stateful exercise; equality of that library is supplemented by the stronger parent-containing comparison. All reports set `independent_qualification=false`.

## Commands

```sh
cargo check --locked --all-targets
cargo test --locked --lib construction::
cargo test --locked --test directed_construction_recursive --test directed_construction_scheduling --test directed_construction_transfer
cargo run --locked --example directed_construction
cargo run --locked --example directed_construction_recursive
```

The last command exits unsuccessfully if the full retained multi-generation criterion is not met. Unit syntax fixtures in the scheduling tests are explicitly labelled and are never counted as acquisition evidence. Small logs and the exact source revision are preserved by `directed-construction.yml`; a passing result must be tied to its actual source commit, not inferred from this document.

## Audit trail

The first green result did not retain its advantage through the last case: see [initial audit](DIRECTED_CONSTRUCTION_INITIAL_AUDIT.md). The expanded run at `84001380` generated four generations with zero candidate future loss on all 16 cases, but correctly FAILED the stronger gate: a broader generation-four schema displaced cheaper specialized siblings. Source-independent ordering by the number of free bindings addresses that specific scheduling defect; it does not hide or discard the failed cases.
