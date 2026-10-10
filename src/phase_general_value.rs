// Opt-in bounded action values for the existing general policy. The state
// address comes only from public factual perception and episode observations.
// This is authored TD/replay mathematics, not an invented learning primitive.
const GENERAL_VALUE_CAPACITY: usize = 2048;
const GENERAL_VALUE_EPISODE_CAPACITY: usize = 256;

#[derive(Clone)]
struct PhaseGeneralValueState {
    key: u64,
    // Packed factual observations for opt-in acquired predicate rules.
    features: Vec<u64>,
    values: Vec<f32>,
    visits: Vec<u32>,
    outcomes: Vec<Vec<PhaseGeneralValueOutcome>>,
}

impl std::fmt::Debug for PhaseGeneralValueState {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        // Fingerprinting every protected action must cover acquired features
        // without formatting thousands of packed sensor words as decimal text.
        let digest=self.features.iter().fold(0xcbf29ce484222325u64,
            |h,&word|(h^word).wrapping_mul(0x100000001b3));
        f.debug_struct("PhaseGeneralValueState").field("key",&self.key)
            .field("feature_words",&self.features.len()).field("feature_digest",&digest)
            .field("values",&self.values).field("visits",&self.visits)
            .field("outcomes",&self.outcomes).finish()
    }
}

#[derive(Debug, Clone)]
struct PhaseGeneralValueOutcome {
    after: Option<u64>,
    count: u32,
    reward_sum: f32,
    cost_sum: f32,
}

#[derive(Debug, Clone)]
struct PhaseGeneralValueTransition {
    before: u64,
    after: u64,
    action: usize,
    reward: f32,
    unchanged: bool,
}

#[derive(Debug, Clone)]
struct PhaseGeneralValueLearning {
    states: Vec<PhaseGeneralValueState>,
    episode: Vec<PhaseGeneralValueTransition>,
    initial: u64,
    initial_frame: Vec<u64>,
    abstraction: Option<PhaseValueAbstraction>,
    memory_link: Option<usize>,
    motors: usize,
}

impl PhaseGeneralValueLearning {
    fn clear_episode(&mut self) {
        self.episode.clear();
        self.initial = 0;
        self.initial_frame.clear();
    }

    fn state(&self, key: u64) -> Option<&PhaseGeneralValueState> {
        self.states.iter().find(|s| s.key == key)
    }

    fn ensure(&mut self, key: u64) {
        if let Some(index)=self.states.iter().position(|s|s.key==key) {
            // Keep the active factual PRE alive while inserting its POST.
            // Least-recently-used eviction must not remove the very state
            // whose executed action is currently being credited.
            if index+1<self.states.len() {
                let state=self.states.remove(index);
                self.states.push(state);
            }
            return;
        }
        if self.states.len() >= GENERAL_VALUE_CAPACITY {
            self.states.remove(0);
        }
        self.states.push(PhaseGeneralValueState {
            key,
            features: Vec::new(),
            values: vec![0.0; self.motors],
            visits: vec![0; self.motors],
            outcomes: vec![Vec::new(); self.motors],
        });
    }

    fn backup(&mut self, transition: &PhaseGeneralValueTransition, terminal: bool) {
        let future = if terminal {
            0.0
        } else {
            self.state(transition.after)
                .map_or(0.0, |s| s.values.iter().copied().fold(0.0, f32::max))
        };
        // Penalize lack of a public perceptual change, never a
        // fabricated correct alternative or an imagined successful POST.
        let cost = if transition.unchanged { 0.012 } else { 0.002 };
        let target = transition.reward + 0.99 * future - cost;
        if let Some(state) = self.states.iter_mut().find(|s| s.key == transition.before) {
            let value = &mut state.values[transition.action];
            *value = (*value + 0.35 * (target - *value)).clamp(-1.0, 1.0);
        }
    }

