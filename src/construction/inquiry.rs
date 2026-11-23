//! A bounded applicability question, opened and settled only by real evidence.
//! This state has no actuator, world labels, scheduler, or learning authority.
use super::vm::{Error, Result};
use serde::{Deserialize, Serialize};

pub(super) const STATE_BYTES: usize = 32 * 1024;
pub(super) const UPDATE_WORK: u64 = 8192;
pub(super) const MAX_AGE: u64 = 192;
pub(super) const MAX_FORECAST_OUTCOMES: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InquiryMode { Active, ObserveOnly }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InquiryPrediction {
    pub bank: usize,
    pub support: u64,
    pub outcomes: Vec<(Vec<i64>, f64)>,
    pub unknown_probability: f64,
}
impl InquiryPrediction {
    pub(super) fn probability(&self, post: &[i64]) -> f64 {
        let p = self.outcomes.iter().find(|(s, _)| s == post).map_or(0.0, |(_, p)| *p);
        (p * self.support as f64 + 0.5)
            / (self.support as f64 + 0.5 * (self.outcomes.len() + 1) as f64)
    }
    pub(super) fn validate(&self, width: usize, events: u64) -> Result<()> {
        let mass = self.outcomes.iter().map(|(_, p)| p).sum::<f64>();
        if self.bank > 1 || self.support > events || self.outcomes.len() > MAX_FORECAST_OUTCOMES
            || !self.unknown_probability.is_finite() || !(0.0..=1.0).contains(&self.unknown_probability)
            || self.outcomes.iter().any(|(s, p)| s.len() != width || !p.is_finite() || !(0.0..=1.0).contains(p))
            || (mass + self.unknown_probability - 1.0).abs() > 1e-8
            || self.outcomes.iter().map(|(s, _)| s).collect::<std::collections::BTreeSet<_>>().len() != self.outcomes.len()
            || (self.support == 0 && (!self.outcomes.is_empty() || self.unknown_probability != 1.0))
        { return Err(Error::InvalidData); }
        Ok(())
    }
}

