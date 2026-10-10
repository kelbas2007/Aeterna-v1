// Opt-in factorized causal acquisition. No external feature labels, motor
// meanings, effects, preconditions, graph or task plan enter this module.
// This research mode currently assumes stable binary raw sensory channels.
// All model updates are from protected factual PRE / executed motor / POST.
// Each reusable rule is enabled by a real conducting phase synapse.
#[derive(Debug, Clone)]
struct PhaseFactorRule {
    motor: usize,
    // Coordinate, observed old value, observed new value.
    effects: Vec<(usize, u8, u8)>,
    positives: Vec<Vec<u8>>,
    guards: Vec<(usize, u8)>,
    evidence_synapse: usize,
    observations: u32,
}

#[derive(Debug, Clone)]
struct PhaseFactorTrial {
    before: Vec<u8>,
    motor: usize,
    no_op: bool,
    observations: u32,
}

#[derive(Debug, Clone)]
struct PhaseFactorState {
    rules: Vec<PhaseFactorRule>,
    trials: Vec<PhaseFactorTrial>,
    contradictions: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseFactorChoice {
    pub action: usize,
    pub evidence_synapse: Option<usize>,
    pub steps: usize,
    pub planned: bool,
    pub support: f32,
}

fn factor_bits(raw: &[f32]) -> Option<Vec<u8>> {
    raw.iter().map(|&x| {
        if (x - 0.0).abs() < 0.001 { Some(0) }
        else if (x - 1.0).abs() < 0.001 { Some(1) }
        else { None }
    }).collect()
}

fn factor_effects(before: &[u8], after: &[u8]) -> Vec<(usize, u8, u8)> {
    before.iter().zip(after).enumerate().filter_map(|(i, (&old, &new))|
        if old != new { Some((i, old, new)) } else { None }
    ).collect()
}

fn factor_match(bits: &[u8], conditions: &[(usize, u8)]) -> bool {
    conditions.iter().all(|&(i, v)| bits[i] == v)
}

fn factor_rebuild_guards(rule: &mut PhaseFactorRule, trials: &[PhaseFactorTrial]) {
    let mut guards: Vec<(usize, u8)> = rule.effects.iter()
        .map(|&(index, old, _)| (index, old)).collect();
    // Distinguish necessary conditions from incidental correlations by
    // starting with the actual changed coordinates. Introduce other
    // conditions only when repeatedly observed factual no-ops require them.
    // This bounded generalization bias favors present independent signals
    // over spurious absence of unrelated objects when evidence is tied.
    let mut negatives = trials.iter().filter(|trial|
        trial.motor == rule.motor && trial.no_op
            && trial.observations >= 2
            && factor_match(&trial.before, &guards)
    ).collect::<Vec<_>>();
    while !negatives.is_empty() {
        let mut best: Option<(usize, u8, usize, usize)> = None;
        let len = rule.positives[0].len();
        for index in 0..len {
            if guards.iter().any(|&(i, _)| i == index) { continue; }
            let value = rule.positives[0][index];
            if !rule.positives.iter().all(|p| p[index] == value) { continue; }
            let separated = negatives.iter()
                .filter(|t| t.before[index] != value).count();
            if separated == 0 { continue; }
            // Prefer a factual positive condition to an arbitrary absence
            // when both explain the identical number of negative samples.
            let priority = usize::from(value == 1);
            if best.is_none_or(|(_, _, n, p)| separated > n
                || (separated == n && priority > p)) {
                best = Some((index, value, separated, priority));
            }
        }
        let Some((index, value, _, _)) = best else { break; };
        guards.push((index, value));
        negatives.retain(|t| t.before[index] == value);
    }
    rule.guards = guards;
}

impl EvoPhase {
    pub fn enable_phase_native_factor_causality(&mut self) -> bool {
        let Some(native) = self.phase_native.as_mut() else { return false; };
        if native.factor_causality.is_some() || !self.config.structural_growth_enabled
            || self.config.sensory_cells > 1024 { return false; }
        native.factor_causality = Some(PhaseFactorState {
            rules: Vec::new(), trials: Vec::new(), contradictions: 0,
        });
        true
    }

