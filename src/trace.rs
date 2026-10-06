use crate::hdc::PhaseVector;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeTrace {
    fragments: Vec<u64>,
}

impl ShapeTrace {
    pub fn new(mut fragments: Vec<u64>) -> Self {
        fragments.sort_unstable();
        Self { fragments }
    }

    pub fn fragments(&self) -> &[u64] {
        &self.fragments
    }

    pub fn similarity(&self, other: &Self) -> f32 {
        if self.fragments.is_empty() || other.fragments.is_empty() {
            return 0.0;
        }

        // Multiset containment, normalized by the smaller observation.
        // A clean four-point relation has four triangle fragments. One dropped
        // point leaves one of those learned fragments; one distractor preserves
        // all original fragments while adding extras. Rotation and uniform scale
        // preserve every normalized triangle fragment.
        let mut i = 0usize;
        let mut j = 0usize;
        let mut overlap = 0usize;

        while i < self.fragments.len() && j < other.fragments.len() {
            match self.fragments[i].cmp(&other.fragments[j]) {
                std::cmp::Ordering::Less => i += 1,
                std::cmp::Ordering::Greater => j += 1,
                std::cmp::Ordering::Equal => {
                    overlap += 1;
                    i += 1;
                    j += 1;
                }
            }
        }

        overlap as f32 / self.fragments.len().min(other.fragments.len()) as f32
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
        let clean = ShapeTrace::new(vec![11, 22, 33, 44]);
        let dropout = ShapeTrace::new(vec![22]);
        let distractor = ShapeTrace::new(vec![5, 11, 22, 33, 44, 99]);

        assert_eq!(clean.similarity(&dropout), 1.0);
        assert_eq!(clean.similarity(&distractor), 1.0);
        assert_eq!(clean.similarity(&ShapeTrace::new(vec![77, 88])), 0.0);
    }
}
