// Included by ScientificRuntime: one EvoPhase and one permission boundary.
use crate::construction::{Config as UConfig,Learner as ULearner};
use crate::construction::autonomy::{ActionOffer as UOffer,AutonomyPolicy as UPolicy};
use crate::construction::vm::{Error as UError,Result as UResult,fingerprint as uhash};
use crate::carrier::{PhaseNativeConfig,PhaseMetaControlConfig};
use serde::{Serialize as USerialize,Deserialize as UDeserialize};
/// Reserve a graph ceiling only after accounting for the other planner's
/// reservation. A later query takes the MIN with actual remaining work.
fn observable_graph_budget_ceiling(total:u64,autobiographical:u64)->u64{
    16_000_u64.min(total.saturating_sub(autobiographical)
        .saturating_sub(4_000))
}
#[derive(Debug,USerialize,UDeserialize)]
struct UnifiedDomain {name:String,learner:ULearner,last_construction:usize,pending_credit:Option<[f32;5]>}
#[derive(Debug,USerialize,UDeserialize)]
struct UnifiedConstruction {
    current:UnifiedDomain,archived:Vec<UnifiedDomain>,nonce:u64,operations:[u64;4],
    computation_work:u64,total_external_cost:u64,max_external_cost:u64,freeze_controller:bool,
    #[serde(default,skip_serializing_if="Option::is_none")]
    education:Option<crate::education::School>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    self_revision:Option<MetaRevision>,
    /// Credit native U1 utility ONLY if the executed motor was freely
    /// selected from at least two actually offered, affordable actions.
    /// Physical PRE/action/POST learning is NEVER gated by this option.
    #[serde(default,skip_serializing_if="Option::is_none")]
    agency_credit:Option<AgencyCreditReport>,
    /// Optional, separate causal information vs actual goal-control credit.
    #[serde(default,skip_serializing_if="Option::is_none")]
    instrumental_credit:Option<InstrumentalCredit>,
    /// Opt-in: a native-selected causal route may continue as one bounded
    /// temporally extended choice, never a new physical action authority.
    #[serde(default,skip_serializing_if="Option::is_none")]
    temporal_options:Option<TemporalOptionLane>,
}


/// The carrier executed its OWN native-U1-selected high-level option, not
/// an externally authored motor schedule. Every subordinate physical action
/// remains subject to fresh HumanProtection authority and actual POST audit.
#[derive(Debug,Clone,USerialize,UDeserialize)]
struct TemporalOptionStep {action:usize,expected_post:Vec<i64>}
#[derive(Debug,Clone,USerialize,UDeserialize)]
struct TemporalOption {
    goal:Vec<Option<i64>>,
    remaining:Vec<TemporalOptionStep>,
    /// Last actual committed external sensor state. No imagined POSTs.
    last_verified_post:Vec<i64>,
    event:usize,
    /// Initial U1-chosen higher-level option's native evidence fields.
    initiating_fields:[f32;5],
    /// Native utility belongs to the selected *entire* causal route,
    /// not to its first potentially invisible preparation.
    #[serde(default)]
    credit_eligible:bool,
    #[serde(default)]
    realized_cost:u64,
    #[serde(default)]
    executed_steps:usize,
    /// 0 for historical causal-suffix options; 2..5 for self-learned routes.
    #[serde(default)]
    learned_depth:usize,
}
#[derive(Debug,Default,Clone,USerialize,UDeserialize)]
struct TemporalOptionLane {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pending:Option<TemporalOption>,
    started:u64,continued:u64,completed:u64,revoked:u64,
    #[serde(default)]
    terminal_credits:u64,
    #[serde(default)]
    contradictory_credits:u64,
    #[serde(default)]
    completed_real_cost:u64,
    /// Opt-in source of tentative routes from the organism's own successful
    /// physical experience; the existing native U1 remains the ONLY voter.
    #[serde(default)]
    autobiographical_enabled:bool,
    #[serde(default)]
    autobiographical_started:u64,
    #[serde(default)]
    variable_depth_enabled:bool,
    /// Adopt added action memory ONLY when incumbent short explanation is
    /// no longer supported and real matched controls justify extension.
    #[serde(default)]
    complexity_need_gate_enabled:bool,
    // These fields DID NOT EXIST in version-1 PR10 checkpoints. Omit
    // absent/default values when reserializing BEFORE integrity hashing.
    // Deserialization still supplies zeros; a historical checkpoint stays
    // byte-identical in the cognition field under the original V1 hash.
    #[serde(default,skip_serializing_if="std::ops::Not::not")]
    contextual_experiments_enabled:bool,
    #[serde(default,skip_serializing_if="agency_no_count")]
    contextual_experiment_proposed:u64,
    #[serde(default,skip_serializing_if="agency_no_count")]
    contextual_experiment_selected:u64,
    /// New opt-in factual observable-state path search, native U1 only.
    /// Omit disabled flag so PRE-PR12 V1 checkpoint hashes still match.
    #[serde(default,skip_serializing_if="std::ops::Not::not")]
    observable_graph_enabled:bool,
    #[serde(default)]
    learned_depth_selected:[u64;6],
    #[serde(default)]
    learned_depth_completed:[u64;6],
}
#[derive(Debug,Clone,USerialize,UDeserialize)]
pub struct TemporalOptionReport {
    pub pending_steps:usize,
    pub started:u64,pub continued:u64,
    pub completed:u64,pub revoked:u64,
    pub terminal_credits:u64,pub contradictory_credits:u64,
    pub completed_real_cost:u64,
    pub autobiographical_enabled:bool,pub autobiographical_started:u64,
    pub variable_depth_enabled:bool,
    pub complexity_need_gate_enabled:bool,
    pub contextual_experiments_enabled:bool,
    pub contextual_experiment_proposed:u64,
    pub contextual_experiment_selected:u64,
    pub observable_graph_enabled:bool,
    pub learned_depth_selected:[u64;6],
    pub learned_depth_completed:[u64;6],
}
impl TemporalOptionLane {
    fn report(&self)->TemporalOptionReport{
        TemporalOptionReport{
            pending_steps:self.pending.as_ref().map_or(0,|o|o.remaining.len()),
            started:self.started,continued:self.continued,
            completed:self.completed,revoked:self.revoked,
            terminal_credits:self.terminal_credits,
            contradictory_credits:self.contradictory_credits,
            completed_real_cost:self.completed_real_cost,
            autobiographical_enabled:self.autobiographical_enabled,
            autobiographical_started:self.autobiographical_started,
            variable_depth_enabled:self.variable_depth_enabled,
            complexity_need_gate_enabled:self.complexity_need_gate_enabled,
            contextual_experiments_enabled:self.contextual_experiments_enabled,
            contextual_experiment_proposed:self.contextual_experiment_proposed,
            contextual_experiment_selected:self.contextual_experiment_selected,
            observable_graph_enabled:self.observable_graph_enabled,
            learned_depth_selected:self.learned_depth_selected,
            learned_depth_completed:self.learned_depth_completed,
        }
    }
    fn reject_pending(&mut self){
        if self.pending.take().is_some(){
            self.revoked=self.revoked.saturating_add(1);
        }
    }
    fn validate(&self,u:&UnifiedConstruction)->UResult<()>{
        let cfg=u.current.learner.carrier_config();
        if !u.current.learner.adaptive_history_suffix_enabled() ||
            self.completed.saturating_add(self.revoked)>self.started ||
            self.terminal_credits>self.completed ||
            self.autobiographical_started>self.started ||
            (self.autobiographical_started>0 && !self.autobiographical_enabled) ||
            (self.variable_depth_enabled && !self.autobiographical_enabled) ||
            (self.complexity_need_gate_enabled && !self.variable_depth_enabled) ||
            (self.contextual_experiments_enabled && !self.complexity_need_gate_enabled) ||
            (self.contextual_experiment_proposed>0 && !self.contextual_experiments_enabled) ||
            (self.contextual_experiment_selected>self.contextual_experiment_proposed) ||
            (self.observable_graph_enabled &&
                !u.current.learner.self_directed_control_enabled()) ||
            self.learned_depth_selected[..2].iter().any(|&n|n>0) ||
            self.learned_depth_completed[..2].iter().any(|&n|n>0) ||
            self.learned_depth_completed.iter().sum::<u64>()>self.completed ||
            self.learned_depth_selected.iter().sum::<u64>()>self.autobiographical_started ||
            self.learned_depth_completed.iter().zip(self.learned_depth_selected)
                .any(|(done,chosen)|*done>chosen) ||
            self.contradictory_credits>self.revoked ||
            self.completed_real_cost>u.total_external_cost ||
            self.continued>self.started.saturating_mul(cfg.planning_depth as u64){
            return Err(UError::InvalidData)
        }
        if let Some(p)=self.pending.as_ref(){
            if !(1..=4).contains(&p.remaining.len()) ||
                p.remaining.len()>=cfg.planning_depth ||
                p.goal.len()!=cfg.width ||!p.goal.iter().any(Option::is_some) ||
                p.realized_cost>u.total_external_cost ||
                p.executed_steps.saturating_add(p.remaining.len())>cfg.planning_depth ||
                (p.credit_eligible && p.executed_steps==0) ||
                (p.learned_depth!=0 &&
                    (!(2..=5).contains(&p.learned_depth) ||
                     p.executed_steps.saturating_add(p.remaining.len())!=p.learned_depth)) ||
                p.last_verified_post!=u.current.learner.current() ||
                p.event!=u.current.learner.records().len() ||
                p.initiating_fields.iter().any(|v|!v.is_finite()||
                    !(0.0..=1.0).contains(v)) ||
                p.remaining.iter().any(|r|r.action>=cfg.actions||
                    r.expected_post.len()!=cfg.width) ||
                p.remaining.last().is_none_or(|last|
                    udistance(&last.expected_post,Some(&p.goal))!=0.0){
                return Err(UError::InvalidData)
            }
        }
        Ok(())
    }
}

