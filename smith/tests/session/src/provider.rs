//! Contracts: smith/docs/design/domain/session.md, 10;
//! skein/docs/foundation/testing-strategy.md, 2.2.

//! Smith application argument menu, separate from Skein's provider-neutral fake.
//! These retained source fixtures know checkout paths and commands; the generic
//! fake only selects bounded opaque menu entries.
//! Contract: skein/docs/foundation/testing-strategy.md, sections 4 and 7.

mod counting;
pub(crate) use counting::{Counting, report};
pub use counting::{MeasuredBlock, MeasuredPrompt};

use skein_fake_llm_domain::api::{Bounds, InvalidInput, Menu, Slot};

/// A number no session in the world can issue.
pub const UNISSUED_JOB: u32 = 999_999;

/// Complete application-owned random inputs, including the preserved deliberate
/// unknown name, malformed object and missing fields. Script and menu wrappers
/// and bytes jointly obey the provider `Config.script_bytes` admission.
#[must_use]
pub fn menu() -> Menu {
    const UNKNOWN_TOOL_ARGUMENTS: &[u8] = br#"{"path":"src/lib.rs","content":"src/lib.rs: rewritten","command":"cargo test","job":1,"yield_seconds":0,"pattern":"answer","edits":[{"old":"42","new":"43"}]}"#;
    let arguments: [&[u8]; 5] = [
        br#"{"path":"src/lib.rs","content":"src/lib.rs: rewritten","command":"cargo test","job":"{{#job}}","yield_seconds":0,"pattern":"answer","edits":[{"old":"42","new":"43"}]}"#,
        br#"{"path":"./README.md","content":"README.md: rewritten","command":"ls","job":"{{#job}}","yield_seconds":0,"pattern":"hello","edits":[{"old":"hello","new":"goodbye"}]}"#,
        br#"{"path":"../outside.txt","content":"outside.txt: rewritten","command":"true","pattern":"outside","edits":[{"old":"a","new":"b"}]}"#,
        br#"{"path":"docs","content":"docs: rewritten","command":"cat README.md","job":"{{#job}}","yield_seconds":0,"pattern":"guide","edits":[{"old":"guide","new":"manual"}]}"#,
        br#"{"path":"notes.md","content":"notes.md: noted","command":"false","job":"{{#job}}","yield_seconds":0,"pattern":"noted","edits":[{"old":"noted","new":"kept"}]}"#,
    ];
    Menu {
        arguments: arguments.into_iter().map(Into::into).collect(),
        invalid: Box::new([
            InvalidInput {
                name: Some(b"delete_repository".as_slice().into()),
                arguments: UNKNOWN_TOOL_ARGUMENTS.into(),
            },
            InvalidInput { name: None, arguments: br#"{"path":"#.as_slice().into() },
            InvalidInput { name: None, arguments: b"{}".as_slice().into() },
        ]),
        slots: Box::new([Slot {
            name: b"job".as_slice().into(),
            before: b"job ".as_slice().into(),
            after: b" ".as_slice().into(),
            unissued: UNISSUED_JOB.to_string().into_bytes().into_boxed_slice(),
            weight: 100,
            numeric: Some(Bounds { low: 1, high: u64::from(u32::MAX) }),
        }]),
    }
}
