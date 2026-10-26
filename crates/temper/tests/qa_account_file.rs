//! M5 QA-B: FILE refusals and an open client during an account move.
pub mod support;

#[test]
fn refused_files_leave_the_account_with_its_holder() {
    let clock = skein_shell::Clock::new();
    let started = clock.now().now;
    let scratch = support::Scratch::new("qa_b_account_file");
    support::account_file_qa::files(scratch.path());
    scratch.remove();
    eprintln!("QA-B FILE probes: {:?}", clock.now().now.saturating_since(started));
}

#[test]
fn an_open_terminal_sees_the_move_and_the_tasks_moving_wait() {
    let scratch = support::Scratch::new("qa_b_account_terminal");
    support::account_file_qa::terminal(scratch.path());
    scratch.remove();
}

#[test]
fn an_old_jigm_imports_and_its_next_export_uses_the_new_form() {
    use support::account_move_qa::{Fixture, success};
    let fixture = Fixture::new("qa-old-jigm-conversion");
    success(&fixture.a().call(&["export", &fixture.number(), fixture.path()], None));
    let exported = std::fs::read(&fixture.file).expect("synthetic account export");
    support::write_private(&fixture.file, &old_encoder(&exported));
    success(&fixture.b.call(&["import", fixture.path(), "--key", "qa-old-jigm"], None));
    assert!(!fixture.file.exists(), "old file spent after adoption");
    let account = fixture.b.account_state("granted");
    let number = account["number"].as_u64().expect("adopted account number").to_string();
    success(&fixture.b.call(&["export", &number, fixture.path()], None));
    let converted = std::fs::read(&fixture.file).expect("next export");
    assert!(converted.starts_with(&[0, 1]), "next export uses the generated u16 version");
}

// The encoder at jig 365afd554^: JIGM, u16 version 1, then six
// u32-prefixed parts. Reuse only the synthetic export's parts, whose issuer
// has a runtime loopback port; no current moved-file encoder makes this file.
fn old_encoder(exported: &[u8]) -> Box<[u8]> {
    use skein_lib::{Reader, Writer};
    const PARTS: usize = 6;
    const MAGIC_BYTES: usize = 4;
    let mut input = Reader::new(exported);
    assert_eq!(input.u16(), Some(1), "source's generated form");
    let mut out = Writer::new(exported.len().checked_add(MAGIC_BYTES).expect("old form size"));
    out.put(b"JIGM").expect("room");
    out.put(&1_u16.to_be_bytes()).expect("room");
    for _ in 0..PARTS {
        let len = input.u32().expect("part length");
        let part = input.bytes(len).expect("part");
        out.put(&len.to_be_bytes()).expect("room");
        out.put(part).expect("room");
    }
    assert!(input.is_empty(), "all six exported parts retained");
    out.finish()
}
