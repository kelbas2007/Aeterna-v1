# CROSSWORLD-OBJECT-2 — one-pointing word grounding, fresh external seeds

Status: OPEN development, 2026-10-10. Separate from previous seed run.

Previous [run 38037219734](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38037219734)
used arbitrarily repeated pointing and scored 725/725 observed referents,
including 115/115 doors in a different environment family. It had
0 task rewards, and does NOT establish autonomous object-language
semantics. This new test keeps the same generic native visual-to-word
synapse mechanism and U1/HP-executed actions, but restricts the teacher
to exactly ONE accepted exposure per word for training, with
\`--max-lessons 1\`. All other observations are passive.

New train seeds: DoorKey 6100..6111; new heldout seeds: DoorKey,
MultiRoom, Empty 7100..7107 each. Single persistent Rust organism,
checkpoint/restart, no teacher during heldout. The teacher points to
actual public visible tile types identified from outside through the
public MiniGrid categorical type ID; only the tile index and spoken
string are delivered. The learner has no object semantic type table.
Grounding success criterion remains 100% correct visible reference,
0 false positive, at least one correct novel MultiRoom door, and one
real teacher exposure each for key and door. All failures are printed.

CAVEAT: One presentation of a discrete *pre-symbolized* categorical
object code can be memorized without acquiring general visual object
recognition. A pass here is limited one-shot **deictic association
and reference** within a shared simulator visual ontology, not
learning physical affordances or task solutions. A fair AGI claim
would require RGB frames or different object encodings + causal
manipulation transfer, not only recognized categorical classes.
