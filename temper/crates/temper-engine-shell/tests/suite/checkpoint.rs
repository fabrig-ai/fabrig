use super::path;
use jig_protocol_store::CheckpointName;
use std::os::unix::ffi::OsStrExt;
use temper_engine_shell::checkpoint::{Request, run};

#[test]
fn stopped_checkpoint_stands_with_its_record_and_is_removed() {
    let (_scratch, directories) = super::deployment::deployment();
    super::fixture::upgrade_fixture(&directories);
    assert_eq!(run("temper-engine", &directories, &Request::Take, &[], b"/", "test-build").unwrap(), 0);
    let checkpoints = path(&directories.data).join("state/checkpoints");
    let entries: Vec<_> = std::fs::read_dir(&checkpoints).unwrap().collect();
    assert_eq!(entries.len(), 1);
    let checkpoint = entries[0].as_ref().unwrap().path();
    assert!(checkpoint.join("checkpoint").is_file());
    let name = CheckpointName::parse(checkpoint.file_name().unwrap().as_bytes()).unwrap();
    assert_eq!(run("temper-engine", &directories, &Request::Remove(name), &[], b"/", "test-build").unwrap(), 0);
    assert!(!checkpoint.exists());
}
