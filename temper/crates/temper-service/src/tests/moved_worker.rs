#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "ordinary tests insert refused document members"
)]
use super::fixtures;
use crate::startup;
use jig_protocol_config::Directories;
use serde_json::{Value, json};

fn moved_limit(member: &str, words: &str, worker_alone: bool) {
    for value in [json!(true), json!(false), json!("yes"), json!(1_u64), Value::Null, json!({}), json!([])] {
        let mut document: Value = serde_json::from_slice(&super::stock::generated()).unwrap();
        if worker_alone {
            document.as_object_mut().unwrap().remove("engine");
            document["workers"][0]["engine"] = json!({"socket":"127.0.0.1:7102"});
            document["application"] = json!({"limits":{"overrides":{}}});
        }
        let (parent, name) = member.split_once('.').unwrap();
        if parent == "workers[0]" {
            document["workers"][0][name] = value;
        } else {
            document[parent][name] = value;
        }
        let found = startup::shape(&serde_json::to_vec(&document).unwrap(), &Directories::working()).unwrap_err();
        assert_eq!(startup::lines(&found).as_ref(), format!("{member}: {words}\n").as_bytes());
    }
}
#[test]
fn engine_limits_is_refused_in_a_box_document() {
    moved_limit(
        "engine.limits",
        "moved: jig's limits are the setting limits; ask temper, or use temper ctl settings apply",
        false,
    );
}
#[test]
fn engine_memory_is_refused_in_a_box_document() {
    moved_limit(
        "engine.memory_bytes",
        "moved: the root's budget is transcript_memory, in the setting temper-limits",
        false,
    );
}
#[test]
fn application_limits_is_refused_in_a_box_document() {
    moved_limit(
        "application.limits",
        "moved: temper's limits are the setting temper-limits; ask temper, or use temper ctl settings apply",
        false,
    );
}
#[test]
fn worker_memory_is_refused_in_a_box_document() {
    moved_limit("workers[0].memory_bytes", "moved: nothing read a worker's memory_bytes; remove it", false);
}
#[test]
fn worker_memory_is_refused_in_a_worker_alone_document() {
    moved_limit("workers[0].memory_bytes", "moved: nothing read a worker's memory_bytes; remove it", true);
}

#[test]
fn moved_worker_members_print_their_paths_and_destinations_in_every_shape() {
    for document in [fixtures::document(), fixtures::worker_alone_document()] {
        for (name, words) in [
            ("graces", "moved: temper fixes a worker's graces (temper help server)"),
            ("programs", "moved: temper fixes the login checks' interval"),
        ] {
            let input = core::str::from_utf8(&document)
                .unwrap()
                .replace("\"workers\":[{", &format!("\"workers\":[{{\"{name}\":null,"));
            let found = startup::shape(input.as_bytes(), &Directories::working()).unwrap_err();
            assert_eq!(startup::lines(&found).as_ref(), format!("workers[0].{name}: {words}\n").as_bytes());
        }
    }
    for document in [fixtures::document(), fixtures::worker_alone_document(), fixtures::engine_alone_document()] {
        let mut input: Value = serde_json::from_slice(&document).unwrap();
        input["application"]["workers"] = Value::Null;
        let input = serde_json::to_vec(&input).unwrap();
        let found = startup::shape(&input, &Directories::working()).unwrap_err();
        assert_eq!(
            startup::lines(&found).as_ref(),
            b"application.workers: moved: a worker's workspaces are the data directory's; its git is found on PATH\n"
        );
    }
}
