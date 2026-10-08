// Operational state resolution for unified proposal fields.
// This is recognition/applicability, not a ranking of cognitive modules.
// All applicable promoted refinements enter the same set. Missing required
// evidence or ambiguous distinct states abstain instead of guessing a parent.

impl EvoPhase {
    fn unified_refined_path_conducts(
        &self,
        base: usize,
        gate_source: usize,
        refined: usize,
        links: [usize; 2],
        floor: f32,
    ) -> bool {
        let Some(base_link) = self.synapses.get(links[0]) else { return false; };
        let Some(gate_link) = self.synapses.get(links[1]) else { return false; };
        base_link.from == base && base_link.to == refined
            && gate_link.from == gate_source && gate_link.to == refined
            && conductance(&self.cells, base_link, floor)
                .min(conductance(&self.cells, gate_link, floor)) > 1.0e-8
    }

    fn unified_operational_entry(&self, sensory: &[f32], base: usize) -> Option<usize> {
        let native = self.phase_native.as_ref()?;
        let floor = native.config.coherence_floor;
        let mut active = Vec::new();

        if let Some(context) = native.contextual.as_ref() {
            for witness in context.candidates.iter()
                .filter(|w| w.base_cell == base && w.promoted && !w.retired)
            {
                let previous = context.previous_base?;
                let Some(side) = witness.predecessor_cells.iter().position(|&p| p == previous)
                    else { continue; };
                let entry = witness.state_cells[side];
                if !self.unified_refined_path_conducts(
                    base, previous, entry, witness.input_synapses[side], floor,
                ) { return None; }
                active.push(entry);
            }
        }

        if let Some(perceptual) = native.perceptual.as_ref() {
            for witness in perceptual.candidates.iter()
                .filter(|w| w.base_cell == base && w.promoted && !w.retired)
            {
                let raw = self.phase_raw_features(sensory)?;
                let side = Self::percept_side(&raw, witness)?;
                let entry = witness.state_cells[side];
                if !self.unified_refined_path_conducts(
                    base, witness.feature_cells[side], entry,
                    witness.input_synapses[side], floor,
                ) { return None; }
                active.push(entry);
            }
        }

        if let Some(compositional) = native.compositional.as_ref() {
            for witness in compositional.candidates.iter()
                .filter(|w| w.base_cell == base && w.promoted && !w.retired)
            {
                let raw = self.phase_raw_features(sensory)?;
                let side = usize::from(witness.program.eval(&raw));
                let entry = witness.state_cells[side];
                if !self.unified_refined_path_conducts(
                    base, witness.program_cells[side], entry,
                    witness.input_synapses[side], floor,
                ) { return None; }
                active.push(entry);
            }
        }

        active.sort_unstable();
        active.dedup();
        match active.as_slice() {
            [] => Some(base),
            [entry] => Some(*entry),
            _ => None,
        }
    }
}
