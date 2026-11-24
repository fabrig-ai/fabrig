use super::test_fixtures as fixtures;
use super::{
    Inputs, OwnedCommands, Path, Preparation, SetupFacts, io, prepare_directory_with, report_exit, setup_file_with,
    start_with_effects,
};
use crate::configuration_tests::read_scripted;
use fixtures::{
    ADMIN_FAILURE, APP_INI, DIRECTORY, LOGIN, NEWER, ROOT, RUN_ROWS, RUNNER_HELD, SECRETS, SLOT, SOURCE_LIST, Script,
    UNRECORDED, capture, directories,
};
use skein_io::kernel::{Kind, Stat};
use skein_lib::Secret;
use std::{
    cell::{Cell, RefCell},
    ffi::OsString,
    fmt::Write as _,
};
use temper_config_shell::{File, OpenId, OpenIdClient, Shape};
use temper_forge_setup::machine::BoxKind;
use temper_forge_setup_shell::{prepare, setup_file};

const USER: u32 = 1000;
const GROUP_BYTES: u32 = 16; // The group file's bounded read.
const SETUP_BYTES: u32 = 65536; // The immutable setup's bounded private read.
const GROUP: &str = "/fixture/box/groups/1";
const SETUP: &str = "/fixture/forge-1/setup";
const PREPARE: &[&str] = &["prepare", "1"];
const RUN: &[&str] = &["run", "1", "forgejo"];

#[derive(Debug, PartialEq, Eq)]
struct Output {
    exit: u8,
    stdout: String,
    stderr: String,
}

fn expected(exit: u8, stderr: &str) -> Output {
    Output { exit, stdout: String::new(), stderr: stderr.into() }
}

fn configuration(source_port: Option<u16>) -> Box<[u8]> {
    let mut file = File { workers: Some(([127, 0, 0, 1], 7200).into()), ..File::default() };
    if let Some(port) = source_port {
        file.owner = Some("ada@example.invalid".into());
        file.openid = Some(OpenId {
            key: Some("provider.key".into()),
            addresses: Some(vec![SLOT]),
            clients: vec![OpenIdClient {
                id: "forgejo".into(),
                secret: None,
                redirect: "http://127.0.0.1:3000/user/oauth2/temper/callback".into(),
            }],
        });
        file.listen = Some(([127, 0, 0, 1], port).into());
    }
    temper_config_shell::render(&file, Shape::EngineAlone)
}

fn directory() -> Stat {
    Stat { kind: Kind::Directory, size: 0, mode: 0o700, owner: USER, links: 1 }
}

struct Facts<'a> {
    group: Result<&'a [u8], skein_shell::Unread>,
    no_follow: Result<(), io::ErrorKind>,
    directory: Stat,
    configuration: &'a [u8],
    setup: &'a [u8],
}

impl<'a> Facts<'a> {
    fn owned(configuration: &'a [u8], setup: &'a [u8]) -> Self {
        Self { group: Ok(b"owned\n"), no_follow: Ok(()), directory: directory(), configuration, setup }
    }
}

