# G2 — CARRIER-OWNED INFORMATIVE EXPERIMENT SELECTION

Status: **PRE-REGISTERED BEFORE G2 IMPLEMENTATION**

Date: 2026-10-06

## Question

After experience has produced multiple EvoPhase-owned predictive hypotheses that are all compatible with the current raw observation, can the organism choose a physical probe because those hypotheses predict different factual outcomes for that probe?

This gate tests active epistemic learning. It does not test acquired exploration strategies yet; that is G6.

## Relation to G1

G1-B established a narrow carrier-owned relational trace from raw 12x12 raster input and translation transfer.

G2 reuses that qualified relational representation. The host still receives and transports only raw rasters, opaque motor tokens, episode boundaries and factual outcomes. It does not pass a latent law label or the informative action.

## Training / tuition worlds

There are two hidden deterministic world laws, Hα and Hβ. The labels Hα/Hβ exist only in the evaluator.

Each tuition episode starts from the same relational start scene and contains three opaque probe actions.

The organism is exposed to complete factual probe traces during tuition using a fixed preregistered action schedule shared by all arms. This controlled tuition is allowed because G2 tests **probe selection from acquired rival models**, not curriculum invention.

Across the two hidden laws:
- probe 0 has the same factual relational consequence;
- probe 1 has different factual relational consequences;
- probe 2 has a smaller or redundant difference.

The carrier is not told this structure. It must acquire whole predictive episode models from factual transitions.

Training scenes and post-scenes are translated across episodes so absolute raster addresses cannot identify the law.

## Carrier-owned rival models

A learned world hypothesis contains:
- a relational trace for the precondition;
- an acquired predicted relational post-trace for each observed opaque probe;
- support / confidence / revision;
- provenance to factual training episodes.

Hypotheses are merged only when their acquired predictions are mutually compatible.

Different incompatible factual transition patterns remain as rival hypotheses. No host code assigns a latent-law ID to a carrier hypothesis.

## Held-out unknown worlds

A held-out episode belongs to one of the previously experienced laws but is rendered at unseen absolute translations.

At the initial observation both learned hypotheses must remain compatible.

The organism may select physical probes sequentially.

GENUINE computes epistemic value from **disagreement among active EvoPhase-owned predictions**.

GENERIC_EXPLORATION has the same hypotheses, same factual memory, same action budget and same raw observations but is not allowed to use hypothesis disagreement when choosing a probe. It uses the ordinary generic novelty/exploration ordering.

## Epistemic score

For each available opaque probe, GENUINE evaluates the dispersion of predicted relational post-traces across currently active hypotheses.

A probe has epistemic value only when:
- at least two active hypotheses make predictions for it;
- those predictions differ.

After factual POST, incompatible hypotheses are suppressed in the current epistemic episode, but the global learned hypotheses remain in memory.

No MODEL or IMAGINED outcome is written as REAL.

## Acceptance

G2 PASS requires:

1. At least two distinct carrier-owned hypotheses are acquired from tuition without latent-law labels entering cognition.
2. At held-out start, both rival hypotheses are active.
3. GENUINE selects probe 1 first because its acquired predictions disagree most.
4. GENERIC_EXPLORATION selects probe 0 first under the same initial state/budget.
5. One factual result of GENUINE probe 1 reduces the active rival set to one.
6. GENERIC_EXPLORATION requires more factual probes than GENUINE to reach one surviving rival on each held-out law.
7. The final surviving hypothesis predicts an additional withheld consequence correctly before that consequence is revealed.
8. Raw held-out scenes are at unseen absolute translations and still match through G1 relational representation.
9. Matched arms share identical learned hypotheses, resource limits and initial factual state; only disagreement readout for probe selection differs.
10. Rust tests and release build PASS.
11. Total tuition interactions and held-out physical probes are reported.

## Failure conditions

FAIL if:
- the host supplies the informative action;
- hypothesis identity is copied from evaluator law labels;
- only one hypothesis exists at held-out start;
- disagreement is calculated from evaluator truth instead of carrier predictions;
- the first probe does not reduce the rival set;
- generic control has different learned data or a smaller budget;
- only a software-unit test passes but no end-to-end held-out factual probe sequence is exercised.

## Cost accounting

Report separately:
- tuition factual probes used to acquire the rival models;
- held-out probes per world for GENUINE;
- held-out probes per world for GENERIC_EXPLORATION.

A held-out probe reduction does not by itself prove total amortized efficiency if tuition dominates. That distinction must remain explicit.

## Next gate after PASS

G3: repeated successful EvoPhase dynamics must consolidate into the first acquired reusable macro/program and transfer to a new binding.
