//! The shipped box, first repository and first landed task across a restart.
use crate::support;

use serde_json::{Value, json};
use std::fs;
use support::{
    Scratch, boxed,
    ctl::{self, Call, Step},
    first_repository::{self, assert_box, flags, generated, stage_binaries},
};

fn rejects_orgs(member: &str) {
    let scratch = Scratch::new(&format!("qa-first-repository-{member}"));
    let root = scratch.path();
    generated(root);
    let path = root.join("temper.json");
    let mut document: Value = serde_json::from_slice(&fs::read(&path).expect("template")).expect("JSON");
    if member == "person.orgs" {
        document["application"]["machine"]["owned"]["person"]["orgs"] = json!(["work"]);
    } else {
        document["application"]["machine"]["owned"]["orgs"] = json!(["work"]);
    }
    support::write_private(&path, &serde_json::to_vec(&document).expect("JSON"));
    let ended = support::owned::prepare(root, 1);
    let error = String::from_utf8_lossy(&ended.stderr);
    assert_eq!(ended.status.code(), Some(1), "an obsolete owned org list must refuse command startup: {error}");
    assert!(
        error.contains("application.machine: invalid form"),
        "refusal must identify the machine configuration: {error}"
    );
    scratch.remove();
}

#[test]
fn owned_person_orgs_are_refused_at_start() {
    // protocol/setup.md, 8: an owned entry's person.orgs is an unknown member.
    rejects_orgs("person.orgs");
}
#[test]
fn owned_entry_orgs_are_refused_at_start() {
    // protocol/setup.md, 8: the older entry-level orgs is also an unknown member.
    rejects_orgs("orgs");
}

#[test]
fn first_landed_task_and_adoption_survive_a_box_restart_without_setup_replay() {
    // temper/docs/design/draft/playground-reset.md, 3: first intent, adoption and its landed task survive.
    let scratch = Scratch::new("qa-first-repository-restart");
    let root = scratch.path();
    generated(root);
    stage_binaries(root);
    let mut steps = vec![
        Step::Run(Call::sign_in()),
        Step::Run(Call::new(&["forge", "setup"])),
        Step::Run(Call::new(&["forge", "list", "--json"])),
    ];
    steps.extend(first_repository::task_steps(root));
    let first = boxed::run_ctl_with_llm(
        root,
        flags(root),
        root,
        steps,
        boxed::BOX_DEADLINE,
        |_| first_repository::live(root, root, root),
        first_repository::llm(root),
    );
    assert_box(&first);
    let stderr = String::from_utf8_lossy(&first.stderr);
    let evidence = first.checked.as_ref().expect("first task evidence");
    first_repository::assert_task("first", &first.calls[3..], &stderr, evidence);
    let adoption = String::from_utf8_lossy(&first.calls[1].stdout);
    assert!(
        adoption.contains("work/hello") && adoption.contains("owned"),
        "first setup must adopt work/hello: {adoption}"
    );
    let rows = ctl::json(&first.calls[2]);
    let row = first_repository::owned_row(&rows);
    assert_eq!(row["first"]["state"], "made", "{row}");
    let forge = row["number"].as_u64().expect("forge number");
    let first_intent = row["first"].clone();
    assert!(!root.join(format!("forge-{forge}/admin_token")).exists(), "the handover must be consumed");
    for member in ["api_token", "git_token", "user", "webhook_secret"] {
        assert!(
            !root.join(format!("forge-{forge}")).join(member).exists(),
            "owned credentials must remain records, never {member} files"
        );
    }
    let mut steps = vec![
        Step::Run(Call::new(&["forge", "list", "--until", "settled", "--timeout", "60s"])),
        Step::Run(Call::new(&["forge", "list", "--json"])),
        Step::Run(Call::new(&["forge", "setup", &forge.to_string()])),
    ];
    steps.extend(first_repository::found_steps());
    let restarted = boxed::run_ctl_with_exits(
        root,
        flags(root),
        root,
        steps,
        boxed::BOX_DEADLINE,
        || first_repository::live(root, root, root),
        &[(2, 2)],
    );
    assert_box(&restarted);
    let stderr = String::from_utf8_lossy(&restarted.stderr);
    first_repository::assert_task(
        "restarted",
        &restarted.calls[3..],
        &stderr,
        restarted.checked.as_ref().expect("restored evidence"),
    );
    let rows = ctl::json(&restarted.calls[1]);
    let row = first_repository::owned_row(&rows);
    assert_eq!(row["number"], forge, "the same forge record survives");
    assert_eq!(row["first"], first_intent, "the first repository intent and result survive: {row}");
    assert!(
        String::from_utf8_lossy(&restarted.calls[2].stdout).contains("not offered")
            || String::from_utf8_lossy(&restarted.calls[2].stderr).contains("not offered"),
        "setup after restart must not create or adopt again"
    );
    scratch.remove();
}
