//! Landed QA's focused value tests and seeded invariants (qa/README.md, 6).
mod build_files;
mod builds;
mod configuration;
mod events;
mod fuzzy;
mod gaps;
mod hold;
mod home;
mod installation;
mod ledger;
mod passes;
mod place;
mod reference;
mod run;
mod run_passes;
mod stat;
mod world;

use temper_qa_tool::{CaseInput, Ledger, Pages};

const HAPPY_PATH: &str = "QA judges the happy path alone: v0.1.0's stories and each part's bar, used as intended (temper/docs/design/draft/v0.1.0.md, 4). Edge cases are off it: malformed, out-of-range or oversized input, limits and bounds, refusals of bad input and their exit codes, errors, races and recovery.";

fn pages() -> Pages {
    [
        ("ctl".into(), vec!["task".into(), "project".into()]),
        ("web".into(), vec!["work".into(), "home".into()]),
        ("tui".into(), vec!["chat".into()]),
    ]
    .into()
}
fn landing(ledger: &mut Ledger, number: u64) {
    assert_eq!(ledger.record_landing(&format!("{number:040x}"), number + 10, None, "a landing", 0), Some(number));
}
fn session(ledger: &mut Ledger, landing: u64) -> u64 {
    ledger.start_session(landing, "low", pages(), vec!["fresh".into()], Vec::new(), 0).expect("next triage")
}
fn case() -> CaseInput {
    CaseInput::decode(br#"{"surface":"web","page":"work","setup":"fresh","steps":[{"do":"open Work","expect":"Drafts lists the draft"}]}"#, "-").expect("case")
}
