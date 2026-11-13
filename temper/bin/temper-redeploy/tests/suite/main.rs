//! The binary's table and pre-host refusals (testing/tools.md, 6; redeploy.md, 2, 7).
use serde_json::json;
use std::process::{Command, Output, Stdio};
use temper_redeploy_tool::{
    TABLE,
    fake::{NEW, description},
};

fn binary(scratch: &skein_scratch::Scratch, arguments: &[&str]) -> Output {
    Command::new(skein_scratch::paths::binary("temper-redeploy"))
        .args(arguments)
        .env_clear()
        .current_dir(scratch.path())
        .stdin(Stdio::null())
        .output()
        .expect("run binary")
}
fn paragraph(text: &str) -> String {
    text.split("\n\n").next().expect("opening paragraph").lines().collect::<Vec<_>>().join(" ")
}
#[test]
fn the_binary_answers_its_table() {
    let scratch = skein_scratch::Scratch::new("redeploy-table");
    for arguments in [
        vec!["--help"],
        vec!["-h"],
        vec!["help"],
        vec!["--version"],
        vec!["stage", "--help"],
        vec!["run", "--help"],
        vec!["referee", "--help"],
        vec!["nope"],
    ] {
        let output = binary(&scratch, &arguments);
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        match arguments.as_slice() {
            ["--help" | "-h" | "help"] => {
                assert_eq!(output.status.code(), Some(0));
                assert_eq!(
                    paragraph(&stdout),
                    "temper-redeploy: a deployment's redeploy: the engine alone by default, workers in turn, every part together across a breaking change, and its referee."
                );
                assert_eq!(stdout, TABLE.help());
                let body = stdout.split_once("\n\n").unwrap().1;
                for internal in ["stage", "--workers", "--together", "--record"] {
                    assert!(!body.contains(internal), "{internal}");
                }
            }
            ["--version"] => {
                assert_eq!(output.status.code(), Some(0));
                let prefix = format!("temper-redeploy {} (", env!("CARGO_PKG_VERSION"));
                let commit =
                    stdout.strip_prefix(&prefix).and_then(|text| text.strip_suffix(")\n")).expect("version line");
                let hash = commit.strip_suffix("-dirty").unwrap_or(commit);
                assert!(commit == "unknown" || (hash.len() == 12 && hash.bytes().all(|byte| byte.is_ascii_hexdigit())));
            }
            ["nope"] => {
                assert_eq!(output.status.code(), Some(2));
                assert_eq!(stdout, "");
                assert_eq!(stderr, "temper-redeploy: no command nope; temper-redeploy --help says more\n");
                continue;
            }
            _ => {
                assert_eq!(output.status.code(), Some(0));
                assert_eq!(
                    paragraph(&stdout),
                    "internal: not part of temper-redeploy's public command line; it may change without notice."
                );
            }
        }
        assert_eq!(stderr, "");
    }
}
#[test]
fn description_and_argument_refusals_have_their_exits_without_a_host() {
    let scratch = skein_scratch::Scratch::new("redeploy-refusals");
    let original = serde_json::to_value(description(1)).unwrap();
    for (path, case) in [
        ("workers[0].swap", 0),
        ("engine.extra", 1),
        ("engine.run", 2),
        ("engine.log", 3),
        ("workers[0].swap", 4),
        ("workers[1].name", 5),
    ] {
        let mut value = original.clone();
        match case {
            0 => {
                value["workers"][0].as_object_mut().unwrap().remove("swap");
            }
            1 => value["engine"]["extra"] = json!("extra"),
            2 => value["engine"]["run"] = json!([]),
            3 => value["engine"]["log"] = json!("log engine"),
            4 => value["workers"][0]["swap"] = json!("swap {from} {to} {since}"),
            5 => {
                let worker = value["workers"][0].clone();
                value["workers"].as_array_mut().unwrap().push(worker);
            }
            _ => unreachable!(),
        }
        std::fs::write(scratch.path().join("description.json"), value.to_string()).unwrap();
        let output = binary(&scratch, &["stage", "description.json", NEW]);
        assert_eq!(output.status.code(), Some(1), "{path}: {output:?}");
        assert!(String::from_utf8(output.stderr).unwrap().contains(path));
        assert_eq!(output.stdout, [] as [u8; 0]);
    }
    for commit in ["ABCDEF012345", "abcdef", "abcdef01234g", "abcdef0123456"] {
        let output = binary(&scratch, &["stage", "description.json", commit]);
        assert_eq!(output.status.code(), Some(2));
        assert!(
            String::from_utf8(output.stderr).unwrap().contains("COMMIT must be twelve lowercase hexadecimal digits")
        );
    }
    let output =
        binary(&scratch, &["run", "description.json", NEW, "--record", "record.json", "--workers", "--together"]);
    assert_eq!(output.status.code(), Some(2));
    let command = TABLE.commands.iter().find(|command| command.reach == skein_shell::Reach::Words(&["run"])).unwrap();
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        TABLE.refusal(command, "--workers and --together exclude each other")
    );
    std::fs::write(scratch.path().join("description.json"), original.to_string()).unwrap();
    std::fs::write(scratch.path().join("record.json"), "kept").unwrap();
    let output = binary(&scratch, &["run", "description.json", NEW, "--record", "record.json"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8(output.stderr).unwrap().contains("record"));
    assert_eq!(std::fs::read_to_string(scratch.path().join("record.json")).unwrap(), "kept");
}
