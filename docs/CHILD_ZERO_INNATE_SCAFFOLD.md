# AETERNA CHILD-ZERO — innate starting equipment, not an object dictionary

Date: 2026-10-10. Architecture / research contract.
**Not a claim that a scientifically complete list of innately specified human concepts is known.**
Biological development and prenatal experience are not cleanly separable in the newborn.
The correct design target is a STARTING SYSTEM that enables autonomous acquisition
of concepts, skills, memory and language, not a list of adult concepts installed at birth.

## Evidence levels and what is actually present

The table distinguishes well-supported neonatal sensorimotor/physiological
capacities, experimentally observed early **biases** whose precise innate
mechanisms are debated, and later-emerging conceptual claims that MUST
NOT be installed as ready-made facts.

| Category | Human developmental status | Proposed functional equivalent in AETERNA | Present in code? |
|---|---|---|---|
| Automatic survival / arousal regulation | Neonatal respiratory, autonomic, sleep/arousal regulation; sucking/rooting/startle/grasp reflexes are documented | Homeostatic stability and bounded fatigue proxy; never fake breathing or biological pain | REGULATION scaffold yes, human bodily sensors NO |
| Protective reflexes and withdrawal | Multiple neonatal unconditioned reflexes | Maintain Human Protection's pre-existing independent actuation veto; do not give innate permission to harm | Human Protection separate, already enforced |
| Orient to strong sensory contrast/change | Newborn perception and orienting; sensory acuity grows with experience | Physical attention/orienting synapse, effective before any lesson | ORIENTING yes |
| Habituate to repeated stimuli | Fundamental early developmental learning | Adaptive suppression of repeated/uninformative signal | HABITUATION yes |
| Sensory temporal order / continuity | Early temporal perceptual abilities; mature object permanence emerges/develops, not adult-complete at birth | Weak expectation that events are temporally continuous; violation can trigger investigation | CONTINUITY predisposition yes; independent object permanence NO |
| Sensorimotor contingency / self-vs-environment | Body reflexes and early learning of action consequences | Treat own opaque motor PRE/POST as privileged causal evidence, but never import meaning from evaluator | AGENCY readiness + factual contingency yes |
| Approach / exploration / infant curiosity | Infant learning is actively motivated, but motivations evolve | Prior for safe novelty and information gain, not a hard-coded game goal | ORIENTING and AGENCY, opt-in yes |
| Rough magnitude, intensity, number discrimination | Some neonatal/early numerical discrimination evidence; exact early capacity is contested and context sensitive | Sensitivity to approximate magnitude/large changes, NOT exact integer arithmetic or a symbol "three" | MAGNITUDE ready only; counting not implemented |
| Multisensory coordination and common cause | Early capacities with continuing experience | Link coincident audio/vision/touch through synchronized event timing, not feature names | Required interfaces NOT yet present |
| Attraction to face-like/social signals | Early face-like preference observed; specialized innate face-template versus general vision bias disputed | Social-orienting bias only when real visual/social channels are connected | SOCIAL readiness dormant, NO face detector |
| Attention to voice/prosody | Neonatal speech perception; mother's voice/rhythm familiarity partly acquired prenatally | Rhythm/prosody readiness on authentic audio streams, no universal language dictionary | SPEECH readiness dormant, NO audio |
| Early bonding and contingent social response | Very early social interaction shaped by physiology/caregiving and experience | Trust calibration, responsiveness, safe attachment-like assistance, no claim of love/empathy | Not built |
| Body position, touch, proprioception | Biological sensor systems operate from birth, mature with movement | Support embodied proprioception and motor consequences when attached, do not invent virtual body state | Bounded motion proxy present, physical proprioception missing |
| Place, geometry and approximate trajectory | Several primitive spatial abilities develop across infancy | Weak continuity/effect prior, map must form from history | No built-in map facts |
| Event grouping and object candidates | Early capacities with active developmental debates about knowledge at birth | Discover persistent correlated features from perception and action | Existing candidate category signatures, restricted categorical MiniGrid |
| Proto-memory / recognition / prenatal learning | Familiar voices, rhythm/prosody patterns can be recognized at birth due to prenatal exposure | Memory architecture is present at reset, but **content must come from actual past experience** | Checkpoint and associative mechanisms yes; no fabricated prenatal memory |
| Emotion / negative and positive valence | Physiological and affective systems are present; concepts of complex emotions develop | Internal regulation/incentives as operational signals, not subjective feelings or conscious distress | Small safe regulation proxy yes; no claim of emotion |
| Language readiness | Newborns have substantial phonetic/prosodic discrimination, but not an innate lexicon/grammar of a particular language | Sensitivity to patterns, speech rhythm, co-occurring utterance/referent; semantics must be learned and grounded | Ready flag only; grounded words require actual pointing |
| Moral norms / theory of mind / causal tool semantics | NOT safely attributed as a fixed, complete newborn concepts dictionary | Must be formed from social, action and other real experience | Intentionally absent |

