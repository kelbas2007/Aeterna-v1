use crate::hdc::PhaseVector;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeTrace {
    oriented_fragments: Vec<u64>,
    invariant_fragments: Vec<u64>,
}

impl ShapeTrace {
    pub fn new(invariant_fragments: Vec<u64>) -> Self {
        Self::from_fragments(Vec::new(), invariant_fragments)
    }

    pub fn from_fragments(
        mut oriented_fragments: Vec<u64>,
        mut invariant_fragments: Vec<u64>,
    ) -> Self {
        oriented_fragments.sort_unstable();
        invariant_fragments.sort_unstable();
        Self {
            oriented_fragments,
            invariant_fragments,
        }
    }

    pub fn oriented_fragments(&self) -> &[u64] {
        &self.oriented_fragments
    }

    pub fn invariant_fragments(&self) -> &[u64] {
        &self.invariant_fragments
    }

    pub fn similarity(&self, other: &Self) -> f32 {
        // Two generic views coexist:
        // 1) translation-equivariant oriented pair fragments, which are strong
        //    under dropout/distractors;
        // 2) normalized triangle-shape fragments, which survive rotation/scale.
        //
        // No task label chooses individual fragments. The carrier accepts the
        // strongest factual geometric correspondence available.
        Self::containment(&self.oriented_fragments, &other.oriented_fragments)
            .max(Self::containment(
                &self.invariant_fragments,
                &other.invariant_fragments,
            ))
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

        assert_eq!(clean.similarity(&dropout), 1.0);
        assert_eq!(clean.similarity(&distractor), 1.0);
        assert_eq!(clean.similarity(&rotated), 1.0);
        assert_eq!(
            clean.similarity(&ShapeTrace::from_fragments(vec![301, 302], vec![77, 88])),
            0.0
        );
    }
}
