//! Noise-aware object and producer roles use the existing RSP2 search engine.
//! The searched object is the executable parameter update, not an answer key.
use super::{normalized, valid_rule, Seed, Series, LAGS};
use crate::construction::{
    search_policy::{search, Objective, SearchLimits, SearchPolicy, SearchResult, FEATURES},
    vm::{Expr, Function, Library, Meter},
    Error, Result,
};
use serde::{Deserialize, Serialize};
const POLICY_RECORD: u64 = 0x52535031;
const OBJECT_WORK: u64 = 300_000;
const META_WORK: u64 = 4_000_000;
const SIMULATION_WORK: u64 = 50_000;
const SUPPORT: usize = 16;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Lesson {
    pub values: Vec<f64>,
    pub kind: u8,
    pub origin: u64,
    pub first_origin: u64,
    pub initial: Seed,
}
impl Lesson {
    fn validate(&self) -> Result<()> {
        if self.values.len() != 48
            || self.kind > 2
            || self.origin == 0
            || self.first_origin == 0
            || self.first_origin > self.origin
            || !self.initial.valid()
            || self.values.iter().any(|x| !x.is_finite() || x.abs() > 1e6)
        {
            return Err(Error::InvalidData);
        }
        Ok(())
    }
    fn baseline(&self, probe: bool, meter: &mut Meter) -> Result<f64> {
        meter.charge((self.values.len() * 8) as u64)?;
        let range = if probe { SUPPORT..48 } else { 0..SUPPORT };
        let mut sum = 0.;
        let mut count = 0;
        for i in range {
            if self.initial.history.len() + i >= LAGS {
                let previous = if i == 0 {
                    *self.initial.history.last().ok_or(Error::InvalidData)?
                } else {
                    self.values[i - 1]
                };
                sum += (normalized(self.kind, self.values[i]) - normalized(self.kind, previous))
                    .powi(2);
                count += 1;
            }
        }
        Ok(sum / count.max(1) as f64)
    }
}

fn object_limits() -> SearchLimits {
    SearchLimits {
        work: OBJECT_WORK,
        evaluations: 32,
        max_nodes: 11,
        frontier: 96,
        edits_per_expansion: 16,
    }
}
fn meta_limits() -> SearchLimits {
    SearchLimits {
        work: META_WORK,
        evaluations: 32,
        max_nodes: 15,
        frontier: 64,
        edits_per_expansion: 16,
    }
}

fn simulate(rule: &Expr, lesson: &Lesson, probe: bool, meter: &mut Meter) -> Result<f64> {
    valid_rule(rule)?;
    lesson.validate()?;
    let mut series = lesson.initial.restore();
    let mut loss = 0.;
    let mut n = 0;
    let end = if probe { 48 } else { SUPPORT };
    for (i, &value) in lesson.values[..end].iter().enumerate() {
        meter.charge((LAGS * 32 + 32) as u64)?;
        if series.history.len() >= LAGS && (!probe || i >= SUPPORT) {
            let p = series.weights.predict(&series.history);
            loss += (normalized(lesson.kind, p) - normalized(lesson.kind, value)).powi(2);
            n += 1;
        }
        series.weights.learn(
            &series.history,
            series.energy,
            series.count,
            value,
            rule,
            meter,
        )?;
        series.append(value);
    }
    Ok(loss / n.max(1) as f64)
}

struct RuleObjective<'a> {
    lessons: &'a [Lesson],
    library: Library,
    baseline: Option<u64>,
}
impl Objective for RuleObjective<'_> {
    fn input_width(&self) -> usize {
        6
    }
    fn library(&self) -> &Library {
        &self.library
    }
    fn solved(&self, loss: u64) -> bool {
        self.baseline
            .is_some_and(|b| b > 0 && loss.saturating_mul(100) <= b.saturating_mul(95))
    }
    fn score(&mut self, e: &Expr, m: &mut Meter) -> Result<u64> {
        valid_rule(e)?;
        let mut loss = 0.;
        for l in self.lessons {
            let baseline = l.baseline(false, m)?.max(1e-18);
            loss += simulate(e, l, false, m)? / baseline;
        }
        let score = (loss / self.lessons.len().max(1) as f64 * 1_000_000.).clamp(0., 1e12) as u64;
        if self.baseline.is_none() {
            self.baseline = Some(score);
        }
        Ok(score)
    }
}

