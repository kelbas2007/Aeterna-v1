// Included in external_reward. A mixture over the SAME retained factual rows.
// Unobserved prototype features are marginalized using the learner's own
// outcome moments; they are never interpreted as observed zeroes. Neither
// corpus identities, class labels nor new fitted prototypes enter this model.
impl ConsolidatedMemory {
    fn predict_partial(
        &self,
        pre: &[i64],
        actions: usize,
        rows: &[Case],
        meter: &mut Meter,
    ) -> Option<(Vec<f64>, Vec<u64>)> {
        if actions > 16 || pre[1] == 0 {
            return None;
        }
        let d = self.domains.get(&pre[0])?;
        let mut count = [0_usize; 16];
        for row in rows {
            meter.charge(1).ok()?;
            if row.pre[0] == pre[0] && row.reward > 0.0 && row.action < actions {
                count[row.action] += 1;
            }
        }
        if count.iter().filter(|&&n| n > 0).count() < 2 {
            return None;
        }
        // Cache class-feature arithmetic once, then integrate each component.
        let mut mean = [[0.0; 30]; 16];
        let mut inv_known = [[0.0; 30]; 16];
        let mut log_known = [[0.0; 30]; 16];
        let mut missing_distance = [[0.0; 30]; 16];
        let mut missing_score = [[0.0; 30]; 16];
        let features = (0..30)
            .filter(|&j| pre[1] & (1 << j) != 0)
            .collect::<Vec<_>>();
        for a in 0..actions {
            if count[a] == 0 {
                continue;
            }
            let shrink = (count[a] as f64).sqrt();
            let c = &d.outcomes[a].features;
            let g = &d.global.features;
            for &j in &features {
                meter.charge(32).ok()?;
                let gv = g.m2[j] / g.count[j].max(1) as f64;
                mean[a][j] = if c.count[j] > 0 { c.mean[j] } else { g.mean[j] };
                let cv = if c.count[j] > 0 {
                    c.m2[j] / c.count[j] as f64
                } else {
                    gv
                };
                let variance = cv.max(gv * 0.05).max(64.0);
                let known = (variance / shrink).max(64.0);
                inv_known[a][j] = 1.0 / known;
                log_known[a][j] = known.ln();
                let delta = pre[j + 2] as f64 - mean[a][j];
                missing_distance[a][j] = delta * delta / variance;
                missing_score[a][j] = -0.5 * (variance.ln() + missing_distance[a][j]);
            }
        }
        let mut maximum = [f64::NEG_INFINITY; 16];
        let mut mass = [0.0_f64; 16];
        let mut squared_mass = [0.0_f64; 16];
        let mut nearest = [f64::INFINITY; 16];
        for row in rows {
            meter.charge(1).ok()?;
            if row.pre[0] != pre[0] || row.reward <= 0.0 || row.action >= actions {
                continue;
            }
            let a = row.action;
            let mut score = 0.0;
            let mut distance = 0.0;
            for &j in &features {
                meter.charge(2).ok()?;
                if row.pre[1] & (1 << j) != 0 {
                    let delta = pre[j + 2] as f64 - row.pre[j + 2] as f64;
                    let z = delta * delta * inv_known[a][j];
                    distance += z;
                    score -= 0.5 * (log_known[a][j] + z);
                } else {
                    distance += missing_distance[a][j];
                    score += missing_score[a][j];
                }
            }
            nearest[a] = nearest[a].min(distance / features.len() as f64);
            meter.charge(32).ok()?;
            // Stable log-sum-exp and effective component support. Scanning
            // order and serialized factual rows fully determine the result.
            if score > maximum[a] {
                let ratio = (maximum[a] - score).exp();
                mass[a] = mass[a] * ratio + 1.0;
                squared_mass[a] = squared_mass[a] * ratio * ratio + 1.0;
                maximum[a] = score;
            } else {
                let ratio = (score - maximum[a]).exp();
                mass[a] += ratio;
                squared_mass[a] += ratio * ratio;
            }
        }
        let mut logits = (0..actions)
            .map(|a| {
                if count[a] == 0 {
                    f64::NEG_INFINITY
                } else {
                    maximum[a] + mass[a].ln() - (count[a] as f64).ln()
                }
            })
            .collect::<Vec<_>>();
        let best = (0..actions).max_by(|&a, &b| logits[a].total_cmp(&logits[b]))?;
        if !logits[best].is_finite() {
            return None;
        }
        let top = logits[best];
        let coverage = (-nearest[best].sqrt() / 4.0).exp();
        let mut total = 0.0;
        for value in &mut logits {
            meter.charge(32).ok()?;
            *value = (*value - top).exp();
            total += *value;
        }
        for value in &mut logits {
            *value = (*value / total * coverage).clamp(0.0, 1.0);
        }
        let support = (0..actions)
            .map(|a| {
                if count[a] == 0 {
                    0
                } else {
                    (mass[a] * mass[a] / squared_mass[a]).round().max(1.0) as u64
                }
            })
            .collect();
        Some((logits, support))
    }
}

#[cfg(test)]
mod partial_posterior_tests {
    use super::*;
    fn state(mask: i64, x: i64, y: i64) -> Vec<i64> {
        let mut s = vec![0; 32];
        s[0] = 9;
        s[1] = mask;
        s[2] = x;
        s[3] = y;
        s
    }
    #[test]
    fn multimodal_partial_facts_survive_without_fabricated_full_observations() {
        let mut m = ExternalMemory::new(true, false);
        for x in [-1000, 1000] {
            let s = state(1, x, 0);
            m.observe(s.clone(), 2, s, 1.0, 1, None, true);
        }
        let s = state(1, 0, 0);
        m.observe(s.clone(), 3, s, 1.0, 1, None, true);
        m.finish();
        m.enable_consolidated_memory().unwrap();
        m.enable_partial_posterior().unwrap();
        let rows = serde_json::to_string(&m.rows).unwrap();
        // A distant second measurement was never made in any prototype.
        // It is integrated, rather than dropping every partial row.
        let query = state(3, -990, 5000);
        let (p, _) = m
            .consolidated_prediction(&query, 16, None, &mut Meter::new(100_000))
            .unwrap();
        assert!(p[2] > p[3]);
        assert_eq!(rows, serde_json::to_string(&m.rows).unwrap());
        assert!(m
            .consolidated_prediction(&query, 16, None, &mut Meter::new(1024))
            .is_none());
        m.observe(query.clone(), 2, query.clone(), 0.0, 1, None, true);
        m.finish();
        let (p, _) = m
            .consolidated_prediction(&query, 16, None, &mut Meter::new(100_000))
            .unwrap();
        assert_eq!(p[2], 0.0, "actual contradictions retain authority");
    }
}
