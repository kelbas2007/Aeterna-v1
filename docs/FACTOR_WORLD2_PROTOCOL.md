# FACTOR-WORLD-2 — causal rule recombination and surprise development

Date: 2026-10-10; research/beyond-intel4.
OPEN DEVELOPMENT, not independent/frozen scientific qualification.
Previous FACTOR-WORLD-1 remains FAIL 0/4, unchanged.

## Mechanism under test
A new optional phase-native factor causality mechanism observes only
valid binary raw PRE, opaque executed action, and validated raw POST.
It discovers which *sensory coordinates* actually changed and stores
motor/effect descriptions with minimum precondition values from changed
coordinates. Repeated factual action no-ops may add necessary guards,
with a generic preference for positive sensor evidence over irrelevant
absence when observations are underdetermined. Conditions and effects
are not supplied by the environment. Conducting motor->rule physical
synapses gate each rule; search composes effects with bounded BFS
on IMAGINED raw states, not observed whole-state transition nodes.
Search, condition construction and exploration are Rust code, not
new algorithms invented by the neural substrate. U1 still selects
the proposal; Human Protection alone authorizes external execution.

## Input and withholding
Sensory vectors consist of eight stable binary feature-bearing pixel
positions in a 400-element raw array. Their positions and six opaque
motor identities are independently shuffled per organism. The
environment contains room, key inventory, battery, supply inventory,
key on shelf and supply on shelf. Object/feature identity is *not*
supplied to the mechanism. This is an unusually favorable compositional
sensory encoding, NOT unsupervised object segmentation.

Four organism lifetimes each undergo 140 interleaved episodes in
a key-only world (target gated exit) and 140 episodes in a
supply-only world (target charger with supply). Each episode begins
at its corresponding actual start world and runs at most 22
protected, autonomously chosen motor steps. No training state
may simultaneously hold key AND supply, asserted over all factual
states. The model is checkpointed/restarted and frozen for first
heldout, where both independent objects are available and the
desired exit requires both and a charged battery. Evaluator-only
oracle confirms five actions suffice. The original factoring
mechanism is shared across all permutations.

Then the model is unfrozen and a new heldout episode injects one
unannounced, one-time gate actuator failure that discharges battery
without opening the gate. The organism must plan from updated factual
feedback, recharge and retry. No hidden failure flag is shown to
cognition.

## Predeclared open development target (not qualification)
Every one of four organisms must:
- reach at least 100/140 key and 100/140 supply training goals;
- acquire at least five reusable factored effect rules;
- obtain nonempty composed starting plan after checkpoint;
- reach novel composition in exactly five actions with >=4 planned
  actions and unchanged U1 weights under frozen learning;
- prove that physically zeroing the first necessary rule synapse
  removes the goal route, and restoring it restores the route;
- experience exactly one real surprise discharge, then recover
  within 12 actions, with no safety blocks or unsupported execution.

The first full run's negative outcome, if any, must be preserved,
including compile/runtime bugs and algorithm failures. Subsequent
modifications are development repairs and must never be silently
described as frozen first-attempt qualification.

## Interpretations / limits
Even if the experiment passes, this is a narrow synthetic
compositional generalization test with binary raw feature channels,
a domain-independent but handcrafted Rust effect/guard induction
procedure and explicit bounded BFS. It does NOT establish the
organism invented factor ontology, invented search, learned natural
visual object persistence, reliable stochastic inference, general
planning, or AGI. Confirming unseen sensory topologies or learned
object discovery requires separate challenges.
