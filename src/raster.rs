use std::collections::BTreeMap;

use crate::hdc::PhaseVector;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OffsetCount {
    pub dx: i16,
    pub dy: i16,
    pub count: u16,
}

#[derive(Debug, Clone)]
pub struct RelationalMotif {
    pub action: usize,
    pub offsets: Vec<OffsetCount>,
    pub need: f32,
    pub confidence: f32,
    pub utility: f32,
    pub support: u32,
    pub revision: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingMotif {
    pub action: usize,
    pub offsets: Vec<OffsetCount>,
    pub target_need: bool,
    pub support: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct ActionContextTrace {
    pub action: usize,
    pub embedding: PhaseVector,
    pub visits: u32,
}

pub(crate) fn scene_embedding(
    sensory: &[f32],
    roles: &[PhaseVector],
) -> Option<PhaseVector> {
    assert_eq!(sensory.len(), roles.len());
    let active: Vec<&PhaseVector> = sensory
        .iter()
        .zip(roles)
        .filter(|(value, _)| **value >= 0.5)
        .map(|(_, role)| role)
        .collect();

    if active.is_empty() {
        None
    } else {
        Some(PhaseVector::bundle(&active))
    }
}

fn offset_histogram(
    sensory: &[f32],
    width: usize,
    height: usize,
    radius: i16,
) -> BTreeMap<(i16, i16), u16> {
    assert!(width > 0 && height > 0);
    assert_eq!(sensory.len(), width * height);

    let points: Vec<(i16, i16)> = sensory
        .iter()
        .enumerate()
        .filter(|(_, value)| **value >= 0.5)
        .map(|(index, _)| ((index % width) as i16, (index / width) as i16))
        .collect();

    let mut histogram = BTreeMap::new();
    for i in 0..points.len() {
        for j in (i + 1)..points.len() {
            let dx = points[j].0 - points[i].0;
            let dy = points[j].1 - points[i].1;
            if dx == 0 && dy == 0 {
                continue;
            }
            if dx.abs().max(dy.abs()) > radius {
                continue;
            }
            *histogram.entry((dx, dy)).or_insert(0) += 1;
        }
    }

    histogram
}

pub(crate) fn motif_signature(
    sensory: &[f32],
    width: usize,
    height: usize,
    radius: i16,
    max_offsets: usize,
) -> Vec<OffsetCount> {
    if max_offsets == 0 {
        return Vec::new();
    }

    let histogram = offset_histogram(sensory, width, height, radius);
    let mut entries: Vec<OffsetCount> = histogram
        .into_iter()
        .map(|((dx, dy), count)| OffsetCount { dx, dy, count })
        .collect();

    entries.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| a.dy.cmp(&b.dy))
            .then_with(|| a.dx.cmp(&b.dx))
    });
    entries.truncate(max_offsets);
    entries
}

pub(crate) fn motif_matches(
    motif: &RelationalMotif,
    sensory: &[f32],
    width: usize,
    height: usize,
    radius: i16,
) -> bool {
    if motif.offsets.is_empty() {
        return false;
    }

    let histogram = offset_histogram(sensory, width, height, radius);
    motif.offsets.iter().all(|required| {
        histogram
            .get(&(required.dx, required.dy))
            .copied()
            .unwrap_or(0)
            >= required.count
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raster(width: usize, height: usize, points: &[(usize, usize)]) -> Vec<f32> {
        let mut values = vec![0.0; width * height];
        for (x, y) in points {
            values[y * width + x] = 1.0;
        }
        values
    }

    #[test]
    fn relative_signature_survives_translation() {
        let a = raster(12, 12, &[(1, 1), (2, 1), (3, 1)]);
        let b = raster(12, 12, &[(7, 8), (8, 8), (9, 8)]);
        let c = raster(12, 12, &[(4, 2), (4, 3), (4, 4)]);

        let sa = motif_signature(&a, 12, 12, 3, 2);
        let sb = motif_signature(&b, 12, 12, 3, 2);
        let sc = motif_signature(&c, 12, 12, 3, 2);

        assert_eq!(sa, sb);
        assert_ne!(sa, sc);
    }
}
