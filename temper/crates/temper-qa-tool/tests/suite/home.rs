//! Whole-document transactions and refused writes (qa/ledger.md, 3).
use skein_scratch::Scratch;
use temper_qa_tool::{Home, Ledger, Refusal};

#[test]
fn home_reads_empty_then_commits_whole_and_discards_refused_operations() {
    let scratch = Scratch::new("qa-home");
    std::fs::write(
        scratch.path().join("qa.json"),
        br#"{"owner":"qa","git":{"directory":"repo"},"agent":{"command":["agent"],"triage":"low"}}"#,
    )
    .expect("config");
    let home = Home::open(scratch.path()).expect("home");
    assert_eq!(home.run_lock, scratch.path().join("run.lock"));
    assert_eq!(home.session_directory(1), scratch.path().join("sessions/1"));
    assert_eq!(home.builds, scratch.path().join("builds"));
    home.read(|ledger| {
        assert_eq!(ledger, &Ledger::default());
        Ok(())
    })
    .expect("empty read");
    assert!(!home.ledger.exists());
    home.transact(|ledger| {
        ledger.record_head("first");
        Ok(())
    })
    .expect("commit");
    let bytes = std::fs::read(&home.ledger).expect("ledger");
    let refusal = home.transact(|ledger| {
        ledger.record_head("lost");
        Err::<(), _>(Refusal("no".into()))
    });
    assert!(refusal.is_err());
    assert_eq!(std::fs::read(&home.ledger).expect("unchanged"), bytes);
    assert!(!scratch.path().join("ledger.tmp").exists());
    home.transact(|ledger| {
        assert_eq!(ledger.head.as_deref(), Some("first"));
        ledger.record_head("second");
        Ok(())
    })
    .expect("reread");
    home.read(|ledger| {
        assert_eq!(ledger.head.as_deref(), Some("second"));
        Ok(())
    })
    .expect("read");
    std::fs::write(&home.ledger, b"broken document").expect("bad document");
    let refusal = home
        .transact(|ledger| {
            ledger.record_head("must not replace");
            Ok(())
        })
        .expect_err("refused");
    assert!(refusal.to_string().starts_with("ledger.json: "));
    assert_eq!(std::fs::read(&home.ledger).expect("unchanged"), b"broken document");
}
