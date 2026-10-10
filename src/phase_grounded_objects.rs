// Opt-in, deictically supervised object-word memory, stored with the same
// carrier as action and causal learning. The host tells us only where a
// tutor points in the CURRENT factual view and what word was spoken.
//
// NO built-in "door", "key", MiniGrid enum, mission lookup, world identity or
// object semantics. The currently supported sensor format is a regularly
// packed grid of categorical channels. This is not RGB segmentation.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PhaseVisualCategory {
    signature: Vec<u8>,
    cell: usize,
    sightings: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PhaseGroundedWord {
    word: String,
    category: usize,
    lexical_cell: usize,
    evidence_synapse: usize,
    demonstrations: u32,
}

#[derive(Debug, Clone)]
struct PhaseLearnedAffordance {
    category: usize,
    tile: usize,
    action: usize,
    word: String,
    synapse: usize,
    observed_successes: u32,
    // These are real measured changes in the target tile, not reward guesses.
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseWordAction {
    pub action: usize,
    pub synapse: usize,
    pub tile: usize,
    pub strength: f32,
}

#[derive(Debug, Clone)]
struct PhaseGroundedObjects {
    width: usize,
    height: usize,
    channels: usize,
    bits_per_channel: usize,
    identity_channels: usize,
    categories: Vec<PhaseVisualCategory>,
    words: Vec<PhaseGroundedWord>,
    frames_seen: u64,
    affordances: Vec<PhaseLearnedAffordance>,
    intent_word: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhaseGroundedReferent {
    pub tile_index: usize,
    pub strength: f32,
    pub demonstrations: u32,
}

impl PhaseGroundedObjects {
    fn dim(&self) -> usize {
        self.width * self.height * self.channels * self.bits_per_channel
    }

    fn tile_signature(&self, raw: &[f32], tile: usize) -> Option<Vec<u8>> {
        if raw.len() != self.dim() || tile >= self.width * self.height {
            return None;
        }
        let stride = self.channels * self.bits_per_channel;
        let mut signature = Vec::new();
        for channel in 0..self.identity_channels {
            let base = tile * stride + channel * self.bits_per_channel;
            let mut value = 0u8;
            for bit in 0..self.bits_per_channel {
                let v = raw[base + bit];
                if !v.is_finite() { return None; }
                if v >= 0.999 && v <= 1.001 {
                    value |= 1 << bit;
                } else if v < -0.001 || v > 0.001 {
                    return None;
                }
            }
            signature.push(value);
        }
        Some(signature)
    }
}

impl EvoPhase {
    /// Sensor *layout* only, not ontology or visual semantics. The first
    /// identity_channels are used as a candidate stable visual signature;
    /// remaining channels may vary (e.g., open/closed state).
    pub fn enable_phase_native_grounded_objects(
        &mut self, width: usize, height: usize,
        channels: usize, bits_per_channel: usize,
        identity_channels: usize,
    ) -> bool {
        let dims = width.checked_mul(height)
            .and_then(|n| n.checked_mul(channels))
            .and_then(|n| n.checked_mul(bits_per_channel));
        if width == 0 || height == 0 || channels == 0
            || identity_channels == 0 || identity_channels > channels
            || bits_per_channel == 0 || bits_per_channel > 8
            || dims != Some(self.config.sensory_cells)
        { return false; }
        let Some(native) = self.phase_native.as_mut() else { return false; };
        if native.grounded_objects.is_some() { return false; }
        native.grounded_objects = Some(PhaseGroundedObjects {
            width, height, channels, bits_per_channel, identity_channels,
            categories: Vec::new(), words: Vec::new(), frames_seen: 0,
            affordances: Vec::new(), intent_word: None,
        });
        true
    }

    pub fn phase_native_grounded_categories(&self) -> usize {
        self.phase_native.as_ref()
            .and_then(|n| n.grounded_objects.as_ref())
            .map_or(0, |g| g.categories.len())
    }

