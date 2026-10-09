#!/usr/bin/env python3
"""Generic regime-change handling for argument-call evidence, before testing.

The first two open runs exposed stale per-action facts and successful-action
selection bias. No task labels, argument pairs, expected results or evaluator
thresholds are used by this patch. A candidate fits a contradiction-free tail
of an EARLIER prefix; the later eight facts validate that frozen candidate.
"""
from pathlib import Path

PATH = Path(__file__).resolve().parents[1] / 'src/phase_primitive_arguments.rs'
START = 'fn primitive_argument_match('
END = '\nimpl EvoPhase {'
REPLACEMENT = r'''fn primitive_argument_match(
    facts:&std::collections::VecDeque<InductionFact>,
    state:&PhasePrimitiveState,
    config:&PhaseInductionConfig,
    cells:&[PhaseCell],
    links:&[PhaseSynapse],
) -> Option<PrimitiveArgumentMatch> {
    if !state.argument_transfer_enabled { return None; }
    let width=facts.front()?.input.len();
    if width > 8 { return None; }
    let future=config.min_future_checks as usize;
    let required_fit=config.min_child_support*2;
    let mut budget=32768usize;
    let mut best:Option<PrimitiveArgumentMatch>=None;
    let mut best_fit=0usize;
    for (operation_index,op) in state.operations.iter().enumerate() {
        if !op.admitted { continue; }
        let Some(source_inputs)=primitive_argument_sources(
            operation_index,state,width,cells,links) else { continue; };
        // A definition may be supported only by facts observed after it was
        // published. Source-task labels are never recycled as target facts.
        let newer=facts.iter().filter(|f|f.sequence>op.published_sequence)
            .collect::<Vec<_>>();
        if newer.len() < required_fit+future { continue; }
        let split=newer.len()-future;
        let fit=&newer[..split];
        for arguments in primitive_argument_permutations(width,source_inputs.len()) {
            let mut candidate=PrimitiveArgumentMatch {
                operation_index,source_inputs:source_inputs.clone(),arguments,
                fit_facts:0,future_checks:future,
            };
            // A factual contradiction revokes support for this binding, not
            // the immutable underlying operation. Fit a recent consistent
            // segment using the PREFIX ONLY; later outcomes cannot select it.
            let mut start=0usize;
            for (i,fact) in fit.iter().enumerate() {
                if budget==0 { return None; }
                budget-=1;
                if !primitive_argument_read(&candidate,state,config,&fact.input,cells,links)
                    .is_some_and(|(v,_)|(v-f32::from(fact.success)).abs()
                        <=1.0-config.minimum_outcome) {
                    start=i+1;
                }
            }
            let supported=&fit[start..];
            if supported.len()<required_fit || supported.len()<=best_fit {continue;}
            // A successful policy need not deliberately produce failures.
            // Instead require factual variation in EVERY selected argument,
            // with replicated observations on both sides of that variation.
            // Merely replaying one successful input cannot admit a binding.
            let diverse=candidate.arguments.iter().all(|&arg| {
                let lo=supported.iter().map(|f|f.input[arg]).fold(f32::INFINITY,f32::min);
                let hi=supported.iter().map(|f|f.input[arg]).fold(f32::NEG_INFINITY,f32::max);
                if !lo.is_finite() || !hi.is_finite() || hi-lo<=1.0e-6 {return false;}
                let middle=lo+(hi-lo)*0.5;
                supported.iter().filter(|f|f.input[arg]<=middle).count()
                    >=config.min_child_support
                    &&supported.iter().filter(|f|f.input[arg]>middle).count()
                        >=config.min_child_support
            });
            if !diverse {continue;}
            candidate.fit_facts=supported.len();
            best_fit=supported.len();
            best=Some(candidate);
        }
    }
    // Freeze the best prefix-supported candidate. A failing later suffix
    // returns no call; it does not choose another candidate using those facts.
    let chosen=best?;
    let op=&state.operations[chosen.operation_index];
    let newer=facts.iter().filter(|f|f.sequence>op.published_sequence)
        .collect::<Vec<_>>();
    let suffix=&newer[newer.len()-future..];
    for fact in suffix {
        if budget==0 {return None;}
        budget-=1;
        if !primitive_argument_read(&chosen,state,config,&fact.input,cells,links)
            .is_some_and(|(v,_)|(v-f32::from(fact.success)).abs()
                <=1.0-config.minimum_outcome) {return None;}
    }
    Some(chosen)
}
'''

text = PATH.read_text()
if text.count(START) != 1 or text.count(END) != 1:
    raise RuntimeError('Unexpected argument matcher source; refusing replacement')
a = text.index(START)
b = text.index(END, a)
PATH.write_text(text[:a] + REPLACEMENT + text[b:])
print('Applied prefix-only contradiction revision; withheld suffix never chooses a rival')