/// Opt-in grounded *instrumental* credit. A causal/world observation is never
/// labelled a successful choice merely because a sensor reading was novel.
/// The unfinished preparation is an eligibility trace, not trusted POST.
#[derive(Debug,Clone,USerialize,UDeserialize)]
struct InstrumentalPending {
    goal:Vec<Option<i64>>,expected_post:Vec<i64>,
    fields:[f32;5],event:usize,
}
#[derive(Debug,Default,Clone,USerialize,UDeserialize)]
struct InstrumentalCredit {
    facts_without_progress:u64,
    actual_progress_credited:u64,
    validated_preparations_credited:u64,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pending:Option<InstrumentalPending>,
}
#[derive(Debug,Clone,USerialize,UDeserialize)]
pub struct InstrumentalCreditReport {
    pub facts_without_progress:u64,
    pub actual_progress_credited:u64,
    pub validated_preparations_credited:u64,
    pub pending_preparation:bool,
}
impl InstrumentalCredit {
    fn report(&self)->InstrumentalCreditReport {
        InstrumentalCreditReport{
            facts_without_progress:self.facts_without_progress,
            actual_progress_credited:self.actual_progress_credited,
            validated_preparations_credited:self.validated_preparations_credited,
            pending_preparation:self.pending.is_some(),
        }
    }
    fn validate(&self,current:&UnifiedDomain,archived:&[UnifiedDomain],
                freeze:bool)->UResult<()> {
        let recorded=archived.iter().chain(std::iter::once(current))
            .map(|d|d.learner.records().len()).sum::<usize>();
        if self.facts_without_progress
            .checked_add(self.actual_progress_credited)
            .is_none_or(|n|n>recorded as u64)
            || self.validated_preparations_credited>self.actual_progress_credited {
            return Err(UError::InvalidData)
        }
        if let Some(p)=&self.pending {
            let cfg=current.learner.carrier_config();
            if freeze || p.goal.len()!=cfg.width ||
                p.expected_post.len()!=cfg.width ||
                p.event!=current.learner.records().len() ||
                p.fields.iter().any(|v|!v.is_finite()||!(0.0..=1.0).contains(v)){
                return Err(UError::InvalidData)
            }
        }
        Ok(())
    }
}
/// Independent counters: outcome evidence and action-choice credit are not
/// synonyms. No evaluator-supplied label, action role, or source ID.
fn agency_no_count(value:&u64)->bool{*value==0}
#[derive(Debug,Clone,USerialize,UDeserialize)]
pub struct AgencyCreditReport {
    pub externally_forced_actions:u64,
    pub self_selected_actions:u64,
    /// These execute a previously native-selected higher-level option;
    /// they are NOT freshly selected U1 primitive motor actions.
    #[serde(default,skip_serializing_if="agency_no_count")]
    pub option_continuation_actions:u64,
}
#[derive(Debug,Clone,USerialize,UDeserialize)]
pub struct UnifiedPost {pub observation:Vec<i64>,pub cost:u64,pub reward:f32}
#[derive(Debug,Clone,USerialize,UDeserialize)]
pub struct UnifiedStep {
    pub action:usize,pub source:String,pub operations:Vec<String>,pub selected_fields:[f32;5],
    pub native_score:f32,pub factual_utility:f32,pub event:usize,pub computation_work:u64,
    /// Diagnostic only; a proposed path is not a selected or executed path.
    #[serde(default,skip_serializing_if="std::ops::Not::not")]
    pub certified_chain_proposed:bool,
    /// A proposed invisible prerequisite is NOT evidence it was executed.
    #[serde(default,skip_serializing_if="std::ops::Not::not")]
    pub predictive_history_plan_proposed:bool,
    /// Proposal availability is NOT the same as executed curiosity.
    #[serde(default,skip_serializing_if="std::ops::Not::not")]
    pub active_pair_probe_proposed:bool,
    /// Novel action-context coverage is evidence-gathering, never a
    /// fabricated prediction of the terminal goal.
    #[serde(default,skip_serializing_if="std::ops::Not::not")]
    pub contextual_experiment_proposed:bool,
    /// An entirely real PRE/motor/POST route through visibly distinct states.
    /// This is a tentative historical plan, NOT certified general dynamics.
    #[serde(default,skip_serializing_if="std::ops::Not::not")]
    pub observable_empirical_route_proposed:bool,
    /// Explicitly a proposal based on a TWO-action latent history suffix.
    #[serde(default,skip_serializing_if="std::ops::Not::not")]
    pub adaptive_suffix_plan_proposed:bool,
    /// Tentative three-motor option found from the organism's OWN real POSTs.
    #[serde(default,skip_serializing_if="std::ops::Not::not")]
    pub autobiographical_option_proposed:bool,
    /// Exposed meta-decision from real history; not an extra action actor.
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub complexity_need_decision:Option<String>,
}
#[derive(Debug,Clone,USerialize,UDeserialize)]
pub struct UnifiedReport {
    pub domain:String,pub archived_domains:Vec<String>,pub learner:crate::construction::SnapshotReport,
    pub native_meta_observations:u64,pub native_weights:[f32;5],pub operations:[u64;4],
    pub computation_work:u64,pub total_external_cost:u64,pub controller_frozen:bool,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub self_revision:Option<MetaRevisionReport>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub agency_credit:Option<AgencyCreditReport>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub instrumental_credit:Option<InstrumentalCreditReport>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub temporal_options:Option<TemporalOptionReport>,
}
#[derive(USerialize,UDeserialize)]
#[serde(deny_unknown_fields)]
struct UnifiedStored {version:u32,carrier:Vec<u8>,cognition:UnifiedConstruction,sequence:u64,emergency:bool,protection_sequence:u64,hash:u64}
#[derive(Clone)]
struct UnifiedOption {p:PhaseCognitiveProposal,kind:u8,source:&'static str,output:Option<Vec<i64>>}
fn uword(mut n:u64)->u64 {n=n.wrapping_add(0x9e3779b97f4a7c15);n=(n^(n>>30)).wrapping_mul(0xbf58476d1ce4e5b9);n=(n^(n>>27)).wrapping_mul(0x94d049bb133111eb);n^(n>>31)}
fn usensory(x:&[i64])->Vec<f32>{x.iter().map(|&v| (0.5+0.5*(v as f64)/(1.0+(v as f64).abs())) as f32).collect()}
fn udistance(x:&[i64],goal:Option<&[Option<i64>]>)->f64 {
    goal.map(|g| x.iter().zip(g).filter_map(|(&v,t)|t.map(|a|(v as i128-a as i128).abs() as f64)).sum()).unwrap_or(0.0)
}
/// A bounded, authored terminal reward: only a REAL reached goal earns
/// positive U1 credit, discounted by the sum of actual protected action costs.
/// This is NOT a self-invented utility formula or evidence of cold transfer.
fn temporal_realized_goal_utility(actual_cost:u64)->f32 {
    (1.0/(1.0+0.10*(actual_cost as f32).ln_1p())).clamp(0.0,1.0)
}
fn ufields(p:&[i64],goal:Option<&[Option<i64>]>,novelty:f32,disagreement:f32,confidence:f32,cost:u64)->[f32;5] {
    [if goal.is_some(){(1.0/(1.0+udistance(p,goal))) as f32}else{0.0},novelty.clamp(0.0,1.0),disagreement.clamp(0.0,1.0),confidence.clamp(0.0,1.0),1.0/(1.0+(cost as f32).ln_1p())]
}

// Experimental same-organism evaluator revision. This does not edit Rust,
// execute actuators, access hidden world outcomes, or bypass native U1.
use crate::construction::vm::{Expr as MExpr,Ty as MTy,Context as MContext,Library as MLibrary,Meter as MMeter};
use crate::construction::induction::{Example as MExample,InductionConfig as MInductionConfig};
use crate::construction::search_policy::{Objective as MObjective,SearchPolicy as MSearchPolicy,SearchLimits as MSearchLimits};
const M_MAX_EPISODE:usize=8;
#[derive(Debug,Clone,USerialize,UDeserialize)]
struct MetaWitness {episode:u64,features:[i64;5],outcome:i64}
#[derive(Debug,Clone,USerialize,UDeserialize)]
struct MetaShadow {program:MExpr,trained_episodes:u64}
#[derive(Debug,Clone,USerialize,UDeserialize)]
struct MetaReceipt {
    program:MExpr,incumbent:Option<MExpr>,trained_episodes:u64,
    future:Vec<MetaWitness>,before:u64,after:u64,
    #[serde(default,skip_serializing_if="std::ops::Not::not")]
    revoked:bool,
    #[serde(default,skip_serializing_if="Vec::is_empty")]
    rollback_witness:Vec<MetaWitness>,
}
#[derive(Debug,Default,USerialize,UDeserialize)]
struct MetaRevision {
    active:Option<MExpr>,shadow:Option<MetaShadow>,
    rows:Vec<MetaWitness>,pending_steps:Vec<[i64;5]>,
    pending_goal:Option<Vec<Option<i64>>>,
    episodes:u64,attempts:u32,rejected:u32,work:u64,next_trial:u64,
    #[serde(default,skip_serializing_if="meta_zero_u32")]
    rollbacks:u32,
    receipts:Vec<MetaReceipt>,
    /// One-bit readout ablation for a frozen, otherwise identical organism.
    #[serde(default,skip_serializing_if="meta_false")]
    readout_ablated:bool,
    /// Ordinary decisions must keep the old readout until causal action proof.
    #[serde(default,skip_serializing_if="meta_false")]
    behavior_guard_enabled:bool,
    /// One-use: even a protected callback fault consumes the trial attempt.
    #[serde(default,skip_serializing_if="meta_false")]
    behavior_attempted:bool,
    #[serde(default,skip_serializing_if="Option::is_none")]
    behavioral_audit:Option<MetaBehaviorAudit>,
    /// Only the evaluation sandbox can flip this; never serialized into a
    /// deployable checkpoint or externally exposed as actuator authority.
    #[serde(skip)]
    sandbox_candidate_arm:bool,
}
#[derive(Debug,Clone,USerialize,UDeserialize)]
pub struct MetaRevisionReport {
    pub episodes:u64,pub steps:usize,pub attempts:u32,pub rejected:u32,
    pub generations:usize,pub shadow:bool,pub computation_work:u64,
    pub unfinished_steps:usize,pub rollbacks:u32,pub active_program:Option<String>,
    #[serde(default)]
    pub readout_ablated:bool,
    #[serde(default)]
    pub behavior_guard_enabled:bool,
    #[serde(default)]
    pub behavior_attempted:bool,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub behavioral_outcome:Option<MetaBehaviorSummary>,
}

// T-SR2: an evaluator-only behavioral canary. All results below are collected
// from actual protected POST callbacks in independent FROZEN sandbox siblings.
// No post labels, scripted action sequence or unexecuted guessed outcome.
#[derive(Debug,Clone,USerialize,UDeserialize)]
pub struct MetaBehaviorTask {
    /// Used by the evaluator's non-regression check, NEVER forwarded to U1.
    pub cohort:u16,
    pub goal:Vec<Option<i64>>,
    pub offers:Vec<UOffer>,
    pub max_actions:u8,
}
#[derive(Debug,Clone,USerialize,UDeserialize)]
pub struct MetaBehaviorStep {
    pub action:usize,
    pub post:Vec<i64>,
    pub cost:u64,
    pub selected_goal_score:f32,
}
#[derive(Debug,Clone,USerialize,UDeserialize)]
pub struct MetaBehaviorArm {
    pub steps:Vec<MetaBehaviorStep>,
}
#[derive(Debug,Clone,USerialize,UDeserialize)]
pub struct MetaBehaviorCase {
    pub task:MetaBehaviorTask,
    /// [incumbent without new VM readout, candidate with VM readout].
    pub arms:[MetaBehaviorArm;2],
}
#[derive(Debug,Clone,PartialEq,Eq,USerialize,UDeserialize)]
pub struct MetaBehaviorSummary {
    pub cases:usize,
    pub incumbent_success:usize,pub candidate_success:usize,
    pub incumbent_actions:usize,pub candidate_actions:usize,
    pub incumbent_cost:u64,pub candidate_cost:u64,
    pub changed_initial_actions:usize,pub changed_initial_scores:usize,
    pub nonregressed_cohorts:bool,
    pub qualified:bool,
}
#[derive(Debug,Clone,USerialize,UDeserialize)]
pub struct MetaBehaviorAudit {
    /// Diagnostic identity of the EXACT common checkpoint, not a signature.
    pub checkpoint_fingerprint:u64,
    pub initial_observation:Vec<i64>,
    pub cases:Vec<MetaBehaviorCase>,
    pub summary:MetaBehaviorSummary,
}
impl MetaBehaviorAudit {
    fn compute_summary(&self)->UResult<MetaBehaviorSummary> {
        if !(8..=64).contains(&self.cases.len()) ||
            self.initial_observation.is_empty() || self.initial_observation.len()>6 {
            return Err(UError::InvalidData)
        }
        let mut successes=[0usize;2];
        let mut actions=[0usize;2];
        let mut costs=[0u64;2];
        let mut cohorts=std::collections::BTreeMap::<u16,[usize;3]>::new();
        let mut changed_actions=0usize;
        let mut changed_scores=0usize;
        for trial in &self.cases{
            let t=&trial.task;
            if t.goal.len()!=self.initial_observation.len() ||
                !t.goal.iter().any(Option::is_some) ||
                udistance(&self.initial_observation,Some(&t.goal))==0.0 ||
                !(1..=24).contains(&t.max_actions) ||
                t.offers.is_empty() || t.offers.len()>8 ||
                !t.offers.iter().any(|o|o.available) {
                return Err(UError::InvalidData)
            }
            let entry=cohorts.entry(t.cohort).or_default();
            entry[0]+=1;
            for arm in 0..2 {
                let list=&trial.arms[arm].steps;
                if list.is_empty() || list.len()>t.max_actions as usize{
                    return Err(UError::InvalidData)
                }
                let mut state=&self.initial_observation;
                let mut reached=false;
                let mut total=0u64;
                for row in list {
                    if reached || row.action>=t.offers.len() ||
                        !t.offers[row.action].available ||
                        row.cost>t.offers[row.action].max_cost ||
                        row.post.len()!=self.initial_observation.len() ||
                        !row.selected_goal_score.is_finite() ||
                        !(0.0..=1.0).contains(&row.selected_goal_score){
                        return Err(UError::InvalidData)
                    }
                    state=&row.post;
                    total=total.checked_add(row.cost).ok_or(UError::Capacity)?;
                    reached=udistance(state,Some(&t.goal))==0.0;
                }
                // Every failure must consume its entire declared horizon.
                if !reached && list.len()!=t.max_actions as usize{
                    return Err(UError::InvalidData)
                }
                successes[arm]+=usize::from(reached);
                entry[arm+1]+=usize::from(reached);
                actions[arm]+=list.len();
                costs[arm]=costs[arm].checked_add(total).ok_or(UError::Capacity)?;
            }
            let baseline=&trial.arms[0].steps[0];
            let challenger=&trial.arms[1].steps[0];
            changed_actions+=usize::from(baseline.action!=challenger.action);
            changed_scores+=usize::from(
                (baseline.selected_goal_score-challenger.selected_goal_score).abs()>1e-6);
        }
        if cohorts.len()<2 || cohorts.values().any(|c|c[0]<4){
            return Err(UError::InvalidData)
        }
        let nonregressed=cohorts.values().all(|c|c[2]>=c[1]);
        let qualified=nonregressed && successes[1]>=successes[0] &&
            actions[1].saturating_mul(10)<=actions[0].saturating_mul(9) &&
            costs[1]<=costs[0] && changed_actions>0 && changed_scores>0;
        Ok(MetaBehaviorSummary {
            cases:self.cases.len(),
            incumbent_success:successes[0],candidate_success:successes[1],
            incumbent_actions:actions[0],candidate_actions:actions[1],
            incumbent_cost:costs[0],candidate_cost:costs[1],
            changed_initial_actions:changed_actions,
            changed_initial_scores:changed_scores,
            nonregressed_cohorts:nonregressed,qualified,
        })
    }
    fn validate(&self)->UResult<()> {
        if self.compute_summary()?!=self.summary {
            return Err(UError::InvalidData)
        }
        Ok(())
    }
}
fn meta_zero_u32(n:&u32)->bool{*n==0}
fn meta_false(v:&bool)->bool{!*v}
struct MetaObjective<'a>{examples:&'a [MetaWitness],lib:MLibrary}
impl MObjective for MetaObjective<'_>{
    fn input_width(&self)->usize{5}
    fn library(&self)->&MLibrary{&self.lib}
    fn score(&mut self,p:&MExpr,meter:&mut MMeter)->UResult<u64>{
        let mut total=0u64;
        for r in self.examples{
            let prediction=p.eval(MContext{input:&r.features,action:0,state:&[],args:&[]},
                &self.lib,meter)?.int()?.clamp(0,100);
            total=total.saturating_add(prediction.abs_diff(r.outcome));
        }
        Ok(total)
    }
}
impl MetaRevision {
    fn report(&self)->MetaRevisionReport{
        MetaRevisionReport{episodes:self.episodes,steps:self.rows.len(),attempts:self.attempts,
            rejected:self.rejected,generations:self.receipts.len(),
            shadow:self.shadow.is_some(),computation_work:self.work,
            unfinished_steps:self.pending_steps.len(),rollbacks:self.rollbacks,
            active_program:self.active.as_ref().map(|p|format!("{p:?}")),
            readout_ablated:self.readout_ablated,
            behavior_guard_enabled:self.behavior_guard_enabled,
            behavior_attempted:self.behavior_attempted,
            behavioral_outcome:self.behavioral_audit.as_ref().map(|a|a.summary.clone())}
    }
    fn features(f:[f32;5])->[i64;5] {
        f.map(|v|(v.clamp(0.0,1.0)*100.0).round() as i64)
    }
    fn valid_program(p:&MExpr)->bool{
        let lib=MLibrary::default();
        p.nodes()<=11 && p.ty(&lib)==Ok(MTy::Int) &&
            p.references_valid(5,0,0,&lib)
    }
    fn score(p:Option<&MExpr>,features:&[i64;5])->Option<i64>{
        let Some(expr)=p else {return Some(features[0])};
        if !Self::valid_program(expr){return None}
        let mut meter=MMeter::new(256);
        expr.eval(MContext{input:features,action:0,state:&[],args:&[]},
            &MLibrary::default(),&mut meter).ok()?.int().ok().map(|v|v.clamp(0,100))
    }
    fn loss(p:Option<&MExpr>,examples:&[MetaWitness])->Option<u64>{
        examples.iter().try_fold(0_u64,|sum,r|{
            Some(sum.saturating_add(Self::score(p,&r.features)?.abs_diff(r.outcome)))
        })
    }
    fn adjust(&self,fields:&mut [f32;5]){
        if self.readout_ablated ||
           (self.behavior_guard_enabled && !self.sandbox_candidate_arm &&
            !self.behavioral_audit.as_ref().is_some_and(|a|a.summary.qualified)) {
            return
        }
        if let Some(code)=self.active.as_ref(){
            if let Some(v)=Self::score(Some(code),&Self::features(*fields)){
                fields[0]=v as f32/100.0
            }
        }
    }
    fn validate(&self)->UResult<()>{
        if self.rows.len()>128 || self.pending_steps.len()>M_MAX_EPISODE ||
            self.receipts.len()>3 || self.attempts>12 || self.rejected>self.attempts ||
            (self.readout_ablated && self.active.is_none()) ||
            (!self.behavior_guard_enabled && (self.behavior_attempted || self.behavioral_audit.is_some())) ||
            (self.behavioral_audit.is_some() && !self.behavior_attempted) ||
            self.behavioral_audit.as_ref().is_some_and(|a|a.validate().is_err()) ||
            self.rollbacks!=self.receipts.iter().filter(|p|p.revoked).count() as u32 ||
            self.pending_goal.is_none()!=self.pending_steps.is_empty() ||
            self.pending_goal.as_ref().is_some_and(|g|g.len()>32) ||
            self.active.as_ref().is_some_and(|p|!Self::valid_program(p)) ||
            self.shadow.as_ref().is_some_and(|s|s.trained_episodes>self.episodes||!Self::valid_program(&s.program)) ||
            self.rows.iter().any(|r|r.episode>self.episodes||
                !(0..=100).contains(&r.outcome)||
                r.features.iter().any(|x|!(0..=100).contains(x)))||
            self.rows.windows(2).any(|w|w[0].episode>w[1].episode)||
            self.pending_steps.iter().flatten().any(|x|!(0..=100).contains(x))||
            self.receipts.last().is_some_and(|p|if p.revoked {
                self.active!=p.incumbent
            }else{self.active.as_ref()!=Some(&p.program)}){
            return Err(UError::InvalidData)
        }
        for p in &self.receipts{
            if !Self::valid_program(&p.program)||p.future.len()<4||p.future.len()>64||
                p.future.iter().any(|r|r.episode<=p.trained_episodes)||
                Self::loss(p.incumbent.as_ref(),&p.future)!=Some(p.before)||
                Self::loss(Some(&p.program),&p.future)!=Some(p.after)||
                p.before==0||p.after.saturating_mul(100)>p.before.saturating_mul(85){
                return Err(UError::InvalidData)
            }
            if p.revoked{
                let later=p.future.last().map_or(u64::MAX,|r|r.episode);
                let reobserved=p.rollback_witness.iter()
                    .map(|r|r.episode).collect::<std::collections::BTreeSet<_>>().len();
                if p.rollback_witness.len()<4||p.rollback_witness.len()>64||reobserved<4||
                    p.rollback_witness.iter().any(|r|r.episode<=later)||
                    !p.rollback_witness.iter().any(|r|r.outcome==0)||
                    !p.rollback_witness.iter().any(|r|r.outcome>=90){
                    return Err(UError::InvalidData)
                }
                let old=Self::loss(p.incumbent.as_ref(),&p.rollback_witness).ok_or(UError::InvalidData)?;
                let new=Self::loss(Some(&p.program),&p.rollback_witness).ok_or(UError::InvalidData)?;
                if new<=old.saturating_add(100)||new.saturating_mul(100)<old.saturating_mul(125){
                    return Err(UError::InvalidData)
                }
            }else if !p.rollback_witness.is_empty(){return Err(UError::InvalidData)}
        }
        Ok(())
    }
    // A changed goal invalidates pending credit instead of fabricating labels.
    // Only the REAL outcome of a completed action can close an episode.
    fn observe(&mut self,fields:[f32;5],goal:&[Option<i64>],pre:&[i64],post:&[i64])->u64{
        if pre.len()!=goal.len()||post.len()!=goal.len()||
            !goal.iter().any(Option::is_some){
            // No goal or malformed observation interrupts credit assignment.
            // Never span an episode across unobserved external operations.
            self.pending_steps.clear();self.pending_goal=None;
            return 0
        }
        if self.pending_goal.as_deref()!=Some(goal){
            self.pending_steps.clear();
            self.pending_goal=Some(goal.to_vec());
        }
        if udistance(pre,Some(goal))==0.0 {
            self.pending_steps.clear();
            self.pending_goal=None;
            return 0
        }
        self.pending_steps.push(Self::features(fields));
        let reached=udistance(post,Some(goal))==0.0;
        if !reached&&self.pending_steps.len()<M_MAX_EPISODE{return 0}
        let len=self.pending_steps.len();
        self.episodes=self.episodes.saturating_add(1);
        for (i,features) in self.pending_steps.drain(..).enumerate(){
            // Developer-specified objective, not authored evaluator code.
            // Earlier actions inherit future success with bounded time cost.
            let outcome=if reached {(100-5*(len-i-1) as i64).max(0)}else{0};
            self.rows.push(MetaWitness{episode:self.episodes,features,outcome});
        }
        self.pending_goal=None;
        if self.rows.len()>128 {
            let drop_count=self.rows.len()-128;self.rows.drain(..drop_count);
        }
        let before=self.work;
        self.revoke_on_new_contradictory_episodes();
        if self.shadow.is_some(){self.prospective_gate()}else{self.propose_from_own_error()}
        self.work.saturating_sub(before)
    }
    /// Post-admission evidence can revoke a previously accepted evaluator.
    /// It measures predictive drift only, NOT counterfactual policy benefit.
    fn revoke_on_new_contradictory_episodes(&mut self){
        let Some(last)=self.receipts.last().cloned() else{return};
        if last.revoked||self.active.as_ref()!=Some(&last.program){return}
        let boundary=last.future.last().map_or(last.trained_episodes,|r|r.episode);
        let followup:Vec<_>=self.rows.iter().filter(|r|r.episode>boundary)
            .take(64).cloned().collect();
        let distinct=followup.iter().map(|r|r.episode)
            .collect::<std::collections::BTreeSet<_>>().len();
        if distinct<4||!followup.iter().any(|r|r.outcome==0)||
            !followup.iter().any(|r|r.outcome>=90){return}
        let old=Self::loss(last.incumbent.as_ref(),&followup).unwrap_or(u64::MAX);
        let new=Self::loss(Some(&last.program),&followup).unwrap_or(u64::MAX);
        if new>old.saturating_add(100)&&
            new.saturating_mul(100)>=old.saturating_mul(125){
            self.active=last.incumbent;
            self.shadow=None;
            let receipt=self.receipts.last_mut().expect("validated receipt");
            receipt.revoked=true;
            receipt.rollback_witness=followup;
            self.rollbacks=self.rollbacks.saturating_add(1);
            self.next_trial=self.episodes.saturating_add(6);
        }
    }
    fn propose_from_own_error(&mut self){
        if self.receipts.len()>=3||self.attempts>=12||self.episodes<self.next_trial{return}
        let training=&self.rows[self.rows.len().saturating_sub(96)..];
        let distinct=training.iter().map(|r|r.episode).collect::<std::collections::BTreeSet<_>>().len();
        if training.len()<24||distinct<8||!training.iter().any(|r|r.outcome==0)||
           !training.iter().any(|r|r.outcome>=90){return}
        let incumbent_loss=Self::loss(self.active.as_ref(),training).unwrap_or(u64::MAX);
        if incumbent_loss<150{return}
        self.attempts+=1;self.next_trial=self.episodes.saturating_add(6);
        let samples:Vec<MExample>=training.iter().map(|r|MExample{
            input:r.features.to_vec(),output:r.outcome}).collect();
        let induced=crate::construction::induction::infer(
            &samples,&MLibrary::default(),
            &MInductionConfig{max_nodes:7,max_terms:1024,work:100_000});
        let mut program=None;
        if let Ok(out)=induced {self.work=self.work.saturating_add(out.work);program=out.expression;}
        if program.is_none(){
            let mut objective=MetaObjective{examples:training,lib:MLibrary::default()};
            let limits=MSearchLimits{work:150_000,evaluations:128,max_nodes:9,
                frontier:256,edits_per_expansion:16};
            let seed=uword(self.episodes^self.work);
            match crate::construction::search_policy::search(&MSearchPolicy::default(),
                MExpr::Input(0),&mut objective,&limits,seed){
                Ok(out)=>{self.work=self.work.saturating_add(out.work);program=Some(out.program);}
                Err(_)=>{self.work=self.work.saturating_add(limits.work);}
            }
        }
        let Some(code)=program else {self.rejected+=1;return};
        if !Self::valid_program(&code)||self.active.as_ref()==Some(&code)||
            (self.active.is_none()&&code==MExpr::Input(0)){
            self.rejected+=1;return
        }
        let next_loss=Self::loss(Some(&code),training).unwrap_or(u64::MAX);
        if next_loss.saturating_mul(100)>incumbent_loss.saturating_mul(80){
            self.rejected+=1;return
        }
        self.shadow=Some(MetaShadow{program:code,trained_episodes:self.episodes});
    }
    fn prospective_gate(&mut self){
        let Some(shadow)=&self.shadow else {return};
        let future:Vec<_>=self.rows.iter().filter(|r|r.episode>shadow.trained_episodes)
            .take(64).cloned().collect();
        let independent=future.iter().map(|r|r.episode).collect::<std::collections::BTreeSet<_>>().len();
        if independent<4{return}
        let has_both=future.iter().any(|r|r.outcome==0)&&future.iter().any(|r|r.outcome>=90);
        if !has_both&&independent<8{return}
        let before=Self::loss(self.active.as_ref(),&future).unwrap_or(u64::MAX);
        let after=Self::loss(Some(&shadow.program),&future).unwrap_or(u64::MAX);
        let mut no_regression=has_both;
        for failed in [false,true]{
            let group:Vec<_>=future.iter().filter(|r|(r.outcome==0)==failed).cloned().collect();
            if group.is_empty(){no_regression=false;break}
            if Self::loss(Some(&shadow.program),&group).unwrap_or(u64::MAX)>
                Self::loss(self.active.as_ref(),&group).unwrap_or(u64::MAX){
                no_regression=false
            }
        }
        if no_regression&&before>0&&after.saturating_mul(100)<=before.saturating_mul(85){
            let program=shadow.program.clone();
            self.receipts.push(MetaReceipt{program:program.clone(),incumbent:self.active.clone(),
                trained_episodes:shadow.trained_episodes,future,before,after,
                revoked:false,rollback_witness:Vec::new()});
            self.active=Some(program);
        }else{self.rejected+=1}
        self.shadow=None;
    }
}