    fn finish(&mut self, rewarded: bool) {
        if self.episode.is_empty() {
            return;
        }
        let witnessed = self.episode.clone();
        // Retain measured transition statistics across episodes. A reward
        // discovered later can inform earlier witnessed paths; merely
        // replaying the most recent episode could never do that.
        for (index, transition) in witnessed.iter().enumerate() {
            let after = if index + 1 == witnessed.len() {
                None
            } else {
                Some(transition.after)
            };
            if let Some(state) = self.states.iter_mut().find(|s| s.key == transition.before) {
                let outcomes = &mut state.outcomes[transition.action];
                if !outcomes.iter().any(|o| o.after == after) {
                    if outcomes.len() >= 4 {
                        let least = outcomes
                            .iter()
                            .enumerate()
                            .min_by_key(|(_, o)| o.count)
                            .unwrap()
                            .0;
                        outcomes.remove(least);
                    }
                    outcomes.push(PhaseGeneralValueOutcome {
                        after,
                        count: 0,
                        reward_sum: 0.0,
                        cost_sum: 0.0,
                    });
                }
                let outcome = outcomes.iter_mut().find(|o| o.after == after).unwrap();
                outcome.count = outcome.count.saturating_add(1);
                outcome.reward_sum += transition.reward;
                outcome.cost_sum += if transition.unchanged { 0.012 } else { 0.002 };
            }
        }
        for _ in 0..4 {
            for (index, transition) in witnessed.iter().enumerate().rev() {
                self.backup(transition, index + 1 == witnessed.len());
            }
        }
        // MODEL predictions are expected values, never extra factual
        // rewards/visits. All model edges came from actual external POST.
        for _ in 0..6 {
            let future = self
                .states
                .iter()
                .map(|s| (s.key, s.values.iter().copied().fold(0.0, f32::max)))
                .collect::<std::collections::HashMap<_, _>>();
            for state in &mut self.states {
                for (action, outcomes) in state.outcomes.iter().enumerate() {
                    let count = outcomes.iter().map(|o| o.count as f32).sum::<f32>();
                    if count == 0.0 {
                        continue;
                    }
                    let target = outcomes
                        .iter()
                        .map(|o| {
                            let next = o
                                .after
                                .and_then(|key| future.get(&key).copied())
                                .unwrap_or(0.0);
                            o.reward_sum - o.cost_sum + 0.99 * next * o.count as f32
                        })
                        .sum::<f32>()
                        / count;
                    state.values[action] = (state.values[action]
                        + 0.75 * (target - state.values[action]))
                        .clamp(-1.0, 1.0);
                }
            }
        }
        if rewarded {
            self.episode.clear();
        }
        if let Some(abstraction) = self.abstraction.as_mut() {
            abstraction.completed = abstraction.completed.saturating_add(1);
            if abstraction.completed % 16 == 0 {
                abstraction.fit(&self.states);
            }
        }
    }
}

impl EvoPhase {
    pub fn phase_native_context_value_enabled(&self) -> bool {
        self.phase_native
            .as_ref()
            .and_then(|n| n.general_policy.as_ref())
            .is_some_and(|p| p.value_learning.is_some())
    }

    pub fn enable_phase_native_context_value_learning(&mut self) -> bool {
        let Some(policy) = self
            .phase_native
            .as_ref()
            .and_then(|n| n.general_policy.as_ref())
        else {
            return false;
        };
        if policy.value_learning.is_some() {
            return false;
        }
        let memory = policy.developmental_memory;
        let memory_link = if memory {
            let Some(cell) = self.dormant_range().find(|&c| !self.cells[c].recruited) else {
                return false;
            };
            self.cells[cell].recruited = true;
            let index = self.native_synapse(0, cell);
            let synapse = &mut self.synapses[index];
            synapse.weight = 0.6;
            synapse.confidence = 1.0;
            synapse.eligibility = 1.0;
            synapse.phase_offset =
                wrap_phase(self.cells[synapse.to].phase - self.cells[synapse.from].phase);
            Some(index)
        } else {
            None
        };
        let policy = self
            .phase_native
            .as_mut()
            .unwrap()
            .general_policy
            .as_mut()
            .unwrap();
        if let Some(work) = policy.relational_workspace.as_mut() {
            work.online_acquisition = true;
        }
        policy.value_learning = Some(PhaseGeneralValueLearning {
            states: Vec::new(),
            episode: Vec::new(),
            initial: 0,
            initial_frame: Vec::new(),
            abstraction: None,
            memory_link,
            motors: self.config.motor_cells,
        });
        true
    }

    pub fn phase_native_value_state_count(&self) -> usize {
        self.phase_native
            .as_ref()
            .and_then(|n| n.general_policy.as_ref())
            .and_then(|p| p.value_learning.as_ref())
            .map_or(0, |v| v.states.len())
    }

    pub fn phase_native_value_memory_link(&self) -> Option<usize> {
        self.phase_native
            .as_ref()?
            .general_policy
            .as_ref()?
            .value_learning
            .as_ref()?
            .memory_link
    }

    pub fn phase_native_value_context_key(&self, raw: &[f32]) -> Option<u64> {
        self.general_value_key(raw, false)
    }

    fn general_value_key(&self, raw: &[f32], post: bool) -> Option<u64> {
        if raw.len()!=self.config.sensory_cells {return None;}
        Self::general_frame_features(raw)?;
        let native = self.phase_native.as_ref()?;
        let policy = native.general_policy.as_ref()?;
        let value = policy.value_learning.as_ref()?;
        let mut key = Self::general_hash(raw);
        let memory = value.memory_link.is_some_and(|link| {
            conductance(
                &self.cells,
                &self.synapses[link],
                native.config.coherence_floor,
            ) > 1e-7
        });
        if memory {
            // With an object workspace, encounter content is the context.
            // An initial camera pose must not create a separate navigation
            // skill namespace for every starting viewpoint.
            if policy.relational_workspace.is_none() {
                key ^= value.initial.rotate_left(23);
            }
            if let Some(workspace) = policy.relational_workspace.as_ref() {
                if conductance(
                    &self.cells,
                    &self.synapses[workspace.synapse],
                    native.config.coherence_floor,
                ) <= 1e-7
                {
                    return Some(key);
                }
                let mut work = workspace.clone();
                if post {
                    work.next(raw);
                }
                // Observed encounter order, not object semantics or a target.
                // Preserve multiplicity: two witnessed instances must not
                // collapse to the same set as one instance of each appearance.
                for signature in &work.first_appearances {
                    for &field in signature {
                        key = (key ^ u64::from(field)).wrapping_mul(0x100000001b3);
                    }
                    key = (key ^ 256).wrapping_mul(0x100000001b3);
                }
                key ^= work.subject_scene.rotate_left(17);
            }
        }
        Some(key)
    }

