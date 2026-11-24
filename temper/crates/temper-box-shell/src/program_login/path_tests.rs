use super::*;
use crate::tests::{
    configuration::file,
    directory_facts::{Disk, USER},
};
use skein_fake_machine::Item;
use skein_io::kernel::Kind;
use smith_agent_document::program::prepare_with;
use std::{cell::RefCell, io};
use temper_config_shell::{Facts, Shape};
use temper_worker_shell::{
    agent::program_directory_refusal,
    program_login::{exec_with, from_machine_with},
};

const PREFIX: &str = "temper-box program-login: ";
const ROOT: &str = "/fixture";
const PROGRAMS: &str = "/fixture/programs";
const SOURCE: &str = "/fixture/programs/source";
const CLAUDE: &str = "/fixture/program-bin/claude";

#[derive(Debug, PartialEq, Eq)]
enum Call {
    Read,
    LocalSetting,
    Adapter,
    CurrentExe,
    Executable(PathBuf),
    Inspect(PathBuf),
    Make(PathBuf, bool, u32),
    Exec,
}

fn disk(mut items: Vec<Item>) -> Disk {
    items.extend([Item::directory(b"program-bin"), Item::file(b"program-bin/claude", b"program").mode(0o700)]);
    Disk::new(&items)
}

fn configuration(text: &[u8]) -> Result<Read, String> {
    let temper_config_shell::Read { file, document, keys } = temper_config_shell::read_with(
        Shape::Standalone,
        || Err(io::ErrorKind::NotFound.into()),
        |max| {
            if text.len() > usize::try_from(max).expect("configuration bound") {
                Err(skein_shell::Unread::TooLarge { max })
            } else {
                Ok(text.into())
            }
        },
        || Ok(Facts { login: "ada".into(), hostname: "fixture".into(), provider_key: false }),
    )
    .map_err(|why| why.to_string())?;
    Ok(Read { file, document, keys })
}

fn run_login(disk: &Disk, text: &[u8], name: &str) -> (String, Vec<Call>) {
    let directories = crate::tests::configuration::directories(Path::new(ROOT));
    let calls = RefCell::new(Vec::new());
    let record = |call| calls.borrow_mut().push(call);
    let executable = |path: &Path| {
        record(Call::Executable(path.into()));
        disk.stat(path).is_ok_and(|stat| stat.kind == Kind::File && stat.mode & 0o111 != 0)
    };
    let result = run_with(
        || {
            command_with(
                &directories,
                OsStr::new(name),
                [("PATH".into(), "/fixture/program-bin".into())],
                || {
                    record(Call::Read);
                    configuration(text)
                },
                executable,
                || {
                    record(Call::LocalSetting);
                    4
                },
                || {
                    record(Call::Adapter);
                    Ok(None)
                },
                || {
                    record(Call::CurrentExe);
                    Ok("/fixture/bin/temper-box".into())
                },
                |part, name, environment, agent| {
                    assert_eq!(agent, Path::new("/fixture/bin/temper-box"));
                    from_machine_with(part, &directories, name, environment, executable, |programs, program, name| {
                        assert_eq!(programs, Path::new(PROGRAMS));
                        assert_eq!(program, Path::new(CLAUDE), "PATH supplies Claude without placing a launcher");
                        prepare_with(
                            programs,
                            name,
                            USER,
                            |path| {
                                record(Call::Inspect(path.into()));
                                disk.stat(path)
                            },
                            |path, parents, mode| {
                                record(Call::Make(path.into(), parents, mode));
                                disk.make(path, parents, mode)
                            },
                        )
                        .map_err(|why| program_directory_refusal(&why))
                    })
                },
            )
        },
        |command| {
            exec_with(command, |command| {
                record(Call::Exec);
                assert_eq!(command.get_program(), OsStr::new(CLAUDE));
                assert_eq!(command.get_args().collect::<Vec<_>>(), ["auth", "login"]);
                for path in [PROGRAMS, SOURCE] {
                    let stat = disk.stat(Path::new(path)).expect("private directory before exec");
                    assert_eq!((stat.kind, stat.mode, stat.owner), (Kind::Directory, 0o700, USER), "{path}");
                }
                io::ErrorKind::NotFound.into()
            })
        },
    );
    (result.expect_err("login refusal"), calls.into_inner())
}

fn no_deployment_resources(disk: &Disk) {
    for name in [".claude", "state", "workspaces", "worker.secret", "secrets", "accounts", "charters"] {
        disk.assert_absent(&Path::new(ROOT).join(name));
    }
}

