//! One source for the twins and their QA scripts (testing/qa/setups.md, 2, 4–5).
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
fn ctl_hello_scripts_are_only_the_twins_review_with_qa_verdict_text_in_box_form() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let twin = std::fs::read(root.join("tests/real-loop/tests/fixtures/b0/llm.json")).expect("B0 twin");
    let scripts =
        std::fs::read(root.join("docs/development/qa/setups/ctl-hello/scripts.json")).expect("ctl-hello scripts");
    let fixture: Value = serde_json::from_slice(&twin).expect("fixture JSON");
    let setup: Value = serde_json::from_slice(&scripts).expect("setup JSON");
    let mut review = fixture["cued"]
        .as_array()
        .expect("cued")
        .iter()
        .find(|script| script["cue"] == "Review this change at the named head")
        .expect("review")
        .clone();
    let last = review["turns"].as_array_mut().expect("turns").last_mut().expect("last turn");
    let finish = last["lines"].as_array_mut().expect("lines").last_mut().expect("finish line");
    assert_eq!(finish["call"], "finish");
    finish["arguments"]["text"] =
        json!("Approve. QA's scripted review read the pull request, the files it changes and its checks.");
    assert_eq!(setup, json!({"cued": [review]}), "only B0's review, all but its verdict text");
    temper_fake::llm::Document::decode_with_scripts(&twin, Some(&scripts)).expect("box --scripts form");
}

#[test]
fn b0_plan_scripts_match_the_twin_without_the_chat_wakeup() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let twin = std::fs::read(root.join("tests/real-loop/tests/fixtures/b0/llm.json")).expect("B0 twin");
    let scripts = std::fs::read(root.join("docs/development/qa/setups/b0-plan/scripts.json")).expect("B0 QA scripts");
    let mut fixture: Value = serde_json::from_slice(&twin).expect("fixture JSON");
    let setup: Value = serde_json::from_slice(&scripts).expect("setup JSON");
    let chat = fixture["cued"]
        .as_array_mut()
        .expect("cued")
        .iter_mut()
        .find(|script| script["cue"] == "## Replying")
        .expect("chat script");
    let turns = chat["turns"].as_array_mut().expect("chat turns");
    let wake = turns
        .iter()
        .position(|turn| {
            let line = &turn["lines"][1];
            line["call"] == "message"
                && line["arguments"]["form"] == "question"
                && line["arguments"]["target"] == "{{#WORK}}"
        })
        .expect("chat's question to the work before go-ahead");
    turns.remove(wake);
    let reply = &mut turns[wake]["lines"][0]["text"];
    let plan = reply
        .as_str()
        .and_then(|text| text.strip_suffix("I’ve asked the work to wait for your go-ahead before making any changes."))
        .expect("chat's plan reply after the question");
    *reply = json!(format!("{plan}The work waits for your go-ahead before making any changes."));
    assert!(
        setup == json!({"cued":fixture["cued"], "asked":fixture["asked"]}),
        "only the twin's cued and asked, without the chat's question to the work and with the matching plan reply"
    );
    temper_fake::llm::Document::decode_with_scripts(&twin, Some(&scripts)).expect("box --scripts form");
}
