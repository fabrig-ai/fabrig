//! Whole command lists from deployment documents, including dormant groups.
use crate::steps;
use temper_forge_setup::{Group, Kind, Stage, Step};

fn command(name: &str, arguments: &[&str], kind: Kind) -> Step {
    Step::new(name.as_bytes().into(), arguments.iter().map(|word| Box::from(word.as_bytes())).collect(), kind)
}

#[test]
fn the_fake_slot_names_its_forge_by_the_argument_and_has_a_remover() {
    let machine = temper_forge_setup::machine::FakeMachine {
        listen: "127.0.0.1:3007".parse().unwrap(),
        remote: b"http://127.0.0.1:3007".as_slice().into(),
        git: b"/data/playground/bin/git".as_slice().into(),
    };
    let llm = "127.0.0.1:8370".parse().unwrap();
    let flags = ["--config", "C", "--data", "D", "--cache", "K"].map(|word| Box::from(word.as_bytes()));
    assert_eq!(
        steps::fake_slot(&machine, llm, &flags),
        Group {
            kind: Some(temper_forge_setup::machine::BoxKind::Fake),
            remover: Some(command(
                "remove $forge",
                &["forge", "remove", "--config", "C", "--data", "D", "--cache", "K", "$forge"],
                Kind::Once,
            )),
            forge: 0,
            label: b"Playground".as_slice().into(),
            recorded: false,
            before: 0,
            steps: Box::new([
                (
                    Stage::Run,
                    command("fake llm", &["fake", "llm", "--config", "C", "--data", "D", "--cache", "K"], Kind::Long)
                        .listening(Box::new([llm])),
                ),
                (
                    Stage::Prepare,
                    command(
                        "prepare $forge",
                        &["forge", "prepare", "--config", "C", "--data", "D", "--cache", "K", "$forge"],
                        Kind::Once,
                    ),
                ),
                (
                    Stage::Run,
                    command(
                        "fake forge $forge",
                        &["fake", "forge", "--config", "C", "--data", "D", "--cache", "K", "$forge"],
                        Kind::Long,
                    )
                    .listening(Box::new([machine.listen])),
                ),
            ]),
        }
    );
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
            {"setting":{"kind":"project-ways","project":1},"set":{"drafts":true,"goals":true,"changes":false,"away":true,"way":0}}
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