fn one_refusal(why: &str, row: &str) {
    assert!(why.starts_with(PREFIX), "{row}: {why}");
    assert_eq!(why.lines().count(), 1, "{row}: {why}");
    assert!(!why.contains("exec:"), "{row}: {why}");
}

fn no_make_or_exec(calls: &[Call]) {
    assert!(!calls.iter().any(|call| matches!(call, Call::Make(..) | Call::Exec)), "no effects: {calls:?}");
}

#[test]
fn invalid_login_input_refuses_before_preparing_programs() {
    let valid = temper_config_shell::render(&file(), Shape::Standalone);
    let oversized = vec![b' '; 1_048_577];
    for (row, text, name, message) in [
        ("malformed", b"[".as_slice(), "source", "temper.toml:"),
        ("wrong-type", b"owner = 42\n", "source", "owner:"),
        ("oversized", oversized.as_slice(), "source", "past 1048576 bytes"),
        ("bad-name", valid.as_ref(), "../escape", "source name must be 1 to 32 of a-z, 0-9 and -"),
    ] {
        let disk = disk(vec![]);
        let (why, calls) = run_login(&disk, text, name);
        one_refusal(&why, row);
        assert!(why.contains(message), "{row}: {why}");
        no_make_or_exec(&calls);
        if row == "bad-name" {
            assert_eq!(calls, [Call::Read, Call::LocalSetting, Call::Adapter, Call::CurrentExe]);
        } else {
            assert_eq!(calls, [Call::Read]);
        }
        for path in [PROGRAMS, "/fixture/escape", "/fixture/missing"] {
            disk.assert_absent(Path::new(path));
        }
        no_deployment_resources(&disk);
    }
}

#[test]
fn login_refuses_either_unsafe_directory_name_before_making_anything() {
    let valid = temper_config_shell::render(&file(), Shape::Standalone);
    for (row, item, reason) in [
        ("program-link", Item::link(b"programs", b"missing"), "symbolic link"),
        ("program-mode", Item::directory(b"programs").mode(0o750), "nonprivate mode bits 0o50"),
        ("program-file", Item::file(b"programs", b"file"), "not a directory"),
        ("source-link", Item::link(b"programs/source", b"../peer"), "symbolic link"),
        ("source-mode", Item::directory(b"programs/source").mode(0o750), "nonprivate mode bits 0o50"),
        ("source-file", Item::file(b"programs/source", b"file"), "not a directory"),
        ("source-dangling", Item::link(b"programs/source", b"../missing"), "symbolic link"),
    ] {
        let source = row.starts_with("source-");
        let mut items = Vec::new();
        if source {
            items.push(Item::directory(b"programs").mode(0o700));
        }
        if row == "source-link" {
            items.push(Item::directory(b"peer").mode(0o700));
        }
        items.push(item);
        let disk = disk(items);
        let target = if source { SOURCE } else { PROGRAMS };
        let (why, calls) = run_login(&disk, valid.as_ref(), "source");
        one_refusal(&why, row);
        assert!(why.ends_with(&format!("programs.claude_code.directory: {target}: {reason}\n")), "{row}: {why}");
        no_make_or_exec(&calls);
        let mut expected = vec![Call::Inspect(PROGRAMS.into())];
        if source {
            expected.push(Call::Inspect(SOURCE.into()));
        } else {
            assert!(disk.stat(Path::new(SOURCE)).is_err(), "missing source stays absent");
        }
        assert_eq!(&calls[5..], expected);
        disk.assert_absent(Path::new("/fixture/missing"));
        no_deployment_resources(&disk);
    }
}

#[test]
fn login_prepares_and_rechecks_private_directories_before_exec() {
    let valid = temper_config_shell::render(&file(), Shape::Standalone);
    for present in [false, true] {
        let items = if present { vec![Item::directory(b"programs").mode(0o700)] } else { vec![] };
        let disk = disk(items);
        let (why, calls) = run_login(&disk, valid.as_ref(), "source");
        assert_eq!(why, format!("{PREFIX}exec: {}\n", io::Error::from(io::ErrorKind::NotFound)));
        let mut expected = vec![
            Call::Read,
            Call::LocalSetting,
            Call::Adapter,
            Call::CurrentExe,
            Call::Executable(CLAUDE.into()),
            Call::Inspect(PROGRAMS.into()),
            Call::Inspect(SOURCE.into()),
        ];
        if !present {
            expected.push(Call::Make(PROGRAMS.into(), true, 0o700));
        }
        expected.extend([
            Call::Inspect(PROGRAMS.into()),
            Call::Make(SOURCE.into(), false, 0o700),
            Call::Inspect(SOURCE.into()),
            Call::Exec,
        ]);
        assert_eq!(calls, expected);
        no_deployment_resources(&disk);
    }
}
