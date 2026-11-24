use super::fixtures::{
    self, ADMIN_FAILURE, APP_INI, Effect, FORGEJO, FORGEJO_LOCK, FORGEJO_PIN, NEWER, RUN_ROWS, RUNNER_CONFIGURATION,
    RUNNER_HELD, RUNNER_LOCK, SLOT, SOURCE_LIST, STATE, Script, UNRECORDED, capture, directories, place,
};
use skein_lib::{Duration, Secret};
use std::{cell::RefCell, io, path::Path};
use temper_forge_setup::{
    machine::BoxKind,
    prepare::Prepare,
    run as plan,
    setup::{Mode, Owned},
    spec::Which,
    state,
};
use temper_forge_setup_shell::{command, prepare, run, setup_file, words};

const BINARY: &str = "temper-box";
const SOURCE_DEADLINE: Duration = Duration::from_millis(200);
const DISCOVERY: &[u8] = b"http://127.0.0.1:31000/.well-known/openid-configuration";
const LEFTOVER: &str = "/fixture/forge-1/forgejo/left-over";

pub(super) fn owned(runner: bool) -> Owned {
    let dirs = jig_protocol_config::Directories::given(b"/fixture", b"/fixture", b"/fixture").expect("fixture dirs");
    let setup = setup_file::read_with(&dirs, SLOT, BoxKind::Owned, |_, _| {
        Ok(Secret::new(fixtures::setup_bytes(runner, false)))
    })
    .expect("fixture setup");
    let Mode::Owned(owned) = setup.entry(SLOT, &dirs).expect("forge entry").mode else { panic!("owned fixture") };
    owned
}

fn prepare(runner: bool) -> Prepare {
    Prepare::new(owned(runner), place(), None)
}

fn outcome(host: &mut Script, machine: Prepare) -> (u8, String) {
    let result = prepare::run_with(host, &place(), machine)
        .map(|()| 0)
        .map_err(|why| words::Refused::new("prepare", Some(SLOT), words::prepare_failure(BINARY, why, &place())));
    let mut stderr = String::from_utf8(host.notes.clone()).expect("source notes are text");
    let exit = report(result, &mut stderr);
    (exit, stderr)
}

fn report(result: Result<u8, words::Refused>, stderr: &mut String) -> u8 {
    words::report(BINARY, result, false, |line| panic!("unexpected stdout: {line}"), |line| stderr.push_str(&line))
}

#[test]
fn prepare_recovers_after_admin_failure() {
    let mut host = Script { fail_admin: true, ..Script::default() };
    let failed = outcome(&mut host, prepare(false));
    let abandoned = state::decode(&host.files[Path::new(STATE)]).expect("record kept after failure");
    host.set(LEFTOVER, b"x");
    host.effects.clear();
    host.fail_admin = false;
    let retried = outcome(&mut host, prepare(false));
    let finished = state::decode(&host.files[Path::new(STATE)]).expect("finished record");

    assert_eq!(failed, (1, ADMIN_FAILURE.into()));
    assert!(!abandoned.prepared);
    assert_eq!(retried, (0, String::new()));
    assert!(host.effects.contains(&Effect::Clear));
    assert!(!host.files.contains_key(Path::new(LEFTOVER)));
    assert!(finished.prepared);
    assert_eq!(host.effects.last(), Some(&Effect::Write(STATE.into())), "prepared state written last");
}

#[test]
fn prepare_lock_rows() {
    let mut first = Script { runner_held: true, ..Script::default() };
    first.unfinished();
    let first_outcome = outcome(&mut first, prepare(true));
    let mut repeat = Script::default();
    prepare::run_with(&mut repeat, &place(), prepare(true)).expect("supply matching runner configuration");
    let before = repeat.files[Path::new(RUNNER_CONFIGURATION)].clone();
    repeat.effects.clear();
    repeat.runner_held = true;
    let repeat_outcome = outcome(&mut repeat, prepare(true));

    assert_eq!(first_outcome, (1, RUNNER_HELD.into()));
    assert_eq!(
        first.effects,
        [
            Effect::Lock(FORGEJO_LOCK.into()),
            Effect::Read(STATE.into()),
            Effect::Read(RUNNER_CONFIGURATION.into()),
            Effect::Lock(RUNNER_LOCK.into())
        ],
        "nothing is cleared or written before the runner lock is acquired"
    );
    assert_eq!(repeat_outcome, (0, String::new()));
    assert_eq!(
        repeat.effects.iter().filter(|effect| matches!(effect, Effect::Lock(_))).collect::<Vec<_>>(),
        [&Effect::Lock(FORGEJO_LOCK.into())]
    );
    assert!(!repeat.effects.contains(&Effect::Write(RUNNER_CONFIGURATION.into())));
    assert_eq!(repeat.files[Path::new(RUNNER_CONFIGURATION)], before);
}