fn command(facts: &Facts<'_>, arguments: &[&str], commands: &OwnedCommands<'_>) -> Output {
    let dirs = directories();
    let arguments = arguments.iter().map(OsString::from).collect::<Vec<_>>();
    let group_stat = Cell::new(false);
    let setup = SetupFacts {
        group_read: &|path, bound| {
            assert_eq!(path, Path::new(GROUP));
            assert_eq!(bound, GROUP_BYTES);
            facts.group.map(Box::from)
        },
        group_stat: &|path| {
            assert_eq!(path, Path::new(GROUP));
            assert!(
                matches!(facts.group, Err(skein_shell::Unread::Absent)),
                "only verify absence after an absent read"
            );
            assert!(!group_stat.replace(true), "one absence check");
            facts.no_follow.map_err(io::Error::from)
        },
        setup_present: &|path| {
            assert_eq!(path, Path::new(SETUP));
            Ok(true)
        },
        setup_read: &|dirs, number, kind| {
            assert_eq!((number, kind), (SLOT, BoxKind::Owned));
            setup_file::read_with(dirs, number, kind, |path, bound| {
                assert_eq!(path, Path::new(SETUP));
                assert_eq!(bound, SETUP_BYTES);
                Ok(Secret::new(facts.setup.into()))
            })
        },
        setup_write: &|_, _, _| panic!("an existing setup is never rewritten"),
    };
    let inputs = Inputs {
        reader: crate::setup::check,
        listener_at: |_| panic!("no inherited listener"),
        setup: &|dirs, number, verb, flags| setup_file_with(dirs, number, verb, flags, &setup),
        configuration: &|_, shape| read_scripted(facts.configuration, shape),
        sign_in: &|_, _, _, _, _| panic!("not a sign-in"),
    };
    let preparation = Preparation {
        directory: &|path, create| {
            assert_eq!(path, Path::new(DIRECTORY));
            prepare_directory_with(
                path,
                create,
                |given| {
                    assert_eq!(given, path);
                    true
                },
                |_, _| panic!("the setup's directory exists"),
                || USER,
                |given| {
                    assert_eq!(given, path);
                    Ok(facts.directory)
                },
            )
        },
        fake: &|_| panic!("the setup is owned"),
    };
    let result = start_with_effects(
        "temper-box",
        Shape::EngineAlone,
        &dirs,
        &arguments,
        None,
        &[],
        (&inputs, &preparation, commands),
    );
    assert_eq!(group_stat.get(), matches!(facts.group, Err(skein_shell::Unread::Absent)));
    let mut output = expected(0, "");
    output.exit = report_exit(
        "temper-box",
        &arguments,
        result,
        |line| output.stdout = format!("{line}\n"),
        |line| output.stderr = line,
    );
    output
}

fn login(user: u32) -> Box<[u8]> {
    assert_eq!(user, USER);
    LOGIN.into()
}

fn prepare(facts: &Facts<'_>, files: &mut Script) -> Output {
    let mut files = RefCell::new(files);
    let mut output = command(
        facts,
        PREPARE,
        &OwnedCommands {
            login: &|user| Ok(login(user)),
            prepare: &|_, place, machine| {
                assert_eq!(place, &fixtures::place());
                prepare::run_with(&mut **files.borrow_mut(), place, machine)
            },
            run: &|_, _| panic!("prepare does not exec"),
        },
    );
    let notes = std::str::from_utf8(&files.get_mut().notes).expect("source notes are text");
    output.stderr.insert_str(0, notes);
    output
}

#[test]
fn owned_prepare_without_group() {
    let config = configuration(None);
    let setup = fixtures::setup_bytes(true, false);
    let facts = Facts {
        group: Err(skein_shell::Unread::Absent),
        no_follow: Err(io::ErrorKind::NotFound),
        ..Facts::owned(&config, &setup)
    };
    let mut files = Script::default();
    assert_eq!(prepare(&facts, &mut files), expected(0, ""));
    assert!(files.files.contains_key(Path::new(APP_INI)));
    assert!(!files.files.keys().any(|path| path.starts_with(Path::new(ROOT).join("box"))));
}

