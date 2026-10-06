use std::fs;
use std::path::{Path, PathBuf};

fn production_prefix(content: &str) -> &str {
    content.split("#[cfg(test)]").next().unwrap_or(content)
}

fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir).expect("read src directory") {
        let path = entry.expect("src entry").path();
        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    out
}

#[test]
fn production_source_has_no_evaluator_world_label_leakage() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("src");

    let forbidden = [
        "HiddenContext",
        "PrivateClass",
        "HiddenLaw",
        "Alpha",
        "Beta",
        "correct_action",
        "start_scene",
        "post_scene",
        "final_need",
        "changed_need",
        "old_need",
        "../tests",
        "tests/",
    ];

    let mut violations = Vec::new();

    for path in rust_files(&src) {
        let content = fs::read_to_string(&path).expect("read source file");
        let production = production_prefix(&content);

        for token in forbidden {
            if production.contains(token) {
                violations.push(format!("{} contains evaluator token {:?}", path.display(), token));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "full-ownership source audit failed:\n{}",
        violations.join("\n")
    );
}

#[test]
fn runtime_has_no_external_model_or_network_dependency() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cargo = fs::read_to_string(root.join("Cargo.toml")).expect("read Cargo.toml");

    for forbidden in [
        "reqwest",
        "tokio",
        "openai",
        "anthropic",
        "huggingface",
        "tch",
        "onnxruntime",
    ] {
        assert!(
            !cargo.to_lowercase().contains(forbidden),
            "runtime dependency {:?} violates the no-external-model/network boundary",
            forbidden
        );
    }
}