#[test]
fn prepare_newer_leaves_inventory() {
    let mut host = Script::default();
    host.newer();
    let before = host.files.clone();
    assert_eq!(outcome(&mut host, prepare(true)), (1, NEWER.into()));
    assert_eq!(host.files, before, "state and app.ini intact, no backup");
    assert_eq!(
        host.effects,
        [Effect::Lock(FORGEJO_LOCK.into()), Effect::Read(STATE.into()), Effect::Read(RUNNER_CONFIGURATION.into())]
    );
}

#[test]
fn prepare_refuses_files_without_a_record() {
    let mut host = Script::default();
    host.set(APP_INI, b"unrecorded ini bytes");
    assert_eq!(outcome(&mut host, prepare(true)), (1, UNRECORDED.into()));
}

#[test]
fn checked_run_rows() {
    for &row in RUN_ROWS {
        let launch = plan::launch(&owned(row.configured_runner), &place(), Which::Forgejo).expect("forgejo launch");
        let (failure, order) = fixtures::refuse_run(&launch, row);
        let reason = words::run_failure(BINARY, failure, &launch, SLOT, &directories());
        let mut stderr = String::new();
        let exit = report(Err(words::Refused::new("run", Some(SLOT), reason)), &mut stderr);
        assert_eq!((exit, stderr.as_str()), (1, row.stderr), "{}", row.name);
        assert_eq!(order, ["lock", "read"], "{}", row.name);
    }
}

#[test]
fn checked_run_passes_its_lock_to_the_exec() {
    let launch = plan::launch(&owned(false), &place(), Which::Forgejo).expect("forgejo launch");
    let order = RefCell::new(Vec::new());
    let failure = run::run_with(
        || {
            run::checked_with(
                &launch,
                |path| {
                    assert_eq!(path, Path::new(FORGEJO_LOCK));
                    order.borrow_mut().push("lock");
                    Ok("acquired forgejo lock")
                },
                |path, _| {
                    assert_eq!(path, Path::new(STATE));
                    order.borrow_mut().push("read");
                    Ok(fixtures::record(FORGEJO_PIN, true, false))
                },
                |spec| {
                    order.borrow_mut().push("version");
                    assert_eq!(spec.program.as_ref(), FORGEJO.as_bytes());
                    assert_eq!(spec.arguments.iter().map(AsRef::as_ref).collect::<Vec<&[u8]>>(), [b"--version"]);
                    Ok(capture(0, b"forgejo version 17.0.1\r\nignored second line", b""))
                },
            )
        },
        |lock| {
            order.borrow_mut().push("exec");
            assert_eq!(lock, "acquired forgejo lock");
            io::ErrorKind::PermissionDenied.into()
        },
    );
    assert!(matches!(failure, run::Failed::Exec(why) if why.kind() == io::ErrorKind::PermissionDenied));
    assert_eq!(order.into_inner(), ["lock", "read", "version", "exec"]);
}

#[test]
fn prepare_source_outcomes() {
    const NOTE: &str = "forge source: discovery address not updated: ";
    const LIST: &[&str] = &["list"];
    const UPDATE: &[&str] = &["list", "update-oauth"];
    const VERB_AT: usize = 6; // Two path flags and their values, then `admin auth`.
    let listed = capture(0, SOURCE_LIST, b"");
    let expired = || Err(command::Error::Kernel("source deadline expired".into()));
    let spawn = || Err(command::Error::Spawn(skein_io::Error::Other));
    for (responses, expected_calls, tail) in [
        (vec![Ok(listed.clone()), Ok(capture(1, b"", b"refused\nsecond line\n"))], UPDATE, "refused second line "),
        (vec![expired()], LIST, "source deadline expired"),
        (vec![Ok(listed.clone()), expired()], UPDATE, "source deadline expired"),
        (vec![spawn()], LIST, "spawn: Other"),
        (vec![Ok(listed.clone()), spawn()], UPDATE, "spawn: Other"),
    ] {
        let mut host = Script::default();
        prepare::run_with(&mut host, &place(), prepare(false)).expect("supply completed preparation");
        host.source = responses.into();
        let machine = prepare(false).with_source_address(Some(DISCOVERY.into()), SOURCE_DEADLINE);
        let (exit, stderr) = outcome(&mut host, machine);
        let calls: Vec<_> = host
            .source_calls
            .iter()
            .map(|(args, deadline)| {
                assert_eq!(*deadline, SOURCE_DEADLINE);
                args[VERB_AT].as_str()
            })
            .collect();
        assert_eq!(exit, 0, "{tail}");
        assert_eq!(stderr, format!("{NOTE}{tail}\n"));
        assert_eq!(calls, expected_calls, "{tail}");
        assert!(host.source.is_empty(), "all supplied outcomes used");
    }
}
