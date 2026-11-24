//! Whole-document transactions and refused writes (temper/docs/design/testing/qa/ledger.md, 3).
use std::path::Path;
use temper_qa_tool::{
    Home, Ledger, Refusal,
    faces::{DirectoryMode, FilesFace, WriteMode, WriteOptions},
    testing::Tree,
};

#[test]
fn home_reads_empty_then_commits_whole_and_discards_refused_operations() {
    let mut tree = Tree::default();
    let directory = Path::new("/qa-home");
    tree.directory(directory, DirectoryMode::One).expect("home directory");
    let replace = WriteOptions::private(WriteMode::Replace);
    tree.write(
        &directory.join("qa.json"),
        br#"{"owner":"qa","git":{"directory":"repo"},"agent":{"command":["agent"],"triage":"low","pass":"high"},"request":{"command":["file"]}}"#,
        replace,
        false,
    )
    .expect("config");
    let home = Home::open(directory, &mut tree).expect("home");
    assert_eq!(home.run_lock, directory.join("run.lock"));
    assert_eq!(home.session_directory(1), directory.join("sessions/1"));
    assert_eq!(home.builds, directory.join("builds"));
    home.read(&mut tree, |ledger| {
        assert_eq!(ledger, &Ledger::default());
        Ok(())
    })
    .expect("empty read");
    assert!(!tree.exists(&home.ledger));
    let listener = tree.socket(directory.join("run.sock")).expect("wake socket");
    home.transact(&mut tree, |ledger, _| {
        ledger.record_head("first");
        Ok(())
    })
    .expect("commit");
    assert!(tree.accept(&listener).expect("wake after commit").is_some());
    let bytes = tree.read(&home.ledger).expect("ledger");
    let refusal = home.transact(&mut tree, |ledger, _| {
        ledger.record_head("lost");
        Err::<(), _>(Refusal("no".into()))
    });
    assert!(refusal.is_err());
    assert_eq!(tree.read(&home.ledger).expect("unchanged"), bytes);
    assert!(!tree.exists(directory.join("ledger.tmp")));
    assert!(tree.accept(&listener).expect("no wake after refusal").is_none());
    home.transact(&mut tree, |ledger, _| {
        assert_eq!(ledger.head.as_deref(), Some("first"));
        ledger.record_head("second");
        Ok(())
    })
    .expect("reread");
    home.read(&mut tree, |ledger| {
        assert_eq!(ledger.head.as_deref(), Some("second"));
        Ok(())
    })
    .expect("read");
    tree.write(&home.ledger, b"broken document", replace, false).expect("bad document");
    let refusal = home
        .transact(&mut tree, |ledger, _| {
            ledger.record_head("must not replace");
            Ok(())
        })
        .expect_err("refused");
    assert!(refusal.to_string().starts_with("ledger.json: "));
    assert_eq!(tree.read(&home.ledger).expect("unchanged"), b"broken document");
}