    /// Index for counterfactual lesion of the actual learned word link.
    pub fn phase_native_grounded_word_synapse(
        &self, word: &str,
    ) -> Option<usize> {
        self.phase_native.as_ref()?.grounded_objects.as_ref()?
            .words.iter().find(|w| w.word == word.trim().to_lowercase())
            .map(|w| w.evidence_synapse)
    }

    pub fn phase_native_grounded_words(&self) -> usize {
        self.phase_native.as_ref()
            .and_then(|n| n.grounded_objects.as_ref())
            .map_or(0, |g| g.words.len())
    }

    pub fn phase_native_grounded_frames(&self) -> u64 {
        self.phase_native.as_ref()
            .and_then(|n| n.grounded_objects.as_ref())
            .map_or(0, |g| g.frames_seen)
    }

    pub fn phase_native_learned_affordances(&self) -> usize {
        self.phase_native.as_ref().and_then(|n| n.grounded_objects.as_ref())
            .map_or(0, |g| g.affordances.len())
    }

    pub fn is_phase_native_affordance_synapse(&self, index: usize) -> bool {
        self.phase_native.as_ref().and_then(|n| n.grounded_objects.as_ref())
            .is_some_and(|g| g.affordances.iter().any(|a| a.synapse == index))
    }

    /// A word-grounded interaction is a task request, not knowledge of which
    /// motor works. An unknown word cannot invent a new motor.
    pub fn set_phase_native_word_intent(&mut self, word: &str) -> bool {
        let Some(g) = self.phase_native.as_mut()
            .and_then(|n| n.grounded_objects.as_mut()) else { return false; };
        let w = word.trim().to_lowercase();
        if !g.words.iter().any(|learned| learned.word == w) { return false; }
        g.intent_word = Some(w);
        true
    }

    pub fn clear_phase_native_word_intent(&mut self) {
        if let Some(g) = self.phase_native.as_mut()
            .and_then(|n| n.grounded_objects.as_mut()) {
            g.intent_word = None;
        }
    }

    /// A human demonstrator has executed a real external motor, and submits
    /// its *factual* before/after visual views with the pointed, already
    /// named object. We credit only a local observed change at that object
    /// and refuse a camera jump (any other pixel/tile changing).
    /// This is explicitly motor-supervised learning, NOT autonomous discovery.
    pub fn learn_phase_native_demonstrated_affordance(
        &mut self, word: &str, tile: usize, action: usize,
        before: &[f32], after: &[f32],
    ) -> bool {
        if action >= self.config.motor_cells || before.len() != after.len()
            || before.len() != self.config.sensory_cells { return false; }
        let learning = self.phase_native.as_ref()
            .is_some_and(|n| n.config.learning_enabled && n.grounded_objects.is_some());
        if !learning || self.current_real.as_ref()
            .is_none_or(|r| r.sensory != before) { return false; }
        let Some(mut native) = self.phase_native.take() else { return false; };
        let Some(mut memory) = native.grounded_objects.take() else {
            self.phase_native = Some(native); return false;
        };
        let normalized = word.trim().to_lowercase();
        let lex = memory.words.iter().find(|w| w.word == normalized).cloned();
        let Some(lex) = lex else {
            native.grounded_objects = Some(memory);
            self.phase_native = Some(native); return false;
        };
        let target = memory.tile_signature(before, tile);
        let correct = target.as_ref()
            == Some(&memory.categories[lex.category].signature);
        let stride = memory.channels * memory.bits_per_channel;
        let start = tile.saturating_mul(stride);
        let end = start.saturating_add(stride);
        if end > before.len() {
            native.grounded_objects = Some(memory);
            self.phase_native = Some(native);
            return false;
        }
        let local_changed = tile < memory.width * memory.height
            && before[start..end] != after[start..end];
        let rest_stable = before.iter().zip(after).enumerate()
            .all(|(i, (a, b))| (i >= start && i < end) || a == b);
        let witness = correct && local_changed && rest_stable;
        if witness && memory.affordances.len() < 64 {
            if let Some(record) = memory.affordances.iter_mut().find(|a|
                a.category == lex.category && a.tile == tile && a.action == action
            ) {
                record.observed_successes = record.observed_successes.saturating_add(1);
                let syn = &mut self.synapses[record.synapse];
                syn.weight = (syn.weight + 0.125).min(1.0);
            } else {
                // Word -> action is only active when BOTH the word binding
                // and this directly observed motor affordance conduct.
                let link = self.native_synapse(
                    lex.lexical_cell, self.motor_cell(action));
                let syn = &mut self.synapses[link];
                syn.phase_offset = wrap_phase(
                    self.cells[syn.to].phase - self.cells[syn.from].phase);
                syn.weight = 0.125;
                syn.confidence = 1.0;
                syn.eligibility = 1.0;
                memory.affordances.push(PhaseLearnedAffordance {
                    category: lex.category, tile, action, word: normalized,
                    synapse: link, observed_successes: 1,
                });
            }
        }
        native.grounded_objects = Some(memory);
        self.phase_native = Some(native);
        witness
    }

