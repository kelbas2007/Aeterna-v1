# POST-INTEL-2 local evidence coverage — result

Date: 2026-10-08
Branch: `unified-operation-competition`

## Verdict

**MECHANISM PASS / SYSTEM INSUFFICIENT.**

The preregistered local-evidence-first learned-drive change behaved as intended and
did not restore a full World C solution by itself.

## Generic regressions

After the cognitive change:

- P4 learned-drive regression: PASS;
- G16 abstract learned-drive regression: PASS.

The learned drive weights remained effectively:

`[0.99999994, 1.0]`.

## Burned-pack diagnostic only

The old INTEL-2 pack remains burned FAIL and is not requalified.

Before the redesign, both junction histories executed only motor 0 for 450
trials and no context candidate was born.

After local-evidence-first ordering, initial junction coverage became:

`0 -> 1 -> 2 -> 3 -> 4 -> 5`

and the diagnostic reached:

- 72 history trials;
- promoted context: true;
- eligible observations: 32;
- context switches: 31;
- log evidence: 18.24776870998912;
- no unsupported action during acquisition.

Thus the starvation mechanism identified in the protocol is closed.

## Remaining failure

Held-out score remained 16/32:

- predecessor side 0: 16/16;
- predecessor side 1: 0/16.

On side 1 the promoted context selector returned:

`(true, None)`

meaning the refined representation was required but had not yet acquired a
supported action. The unified collector then omitted that context proposal and
allowed the ambiguous inherited parent to continue with goal action 1.

Therefore the next bottleneck is not local exploration coverage. It is loss of
representation-ownership semantics in unified proposal assembly.
