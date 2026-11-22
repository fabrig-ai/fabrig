use super::*;
use jig_protocol_store::{CheckpointRefusal, Checkpointed, Generations, OpenRefusal, Unready};
use service::checkpoint::Outcome;

fn words(words: &[&[u8]]) -> Box<[Box<[u8]>]> {
    words.iter().map(|word| Box::from(*word)).collect()
}
const NAME: &[u8] = b"00000000000000000012-00000000000000ff";
const NAME_VALUE: CheckpointName = CheckpointName { store: 12, suffix: 255 };

#[test]
fn arguments_take_hold_and_remove_preserve_the_child_and_reject_other_forms() {
    let environment = vec![(Box::from(b"HOME".as_slice()), Box::from(b"/home/test".as_slice()))];
    assert_eq!(configure(&[], &environment, b"/work").unwrap().1, Request::Take);
    let (directories, request) =
        configure(&words(&[b"--remove", NAME, b"--data", b"state"]), &environment, b"/work").unwrap();
    assert_eq!(directories.data.as_ref(), b"/work/state");
    assert_eq!(request, Request::Remove(NAME_VALUE));
    assert_eq!(
        configure(&words(&[b"--hold", NAME, b"--", b"cmd", b"--data", b"arg"]), &environment, b"/work").unwrap().1,
        Request::Hold { name: NAME_VALUE, command: words(&[b"cmd", b"--data", b"arg"]) }
    );
    for invalid in [
        words(&[b"--remove", b"12-ff"]),
        words(&[b"--hold", b"12-ff", b"--", b"cmd"]),
        words(&[b"--hold", NAME, b"--remove", NAME, b"--", b"cmd"]),
        words(&[b"--", b"cmd"]),
        words(&[b"cmd"]),
        words(&[b"--hold", NAME]),
        words(&[b"--hold", NAME, b"--"]),
        words(&[b"--remove", NAME, b"--", b"cmd"]),
    ] {
        assert!(configure(&invalid, &environment, b"/work").is_err(), "{invalid:?}");
    }
    let args = words(&[b"checkpoint", b"--remove", NAME]);
    let skein_shell::Reached::Command(row, rest) = crate::ENGINE.reach(b"temper-engine", &args) else {
        panic!("public checkpoint row")
    };
    assert_eq!(row, &crate::commands::CHECKPOINT);
    assert_eq!(row.set, skein_shell::Set::Public);
    assert_eq!(configure(rest, &environment, b"/work").unwrap().1, Request::Remove(NAME_VALUE));
}

#[test]
fn terminals_render_exact_checkpoint_lines_and_child_exits_for_each_binary() {
    let taken = Outcome::Taken(Checkpointed {
        name: NAME_VALUE,
        secrets: 8,
        linked: 3,
        copied: 2,
        bytes: 90,
        paused: 999,
        took: 7_999_999,
        held: 81,
        removed: 4,
        skipped: 1,
    });
    for binary in ["temper-engine", "temper-box"] {
        let prefix = format!("{binary} checkpoint");
        let name = String::from_utf8_lossy(NAME);
        assert_eq!(
            lines(binary, &taken, b"/deployment"),
            (
                format!(
                    "{prefix}: {name} at commit 12, secrets at 8: 3 files linked, 2 copied (90 bytes) in 7 ms; 81 bytes held; 4 older removed, 1 skipped\n"
                ),
                0
            )
        );
        assert_eq!(
            lines(binary, &Outcome::Removed(NAME_VALUE), b"/deployment"),
            (format!("{prefix}: removed {name}\n"), 0)
        );
        for (why, text) in [
            (CheckpointRefusal::InUse, format!("{name} is in use")),
            (CheckpointRefusal::Missing, format!("no checkpoint {name}")),
        ] {
            assert_eq!(
                lines(binary, &Outcome::Refused { name: Some(NAME_VALUE), why }, b"/deployment"),
                (format!("{prefix}: {text}\n"), 1)
            );
        }
        assert_eq!(
            lines(binary, &Outcome::Refused { name: None, why: CheckpointRefusal::InProgress }, b"/deployment"),
            (format!("{prefix}: refused: checkpoint in progress\n"), 1)
        );
        assert_eq!(
            lines(binary, &Outcome::OpenRefused(OpenRefusal::Generations { why: Generations::Busy }), b"/deployment"),
            (
                format!(
                    "{prefix}: /deployment is in use by another process; while the engine serves, take one with temper ctl DIR deployment checkpoint\n"
                ),
                1
            )
        );
        let why = OpenRefusal::Unready { why: Unready::Empty };
        assert_eq!(
            lines(binary, &Outcome::OpenRefused(why.clone()), b"/deployment"),
            crate::lines::store_refusal(binary, service::refused_start(&why, b"/deployment"))
        );
        for (why, text) in [
            (None, "not found"),
            (Some(skein_io::Error::Busy), "no process slot"),
            (Some(skein_io::Error::Other), "io refused the command"),
        ] {
            assert_eq!(
                lines(binary, &Outcome::CantRun { command: Box::from(b"cmd".as_slice()), why }, b"/deployment"),
                (format!("{prefix}: can't run cmd: {text}\n"), 127)
            );
        }
        for (exit, code) in [(skein_io::kernel::Exit::Code(3), 3), (skein_io::kernel::Exit::Signal(15), 143)] {
            assert_eq!(lines(binary, &Outcome::Child(exit), b"/deployment"), (String::new(), code));
        }
    }
}