    /// At inference, no demonstrator motor is available. Same lexical and
    /// object-affordance physical synapses must both conduct. A novel room
    /// may reuse a learned relative object position; otherwise abstain.
    pub fn choose_phase_native_grounded_word_action(
        &self, raw: &[f32],
    ) -> Option<PhaseWordAction> {
        let native = self.phase_native.as_ref()?;
        let g = native.grounded_objects.as_ref()?;
        let word = g.intent_word.as_ref()?;
        let lex = g.words.iter().find(|w| &w.word == word)?;
        let lexical = conductance(&self.cells,
            &self.synapses[lex.evidence_synapse],native.config.coherence_floor);
        if lexical <= 1.0e-8 { return None; }
        let mut best: Option<PhaseWordAction> = None;
        for affordance in g.affordances.iter().filter(|a|
            a.word == *word && a.category == lex.category
        ) {
            if g.tile_signature(raw,affordance.tile).as_ref()
                != Some(&g.categories[lex.category].signature) { continue; }
            let physical = conductance(&self.cells,
                &self.synapses[affordance.synapse],native.config.coherence_floor);
            let score = physical.min(lexical);
            if score > best.map_or(1.0e-8, |previous| previous.strength + 1.0e-6) {
                best = Some(PhaseWordAction {
                    action: affordance.action, synapse: affordance.synapse,
                    tile: affordance.tile, strength: score
                });
            }
        }
        best
    }

    pub fn phase_native_word_intent_active(&self) -> bool {
        self.phase_native.as_ref().and_then(|n| n.grounded_objects.as_ref())
            .is_some_and(|g| g.intent_word.is_some())
    }

    pub fn is_phase_native_grounded_word_synapse(&self, index: usize) -> bool {
        self.phase_native.as_ref().and_then(|n| n.grounded_objects.as_ref())
            .is_some_and(|g| g.words.iter()
                .any(|w| w.evidence_synapse == index))
    }

    /// Passive sensory object memory: same visual signatures survive changes
    /// of position, camera orientation, episode and independent world.
    /// Names are NOT inferred from these unlabeled observations.
    pub fn observe_phase_native_object_view(&mut self, raw: &[f32]) -> bool {
        let learning = self.phase_native.as_ref()
            .is_some_and(|n| n.config.learning_enabled
                && n.grounded_objects.is_some());
        if !learning { return false; }
        let Some(mut native) = self.phase_native.take() else { return false; };
        let Some(mut memory) = native.grounded_objects.take() else {
            self.phase_native = Some(native);
            return false;
        };
        if raw.len() != memory.dim() {
            native.grounded_objects = Some(memory);
            self.phase_native = Some(native);
            return false;
        }
        let mut signatures = Vec::new();
        for index in 0..memory.width * memory.height {
            if let Some(s) = memory.tile_signature(raw, index) {
                if !signatures.contains(&s) { signatures.push(s); }
            } else {
                native.grounded_objects = Some(memory);
                self.phase_native = Some(native);
                return false;
            }
        }
        for signature in signatures {
            if let Some(cat) = memory.categories.iter_mut()
                .find(|c| c.signature == signature) {
                cat.sightings = cat.sightings.saturating_add(1);
            } else if memory.categories.len() < 64 {
                let Some(cell) = self.dormant_range()
                    .find(|&c| !self.cells[c].recruited) else { break };
                self.cells[cell].recruited = true;
                memory.categories.push(PhaseVisualCategory {
                    signature, cell, sightings: 1
                });
            }
        }
        memory.frames_seen = memory.frames_seen.saturating_add(1);
        native.grounded_objects = Some(memory);
        self.phase_native = Some(native);
        true
    }

