# Research basis — 2026

Это не список гарантий Aeterna-v1. Здесь зафиксированы применимые принципы и границы переноса.

## Local predictive learning

Stöckl, Yang & Maass, Nature Communications 2024: local next-observation prediction can self-organize a high-dimensional cognitive map and support online planning. Для проекта важен принцип локального predictive residual; их конкретная сеть не переносится автоматически.

https://www.nature.com/articles/s41467-024-46586-0

## Phase + assembly context

Russo et al., Nature Communications 2024: firing phase can distinguish context and assembly membership. Для EvoPhase это опора использовать phase as information, а не декоративный clock.

https://www.nature.com/articles/s41467-024-52988-x

## Dendritic contextual computation

Baek et al., Nature Electronics 2024: dendritic nonlinear integration and silent synapses support spatiotemporal/context-sensitive computation. Это поддерживает local branches/gates, но не доказывает наш learning algorithm.

https://www.nature.com/articles/s41928-024-01171-7

## Continual plasticity

Dohare et al., Nature 2024: continual learners can lose plasticity; maintaining variability and replacing low-utility units can preserve learnability. Поэтому long-lived EvoPhase needs bounded structural lifecycle, not only retention.

https://www.nature.com/articles/s41586-024-07711-7

## Relational concept invention

Shah et al., CoRL 2025: relational concepts can be invented from demonstrations and reused for zero-shot generalization in their setting. This supports testing acquired relations rather than task memorization.

https://proceedings.mlr.press/v305/shah25a.html

## Hierarchical skill discovery

Harvey et al., ICML 2026: unlabeled trajectories can be segmented into reusable skills and hierarchies. For Aeterna-v1 the analogue is consolidation of recurrent EvoPhase motifs.

https://proceedings.mlr.press/v306/harvey26a.html

## Learning plasticity

Shen et al., NeurIPS 2025: plasticity rules themselves can be optimized in SNNs. We do not import their method, but it supports eventually making part of the learning rule adaptive rather than forever developer-fixed.

https://proceedings.neurips.cc/

## Sequence self-organization

Bell, Duffy & Fairhall, NeurIPS 2024: local plasticity can self-organize and maintain sequence circuits. Relevant to carrier-owned temporal motifs.

https://proceedings.neurips.cc/

## Explicit exclusions

DreamCoder/Stitch are useful analogies for reusable abstraction, but a fixed AST/DSL is not accepted as the owner of intelligence here. Dreamer-style world models support imagination as a principle, but we do not import an actor/critic that would replace EvoPhase ownership.
