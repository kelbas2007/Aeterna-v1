# DC-2: self-selected prediction obligations in one lifetime

Status: protocol fixed before the first DC-2 execution. Starting point: `3218662135f6fcf3ba8d766c019ea633f9447914` (`34cbc2c8` executable source). Research branch only; main, INTEL-4 and the action permission boundary remain unchanged.

## Question and falsifiable hypothesis

Can the existing construction runtime select its own learning problems, acquire reusable executable structure and preserve learned laws when learnable dependencies coexist with high-amplitude unpredictable observations? There is no sequence of teacher-selected tasks, injected program, supplied constructor, environment reset or hidden-state API.

Hypothesis: a scheduler that uses its own completed prospective outcomes, with a fair exploration reserve, avoids starvation caused by selecting the largest raw prediction error. All compared schedulers must use the same synthesis implementation, grammar, per-search budget, protection boundary and number of physical interaction steps. Total work is measured, not assumed equal.

This is a development experiment authored with the mechanism, NOT independent qualification or a demonstration of general intelligence. Negative results are retained. Changing budgets, laws, thresholds or implementation after seeing results creates a new development revision; it does not retroactively pass this protocol.

## Derivation

Suppose output j is edited while existing state cells and all other output expressions are preserved. The VM evaluates output expressions from the same pre-action input and state; outputs do not write one another. On factual teacher-forced replay, total absolute error decomposes exactly as `L_total = L_other + L_j`. Thus zero error on j is a valid stopping condition for this local search even if unrelated channels remain unpredictable. It is NOT evidence that the whole world is solved. State updates used in the search are still evaluated and metered; subsequent model installation still requires frozen future observations and the existing retention/audit gates.

If an unresolved nuisance channel maintains error strictly above every learnable channel and the selector always chooses maximum error, those other channels receive no search attempts. This conditional starvation result follows directly from the selection rule. It does not assert that every real noise process satisfies the condition.

Three linked changes are tested: select unresolved obligations using completed future outcomes; factor out unchanged-output error during local construction; reserve fair computation for remaining candidate state updates rather than geometrically starving later grammar leaves. None is called a world-first result. Intrinsic learning progress has substantial prior art: Kaplan and Oudeyer, *In Search of the Neural Circuits of Intrinsic Motivation* (2007), https://www.frontiersin.org/journals/neuroscience/articles/10.3389/neuro.01.1.1.017.2007/full ; Forestier et al., *Intrinsically Motivated Goal Exploration Processes with Automatic Curriculum Learning*, https://arxiv.org/abs/1708.02190 . Our object of investigation is this runtime's executable-structure acquisition under simultaneous obligations, not a claim to have invented curiosity.

## Frozen development setup

World: eight observable coordinates, three actions, one private previous-action register. Four laws coexist: h*x, h*y, h*(x+y), h*(x+z). Three visible exogenous factors change independently each step in [-2,2]; an additional nuisance coordinate changes in [-10000,10000]. Channel positions are permuted from the environment seed. The learner receives none of the mapping, law list, seed, hidden register or future random draws. It sees only the complete current observation and factual action consequences.

Each arm starts cold and remains a single persistent learner. Arms: Progress, Fair (least-attempted unresolved target), LargestResidual. The largest-residual arm receives the same improved targeted solver, making it a controlled scheduler ablation rather than a slower historic implementation.

Initial development seeds: 917 + 1009*j, j=0..3. 160 autonomous interaction steps; minimum training 8; shadow horizon 12; search every 4; search budget 2,000,000 per arm; expression size 9; frontier 32768; at most 12 candidate memory updates; 4 state cells; deterministic-world pruning disabled. No threshold is tuned during this run.

After learning, freeze the program, library and memory, then evaluate 48 new interventions. Probe results cannot train the learner. Report each of the four laws separately. Also evaluate cold identity and reset-memory-every-step controls on the same probe. Compare the checkpoint bytes before and after the probe. A visible partial setpoint goal (four outputs equal zero, other coordinates unspecified) is presented only afterwards, with at most three protected actions; this goal phase is secondary and is not counted as frozen transfer.

## Decision rules and evidence

Primary: number of laws with zero errors in all 48 frozen probe steps; paired Progress-minus-LargestResidual differences per world, reported with actual total construction work. A positive development result requires strictly more frozen-exact laws in Progress in all four worlds, with at least two learned laws per Progress world. Fair may equal or beat Progress; that result must not be hidden or called proof of the learned priority formula.

Secondary, reported without replacing the primary: acquired constructor count, highest generation, executed constructor ancestry and existing same-library/ancestor counterfactual comparisons; memory ablation errors; masked-goal actions; selected target histories; retained models; checkpoint continuity; all exceptions. A new constructor's existence is not proof of improved learning efficiency. Learned constructor use in one lifetime is not by itself recursively improved learning.

Implementation integrity tests and scientific outcomes are separate. The experiment executable emits JSON on positive and negative scientific outcomes. Compilation or integrity failures are failures; an Actions green badge alone is never the scientific verdict. Publish exact source hashes, full small result JSON and CI identity. A later prospective replication must freeze source first and choose new environment seeds afterwards; it still does not make the environment family independently designed.
