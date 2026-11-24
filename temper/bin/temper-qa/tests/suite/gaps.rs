//! Gap commands and their read-only views (qa/gaps.md, 8).
use super::commands::{home, run, success};
use std::fs;
use temper_qa_tool::faces::RealFiles;

#[test]
fn triage_gap_records_owners_and_lists_newest_first() {
    let (_scratch, home) = home();
    let mut config: serde_json::Value =
        serde_json::from_slice(&fs::read(home.directory.join("qa.json")).expect("config")).expect("JSON");
    config["areas"] = serde_json::json!({"ctl":"tools"});
    fs::write(home.directory.join("qa.json"), serde_json::to_vec(&config).expect("JSON")).expect("config");
    success(&run(&home, &["triage", "gap", "ctl", "cannot judge"], Some("1"), None), b"gap 1\n");
    home.read(&mut RealFiles, |ledger| {
        assert_eq!(ledger.gaps[0].owner, "tools");
        Ok(())
    })
    .expect("recorded gap");
}