impl ScientificRuntime {
    pub fn new_constructing(config:UConfig,mut policy:UPolicy,initial:Vec<i64>,name:&str)->UResult<Self> {
        config.validate()?;
        if initial.len()!=config.width || name.is_empty() || name.len()>128 {return Err(UError::InvalidData);}
        if let Some(e)=&mut policy.evidence {e.total_alpha=e.total_alpha.min(0.005);}
        let mut learner=ULearner::new(config.clone(),initial.clone())?;
        learner.enable_autonomy(policy.clone())?;learner.manage_from_carrier()?;
        let mut evo=EvoPhase::new(crate::EvoConfig{sensory_cells:config.width,motor_cells:config.actions,hdc_dim:32,phase_learning_rate:1.0,..Default::default()});
        evo.enable_phase_native_planning(PhaseNativeConfig::default());
        if !evo.enable_phase_native_meta_control(PhaseMetaControlConfig::default()){return Err(UError::InvalidData);}
        evo.set_phase_native_meta_monotone_evidence(true);evo.observe_initial_real(&usensory(&initial),false);
        let mut rt=Self::new(evo).map_err(|_|UError::InvalidData)?;
        rt.constructing=Some(UnifiedConstruction{current:UnifiedDomain{name:name.into(),learner,last_construction:0,pending_credit:None},
            archived:Vec::new(),nonce:0,operations:[0;4],computation_work:0,total_external_cost:0,max_external_cost:policy.max_external_cost,freeze_controller:false,education:None,self_revision:None,agency_credit:None,instrumental_credit:None,temporal_options:None});
        rt.fresh_observation_required=false;Ok(rt)
    }
    pub fn construction_learner(&self)->Option<&ULearner>{Some(&self.constructing.as_ref()?.current.learner)}
    /// Decide before any experience whether the native action policy may be
    /// credited for outcomes of externally FORCED identification actions.
    /// When enabled, those actions still teach the causal/world Learner,
    /// but they cannot train U1's choice weights.
    pub fn enable_constructing_agency_aware_credit(&mut self)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller || u.agency_credit.is_some() ||
            !u.archived.is_empty() || !u.current.learner.records().is_empty() ||
            u.total_external_cost!=0{return Err(UError::Busy)}
        u.agency_credit=Some(AgencyCreditReport{
            externally_forced_actions:0,self_selected_actions:0,
            option_continuation_actions:0,
        });
        Ok(())
    }
    /// A U1 choice is credited for goal utility only after REAL progress.
    /// Factual acquisition remains unconstrained and can be goal-free.
    pub fn enable_constructing_instrumental_credit(&mut self)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller || u.instrumental_credit.is_some() ||
            u.agency_credit.is_none() || u.temporal_options.is_some() ||
            !u.archived.is_empty() ||
            !u.current.learner.records().is_empty() ||
            u.total_external_cost!=0 {return Err(UError::Busy)}
        u.instrumental_credit=Some(InstrumentalCredit::default());
        Ok(())
    }
    /// Opt the SAME construction Learner into factual, self-selected probing.
    /// Goal and allowed offers are still host-supplied constraints, while the
    /// chosen action must compete in the existing U1 decision and pass the
    /// original human protection/one-use execution permit.
    pub fn enable_constructing_self_directed_control(&mut self)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller{return Err(UError::Blocked)}
        u.current.learner.enable_self_directed_control()
    }
    /// Abatable research flag, persists in the SAME Learner checkpoint.
    /// Never changes HumanProtection, actuator permissions or reward.
    pub fn enable_constructing_certified_chains(&mut self)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller{return Err(UError::Blocked)}
        u.current.learner.enable_certified_chains()
    }
    /// Same Learner/U1, with a factual predictor keyed by action history.
    pub fn enable_constructing_predictive_history_states(&mut self)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller{return Err(UError::Blocked)}
        u.current.learner.enable_predictive_history_states()
    }
    /// The SAME factual model may discriminate identical visible states by
    /// observed action suffix length one or two. No new action authority.
    pub fn enable_constructing_adaptive_history_suffix(&mut self)->UResult<()>{
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller{return Err(UError::Blocked)}
        u.current.learner.enable_adaptive_history_suffix()
    }
    /// Install higher-level OPTION EXECUTION only for already learned,
    /// prospectively certified causal routes. No plan may be host-supplied.
    /// U1 must explicitly select the first primitive action itself.
    pub fn enable_constructing_temporal_options(&mut self)->UResult<()>{
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller||u.temporal_options.is_some() ||
            !u.current.learner.adaptive_history_suffix_enabled() ||
            u.instrumental_credit.is_some(){
            return Err(UError::Busy)
        }
        u.temporal_options=Some(TemporalOptionLane::default());
        Ok(())
    }
    /// Discover short executable action hypotheses from this organism's OWN
    /// unscheduled successful physical histories, then leave selection to U1.
    /// This optional stage is ablatable without rewriting the Learner or U1.
    pub fn enable_constructing_autobiographical_options(&mut self)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller || u.instrumental_credit.is_some() ||
            !u.current.learner.self_directed_control_enabled() ||
            !u.current.learner.adaptive_history_suffix_enabled(){
            return Err(UError::Busy)
        }
        let lane=u.temporal_options.as_mut().ok_or(UError::NoEvidence)?;
        if lane.autobiographical_enabled || lane.pending.is_some(){
            return Err(UError::Busy)
        }
        lane.autobiographical_enabled=true;
        Ok(())
    }
    /// Learning the required action-history length from real contradictions
    /// is a separate PRE-EXPERIENCE opt-in. It never alters primitive U1 or
    /// the protection boundary and cannot be enabled after peeking at outcomes.
    pub fn enable_constructing_variable_depth_options(&mut self)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller || u.instrumental_credit.is_some() ||
            u.total_external_cost!=0 ||
            !u.current.learner.records().is_empty(){
            return Err(UError::Busy)
        }
        let lane=u.temporal_options.as_mut().ok_or(UError::NoEvidence)?;
        if !lane.autobiographical_enabled || lane.variable_depth_enabled ||
            lane.pending.is_some(){
            return Err(UError::Busy)
        }
        lane.variable_depth_enabled=true;
        Ok(())
    }
    /// Opt-in pre-experience decision on whether more history is NEEDED.
    /// Keeps the same protected learner and the original native U1 as actor.
    pub fn enable_constructing_complexity_need_gate(&mut self)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller || u.instrumental_credit.is_some() ||
            u.total_external_cost!=0 ||
            !u.current.learner.records().is_empty(){
            return Err(UError::Busy)
        }
        let lane=u.temporal_options.as_mut().ok_or(UError::NoEvidence)?;
        if !lane.variable_depth_enabled || lane.complexity_need_gate_enabled ||
            lane.pending.is_some(){return Err(UError::Busy)}
        lane.complexity_need_gate_enabled=true;
        Ok(())
    }
    /// Opt-in BEFORE physical experience to allow the same U1 to test
    /// bounded executable paths made only from actual distinct-state
    /// PRE/action/POST edges; no free motor authority.
    pub fn enable_constructing_observable_factual_routes(&mut self)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller || u.instrumental_credit.is_some() ||
            u.total_external_cost!=0 ||
            !u.current.learner.records().is_empty() ||
            !u.current.learner.self_directed_control_enabled(){
            return Err(UError::Busy)
        }
        let lane=u.temporal_options.as_mut().ok_or(UError::NoEvidence)?;
        if lane.observable_graph_enabled || lane.pending.is_some(){
            return Err(UError::Busy)
        }
        lane.observable_graph_enabled=true;
        Ok(())
    }
    /// Opt-in to active context-coverage proposals ONLY while existing
    /// simplicity-first control declares insufficient evidence. No
    /// pre-scripted actor or unauthorized motor sequence is introduced.
    pub fn enable_constructing_contextual_experiments(&mut self)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller || u.instrumental_credit.is_some() ||
            u.total_external_cost!=0 ||
            !u.current.learner.records().is_empty() ||
            !u.current.learner.active_pair_probes_enabled(){
            return Err(UError::Busy)
        }
        let lane=u.temporal_options.as_mut().ok_or(UError::NoEvidence)?;
        if !lane.complexity_need_gate_enabled ||
            lane.contextual_experiments_enabled || lane.pending.is_some(){
            return Err(UError::Busy)
        }
        lane.contextual_experiments_enabled=true;
        Ok(())
    }
    /// The same U1 may elect to test an underobserved action pair. No other
    /// action authority and no externally scheduled experiment steps.
    pub fn enable_constructing_active_causal_probes(&mut self)->UResult<()>{
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller {return Err(UError::Blocked)}
        u.current.learner.enable_active_pair_probes()
    }
    /// Activates a same-organism, automatically triggered meta-synthesis lane.
    pub fn enable_constructing_trajectory_self_revision(&mut self)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller||u.self_revision.is_some(){return Err(UError::Busy)}
        u.self_revision=Some(MetaRevision::default());Ok(())
    }
    pub fn trajectory_self_revision_report(&self)->Option<MetaRevisionReport>{
        Some(self.constructing.as_ref()?.self_revision.as_ref()?.report())
    }
    /// Behavior-first self-revision requires the gate to be enabled BEFORE
    /// collecting the first goal episode. All older opt-in paths retain their
    /// historical prediction-only research semantics.
    pub fn enable_constructing_behavioral_gate(&mut self)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller{return Err(UError::Blocked)}
        let r=u.self_revision.as_mut().ok_or(UError::NoEvidence)?;
        if r.behavior_guard_enabled||r.episodes!=0||r.active.is_some()||
            r.shadow.is_some()||r.behavior_attempted{return Err(UError::Busy)}
        r.behavior_guard_enabled=true;
        Ok(())
    }
    /// Execute a one-use prospectively defined batch of VIRTUAL trial worlds.
    ///
    /// The caller owns independent world instances indexed by (case, arm),
    /// initialized to the checkpoint's current *observed* state. It provides
    /// fresh HAZARD evidence and the ACTUAL POST after each selected action.
    /// The runtime chooses ALL actions; cohort tags are never inputs to U1.
    /// Each arm has a distinct, frozen, restored protected authority.
    ///
    /// The candidate evaluator cannot enter ordinary U1 arbitration until
    /// the bounded behavior gate passes. Even a callback error consumes the
    /// trial attempt, preventing free peeking/repetition.
    pub fn qualify_constructing_behavioral_gate<H,F>(
        &mut self,tasks:&[MetaBehaviorTask],mut hazard:H,mut execute:F
    )->UResult<MetaBehaviorAudit>
    where H:FnMut(usize,usize,usize)->HumanProtectionEvidence,
          F:FnMut(usize,usize,usize)->UResult<UnifiedPost> {
        {
            let u=self.constructing.as_ref().ok_or(UError::NoEvidence)?;
            let r=u.self_revision.as_ref().ok_or(UError::NoEvidence)?;
            let cfg=u.current.learner.carrier_config();
            if u.freeze_controller || !r.behavior_guard_enabled ||
                r.behavior_attempted || r.active.is_none() || r.shadow.is_some() ||
                !r.pending_steps.is_empty() || r.readout_ablated {
                return Err(UError::Busy)
            }
            if cfg.width>6 || cfg.actions>8 || cfg.planning_work>100_000 ||
                !(8..=64).contains(&tasks.len()){
                return Err(UError::Capacity)
            }
            let mut cohorts=std::collections::BTreeMap::<u16,usize>::new();
            let current=u.current.learner.current();
            let mut worst_case=0_u64;
            for t in tasks {
                if t.goal.len()!=current.len() || !t.goal.iter().any(Option::is_some)
                    || udistance(current,Some(&t.goal))==0.0 ||
                    t.offers.len()!=cfg.actions ||
                    !(1..=24).contains(&t.max_actions) ||
                    !t.offers.iter().any(|o|o.available) {
                    return Err(UError::InvalidData)
                }
                *cohorts.entry(t.cohort).or_default()+=1;
                let max=t.offers.iter().filter(|o|o.available)
                    .map(|o|o.max_cost).max().unwrap_or(0);
                worst_case=worst_case.checked_add(max.checked_mul(
                    2*t.max_actions as u64).ok_or(UError::Capacity)?)
                    .ok_or(UError::Capacity)?;
            }
            if cohorts.len()<2 || cohorts.values().any(|&v|v<4) {
                return Err(UError::InvalidData)
            }
            if worst_case>u.max_external_cost.saturating_sub(u.total_external_cost){
                return Err(UError::Budget)
            }
        }
        // Snapshot both arms BEFORE any trials, but record the one-use
        // attempt on the live organism BEFORE executing a single callback.
        let bytes=self.checkpoint_constructing()?;
        let initial=self.constructing.as_ref().ok_or(UError::NoEvidence)?
            .current.learner.current().to_vec();
        self.constructing.as_mut().ok_or(UError::NoEvidence)?
            .self_revision.as_mut().ok_or(UError::NoEvidence)?
            .behavior_attempted=true;
        let mut charged_external=0_u64;
        let mut charged_work=0_u64;
        let attempted=(||->UResult<MetaBehaviorAudit>{
            let mut cases=Vec::new();
            for (index,task) in tasks.iter().enumerate() {
                let mut arms=Vec::new();
                for arm in 0..2 {
                    let mut runner=Self::from_constructing_checkpoint(&bytes)?;
                    runner.freeze_constructing()?;
                    if arm==0{
                        runner.ablate_frozen_trajectory_evaluator()?;
                    }else{
                        // An ephemeral canary may test the candidate, but
                        // cannot serialize this authority into a checkpoint.
                        runner.constructing.as_mut().ok_or(UError::NoEvidence)?
                            .self_revision.as_mut().ok_or(UError::NoEvidence)?
                            .sandbox_candidate_arm=true;
                    }
                    let mut before_work=runner.unified_report()
                        .ok_or(UError::NoEvidence)?.computation_work;
                    let mut steps=Vec::new();
                    for _ in 0..task.max_actions {
                        let mut factual_post=None::<UnifiedPost>;
                        // Capture work *even when this individual protected
                        // step fails*: '?' otherwise exits before the final
                        // arm accounting. No failed sandbox gets free compute.
                        let attempted_step=runner.step_constructing(
                            Some(&task.goal),&task.offers,
                            |action|hazard(index,arm,action),
                            |action|{
                                let p=execute(index,arm,action)?;
                                charged_external=charged_external.saturating_add(p.cost);
                                factual_post=Some(p.clone());
                                Ok(p)
                            }
                        );
                        let after_work=runner.unified_report()
                            .ok_or(UError::NoEvidence)?.computation_work;
                        charged_work=charged_work.saturating_add(
                            after_work.saturating_sub(before_work));
                        before_work=after_work;
                        let outcome=attempted_step?;
                        let post=factual_post.ok_or(UError::InvalidData)?;
                        steps.push(MetaBehaviorStep {
                            action:outcome.action,post:post.observation,
                            cost:post.cost,
                            selected_goal_score:outcome.selected_fields[0],
                        });
                        if udistance(&steps.last().ok_or(UError::InvalidData)?.post,
                            Some(&task.goal))==0.0{break}
                    }
                    arms.push(MetaBehaviorArm{steps});
                }
                let arms:[MetaBehaviorArm;2]=arms.try_into().map_err(|_|UError::InvalidData)?;
                cases.push(MetaBehaviorCase{task:task.clone(),arms});
            }
            let mut result=MetaBehaviorAudit{
                checkpoint_fingerprint:uhash(&bytes),
                initial_observation:initial,
                cases,
                summary:MetaBehaviorSummary{
                    cases:0,incumbent_success:0,candidate_success:0,
                    incumbent_actions:0,candidate_actions:0,
                    incumbent_cost:0,candidate_cost:0,
                    changed_initial_actions:0,changed_initial_scores:0,
                    nonregressed_cohorts:false,qualified:false,
                },
            };
            result.summary=result.compute_summary()?;
            result.validate()?;
            Ok(result)
        })();
        // The primary organism pays the actually performed sandbox action
        // costs regardless of success. Clones never hand back an actuator
        // permit and cannot rewrite the primary cognition or its SNN weights.
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        u.total_external_cost=u.total_external_cost.saturating_add(charged_external);
        u.computation_work=u.computation_work.saturating_add(charged_work);
        let result=attempted?;
        u.self_revision.as_mut().ok_or(UError::NoEvidence)?
            .behavioral_audit=Some(result.clone());
        Ok(result)
    }
    /// Explicit research ablation: only switch off the VM scorer in a frozen
    /// replay. The certified program, journal, SNN weights, actions and
    /// HumanProtection remain unchanged. No way to use this to train.
    pub fn ablate_frozen_trajectory_evaluator(&mut self)->UResult<()>{
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if !u.freeze_controller{return Err(UError::Busy)}
        let revision=u.self_revision.as_mut().ok_or(UError::NoEvidence)?;
        if revision.active.is_none()||revision.readout_ablated{return Err(UError::NoEvidence)}
        revision.readout_ablated=true;
        Ok(())
    }
    pub fn unified_report(&self)->Option<UnifiedReport>{
        let u=self.constructing.as_ref()?;
        Some(UnifiedReport{domain:u.current.name.clone(),archived_domains:u.archived.iter().map(|d|d.name.clone()).collect(),learner:u.current.learner.report(),
            native_meta_observations:self.organism.phase_native_meta_observations(),native_weights:self.organism.phase_native_meta_weights()?,operations:u.operations,
            computation_work:u.computation_work,total_external_cost:u.total_external_cost,controller_frozen:u.freeze_controller,
            self_revision:u.self_revision.as_ref().map(MetaRevision::report),
            agency_credit:u.agency_credit.clone(),
            instrumental_credit:u.instrumental_credit.as_ref().map(InstrumentalCredit::report),
            temporal_options:u.temporal_options.as_ref().map(TemporalOptionLane::report)})
    }
    // A new sensor contract does not reinterpret numeric variables as pixels.
    // U1 weights and complete old-domain cognitive checkpoints remain present.
    pub fn enter_constructing_domain(&mut self,name:&str,config:UConfig,mut policy:UPolicy,initial:Vec<i64>)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.archived.len()>=7 || u.freeze_controller || name.is_empty() || name.len()>128 || name==u.current.name
            || u.archived.iter().any(|d|d.name==name) || config.width!=self.organism.config().sensory_cells
            || config.actions!=self.organism.config().motor_cells {return Err(UError::InvalidData);}
        let k=u.archived.len()+2;
        if let Some(e)=&mut policy.evidence{e.total_alpha=e.total_alpha.min(0.01/(k*(k+1)) as f64);}
        policy.max_external_cost=policy.max_external_cost.min(u.max_external_cost.saturating_sub(u.total_external_cost));
        let mut learner=ULearner::new(config,initial.clone())?;learner.enable_autonomy(policy)?;learner.manage_from_carrier()?;
        let old=std::mem::replace(&mut u.current,UnifiedDomain{name:name.into(),learner,last_construction:0,pending_credit:None});
        u.archived.push(old);
        if let Some(c)=u.instrumental_credit.as_mut(){c.pending=None;}
        // A new Learner has NONE of the history opt-ins required by
        // TemporalOptionLane::validate. Keeping the old lane makes the new
        // joint checkpoint unloadable. Revoke any pending macro and remove
        // its old-domain-only option configuration before the transition.
        if let Some(opt)=u.temporal_options.as_mut(){opt.reject_pending();}
        u.temporal_options=None;
        if let Some(revision)=u.self_revision.as_mut(){revision.pending_steps.clear();revision.pending_goal=None;}self.organism.observe_initial_real(&usensory(&initial),false);self.record(LifetimeEventKind::ExternalObservation);Ok(())
    }
    /// The WORLD, not U1, initiated a new episode. A trusted sensor
    /// supplies its ACTUAL new observation. Existing causally learned facts,
    /// native policy synapses, action costs and HumanProtection audit remain.
    /// No actuator is invoked and no POST journal record is fabricated.
    pub fn observe_constructing_external_restart(
        &mut self,observed:Vec<i64>
    )->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        u.current.learner.carrier_observe_external_restart(observed.clone())?;
        u.current.pending_credit=None;
        if let Some(revision)=u.self_revision.as_mut(){
            revision.pending_steps.clear();
            revision.pending_goal=None;
        }
        if let Some(c)=u.instrumental_credit.as_mut(){
            c.pending=None;
        }
        if let Some(option)=u.temporal_options.as_mut(){option.reject_pending();}
        self.organism.observe_initial_real(&usensory(&observed),false);
        self.fresh_observation_required=false;
        self.record(LifetimeEventKind::ExternalObservation);
        Ok(())
    }
    pub fn freeze_constructing(&mut self)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        u.current.learner.freeze_learning()?;
        if let Some(c)=u.instrumental_credit.as_mut(){c.pending=None;}
        if let Some(s)=&mut u.education{s.freeze();}u.freeze_controller=true;self.organism.set_phase_native_meta_learning_enabled(false);Ok(())
    }
    // Matched ablation: same experience, programs and budgets; only U1 zeroed.
    pub fn ablate_constructing_controller(&mut self)->UResult<()> {
        let u=self.constructing.as_ref().ok_or(UError::NoEvidence)?;if !u.freeze_controller {return Err(UError::Busy);}
        let mut data:serde_json::Value=serde_json::from_slice(&self.organism.constructing_carrier_checkpoint().ok_or(UError::InvalidData)?).map_err(|_|UError::InvalidData)?;
        for s in data["synapses"].as_array_mut().ok_or(UError::InvalidData)? {s["weight"]=serde_json::json!(0.0);}
        self.organism=EvoPhase::from_constructing_carrier_checkpoint(&serde_json::to_vec(&data).map_err(|_|UError::InvalidData)?).ok_or(UError::InvalidData)?;Ok(())
    }
    pub fn step_constructing<H,F>(&mut self,goal:Option<&[Option<i64>]>,offers:&[UOffer],hazard:H,execute:F)->UResult<UnifiedStep>
    where H:FnOnce(usize)->HumanProtectionEvidence,F:FnOnce(usize)->UResult<UnifiedPost> {
        if self.protection.emergency_stop_latched(){return Err(UError::Blocked);}
        let mut u=self.constructing.take().ok_or(UError::NoEvidence)?;
        let result=self.step_constructing_inner(&mut u,goal,offers,hazard,execute);self.constructing=Some(u);result
    }
    fn step_constructing_inner<H,F>(&mut self,u:&mut UnifiedConstruction,goal:Option<&[Option<i64>]>,offers:&[UOffer],hazard:H,execute:F)->UResult<UnifiedStep>
    where H:FnOnce(usize)->HumanProtectionEvidence,F:FnOnce(usize)->UResult<UnifiedPost> {
        let cfg=u.current.learner.carrier_config().clone();
        if offers.len()!=cfg.actions || goal.is_some_and(|g|g.len()!=cfg.width){return Err(UError::InvalidData);}
        // Once all *specified* goal coordinates already match the last
        // factual observation, do not issue an unneeded protected action.
        // Masked goals may intentionally leave unrelated state unconstrained.
        if u.current.learner.self_directed_control_enabled()
            && goal.is_some_and(|g|g.iter().any(Option::is_some)
                && udistance(u.current.learner.current(),Some(g))==0.0)
        {return Err(UError::NoEvidence)}
        let remaining=u.max_external_cost.saturating_sub(u.total_external_cost);
        let allowed:Vec<_>=offers.iter().map(|o|o.available&&o.max_cost<=remaining).collect();
        if !allowed.iter().any(|&a|a){
            // A previously selected macro cannot retain actuator authority
            // across a state in which no motor is currently permitted.
            if let Some(lane)=u.temporal_options.as_mut(){lane.reject_pending()}
            return Err(UError::Blocked)
        }
        // This is the organism's observable opportunity to CHOOSE,
        // not the actor's goal, motor role, reward, or proposal source.
        let policy_has_choice=allowed.iter().filter(|&&v|v).count()>=2;
        // A previous native U1 decision may have selected an entire learned
        // option. Its next primitive action is not a NEW U1 choice. Only
        // actual, matching protected POSTs allow continuation.
        let mut continuation=None::<UnifiedOption>;
        if let Some(p)=u.temporal_options.as_ref()
            .and_then(|lane|lane.pending.as_ref()).cloned(){
            let same=goal.is_some_and(|g|g==p.goal.as_slice()) &&
                p.last_verified_post==u.current.learner.current() &&
                p.event==u.current.learner.records().len();
            let next=p.remaining.first();
            if same && next.is_some_and(|step|
                step.action<allowed.len() && allowed[step.action]){
                let next=next.ok_or(UError::InvalidData)?;
                continuation=Some(UnifiedOption{
                    p:PhaseCognitiveProposal{
                        proposal_id:0x800+next.action as u64,
                        action:next.action,fields:p.initiating_fields
                    },
                    kind:3,source:"temporal-option-continuation",
                    output:Some(next.expected_post.clone())
                });
            }else{
                u.temporal_options.as_mut()
                    .ok_or(UError::NoEvidence)?.reject_pending();
                if same{return Err(UError::Blocked)}
            }
        }
        let is_option_continuation=continuation.is_some();
        let native_credit_eligible=(u.agency_credit.is_none() ||
            policy_has_choice) && !is_option_continuation;
        let (predictions,overhead)=u.current.learner.carrier_predictions(&allowed)?;u.computation_work=u.computation_work.saturating_add(overhead);
        let pressure=u.current.learner.carrier_pressure();
        let mut options:Vec<UnifiedOption>=predictions.iter().map(|p|UnifiedOption{p:PhaseCognitiveProposal{proposal_id:0x100+p.action as u64,action:p.action,
            fields:ufields(&p.output,goal,p.novelty,p.disagreement,p.confidence,1)},kind:3,source:"predict",output:Some(p.output.clone())}).collect();
        // The existing Learner already knows how to move from unexplained
        // consequences toward information-gathering, then exploit verified
        // transitions. Previously it was reachable only in the separate
        // ProtectedConstructionRuntime. Here it is a *proposal*, never a
        // bypass around U1 or the human-protection permit.
        let mut native_route=None::<Vec<(usize,Vec<i64>)>>;
        let mut autobiographical_route=None::<Vec<(usize,Vec<i64>)>>;
        let mut observable_graph_route=None::<Vec<(usize,Vec<i64>)>>;
        let mut complexity_need_decision=None::<String>;
        let mut contextual_probe_source=false;
        let mut base_factual_work=0_u64;
        if u.current.learner.self_directed_control_enabled() &&
            !is_option_continuation {
            // Experimental multi-step search is enabled only when the caller
            // explicitly allocates a larger bounded work budget. The published
            // 4k/20k development runs retain their EXACT previous behavior.
            // All chain edges must be validated independently against actual
            // chronological holdouts, unlike exploratory imagined plans.
            // One shared finite factual-computation budget. The new
            // experiment remains opt-in; existing 4k/20k runs are identical.
            let variable_depth=u.temporal_options.as_ref()
                .is_some_and(|o|o.variable_depth_enabled);
            let necessity_gate=u.temporal_options.as_ref()
                .is_some_and(|o|o.complexity_need_gate_enabled);
            let memory_budget=if goal.is_some() && cfg.planning_work>=32_000 &&
                u.temporal_options.as_ref().is_some_and(|o|o.autobiographical_enabled){
                if variable_depth{20_000_u64}else{5_000_u64}
            }else{0};
            // The graph cannot reserve 16k ON TOP of a 20k memory allowance
            // inside the minimum supported 32k total. Reserve the largest
            // jointly affordable ceiling, leaving 4k for native U1.
            let graph_reserve=if goal.is_some() && cfg.planning_work>=32_000 &&
                u.temporal_options.as_ref().is_some_and(|o|o.observable_graph_enabled){
                observable_graph_budget_ceiling(cfg.planning_work,memory_budget)
            }else{0};
            let history_budget=if goal.is_some() &&
                u.current.learner.predictive_history_states_enabled() &&
                cfg.planning_work>=32_000 {
                cfg.planning_work.saturating_sub(4_000)
                    .saturating_sub(memory_budget).saturating_sub(graph_reserve)
            } else {0};
            let (history,history_work)=if let Some(g)=goal {
                if u.current.learner.adaptive_history_suffix_enabled(){
                    u.current.learner.carrier_adaptive_history_plan_metered(
                        g,&allowed,history_budget)
                } else {
                    u.current.learner.carrier_predictive_history_plan_metered(
                        g,&allowed,history_budget)
                }
            } else {(None,0)};
            let (autobiographical,memory_work)=if let Some(g)=goal{
                if necessity_gate {
                    // Complexity is NOT a goal. Retain the simplest
                    // non-contradicted learned explanatory route first.
                    // Physical failure of that exact shorter route is
                    // written in the SAME journal and withdraws it.
                    let (incumbent,cheap_work)=u.current.learner
                        .carrier_autobiographical_option_metered(
                            g,&allowed,memory_budget.min(5_000));
                    if incumbent.is_some(){
                        complexity_need_decision=Some("retain_simpler".into());
                        (incumbent,cheap_work)
                    }else {
                        let (extended,deep_work)=u.current.learner
                            .carrier_variable_depth_option_metered(
                                g,&allowed,
                                memory_budget.saturating_sub(cheap_work),true);
                        complexity_need_decision=Some(if extended.as_ref()
                            .is_some_and(|p|p.depth>3) {
                                "extend_after_contradiction"
                            }else if extended.is_some(){
                                "retain_low_complexity"
                            }else{
                                "investigate_insufficient_evidence"
                            }.into());
                        (extended,cheap_work.saturating_add(deep_work))
                    }
                }else if variable_depth{
                    u.current.learner.carrier_variable_depth_option_metered(
                        g,&allowed,memory_budget,false)
                }else{
                    u.current.learner.carrier_autobiographical_option_metered(
                        g,&allowed,memory_budget)
                }
            }else{(None,0)};
            // Spend from ACTUAL remaining metered work, never an independent
            // allocation; history and autobiography have already charged.
            let graph_budget=graph_reserve.min(cfg.planning_work
                .saturating_sub(history_work).saturating_sub(memory_work)
                .saturating_sub(4_000));
            let (observable_graph,graph_work)=if let Some(g)=goal{
                u.current.learner.carrier_observable_goal_route_metered(
                    g,&allowed,graph_budget)
            }else{(None,0)};
            let probe_budget=if u.current.learner.active_pair_probes_enabled()
                && !u.freeze_controller && cfg.planning_work>=32_000 {
                cfg.planning_work.saturating_sub(history_work)
                    .saturating_sub(memory_work).saturating_sub(graph_work)
                    .saturating_sub(4_000).min(6_000)
            }else{0};
            let allow_contextual=u.temporal_options.as_ref()
                .is_some_and(|l|l.contextual_experiments_enabled) &&
                complexity_need_decision.as_deref()==
                    Some("investigate_insufficient_evidence");
            let (probe,probe_work)=if allow_contextual{
                let (plan,work)=u.current.learner
                    .carrier_contextual_experiment_metered(
                        &allowed,offers,probe_budget);
                if plan.is_some(){
                    contextual_probe_source=true;
                    (plan,work)
                }else{
                    let (fallback,extra)=u.current.learner
                        .carrier_active_causal_probe_metered(&allowed,offers,
                            probe_budget.saturating_sub(work));
                    (fallback,work.saturating_add(extra))
                }
            }else{
                u.current.learner.carrier_active_causal_probe_metered(
                    &allowed,offers,probe_budget)
            };
            let chain_budget=if goal.is_some() && u.current.learner.certified_chains_enabled() && cfg.planning_work>=32_000 {
                cfg.planning_work.saturating_sub(history_work).saturating_sub(memory_work).saturating_sub(graph_work).saturating_sub(probe_work).saturating_sub(4_000)
            } else {0};
            let (chain,chain_work)=if let Some(g)=goal {
                u.current.learner.carrier_certified_goal_chain_metered(g,&allowed,chain_budget)
            } else {(None,0)};
            let residual=cfg.planning_work.saturating_sub(history_work)
                .saturating_sub(memory_work).saturating_sub(graph_work)
                .saturating_sub(probe_work).saturating_sub(chain_work);
            let (action,chosen_work)=u.current.learner
                .choose_offered_metered_budget(goal,Some(offers),residual)?;
            if !allowed[action] {return Err(UError::Blocked)}
            let evidence=predictions.iter().find(|p|p.action==action).ok_or(UError::InvalidData)?;
            // Share exactly ONE configured factual work budget between the
            // validated chain, legacy probe selection and direct verification.
            let remaining=residual.saturating_sub(chosen_work);
            let (factual_prediction,factual_work)=u.current.learner
                .carrier_factual_action_consequence_metered(action,remaining);
            let actual_metered_work=history_work.saturating_add(memory_work)
                .saturating_add(graph_work).saturating_add(probe_work)
                .saturating_add(chain_work).saturating_add(chosen_work)
                .saturating_add(factual_work);
            // Reject an over-budget planner BEFORE preparing or permitting
            // a physical motor. Future search extensions must share this
            // invariant instead of claiming separate implicit allowances.
            if actual_metered_work>cfg.planning_work{return Err(UError::Budget)}
            base_factual_work=actual_metered_work;
            u.computation_work=u.computation_work.checked_add(actual_metered_work)
                .ok_or(UError::Capacity)?;
            if let Some(probe)=probe{
                if !allowed[probe.action] {return Err(UError::Blocked)}
                // Unknown predicted goal result: zero, not an optimistic
                // progress label. Relevance is physical contextual novelty.
                let fields=[0.0,probe.novelty,probe.disagreement,0.0,
                    1.0/(1.0+(probe.quoted_cost as f32).ln_1p())];
                options.push(UnifiedOption{p:PhaseCognitiveProposal{
                    proposal_id:0x700+probe.action as u64,
                    action:probe.action,fields
                },kind:3,source:if contextual_probe_source{
                    "contextual-causal-experiment"
                }else{
                    "causal-pair-probe"
                },output:None});
            }
            if let Some(memory)=autobiographical{
                // Quote EVERY physical motor, including repeats. The first
                // primitive may be cheap while later ones are costly.
                // Do not offer a multi-action plan if its whole quoted
                // cost exceeds the remaining actual external budget.
                let route_quote=memory.route.iter().try_fold(0_u64,
                    |sum,(action,_)|offers.get(*action)
                        .and_then(|offer|sum.checked_add(offer.max_cost)));
                if let Some(quoted_cost)=route_quote.filter(|&total|
                    total<=u.max_external_cost.saturating_sub(u.total_external_cost)){
                    // The physical success and the contrasting last-motor
                    // failure came from UNSCHEDULED, executed, protected POSTs.
                    // Candidate is tentative until a new real replay succeeds.
                    let observed=predictions.iter().find(|p|
                        p.action==memory.first_action).ok_or(UError::InvalidData)?;
                    let mut fields=ufields(&memory.terminal,goal,observed.novelty,
                        observed.disagreement,0.6,quoted_cost);
                    fields[0]/=1.0+0.10*(memory.depth.saturating_sub(1) as f32);
                    autobiographical_route=Some(memory.route.clone());
                    options.push(UnifiedOption{p:PhaseCognitiveProposal{
                        proposal_id:0x900+memory.first_action as u64,
                        action:memory.first_action,fields},
                        kind:3,source:"autobiographical-option",
                        output:Some(memory.first_consequence)});
                }
            }
            if let Some(plan)=observable_graph {
                // Entire tentative candidate can compete with U1 ONLY if
                // the physically quoted TOTAL route is affordable.
                let cost=plan.route.iter().try_fold(0_u64,|total,(motor,_)|
                    offers.get(*motor)
                        .and_then(|offer|total.checked_add(offer.max_cost)));
                if let Some(quote)=cost.filter(|&n|
                    n<=u.max_external_cost.saturating_sub(u.total_external_cost)){
                    let observed=predictions.iter()
                        .find(|p|p.action==plan.first_action)
                        .ok_or(UError::InvalidData)?;
                    let mut fields=ufields(&plan.terminal,goal,
                        observed.novelty,observed.disagreement,0.7,quote);
                    fields[0]/=1.0+0.10*(plan.depth.saturating_sub(1) as f32);
                    observable_graph_route=Some(plan.route.clone());
                    options.push(UnifiedOption{p:PhaseCognitiveProposal{
                        proposal_id:0xA00+plan.first_action as u64,
                        action:plan.first_action,fields
                    },kind:3,source:"observable-empirical-route",
                    output:Some(plan.first_consequence)});
                }
            }
            native_route=history.as_ref()
                .filter(|plan|plan.history_suffix_depth>=2 &&
                    u.temporal_options.is_some() && plan.route.len()>1)
                .map(|plan|plan.route.clone());
            if let Some(plan)=history {
                if !allowed[plan.first_action]{return Err(UError::Blocked)}
                let observed=predictions.iter().find(|p|p.action==plan.first_action)
                    .ok_or(UError::InvalidData)?;
                // The very FIRST action may leave sensors unchanged. The
                // value comes from an empirically validated future edge,
                // never a fabricated immediate POST.
                let mut fields=ufields(&plan.terminal,goal,observed.novelty,
                    observed.disagreement,0.9,offers[plan.first_action].max_cost
                        .saturating_mul(plan.depth as u64));
                fields[0]/=1.0+0.10*(plan.depth.saturating_sub(1) as f32);
                options.push(UnifiedOption{p:PhaseCognitiveProposal{
                    proposal_id:0x600+plan.first_action as u64,
                    action:plan.first_action,fields
                },kind:3,source:if plan.history_suffix_depth>=2{
                    "predictive-suffix"
                }else{"predictive-history"},
                    output:Some(plan.first_consequence)});
            }
            if let Some(plan)=chain {
                if !allowed[plan.first_action]{return Err(UError::Blocked)}
                let observed=predictions.iter().find(|p|p.action==plan.first_action)
                    .ok_or(UError::InvalidData)?;
                // Terminal goal value is a *discounted* predicted consequence
                // of multiple certified edges; only FIRST action is executed.
                let mut fields=ufields(&plan.endpoint,goal,observed.novelty,
                    observed.disagreement,1.0/(plan.depth as f32),
                    offers[plan.first_action].max_cost.saturating_mul(plan.depth as u64));
                fields[0]/=1.0+0.15*(plan.depth.saturating_sub(1) as f32);
                options.push(UnifiedOption{p:PhaseCognitiveProposal{
                    proposal_id:0x500+plan.first_action as u64,
                    action:plan.first_action,fields
                },kind:3,source:"factual-chain",output:Some(plan.first_consequence)});
            }
            // Only empirically calibrated consequences are allowed to compete.
            // An unknown prediction is not secretly a high-novelty U1 option.
            // Ordinary U1 handles observation and exploratory choice instead.
            if let Some(factual)=factual_prediction {
            // One common five-field contract: verified-goal proximity,
            // local novelty, epistemic disagreement, factual support,
            // and quoted action economy. No motor IDs or world role labels.
            let fields=ufields(&factual,goal,evidence.novelty,
                evidence.disagreement,0.8,offers[action].max_cost);
            options.push(UnifiedOption{p:PhaseCognitiveProposal{
                proposal_id:0x400+action as u64,action,fields
            },kind:3,source:"factual-experience",output:Some(factual)});
            }
            // No separate high-novelty proposal without prospective support.
            // Ordinary U1 already owns uncertainty-driven action selection:
            // an unverified factual hypothesis must ABSTAIN, not override it.
        }
        let certified_chain_proposed=options.iter().any(|o|o.source=="factual-chain");
        let predictive_history_plan_proposed=options.iter().any(|o|
            o.source=="predictive-history" || o.source=="predictive-suffix");
        let adaptive_suffix_plan_proposed=options.iter().any(|o|
            o.source=="predictive-suffix");
        let active_pair_probe_proposed=options.iter().any(|o|o.source=="causal-pair-probe");
        let contextual_experiment_proposed=options.iter()
            .any(|o|o.source=="contextual-causal-experiment");
        let observable_empirical_route_proposed=options.iter()
            .any(|o|o.source=="observable-empirical-route");
        if contextual_experiment_proposed {
            if let Some(l)=u.temporal_options.as_mut(){
                l.contextual_experiment_proposed=l.contextual_experiment_proposed
                    .checked_add(1).ok_or(UError::Capacity)?;
            }
        }
        let autobiographical_option_proposed=options.iter().any(|o|
            o.source=="autobiographical-option");
        let mut tried=[false;3];let mut helpers=Vec::new();let mut operations=Vec::new();let mut selected=None;
        let mut meta_helper_work=0_u64;
        for round in 0..4 {
            if let Some(approved)=continuation.take(){
                // Same native choice, bounded saved option, and no
                // second U1 primitive arbitration. Safety still screens.
                operations.push("persisted-native-option".into());
                selected=Some((approved,0.0));break
            }
            let mut candidates=options.clone();
            let can_construct=u.current.learner.carrier_can_construct() && u.current.learner.records().len().saturating_sub(u.current.last_construction)>=cfg.search_every;
            if round<3 {
                let action=allowed.iter().position(|&a|a).ok_or(UError::Blocked)?;
                if !tried[0] && can_construct {candidates.push(UnifiedOption{p:PhaseCognitiveProposal{proposal_id:1,action,fields:[0.0,pressure,pressure,0.0,0.15]},kind:0,source:"construct",output:None});}
                if !tried[1] && goal.is_some(){candidates.push(UnifiedOption{p:PhaseCognitiveProposal{proposal_id:2,action,fields:[0.5,0.5,pressure,0.5,0.4]},kind:1,source:"plan",output:None});}
                if !tried[2] && !u.current.learner.records().is_empty(){candidates.push(UnifiedOption{p:PhaseCognitiveProposal{proposal_id:3,action,fields:[0.0,0.5,pressure,0.5,0.7]},kind:2,source:"recall",output:None});}
            }
            // Only a prospectively admitted program can rewrite goal evidence.
            // The new VM expression has no actuator or safety authority.
            if let Some(revision)=&u.self_revision {
                for c in &mut candidates {if c.kind==3 {revision.adjust(&mut c.p.fields);}}
            }
            let raw:Vec<_>=candidates.iter().map(|x|x.p).collect();u.nonce=u.nonce.checked_add(1).ok_or(UError::Capacity)?;
            let choice=self.organism.choose_phase_meta_exploratory(&raw,uword(u.nonce),!u.freeze_controller).ok_or(UError::NoEvidence)?;
            let chosen=candidates.into_iter().find(|p|p.p.proposal_id==choice.proposal_id).ok_or(UError::InvalidData)?;
            u.operations[chosen.kind as usize]+=1;operations.push(chosen.source.to_string());
            if chosen.kind==3 {selected=Some((chosen,choice.score));break;}
            tried[chosen.kind as usize]=true;
            match chosen.kind {
                0=>{
                    u.current.last_construction=u.current.learner.records().len();let before=u.current.learner.report().construction_work;
                    let result=u.current.learner.construct();u.computation_work=u.computation_work.saturating_add(u.current.learner.report().construction_work.saturating_sub(before));
                    if result.is_ok() && u.current.learner.report().shadow_pending {
                        if native_credit_eligible && u.instrumental_credit.is_none() {
                            u.current.pending_credit=Some(chosen.p.fields);
                        }
                    }
                    else if !u.freeze_controller && native_credit_eligible &&
                        u.instrumental_credit.is_none() {
                        self.organism.observe_phase_native_meta_utility(chosen.p.fields,0.0);
                    }
                    if let Err(e)=result {if !matches!(e,UError::NoEvidence|UError::Budget|UError::Capacity){return Err(e);}}
                },
                1=>{
                    let remaining=cfg.planning_work.saturating_sub(base_factual_work)
                        .saturating_sub(meta_helper_work);
                    let (plans,work)=u.current.learner.carrier_plans(
                        goal.ok_or(UError::InvalidData)?,&allowed,remaining)?;
                    if work>remaining{return Err(UError::Budget)}
                    meta_helper_work=meta_helper_work.checked_add(work).ok_or(UError::Capacity)?;
                    u.computation_work=u.computation_work.checked_add(work).ok_or(UError::Capacity)?;
                    helpers.push(("plan",chosen.p.fields));
                    for (a,p,depth) in plans {
                        let confidence=predictions.iter().find(|x|x.action==a).map(|x|x.confidence).unwrap_or(0.0);
                        options.push(UnifiedOption{p:PhaseCognitiveProposal{proposal_id:0x200+a as u64,action:a,fields:ufields(&p,goal,0.0,pressure,confidence,depth as u64)},kind:3,source:"plan",output:Some(p)});
                    }
                },
                2=>{
                    let remaining=cfg.planning_work.saturating_sub(base_factual_work)
                        .saturating_sub(meta_helper_work);
                    let (memories,work)=u.current.learner.carrier_recall(
                        &allowed,remaining)?;
                    if work>remaining{return Err(UError::Budget)}
                    meta_helper_work=meta_helper_work.checked_add(work).ok_or(UError::Capacity)?;
                    u.computation_work=u.computation_work.checked_add(work).ok_or(UError::Capacity)?;
                    helpers.push(("recall",chosen.p.fields));
                    for (a,p,similarity) in memories {options.push(UnifiedOption{p:PhaseCognitiveProposal{proposal_id:0x300+a as u64,action:a,fields:ufields(&p,goal,0.0,pressure,similarity,1)},kind:3,source:"recall",output:Some(p)});}
                },_=>return Err(UError::InvalidData)
            }
        }
        // A helper runs after the initial meter; repeat the shared limit
        // check immediately BEFORE acquiring any physical motor permit.
        if base_factual_work.saturating_add(meta_helper_work)>cfg.planning_work{
            return Err(UError::Budget)
        }
        let (option,score)=selected.ok_or(UError::NoEvidence)?;let a=option.p.action;
        // A selected option is one native decision. No intrinsic/novelty
        // success may be assigned before its real terminal consequence.
        let selected_temporal_route=u.temporal_options.is_some() &&
            ((option.source=="predictive-suffix" && native_route.is_some()) ||
             (option.source=="autobiographical-option" &&
                autobiographical_route.is_some()) ||
             (option.source=="observable-empirical-route" &&
                observable_graph_route.is_some()));
        let mut delayed_option_credit=None::<([f32;5],f32)>;
        let original_fields=options.iter().find(|c|c.p.proposal_id==option.p.proposal_id)
            .map(|c|c.p.fields).unwrap_or(option.p.fields);
        let token=u.current.learner.carrier_prepare(a)?;
        let decision=self.protection.screen(a,hazard(a));let record=decision.record().clone();
        let Some(permit)=decision.into_permit() else {
            if is_option_continuation{
                if let Some(l)=u.temporal_options.as_mut(){l.reject_pending()}
            }
            if let Some(c)=u.instrumental_credit.as_mut(){c.pending=None;}
            u.current.learner.carrier_cancel(token)?;
            self.record(LifetimeEventKind::Blocked(record));
            return Err(UError::Blocked)
        };
        let permission=permit.sequence();
        if self.protection.consume_permit(permit).is_err(){
            if is_option_continuation{
                if let Some(l)=u.temporal_options.as_mut(){l.reject_pending()}
            }
            if let Some(c)=u.instrumental_credit.as_mut(){c.pending=None;}
            u.current.learner.carrier_fault(token,"permission fault".into());
            self.latch_fault("permission fault".into());
            return Err(UError::Blocked)
        }
        let post=match execute(a) {
            Ok(p)=>p,Err(e)=>{
                if is_option_continuation{
                    if let Some(l)=u.temporal_options.as_mut(){l.reject_pending()}
                }
                if let Some(c)=u.instrumental_credit.as_mut(){c.pending=None;}
                u.current.learner.carrier_fault(token,e.to_string());
                self.latch_fault(e.to_string());
                return Err(e)
            }
        };
        if post.observation.len()!=cfg.width || !post.reward.is_finite() || !(-1.0..=1.0).contains(&post.reward) || post.cost>offers[a].max_cost {
            if is_option_continuation{
                if let Some(l)=u.temporal_options.as_mut(){l.reject_pending()}
            }
            if let Some(c)=u.instrumental_credit.as_mut(){c.pending=None;}
            u.total_external_cost=u.total_external_cost.saturating_add(post.cost);u.current.learner.carrier_fault(token,"invalid external POST or cost".into());self.latch_fault("invalid external POST or cost".into());return Err(UError::InvalidData);
        }
        let before=udistance(u.current.learner.current(),goal);let after=udistance(&post.observation,goal);
        let novel=!u.current.learner.records().iter().any(|r|r.transition.post==post.observation);
        let progress=if goal.is_some(){((before-after)/(1.0+before)).max(0.0) as f32}else{0.0};
        let value=(0.8*post.reward.max(progress).max(0.0)+0.2*(if novel {1.0}else{0.0})).clamp(0.0,1.0);
        let verdicts=u.current.learner.promotions().len();u.total_external_cost=u.total_external_cost.saturating_add(post.cost);
        u.current.learner.carrier_commit(token,post.observation.clone(),permission,post.cost)?;
        if option.source=="contextual-causal-experiment" {
            if let Some(l)=u.temporal_options.as_mut(){
                l.contextual_experiment_selected=l.contextual_experiment_selected
                    .checked_add(1).ok_or(UError::Capacity)?;
            }
        }
        self.organism.observe_initial_real(&usensory(&post.observation),false);
        if is_option_continuation{
            let lane=u.temporal_options.as_mut().ok_or(UError::InvalidData)?;
            if option.output.as_ref()!=Some(&post.observation){
                // One executed contradiction is a real FAILED option,
                // not a fresh motor vote. External veto/restart is NOT failure.
                let prior=lane.pending.as_ref().ok_or(UError::InvalidData)?;
                if prior.credit_eligible && !u.freeze_controller{
                    delayed_option_credit=Some((prior.initiating_fields,0.0));
                    lane.contradictory_credits=lane.contradictory_credits
                        .checked_add(1).ok_or(UError::Capacity)?;
                }
                lane.reject_pending();
            }else{
                let pending=lane.pending.as_mut().ok_or(UError::InvalidData)?;
                pending.remaining.remove(0);
                pending.last_verified_post=post.observation.clone();
                pending.event=u.current.learner.records().len();
                pending.realized_cost=pending.realized_cost.checked_add(post.cost)
                    .ok_or(UError::Capacity)?;
                pending.executed_steps=pending.executed_steps.checked_add(1)
                    .ok_or(UError::Capacity)?;
                lane.continued=lane.continued.checked_add(1)
                    .ok_or(UError::Capacity)?;
                let route_finished=lane.pending.as_ref()
                    .is_some_and(|p|p.remaining.is_empty());
                if after==0.0{
                    // Only the observed terminal physical goal completes it.
                    let finished=lane.pending.take().ok_or(UError::InvalidData)?;
                    lane.completed=lane.completed.checked_add(1)
                        .ok_or(UError::Capacity)?;
                    lane.completed_real_cost=lane.completed_real_cost
                        .checked_add(finished.realized_cost).ok_or(UError::Capacity)?;
                    if (2..=5).contains(&finished.learned_depth){
                        let depth=finished.learned_depth;
                        lane.learned_depth_completed[depth]=lane.learned_depth_completed[depth]
                            .checked_add(1).ok_or(UError::Capacity)?;
                    }
                    if finished.credit_eligible && !u.freeze_controller{
                        delayed_option_credit=Some((finished.initiating_fields,
                            temporal_realized_goal_utility(finished.realized_cost)));
                        lane.terminal_credits=lane.terminal_credits.checked_add(1)
                            .ok_or(UError::Capacity)?;
                    }
                }else if route_finished{
                    // Matching an imagined partial route never forges success.
                    let prior=lane.pending.as_ref().ok_or(UError::InvalidData)?;
                    if prior.credit_eligible && !u.freeze_controller{
                        delayed_option_credit=Some((prior.initiating_fields,0.0));
                        lane.contradictory_credits=lane.contradictory_credits
                            .checked_add(1).ok_or(UError::Capacity)?;
                    }
                    lane.reject_pending();
                }
            }
        }else if selected_temporal_route{
            // Both prospectively inferred causal chains and firsthand
            // autobiographical hypotheses are chosen BY the same native U1.
            let selected_memory=option.source=="autobiographical-option";
            let selected_graph=option.source=="observable-empirical-route";
            let route=if selected_memory{autobiographical_route}
                else if selected_graph{observable_graph_route}
                else{native_route};
            if let (Some(lane),Some(route),Some(g))=
                (u.temporal_options.as_mut(),route,goal){
                // Only a genuine native-selected FIRST action and a
                // matching factual POST may activate an option.
                if route.len()>1 && route[0].0==a &&
                    route[0].1==post.observation && after!=0.0 &&
                    route.last().is_some_and(|x|
                        udistance(&x.1,Some(g))==0.0){
                    let learned_depth=if selected_memory{route.len()}else{0};
                    let remaining=route.into_iter().skip(1)
                        .map(|(action,expected_post)|
                            TemporalOptionStep{action,expected_post})
                        .collect::<Vec<_>>();
                    if !remaining.is_empty() &&
                        remaining.len()<cfg.planning_depth{
                        lane.started=lane.started.checked_add(1)
                            .ok_or(UError::Capacity)?;
                        if selected_memory{
                            lane.autobiographical_started=lane.autobiographical_started
                                .checked_add(1).ok_or(UError::Capacity)?;
                            if !(2..=5).contains(&learned_depth){
                                return Err(UError::InvalidData)
                            }
                            lane.learned_depth_selected[learned_depth]=
                                lane.learned_depth_selected[learned_depth]
                                    .checked_add(1).ok_or(UError::Capacity)?;
                        }
                        lane.pending=Some(TemporalOption{
                            goal:g.to_vec(),remaining,
                            last_verified_post:post.observation.clone(),
                            event:u.current.learner.records().len(),
                            initiating_fields:option.p.fields,
                            credit_eligible:policy_has_choice && !u.freeze_controller,
                            realized_cost:post.cost,executed_steps:1,
                            learned_depth
                        });
                    }
                }
            }
            if selected_temporal_route &&
                u.temporal_options.as_ref().is_some_and(|l|l.pending.is_none()) &&
                policy_has_choice && !u.freeze_controller{
                // A failed initial prediction is known only AFTER real POST.
                // A lucky immediate real goal is still genuine, not imagined.
                delayed_option_credit=Some((option.p.fields,
                    if after==0.0{temporal_realized_goal_utility(post.cost)}
                    else{0.0}));
            }
        }
        if !u.freeze_controller {
            if let Some(ledger)=u.agency_credit.as_mut() {
                if is_option_continuation{
                    ledger.option_continuation_actions=ledger.option_continuation_actions
                        .checked_add(1).ok_or(UError::Capacity)?;
                }else if policy_has_choice {
                    ledger.self_selected_actions=ledger.self_selected_actions
                        .checked_add(1).ok_or(UError::Capacity)?;
                } else {
                    ledger.externally_forced_actions=ledger.externally_forced_actions
                        .checked_add(1).ok_or(UError::Capacity)?;
                }
            }
            if native_credit_eligible && u.instrumental_credit.is_none() {
                if !selected_temporal_route {
                    self.organism.observe_phase_native_meta_utility(option.p.fields,value);
                    for (source,fields) in helpers {
                        self.organism.observe_phase_native_meta_utility(
                            fields,if source==option.source{value}else{0.0});
                    }
                }
                if u.current.learner.promotions().len()>verdicts {
                    if let Some(fields)=u.current.pending_credit.take(){
                        let accepted=u.current.learner.promotions().last().is_some_and(|p|p.accepted);
                        self.organism.observe_phase_native_meta_utility(fields,if accepted{1.0}else{0.0});
                    }
                }
            }else if u.current.learner.promotions().len()>verdicts{
                // An unqualified operation cannot become an invented U1
                // motor-choice success even if a separate model improved.
                u.current.pending_credit=None;
            }
            // Credited once to the ORIGINAL U1 option decision, after a
            // genuine physical terminal success or executed contradiction.
            // Continuation steps cannot independently receive this credit.
            if let Some((fields,utility))=delayed_option_credit{
                self.organism.observe_phase_native_meta_utility(fields,utility);
            }
            if let Some(c)=u.instrumental_credit.as_mut(){
                let event=u.current.learner.records().len();
                let mut valid_pending=None;
                if let Some(prior)=c.pending.take(){
                    if goal.is_some_and(|g|g==prior.goal.as_slice()) &&
                        prior.expected_post==u.current.learner.records()
                            .last().ok_or(UError::InvalidData)?.transition.pre &&
                        prior.event.checked_add(1)==Some(event){
                        valid_pending=Some(prior);
                    }
                }
                // Only the actually observed reduction in distance to the
                // current goal is native instrumental utility. New post
                // states alone are factual evidence, not goal success.
                if native_credit_eligible && goal.is_some() && after<before{
                    c.actual_progress_credited=c.actual_progress_credited
                        .checked_add(1).ok_or(UError::Capacity)?;
                    if let Some(prior)=valid_pending{
                        let half=0.5*progress;
                        self.organism.observe_phase_native_meta_utility(
                            prior.fields,half);
                        self.organism.observe_phase_native_meta_utility(
                            option.p.fields,half);
                        c.validated_preparations_credited=
                            c.validated_preparations_credited
                                .checked_add(1).ok_or(UError::Capacity)?;
                    }else{
                        self.organism.observe_phase_native_meta_utility(
                            option.p.fields,progress);
                    }
                }else{
                    c.facts_without_progress=c.facts_without_progress
                        .checked_add(1).ok_or(UError::Capacity)?;
                }
                if native_credit_eligible && goal.is_some() && !(after<before)
                    && option.source=="predictive-history" &&
                    option.output.as_ref()==Some(&post.observation) {
                    // Proposed hidden-state preparation was actually
                    // executed and the predicted immediate POST was observed.
                    // It is NOT credited until a later REAL goal advance.
                    c.pending=Some(InstrumentalPending{
                        goal:goal.ok_or(UError::InvalidData)?.to_vec(),
                        expected_post:post.observation.clone(),
                        fields:option.p.fields,event,
                    });
                }
            }
        }
        if !u.freeze_controller{
            if let Some(revision)=u.self_revision.as_mut(){
                if let Some(g)=goal{
                    // Exclusively an already committed, authorized REAL POST.
                    let actual=u.current.learner.records().last().ok_or(UError::InvalidData)?;
                    let meta_work=revision.observe(original_fields,g,
                        &actual.transition.pre,&actual.transition.post);
                    u.computation_work=u.computation_work.checked_add(meta_work).ok_or(UError::Capacity)?;
                }else{
                    // A goal-less real step breaks the causal credit episode.
                    revision.pending_steps.clear();revision.pending_goal=None;
                }
            }
        }
        let _=option.output;
        self.record(LifetimeEventKind::Executed{action:a,suppressed:0,learned:!u.freeze_controller});
        Ok(UnifiedStep{action:a,source:option.source.into(),operations,selected_fields:option.p.fields,native_score:score,factual_utility:value,event:u.current.learner.records().len(),computation_work:u.computation_work,certified_chain_proposed,predictive_history_plan_proposed,active_pair_probe_proposed,contextual_experiment_proposed,observable_empirical_route_proposed,adaptive_suffix_plan_proposed,autobiographical_option_proposed,complexity_need_decision})
    }
    pub fn checkpoint_constructing(&self)->UResult<Vec<u8>> {
        let u=self.constructing.as_ref().ok_or(UError::NoEvidence)?;u.current.learner.checkpoint()?;
        let carrier=self.organism.constructing_carrier_checkpoint().ok_or(UError::InvalidData)?;
        let hash=uhash(&serde_json::to_vec(&(1u32,&carrier,u,self.sequence,self.emergency_latched(),self.protection.audit_position().0)).map_err(|_|UError::InvalidData)?);
        let value=serde_json::json!({"version":1,"carrier":carrier,"cognition":u,"sequence":self.sequence,"emergency":self.emergency_latched(),"protection_sequence":self.protection.audit_position().0,"hash":hash});
        serde_json::to_vec(&value).map_err(|_|UError::InvalidData)
    }
    // Cognition only; every action still needs fresh trusted hazard evidence.
    pub fn from_constructing_checkpoint(bytes:&[u8])->UResult<Self> {
        if bytes.len()>32*1024*1024{return Err(UError::Capacity);}
        let s:UnifiedStored=serde_json::from_slice(bytes).map_err(|_|UError::InvalidData)?;
        let hash=uhash(&serde_json::to_vec(&(s.version,&s.carrier,&s.cognition,s.sequence,s.emergency,s.protection_sequence)).map_err(|_|UError::InvalidData)?);
        if s.version!=1 || s.hash!=hash || s.cognition.archived.len()>7 {return Err(UError::InvalidData);}
        if let Some(school)=&s.cognition.education {school.validate()?; if s.cognition.freeze_controller && !school.is_frozen(){return Err(UError::InvalidData);}}
        if let Some(revision)=&s.cognition.self_revision{revision.validate()?;}
        if let Some(options)=&s.cognition.temporal_options{
            options.validate(&s.cognition)?;
        }
        if s.cognition.instrumental_credit.is_some()&&s.cognition.agency_credit.is_none(){
            return Err(UError::InvalidData)
        }
        if let Some(c)=&s.cognition.instrumental_credit{
            c.validate(&s.cognition.current,&s.cognition.archived,
                s.cognition.freeze_controller)?;
        }
        if let Some(agency)=&s.cognition.agency_credit {
            let recorded:usize = s.cognition.archived.iter()
                .chain(std::iter::once(&s.cognition.current))
                .map(|d|d.learner.records().len()).sum();
            if agency.externally_forced_actions
                .checked_add(agency.self_selected_actions)
                .and_then(|n|n.checked_add(agency.option_continuation_actions))
                .is_none_or(|n|n>recorded as u64) {
                return Err(UError::InvalidData)
            }
        }
        let evo=EvoPhase::from_constructing_carrier_checkpoint(&s.carrier).ok_or(UError::InvalidData)?;
        let mut names=std::collections::BTreeSet::new();let mut costs=0u64;
        for d in s.cognition.archived.iter().chain(std::iter::once(&s.cognition.current)) {
            let _=ULearner::from_checkpoint(&d.learner.checkpoint()?)?;
            if !d.learner.carrier_is_managed() || d.learner.carrier_config().width!=evo.config().sensory_cells
                || d.learner.carrier_config().actions!=evo.config().motor_cells || !names.insert(&d.name)
                || d.name.is_empty() || d.name.len()>128 || d.last_construction>d.learner.records().len()
                || d.pending_credit.is_some_and(|f|f.iter().any(|x|!x.is_finite()||!(0.0..=1.0).contains(x))) {return Err(UError::InvalidData);}
            costs=costs.saturating_add(d.learner.records().iter().map(|r|r.external_cost).sum::<u64>());
        }
        if costs>s.cognition.total_external_cost || s.cognition.max_external_cost==0
            || evo.current_real().map(|r|r.sensory.clone())!=Some(usensory(s.cognition.current.learner.current())) {return Err(UError::InvalidData);}
        let mut rt=Self::new(evo).map_err(|_|UError::InvalidData)?;rt.sequence=s.sequence;rt.constructing=Some(s.cognition);rt.fresh_observation_required=false;
        rt.protection=HumanProtection::from_audit_position(s.protection_sequence,s.emergency);Ok(rt)
    }
    pub fn restore_constructing(&mut self,bytes:&[u8])->UResult<()> {
        let mut candidate=Self::from_constructing_checkpoint(bytes)?;
        let old=self.constructing.as_ref().ok_or(UError::NoEvidence)?;let new=candidate.constructing.as_ref().ok_or(UError::NoEvidence)?;
        if new.current.name!=old.current.name || new.current.learner.checkpoint()?!=old.current.learner.checkpoint()?
            || serde_json::to_vec(&old.education).map_err(|_|UError::InvalidData)?!=serde_json::to_vec(&new.education).map_err(|_|UError::InvalidData)?
            || serde_json::to_vec(&old.self_revision).map_err(|_|UError::InvalidData)?!=serde_json::to_vec(&new.self_revision).map_err(|_|UError::InvalidData)?
            || serde_json::to_vec(&old.temporal_options).map_err(|_|UError::InvalidData)?!=serde_json::to_vec(&new.temporal_options).map_err(|_|UError::InvalidData)?
            || serde_json::to_vec(&old.agency_credit).map_err(|_|UError::InvalidData)?!=serde_json::to_vec(&new.agency_credit).map_err(|_|UError::InvalidData)?
            || serde_json::to_vec(&old.instrumental_credit).map_err(|_|UError::InvalidData)?!=serde_json::to_vec(&new.instrumental_credit).map_err(|_|UError::InvalidData)?
            || new.nonce<old.nonce || new.computation_work<old.computation_work || new.total_external_cost!=old.total_external_cost {return Err(UError::Stale);}
        if candidate.emergency_latched(){let mut e=Self::absent_evidence();e.emergency_stop=true;self.protection.screen(0,e);}
        self.organism=candidate.organism;self.constructing=candidate.constructing.take();Ok(())
    }
}

