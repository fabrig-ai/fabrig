//! Landed QA's focused value tests and seeded invariants (qa/README.md, 6).
mod builds;
mod configuration;
mod fuzzy;
mod home;
mod hold;
mod ledger;
mod passes;
mod reference;
mod run;
mod world;

use temper_qa_tool::{CaseInput, Ledger, Pages};

fn pages() -> Pages {
    [("web".into(), vec!["work".into(), "home".into()]), ("tui".into(), vec!["chat".into()])].into()
}
fn landing(ledger: &mut Ledger, number: u64) {
    assert_eq!(ledger.record_landing(&format!("{number:040x}"), number + 10, None, "a landing", 0), Some(number));
}
fn session(ledger: &mut Ledger, landing: u64) -> u64 {
    ledger.start_session(landing, "low", pages(), vec!["fresh".into()], 0).expect("next triage")
}
fn case() -> CaseInput {
    CaseInput::decode(br#"{"surface":"web","page":"work","setup":"fresh","steps":[{"do":"open Work","expect":"Drafts lists the draft"}]}"#, "-").expect("case")
}
