use std::f32::consts::{PI, TAU};

pub fn wrap_phase(mut x: f32) -> f32 {
    while x < 0.0 {
        x += TAU;
    }
    while x >= TAU {
        x -= TAU;
    }
    x
}

pub fn signed_phase_error(target: f32, actual: f32) -> f32 {
    let mut d = wrap_phase(target) - wrap_phase(actual);
    if d > PI {
        d -= TAU;
    } else if d < -PI {
        d += TAU;
    }
    d
}

pub fn phase_similarity(a: f32, b: f32) -> f32 {
    signed_phase_error(a, b).cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_error_wraps_short_way() {
        let a = 0.05;
        let b = std::f32::consts::TAU - 0.05;
        assert!(signed_phase_error(a, b).abs() < 0.11);
    }
}
