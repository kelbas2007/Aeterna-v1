# DC-0 adversarial review and negative findings

## Initial checked revision b6221198

GitHub Actions run 38975790589 compiled every target, passed 24 construction
unit tests and 13 selected legacy regressions, and ran the original demo.
Artifact 12079226177 was inspected, not merely its green workflow status.

The autonomous demonstration was INSUFFICIENT: exploration alternated actions.
It learned a one-cell computation that became `output = (action == 0)` after
the first step. This predicts a delayed-action device under alternation but is
not the device's general rule. A reached goal and one accepted program did not
prove identification of hidden state. The initial demo had zero constructors.
Do not cite that run as evidence of autonomous acquisition of the true rule.

## Corrections introduced after inspection

1. A model-independent exploratory reserve uses only factual event count and
   the available action count. It does not access device state or task labels.
2. After autonomous acquisition, freeze program, library and reconstructed
   state. Probe 24 non-alternating actions without learning by that frozen
   program. Compare against action-negation and cold-identity controls.
3. Live checkpoint restoration requires an identical cognitive generation:
   journal, costs, consumed trials and fault state cannot be rewound using a
   snapshot whose current observation merely happens to match.
4. Generic inverse-solver results cannot be credited to a learned constructor.
5. Transfer tests alter only future labels and assert that synthesis cost,
   training loss and program size stay unchanged. Consumed evaluation batches
   cannot be reused. A same-library comparison must not invent an improvement
   when no constructor exists.

These are development regressions, not independent qualification. Passing them
establishes only the tested behavior. Constructors/guards remain bounded program
schemas, not demonstrated open-ended recursive improvement. The default hidden
state demo need not produce any constructors; its claim is stateful acquisition,
frozen prediction and protected action only.

## Runtime clarification

`restore_cognition` supports an exact live round-trip, not arbitrary recovery of
an older live-world snapshot. `Learner::from_checkpoint` reconstructs standalone
cognitive state; attaching that to a separately restored real environment and
permission authority is a host integration responsibility, not yet an automatic
cross-process protected-runtime restart. `rollback_cognition` reverts learned
computation while retaining factual history and costs.

The typed executor supports bounded Fold evaluation, but the default search does
not induce arbitrary loops. The runtime adds state cells; repairing arbitrary
existing state transitions is not implemented. Integer channels are assumed;
byte/sequence/raw-image perception and native SNN realization are not provided.