/// An immutable forecast made before U1-selected physical execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InquiryDecision {
    pub question_id: u64,
    pub event: u64,
    pub action: usize,
    /// Reducible disagreement of two supported factual distributions here.
    pub immediate_information_gain: f64,
    /// Best reachable diagnostic opportunity within the bounded plan. This is
    /// not a sum of imaginary repeated samples and never a goal-success label.
    pub expected_information_gain: f64,
    pub expected_goal_probability: f64,
    pub bank_probability: [f64; 2],
    pub predictions: Vec<InquiryPrediction>,
}
impl InquiryDecision {
    pub(super) fn validate(&self, width: usize, actions: usize, events: u64) -> Result<()> {
        if self.action >= actions || self.event != events || self.question_id >= events
            || self.predictions.len() != 2 || self.predictions[0].bank != 0 || self.predictions[1].bank != 1
            || !self.expected_goal_probability.is_finite() || !(0.0..=1.0).contains(&self.expected_goal_probability)
            || [self.immediate_information_gain, self.expected_information_gain].iter()
                .any(|x| !x.is_finite() || !(0.0..=std::f64::consts::LN_2 + 1e-8).contains(x))
            || self.expected_information_gain + 1e-8 < self.immediate_information_gain
            || self.expected_information_gain <= 0.0
            || self.bank_probability.iter().any(|p| !p.is_finite() || !(0.0..=1.0).contains(p))
            || (self.bank_probability.iter().sum::<f64>() - 1.0).abs() > 1e-8
        { return Err(Error::InvalidData); }
        for p in &self.predictions { p.validate(width, events)?; }
        if self.immediate_information_gain > 1e-12 {
            if self.predictions.iter().any(|p| p.support < 8 || p.unknown_probability >= 0.05) {
                return Err(Error::InvalidData);
            }
            let entropy = |p:&[f64]| p.iter().filter(|&&x|x>0.0).map(|x|-x*x.ln()).sum::<f64>();
            let mut mixture = std::collections::BTreeMap::<&Vec<i64>,f64>::new();
            let mut conditional = 0.0;
            let mut unknown = 0.0;
            for (p,prior) in self.predictions.iter().zip(self.bank_probability) {
                for (s,mass) in &p.outcomes { *mixture.entry(s).or_default() += prior*mass; }
                unknown += prior*p.unknown_probability;
                conditional += prior*entropy(&p.outcomes.iter().map(|(_,x)|*x)
                    .chain(std::iter::once(p.unknown_probability)).collect::<Vec<_>>());
            }
            let information = (entropy(&mixture.values().copied().chain(std::iter::once(unknown))
                .collect::<Vec<_>>()) - conditional).max(0.0);
            if (information-self.immediate_information_gain).abs()>1e-8 { return Err(Error::InvalidData); }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InquiryQuestion {
    pub id: u64,
    pub opened_event: u64,
    pub triggering_action: usize,
    pub triggering_pre: Vec<i64>,
    pub predicted_post: Vec<i64>,
    pub actual_post: Vec<i64>,
    pub prior_bank: usize,
    pub prediction_probability: f64,
    pub observed_probability: f64,
    pub support: u64,
    pub context_fingerprint: u64,
    pub age: u64,
    pub evidence_events: u64,
    pub factual_experiments: u64,
    /// Signed prequential evidence for bank zero versus bank one.
    pub log_bayes_factor: f64,
    pub candidate_bank: Option<usize>,
    pub candidate_confirmations: u64,
}
impl InquiryQuestion {
    pub(super) fn bank_probability(&self) -> [f64; 2] {
        let p = (1.0 / (1.0 + (-self.log_bayes_factor).exp())).clamp(0.025, 0.975);
        [p, 1.0 - p]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InquiryResolution {
    pub question_id: u64,
    pub opened_event: u64,
    pub ended_event: u64,
    /// "resolved" requires factual discrimination; "expired" stays unknown.
    pub status: String,
    pub bank: Option<usize>,
    pub evidence_events: u64,
    pub factual_experiments: u64,
    pub log_bayes_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InquiryReport {
    pub mode: InquiryMode,
    pub observations: u64,
    pub questions_opened: u64,
    pub questions_resolved: u64,
    pub questions_expired: u64,
    pub selected_experiments: u64,
    pub factual_experiments: u64,
    pub active: Option<InquiryQuestion>,
    pub last_resolution: Option<InquiryResolution>,
    pub extra_inference_work: u64,
    pub state_bytes: usize,
    pub dynamic_fingerprint: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct InquiryState {
    pub mode: InquiryMode,
    enabled_at: u64,
    observations: u64,
    questions_opened: u64,
    questions_resolved: u64,
    questions_expired: u64,
    selected_experiments: u64,
    factual_experiments: u64,
    pub active: Option<InquiryQuestion>,
    last_resolution: Option<InquiryResolution>,
    pub pending: Option<InquiryDecision>,
    extra_inference_work: u64,
}
impl InquiryState {
    pub fn new(mode: InquiryMode, events: u64) -> Self {
        Self {mode, enabled_at: events, observations: 0, questions_opened: 0,
            questions_resolved: 0, questions_expired: 0, selected_experiments: 0,
            factual_experiments: 0, active: None, last_resolution: None, pending: None,
            extra_inference_work: 0}
    }
    pub fn report(&self) -> InquiryReport {
        InquiryReport {mode: self.mode, observations: self.observations,
            questions_opened: self.questions_opened, questions_resolved: self.questions_resolved,
            questions_expired: self.questions_expired, selected_experiments: self.selected_experiments,
            factual_experiments: self.factual_experiments, active: self.active.clone(),
            last_resolution: self.last_resolution.clone(), extra_inference_work: self.extra_inference_work,
            state_bytes: STATE_BYTES,
            dynamic_fingerprint: super::vm::fingerprint(&serde_json::to_vec(self).unwrap())}
    }
    pub fn inference_work(&self) -> u64 { self.extra_inference_work }
    pub fn record(&mut self, action: usize, decision: Option<&InquiryDecision>, events: u64) -> Result<()> {
        if self.pending.is_some() { return Err(Error::Busy); }
        if let Some(d) = decision {
            if self.mode != InquiryMode::Active || d.event != events || d.action != action
                || !self.active.as_ref().is_some_and(|q| q.id == d.question_id)
                || d.expected_information_gain <= 0.0
            { return Err(Error::Stale); }
            self.pending = Some(d.clone());
            self.selected_experiments = self.selected_experiments.saturating_add(1);
        }
        Ok(())
    }
    pub fn observe(&mut self, event: u64, action: usize, post: &[i64], predictions: &[InquiryPrediction],
        routed_bank: usize, router_confidence: f64, trigger: Option<InquiryQuestion>) {
        self.observations = self.observations.saturating_add(1);
        self.extra_inference_work = self.extra_inference_work.saturating_add(UPDATE_WORK);
        let selected = self.pending.take().filter(|d| d.event == event && d.action == action);
        if selected.is_some() { self.factual_experiments = self.factual_experiments.saturating_add(1); }
        if self.active.is_none() {
            if let Some(q) = trigger {
                self.active = Some(q);
                self.questions_opened = self.questions_opened.saturating_add(1);
            }
        }
        let Some(q) = self.active.as_mut() else { return };
        q.age = event.saturating_sub(q.opened_event);
        if selected.as_ref().is_some_and(|d| d.question_id == q.id) {
            q.factual_experiments = q.factual_experiments.saturating_add(1);
        }
        // Use committed forecasts for a selected experiment; other factual
        // transitions use forecasts read before the table learns this POST.
        let forecasts = selected.as_ref().map_or(predictions, |d| d.predictions.as_slice());
        if forecasts.len() == 2 && forecasts.iter().all(|p| p.support >= 8 && p.unknown_probability < 0.05) {
            let ratio = (forecasts[0].probability(post) / forecasts[1].probability(post)).ln().clamp(-6.0, 6.0);
            if ratio.abs() >= 2.0_f64.ln() {
                q.log_bayes_factor = (q.log_bayes_factor + ratio).clamp(-12.0, 12.0);
                q.evidence_events = q.evidence_events.saturating_add(1);
                let bank = usize::from(ratio < 0.0);
                if q.candidate_bank == Some(bank) { q.candidate_confirmations = q.candidate_confirmations.saturating_add(1); }
                else { q.candidate_bank = Some(bank); q.candidate_confirmations = 1; }
            }
        }
        let posterior = q.bank_probability();
        let resolved = q.candidate_bank == Some(routed_bank) && q.candidate_confirmations >= 2
            && q.log_bayes_factor.abs() >= 9.0_f64.ln() && posterior[routed_bank] >= 0.8
            && router_confidence >= 0.8;
        let expired = q.age >= MAX_AGE;
        if resolved || expired {
            let q = self.active.take().unwrap();
            self.last_resolution = Some(InquiryResolution {question_id: q.id, opened_event: q.opened_event,
                ended_event: event, status: if resolved {"resolved"} else {"expired"}.into(),
                bank: resolved.then_some(routed_bank), evidence_events: q.evidence_events,
                factual_experiments: q.factual_experiments, log_bayes_factor: q.log_bayes_factor});
            if resolved { self.questions_resolved = self.questions_resolved.saturating_add(1); }
            else { self.questions_expired = self.questions_expired.saturating_add(1); }
        }
    }
    pub fn validate(&self, width: usize, actions: usize, events: u64) -> Result<()> {
        if self.enabled_at.checked_add(self.observations) != Some(events)
            || self.extra_inference_work != self.observations.saturating_mul(UPDATE_WORK)
            || self.questions_opened != self.questions_resolved.saturating_add(self.questions_expired)
                .saturating_add(u64::from(self.active.is_some()))
            || self.questions_opened > self.observations || self.factual_experiments > self.selected_experiments
            || self.factual_experiments > self.observations
            || self.selected_experiments != self.factual_experiments.saturating_add(u64::from(self.pending.is_some()))
            || (self.mode == InquiryMode::ObserveOnly && self.selected_experiments != 0)
        { return Err(Error::InvalidData); }
        if let Some(q) = &self.active {
            if q.id != q.opened_event || q.opened_event < self.enabled_at || q.opened_event >= events
                || q.age != events - 1 - q.opened_event || q.age >= MAX_AGE
                || q.triggering_action >= actions || q.prior_bank > 1 || q.support < 8 || q.support > q.opened_event
                || [&q.triggering_pre, &q.predicted_post, &q.actual_post].iter().any(|s| s.len() != width)
                || q.predicted_post == q.actual_post
                || !q.prediction_probability.is_finite() || !(0.75..=1.0).contains(&q.prediction_probability)
                || !q.observed_probability.is_finite() || !(0.0..0.25).contains(&q.observed_probability)
                || !q.log_bayes_factor.is_finite() || !(-12.0..=12.0).contains(&q.log_bayes_factor)
                || q.candidate_bank.is_some_and(|b| b > 1) || q.candidate_confirmations > q.evidence_events
                || q.evidence_events > q.age + 1 || q.factual_experiments > q.age
                || q.candidate_bank.is_some() != (q.candidate_confirmations > 0)
            { return Err(Error::InvalidData); }
        }
        if let Some(r) = &self.last_resolution {
            if r.question_id != r.opened_event || r.opened_event < self.enabled_at
                || r.opened_event > r.ended_event || r.ended_event >= events
                || !r.log_bayes_factor.is_finite() || !(-12.0..=12.0).contains(&r.log_bayes_factor)
                || r.factual_experiments > r.ended_event - r.opened_event
                || r.evidence_events > r.ended_event - r.opened_event + 1
                || match r.status.as_str() {
                    "resolved" => self.questions_resolved == 0 || !r.bank.is_some_and(|b| b <= 1)
                        || r.evidence_events < 2 || r.log_bayes_factor.abs() < 9.0_f64.ln()
                        || r.bank != Some(usize::from(r.log_bayes_factor < 0.0)),
                    "expired" => self.questions_expired == 0 || r.bank.is_some()
                        || r.ended_event - r.opened_event < MAX_AGE,
                    _ => true,
                }
            { return Err(Error::InvalidData); }
        } else if self.questions_resolved.saturating_add(self.questions_expired) != 0 { return Err(Error::InvalidData); }
        if let Some(d) = &self.pending {
            d.validate(width, actions, events)?;
            if !self.active.as_ref().is_some_and(|q| q.id == d.question_id) { return Err(Error::InvalidData); }
        }
        if serde_json::to_vec(self).map_err(|_| Error::InvalidData)?.len() > STATE_BYTES {
            return Err(Error::Capacity);
        }
        Ok(())
    }
}
