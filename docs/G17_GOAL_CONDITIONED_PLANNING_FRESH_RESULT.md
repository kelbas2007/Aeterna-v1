# FRESH-G17 — goal-conditioned abstract planning

Date: 2026-10-07

## Verdict

**PASS — one-use fresh statistical qualification of bounded goal-conditioned physical planning.**

Workflow: `37617665614`  
Source: `0fcb57f69eca1316c76fd77dcc55b89536a6e507`  
Spec blob SHA: `658495e27ca15aa5153d68fc53b8821e50a68308`  
Authority seed: `37617665614`  
Burned pack digest: `f52c9427c40a5c44`

Observed:

- FULL goal-conditioned planning: **80/80**;
- Wilson95: **[0.954182,1.000000]**;
- every seed: **8/8**;
- goal-switch violations: **0**;
- DEPTH1: **0/80**;
- NO_GOAL: **0/80**;
- opposite/wrong valid goal followed the other goal: **80/80**;
- BROKEN_GOAL: **0/20**;
- BROKEN_ROUTE: **0/20**;
- PI_PHASE_ROUTE: **0/20**;
- exact RESTORE: **20/20**;
- IRRELEVANT_ROUTE_LESION: **10/10**;
- structure/outcome/endpoint violations: **0**;
- REAL/fingerprint mutations: **0**;
- legacy graph violations: **0**;
- all six opaque motors exercised;
- full regressions and Release PASS.

The same physical transition model changed its selected action solely from a different raw goal observation. Transition outcome values remained zero.

This qualifies bounded explicit goal-conditioned abstract planning. It does not establish autonomous goal invention or goal-conditioned active information acquisition.
