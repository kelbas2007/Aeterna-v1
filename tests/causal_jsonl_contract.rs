use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

#[test]
fn causal_jsonl_exposes_wide_inputs_motor_count_and_long_horizon_with_real_post_handshake() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_aeterna-v1"))
        .arg("--jsonl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    let mut exchange = |message: Value| -> Value {
        writeln!(input, "{message}").unwrap();
        input.flush().unwrap();
        let mut line = String::new();
        output.read_line(&mut line).unwrap();
        serde_json::from_str(&line).unwrap()
    };
    assert_eq!(
        exchange(
            json!({"op":"init","width":12,"actions":7,"planning_depth":12,
        "planning_work":120000,"journal_capacity":256})
        )["type"],
        "initialized"
    );
    exchange(json!({"op":"enable_self_directed_control"}));
    assert_eq!(
        exchange(json!({"op":"enable_causal_memory"}))["type"],
        "causal_memory_enabled"
    );
    let offers = (1..=7)
        .map(|cost| json!({"available":true,"max_cost":cost}))
        .collect::<Vec<_>>();
    let choice = exchange(json!({"op":"step","offers":offers}));
    assert_eq!(choice["type"], "action");
    assert!(choice["action"].as_u64().unwrap() < 7);
    let cost = choice["action"].as_u64().unwrap() + 1;
    assert_eq!(
        exchange(json!({"op":"post","result":{"observation":vec![0;12],"reward":0.0,"cost":cost}}))
            ["type"],
        "committed"
    );
    let report = exchange(json!({"op":"causal_report"}));
    assert_eq!(report["report"]["factual_transitions"], 1);
    assert_eq!(
        exchange(json!({"op":"report"}))["report"]["total_external_cost"],
        cost
    );
    assert_eq!(exchange(json!({"op":"quit"}))["type"], "bye");
    drop(exchange);
    drop(input);
    assert!(child.wait().unwrap().success());
}

#[test]
fn causal_jsonl_invalid_horizon_fails_closed_before_initialization() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_aeterna-v1"))
        .arg("--jsonl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    writeln!(
        input,
        "{}",
        json!({"op":"init","planning_depth":18446744073709551615u64})
    )
    .unwrap();
    input.flush().unwrap();
    let mut line = String::new();
    output.read_line(&mut line).unwrap();
    let result: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(result["type"], "error");
    assert!(result["message"].as_str().unwrap().contains("Capacity"));
    drop(input);
    assert!(!child.wait().unwrap().success());
}
