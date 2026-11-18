use super::*;
use jig_protocol_store::{OpenRefusal, Unready};
use std::os::unix::ffi::OsStrExt;

const NAME: &str = "00000000000000000012-00000000000000ff";
const VALUE: CheckpointName = CheckpointName { store: 12, suffix: 255 };
const SAFETY: CheckpointName = CheckpointName { store: 18, suffix: 256 };

fn words(words: &[&str]) -> Box<[Box<[u8]>]> {
    words.iter().map(|word| Box::from(word.as_bytes())).collect()
}

#[test]
fn restore_row_parses_a_canonical_name_and_directories_only() {
    let arguments = words(&["restore", NAME, "--data", "state"]);
    let skein_shell::Reached::Command(row, rest) = crate::ENGINE.reach(b"temper-engine", &arguments) else {
        panic!("restore row")
    };
    assert_eq!(row, &crate::commands::RESTORE);
    assert_eq!(row.set, skein_shell::Set::Public);
    assert_eq!(row.forms, &["NAME [DIRECTORIES]"]);
    assert_eq!(row.line, "restores a checkpoint over the stopped deployment, after a safety checkpoint of it");
    let environment = [(Box::from(b"HOME".as_slice()), Box::from(b"/home/test".as_slice()))];
    let (directories, name) = configure(rest, &environment, b"/work").unwrap();
    assert_eq!(name, VALUE);
    assert_eq!(directories.data.as_ref(), b"/work/state");
    for invalid in [vec![], vec!["12-ff"], vec![NAME, NAME], vec![NAME, "--hold"], vec![NAME, "--", "cmd"]] {
        assert!(configure(&words(&invalid), &environment, b"/work").is_err(), "{invalid:?}");
    }
}

#[test]
fn restore_terminals_render_exact_lines_and_exits_for_each_binary() {
    let restored =
        Restored { name: VALUE, safety: SAFETY, generation: 4, linked: 3, copied: 2, bytes: 90, took: 7_999_999 };
    let safety = String::from_utf8_lossy(&SAFETY.bytes()).into_owned();
    for binary in [crate::ENGINE.binary, "temper-box"] {
        let prefix = format!("{binary} restore");
        assert_eq!(
            lines(binary, VALUE, Ok(&restored), b"/deployment"),
            (
                format!(
                    "{prefix}: restored {NAME} as generation 4: 3 files linked, 2 copied (90 bytes) in 7 ms; the state before it is checkpoint {safety}, and restore {safety} brings it back\n"
                ),
                0
            )
        );
        for (why, text) in [
            (RestoreRefusal::Busy, "/deployment is in use by another process; stop the engine before restoring".into()),
            (RestoreRefusal::Source { why: CheckpointRefusal::InUse }, format!("{NAME} is in use")),
            (RestoreRefusal::Source { why: CheckpointRefusal::Missing }, format!("no checkpoint {NAME}")),
        ] {
            assert_eq!(lines(binary, VALUE, Err(&why), b"/deployment"), (format!("{prefix}: {text}\n"), 1));
        }
        for why in [OpenRefusal::Unready { why: Unready::Empty }, OpenRefusal::Failed] {
            assert_eq!(
                lines(binary, VALUE, Err(&RestoreRefusal::Open { why: why.clone() }), b"/deployment"),
                crate::lines::store_refusal(binary, service::refused_start(&why, b"/deployment"))
            );
        }
        for why in [
            RestoreRefusal::Source { why: CheckpointRefusal::InProgress },
            RestoreRefusal::Unreadable { path: "checkpoint".into() },
            RestoreRefusal::Deployment,
            RestoreRefusal::Safety { why: CheckpointRefusal::InProgress },
            RestoreRefusal::Staging { safety: SAFETY, path: "generations/4".into(), error: None },
            RestoreRefusal::Interrupted {
                safety: SAFETY,
                path: "active".into(),
                error: skein_io::kernel::Error::NotFound,
            },
        ] {
            assert_eq!(
                lines(binary, VALUE, Err(&why), b"/deployment"),
                (format!("{prefix}: refused: {}\n", why.words()), 1)
            );
        }
    }
}

#[test]
fn stopped_restore_switches_generation_and_keeps_source_and_safety_and_refuses_missing() {
    let (_scratch, directories) = crate::upgrade::tests::deployment();
    crate::checkpoint::tests::upgrade_fixture(&directories);
    assert_eq!(
        crate::checkpoint::run(
            crate::ENGINE.binary,
            &directories,
            &crate::checkpoint::Request::Take,
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
    assert_eq!(run(crate::ENGINE.binary, &directories, name, "test-build").unwrap(), 0);
    assert_eq!(std::fs::read_to_string(deployment.join("active")).unwrap().trim().parse::<u64>().unwrap(), active + 1);
    let entries: Vec<_> = std::fs::read_dir(&checkpoints).unwrap().map(|entry| entry.unwrap().file_name()).collect();
    assert_eq!(entries.len(), 2, "source and safety");
    assert!(checkpoints.join(path(&name.bytes())).join("checkpoint").is_file());
    let safety = entries.iter().find(|entry| entry.as_bytes() != name.bytes()).unwrap();
    assert!(checkpoints.join(safety).join("checkpoint").is_file());
    assert!(CheckpointName::parse(safety.as_bytes()).is_some());
    let missing = CheckpointName { store: u64::MAX, suffix: 0 };
    assert_eq!(run(crate::ENGINE.binary, &directories, missing, "test-build").unwrap(), 1);
}
