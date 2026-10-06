# G9 — CARRIER-OWNED BELIEF-STATE PLANNING UNDER PERCEPTUAL ALIASING

Status: **PRE-REGISTERED BEFORE G9 IMPLEMENTATION**

Date: 2026-10-06

## Why G9 exists

G8-FRESH-1 exposed a real boundary: physically different route states can collapse to the same carrier observation and create a shorter imagined path than the external world actually has.

G8-FRESH-2 qualified planning only after a frozen observability seal excluded such worlds.

G9 removes that protection deliberately.

## Question

Can EvoPhase solve a partially observable control problem in which the **same current raw observation** requires different actions depending on factual action/observation history?

The organism must construct and use a carrier-owned recurrent belief state. The evaluator never exposes latent-state IDs.

## World family

Each episode has:

1. a visible initial raw-raster cue;
2. one opaque transition motor leading to an ambiguous corridor observation;
3. the corridor raster is identical across two hidden external contexts;
4. the correct opaque terminal action from that corridor depends on the earlier visible cue/history;
5. a wrong terminal action fails factual Need.

The two hidden contexts are evaluator-only. Cognition receives only:
- raw raster;
- opaque action IDs;
- factual transition sequence;
- factual Need/reward.

No latent context label enters production code.

## Carrier belief state

Allowed generic substrate:

```text
belief_0 = encode(observation_0)
belief_t+1 = bind(belief_t, ACTION_ROLE[action_t], encode(observation_t+1))
```

The exact algebra may be normalized, but it must be generic and task-independent.

The recurrent belief state is MODEL-side carrier state. It is not REAL and it may not overwrite factual observations.

Required property:
- two episodes with identical current corridor observations but different factual histories must produce distinct carrier belief states.

## Learned model

FULL_BELIEF learns action-conditioned transitions/value over carrier belief states.

OBSERVATION_ONLY receives the identical factual episodes but learns over the current observation trace only.

RESET_HISTORY has the belief mechanism available but resets it to the current observation at every step, removing temporal identity.

No evaluator-side policy may choose the terminal motor.

## Development witness

Before any fresh statistical qualification:

- four opaque motor permutations;
- two hidden contexts per permutation;
- held-out absolute raster translations absent from tuition;
- 8 scored episodes total.

Mechanism PASS requires:
1. current ambiguous raw observation is identical across the two contexts;
2. FULL_BELIEF carrier states at that observation are below the frozen match threshold across contexts;
3. FULL_BELIEF succeeds 8/8;
4. OBSERVATION_ONLY succeeds at most 4/8;
5. RESET_HISTORY succeeds at most 4/8;
6. successful terminal motors follow the opaque permutations;
7. pure belief/planning updates do not mutate REAL factual state;
8. G0-G8 regressions and ownership audits PASS;
9. optimized release build PASS.

## Fresh qualification after mechanism PASS

Freeze source/spec, then:
- >=80 one-use worlds / >=10 sub-seeds;
- randomized opaque motor permutation;
- randomized cue/corridor relational codes;
- held-out translations;
- at least two history lengths;
- report Wilson 95% CI and physical action cost;
- FULL_BELIEF vs OBSERVATION_ONLY vs RESET_HISTORY;
- pack burned after first observation.

## Interpretation boundary

A G9 PASS would establish bounded history-conditioned belief-state control/planning under deliberate observation aliasing.

It would not establish:
- arbitrary POMDP solving;
- unrestricted memory length;
- language reasoning;
- unrestricted concept invention;
- AGI.
