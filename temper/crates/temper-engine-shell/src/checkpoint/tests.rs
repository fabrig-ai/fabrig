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
    let scratch = skein_scratch::Scratch::new_in(&skein_scratch::paths::target_tmp(), "checkpoint-path");
    let root = scratch.path();
    for dir in ["first", "second", "working"] {
        std::fs::create_dir(root.join(dir)).unwrap();
    }
    for (file, mode) in [("first/cmd", 0o600), ("second/cmd", 0o700), ("working/local", 0o700)] {
        let file = root.join(file);
        std::fs::write(&file, b"test executable").unwrap();
        std::fs::set_permissions(file, std::fs::Permissions::from_mode(mode)).unwrap();
    }
    std::fs::create_dir(root.join("first/directory")).unwrap();
    let working = root.join("working");
    let working = working.as_os_str().as_bytes();
    let env = vec![
        (Box::from(b"PATH".as_slice()), Box::from(b"../first:../second".as_slice())),
        (Box::from(b"TEMPER_CHECKPOINT".as_slice()), Box::from(b"wrong".as_slice())),
        (Box::from(b"KEPT".as_slice()), Box::from(b"value".as_slice())),
    ];
    assert_eq!(
        program(b"cmd", &env, working).unwrap().as_ref(),
        path(working).join("../second/cmd").as_os_str().as_bytes()
    );
    assert!(program(b"directory", &env, working).is_none());
    assert!(program(b"missing", &env, working).is_none());
    assert!(program(b"cmd", &[], working).is_none());
    assert_eq!(
        program(b"./local", &[], working).unwrap().as_ref(),
        path(working).join("./local").as_os_str().as_bytes()
    );
    let fd = skein_shell::open_root(path(working)).unwrap();
    let child = spawn(&words(&[b"cmd", b"--arg"]), &env, working, &root.join("checkpoint"), fd).unwrap();
    assert_eq!(child.args, words(&[b"cmd", b"--arg"]));
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
    temper_service_shell::close(vec![fd]);
}

// Bring the copied floor to this build once, then stop it before testing the tool.
fn upgrade_fixture(directories: &Directories) {
    use std::os::fd::IntoRawFd;
    let listeners = [0; 3].map(|_| std::net::TcpListener::bind("127.0.0.1:0").unwrap());
    let document_path = directories.document();
    let mut document: serde_json::Value = serde_json::from_slice(&std::fs::read(&document_path).unwrap()).unwrap();
    for (pointer, listener) in [
        "/application/clients/listener/socket",
        "/engine/hosts/listener/0/socket",
        "/application/webhooks/listener/socket",
    ]
    .iter()
    .zip(&listeners)
    {
        *document.pointer_mut(pointer).unwrap() = listener.local_addr().unwrap().to_string().into();
    }
    std::fs::write(document_path, serde_json::to_vec(&document).unwrap()).unwrap();
    let (mut service, deployment) = stopped::prepare("temper-engine", directories).unwrap();
    for listener in listeners {
        let address = listener.local_addr().unwrap();
        service.adopt_listener(skein_io::kernel::Fd::new(listener.into_raw_fd()), address).unwrap();
    }
    let root = skein_shell::open_root(path(&deployment)).unwrap();
    service.adopt_store(root, skein_shell::effective_user()).unwrap();
    let served = temper_service_shell::Served {
        iterate_stage: |service: &mut service::Service, stage, now, wall| {
            service::iterate_stage(service, stage, now, wall);
            if stage == service::Stage::Engine
                && service.engine_component().domain().ready()
                && !service.engine_component().stopping()
            {
                service.shutdown();
                service.shutdown();
            }
        },
        ..crate::served::SERVED
    };
    let mut host = temper_service_shell::Host::with_binary("temper-engine", service, served, directories.clone(), &[]);
    let mut kernel = skein_shell::Kernel::open(skein_shell::Config { operations: host.service.operations() }).unwrap();
    let exit = skein_shell::drive(&mut kernel, &skein_shell::Clock::new(), &mut host);
    assert_eq!(exit, skein_io::kernel::Exit::Code(1), "fixture abort closes the upgraded store");
    assert_eq!(host.service.end(), Some(&service::End::Aborted));
}

#[test]
fn stopped_checkpoint_stands_with_its_record_and_removal_reports_missing_twice() {
    let (_scratch, directories) = crate::upgrade::tests::deployment();
    upgrade_fixture(&directories);
    assert_eq!(run("temper-engine", &directories, &Request::Take, &[], b"/", "test-build").unwrap(), 0);
    let checkpoints = path(&directories.data).join("state/checkpoints");
    let entries: Vec<_> = std::fs::read_dir(&checkpoints).unwrap().collect();
    assert_eq!(entries.len(), 1);
    let checkpoint = entries[0].as_ref().unwrap().path();
    assert!(checkpoint.join("checkpoint").is_file());
    let name = CheckpointName::parse(checkpoint.file_name().unwrap().as_bytes()).unwrap();
    // No real child in focused: a missing command still releases the held lock.
    assert_eq!(
        run(
            "temper-engine",
            &directories,
            &Request::Hold { name, command: words(&[b"no-such-command"]) },
            &[],
            b"/",
            "test-build"
        )
        .unwrap(),
        127
    );
    assert_eq!(run("temper-engine", &directories, &Request::Remove(name), &[], b"/", "test-build").unwrap(), 0);
    assert!(!checkpoint.exists());
    assert_eq!(run("temper-engine", &directories, &Request::Remove(name), &[], b"/", "test-build").unwrap(), 1);
}
