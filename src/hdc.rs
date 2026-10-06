use crate::phase::{phase_similarity, wrap_phase};

#[derive(Debug, Clone, PartialEq)]
pub struct PhaseVector {
    phases: Vec<f32>,
}

impl PhaseVector {
    pub fn from_seed(dim: usize, seed: u64) -> Self {
        let mut x = seed ^ 0x9E37_79B9_7F4A_7C15;
        let mut phases = Vec::with_capacity(dim);
        for _ in 0..dim {
            // xorshift64*: deterministic; not used for security.
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            let z = x.wrapping_mul(0x2545_F491_4F6C_DD1D);
            let unit = (z as f64 / u64::MAX as f64) as f32;
            phases.push(unit * std::f32::consts::TAU);
        }
        Self { phases }
    }

    pub fn dim(&self) -> usize {
        self.phases.len()
    }

    pub fn bind(&self, other: &Self) -> Self {
        assert_eq!(self.dim(), other.dim());
        let phases = self
            .phases
            .iter()
            .zip(&other.phases)
            .map(|(a, b)| wrap_phase(*a + *b))
            .collect();
        Self { phases }
    }

    pub fn powi(&self, power: i32) -> Self {
        let scale = power as f32;
        let phases = self
            .phases
            .iter()
            .map(|phase| wrap_phase(*phase * scale))
            .collect();
        Self { phases }
    }

    pub fn permute_dims(&self, shift: usize) -> Self {
        if self.phases.is_empty() {
            return self.clone();
        }
        let shift = shift % self.phases.len();
        let mut phases = vec![0.0; self.phases.len()];
        for (idx, phase) in self.phases.iter().copied().enumerate() {
            phases[(idx + shift) % self.phases.len()] = phase;
        }
        Self { phases }
    }

    pub fn unbind(&self, role: &Self) -> Self {
        assert_eq!(self.dim(), role.dim());
        let phases = self
            .phases
            .iter()
            .zip(&role.phases)
            .map(|(a, b)| wrap_phase(*a - *b))
            .collect();
        Self { phases }
    }

    pub fn bundle(vectors: &[&Self]) -> Self {
        assert!(!vectors.is_empty());
        let dim = vectors[0].dim();
        assert!(vectors.iter().all(|v| v.dim() == dim));

        let mut phases = Vec::with_capacity(dim);
        for d in 0..dim {
            let mut x = 0.0f32;
            let mut y = 0.0f32;
            for vector in vectors {
                x += vector.phases[d].cos();
                y += vector.phases[d].sin();
            }

            let phase = if x.abs() + y.abs() <= 1.0e-8 {
                0.0
            } else {
                wrap_phase(y.atan2(x))
            };
            phases.push(phase);
        }

        Self { phases }
    }

    pub fn similarity(&self, other: &Self) -> f32 {
        assert_eq!(self.dim(), other.dim());
        let total: f32 = self
            .phases
            .iter()
            .zip(&other.phases)
            .map(|(a, b)| phase_similarity(*a, *b))
            .sum();
        total / self.dim() as f32
    }

    pub fn phases(&self) -> &[f32] {
        &self.phases
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_unbind_round_trip() {
        let a = PhaseVector::from_seed(128, 1);
        let b = PhaseVector::from_seed(128, 2);
        let recovered = a.bind(&b).unbind(&a);
        assert!(recovered.similarity(&b) > 0.999);
    }

    #[test]
    fn translated_group_relation_cancels_common_phase_role() {
        let x = PhaseVector::from_seed(128, 101);
        let y = PhaseVector::from_seed(128, 202);

        let p1 = x.powi(2).bind(&y.powi(3));
        let p2 = x.powi(4).bind(&y.powi(3));
        let q1 = x.powi(7).bind(&y.powi(8));
        let q2 = x.powi(9).bind(&y.powi(8));

        let r1 = p2.unbind(&p1);
        let r2 = q2.unbind(&q1);
        assert!(r1.similarity(&r2) > 0.999);
    }

    #[test]
    fn identical_bundles_are_stable() {
        let a = PhaseVector::from_seed(128, 11);
        let b = PhaseVector::from_seed(128, 22);
        let first = PhaseVector::bundle(&[&a, &b]);
        let second = PhaseVector::bundle(&[&a, &b]);
        assert!(first.similarity(&second) > 0.999);
    }

    #[test]
    fn dimension_permutation_is_deterministic_and_order_sensitive() {
        let a = PhaseVector::from_seed(128, 31);
        let b = PhaseVector::from_seed(128, 32);
        let left = a.permute_dims(1).bind(&b);
        let right = b.permute_dims(1).bind(&a);
        assert!(left.similarity(&left) > 0.999);
        assert!(
            left.similarity(&right) < 0.95,
            "sequence permutation must distinguish reversed histories"
        );
    }
}
