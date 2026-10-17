use aeterna_v1::quantity_relations::*;
use aeterna_v1::word_problems::{ground, execute, Rational};
use aeterna_v1::construction::{Config, Error, autonomy::AutonomyPolicy};
use aeterna_v1::construction::vm::{Expr, Op};
use aeterna_v1::scientific_runtime::ScientificRuntime;
fn factor(bias:Vec<f64>)->FactorModel {
    let k=bias.len();FactorModel{vocabulary:vec!["count:2".into()],idf:vec![1.0],intercept:bias,scales:vec![1.0;k],weights:vec![0;k]}
}
fn model(label:usize)->RelationalModel {
    let mut bias=vec![-10.0;7];bias[label]=10.0;
    RelationalModel{version:1,source_sha256:"0".repeat(64),supervision:"training_derivations".into(),training_examples:10,supported_examples:8,
        relevance_weight:0.5,max_used:4,beam_per_root:3,unary:factor(vec![-20.0,20.0]),pair:factor(bias)}
}
fn runtime()->ScientificRuntime {ScientificRuntime::new_constructing(Config::default(),AutonomyPolicy::default(),vec![0],"relations-test").unwrap()}
#[test]fn fixed_factors_fixture_constructs_new_expression_not_in_a_bank(){let m=model(0);m.validate().unwrap();let a=m.answer("Have 7 and 13 and 19 and 23. How many?").unwrap();assert_eq!(a.answer.as_deref(),Some("62"));assert_eq!(a.interpretations[0].used,vec![0,1,2,3]);assert!(a.candidates_constructed>0);}
#[test]fn ordered_subtraction_is_not_big_minus_small(){let a=model(1).answer("Have 3 and 11. What changes?").unwrap();assert_eq!(a.answer.as_deref(),Some("-8"));}
#[test]fn reverse_relation_is_distinct(){let a=model(2).answer("Have 3 and 11. What changes?").unwrap();assert_eq!(a.answer.as_deref(),Some("8"));}
#[test]fn exact_rational_execution_retained(){let a=model(4).answer("Have 1 and 3. How many?").unwrap();assert_eq!(a.answer.as_deref(),Some("1/3"));}
#[test]fn zero_division_never_becomes_a_numeric_infinity(){let a=model(4).answer("Have 1 and 0. How many?").unwrap();for c in a.interpretations{assert!(Rational::parse(&c.answer).is_ok());}}
#[test]fn numeric_replacement_does_not_change_features(){let a=ground("There are 7 red and 13 blue marbles. How many red marbles?").unwrap();let b=ground("There are 93 red and 2 blue marbles. How many red marbles?").unwrap();for i in 0..2{assert_eq!(features(&a,i,None).unwrap(),features(&b,i,None).unwrap());}assert_eq!(features(&a,0,Some(1)).unwrap(),features(&b,0,Some(1)).unwrap());}
#[test]fn question_is_an_explicit_factor_input(){let a=ground("There are 7 red and 13 blue marbles. How many red marbles?").unwrap();let b=ground("There are 7 red and 13 blue marbles. How many blue marbles?").unwrap();assert_ne!(features(&a,0,Some(1)).unwrap(),features(&b,0,Some(1)).unwrap());}
#[test]fn argument_roles_are_not_an_unordered_bag(){let a=ground("Have 7 red and 13 blue. How many red?").unwrap();assert_ne!(features(&a,0,Some(1)).unwrap(),features(&a,1,Some(0)).unwrap());}
#[test]fn malformed_argument_binding_rejected(){let a=ground("Have 7 and 13").unwrap();assert_eq!(features(&a,0,Some(0)),Err(Error::InvalidData));assert_eq!(features(&a,2,None),Err(Error::InvalidData));}
#[test]fn extra_numeric_scope_abstains(){assert!(model(0).answer("1 2 3 4 5 6 7").unwrap().answer.is_none());}
#[test]fn unsupported_language_is_not_dropped(){assert!(model(0).answer("Сложи 7 и 13").unwrap().answer.is_none());}
#[test]fn untrusted_reserved_slots_are_not_accepted(){assert!(model(0).answer("number0 has 7").unwrap().answer.is_none());}
#[test]fn model_must_explicitly_declare_derivation_supervision(){let mut m=model(0);m.supervision="autonomous_discovery".into();assert!(m.validate().is_err());}
#[test]fn malformed_matrices_and_floats_rejected(){let mut m=model(0);m.pair.weights.pop();assert!(m.validate().is_err());let mut m=model(0);m.relevance_weight=f64::NAN;assert!(m.validate().is_err());let mut m=model(0);m.unary.scales[0]=0.0;assert!(m.validate().is_err());}
#[test]fn invalid_search_capacity_rejected(){for n in [0,1,7,usize::MAX]{let mut m=model(0);m.max_used=n;assert!(m.validate().is_err());}let mut m=model(0);m.beam_per_root=5;assert!(m.validate().is_err());}
#[test]fn code_injection_cannot_be_deserialized_into_a_factor(){let m=model(0);let mut j=serde_json::to_value(m).unwrap();j["execute_shell"]=serde_json::json!("anything");assert!(serde_json::from_value::<RelationalModel>(j).is_err());}
#[test]fn frozen_knowledge_cannot_be_replaced(){let mut r=runtime();r.install_quantity_relations(model(0)).unwrap();r.freeze_constructing().unwrap();assert_eq!(r.install_quantity_relations(model(1)),Err(Error::Busy));}
#[test]fn invalid_install_is_transactional(){let mut r=runtime();let before=r.checkpoint_constructing().unwrap();let mut m=model(0);m.pair.idf.clear();assert!(r.install_quantity_relations(m).is_err());assert_eq!(before,r.checkpoint_constructing().unwrap());}
#[test]fn predictions_do_not_update_knowledge_or_controller(){let mut r=runtime();r.install_quantity_relations(model(0)).unwrap();r.freeze_constructing().unwrap();let before=r.checkpoint_constructing().unwrap();for _ in 0..4{r.answer_relational("Have 7 and 13. How many?").unwrap();}assert_eq!(before,r.checkpoint_constructing().unwrap());}
#[test]fn extension_preserves_controller_and_world_cost(){let mut r=runtime();r.freeze_constructing().unwrap();let a=r.unified_report().unwrap();r.begin_education_extension().unwrap();r.install_quantity_relations(model(0)).unwrap();let b=r.unified_report().unwrap();assert_eq!(a.native_weights,b.native_weights);assert_eq!(a.total_external_cost,b.total_external_cost);assert_eq!(a.learner.events,b.learner.events);}
#[test]fn checkpoint_roundtrip_is_exact(){let mut r=runtime();r.install_quantity_relations(model(0)).unwrap();r.freeze_constructing().unwrap();let bytes=r.checkpoint_constructing().unwrap();let s=ScientificRuntime::from_constructing_checkpoint(&bytes).unwrap();assert_eq!(bytes,s.checkpoint_constructing().unwrap());assert_eq!(serde_json::to_vec(&r.answer_relational("Have 2 and 3").unwrap()).unwrap(),serde_json::to_vec(&s.answer_relational("Have 2 and 3").unwrap()).unwrap());}
#[test]fn score_is_not_an_assertion_of_understanding(){let a=model(0).answer("Have 2 and 3").unwrap();assert!(!a.confidence_is_probability);assert!(!a.semantics_verified);assert!(a.reason.contains("NOT_semantically_verified"));}
#[test]fn standalone_relational_model_roundtrip_validates(){let m=model(0);let b=serde_json::to_vec(&m).unwrap();let restored:RelationalModel=serde_json::from_slice(&b).unwrap();restored.validate().unwrap();assert_eq!(b,serde_json::to_vec(&restored).unwrap());}
#[test]fn every_proposed_composition_replays_exactly(){let a=model(0).answer("Have 2 and 3 and 5. How many?").unwrap();let x=["2","3","5"].map(|s|Rational::parse(s).unwrap());for c in a.interpretations{assert_eq!(execute(&c.program,&x).unwrap().text(),c.answer);}}
#[test]fn single_quantity_needs_no_pair(){let a=model(0).answer("There are 17 oranges. How many?").unwrap();assert_eq!(a.answer.as_deref(),Some("17"));assert!(a.relations.is_empty());}
#[test]fn selected_relations_are_jointly_realized_in_a_tree(){let a=model(1).answer("There are 7, 11, 17. How much?").unwrap();let tree=&a.interpretations[0].program;fn input_count(e:&Expr)->usize{match e{Expr::Input(_)=>1,Expr::Bin(_,a,b)=>input_count(a)+input_count(b),_=>0}}assert_eq!(input_count(tree),a.interpretations[0].used.len());assert!(a.relations.iter().all(|r|["add","subtract","reverse_subtract","multiply","divide","reverse_divide","absent"].contains(&r.selected.as_str())));}
#[test]fn no_external_program_dispatch_added(){let e=Expr::Call(999,vec![]);assert_eq!(execute(&e,&[Rational::parse("1").unwrap()]),Err(Error::InvalidProgram));let v=Op::DivExact;assert_eq!(v,Op::DivExact);}

#[test]fn installed_relations_repair_the_existing_default_answer_path(){let mut r=runtime();r.install_quantity_relations(model(0)).unwrap();r.freeze_constructing().unwrap();assert_eq!(r.answer_word_problem("There are 7 and 13. How many?",false).unwrap().answer.as_deref(),Some("20"));}
