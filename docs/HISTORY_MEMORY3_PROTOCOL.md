# HISTORY-MEMORY-3 — remembered successful episodes in a delayed-cue outside world

Date 2026-10-10, OPEN DEVELOPMENT. This is a NEW, third
architecture (positive recalled event associations) after
two native-history failures; not independent final evidence.
[First source-result](HISTORY_MEMORY1_RESULT.md) and
[second native hypothesis](HISTORY_MEMORY2_PROTOCOL.md) remain
explicitly retained.

## Fundamental change in representation

The generic distributed motor/value policy previously retained only
mean learned action preferences and a leaky short experience trace.
HISTORY-MEMORY-1 and HISTORY-MEMORY-2 exposed an architectural
limitation: two different preceding cues, SAME visible present
observation, produced the same final motor 2 even after
many factual rewarded experiences.

An opt-in *episodic autobiographical event store* now records
\`(previous-experience trace + present observation, actually executed
opaque action, factually positive later outcome)\`. It does
NOT store a simulator's object labels, correct branch name,
goal position, map, motor semantics or oracle action.
On a later event with similar SUBJECTIVE experience it may recall
a factually successful action, physically gated by the same
existing native motor synapse and U1/Human Protection.
If no high-confidence event matches, it falls back to the
generic learned distributed-value policy. The threshold/cosine
retrieval algorithm is **handwritten Rust**, analogous to an
episodic associative learning bias, NOT spontaneously invented
by an SNN.

The externally independent source is Farama MemoryS7; it
requires remembering an object seen in the starting room
(key vs ball) after walking down a corridor and selecting the
matching branch when the original cue is out of view.
No object type, target, "correct" choice, mission,
global agent position, hidden map, simulator action labels
or teacher route enters native cognition.

## Controls / reserved clean seeds

1. Native cold/learned history-alias qualification: identical
   currently visible image, differing past cue, ACTUAL learned
   action must differ and persist through checkpoint.
   Zero prior experience cannot preload the correct action;
   identical-image/no-memory control must not discriminate.
2. Source-identical actual external \`minigrid==3.1.0\` test:
   training seeds 100000..100127, 128 episodes per carrier.
   Heldout 101000..101023, 24 episodes, up to 200
   U1/HP-screened externally executed motors per episode.
   No episode staging, tutoring, carrying insertion,
   simulated object type or reward oracle.
3. Active: generic policy with history memory and positive
   episodic experience recall; control: SAME actor without
   memory. Same seeded random external baseline.
   Actual simulator task terminal reward determines success,
   not predicted rewards or a successful native unit test.
4. Separate memory-less and old leaky memory variants may be
   compared for interpretation but not retuned to these
   consumed seed outcomes. Model stays fixed within the run.

**Strict OPEN development success: at least 12/24 actual
heldout full MemoryS7 rewards and strictly more than the
memoryless and random controls, native history alias pass,
nonzero protected actions, and frozen learned state.**
Any failure yields DEVELOPMENT_FAIL, never fabricated AGI
success or unqualified claim that "object permanence" emerged.
