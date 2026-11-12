//! Evidence directory reads: `temper/docs/design/testing/checks.md`, 5.4.
use crate::evidence;
use skein_scratch::{Scratch, paths};
use std::fs;
use std::path::PathBuf;

fn fixture() -> PathBuf {
    paths::package().join("src/tests/fixtures/evidence-minimal")
}
fn copy_fixture() -> Scratch {
    let scratch = Scratch::new("stage-one-evidence");
    for entry in fs::read_dir(fixture()).expect("fixture") {
        let entry = entry.expect("entry");
        fs::copy(entry.path(), scratch.path().join(entry.file_name())).expect("copy fixture");
    }
    scratch
}
#[test]
fn evidence_reads_a_directory_and_names_a_missing_file() {
    let evidence = evidence::read(&fixture()).expect("minimal evidence");
    assert_eq!(evidence.facts.project, 1);
    assert_eq!(evidence.revision["number"], 1);
    assert_eq!(evidence.items.items.len(), 0);
    assert_eq!(evidence.current.items.len(), 0);
    assert_eq!(evidence.proposal, "Apply the proposed settings.\n");
    assert_eq!(evidence.chat, "Describe the workflow from the process documents.\n");
    assert_eq!(evidence.confirmed.len(), 1);
    assert_eq!(evidence.confirmed[0].id, "box.project");
    assert!(evidence.confirmed[0].present);
    assert!(evidence.description.is_none());
    assert!(evidence.documents.is_empty());
    assert!(evidence.person.is_empty());
    assert!(evidence.judge.is_none());
    let scratch = copy_fixture();
    fs::remove_file(scratch.path().join("items.json")).expect("remove items");
    assert!(evidence::read(scratch.path()).expect_err("missing items").to_string().contains("items.json"));
}
#[test]
fn optional_evidence_is_read_and_invalid_json_names_its_file() {
    let scratch = copy_fixture();
    fs::write(scratch.path().join("description.txt"), "The workflow.").expect("description");
    fs::create_dir_all(scratch.path().join("documents/acme/widget/docs")).expect("documents");
    fs::write(scratch.path().join("documents/acme/widget/docs/rules.md"), "The rules.").expect("rules");
    fs::create_dir(scratch.path().join("person")).expect("person");
    fs::write(scratch.path().join("person/reference-landing.md"), "Landing.").expect("person document");
    fs::write(scratch.path().join("person/ignored.txt"), "Ignored.").expect("other person file");
    fs::write(scratch.path().join("judge.json"), r#"{"future-verdict":true}"#).expect("judge");
    let evidence = evidence::read(scratch.path()).expect("optional evidence");
    assert_eq!(evidence.description.as_deref(), Some("The workflow."));
    assert_eq!(evidence.documents["acme/widget/docs/rules.md"], "The rules.");
    assert_eq!(evidence.person.len(), 1);
    assert_eq!(evidence.person["reference-landing.md"], "Landing.");
    assert_eq!(evidence.judge.expect("judge")["future-verdict"], true);
    fs::write(scratch.path().join("judge.json"), "{").expect("bad judge");
    assert!(evidence::read(scratch.path()).expect_err("invalid judge").to_string().contains("judge.json"));
}
#[test]
fn each_required_evidence_file_is_named_on_failure() {
    for file in
        ["box.json", "revision.json", "items.json", "current.json", "proposal.txt", "chat.txt", "confirmed.json"]
    {
        let scratch = copy_fixture();
        fs::remove_file(scratch.path().join(file)).expect("remove required file");
        assert!(evidence::read(scratch.path()).expect_err("missing file").to_string().contains(file));
    }
    for file in ["box.json", "revision.json", "items.json", "current.json", "confirmed.json"] {
        let scratch = copy_fixture();
        fs::write(scratch.path().join(file), "{").expect("malformed JSON");
        assert!(evidence::read(scratch.path()).expect_err("malformed file").to_string().contains(file));
    }
}