    fn choose_phase_native_value_action(&self, raw: &[f32]) -> Option<PhaseGeneralDecision> {
        let key = self.general_value_key(raw, false)?;
        let native = self.phase_native.as_ref()?;
        let policy = native.general_policy.as_ref()?;
        let value = policy.value_learning.as_ref()?;
        let state = value.state(key);
        let learning = native.config.learning_enabled;
        let mut salt = key
            ^ if learning {
                policy.steps.wrapping_mul(0x9E3779B97F4A7C15)
            } else {
                (policy.recent.len() as u64).wrapping_mul(0x9E3779B97F4A7C15)
            };
        salt ^= salt >> 30;
        salt = salt.wrapping_mul(0xBF58476D1CE4E5B9);
        let exploring = learning && salt % 100 < 18;
        let attempts = (0..self.config.motor_cells)
            .map(|action| {
                policy
                    .recent
                    .iter()
                    .filter(|&&(s, a)| s == key && a == action)
                    .count()
            })
            .collect::<Vec<_>>();
        let inferred = if state.is_none() && !learning {
            self.value_abstraction_prediction(raw)
        } else { None };
        let unknown = state.is_none() && inferred.is_none();
        let untried = attempts.iter().any(|&n| n == 0);
        let mut best = None::<PhaseGeneralDecision>;
        for action in 0..self.config.motor_cells {
            let link = policy.physical_links[action];
            let physical = conductance(
                &self.cells,
                &self.synapses[link],
                native.config.coherence_floor,
            );
            if physical <= 1e-7 {
                continue;
            }
            if unknown && untried && attempts[action] > 0 {
                continue;
            }
            let q = state.map_or_else(|| inferred.as_ref().map_or(0.0, |p| p[action]), |s| s.values[action]);
            let visits = state.map_or(0, |s| s.visits[action]);
            let jitter = ((salt ^ (action as u64).wrapping_mul(0xD6E8FEB86659FD93))
                .wrapping_mul(0xA0761D6478BD642F)
                >> 40) as f32
                / (1u32 << 24) as f32;
            let mut score = if exploring || unknown {
                jitter
            } else {
                q + if learning {
                    0.20 / (1.0 + visits as f32).sqrt()
                } else {
                    0.0
                } + jitter * 1e-6
            };
            // Frozen knowledge does not mean frozen behavior. Recovery from
            // a repeated context/action cycle uses only this episode's real
            // attempts, and never fabricates training rewards or a route.
            if attempts[action] >= 8 {
                score -= 0.15 * (attempts[action] - 7) as f32;
            }
            let decision = PhaseGeneralDecision {
                action,
                score: score * physical,
                synapse: link,
            };
            if best.is_none_or(|d| decision.score > d.score) {
                best = Some(decision);
            }
        }
        best
    }

    fn observe_phase_native_value_transition(
        &mut self,
        action: usize,
        pre: &[f32],
        post: &[f32],
        reward: f32,
    ) -> bool {
        let Some(before) = self.general_value_key(pre, false) else {
            return false;
        };
        let Some(after) = self.general_value_key(post, true) else {
            return false;
        };
        let before_features = self.value_abstraction_features(pre, false);
        let after_features = self.value_abstraction_features(post, true);
        let native = self.phase_native.as_mut().unwrap();
        let learning = native.config.learning_enabled;
        let policy = native.general_policy.as_mut().unwrap();
        policy.steps += 1;
        if policy.recent.len() >= 512 {
            policy.recent.remove(0);
        }
        policy.recent.push((before, action));
        if !learning {
            return true;
        }
        let value = policy.value_learning.as_mut().unwrap();
        value.ensure(before);
        value.ensure(after);
        if let Some(features) = before_features {
            value.states.iter_mut().find(|s|s.key==before).unwrap().features=features;
        }
        if let Some(features) = after_features {
            value.states.iter_mut().find(|s|s.key==after).unwrap().features=features;
        }
        let state = value.states.iter_mut().find(|s| s.key == before).unwrap();
        state.visits[action] = state.visits[action].saturating_add(1);
        let transition = PhaseGeneralValueTransition {
            before,
            after,
            action,
            reward,
            unchanged: pre == post,
        };
        value.backup(&transition, reward > 0.0);
        if value.episode.len() >= GENERAL_VALUE_EPISODE_CAPACITY {
            value.episode.remove(0);
        }
        value.episode.push(transition);
        if reward > 0.0 {
            value.finish(true);
            policy.positive_rewards = policy.positive_rewards.saturating_add(1);
        }
        policy.updates += 1;
        true
    }
}
