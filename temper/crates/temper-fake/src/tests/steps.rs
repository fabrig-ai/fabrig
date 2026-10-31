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
