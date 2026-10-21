// Included in external_reward: these programs belong to the same factual
// memory, not a second learner/actuator. The induction algorithm and predicates
// are authored. The trees, thresholds and motor associations are acquired.

const RULE_TREES: usize = 7;
const RULE_NODES: usize = 63;
const RULE_SAMPLES: usize = 512;
const RULE_WORK: u64 = 4_000_000;
const RULE_LIFETIME_WORK: u64 = 64_000_000;
const RULE_EXAM: usize = 64;
const RULE_KEYS: usize = 8192;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RewardNode {
    votes: [u16; 16],
    split: Option<(usize, i64, bool, usize, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RewardProgram {
    trees: Vec<Vec<RewardNode>>,
    trained_through: u64,
}

fn rule_distribution(counts: &[u16; 16]) -> [f64; 16] {
    let total = counts.iter().map(|&n| f64::from(n)).sum::<f64>();
    std::array::from_fn(|a| f64::from(counts[a]) / total.max(1.0))
}

fn rule_normalize(values: &[f64]) -> [f64; 16] {
    let total = values.iter().sum::<f64>();
    std::array::from_fn(|a| values[a] / total.max(1e-12))
}

fn rule_best(values: &[f64; 16]) -> usize {
    (0..16)
        .max_by(|&a, &b| values[a].total_cmp(&values[b]).then_with(|| b.cmp(&a)))
        .unwrap()
}

fn rule_loss(values: &[f64; 16], action: usize) -> f64 {
    values
        .iter()
        .enumerate()
        .map(|(a, &p)| (p - f64::from(a == action)).powi(2))
        .sum()
}

fn rule_random(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

impl RewardProgram {
    fn infer(&self, state: &[i64], meter: &mut Meter) -> Option<[f64; 16]> {
        if state[1] == 0 {
            return None;
        }
        let mut result = [0.0; 16];
        // Reserve the entire inference so a truncated forest is never used.
        meter
            .charge((self.trees.len() * (RULE_NODES + 16)) as u64)
            .ok()?;
        for tree in &self.trees {
            let mut index = 0;
            for _ in 0..RULE_NODES {
                let node = &tree[index];
                if let Some((axis, threshold, missing_left, left, right)) = node.split {
                    let go_left = if state[1] & (1 << axis) == 0 {
                        missing_left
                    } else {
                        state[axis + 2] <= threshold
                    };
                    index = if go_left { left } else { right };
                } else {
                    let p = rule_distribution(&node.votes);
                    for a in 0..16 {
                        result[a] += p[a] / self.trees.len() as f64;
                    }
                    break;
                }
            }
        }
        Some(result)
    }

    fn build(rows: &[&Case], event: u64, meter: &mut Meter) -> Option<Self> {
        if rows.len() < 96 {
            return None;
        }
        let mut seed = event.wrapping_mul(0x9e3779b97f4a7c15) | 1;
        let mut trees = Vec::new();
        for _ in 0..RULE_TREES {
            let samples: Vec<_> = (0..rows.len())
                .map(|_| rows[rule_random(&mut seed) as usize % rows.len()])
                .collect();
            let mut tree = Vec::new();
            Self::grow(&samples, 0, RULE_NODES, &mut tree, &mut seed, meter)?;
            trees.push(tree);
        }
        Some(Self {
            trees,
            trained_through: event,
        })
    }

    fn grow(
        rows: &[&Case],
        depth: usize,
        capacity: usize,
        tree: &mut Vec<RewardNode>,
        seed: &mut u64,
        meter: &mut Meter,
    ) -> Option<usize> {
        meter.charge(rows.len() as u64 + 16).ok()?;
        let mut counts = [0u16; 16];
        for row in rows {
            counts[row.action] += 1;
        }
        let index = tree.len();
        tree.push(RewardNode {
            votes: counts,
            split: None,
        });
        if rows.len() < 12
            || depth == 7
            || capacity < 3
            || counts.iter().filter(|&&n| n > 0).count() < 2
        {
            return Some(index);
        }
        let impurity = |c: &[u16; 16]| {
            let n = c.iter().map(|&x| f64::from(x)).sum::<f64>();
            n - c.iter().map(|&x| f64::from(x).powi(2)).sum::<f64>() / n.max(1.0)
        };
        let mut best_gain = 0.5;
        let mut best = None;
        let mut axes: Vec<_> = (0..30)
            .filter(|&a| rows.iter().any(|r| r.pre[1] & (1 << a) != 0))
            .collect();
        for i in (1..axes.len()).rev() {
            let j = rule_random(seed) as usize % (i + 1);
            axes.swap(i, j);
        }
        // Data-selected predicates; no sensor names, class labels or task IDs.
        for &axis in axes.iter().take(8) {
            meter.charge(rows.len() as u64 * 10).ok()?;
            let mut values: Vec<_> = rows
                .iter()
                .filter(|r| r.pre[1] & (1 << axis) != 0)
                .copied()
                .collect();
            values.sort_unstable_by_key(|r| r.pre[axis + 2]);
            if values.len() < 2 {
                continue;
            }
            let mut missing = counts;
            for r in &values {
                missing[r.action] -= 1;
            }
            let mut cursor = 0;
            let mut known_left = [0u16; 16];
            for q in 1..8 {
                let threshold = values[(q * values.len() / 8).min(values.len() - 2)].pre[axis + 2];
                while cursor < values.len() && values[cursor].pre[axis + 2] <= threshold {
                    known_left[values[cursor].action] += 1;
                    cursor += 1;
                }
                for missing_left in [false, true] {
                    meter.charge(64).ok()?;
                    let left = std::array::from_fn(|a| {
                        known_left[a] + if missing_left { missing[a] } else { 0 }
                    });
                    let right = std::array::from_fn(|a| counts[a] - left[a]);
                    if left.iter().sum::<u16>() < 4 || right.iter().sum::<u16>() < 4 {
                        continue;
                    }
                    let gain = impurity(&counts) - impurity(&left) - impurity(&right);
                    if gain > best_gain {
                        best_gain = gain;
                        best = Some((axis, threshold, missing_left));
                    }
                }
            }
        }
        if let Some((axis, threshold, missing_left)) = best {
            let (left, right): (Vec<_>, Vec<_>) = rows.iter().copied().partition(|r| {
                if r.pre[1] & (1 << axis) == 0 {
                    missing_left
                } else {
                    r.pre[axis + 2] <= threshold
                }
            });
            let left_at = Self::grow(&left, depth + 1, (capacity - 1) / 2, tree, seed, meter)?;
            let remaining = capacity - (tree.len() - index);
            let right_at = Self::grow(&right, depth + 1, remaining, tree, seed, meter)?;
            tree[index].split = Some((axis, threshold, missing_left, left_at, right_at));
        }
        Some(index)
    }

    fn validate(&self, events: u64) -> bool {
        self.trained_through <= events
            && self.trees.len() == RULE_TREES
            && self.trees.iter().all(|tree| {
                if tree.is_empty() || tree.len() > RULE_NODES {
                    return false;
                }
                let mut seen = std::collections::BTreeSet::new();
                let mut stack = vec![(0, 0)];
                while let Some((i, depth)) = stack.pop() {
                    if i >= tree.len() || !seen.insert(i) || depth > 7 {
                        return false;
                    }
                    let node = &tree[i];
                    let total = node.votes.iter().map(|&x| usize::from(x)).sum::<usize>();
                    if total == 0 || total > RULE_SAMPLES {
                        return false;
                    }
                    if let Some((axis, _, _, l, r)) = node.split {
                        if axis >= 30 || l == r {
                            return false;
                        }
                        stack.push((l, depth + 1));
                        stack.push((r, depth + 1));
                    }
                }
                seen.len() == tree.len()
            })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquiredRuleAudit {
    pub trained_through: u64,
    pub closed_at: u64,
    pub future_examples: usize,
    pub incumbent_correct: usize,
    pub candidate_correct: usize,
    pub fixed_correct: usize,
    pub incumbent_loss: f64,
    pub candidate_loss: f64,
    pub fixed_loss: f64,
    pub accepted: bool,
    pub program_fingerprint: u64,
    pub incumbent_program: Option<u64>,
}

impl AcquiredRuleAudit {
    fn qualifies(&self) -> bool {
        self.candidate_correct >= self.incumbent_correct + 3
            && self.candidate_correct >= self.fixed_correct + 3
            && self.candidate_loss + 0.64 < self.incumbent_loss * 0.98
            && self.candidate_loss + 0.64 < self.fixed_loss * 0.98
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RuleTrial {
    program: RewardProgram,
    comparison: AcquiredRuleAudit,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct RuleDomain {
    active: Option<RewardProgram>,
    shadow: Option<RuleTrial>,
    seen: std::collections::BTreeSet<u64>,
    successes: u64,
    next_proposal: u64,
    audits: Vec<AcquiredRuleAudit>,
    monitoring: usize,
    fixed_correct: usize,
    active_correct: usize,
    rollbacks: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AcquiredRules {
    domains: BTreeMap<i64, RuleDomain>,
    work: u64,
    proposals: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquiredRulesReport {
    pub work: u64,
    pub proposals: u64,
    pub admitted_programs: usize,
    pub active_domains: Vec<i64>,
    pub learned_predicates: usize,
    pub pending_programs: usize,
    pub rollbacks: u64,
    pub audits: BTreeMap<i64, Vec<AcquiredRuleAudit>>,
}

impl AcquiredRules {
    fn report(&self) -> AcquiredRulesReport {
        AcquiredRulesReport {
            work: self.work,
            proposals: self.proposals,
            admitted_programs: self
                .domains
                .values()
                .flat_map(|d| &d.audits)
                .filter(|a| a.accepted)
                .count(),
            active_domains: self
                .domains
                .iter()
                .filter(|(_, d)| d.active.is_some())
                .map(|(&n, _)| n)
                .collect(),
            learned_predicates: self
                .domains
                .values()
                .filter_map(|d| d.active.as_ref())
                .flat_map(|p| &p.trees)
                .flatten()
                .filter(|n| n.split.is_some())
                .count(),
            pending_programs: self.domains.values().filter(|d| d.shadow.is_some()).count(),
            rollbacks: self.domains.values().map(|d| d.rollbacks).sum(),
            audits: self
                .domains
                .iter()
                .map(|(&n, d)| (n, d.audits.clone()))
                .collect(),
        }
    }

    fn observe(
        &mut self,
        row: &Case,
        fixed: [f64; 16],
        old: [f64; 16],
        candidate: Option<[f64; 16]>,
        rows: &[Case],
    ) -> u64 {
        if row.reward <= 0.0 || row.pre[1] == 0 || self.work >= RULE_LIFETIME_WORK {
            return 0;
        }
        if !self.domains.contains_key(&row.pre[0]) && self.domains.len() >= 8 {
            return 0;
        }
        let domain = self.domains.entry(row.pre[0]).or_default();
        let key = fingerprint(&row.pre);
        if domain.seen.len() >= RULE_KEYS || !domain.seen.insert(key) {
            return 0;
        }
        domain.successes += 1;
        let mut meter = Meter::new(RULE_WORK.min(RULE_LIFETIME_WORK - self.work));
        if domain.active.is_some() {
            domain.monitoring += 1;
            domain.fixed_correct += usize::from(rule_best(&fixed) == row.action);
            domain.active_correct += usize::from(rule_best(&old) == row.action);
            if domain.monitoring == RULE_EXAM {
                if domain.active_correct + 3 < domain.fixed_correct {
                    domain.active = None;
                    domain.rollbacks += 1;
                }
                domain.monitoring = 0;
                domain.fixed_correct = 0;
                domain.active_correct = 0;
            }
        }
        if let Some(trial) = &mut domain.shadow {
            if let Some(candidate) = candidate {
                let audit = &mut trial.comparison;
                audit.future_examples += 1;
                audit.incumbent_correct += usize::from(rule_best(&old) == row.action);
                audit.candidate_correct += usize::from(rule_best(&candidate) == row.action);
                audit.fixed_correct += usize::from(rule_best(&fixed) == row.action);
                audit.incumbent_loss += rule_loss(&old, row.action);
                audit.candidate_loss += rule_loss(&candidate, row.action);
                audit.fixed_loss += rule_loss(&fixed, row.action);
                if audit.future_examples == RULE_EXAM {
                    let mut trial = domain.shadow.take().unwrap();
                    trial.comparison.closed_at = row.id;
                    trial.comparison.accepted = trial.comparison.qualifies();
                    if trial.comparison.accepted {
                        domain.active = Some(trial.program);
                        domain.monitoring = 0;
                        domain.fixed_correct = 0;
                        domain.active_correct = 0;
                    }
                    domain.audits.push(trial.comparison);
                    domain.next_proposal = domain.successes + 64;
                }
            }
        }
        if domain.shadow.is_none()
            && domain.successes >= domain.next_proposal
            && domain.audits.len() < 8
        {
            let mut training: Vec<_> = rows
                .iter()
                .filter(|r| r.pre[0] == row.pre[0] && r.reward > 0.0 && r.pre[1] != 0)
                .collect();
            training.push(row);
            if training.len() >= 96 {
                if training.len() > RULE_SAMPLES {
                    training = training[training.len() - RULE_SAMPLES..].to_vec();
                }
                // Training inputs never subsequently count as fresh evidence,
                // including pre-existing experience when enabling after load.
                for r in &training {
                    if domain.seen.len() < RULE_KEYS {
                        domain.seen.insert(fingerprint(&r.pre));
                    }
                }
                if let Some(program) = RewardProgram::build(&training, row.id, &mut meter) {
                    let comparison = AcquiredRuleAudit {
                        trained_through: row.id,
                        closed_at: 0,
                        future_examples: 0,
                        incumbent_correct: 0,
                        candidate_correct: 0,
                        fixed_correct: 0,
                        incumbent_loss: 0.0,
                        candidate_loss: 0.0,
                        fixed_loss: 0.0,
                        accepted: false,
                        program_fingerprint: fingerprint(&program),
                        incumbent_program: domain.active.as_ref().map(fingerprint),
                    };
                    domain.shadow = Some(RuleTrial {
                        program,
                        comparison,
                    });
                    self.proposals += 1;
                }
                domain.next_proposal = domain.successes + 64;
            }
        }
        self.work += meter.spent;
        meter.spent
    }

    fn validate(&self, events: u64) -> bool {
        self.work <= RULE_LIFETIME_WORK
            && self.proposals
                == self
                    .domains
                    .values()
                    .map(|d| d.audits.len() as u64 + u64::from(d.shadow.is_some()))
                    .sum::<u64>()
            && self.domains.len() <= 8
            && self.domains.iter().all(|(&n, d)| {
                (0..=262143).contains(&n)
                    && d.seen.len() <= RULE_KEYS
                    && d.successes <= events
                    && d.successes <= d.seen.len() as u64
                    && d.audits.len() <= 8
                    && d.active.as_ref().is_none_or(|p| p.validate(events))
                    && d.monitoring < RULE_EXAM
                    && d.fixed_correct <= d.monitoring
                    && d.active_correct <= d.monitoring
                    && d.rollbacks <= d.audits.iter().filter(|a| a.accepted).count() as u64
                    && d.active.as_ref().is_none_or(|p| {
                        d.audits
                            .iter()
                            .any(|a| a.accepted && a.program_fingerprint == fingerprint(p))
                    })
                    && d.audits.iter().all(|a| {
                        a.trained_through < a.closed_at
                            && a.closed_at <= events
                            && a.future_examples == RULE_EXAM
                            && a.incumbent_correct <= RULE_EXAM
                            && a.candidate_correct <= RULE_EXAM
                            && a.incumbent_loss.is_finite()
                            && a.candidate_loss.is_finite()
                            && a.incumbent_loss >= 0.0
                            && a.candidate_loss >= 0.0
                            && a.fixed_correct <= RULE_EXAM
                            && a.fixed_loss.is_finite()
                            && a.fixed_loss >= 0.0
                            && a.accepted == a.qualifies()
                    })
                    && d.shadow.as_ref().is_none_or(|t| {
                        t.program.validate(events)
                            && t.comparison.future_examples < RULE_EXAM
                            && t.comparison.trained_through == t.program.trained_through
                            && t.comparison.closed_at == 0
                            && t.comparison.program_fingerprint == fingerprint(&t.program)
                            && t.comparison.incumbent_correct <= t.comparison.future_examples
                            && t.comparison.candidate_correct <= t.comparison.future_examples
                            && t.comparison.fixed_correct <= t.comparison.future_examples
                            && t.comparison.fixed_loss.is_finite()
                            && t.comparison.fixed_loss >= 0.0
                            && t.comparison.incumbent_loss.is_finite()
                            && t.comparison.candidate_loss.is_finite()
                            && t.comparison.incumbent_loss >= 0.0
                            && t.comparison.candidate_loss >= 0.0
                            && !t.comparison.accepted
                    })
            })
    }
}

#[cfg(test)]
mod acquired_rule_tests {
    use super::*;

    fn example(id: u64) -> Case {
        let mut pre = vec![0; 32];
        pre[0] = 4;
        pre[1] = 1;
        pre[2] = id as i64;
        Case {
            id,
            action: if id % 256 < 128 { 2 } else { 11 },
            post: pre.clone(),
            pre,
            reward: 1.0,
            cost: 1,
            value: 1.0,
            distance: 1,
        }
    }
    fn distribution(action: usize) -> [f64; 16] {
        std::array::from_fn(|a| f64::from(a == action))
    }

    #[test]
    fn measured_predicates_are_acquired_and_missing_is_distinct_from_zero() {
        let rows: Vec<_> = (1..=255).map(example).collect();
        let refs: Vec<_> = rows.iter().collect();
        let mut meter = Meter::new(RULE_WORK);
        let program = RewardProgram::build(&refs, 255, &mut meter).unwrap();
        assert!(program.validate(255));
        for id in [12, 90, 165, 240] {
            let row = example(id);
            assert_eq!(
                rule_best(&program.infer(&row.pre, &mut Meter::new(10_000)).unwrap()),
                row.action
            );
        }
        let mut missing = example(12).pre;
        missing[1] = 0;
        assert!(program.infer(&missing, &mut Meter::new(10_000)).is_none());
        let mut tiny = Meter::new(1);
        assert!(program.infer(&example(12).pre, &mut tiny).is_none());
        let mut broken = program.clone();
        broken.trees[0][0].split = Some((0, 123, false, 0, 1));
        assert!(
            !broken.validate(255),
            "persisted cycles must fail before execution"
        );
    }

    #[test]
    fn future_gate_survives_restore_rejects_repeated_inputs_and_rolls_back_regression() {
        let rows: Vec<_> = (1..=128).map(example).collect();
        let mut book = AcquiredRules::default();
        let row = example(129);
        book.observe(&row, distribution(0), distribution(0), None, &rows);
        assert_eq!(book.proposals, 1);
        assert_eq!(
            book.domains[&4]
                .shadow
                .as_ref()
                .unwrap()
                .comparison
                .future_examples,
            0
        );
        // Training examples and a later repeat are not fresh validation.
        book.observe(
            &example(2),
            distribution(0),
            distribution(0),
            Some(distribution(2)),
            &rows,
        );
        assert_eq!(
            book.domains[&4]
                .shadow
                .as_ref()
                .unwrap()
                .comparison
                .future_examples,
            0
        );
        for id in 130..162 {
            let row = example(id);
            book.observe(
                &row,
                distribution(0),
                distribution(0),
                Some(distribution(row.action)),
                &rows,
            );
        }
        let state = serde_json::to_string(&book).unwrap();
        let mut book: AcquiredRules = serde_json::from_str(&state).unwrap();
        assert_eq!(state, serde_json::to_string(&book).unwrap());
        assert!(book.validate(161));
        for id in 162..194 {
            let row = example(id);
            book.observe(
                &row,
                distribution(0),
                distribution(0),
                Some(distribution(row.action)),
                &rows,
            );
        }
        assert_eq!(book.report().admitted_programs, 1);
        assert_eq!(book.report().active_domains, [4]);
        assert!(book.validate(193));
        for id in 194..258 {
            let row = example(id);
            book.observe(&row, distribution(row.action), distribution(0), None, &rows);
        }
        assert_eq!(book.report().rollbacks, 1);
        assert!(book.report().active_domains.is_empty());
        assert!(book.validate(257));
    }
}
