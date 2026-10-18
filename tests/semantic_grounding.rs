use aeterna_v1::education::{AnswerMode,GroundingLimits,TeachingExample,School};
use aeterna_v1::scientific_runtime::ScientificRuntime;
use aeterna_v1::construction::Config;
use aeterna_v1::construction::autonomy::AutonomyPolicy;
fn runtime()->ScientificRuntime {
    let mut r=ScientificRuntime::new_constructing(Config{width:32,actions:4,..Default::default()},AutonomyPolicy::default(),vec![0;32],"semantic-test").unwrap();
    r.enable_school_composition().unwrap();r
}
fn data(template:&str,func:fn(i64,i64)->i64)->Vec<TeachingExample> {
    (0..32).map(|i|{let a=3+i;let b=2+(i*7)%29;TeachingExample{question:template.replace("{a}",&a.to_string()).replace("{b}",&b.to_string()),answer:func(a,b),source:"test-teacher-not-test-answers".into()}}).collect()
}
fn teach(r:&mut ScientificRuntime,t:&str,f:fn(i64,i64)->i64){assert!(r.teach_school(data(t,f)).unwrap().accepted);}
fn answer(r:&ScientificRuntime,q:&str)->Option<String>{r.answer_school(q,AnswerMode::All).unwrap().answer}
fn basic()->ScientificRuntime {
    let mut r=runtime();teach(&mut r,"сумма {a} и {b}",|a,b|a+b);teach(&mut r,"произведение {a} и {b}",|a,b|a*b);
    teach(&mut r,"вычти {a} из {b}",|a,b|b-a);r
}
#[test]fn unseen_nested_program_is_built_from_learned_relations(){let r=basic();assert_eq!(answer(&r,"вычти произведение 3 и 4 из суммы 17 и 23"),None);assert_eq!(answer(&r,"вычти произведение 3 и 4 из сумма 17 и 23"),Some("28".into()));}
#[test]fn no_full_sentence_training_is_needed_for_composition(){let r=basic();let q="сумма произведение 7 и 8 и 9";assert_eq!(answer(&r,q),Some("65".into()));assert_ne!(r.answer_school_surface(q,AnswerMode::All).unwrap().answer,Some("65".into()));}
#[test]fn role_binding_depends_on_learned_order_not_number_bag(){let mut r=basic();teach(&mut r,"из {a} вычти {b}",|a,b|a-b);assert_eq!(answer(&r,"вычти 7 из 20"),Some("13".into()));assert_eq!(answer(&r,"из 7 вычти 20"),Some("-13".into()));}
#[test]fn unknown_negation_is_not_silently_ignored(){let r=basic();assert!(answer(&r,"не сумма 17 и 23").is_none());}
#[test]fn extra_unknown_number_is_not_discarded(){let r=basic();assert!(answer(&r,"сумма 17 и 23, 90").is_none());}
#[test]fn missing_argument_is_reported(){let r=basic();let a=r.understand_school("сумма 17 и",GroundingLimits::default()).unwrap();assert_eq!(a.status,"missing_arguments_or_unlearned_structure");}
#[test]fn unseen_word_has_no_assumed_semantics(){let r=basic();let a=r.understand_school("zqv 17 и 23",GroundingLimits::default()).unwrap();assert_eq!(a.unknown_terms,["zqv"]);assert!(answer(&r,"zqv 17 и 23").is_none());}
#[test]fn opaque_word_meaning_is_acquired_from_examples(){let mut a=runtime();let mut b=runtime();teach(&mut a,"zqv {a} и {b}",|x,y|x*y);teach(&mut b,"zqv {a} и {b}",|x,y|x+y);assert_eq!(answer(&a,"zqv 17 и 23"),Some("391".into()));assert_eq!(answer(&b,"zqv 17 и 23"),Some("40".into()));}
#[test]fn ambiguous_structure_is_not_replaced_by_a_surface_guess(){let mut r=runtime();teach(&mut r,"{a} минус {b}",|a,b|a-b);assert!(answer(&r,"12 минус 6 минус 2").is_none());let a=r.understand_school("12 минус 6 минус 2",GroundingLimits::default()).unwrap();assert_eq!(a.status,"ambiguous_program_meaning");assert_eq!(a.plans.len(),2);}
#[test]fn accidental_same_numeric_answer_does_not_remove_ambiguity(){let mut r=runtime();teach(&mut r,"{a} минус {b}",|a,b|a-b);let a=r.understand_school("4 минус 2 минус 0",GroundingLimits::default()).unwrap();assert_eq!(a.status,"ambiguous_program_meaning");assert!(a.plans.iter().all(|p|p.value==Some(2)));assert!(a.distinguishing_probe.is_some());}
#[test]fn generated_counterexample_distinguishes_competing_meanings(){let mut r=runtime();teach(&mut r,"{a} минус {b}",|a,b|a-b);let p=r.understand_school("4 минус 2 минус 0",GroundingLimits::default()).unwrap().distinguishing_probe.unwrap();assert!(p.predicted_answers.iter().any(|x|x!=&p.predicted_answers[0]));assert_ne!(p.inputs,[4,2,0]);}
#[test]fn parentheses_resolve_structural_ambiguity(){let mut r=runtime();teach(&mut r,"{a} минус {b}",|a,b|a-b);assert_eq!(answer(&r,"(12 минус 6) минус 2"),Some("4".into()));assert_eq!(answer(&r,"12 минус (6 минус 2)"),Some("8".into()));}
#[test]fn unmatched_parentheses_refused(){let r=basic();assert!(answer(&r,"сумма (17 и 23").is_none());}
#[test]fn budget_exhaustion_cannot_be_called_understanding(){let r=basic();let a=r.understand_school("сумма 17 и 23",GroundingLimits{work:1,..Default::default()}).unwrap();assert!(!a.complete_search);assert_eq!(a.status,"comprehension_budget_exhausted");}
#[test]fn ambiguity_cap_fails_closed(){let mut r=runtime();teach(&mut r,"{a} минус {b}",|a,b|a-b);let a=r.understand_school("12 минус 6 минус 2",GroundingLimits{alternatives_per_span:1,..Default::default()}).unwrap();assert!(!a.complete_search);}
#[test]fn arithmetic_overflow_has_no_fabricated_answer(){let r=basic();assert!(answer(&r,"сумма 9223372036854775807 и 1").is_none());}
#[test]fn decimal_is_not_two_integer_arguments(){let r=basic();assert!(answer(&r,"сумма 17,5 и 23").is_none());}
#[test]fn division_domain_checked_after_parsing(){let mut r=runtime();let e=(1..=32).map(|i|{let d=2+(i*7)%29;TeachingExample{question:format!("divide {} by {}",i*d,d),answer:i,source:"integer-teacher".into()}}).collect();assert!(r.teach_school(e).unwrap().accepted);assert_eq!(answer(&r,"divide 42 by 7"),Some("6".into()));assert!(answer(&r,"divide 42 by 0").is_none());assert!(answer(&r,"divide 43 by 7").is_none());}
#[test]fn freeze_guards_parsing_mode_and_all_new_lessons(){let mut r=basic();r.freeze_constructing().unwrap();assert!(r.enable_school_composition().is_err());assert!(r.teach_school(data("новая {a} и {b}",|a,b|a+b)).is_err());}
#[test]fn query_and_counterexample_do_not_change_frozen_state(){let mut r=basic();teach(&mut r,"{a} минус {b}",|a,b|a-b);r.freeze_constructing().unwrap();let before=r.checkpoint_constructing().unwrap();for _ in 0..8{let _=answer(&r,"сумма произведение 3 и 4 и 5");let _=r.understand_school("4 минус 2 минус 0",GroundingLimits::default()).unwrap();}assert_eq!(before,r.checkpoint_constructing().unwrap());}
#[test]fn grammar_reconstructed_from_the_same_checkpoint(){let mut r=basic();r.freeze_constructing().unwrap();let bytes=r.checkpoint_constructing().unwrap();let restored=ScientificRuntime::from_constructing_checkpoint(&bytes).unwrap();assert_eq!(bytes,restored.checkpoint_constructing().unwrap());assert_eq!(answer(&restored,"сумма произведение 7 и 8 и 9"),Some("65".into()));}
#[test]fn validation_failure_cannot_create_a_grammar_production(){let mut r=runtime();let mut e=data("unvalidated {a} и {b}",|a,b|a+b);for x in &mut e[24..]{x.answer+=1;}assert!(!r.teach_school(e).unwrap().accepted);assert!(answer(&r,"unvalidated 17 и 23").is_none());}
#[test]fn old_serialized_school_without_mode_remains_valid(){let s=School::default();let value=serde_json::to_value(s).unwrap();assert!(value.get("composition").is_none());let school:School=serde_json::from_value(value).unwrap();school.validate().unwrap();assert!(!school.composition_enabled());}
#[test]fn learned_wrapper_composes_with_unseen_subexpression(){let mut r=basic();let e=(1..=32).map(|i|TeachingExample{question:format!("вычисли {i}"),answer:i,source:"wrapper-teacher".into()}).collect();assert!(r.teach_school(e).unwrap().accepted);assert_eq!(answer(&r,"вычисли сумма произведение 3 и 4 и 7"),Some("19".into()));}