    pub fn phase_native_factor_causality_enabled(&self) -> bool {
        self.phase_native.as_ref()
            .and_then(|p| p.factor_causality.as_ref()).is_some()
    }

    pub fn phase_native_factor_rule_count(&self) -> usize {
        self.phase_native.as_ref()
            .and_then(|p| p.factor_causality.as_ref())
            .map(|f| f.rules.len()).unwrap_or(0)
    }

    pub fn phase_native_factor_contradictions(&self) -> u32 {
        self.phase_native.as_ref()
            .and_then(|p| p.factor_causality.as_ref())
            .map(|f| f.contradictions).unwrap_or(0)
    }

    fn phase_factor_live(&self, rule: &PhaseFactorRule) -> f32 {
        let Some(native) = self.phase_native.as_ref() else { return 0.0; };
        conductance(&self.cells, &self.synapses[rule.evidence_synapse],
            native.config.coherence_floor)
    }

    /// Reconstruct a route by composing previously observed causal effects,
    /// never by requiring a previously visited whole-state transition.
    pub fn choose_phase_native_factor_goal_plan(
        &self, raw_goal: &[f32],
    ) -> Option<PhaseFactorChoice> {
        let f = self.phase_native.as_ref()?.factor_causality.as_ref()?;
        let start = factor_bits(&self.current_real.as_ref()?.sensory)?;
        let goal = factor_bits(raw_goal)?;
        if start.len() != goal.len() || start == goal { return None; }
        let mut queue = std::collections::VecDeque::new();
        let mut visited = std::collections::HashSet::new();
        visited.insert(start.clone());
        queue.push_back((start, None::<(usize, usize, f32)>, 0usize));
        let mut examined = 0usize;
        while let Some((state, first, depth)) = queue.pop_front() {
            if examined >= 1024 || depth >= 12 { break; }
            examined += 1;
            for rule in &f.rules {
                let live = self.phase_factor_live(rule);
                if live <= 1.0e-8 || !factor_match(&state, &rule.guards) { continue; }
                let mut successor = state.clone();
                for &(index, _, value) in &rule.effects {
                    successor[index] = value;
                }
                if successor == state { continue; }
                let chosen = first.unwrap_or((rule.motor, rule.evidence_synapse, live));
                if successor == goal {
                    return Some(PhaseFactorChoice {
                        action: chosen.0, evidence_synapse: Some(chosen.1),
                        steps: depth + 1, planned: true, support: chosen.2,
                    });
                }
                if visited.insert(successor.clone()) {
                    queue.push_back((successor, Some(chosen), depth + 1));
                }
            }
        }
        None
    }

