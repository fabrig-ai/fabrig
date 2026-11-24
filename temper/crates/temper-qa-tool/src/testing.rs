//! Shared value fixtures for the QA test binaries (tools/crates/design.md, 3.2).
use crate::{CaseInput, Ledger, Pages};
mod agents;
mod clock;
mod git;
pub mod passes;
mod socket;
mod tree;
mod world;

pub use agents::{Agent, Agents, Child, DriveStreams};
pub use clock::Clock;
pub use git::{Git, Node, Range, baseline, baseline_with};
pub use socket::Connection;
pub use tree::{FileHandle, FileOp, Listener, Lock, Tree};
use world::path_io;
pub use world::{Loop, World, land};

pub fn pages() -> Pages {
    [
        ("ctl".into(), vec!["task".into(), "project".into()]),
        ("web".into(), vec!["work".into(), "home".into()]),
        ("tui".into(), vec!["chat".into()]),
    ]
    .into()
}
pub fn landing(ledger: &mut Ledger, number: u64) {
    assert_eq!(ledger.record_landing(&format!("{number:040x}"), number + 10, None, "a landing", 0), Some(number));
}
pub fn session(ledger: &mut Ledger, landing: u64) -> u64 {
    ledger.start_session(landing, "low", pages(), vec!["fresh".into()], Vec::new(), 0).expect("next triage")
}
pub fn case() -> CaseInput {
    CaseInput::decode(br#"{"surface":"web","page":"work","setup":"fresh","steps":[{"do":"open Work","expect":"Drafts lists the draft"}]}"#, "-").expect("case")
}
pub fn id(number: u64) -> String {
    format!("{number:040x}")
}
