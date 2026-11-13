//! Every refusal identifies its JSON member (redeploy.md, 2).
use serde_json::{Value, json};
use temper_redeploy_tool::{
    description::read,
    hosts::{fill, quote},
};

#[test]
fn the_example_reads_and_invalid_members_name_their_paths() {
    let example = include_str!("description.json");
    let parsed = read(example).expect("section 2.1's example");
    assert_eq!(parsed.source, "https://fabrig.ai/temper/v0");
    assert_eq!(parsed.workers[0].name, "w1");
    let original: Value = serde_json::from_str(example).unwrap();
    for (path, change) in [
        ("workers[0].swap", 0),
        ("engine.unknown", 1),
        ("engine.run", 2),
        ("engine.log", 3),
        ("workers[0].swap", 4),
        ("workers[1].name", 5),
        ("engine.stop", 6),
        ("engine.check", 7),
        ("engine.backup", 8),
        ("workers[0].document", 9),
        ("engine.run[0]", 10),
        ("source", 11),
    ] {
        let mut value = original.clone();
        match change {
            0 => {
                value["workers"][0].as_object_mut().unwrap().remove("swap");
            }
            1 => value["engine"]["unknown"] = json!("x"),
            2 => value["engine"]["run"] = json!([]),
            3 => value["engine"]["log"] = json!("log engine"),
            4 => value["workers"][0]["swap"] = json!("swap {since} {from} {to}"),
            5 => {
                let worker = value["workers"][0].clone();
                value["workers"].as_array_mut().unwrap().push(worker);
            }
            6 => value["engine"]["stop"] = json!("stop {since}"),
            7 => value["engine"]["check"] = json!("check {binary"),
            8 => value["engine"]["backup"] = json!("backup }"),
            9 => value["workers"][0]["document"] = json!(12),
            10 => value["engine"]["run"] = json!([""]),
            11 => value["source"] = json!(""),
            _ => unreachable!(),
        }
        let why = read(&value.to_string()).expect_err(path);
        assert!(why.contains(path), "{path}: {why}");
    }
}
#[test]
fn placeholders_are_posix_words_not_substitutions_in_inserted_values() {
    assert_eq!(
        fill("swap {from} {to}", &[("from", "/b/it's here"), ("to", "/usr/local/bin/temper-engine")]),
        "swap '/b/it'\\''s here' '/usr/local/bin/temper-engine'"
    );
    assert_eq!(fill("swap {from} {to}", &[("from", "{to}"), ("to", "$(no);\n")]), "swap '{to}' '$(no);\n'");
    assert_eq!(quote(""), "''");
}