Important reading:
- [MedlinePlus, infant reflexes (updated 2025)](https://medlineplus.gov/ency/article/003292.htm)
- [Simion & Di Giorgio 2015, newborn face preference and alternatives](https://pmc.ncbi.nlm.nih.gov/articles/PMC4496551/)
- [Smyth & Ansari 2020, evidential strength of infant numerosity discrimination](https://pubmed.ncbi.nlm.nih.gov/31448505/)
- [Prenatal learning and language: 2026 systematic review](https://pubmed.ncbi.nlm.nih.gov/42632376/)
- [Embodied active learning and infant intrinsic motivations, Annual Review 2020](https://www.annualreviews.org/content/journals/10.1146/annurev-devpsych-121318-084841)

## What INNATE-SCAFFOLD-1 really implements now

\`src/phase_innate_scaffold.rs\` adds EIGHT physical,
opt-in phase-linked biases **before any world experience**:
(0) continuity expectation,
(1) orienting to novelty,
(2) own-action contingency / proto-agency,
(3) habituation,
(4) approximate sensory magnitude readiness,
(5) regulation/bounded activity,
(6) readiness to orient to social information,
(7) readiness to acquire structured speech patterns.

Each bias is represented by an ACTUAL conducting phase synapse
and may be lesioned independently for scientific controls.
No lexicon or object/task-specific transition rule is created.
The social and speech channels are **not populated with fake
recognition evidence** when only a 588-bit MiniGrid image
is available; readiness does not equal perception.
The inherited novelty and agency biases can weakly modulate
the cold learned-only general-policy motor scores, while the
existing U1/HP boundary remains authoritative.

The scaffold learns generic sensory change, action contingency,
habituation and an **artificial** energy budget from actual
protected PRE/ACTION/POST. This energy proxy is NOT oxygen,
hunger, warmth, neonatal hormonal physiology or consciousness.
Frozen mode cannot modify long-term acquired associations;
transient perception remains available and is excluded from
the learned model fingerprint. Native checkpoint stores learned
associations, not the last private observation.

Native controls test:
1. Before any observations, eight prior phase circuits conduct,
   no acquired examples exist, and no object category/word exists.
2. A physically disabled orienting prior causally reduces
   novelty-based motor bias; restoring the actual synapse
   restores the bias.
3. Factual motor actions change own-action contingency and
   habituation; independent restart preserves acquired evidence.
4. Frozen inference cannot silently update the learned fingerprint.

This is an **engineering analogy to developmental readiness**,
not a scientifically faithful simulation of an entire human
neonate. Neither eight biases nor any available finite list
constitute "all human innate concepts."

## What must be tested next, not simply declared

- Distinguish a persistent entity from appearance changes and occlusion
  WITHOUT a global categorical object ID and without a preloaded key/door
  ontology.
- Hold conflicting event histories when CURRENT visible observations
  are identical; choose a different action based on real past evidence.
- Acquire functional concepts from self-chosen actions, not tutor motor
  labels, and generalize to truly different sensory/motor formats.
- Derive early language/referent relations from multimodal dialogue,
  contingent pointing and manipulation, not syntax templates.
- Test necessity of the innate priors: identical source/seeds with
  all-bias lesion, single-prior lesion, and no-prior ablation; if
  scores are unchanged, do not claim the developmental scaffold
  contributes to intelligence.
- Track costs on CPU-first 16 GB laptop and no fabricated background
  experience when the organism is idle.

**Do not tune for a particular MiniGrid seed.** Preserve native
developmental science limitations, and measure external task
completion independently against a frozen ablated control.
