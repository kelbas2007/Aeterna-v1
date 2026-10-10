# CONTEXT-VALUE-1 — factual contextual action credit

2026-10-10. New open development experiment, not a sealed qualification.
Historical CHILD-EVENT, RELATIONAL-WORKSPACE and authority scores remain intact.

The diagnostic prototype on development seeds 171000..171255 and
172000..172007 solved both cue variants in 3/8 pairs, with 9/16 terminal
rewards. This supports a limited causal memory mechanism but does not pass
the existing strong cue-use criterion. Most remaining failures lacked access
to the relevant cue. Those diagnostic seeds are development data, not new
heldout evidence. Before the following evaluation, the source is fixed.

## Change and ownership boundary

An opt-in value head replaces summed motor eligibility in the existing general
policy. Bounded context addresses come from actual public observation bits,
the first actual observation, and observed appearance encounter order.
With an object workspace the initial camera pose is excluded from the address
so learned experience can be reused from other starting viewpoints. The
initial observed appearance MULTISET preserves occurrence counts; the former
SET representation erased the distinction between one and two observed objects
of a kind. The first later unambiguous appearance retains its actual scene
context, so a branch object is not automatically assigned the meaning of a cue.
The workspace can retain objects first encountered after the first frame.
No cue label, map, target branch, task phase, motor meaning or correct action
enters the learner. Only actual PRE/action/POST/reward trains it.

Action values use authored tabular temporal-difference learning and reverse
replay of witnessed transitions. Bounded empirical successor statistics
allow a newly learned reward to revise earlier witnessed paths across episodes.
Model backups predict expected utility; they do not create factual rewards
or executed visits. A positive completion reward and an actual episode
boundary terminate a trajectory; dense intermediate rewards are not supported
by this prototype's completion-reward contract.
There are at most 2048 state entries, four measured outcome classes per
state/action, and 256 recent episode transitions; least-recently-used eviction
can forget old states while protecting the active PRE/action/POST update.
This is a small bounded memory prototype, not an unlimited knowledge store.
The replacement value policy does not additionally train the unused factor
solver; validated factual POST still commits through the standard REAL boundary.
Both memory access and selected motors require physical relay conductance.
Value arrays and address/replay computations remain Rust software: physical
causal gating does not establish full EvoPhase ownership or self-invented
learning primitives. MiniGrid input is categorical tiles, not raw RGB.

## Frozen behavioral comparison

Use independently maintained Farama MiniGrid 3.1.0.
Train each of three separate lifetimes in MemoryS7 on seeds
181000..181511 (512 episodes). Maximum 200 actions per episode, identical
termination rules and public input fields. Actions are all selected by the
carrier and executed through ScientificRuntime and Human Protection.

Arms: contextual value + memory; identical value learner without memory;
the source-identical legacy general policy + episode replay + relational
workspace. Training budgets and seeds match; actual action counts must be
reported because successful episodes can end earlier.

Restart and freeze before each heldout episode. On new seeds
182000..182031 test original and physically cue-swapped variants of each map
(64 episodes, 32 balanced pairs). Swapping the cue exchanges the physical
success and failure exits; evaluator metadata never enters Rust. Report
rewarded tasks, both-correct pairs, exit changes, cue exposure and identical
input histories. An unexposed both-correct pair is invalid information leakage.
The existing strong scorer requires >=48/64 rewards, >=16/32 both-correct
cue-exposed pairs and zero unexposed both-correct pairs. Additionally the
memory arm must exceed both controls by >=8 both-correct pairs. Otherwise
the overall experiment is FAIL, even if partial cue use is observed.

Frozen MemoryS9 transfer uses seeds 183000..183015 (32 episodes, 16 pairs)
without retraining; report separately. It tests longer layouts within the
same family, not different visual encoding or broad real-world intelligence.

Before source freeze the development diagnostic showed that the agent did
reach the cue in 8/8 pairs but only solved both variants in 3/8 pairs;
five failed episodes alternated two turns until their 200-action budget.
Transient episode attempts now support generic cycle recovery and exploration
of unknown contexts while the persistent learned model remains frozen.
This is an authored recovery rule, not self-invented metacognition or a motor
route. The evaluator still supplies no motor roles or correct replacement.

Native controls: same present frame after different earlier cues changes
action after self-selected learning; memory relay lesion destroys this
dependence; checkpoint retains values without inventing a past cue; frozen
episodes do not modify learned fingerprints; motor lesions forbid action;
capacity remains bounded when new states arrive. Late factual observation
acquisition must preserve the legacy first-view-only mode.

Run manually with `scripts/external_context_value.py`; ordinary CI tests the
native controls and never rescores this used external sample automatically.
