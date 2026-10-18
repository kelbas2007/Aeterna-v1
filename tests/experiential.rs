use aeterna_v1::construction::{experiential::*,search_policy::{Example,ValidationCase,SynthesisTask,SearchPolicy,SearchLimits},vm::{Expr,Library,Op}};
fn case(i:i64)->ValidationCase{
 let samples=|offset:i64|(0..12).map(|j|{let a=i+j+offset;let b=2*j-i+offset;Example{input:vec![a,b,a-b],output:a+b}}).collect();
 ValidationCase{support:SynthesisTask{examples:samples(0),library:Library::default()},probes:SynthesisTask{examples:samples(20),library:Library::default()}}
}
fn limits()->SearchLimits{SearchLimits{work:12000,evaluations:48,max_nodes:9,frontier:64,edits_per_expansion:8}}
#[test]fn generated_programs_are_valid_and_bounded(){let policies=policy_grammar(3,72).unwrap();assert!(policies.len()>40);for p in &policies{p.validate().unwrap();}assert!(policies.iter().any(|p|p.variation.nodes()>7));assert_eq!(policies[0],SearchPolicy::default());}
#[test]fn identical_cohorts_forbidden(){let train:(Vec<_>)=(0..8).map(case).collect();let cfg=StudyConfig{limits:limits(),candidate_limit:16,shortlist:4,train_seed:100,validation_seed:200};assert!(study(&train,&train,cfg).is_err());}
#[test]fn study_never_claims_producer_admission(){let train:Vec<_>=(0..8).map(case).collect();let dev:Vec<_>=(50..58).map(case).collect();let cfg=StudyConfig{limits:limits(),candidate_limit:16,shortlist:4,train_seed:100,validation_seed:200};let s=study(&train,&dev,cfg).unwrap();assert!(!s.adopted_by_rsp2);assert_eq!(s.observations.len(),16);assert_eq!(s.training_cases,8);assert_eq!(s.validation_cases,8);assert!(s.total_metered_work>0);s.proposed.validate().unwrap();}
#[test]fn scramble_does_not_change_simple_unguided_search_results(){let c=vec![case(8),case(20)];let p=SearchPolicy::default();let a=evaluate_policy(&p,&c,&limits(),5,false).unwrap();let b=evaluate_policy(&p,&c,&limits(),5,true).unwrap();assert_eq!(a.verified_cases,b.verified_cases);}
#[test]fn invalid_input_width_and_limits_are_rejected(){assert!(policy_grammar(33,30).is_err());assert!(policy_grammar(3,1).is_err());assert!(study(&[],&[],StudyConfig{limits:limits(),candidate_limit:8,shortlist:4,train_seed:1,validation_seed:2}).is_err());}

#[test]
fn truncated_grammar_includes_every_residual_feature() {
    let policies=policy_grammar(3,128).unwrap();
    assert_eq!(policies.len(),128);
    for feature in [14usize,16,17] {
        assert!(policies.iter().any(|p|{
            let mut present=false;
            p.variation.map(&mut |e| {if let Expr::Input(i)=e {if *i==feature{present=true}}None});
            present
        }), "feature {} excluded by truncation",feature);
    }
}
#[test]
fn study_rejects_missing_support_rows_without_panic() {
    let mut train:Vec<_>=(0..8).map(case).collect();
    let dev:Vec<_>=(50..58).map(case).collect();
    train[0].support.examples.clear();
    let config=StudyConfig{limits:limits(),candidate_limit:8,shortlist:4,train_seed:5,validation_seed:7};
    assert!(matches!(study(&train,&dev,config),Err(aeterna_v1::construction::vm::Error::InvalidData)));
}
