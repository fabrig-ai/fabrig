//! Contracts: smith/docs/design/domain/session.md, 10;
//! skein/docs/foundation/testing-strategy.md, 2.2.

//! Smith application argument menu, separate from Skein's provider-neutral fake.
//! These retained source fixtures know checkout paths and commands; the generic
//! fake only selects bounded opaque menu entries.
//! Contract: smith/docs/design/drawn-agent.md, sections 2–4 and 6.

mod counting;
pub(crate) use counting::{Counting, report};
pub use counting::{MeasuredBlock, MeasuredPrompt};

use skein_fake_llm_domain::api::{Bounds, InvalidInput, Menu, Slot};

/// A number no session in the world can issue.
pub const UNISSUED_JOB: u32 = 999_999;

/// A continuation line number no fixture issues.
pub const UNISSUED_OFFSET: u32 = 999_999;

/// Neither the seed nor a drawn change can create this path.
pub const UNISSUED_PATH: &[u8] = b"never-issued.txt";

/// Slot order also determines which slot wins if two delimiters overlap.
pub const SLOT_NAMES: [&[u8]; 5] = [b"job", b"offset", b"listed", b"found", b"named"];

/// Complete application-owned bodies and deliberate invalid entries.
#[must_use]
pub fn menu() -> Menu {
    weighted_menu(false)
}

pub(super) fn weighted_menu(rewrites: bool) -> Menu {
    // Each body declares every field. This shared prefix also ensures that cuts
    // through the common fields are identical across rows.
    const COMMON: &str = r#""wait_seconds":1,"stop":false,"timeout_seconds":4,"output_bytes":32,"depth":2,"glob":"*""#;
    const NORMAL: &str = r#""offset":1,"window":28,"yield_seconds":0,"command":"ls","pattern":":""#;
    const REWRITE: &str = r#""offset":1,"window":28,"yield_seconds":4,"command":"rewrite README.md","pattern":":""#;
    const FAIL: &str = r#""offset":1,"window":28,"yield_seconds":4,"command":"false","pattern":":""#;
    const CONTINUE: &str = r#""offset":"{{#offset}}","window":28,"yield_seconds":0,"command":"ls","pattern":":""#;
    const ZERO_AND_NUL_PATTERN: &str = r#""offset":0,"window":28,"yield_seconds":0,"command":"ls","pattern":"\u0000""#;
    const NUL_COMMAND: &str = r#""offset":1,"window":28,"yield_seconds":0,"command":"\u0000","pattern":":""#;
    const SOURCE: &str = r#""src/lib.rs: rewritten""#;
    const POLL: &str = r#""{{#job}}""#;
    const FIRST_JOB: &str = "1";
    // Alternate fields are JSON values, not unescaped strings. In particular a
    // slot can supply the never-issued path; null content prevents Write from
    // creating it, but the same body can still inspect, edit, or remove it.
    let body = |path: &str, content: &str, old: &str, new: &str, options: &str, job: &str| {
        let head = [COMMON, r#","job":"#, job, ",", options];
        let edits = [r#","edits":[{"old":""#, old, r#"","new":""#, new, r#"","all":false}]"#];
        let path = [r#""path":""#, path, r#"""#];
        // Put the long value first; cuts inside it need not tokenize the
        // other fields hundreds of times in the comparison test.
        let long = path[1].len() > 256;
        let mut body = Vec::with_capacity(256 + path[1].len());
        body.push(b'{');
        if long {
            for field in path {
                body.extend_from_slice(field.as_bytes());
            }
            body.push(b',');
        }
        for field in head.into_iter().chain(edits).chain([r#","content":"#, content]) {
            body.extend_from_slice(field.as_bytes());
        }
        if !long {
            body.push(b',');
            for field in path {
                body.extend_from_slice(field.as_bytes());
            }
        }
        body.push(b'}');
        body.into_boxed_slice()
    };
    let mut arguments = vec![
        body("{{listed}}", "null", "42", "43", NORMAL, POLL),
        body("{{found}}", "null", "42", "43", NORMAL, FIRST_JOB),
        body("{{named}}", "null", "42", "43", CONTINUE, POLL),
    ];
    if rewrites {
        let rewrite = body("README.md", r#""README.md: generated""#, "generated", "kept", REWRITE, FIRST_JOB);
        // A wider share of one useful body leaves room to draw a read, a
        // rewrite and a dependent write. Tool and body draws stay independent.
        arguments.extend(std::iter::repeat_n(rewrite, 9));
    }
    // Refusing literals need no slot scan or expansion. They still draw the
    // offered tool independently: only the deliberate unknown entry overrides it.
    // This 152-byte content exceeds the drawn payload, not the JSON call bound.
    let large_source = format!("\"src/lib.rs: {}\"", "rewritten ".repeat(14));
    let refusing = [
        body("src/lib.rs", &large_source, "u", "v", NORMAL, FIRST_JOB),
        body("./README.md", r#""README.md: rewritten""#, "hello", "hello", NORMAL, FIRST_JOB),
        body("docs", "null", "guide", "manual", NORMAL, FIRST_JOB),
        body(".", "null", "a", "b", NORMAL, FIRST_JOB),
        body("docs/guide.md", r#""docs/guide.md: rewritten""#, "absent", "x", NORMAL, FIRST_JOB),
        body("notes.md", r#""notes.md: noted""#, "noted", "kept", FAIL, FIRST_JOB),
        body("../outside.txt", SOURCE, "42", "43", NORMAL, FIRST_JOB),
        body(&"x".repeat(257), SOURCE, "42", "43", NORMAL, FIRST_JOB),
        body(".git/config", SOURCE, "42", "43", ZERO_AND_NUL_PATTERN, FIRST_JOB),
        body(r"bad\u0000path", SOURCE, "42", "43", NUL_COMMAND, FIRST_JOB),
        body("/ref/README.md", r#""README.md: amended""#, "reference", "amended", REWRITE, FIRST_JOB),
    ];
    let slot = |index: usize, before: &[u8], after: &[u8], unissued: &[u8], numeric| Slot {
        name: SLOT_NAMES[index].into(),
        before: before.into(),
        after: after.into(),
        unissued: unissued.into(),
        weight: 100,
        with: Box::new([]),
        foreign: 0,
        numeric,
    };
    let numeric = Some(Bounds { low: 1, high: u64::from(u32::MAX) });
    Menu {
        arguments: arguments.into_boxed_slice(),
        invalid: [
            InvalidInput {
                name: Some(b"delete_repository".as_slice().into()),
                arguments: br#"{"path":"src/lib.rs","content":"src/lib.rs: rewritten","command":"cargo test","job":1,"yield_seconds":0,"pattern":"answer","edits":[{"old":"42","new":"43"}]}"#.as_slice().into(),
            },
            InvalidInput { name: None, arguments: br#"{"path":"#.as_slice().into() },
            InvalidInput { name: None, arguments: b"{}".as_slice().into() },
        ].into_iter().chain(refusing.into_iter().map(|arguments| InvalidInput { name: None, arguments })).collect(),
        slots: Box::new([
            slot(0, b"job ", b" ", UNISSUED_JOB.to_string().as_bytes(), numeric),
            slot(1, b"[next line ", b"]", UNISSUED_OFFSET.to_string().as_bytes(), numeric),
            slot(2, b"\n", b"\t", UNISSUED_PATH, None),
            slot(3, b"\n", b":", UNISSUED_PATH, None),
            slot(4, b"`", b"`", UNISSUED_PATH, None),
        ]),
    }
}
