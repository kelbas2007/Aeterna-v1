use crate::hdc::PhaseVector;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeTrace {
    oriented_fragments: Vec<u64>,
    scale_free_oriented_fragments: Vec<u64>,
    invariant_fragments: Vec<u64>,
}

impl ShapeTrace {
    pub fn new(invariant_fragments: Vec<u64>) -> Self {
        Self::from_channels(Vec::new(), Vec::new(), invariant_fragments)
    }

    pub fn from_fragments(
        oriented_fragments: Vec<u64>,
        invariant_fragments: Vec<u64>,
    ) -> Self {
        Self::from_channels(oriented_fragments, Vec::new(), invariant_fragments)
    }

    pub fn from_geometry(
        _point_count: usize,
        oriented_fragments: Vec<u64>,
        invariant_fragments: Vec<u64>,
    ) -> Self {
        // Kept as a compatibility constructor for the first R1 development
        // iterations. Matching is based on relational evidence, not on a
        // nuisance/cardinality branch.
        Self::from_channels(oriented_fragments, Vec::new(), invariant_fragments)
    }

    pub fn from_channels(
        mut oriented_fragments: Vec<u64>,
        mut scale_free_oriented_fragments: Vec<u64>,
        mut invariant_fragments: Vec<u64>,
    ) -> Self {
        oriented_fragments.sort_unstable();
        scale_free_oriented_fragments.sort_unstable();
        invariant_fragments.sort_unstable();
        Self {
            oriented_fragments,
            scale_free_oriented_fragments,
            invariant_fragments,
        }
    }

    pub fn oriented_fragments(&self) -> &[u64] {
        &self.oriented_fragments
    }

    pub fn scale_free_oriented_fragments(&self) -> &[u64] {
        &self.scale_free_oriented_fragments
    }

    pub fn invariant_fragments(&self) -> &[u64] {
        &self.invariant_fragments
    }

    pub fn similarity(&self, other: &Self) -> f32 {
        let oriented =
            Self::containment(&self.oriented_fragments, &other.oriented_fragments);
        let scale_free = Self::containment(
            &self.scale_free_oriented_fragments,
            &other.scale_free_oriented_fragments,
        );
        let invariant =
            Self::containment(&self.invariant_fragments, &other.invariant_fragments);

        // Evidence sufficiency, not nuisance detection:
        // - oriented triples preserve exact surviving substructure;
        // - scale-free oriented triples preserve that substructure under uniform
        //   integer scale;
        // - fully invariant triangle shape supports rotation/reflection, but a
        //   single such triangle is too underdetermined to identify a learned
        //   four-point world on its own.
        //
        // Legacy traces containing only invariant fragments remain comparable.
        let structural_channels_present = !self.oriented_fragments.is_empty()
            || !other.oriented_fragments.is_empty()
            || !self.scale_free_oriented_fragments.is_empty()
            || !other.scale_free_oriented_fragments.is_empty();
        let invariant_evidence = self
            .invariant_fragments
            .len()
            .min(other.invariant_fragments.len());
        let invariant_score =
            if invariant_evidence >= 2 || !structural_channels_present {
                invariant
            } else {
                0.0
            };

        oriented.max(scale_free).max(invariant_score)
    }

    fn containment(a: &[u64], b: &[u64]) -> f32 {
        if a.is_empty() || b.is_empty() {
            return 0.0;
        }

        let mut i = 0usize;
        let mut j = 0usize;
        let mut overlap = 0usize;

        while i < a.len() && j < b.len() {
            match a[i].cmp(&b[j]) {
                std::cmp::Ordering::Less => i += 1,
                std::cmp::Ordering::Greater => j += 1,
                std::cmp::Ordering::Equal => {
                    overlap += 1;
                    i += 1;
                    j += 1;
                }
            }
        }

        overlap as f32 / a.len().min(b.len()) as f32
    }
}

#[derive(Debug, Clone)]
pub enum CarrierTrace {
    Exact(PhaseVector),
    RobustShape(ShapeTrace),
}

impl CarrierTrace {
    pub fn exact(vector: PhaseVector) -> Self {
        Self::Exact(vector)
    }

    pub fn robust_shape(trace: ShapeTrace) -> Self {
        Self::RobustShape(trace)
    }

    pub fn similarity(&self, other: &Self) -> f32 {
        match (self, other) {
            (Self::Exact(a), Self::Exact(b)) => a.similarity(b),
            (Self::RobustShape(a), Self::RobustShape(b)) => a.similarity(b),
            _ => 0.0,
        }
    }

    pub fn exact_vector(&self) -> Option<&PhaseVector> {
        match self {
            Self::Exact(vector) => Some(vector),
            Self::RobustShape(_) => None,
        }
    }
}

impl From<PhaseVector> for CarrierTrace {
    fn from(value: PhaseVector) -> Self {
        Self::Exact(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn robust_shape_similarity_is_subset_tolerant() {
        let clean = ShapeTrace::from_fragments(
            vec![101, 102, 103, 104, 105, 106],
            vec![11, 22, 33, 44],
        );
        let dropout = ShapeTrace::from_fragments(vec![102, 104, 106], vec![22]);
        let distractor = ShapeTrace::from_fragments(
            vec![5, 101, 102, 103, 104, 105, 106, 120],
            vec![5, 11, 22, 33, 44, 99],
        );
        let rotated = ShapeTrace::from_fragments(vec![201, 202, 203], vec![11, 22, 33, 44]);
        let ambiguous_single_invariant = ShapeTrace::from_fragments(vec![301], vec![11]);

        assert_eq!(clean.similarity(&dropout), 1.0);
        assert_eq!(clean.similarity(&distractor), 1.0);
        assert_eq!(clean.similarity(&rotated), 1.0);
        assert_eq!(
            clean.similarity(&ambiguous_single_invariant),
            0.0,
            "one invariant triangle cannot override conflicting structural evidence"
        );
        assert_eq!(
            clean.similarity(&ShapeTrace::from_fragments(vec![301, 302], vec![77, 88])),
            0.0
        );
    }
}