#[test]
fn path_resolution_requires_an_executable_and_spawn_uses_the_invocation() {
    let root = path(b"/fixture");
    let working = root.join("working");
    let facts = |file: &std::path::Path| {
        [
            ("../first/cmd", (true, 0o600)),
            ("../second/cmd", (true, 0o700)),
            ("./local", (true, 0o700)),
            ("../first/directory", (false, 0o700)),
        ]
        .into_iter()
        .find_map(|(name, facts)| (file == working.join(name)).then_some(facts))
    };
    let working = working.as_os_str().as_bytes();
    let env = vec![
        (Box::from(b"PATH".as_slice()), Box::from(b"../first:../second".as_slice())),
        (Box::from(b"TEMPER_CHECKPOINT".as_slice()), Box::from(b"wrong".as_slice())),
        (Box::from(b"KEPT".as_slice()), Box::from(b"value".as_slice())),
    ];
    assert_eq!(
        program(b"cmd", &env, working, &facts).unwrap().as_ref(),
        path(working).join("../second/cmd").as_os_str().as_bytes()
    );
    assert!(program(b"directory", &env, working, &facts).is_none());
    assert!(program(b"missing", &env, working, &facts).is_none());
    assert!(program(b"cmd", &[], working, &facts).is_none());
    assert_eq!(
        program(b"./local", &[], working, &facts).unwrap().as_ref(),
        path(working).join("./local").as_os_str().as_bytes()
    );
    let fd = skein_io::kernel::Fd::new(7);
    let checkpoint = root.join("checkpoint");
    let child = spawn(&words(&[b"cmd", b"-c", b"exit 3", b"x"]), &env, working, &checkpoint, fd, &facts).unwrap();
    assert_eq!(child.program.as_ref(), path(working).join("../second/cmd").as_os_str().as_bytes());
    assert_eq!(child.args, words(&[b"-c", b"exit 3", b"x"]));
    let no_arguments = spawn(&words(&[b"./local"]), &[], working, &checkpoint, fd, &facts).unwrap();
    assert_eq!(no_arguments.program.as_ref(), path(working).join("./local").as_os_str().as_bytes());
    assert!(no_arguments.args.is_empty());
    assert_eq!(child.root, fd);
    assert_eq!(child.dir.as_ref(), b".");
    assert_eq!(child.env.iter().filter(|entry| entry.starts_with(b"TEMPER_CHECKPOINT=")).count(), 1);
    assert!(
        child
            .env
            .contains(&Box::from([b"TEMPER_CHECKPOINT=", root.join("checkpoint").as_os_str().as_bytes()].concat()))
    );
    assert!(child.env.contains(&Box::from(b"KEPT=value".as_slice())));
    assert_eq!(
        child.pipes.as_ref(),
        &(0..3)
            .map(|child| skein_io::kernel::Pipe { child, way: skein_io::kernel::Way::Inherit, parent: None })
            .collect::<Vec<_>>()
    );
}