#[test]
fn prepare_exit_lines() {
    let config = configuration(None);
    let setup = fixtures::setup_bytes(true, false);
    let facts = Facts::owned(&config, &setup);
    let mut files = Script { fail_admin: true, ..Script::default() };
    let failed = prepare(&facts, &mut files);
    files.fail_admin = false;
    let recovered = prepare(&facts, &mut files);
    files.runner_held = true;
    let repeat = prepare(&facts, &mut files);
    let mut held = Script { runner_held: true, ..Script::default() };
    held.unfinished();
    let first = prepare(&facts, &mut held);
    let mut unrecorded = Script::default();
    unrecorded.set(APP_INI, b"unrecorded ini bytes");
    let absent = prepare(&facts, &mut unrecorded);
    let mut newer = Script::default();
    newer.newer();
    let refusal = prepare(&facts, &mut newer);

    assert_eq!(failed, expected(1, ADMIN_FAILURE));
    assert_eq!(recovered, expected(0, ""));
    assert_eq!(repeat, expected(0, ""));
    assert_eq!(first, expected(1, RUNNER_HELD));
    assert_eq!(absent, expected(1, UNRECORDED));
    assert_eq!(refusal, expected(1, NEWER));
}

#[test]
fn prepare_directory_refusals() {
    let config = configuration(None);
    let setup = fixtures::setup_bytes(true, false);
    let expected = expected(
        1,
        "temper-box forge prepare: forge 1: /fixture/forge-1: must be a private directory of this user's, not reached through a link\n",
    );
    for metadata in [
        Stat { mode: 0o755, ..directory() },
        Stat { owner: 1001, ..directory() },
        Stat { kind: Kind::Symlink, ..directory() },
    ] {
        let facts = Facts { directory: metadata, ..Facts::owned(&config, &setup) };
        let commands = OwnedCommands {
            login: &|_| panic!("refused before login"),
            prepare: &|_, _, _| panic!("refused before preparation"),
            run: &|_, _| panic!("refused before run"),
        };
        assert_eq!(command(&facts, PREPARE, &commands), expected, "{metadata:?}");
    }
}

#[test]
fn run_refusal_rows() {
    let config = configuration(None);
    for &row in RUN_ROWS {
        let setup = fixtures::setup_bytes(row.configured_runner, false);
        let facts = Facts::owned(&config, &setup);
        let output = command(
            &facts,
            RUN,
            &OwnedCommands {
                login: &|user| Ok(login(user)),
                prepare: &|_, _, _| panic!("run does not prepare"),
                run: &|_, launch| fixtures::refuse_run(launch, row).0,
            },
        );
        assert_eq!(output, expected(1, row.stderr), "{}", row.name);
        for secret in SECRETS {
            for stream in [&output.stdout, &output.stderr] {
                assert_eq!(skein_lib::bytes::find(stream.as_bytes(), secret), None, "{}", row.name);
            }
        }
    }
}

#[test]
fn prepare_source_address_rows() {
    const PORTS: [u16; 2] = [31000, 31001];
    let setup = fixtures::setup_bytes(false, true);
    let config = configuration(Some(PORTS[0]));
    let mut files = Script::default();
    let first = prepare(&Facts::owned(&config, &setup), &mut files);
    let first_calls = files.source_calls.len();
    let mut argv = String::new();
    for port in PORTS {
        let config = configuration(Some(port));
        files.source = [Ok(capture(0, SOURCE_LIST, b"")), Ok(capture(0, b"", b""))].into();
        let repeat = prepare(&Facts::owned(&config, &setup), &mut files);
        let prefix = "--work-path\n/fixture/forge-1/forgejo\n--config\n/fixture/forge-1/forgejo/app.ini\nadmin\nauth\n";
        write!(argv, "{prefix}list\n{prefix}update-oauth\n--id\n27\n--use-custom-urls\nfalse\n--auto-discover-url\nhttp://127.0.0.1:{port}/.well-known/openid-configuration\n").expect("argv in text");
        let calls = files.source_calls.iter().flat_map(|(args, _)| args.iter().map(String::as_str)).collect::<Vec<_>>();
        assert_eq!(repeat, expected(0, ""), "{port}");
        assert!(files.source.is_empty(), "both source captures used");
        assert!(calls.into_iter().eq(argv.lines()), "expected exactly these source commands: {argv}");
    }
    assert_eq!(first, expected(0, ""));
    assert_eq!(first_calls, 0, "first preparation does not update a source");
}
