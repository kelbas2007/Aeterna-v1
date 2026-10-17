use aeterna_v1::word_problems::*;
use aeterna_v1::construction::vm::{Expr,Op,Error,Value};
use aeterna_v1::scientific_runtime::ScientificRuntime;
use aeterna_v1::construction::{Config,autonomy::AutonomyPolicy};
fn program(op:Op)->Expr{Expr::bin(op,Expr::Input(0),Expr::Input(1))}
fn model()->WordProblemModel {
    WordProblemModel{version:1,feature_kind:"role".into(),source_sha256:"0".repeat(64),training_examples:12,supported_training_examples:10,
        vocabulary:vec!["count:2".into()],idf:vec![1.0],programs:vec![AcquiredProgram{expression:program(Op::Add),support:6},AcquiredProgram{expression:program(Op::Sub),support:4}],
        intercept:vec![0.0,0.0],scales:vec![1.0,1.0],weights:vec![2,-2],developer_margin:1.0}
}
fn runtime()->ScientificRuntime{ScientificRuntime::new_constructing(Config::default(),AutonomyPolicy::default(),vec![0],"word-test").unwrap()}
#[test]fn decimal_and_grouped_quantities_are_not_split(){let g=ground("A paid $1,230.50 and $2.25. How much?").unwrap();assert_eq!(g.numbers.iter().map(Rational::text).collect::<Vec<_>>(),["2461/2","9/4"]);}
#[test]fn exact_rational_division_has_no_float_roundoff(){let x=[Rational::parse("1").unwrap(),Rational::parse("3").unwrap()];assert_eq!(execute(&program(Op::DivExact),&x).unwrap().text(),"1/3");}
#[test]fn integer_vm_contract_is_unchanged(){assert!(Op::DivExact.apply(Value::Int(1),Value::Int(3)).is_err());}
#[test]fn zero_denominator_is_not_a_prediction(){let x=[Rational::parse("1").unwrap(),Rational::parse("0").unwrap()];assert!(execute(&program(Op::DivExact),&x).is_err());}
#[test]fn exact_nested_composition(){let e=Expr::bin(Op::Sub,Expr::Input(0),Expr::bin(Op::Mul,Expr::Input(1),Expr::Input(2)));let x=["100","3","7"].map(|s|Rational::parse(s).unwrap());assert_eq!(execute(&e,&x).unwrap().text(),"79");}
#[test]fn overflow_does_not_wrap(){let x=[Rational::parse("999999999999999999999999999999999999").unwrap(),Rational::parse("99999999999999999").unwrap()];assert_eq!(execute(&program(Op::Mul),&x),Err(Error::Overflow));}
#[test]fn fabricated_numeric_slot_is_rejected(){assert!(ground("number0 has 5 apples").is_err());}
#[test]fn too_many_quantities_abstains(){let a=model().answer("1 2 3 4 5 6 7",false).unwrap();assert!(a.answer.is_none());}
#[test]fn unsupported_language_does_not_silently_drop_words(){assert!(model().answer("Сколько 3 и 7",false).unwrap().answer.is_none());}
#[test]fn margin_is_not_a_probability(){let a=model().answer("There are 7 and 11. How many?",false).unwrap();assert_eq!(a.answer.as_deref(),Some("18"));assert!(!a.confidence_is_probability);}
#[test]fn new_values_are_computed_not_recalled(){let a=model().answer("There are 73 and 119. How many?",false).unwrap();assert_eq!(a.answer.as_deref(),Some("192"));}
#[test]fn same_words_different_question_has_different_role_features(){let a=ground("There are 3 red and 7 blue balls. How many red balls?").unwrap();let b=ground("There are 3 red and 7 blue balls. How many blue balls?").unwrap();assert_ne!(features(&a,"role"),features(&b,"role"));}
#[test]fn model_roundtrip_and_validation(){let m=model();m.validate().unwrap();let s=serde_json::to_vec(&m).unwrap();let n:WordProblemModel=serde_json::from_slice(&s).unwrap();n.validate().unwrap();assert_eq!(s,serde_json::to_vec(&n).unwrap());}
#[test]fn invalid_model_rejected_before_install(){let mut r=runtime();let before=r.checkpoint_constructing().unwrap();let mut m=model();m.weights.clear();assert!(r.install_word_problem_knowledge(m).is_err());assert_eq!(r.checkpoint_constructing().unwrap(),before);}
#[test]fn model_cannot_smuggle_executable_calls(){let mut m=model();m.programs[0].expression=Expr::Call(42,vec![]);assert!(m.validate().is_err());}
#[test]fn repeated_input_and_wrong_support_rejected(){let mut m=model();m.programs[0].expression=Expr::bin(Op::Add,Expr::Input(0),Expr::Input(0));assert!(m.validate().is_err());let mut m=model();m.programs[0].support+=1;assert!(m.validate().is_err());}
#[test]fn invalid_float_and_unsorted_vocabulary_rejected(){let mut m=model();m.idf[0]=f64::NAN;assert!(m.validate().is_err());let mut m=model();m.scales[0]=0.0;assert!(m.validate().is_err());}
#[test]fn frozen_knowledge_cannot_be_replaced_silently(){let mut r=runtime();r.install_word_problem_knowledge(model()).unwrap();r.freeze_constructing().unwrap();assert_eq!(r.install_word_problem_knowledge(model()),Err(Error::Busy));}
#[test]fn explicit_extension_does_not_erase_world_experience_or_weights(){let mut r=runtime();r.freeze_constructing().unwrap();let before=r.unified_report().unwrap();r.begin_education_extension().unwrap();r.install_word_problem_knowledge(model()).unwrap();let after=r.unified_report().unwrap();assert_eq!(before.native_weights,after.native_weights);assert_eq!(before.total_external_cost,after.total_external_cost);assert_eq!(before.learner.events,after.learner.events);}
#[test]fn predictions_cannot_train_or_mutate_checkpoints(){let mut r=runtime();r.install_word_problem_knowledge(model()).unwrap();r.freeze_constructing().unwrap();let before=r.checkpoint_constructing().unwrap();for _ in 0..10{r.answer_word_problem("There are 8 and 9. How many?",false).unwrap();}assert_eq!(before,r.checkpoint_constructing().unwrap());}
#[test]fn old_and_new_interpretation_paths_coexist(){let mut r=runtime();r.install_word_problem_knowledge(model()).unwrap();assert!(r.answer_school("sum 8 and 9",aeterna_v1::education::AnswerMode::All).unwrap().answer.is_none());assert_eq!(r.answer_word_problem("There are 8 and 9",false).unwrap().answer.as_deref(),Some("17"));}
#[test]fn fresh_process_cognition_has_identical_answers(){let mut r=runtime();r.install_word_problem_knowledge(model()).unwrap();r.freeze_constructing().unwrap();let bytes=r.checkpoint_constructing().unwrap();let s=ScientificRuntime::from_constructing_checkpoint(&bytes).unwrap();assert_eq!(bytes,s.checkpoint_constructing().unwrap());let a=r.answer_word_problem("Have 9 and 5",false).unwrap();let b=s.answer_word_problem("Have 9 and 5",false).unwrap();assert_eq!(serde_json::to_vec(&a).unwrap(),serde_json::to_vec(&b).unwrap());}
#[test]fn raw_unmasked_year_count_is_an_input_not_hidden_metadata(){let g=ground("A home for 300000 dollars depreciated 8 % over a 1-year period.").unwrap();assert_eq!(g.numbers.iter().map(Rational::text).collect::<Vec<_>>(),["300000","8","1"]);}
#[test]fn digits_inside_a_word_are_not_arithmetic_quantities(){let g=ground("An MP3 player costs 40 dollars and tax is 2.4 dollars.").unwrap();assert_eq!(g.numbers.iter().map(Rational::text).collect::<Vec<_>>(),["40","12/5"]);assert!(g.tokens.contains(&"mp3".to_string()));}