include!("education_runtime.rs");

include!("recursive_search_runtime.rs");


#[cfg(test)]
mod autonomous_temporal_revision_tests {
    use super::*;
    fn episode(m:&mut MetaRevision,success:bool)->u64{
        let goal=[Some(2_i64)];
        if success{
            // Earlier successful preparation has 0 one-step goal value but
            // an existing epistemic field offers predictive context.
            m.observe([0.0,0.95,0.0,0.0,0.5],&goal,&[0],&[1])+
            m.observe([0.0,1.0,0.0,0.0,0.5],&goal,&[1],&[2])
        }else{
            let mut cost=0;
            for _ in 0..M_MAX_EPISODE{
                cost+=m.observe([0.0,0.0,0.0,0.0,0.5],&goal,&[0],&[0]);
            }
            cost
        }
    }
    #[test]
    fn delayed_outcomes_initiate_vm_program_change_and_prospective_admission(){
        let mut m=MetaRevision::default();let mut total=0;
        for i in 0..8{total+=episode(&mut m,i%2==0);}
        assert!(m.shadow.is_some(),"the organism must propose its own bounded evaluator");
        assert!(m.active.is_none(),"no adoption from training");
        for i in 0..4 {total+=episode(&mut m,i%2==0);}
        assert!(m.active.is_some(),"future real-episode gate must qualify a useful VM program");
        assert_eq!(m.receipts.len(),1);
        let mut f=[0.0,0.95,0.0,0.0,0.5];m.adjust(&mut f);
        assert!(f[0]>=0.80,"adopted executable expression must revise U1 goal evidence");
        assert_eq!(total,m.work);
        m.validate().unwrap();
        let raw=serde_json::to_vec(&m).unwrap();
        let restored:MetaRevision=serde_json::from_slice(&raw).unwrap();
        restored.validate().unwrap();
        assert_eq!(restored.report().active_program,m.report().active_program);
        let mut forged:serde_json::Value=serde_json::from_slice(&raw).unwrap();
        forged["receipts"][0]["after"]=serde_json::json!(999);
        let forged:MetaRevision=serde_json::from_value(forged).unwrap();
        assert!(forged.validate().is_err(),"promotion ledger must replay its heldout evidence");
        // The acquired VM score can become stale; later contradictory REAL
        // trajectories MUST return to the prior evaluation program.
        for i in 0..4{
            let g=[Some(2_i64)];
            if i%2==0{
                m.observe([0.0,0.0,0.0,0.0,0.5],&g,&[0],&[1]);
                m.observe([0.0,0.0,0.0,0.0,0.5],&g,&[1],&[2]);
            }else{
                for _ in 0..M_MAX_EPISODE {
                    m.observe([0.0,1.0,0.0,0.0,0.5],&g,&[0],&[0]);
                }
            }
        }
        assert_eq!(m.rollbacks,1);
        assert!(m.active.is_none(),"contradictory post-admission episodes must revoke the VM scorer");
        assert!(m.receipts[0].revoked);
        m.validate().unwrap();
        let restored:MetaRevision=serde_json::from_slice(&serde_json::to_vec(&m).unwrap()).unwrap();
        restored.validate().unwrap();
    }
    #[test]
    fn interrupted_goal_does_not_create_fake_success_or_failure(){
        let mut m=MetaRevision::default();
        m.observe([0.0,0.5,0.0,0.0,0.0],&[Some(3)],&[0],&[1]);
        m.observe([0.0,0.5,0.0,0.0,0.0],&[Some(4)],&[1],&[2]);
        assert_eq!(m.pending_steps.len(),1);assert_eq!(m.episodes,0);
        assert!(m.rows.is_empty());m.validate().unwrap();
    }
}

#[cfg(test)]
mod observable_graph_shared_budget_tests{
    use super::observable_graph_budget_ceiling;
    #[test]
    fn no_double_reservation_at_minimum_or_larger_planning_limits(){
        for total in [32_000_u64,36_000,48_000,120_000,1_000_000]{
            for memory in [0_u64,5_000,20_000]{
                let ceiling=observable_graph_budget_ceiling(total,memory);
                assert!(ceiling<=16_000);
                assert!(ceiling.saturating_add(memory).saturating_add(4_000)<=total);
            }
        }
        assert_eq!(observable_graph_budget_ceiling(32_000,20_000),8_000);
        assert_eq!(observable_graph_budget_ceiling(120_000,20_000),16_000);
    }
}