    /// A deictic language lesson: a separate teacher points to a visible tile
    /// and says an arbitrary word. The sole supervision is that pairing;
    /// the learner receives NO category code, correct object action, or route.
    /// Same word + different category is refused (ambiguity), not overwritten.
    pub fn teach_phase_native_pointed_word(
        &mut self, word: &str, tile: usize,
    ) -> bool {
        let normalized = word.trim().to_lowercase();
        if normalized.is_empty() || normalized.len() > 48
            || !normalized.chars().all(|c| c.is_alphanumeric() || c == '_')
        { return false; }
        let Some(raw) = self.current_real.as_ref()
            .map(|f| f.sensory.clone()) else { return false };
        let learning = self.phase_native.as_ref()
            .is_some_and(|n| n.config.learning_enabled);
        if !learning { return false; }
        let Some(mut native) = self.phase_native.take() else { return false; };
        let Some(mut memory) = native.grounded_objects.take() else {
            self.phase_native = Some(native);
            return false;
        };
        let signature = memory.tile_signature(&raw, tile);
        let category = signature.as_ref().and_then(|s| memory.categories.iter()
            .position(|c| &c.signature == s));
        let result = if let Some(cat) = category {
            if let Some(existing) = memory.words.iter_mut()
                .find(|w| w.word == normalized) {
                if existing.category != cat {
                    false
                } else {
                    let syn = &mut self.synapses[existing.evidence_synapse];
                    syn.weight = (syn.weight + 0.125).min(1.0);
                    existing.demonstrations = existing.demonstrations.saturating_add(1);
                    true
                }
            } else if memory.words.len() >= 32 {
                false
            } else if let Some(cell) = self.dormant_range()
                .find(|&c| !self.cells[c].recruited) {
                self.cells[cell].recruited = true;
                let synapse = self.native_synapse(memory.categories[cat].cell, cell);
                let syn = &mut self.synapses[synapse];
                syn.phase_offset = wrap_phase(
                    self.cells[syn.to].phase - self.cells[syn.from].phase);
                syn.weight = 0.125;
                syn.confidence = 1.0;
                syn.eligibility = 1.0;
                memory.words.push(PhaseGroundedWord {
                    word: normalized,
                    category: cat,
                    lexical_cell: cell,
                    evidence_synapse: synapse,
                    demonstrations: 1,
                });
                true
            } else { false }
        } else { false };
        native.grounded_objects = Some(memory);
        self.phase_native = Some(native);
        result
    }

    /// Word -> current visible referents. No teacher pointer is needed
    /// for this readout. Unknown/ambiguous or broken-synapse words fail closed.
    pub fn locate_phase_native_grounded_word(
        &self, word: &str, raw: &[f32],
    ) -> Vec<PhaseGroundedReferent> {
        let Some(native) = self.phase_native.as_ref() else { return Vec::new() };
        let Some(memory) = native.grounded_objects.as_ref() else { return Vec::new() };
        let Some(record) = memory.words.iter()
            .find(|w| w.word == word.trim().to_lowercase())
            else { return Vec::new() };
        let strength = conductance(
            &self.cells, &self.synapses[record.evidence_synapse],
            native.config.coherence_floor);
        if strength <= 1.0e-8 || raw.len() != memory.dim() { return Vec::new() }
        let signature = &memory.categories[record.category].signature;
        (0..memory.width * memory.height)
            .filter_map(|tile| {
                if memory.tile_signature(raw, tile).as_ref() != Some(signature) {
                    return None;
                }
                Some(PhaseGroundedReferent {
                    tile_index: tile, strength,
                    demonstrations: record.demonstrations,
                })
            }).collect()
    }
}
