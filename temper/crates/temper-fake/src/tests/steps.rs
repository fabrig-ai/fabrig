//! Whole command lists from deployment documents, including dormant groups.
use crate::steps;
use temper_forge_setup::{Kind, Stage};
#[test]
fn the_fake_slot_names_its_forge_by_the_argument_and_has_a_remover() {
    let llm = "127.0.0.1:8370".parse().unwrap();
    let flags = ["--config", "C", "--data", "D", "--cache", "K"].map(|word| Box::from(word.as_bytes()));
    let group = steps::fake_slot(llm, &flags);
    assert_eq!(group.kind, Some(temper_forge_setup::machine::BoxKind::Fake));
    let remover = group.remover.unwrap();
    assert_eq!(remover.kind(), Kind::Once);
    assert_eq!(remover.arguments().last().unwrap().as_ref(), b"$forge");
    assert_eq!(group.steps.first().unwrap().1.listeners(), &[llm]);
    let (stage, prepare) = group.steps.get(1).unwrap();
    assert_eq!(*stage, Stage::Prepare);
    assert_eq!(prepare.arguments().last().unwrap().as_ref(), b"$forge");
    let (_, forge) = group.steps.last().unwrap();
    assert_eq!(forge.arguments().last().unwrap().as_ref(), b"$forge");
    assert_eq!(forge.listeners(), []);
    let mut bound = forge.clone();
    let address = ([127, 0, 0, 1], 3007).into();
    bound.bind_group_listener(address);
    assert_eq!(bound.listeners(), &[address]);
}

#[test]
fn the_configuring_cue_proposes_project_ways_with_the_hosts_settings_form() {
    let document = crate::backends::model_document(b"127.0.0.1:8370", b"/data").unwrap();
    crate::llm::Document::decode(&document).expect("fake model document");
    let value: serde_json::Value = serde_json::from_slice(&document).unwrap();
    let cue = value["cued"]
        .as_array()
        .unwrap()
        .iter()
        .find(|cue| cue["cue"] == "Show drafts and goals for this project.")
        .expect("configuring cue");
    let call = &cue["turns"][0]["lines"][0];
    assert_eq!(call["call"], "propose_settings");
    assert_eq!(call["arguments"]["against"], 1);
    assert_eq!(
        call["arguments"]["items"],
        serde_json::json!([
            {"setting":{"kind":"project-ways","project":1},"set":{"drafts":true,"goals":true,"changes":false,"way":0}}
        ])
    );
    assert_eq!(cue["turns"][1]["finish"], "stop");
}

#[test]
fn the_fresh_box_change_cue_uses_the_welcomes_hello_repository() {
    let document = crate::backends::model_document(b"127.0.0.1:8370", b"/data").unwrap();
    crate::llm::Document::decode(&document).expect("fake model document");
    let value: serde_json::Value = serde_json::from_slice(&document).unwrap();
    let cue = value["asked"]
        .as_array()
        .unwrap()
        .iter()
        .find(|cue| cue["cue"] == "Land a local hello status change.")
        .expect("fresh-box change cue");
    let call = &cue["turns"][0]["lines"][0];
    assert_eq!(call["call"], "change");
    assert_eq!(call["arguments"]["repository"], "work/hello");
    let producer = value["cued"]
        .as_array()
        .unwrap()
        .iter()
        .find(|cue| cue["cue"] == "## Task\n\nAdd a local hello status line to README.md.")
        .expect("hello producer");
    let write = &producer["turns"][1]["lines"][0];
    assert_eq!(write["call"], "write");
    assert_eq!(write["arguments"]["path"], "README.md");
}
