// Included in external_reward. Select prototypes inside the existing bounded
// memory; factual frequencies, failed trials, U1 and the action port stay shared.
const COVERAGE_SAMPLE: usize = 9;
const COVERAGE_DECISION_WORK: u64 = 65_536;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CoverageMemory {
    enabled_at: u64,
    decisions: u64,
    work: u64,
    non_fifo_evictions: u64,
    incoming_discards: u64,
    skipped_return_updates: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageMemoryReport {
    pub enabled_at: u64,
    pub decisions: u64,
    pub work: u64,
    pub non_fifo_evictions: u64,
    pub incoming_discards: u64,
    pub skipped_return_updates: u64,
    pub positive_action_mask_groups: usize,
    pub capacity: usize,
}

impl CoverageMemory {
    fn validate(&self, events: u64) -> bool {
        self.enabled_at <= events
            && self.non_fifo_evictions <= self.decisions
            && self.incoming_discards <= self.decisions
            && self.skipped_return_updates <= events - self.enabled_at
            && self.work <= self.decisions.saturating_mul(COVERAGE_DECISION_WORK)
            && (self.decisions > 0 || self.work == 0)
    }
    fn report(&self, rows: &[Case]) -> CoverageMemoryReport {
        CoverageMemoryReport {
            enabled_at: self.enabled_at,
            decisions: self.decisions,
            work: self.work,
            non_fifo_evictions: self.non_fifo_evictions,
            incoming_discards: self.incoming_discards,
            skipped_return_updates: self.skipped_return_updates,
            positive_action_mask_groups: rows
                .iter()
                .filter(|r| r.reward > 0.0)
                .map(|r| (r.pre[0], r.action, r.pre[1]))
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            capacity: CAPACITY,
        }
    }
}

impl ExternalMemory {
    pub(crate) fn enable_coverage_memory(&mut self) -> Result<()> {
        if !self.enabled
            || self.adaptive
            || !self.pending.is_empty()
            || self.coverage_memory.is_some()
            || self.consolidated_memory.is_some()
        {
            return Err(Error::Busy);
        }
        self.coverage_memory = Some(CoverageMemory {
            enabled_at: self.events,
            ..Default::default()
        });
        Ok(())
    }

    // The incoming factual prototype participates in the SAME capacity limit.
    // Preserve namespace/outcome quotas; within successes protect action/mask
    // coverage before removing redundancy. No label for an unexecuted motor.
    fn coverage_victim(&self, incoming: &Case) -> (usize, u64, bool) {
        let rows: Vec<&Case> = self.rows.iter().chain(std::iter::once(incoming)).collect();
        let mut work = 0;
        let mut buckets = BTreeMap::<(i64, bool), usize>::new();
        for r in &rows {
            work += 8;
            *buckets.entry((r.pre[0], r.reward > 0.0)).or_default() += 1;
        }
        let bucket = *buckets
            .iter()
            .max_by(|(ka, na), (kb, nb)| {
                let wa = if ka.1 { 3 } else { 1 };
                let wb = if kb.1 { 3 } else { 1 };
                (**na * wb, ka).cmp(&(**nb * wa, kb))
            })
            .unwrap()
            .0;
        let mut members = Vec::new();
        for (i, r) in rows.iter().enumerate() {
            work += 2;
            if (r.pre[0], r.reward > 0.0) == bucket {
                members.push(i);
            }
        }
        let fifo = members[0];
        if !bucket.1 {
            return (fifo, work, false);
        }

        let mut groups = BTreeMap::<(usize, i64), usize>::new();
        for &i in &members {
            work += 8;
            *groups.entry((rows[i].action, rows[i].pre[1])).or_default() += 1;
        }
        // Never delete a sole action/mask representative while the same
        // namespace has a repeated group. If all groups are singletons the
        // finite capacity cannot promise to preserve every possible mask.
        let group = *groups
            .iter()
            .max_by_key(|((_, mask), count)| (**count, std::cmp::Reverse(mask.count_ones())))
            .unwrap()
            .0;
        work += groups.len() as u64 * 2;
        let mut pool = Vec::new();
        for &i in &members {
            work += 2;
            if (rows[i].action, rows[i].pre[1]) == group {
                pool.push(i);
            }
        }
        let mut sample = Vec::new();
        // Bounded rotating inspection, independent of hidden class labels.
        // Include the new point when it belongs to the selected group.
        if pool.contains(&self.rows.len()) {
            sample.push(self.rows.len());
        }
        let slots = COVERAGE_SAMPLE - sample.len();
        let offset = incoming.id as usize % pool.len();
        for j in 0..slots.min(pool.len()) {
            let index = pool[(offset + j * pool.len() / slots) % pool.len()];
            if !sample.contains(&index) {
                sample.push(index);
            }
        }
        if pool.len() <= COVERAGE_SAMPLE {
            sample = pool.clone();
        }
        if sample.len() < 2 {
            return (pool[0], work, pool[0] != fifo);
        }
        let mut pair = None;
        for a in 0..sample.len() {
            for b in a + 1..sample.len() {
                work += 33;
                let left = sample[a];
                let right = sample[b];
                let d = self
                    .distance(&rows[left].pre, &rows[right].pre, &self.strategy.active)
                    .unwrap();
                let older = if rows[left].id < rows[right].id {
                    left
                } else {
                    right
                };
                let key = (d, rows[older].id, older);
                if pair.is_none_or(|best| key < best) {
                    pair = Some(key);
                }
            }
        }
        let victim = pair.unwrap().2;
        debug_assert!(work <= COVERAGE_DECISION_WORK);
        (victim, work, victim != fifo)
    }

    // Episode returns update retained facts in place. Evicted prototypes must
    // not resurrect just because an old pending episode is being finished.
    fn coverage_update_return(&mut self, row: Case) {
        if let Some(i) = self.rows.iter().position(|r| r.id == row.id) {
            let bytes = self
                .bytes
                .saturating_sub(self.rows[i].bytes())
                .saturating_add(row.bytes());
            if bytes <= MODEL_BYTES {
                self.rows[i] = row;
                self.bytes = bytes;
                return;
            }
        }
        self.coverage_memory
            .as_mut()
            .unwrap()
            .skipped_return_updates += 1;
    }
}

#[cfg(test)]
mod coverage_memory_tests {
    use super::*;

    fn row(id: u64, action: usize, mask: i64, x: i64) -> Case {
        let mut pre = vec![0; 32];
        pre[0] = 9;
        pre[1] = mask;
        pre[2] = x;
        Case {
            id,
            post: pre.clone(),
            pre,
            action,
            reward: 1.0,
            cost: 1,
            value: 1.0,
            distance: 1,
        }
    }

    #[test]
    fn capacity_pressure_preserves_sole_measurement_context_and_factual_rates() {
        let mut m = ExternalMemory::new(true, false);
        m.insert(row(1, 3, 7, 100));
        for i in 2..=CAPACITY as u64 {
            m.insert(row(i, 2, 1, i as i64));
        }
        m.events = CAPACITY as u64;
        m.learned = m.events;
        let mut old = m.clone();
        m.enable_coverage_memory().unwrap();
        let rates = fingerprint(&m.reward_rates);
        let next = row(CAPACITY as u64 + 1, 2, 1, 500);
        old.insert(next.clone());
        m.insert(next);
        assert!(!old.rows.iter().any(|r| r.id == 1));
        assert!(m.rows.iter().any(|r| r.id == 1));
        assert_eq!(m.rows.len(), CAPACITY);
        assert_eq!(rates, fingerprint(&m.reward_rates));
        m.events += 1;
        m.learned += 1;
        m.validate(16, m.events, false).unwrap();
        let saved = serde_json::to_string(&m).unwrap();
        let restored: ExternalMemory = serde_json::from_str(&saved).unwrap();
        assert_eq!(saved, serde_json::to_string(&restored).unwrap());
        assert!(
            restored
                .report()
                .coverage_memory
                .unwrap()
                .non_fifo_evictions
                > 0
        );
    }

    #[test]
    fn redundant_twin_is_removed_before_isolated_prototype() {
        let mut m = ExternalMemory::new(true, false);
        m.rows = vec![row(1, 2, 1, 1000), row(2, 2, 1, 0), row(3, 2, 1, 0)];
        let (victim, work, _) = m.coverage_victim(&row(4, 2, 1, 2000));
        assert!(victim == 1 || victim == 2);
        assert!(work > 0 && work <= COVERAGE_DECISION_WORK);
    }

    #[test]
    fn finishing_episode_cannot_resurrect_evicted_fact_and_frozen_feedback_keeps_policy() {
        let mut m = ExternalMemory::new(true, false);
        m.enable_coverage_memory().unwrap();
        let missing = row(1, 3, 7, 100);
        m.events = 1;
        m.learned = 1;
        m.pending.push(Pending {
            row: missing.clone(),
            fields: None,
        });
        m.finish();
        assert!(m.rows.is_empty());
        assert_eq!(
            m.coverage_memory.as_ref().unwrap().skipped_return_updates,
            1
        );
        let before = m.report().learned_fingerprint;
        m.observe(missing.pre.clone(), 3, missing.post, 1.0, 1, None, false);
        assert_eq!(before, m.report().learned_fingerprint);
        m.validate(16, 2, false).unwrap();
        m.coverage_memory.as_mut().unwrap().work = COVERAGE_DECISION_WORK + 1;
        assert!(m.validate(16, 2, false).is_err());
    }
}
