//! One source for the twins and their QA scripts (testing/qa/setups.md, 4–5).
use serde_json::{Value, json};
use std::path::Path;

#[test]
fn b1_plan_scripts_are_exactly_the_twins_cued_and_asked_in_box_form() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let twin = std::fs::read(root.join("tests/real-loop/tests/fixtures/b1/llm.json")).expect("B1 twin");
    let scripts = std::fs::read(root.join("docs/development/qa/setups/b1-plan/scripts.json")).expect("B1 QA scripts");
    let fixture: Value = serde_json::from_slice(&twin).expect("fixture JSON");
    let setup: Value = serde_json::from_slice(&scripts).expect("setup JSON");
    assert!(
        setup == json!({"cued":fixture["cued"], "asked":fixture["asked"]}),
        "only the fixture's cued and asked, with no otherwise"
    );
    temper_fake::llm::Document::decode_with_scripts(&twin, Some(&scripts)).expect("box --scripts form");
}

#[test]
fn b0_plan_scripts_are_exactly_the_twins_cued_and_asked_in_box_form() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let twin = std::fs::read(root.join("tests/real-loop/tests/fixtures/b0/llm.json")).expect("B0 twin");
    let scripts = std::fs::read(root.join("docs/development/qa/setups/b0-plan/scripts.json")).expect("B0 QA scripts");
    let fixture: Value = serde_json::from_slice(&twin).expect("fixture JSON");
    let setup: Value = serde_json::from_slice(&scripts).expect("setup JSON");
    assert!(
        setup == json!({"cued":fixture["cued"], "asked":fixture["asked"]}),
        "only the fixture's cued and asked, with no otherwise"
    );
    temper_fake::llm::Document::decode_with_scripts(&twin, Some(&scripts)).expect("box --scripts form");
}