#[test]fn contradictory_sources_are_not_overridden_by_composition(){
 let mut r=basic();r.teach_school_facts(vec![
 aeterna_v1::education::Fact{question:"сумма 17 и 23".into(),answer:"40".into(),source:"a".into()},
 aeterna_v1::education::Fact{question:"сумма 17 и 23".into(),answer:"41".into(),source:"b".into()}]).unwrap();
 let a=r.answer_school("сумма 17 и 23",AnswerMode::All).unwrap();assert!(a.answer.is_none());assert_eq!(a.reason,"contradictory_teacher_sources");
}

#[test]fn reply_to_generated_question_selects_the_original_meaning(){
 let mut r=runtime();teach(&mut r,"{a} минус {b}",|a,b|a-b);r.freeze_constructing().unwrap();
 let q="12 минус 6 минус 2";let u=r.understand_school(q,GroundingLimits::default()).unwrap();let p=u.distinguishing_probe.unwrap();
 for (i,reply) in p.predicted_answers.iter().enumerate(){let a=r.clarify_school(q,reply.unwrap()).unwrap();assert_eq!(a.answer,u.plans[i].value.map(|v|v.to_string()));}
}
#[test]fn incorrect_clarification_does_not_trigger_arbitrary_program_selection(){
 let mut r=runtime();teach(&mut r,"{a} минус {b}",|a,b|a-b);assert!(r.clarify_school("12 минус 6 минус 2",i64::MAX).unwrap().answer.is_none());
}
#[test]fn clarification_is_episode_evidence_not_frozen_library_training(){
 let mut r=runtime();teach(&mut r,"{a} минус {b}",|a,b|a-b);r.freeze_constructing().unwrap();let before=r.checkpoint_constructing().unwrap();
 let q="12 минус 6 минус 2";let u=r.understand_school(q,GroundingLimits::default()).unwrap();let reply=u.distinguishing_probe.unwrap().predicted_answers[0].unwrap();
 r.clarify_school(q,reply).unwrap();assert_eq!(before,r.checkpoint_constructing().unwrap());assert!(answer(&r,q).is_none());
}
