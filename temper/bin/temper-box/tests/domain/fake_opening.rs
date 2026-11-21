//! Scripted chat openings over committed fixtures.
use serde_json::Value;
use smith_domain_run as run;
use temper_box_kit::fake_opening::{answer, calls, read, says};

#[test]
fn recorded_walk_chat_fixtures_select_their_opening_requests() {
    let tree = skein_scratch::paths::tree();
    for walk in ["b0", "b1"] {
        let directory = tree.join("tests/real-loop/tests/fixtures").join(walk);
        let words = read(&directory.join("person.json"));
        calls(
            &answer(
                &read(&directory.join("llm.json")),
                words["request"].as_str().expect("person request"),
                run::Mode::Open,
            ),
            "goal",
        );
    }
}

#[test]
fn demo_chat_fixtures_select_their_opening_requests() {
    let value: Value = serde_json::from_slice(temper_fake::demo::SCRIPTS).expect("binary scripts");
    let value = &value;
    for (words, prefix) in [
        ("Add CSV export to widget", "I’ll delegate this change"),
        ("Add a dark theme to widget", "I would give widget a dark theme"),
        ("Fix widget's empty search state", "I would have widget say"),
    ] {
        says(&answer(value, words, run::Mode::Open), prefix);
    }
}