    /// Learn the smallest reusable causal effect supported by real action
    /// consequences. Changes not observed in factual POST are never invented.
    pub fn observe_phase_native_factor_transition(
        &mut self, action: usize, pre: &[f32], post: &[f32],
    ) -> bool {
        if action >= self.config.motor_cells || pre.len() != post.len()
            || pre.len() != self.config.sensory_cells { return false; }
        let (Some(before), Some(after)) = (factor_bits(pre), factor_bits(post))
            else { return false; };
        let learning = self.phase_native.as_ref()
            .is_some_and(|n| n.config.learning_enabled && n.factor_causality.is_some());
        if !learning { return false; }
        let effects = factor_effects(&before, &after);
        let Some(mut native) = self.phase_native.take() else { return false; };
        let Some(mut state) = native.factor_causality.take() else {
            self.phase_native = Some(native);
            return false;
        };
        if let Some(trial) = state.trials.iter_mut().find(|t|
            t.motor == action && t.before == before && t.no_op == effects.is_empty()
        ) {
            trial.observations = trial.observations.saturating_add(1);
        } else if state.trials.len() < 1024 {
            state.trials.push(PhaseFactorTrial {
                before: before.clone(), motor: action,
                no_op: effects.is_empty(), observations: 1,
            });
        }
        if !effects.is_empty() {
            let existing = state.rules.iter().position(|rule|
                rule.motor == action && rule.effects == effects);
            let position = if let Some(i) = existing { Some(i) } else {
                if state.rules.len() >= 32 { None } else {
                    let available = self.dormant_range()
                        .find(|&cell| !self.cells[cell].recruited);
                    available.map(|cell| {
                        self.cells[cell].recruited = true;
                        let link = self.native_synapse(self.motor_cell(action), cell);
                        state.rules.push(PhaseFactorRule {
                            motor: action, effects: effects.clone(),
                            positives: Vec::new(), guards: Vec::new(),
                            evidence_synapse: link, observations: 0,
                        });
                        state.rules.len() - 1
                    })
                }
            };
            if let Some(index) = position {
                let rule = &mut state.rules[index];
                rule.observations = rule.observations.saturating_add(1);
                if rule.positives.len() < 128
                    && !rule.positives.iter().any(|p| p == &before) {
                    rule.positives.push(before);
                }
                let syn = &mut self.synapses[rule.evidence_synapse];
                syn.phase_offset = wrap_phase(
                    self.cells[syn.to].phase - self.cells[syn.from].phase);
                syn.confidence = 1.0;
                syn.weight = (syn.weight + 0.125).min(1.0);
                syn.eligibility = 1.0;
            }
        } else {
            state.contradictions = state.contradictions.saturating_add(1);
        }
        for rule in &mut state.rules {
            if !rule.positives.is_empty() {
                factor_rebuild_guards(rule, &state.trials);
            }
        }
        native.factor_causality = Some(state);
        self.phase_native = Some(native);
        true
    }

    /// Goal planning wins when available; otherwise the organism investigates
    /// least-tested factual state/motor pairs, including reachable frontiers.
    pub fn choose_phase_native_factor_action(
        &self, goal: &[f32],
    ) -> Option<PhaseFactorChoice> {
        if let Some(plan) = self.choose_phase_native_factor_goal_plan(goal) {
            return Some(plan);
        }
        let model = self.phase_native.as_ref()?.factor_causality.as_ref()?;
        let start = factor_bits(&self.current_real.as_ref()?.sensory)?;
        if factor_bits(goal)?.len() != start.len() { return None; }
        let mut q = std::collections::VecDeque::from([
            (start.clone(), None::<usize>, 0usize),
        ]);
        let mut seen = std::collections::HashSet::from([start]);
        let mut best: Option<(usize, f32, usize)> = None;
        let mut examined = 0;
        while let Some((state, first, depth)) = q.pop_front() {
            if examined >= 256 || depth > 7 { break; }
            examined += 1;
            for action in 0..self.config.motor_cells {
                let trials = model.trials.iter().find(|t|
                    t.before == state && t.motor == action)
                    .map(|t| t.observations).unwrap_or(0);
                let novelty = 1.0 / (1.0 + trials as f32)
                    / (1.0 + depth as f32 * 0.45);
                let initial = first.unwrap_or(action);
                if best.is_none_or(|(_, previous, _)| novelty > previous + 1.0e-6) {
                    best = Some((initial, novelty, depth + 1));
                }
            }
            if depth == 7 { continue; }
            for rule in &model.rules {
                if self.phase_factor_live(rule) <= 1.0e-8
                    || !factor_match(&state, &rule.guards) { continue; }
                let mut next = state.clone();
                for &(i, _, value) in &rule.effects { next[i] = value; }
                if seen.insert(next.clone()) {
                    q.push_back((next, Some(first.unwrap_or(rule.motor)), depth + 1));
                }
            }
        }
        let (action, novelty, steps) = best?;
        Some(PhaseFactorChoice {
            action, evidence_synapse: None, steps,
            planned: false, support: novelty,
        })
    }

    pub fn is_native_factor_synapse(&self, index: usize) -> bool {
        self.phase_native.as_ref()
            .and_then(|n| n.factor_causality.as_ref())
            .is_some_and(|f| f.rules.iter().any(|r| r.evidence_synapse == index))
    }
}
