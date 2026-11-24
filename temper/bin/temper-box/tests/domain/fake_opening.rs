//! Scripted chat openings over committed fixtures.
use serde_json::Value;
use smith_domain_run as run;
use std::path::Path;
use temper_box_kit::{
    cluster::Cluster,
    fake_opening::{answer, calls, read, says},
    github_land::overlay_llm,
};

fn llm_fixture(source: &str) -> Value {
    let directory = skein_scratch::paths::package().join("tests/fixtures").join(source);
    let mut description = Cluster::description_from(Path::new("/fixture/opening"), &directory);
    description.bind("proc.forge.ready.address", "127.0.0.1:12345");
    let path = description.dir("llm").join("llm.json");
    let (_, bytes) =
        description.render_files("llm").files.into_iter().find(|(file, _)| *file == path).expect("LLM member");
    serde_json::from_slice(&bytes).expect("rendered LLM JSON")
}

#[test]
fn github_coder_fixture_keeps_its_finite_system_cue() {
    calls(&answer(&llm_fixture("github"), "@coding Fix", run::Mode::Finite), "read");
}

#[test]
fn github_chat_fixture_selects_its_delegate_from_the_opening_user_message() {
    calls(&answer(&llm_fixture("github"), "@forge_chat Update README", run::Mode::Open), "delegate");
}

#[test]
fn github_landing_fixture_selects_its_change_from_the_opening_user_message() {
    let mut value = llm_fixture("github");
    overlay_llm(&mut value);
    calls(&answer(&value, "@forge_chat Update README", run::Mode::Open), "change");
}

#[test]
fn cluster_chat_fixtures_select_their_opening_requests() {
    let value = llm_fixture("cluster");
    for (words, expected) in
        [("cluster-first-chat", "First worker replied."), ("cluster-second-chat", "Second worker replied.")]
    {
        says(&answer(&value, words, run::Mode::Open), expected);
    }
    calls(&answer(&value, "Add a file NOTICE to widget", run::Mode::Open), "delegate");
}

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
