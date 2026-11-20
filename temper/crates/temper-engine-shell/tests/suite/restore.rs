use super::{path, shell};
use jig_protocol_store::CheckpointName;
use shell::restore::run;
use std::os::unix::ffi::OsStrExt;

#[test]
fn stopped_restore_switches_generation_and_keeps_source_and_safety_and_refuses_missing() {
    let (_scratch, directories) = super::deployment::deployment();
    super::fixture::upgrade_fixture(&directories);
    assert_eq!(
        shell::checkpoint::run(
            shell::ENGINE.binary,
            &directories,
            &shell::checkpoint::Request::Take,
            &[],
            b"/",
            "test-build"
        )
        .unwrap(),
        0
    );
    let deployment = path(&directories.data).join("state");
    let checkpoints = deployment.join("checkpoints");
    let name = std::fs::read_dir(&checkpoints).unwrap().next().unwrap().unwrap().file_name();
    let name = CheckpointName::parse(name.as_bytes()).unwrap();
    let active = std::fs::read_to_string(deployment.join("active")).unwrap().trim().parse::<u64>().unwrap();
    assert_eq!(run(shell::ENGINE.binary, &directories, name, "test-build").unwrap(), 0);
    assert_eq!(std::fs::read_to_string(deployment.join("active")).unwrap().trim().parse::<u64>().unwrap(), active + 1);
    let entries: Vec<_> = std::fs::read_dir(&checkpoints).unwrap().map(|entry| entry.unwrap().file_name()).collect();
    assert_eq!(entries.len(), 2, "source and safety");
    assert!(checkpoints.join(path(&name.bytes())).join("checkpoint").is_file());
    let safety = entries.iter().find(|entry| entry.as_bytes() != name.bytes()).unwrap();
    assert!(checkpoints.join(safety).join("checkpoint").is_file());
    assert!(CheckpointName::parse(safety.as_bytes()).is_some());
    let missing = CheckpointName { store: u64::MAX, suffix: 0 };
    assert_eq!(run(shell::ENGINE.binary, &directories, missing, "test-build").unwrap(), 1);
}
