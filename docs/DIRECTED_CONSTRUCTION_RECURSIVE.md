# DC-1: prospective recursive construction

Implementation on `research/beyond-intel4`; not a change to the frozen INTEL-4 qualification or EvoPhase's physical permission boundary.

## Operational meaning

A constructor is an executable typed search schema. Two prospectively accepted solutions produced by the same constructor can induce a strict refinement of its code. The stored `Refinement` is a capture-free substitution program over the parent's holes. Replaying it must reconstruct the child code exactly. That child is itself used by the same search and learning cycle; there is no special last generation.

This implements recursion in bounded schema metaprogramming, not unrestricted native-code rewriting. Type rules, evidence acceptance, execution fuel, maximum memory and external authority remain immutable. Generation numbers alone do not establish improvement.

## One loop, including the existing runtime

`Learner::construct` invokes `Registry::begin_trial`, before any shadow POST is available. It freezes the candidate, ordinary same-library search, and (for a refined winner) search with the winning subtree removed. All arms have the same parent, acquired function library, observations, grammar and work limit.

`Learner::finish_shadow` supplies only subsequent factual records to `complete_trial`. The NLL model-installation rule may veto the candidate. Recursive acquisition additionally requires held-forward absolute prediction improvement and training-record retention. Only then can accepted search traces induce another constructor.

Search gives learned schemas a fair bounded first opportunity, with at least a quarter reserved for broad fallback on failure. Individual failed hints cannot starve all other hints. Constructor credit requires actual instantiation of that constructor, never the generic inverse solver.

## Evidence and accounting

Trial commitments are single-use, remain consumed on failure/cancellation, and persist in checkpoints. Changing row IDs cannot reopen the same acquired problem. Future labels are absent from `begin_trial`; changing them can change acceptance, not the frozen searches.

A proposed constructor is not automatically useful. Search improvement requires at least two prospective uses, retained quality, and accumulated savings over the same-library control after charging the entire registry's measured construction work. A recursive improvement additionally beats the parent-containing control. Counterfactual search and validation CPU work are recorded separately in the lifetime ledger: this development verification is not free deployment work. Meter units are instrumented operations, not wall-clock seconds.

The fixed development exercise starts with zero constructors and no injected solutions. The environment supplies hidden-state dynamics; the registry sees only recorded PRE/action/POST. The exercise intentionally supplies a sequence of related environments: it does not independently demonstrate autonomous curriculum discovery or general intelligence. All reports set `independent_qualification=false`.

## Commands

```sh
cargo check --locked --all-targets
cargo test --locked --lib construction::
cargo test --locked --test directed_construction_recursive --test directed_construction_transfer
cargo run --locked --example directed_construction
cargo run --locked --example directed_construction_recursive
```

The last command fails if cold-to-second-generation acquisition and prospective ancestor-controlled improvement are not observed. The source and small logs are preserved by `directed-construction.yml`; a passing result must be tied to its actual source commit, not inferred from this document.
