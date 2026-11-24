use super::*;
use jig_protocol_store::{OpenRefusal, Unready};

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
        let missing = CheckpointName { store: u64::MAX, suffix: 0 };
        assert_eq!(
            lines(binary, missing, Err(&RestoreRefusal::Source { why: CheckpointRefusal::Missing }), b"/deployment"),
            (format!("{prefix}: no checkpoint 18446744073709551615-0000000000000000\n"), 1)
        );
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