fn search_rule(
    policy: &SearchPolicy,
    lessons: &[Lesson],
    seed: u64,
    meter: &mut Meter,
) -> Result<SearchResult> {
    search_rule_from(policy, &Expr::Input(0), lessons, seed, meter)
}
fn search_rule_from(
    policy: &SearchPolicy,
    root: &Expr,
    lessons: &[Lesson],
    seed: u64,
    meter: &mut Meter,
) -> Result<SearchResult> {
    if lessons.is_empty() {
        return Err(Error::NoEvidence);
    }
    if meter.remaining() < OBJECT_WORK {
        return Err(Error::Budget);
    }
    let mut objective = RuleObjective {
        lessons,
        library: Library::default(),
        baseline: None,
    };
    let result = search(policy, root.clone(), &mut objective, &object_limits(), seed);
    match result {
        Ok(r) => {
            meter.charge(r.work)?;
            Ok(r)
        }
        Err(e) => {
            meter.charge(OBJECT_WORK)?;
            Err(e)
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Score {
    loss: f64,
    work: u64,
    trace: u64,
    runs: u64,
}
impl Score {
    fn value(&self) -> f64 {
        self.loss * OBJECT_WORK as f64 + self.work as f64
    }
    fn add(&mut self, s: &Self) {
        self.loss += s.loss;
        self.work = self.work.saturating_add(s.work);
        self.trace ^= s.trace.rotate_left(self.runs as u32 % 64);
        self.runs += s.runs;
    }
    fn valid(&self) -> bool {
        self.loss.is_finite() && self.loss >= 0. && self.runs <= 32
    }
}
fn evaluate(
    policy: &SearchPolicy,
    lessons: &[Lesson],
    seed: u64,
    meter: &mut Meter,
) -> Result<Score> {
    let mut score = Score::default();
    for (i, l) in lessons.iter().enumerate() {
        let start = meter.spent;
        let run = search_rule(
            policy,
            std::slice::from_ref(l),
            seed.wrapping_add(i as u64),
            meter,
        )?;
        let ratio = simulate(&run.program, l, true, meter)? / l.baseline(true, meter)?.max(1e-18);
        score.add(&Score {
            loss: ratio,
            work: meter.spent - start,
            trace: run.trace_hash,
            runs: 1,
        });
    }
    Ok(score)
}

struct PolicyObjective<'a> {
    lessons: &'a [Lesson],
    seed: u64,
    library: Library,
    baseline: Option<u64>,
}
impl Objective for PolicyObjective<'_> {
    fn input_width(&self) -> usize {
        FEATURES
    }
    fn library(&self) -> &Library {
        &self.library
    }
    fn solved(&self, loss: u64) -> bool {
        self.baseline
            .is_some_and(|b| b > 0 && loss.saturating_mul(100) <= b.saturating_mul(95))
    }
    fn score(&mut self, e: &Expr, m: &mut Meter) -> Result<u64> {
        let policy = SearchPolicy::from_code(e)?;
        let required = (OBJECT_WORK + SIMULATION_WORK) * self.lessons.len() as u64;
        if m.remaining() < required {
            return Err(Error::Budget);
        }
        let score = evaluate(&policy, self.lessons, self.seed, m)?;
        let value = score.value().clamp(0., 1e12) as u64;
        if self.baseline.is_none() {
            self.baseline = Some(value);
        }
        Ok(value)
    }
}
fn produce(
    producer: &SearchPolicy,
    target: &SearchPolicy,
    lessons: &[Lesson],
    seed: u64,
    meter: &mut Meter,
) -> Result<SearchResult> {
    if meter.remaining() < META_WORK {
        return Err(Error::Budget);
    }
    let library = Library {
        functions: vec![Function {
            id: POLICY_RECORD,
            arity: 3,
            body: Expr::Arg(0),
        }],
    };
    let mut objective = PolicyObjective {
        lessons,
        seed: seed ^ 0xF00D,
        library,
        baseline: None,
    };
    let result = search(
        producer,
        target.code(),
        &mut objective,
        &meta_limits(),
        seed,
    );
    match result {
        Ok(r) => {
            meter.charge(r.work)?;
            Ok(r)
        }
        Err(e) => {
            meter.charge(META_WORK)?;
            Err(e)
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pair {
    before: Score,
    after: Score,
}
impl Pair {
    fn passes(&self) -> bool {
        self.before.runs > 0
            && self.before.runs == self.after.runs
            && self.before.trace != self.after.trace
            && self.after.loss <= 1.02 * self.before.loss + 1e-12
            && self.after.value() <= 0.95 * self.before.value()
    }
    fn valid(&self) -> bool {
        self.before.valid() && self.after.valid() && self.before.runs == self.after.runs
    }
}
fn compare(
    before: &SearchPolicy,
    after: &SearchPolicy,
    lessons: &[Lesson],
    seed: u64,
    meter: &mut Meter,
) -> Result<Pair> {
    let needed = (OBJECT_WORK + SIMULATION_WORK) * lessons.len() as u64 * 2;
    if meter.remaining() < needed {
        return Err(Error::Budget);
    }
    Ok(Pair {
        before: evaluate(before, lessons, seed, meter)?,
        after: evaluate(after, lessons, seed, meter)?,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pending {
    retention_checked: usize,
    before: SearchPolicy,
    candidate: SearchPolicy,
    production: SearchResult,
    after_origin: u64,
    gate: Vec<Lesson>,
    stage: u8,
    direct: Option<Pair>,
    children: Vec<SearchPolicy>,
    creation_work: Vec<u64>,
    producers: Vec<Pair>,
    retention: Option<Pair>,
}
impl Pending {
    fn verdict(&self) -> bool {
        self.candidate != self.before
            && self.direct.as_ref().is_some_and(Pair::passes)
            && self.producers.len() == 2
            && self.producers.iter().all(Pair::passes)
            && self
                .retention
                .as_ref()
                .is_none_or(|p| p.after.loss <= 1.02 * p.before.loss + 1e-12)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Audit {
    before: SearchPolicy,
    candidate: SearchPolicy,
    production: SearchResult,
    direct: Pair,
    producers: Vec<Pair>,
    retention: Option<Pair>,
    accepted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchReport {
    pub long_term_lessons: usize,
    pub remembered_counterexamples: usize,
    pub attempts: u64,
    pub rule_searches: u64,
    pub rule_proposals: u64,
    pub policy_generations: u64,
    pub policy: SearchPolicy,
    pub policy_id: u64,
    pub work: u64,
    pub retained_lessons: usize,
    pub pending_stage: Option<u8>,
    pub last_error: Option<String>,
    pub last_direct: Option<(f64, f64)>,
    pub last_producers: Vec<(f64, f64)>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RuleSearch {
    anchors: Vec<(u64, Lesson)>,
    counterexamples: Vec<Lesson>,
    memory_seed: u64,
    policy: SearchPolicy,
    initial: SearchPolicy,
    archived_policy: SearchPolicy,
    lessons: Vec<Lesson>,
    retained: Vec<Lesson>,
    pending: Option<Pending>,
    audits: Vec<Audit>,
    rng: u64,
    attempts: u64,
    rule_searches: u64,
    proposals: u64,
    generations: u64,
    work: u64,
    next_rule: u64,
    next_meta: u64,
    used_until: u64,
    last_error: Option<String>,
}
impl RuleSearch {
    pub(super) fn new(seed: u64, policy: SearchPolicy) -> Self {
        Self {
            anchors: Vec::new(),
            counterexamples: Vec::new(),
            memory_seed: seed ^ 0x414e43484f52,
            initial: policy.clone(),
            archived_policy: policy.clone(),
            policy,
            lessons: Vec::new(),
            retained: Vec::new(),
            pending: None,
            audits: Vec::new(),
            rng: seed,
            attempts: 0,
            rule_searches: 0,
            proposals: 0,
            generations: 0,
            work: 0,
            next_rule: 256,
            next_meta: 512,
            used_until: 0,
            last_error: None,
        }
    }
    fn seed(&mut self) -> u64 {
        self.rng = super::super::word(&mut self.rng);
        self.rng
    }
    pub(super) fn remember(&mut self, l: Lesson) {
        if l.values.windows(2).all(|p| (p[0] - p[1]).abs() < 1e-12) {
            return;
        }
        let mut state = self.memory_seed ^ l.origin;
        let rank = super::super::word(&mut state);
        if self.anchors.len() < 16 {
            self.anchors.push((rank, l.clone()));
        } else if let Some((index, _)) = self.anchors.iter().enumerate().max_by_key(|(_, x)| x.0) {
            if rank < self.anchors[index].0 {
                self.anchors[index] = (rank, l.clone());
            }
        }
        self.lessons.push(l);
        if self.lessons.len() > 64 {
            self.lessons.remove(0);
        }
    }
    pub(super) fn remember_rejection(&mut self) {
        if let Some(lesson) = self.lessons.last() {
            if !self
                .counterexamples
                .iter()
                .any(|l| l.origin == lesson.origin)
            {
                self.counterexamples.push(lesson.clone());
            }
            if self.counterexamples.len() > 4 {
                self.counterexamples.remove(0);
            }
        }
    }
    fn curriculum(&self, recent: &[Lesson]) -> Vec<Lesson> {
        let mut training: Vec<_> = recent.iter().rev().take(2).cloned().collect();
        for lesson in self
            .counterexamples
            .iter()
            .rev()
            .chain(self.anchors.iter().map(|(_, l)| l))
        {
            if training.len() >= 4 {
                break;
            }
            if !training.iter().any(|l| l.origin == lesson.origin) {
                training.push(lesson.clone());
            }
        }
        training
    }
    pub(super) fn advance(
        &mut self,
        raw_values: u64,
        recursive: bool,
        allow_rule: bool,
        incumbent: &Expr,
        meter: &mut Meter,
    ) -> Result<Option<Expr>> {
        let before = meter.spent;
        let result = self.advance_inner(raw_values, recursive, allow_rule, incumbent, meter);
        self.work = self.work.saturating_add(meter.spent - before);
        match result {
            Ok(r) => {
                self.last_error = None;
                Ok(r)
            }
            Err(Error::Budget) => Ok(None),
            Err(e) => {
                self.last_error = Some(format!("{e:?}"));
                Ok(None)
            }
        }
    }
    fn advance_inner(
        &mut self,
        raw_values: u64,
        recursive: bool,
        allow_rule: bool,
        incumbent: &Expr,
        meter: &mut Meter,
    ) -> Result<Option<Expr>> {
        if self.pending.is_some() {
            self.advance_pending(meter)?;
        }
        let available: Vec<_> = self
            .lessons
            .iter()
            .filter(|l| l.origin > self.used_until)
            .cloned()
            .collect();
        if available.len() < 4 {
            return Ok(None);
        }
        let training = self.curriculum(&available);
        if allow_rule && raw_values >= self.next_rule && meter.remaining() >= OBJECT_WORK {
            let seed = self.seed();
            self.rule_searches += 1;
            self.next_rule = raw_values.saturating_add(256);
            let run = search_rule_from(&self.policy, incumbent, &training, seed, meter)?;
            if run.solved && run.program != *incumbent {
                self.proposals += 1;
                return Ok(Some(run.program));
            }
            return Ok(None);
        }
        if recursive
            && self.pending.is_none()
            && raw_values >= self.next_meta
            && meter.remaining() >= META_WORK
        {
            let seed = self.seed();
            self.attempts += 1;
            self.next_meta = raw_values.saturating_add(512);
            let lessons = if training.len() > 2 {
                vec![
                    training[0].clone(),
                    training.last().ok_or(Error::NoEvidence)?.clone(),
                ]
            } else {
                training.clone()
            };
            let production = produce(&self.policy, &self.policy, &lessons, seed, meter)?;
            let candidate = SearchPolicy::from_code(&production.program)?;
            if candidate != self.policy {
                self.pending = Some(Pending {
                    retention_checked: 0,
                    before: self.policy.clone(),
                    candidate,
                    production,
                    after_origin: raw_values,
                    gate: Vec::new(),
                    stage: 0,
                    direct: None,
                    children: Vec::new(),
                    creation_work: Vec::new(),
                    producers: Vec::new(),
                    retention: None,
                });
            }
            return Ok(None);
        }
        Ok(None)
    }
    fn advance_pending(&mut self, meter: &mut Meter) -> Result<()> {
        let p = self.pending.as_mut().ok_or(Error::NoEvidence)?;
        if p.gate.is_empty() {
            let gate: Vec<_> = self
                .lessons
                .iter()
                .filter(|l| l.first_origin > p.after_origin)
                .take(12)
                .cloned()
                .collect();
            if gate.len() < 12 {
                return Ok(());
            }
            p.gate = gate;
        }
        let seed = p.production.trace_hash;
        match p.stage {
            0 => {
                let pair = compare(&p.before, &p.candidate, &p.gate[..4], seed, meter)?;
                p.direct = Some(pair);
                p.stage = 1;
            }
            1..=4 => {
                let role = (p.stage - 1) as usize;
                let group = role / 2;
                let producer = if role % 2 == 0 {
                    &p.before
                } else {
                    &p.candidate
                };
                let result = produce(
                    producer,
                    &p.before,
                    &p.gate[4 + group * 4..6 + group * 4],
                    seed.wrapping_add(group as u64 * 0x10001),
                    meter,
                )?;
                p.children.push(SearchPolicy::from_code(&result.program)?);
                p.creation_work.push(result.work);
                p.stage += 1;
            }
            5..=6 => {
                let group = (p.stage - 5) as usize;
                let mut pair = compare(
                    &p.children[group * 2],
                    &p.children[group * 2 + 1],
                    &p.gate[6 + group * 4..8 + group * 4],
                    seed.wrapping_add(group as u64 * 0x20003),
                    meter,
                )?;
                pair.before.work = pair.before.work.saturating_add(p.creation_work[group * 2]);
                pair.after.work = pair
                    .after
                    .work
                    .saturating_add(p.creation_work[group * 2 + 1]);
                p.producers.push(pair);
                p.stage += 1;
            }
            7 => {
                if p.retention_checked < self.retained.len() {
                    let end = (p.retention_checked + 2).min(self.retained.len());
                    let pair = compare(
                        &p.before,
                        &p.candidate,
                        &self.retained[p.retention_checked..end],
                        seed ^ 0x524554,
                        meter,
                    )?;
                    if let Some(total) = p.retention.as_mut() {
                        total.before.add(&pair.before);
                        total.after.add(&pair.after);
                    } else {
                        p.retention = Some(pair);
                    }
                    p.retention_checked = end;
                }
                if p.retention_checked == self.retained.len() {
                    p.stage = 8;
                }
            }
            _ => {}
        }
        // A failed direct gate cannot consume more work on a producer claim.
        let failed = p.direct.as_ref().is_some_and(|d| !d.passes());
        if p.stage == 8 || failed {
            let p = self.pending.take().ok_or(Error::NoEvidence)?;
            let accepted = p.verdict();
            self.used_until = self
                .used_until
                .max(p.gate.last().map_or(p.after_origin, |l| l.origin));
            if accepted {
                self.policy = p.candidate.clone();
                self.generations += 1;
                for l in &p.gate[..2] {
                    if !self.retained.iter().any(|r| r.origin == l.origin) {
                        self.retained.push(l.clone());
                    }
                }
                while self.retained.len() > 16 {
                    self.retained.remove(8);
                }
            }
            self.audits.push(Audit {
                before: p.before,
                candidate: p.candidate,
                production: p.production,
                direct: p.direct.ok_or(Error::NoEvidence)?,
                producers: p.producers,
                retention: p.retention,
                accepted,
            });
            if self.audits.len() > 64 {
                let old = self.audits.remove(0);
                if old.accepted {
                    self.archived_policy = old.candidate;
                }
            }
        }
        Ok(())
    }
    pub(super) fn report(&self) -> SearchReport {
        let last = self.audits.last();
        SearchReport {
            long_term_lessons: self.anchors.len(),
            remembered_counterexamples: self.counterexamples.len(),
            attempts: self.attempts,
            rule_searches: self.rule_searches,
            rule_proposals: self.proposals,
            policy_generations: self.generations,
            policy: self.policy.clone(),
            policy_id: self.policy.fingerprint(),
            work: self.work,
            retained_lessons: self.lessons.len(),
            pending_stage: self.pending.as_ref().map(|p| p.stage),
            last_error: self.last_error.clone(),
            last_direct: last.map(|a| (a.direct.before.value(), a.direct.after.value())),
            last_producers: last.map_or_else(Vec::new, |a| {
                a.producers
                    .iter()
                    .map(|p| (p.before.value(), p.after.value()))
                    .collect()
            }),
        }
    }
    pub(super) fn validate(&self) -> Result<()> {
        self.policy.validate()?;
        self.initial.validate()?;
        self.archived_policy.validate()?;
        if self.lessons.len() > 64
            || self.anchors.len() > 16
            || self.counterexamples.len() > 4
            || self.retained.len() > 16
            || self.audits.len() > 64
            || self.proposals > self.rule_searches
            || self.generations > self.attempts
        {
            return Err(Error::InvalidData);
        }
        for l in self
            .lessons
            .iter()
            .chain(&self.retained)
            .chain(self.anchors.iter().map(|(_, l)| l))
            .chain(&self.counterexamples)
        {
            l.validate()?;
        }
        if self.lessons.windows(2).any(|p| p[0].origin >= p[1].origin) {
            return Err(Error::InvalidData);
        }
        let mut policy = self.archived_policy.clone();
        for a in &self.audits {
            a.before.validate()?;
            a.candidate.validate()?;
            let mut code = a.before.code();
            for edit in &a.production.edits_to_winner {
                code = edit.apply(&code)?;
            }
            let accepted = a.candidate != a.before
                && a.direct.passes()
                && a.producers.len() == 2
                && a.producers.iter().all(Pair::passes)
                && a.retention
                    .as_ref()
                    .is_none_or(|p| p.after.loss <= 1.02 * p.before.loss + 1e-12);
            if a.before != policy
                || code != a.candidate.code()
                || a.production.program != code
                || a.production.policy_id != a.before.fingerprint()
                || !a.direct.valid()
                || a.producers.iter().any(|p| !p.valid())
                || a.retention.as_ref().is_some_and(|p| !p.valid())
                || accepted != a.accepted
            {
                return Err(Error::InvalidData);
            }
            if a.accepted {
                policy = a.candidate.clone();
            }
        }
        if policy != self.policy {
            return Err(Error::InvalidData);
        }
        if let Some(p) = &self.pending {
            p.before.validate()?;
            p.candidate.validate()?;
            let mut code = p.before.code();
            for edit in &p.production.edits_to_winner {
                code = edit.apply(&code)?;
            }
            if p.before != self.policy
                || code != p.candidate.code()
                || p.production.program != code
                || p.stage > 7
                || p.retention_checked > self.retained.len()
                || p.gate.len() > 12
                || p.children.len() > 4
                || p.creation_work.len() != p.children.len()
                || p.producers.len() > 2
                || p.direct.as_ref().is_some_and(|p| !p.valid())
            {
                return Err(Error::InvalidData);
            }
            for l in &p.gate {
                l.validate()?;
                if l.first_origin <= p.after_origin {
                    return Err(Error::InvalidData);
                }
            }
            for child in &p.children {
                child.validate()?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn lesson() -> Lesson {
        let values: Vec<f64> = serde_json::from_str(include_str!(
            "../../../tests/data/temporal_cpu_development.json"
        ))
        .unwrap();
        Lesson {
            values: values[..48].to_vec(),
            kind: 1,
            origin: 48,
            first_origin: 1,
            initial: Seed::capture(&Series::default()),
        }
    }
    #[test]
    fn temporal_search_cannot_read_its_future_probes() {
        let a = lesson();
        let mut b = a.clone();
        for (i, x) in b.values[SUPPORT..].iter_mut().enumerate() {
            *x = if i % 2 == 0 { 1e5 } else { -1e5 };
        }
        let mut m = Meter::new(1_000_000);
        let p = SearchPolicy::default();
        let x = search_rule(&p, std::slice::from_ref(&a), 123, &mut m).unwrap();
        let mut m = Meter::new(1_000_000);
        let y = search_rule(&p, std::slice::from_ref(&b), 123, &mut m).unwrap();
        assert_eq!(x, y);
        let mut m = Meter::new(200_000);
        let la = simulate(&x.program, &a, true, &mut m).unwrap();
        let lb = simulate(&x.program, &b, true, &mut m).unwrap();
        assert!((la - lb).abs() > 1e-8);
    }
    #[test]
    fn temporal_noise_accepts_improvement_without_exact_fit() {
        let l = lesson();
        let mut o = RuleObjective {
            lessons: std::slice::from_ref(&l),
            library: Library::default(),
            baseline: None,
        };
        let mut m = Meter::new(100_000);
        let baseline = o.score(&Expr::Input(0), &mut m).unwrap();
        assert!(baseline > 0);
        assert!(!o.solved(baseline));
        assert!(o.solved(baseline * 94 / 100));
        assert!(!o.solved(baseline * 96 / 100));
    }
    #[test]
    fn temporal_research_replays_the_actual_parameter_context() {
        let a = lesson();
        let mut b = a.clone();
        b.initial.history = vec![0.05; LAGS];
        b.initial.count = 64;
        b.initial.energy = 0.01;
        b.initial.weights.logits = vec![-20.; LAGS];
        b.initial.weights.logits[LAGS - 1] = 0.;
        let mut m = Meter::new(150_000);
        let first = simulate(&Expr::Input(0), &a, false, &mut m).unwrap();
        let second = simulate(&Expr::Input(0), &b, false, &mut m).unwrap();
        assert!((first - second).abs() > 1e-10);
    }
    #[test]
    fn temporal_direct_improvement_cannot_become_a_recursive_admission() {
        let pair = Pair {
            before: Score {
                loss: 1.,
                work: 1000,
                trace: 1,
                runs: 1,
            },
            after: Score {
                loss: 0.5,
                work: 500,
                trace: 2,
                runs: 1,
            },
        };
        assert!(pair.passes());
        let policy = SearchPolicy::default();
        let mut m = Meter::new(5_000_000);
        let production = produce(&policy, &policy, &[lesson()], 17, &mut m).unwrap();
        let mut pending = Pending {
            retention_checked: 0,
            before: policy.clone(),
            candidate: SearchPolicy::from_code(&production.program).unwrap(),
            production,
            after_origin: 48,
            gate: Vec::new(),
            stage: 1,
            direct: Some(pair.clone()),
            children: Vec::new(),
            creation_work: Vec::new(),
            producers: Vec::new(),
            retention: None,
        };
        assert!(!pending.verdict());
        pending.producers.push(pair.clone());
        assert!(!pending.verdict());
        pending.producers.push(pair);
        assert_eq!(pending.verdict(), pending.candidate != pending.before);
    }
}
