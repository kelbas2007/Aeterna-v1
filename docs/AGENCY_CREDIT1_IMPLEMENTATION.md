# AGENCY-CREDIT1 — independent factual learning versus choice utility

Date 2026-10-19. PR9 experimental branch. **No new AGI/generalization claim.**

## Implemented in the same unified organism

Invoke `enable_constructing_agency_aware_credit()` on `ScientificRuntime` before acquiring any construction experience, or send JSONL `{"op":"enable_agency_aware_credit"}` immediately after `init`.

The opt-in tracks two mutually exclusive events per actually executed, protected action:

- exactly one feasible offered motor — `externally_forced_actions`; record REAL PRE/action/POST, all factual effects, action costs and safe sensory updates, **but do NOT train native U1 meta-choice weights**;
- two or more feasible offered motors — `self_selected_actions`; the actual motor was selected by the existing U1 and normal native U1 meta-utility credit applies.

The feasibility mask is the same used for protected decision arbitration, after availability and quoted cost restrictions. No module name, action meaning, expected reward or counterfactual outcome can bypass this check. Cognitive helper proposals cannot earn action-choice credit for a forced motor. The option and event counts persist inside the existing joint checkpoint and are checked on restoration.

Original behavior when the option is OFF remains byte-compatible. Other training lanes and HumanProtection were not modified. Old RSP2 search policy remains unchanged. The option cannot be enabled after the first factual observation, switched off, or re-enabled after a freeze.

Read-only `unified_report().agency_credit` exposes counts without treating model acquisition as an autonomous policy choice.

## Evidence and limitations

The reason for this change is the EXPOSED 2x2 development ablation at 24 motor-role permutations, documented in `docs/AGENCY_CREDIT1_PROTOCOL.md` and test `history_policy_credit_development.rs`. Manual native-U1 credit suppression while collecting identifiability observations yielded 20/24 solved and 232 actions for the history-enabled arm versus 17/24 and 346 actions for its control. These are not independent new-world results.

New engineering tests insist forced actions leave native meta-weight and observation counts unchanged, still fill factual journals, that freely selectable protected actions resume U1 learning, that checkpoints preserve agency counts, and that HumanProtection denial never mints evidence.

**Remaining limitation:** these rules distinguish which action was genuinely selected, not which cognitive operation caused long-term success. The algorithm is still bounded to given goals, offered primitive motors and current numeric sensors. The mechanism is a conservative credit-assignment correction, not open-ended self-improvement. An independent disjoint multi-step world must show a real U1 action selection and performance advantage before science promotion.
