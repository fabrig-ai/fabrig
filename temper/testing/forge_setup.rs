//! In-memory files and command captures for the owned forge shell step tests.
use skein_lib::{Duration, Secret};
use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    io,
    path::{Path, PathBuf},
};
use temper_forge_setup::{
    place::Place,
    prepare::{Exit, Ran},
    run::Launch,
    spec::Spec,
    state::{self, Secrets, State},
};
use temper_forge_setup_shell::{command, prepare, process, run};

// Versions of the prepared installation supplied by the fixture.
pub const FORGEJO_PIN: &str = "17.0.1";
const RUNNER_PIN: &str = "13.2.1";

pub const SLOT: u16 = 1;
pub const ROOT: &str = "/fixture";
pub const DIRECTORY: &str = "/fixture/forge-1";
pub const APP_INI: &str = "/fixture/forge-1/forgejo/app.ini";
pub const STATE: &str = "/fixture/forge-1/state";
pub const RUNNER_CONFIGURATION: &str = "/fixture/forge-1/runner/config.yaml";
pub const FORGEJO_LOCK: &str = "/fixture/forge-1/forgejo.lock";
pub const RUNNER_LOCK: &str = "/fixture/forge-1/runner.lock";
pub const FORGEJO: &str = "/programs/forgejo";
pub const RUNNER: &str = "/programs/runner";
pub const LOGIN: &[u8] = b"sam";
pub const SOURCE_LIST: &[u8] = b"ID Name Type Enabled\n27 temper OAuth2 true\n";
pub const SECRETS: [&[u8]; 3] = [b"secret-key-never-print", b"internal-token-never-print", b"jwt-secret-never-print"];
pub const ADMIN_FAILURE: &str = "temper-box forge prepare: forge 1: CreateAdmin ended with status 1: broken\n";
pub const RUNNER_HELD: &str = "temper-box forge prepare: forge 1: runner is running or being prepared already\n";
pub const UNRECORDED: &str =
    "temper-box forge prepare: forge 1: /fixture/forge-1: Forgejo files have no preparation state\n";
pub const NEWER: &str = "temper-box forge prepare: forge 1: forgejo is prepared at 999.0.0, newer than this temper-box's pin 17.0.1: its migrations only go forward\n";

pub fn directories() -> temper_shell::directories::Directories {
    let args = ["--config", ROOT, "--data", ROOT, "--cache", ROOT].map(|word| Box::from(word.as_bytes()));
    temper_shell::directories::resolve(&args, &[], b"/").expect("absolute fixture directories").0
}

pub fn place() -> Place {
    Place { directory: DIRECTORY.as_bytes().into(), user: LOGIN.into() }
}

pub fn setup_bytes(runner: bool, source: bool) -> Box<[u8]> {
    let runner = if runner {
        format!(r#"{{"binary":"{RUNNER}","labels":["temper-host:host"],"capacity":1,"path":"/usr/bin:/bin"}}"#)
    } else {
        "null".into()
    };
    let source = if source { r#", "sign_in":"forgejo""# } else { "" };
    format!(
        r#"{{"version":1,"kind":"owned","listen":{{"socket":"127.0.0.1:3000"}},
          "remote":"http://127.0.0.1:3000","forgejo":"{FORGEJO}","runner":{runner},
          "person":{{"login":"ada","email":"ada@example.invalid"}}{source}}}"#
    )
    .into_bytes()
    .into_boxed_slice()
}

pub fn record(forgejo: &str, prepared: bool, runner: bool) -> Secret {
    let [secret_key, internal_token, jwt_secret] = SECRETS.map(|bytes| Secret::new(bytes.into()));
    state::encode(&State {
        forgejo: forgejo.as_bytes().into(),
        prepared,
        runner: runner.then(|| RUNNER_PIN.as_bytes().into()),
        secrets: Secrets { secret_key, internal_token, jwt_secret },
    })
}

#[derive(Clone, Copy)]
pub struct RunRow {
    pub name: &'static str,
    pub forgejo: &'static str,
    pub prepared: bool,
    pub recorded_runner: bool,
    pub configured_runner: bool,
    pub stderr: &'static str,
}

pub const RUN_ROWS: &[RunRow] = &[
    RunRow {
        name: "unfinished",
        forgejo: FORGEJO_PIN,
        prepared: false,
        recorded_runner: true,
        configured_runner: true,
        stderr: "temper-box forge run: forge 1: its preparation did not end: run temper-box forge prepare --config /fixture --data /fixture --cache /fixture 1\n",
    },
    RunRow {
        name: "newer",
        forgejo: "999.0.0",
        prepared: true,
        recorded_runner: true,
        configured_runner: true,
        stderr: "temper-box forge run: forge 1: prepared at 999.0.0 / 13.2.1, this temper-box pins 17.0.1 / 13.2.1: no command of this temper-box runs it\n",
    },
    RunRow {
        name: "unreadable",
        forgejo: "private-looking-garbage",
        prepared: true,
        recorded_runner: true,
        configured_runner: true,
        stderr: "temper-box forge run: forge 1: prepared at a version this temper-box cannot read: no command of this temper-box runs it\n",
    },
    RunRow {
        name: "runner added",
        forgejo: FORGEJO_PIN,
        prepared: true,
        recorded_runner: false,
        configured_runner: true,
        stderr: "temper-box forge run: forge 1: prepared without a runner, and the setup has one: run temper-box forge prepare --config /fixture --data /fixture --cache /fixture 1\n",
    },
    RunRow {
        name: "runner removed",
        forgejo: FORGEJO_PIN,
        prepared: true,
        recorded_runner: true,
        configured_runner: false,
        stderr: "temper-box forge run: forge 1: prepared with a runner, and the setup has none: run temper-box forge prepare --config /fixture --data /fixture --cache /fixture 1\n",
    },
];

pub fn refuse_run(launch: &Launch, row: RunRow) -> (run::Failed, Vec<&'static str>) {
    let steps = RefCell::new(Vec::new());
    let failure = run::run_with(
        || {
            run::checked_with(
                launch,
                |path| {
                    assert_eq!(path, Path::new(FORGEJO_LOCK));
                    steps.borrow_mut().push("lock");
                    Ok(())
                },
                |path, bound| {
                    assert_eq!(path, Path::new(STATE));
                    assert_eq!(bound, PRIVATE_READ_BYTES);
                    steps.borrow_mut().push("read");
                    Ok(record(row.forgejo, row.prepared, row.recorded_runner))
                },
                |_| panic!("{}: inadmissible state must not capture a version", row.name),
            )
        },
        |()| panic!("{}: inadmissible state must not exec", row.name),
    );
    (failure, steps.into_inner())
}

// The shell's private state and runner settings bounds, and the preparation's seed width.
const PRIVATE_READ_BYTES: u32 = 1 << 20;
const SEED_BYTES: usize = 32;

#[derive(Debug, PartialEq, Eq)]
pub enum Effect {
    Lock(PathBuf),
    Read(PathBuf),
    Clear,
    Layout,
    Write(PathBuf),
}

#[derive(Default)]
pub struct Script {
    pub files: BTreeMap<PathBuf, Vec<u8>>,
    pub effects: Vec<Effect>,
    pub runner_held: bool,
    pub fail_admin: bool,
    pub source: VecDeque<Result<Ran, command::Error>>,
    pub source_calls: Vec<(Vec<String>, Duration)>,
    pub notes: Vec<u8>,
}

impl Script {
    pub fn set(&mut self, path: &str, bytes: &[u8]) {
        self.files.insert(path.into(), bytes.into());
    }

    pub fn unfinished(&mut self) {
        self.set(STATE, record(FORGEJO_PIN, false, true).expose());
        self.set(APP_INI, b"unfinished ini bytes");
    }

    pub fn newer(&mut self) {
        self.set(STATE, record("999.0.0", true, true).expose());
        self.set(APP_INI, b"newer ini bytes");
    }
}

pub fn capture(exit: u8, stdout: &[u8], stderr: &[u8]) -> Ran {
    Ran { exit: Exit::Code(exit), stdout: Secret::new(stdout.into()), stderr: Secret::new(stderr.into()) }
}

impl prepare::Host for Script {
    type Lock = ();

    fn lock(&mut self, path: &Path) -> Result<Self::Lock, process::Refused> {
        self.effects.push(Effect::Lock(path.into()));
        if path == Path::new(RUNNER_LOCK) && self.runner_held {
            return Err(process::Refused::Held);
        }
        assert!([Path::new(FORGEJO_LOCK), Path::new(RUNNER_LOCK)].contains(&path));
        Ok(())
    }
    fn read_private(&mut self, path: &Path, bound: u32) -> Result<Secret, skein_shell::Unread> {
        assert_eq!(bound, PRIVATE_READ_BYTES);
        assert!([Path::new(STATE), Path::new(RUNNER_CONFIGURATION)].contains(&path));
        self.effects.push(Effect::Read(path.into()));
        self.files.get(path).map(|bytes| Secret::new(bytes.as_slice().into())).ok_or(skein_shell::Unread::Absent)
    }
    fn exists(&mut self, path: &Path) -> bool {
        self.files.contains_key(path)
    }
    fn present(&mut self, place: &Place) -> io::Result<bool> {
        assert_eq!(place.directory.as_ref(), DIRECTORY.as_bytes());
        Ok(self.files.keys().any(|path| path.starts_with("/fixture/forge-1/forgejo")))
    }
    fn clear(&mut self, place: &Place) -> io::Result<()> {
        assert_eq!(place.directory.as_ref(), DIRECTORY.as_bytes());
        self.effects.push(Effect::Clear);
        self.files.retain(|path, _| !path.starts_with("/fixture/forge-1/forgejo"));
        Ok(())
    }
    fn layout(&mut self, place: &Place) -> io::Result<()> {
        assert_eq!(place.directory.as_ref(), DIRECTORY.as_bytes());
        self.effects.push(Effect::Layout);
        Ok(())
    }
    fn write(&mut self, path: &Path, contents: &[u8]) -> io::Result<()> {
        self.effects.push(Effect::Write(path.into()));
        self.files.insert(path.into(), contents.to_vec());
        Ok(())
    }
    fn backup_exists(&mut self, path: &Path) -> io::Result<bool> {
        panic!("none of these preparations looks for a backup: {}", path.display())
    }
    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        panic!("none of these preparations renames a backup: {} to {}", from.display(), to.display())
    }
    fn draw(&mut self) -> Result<Secret, i32> {
        Ok(Secret::new(Box::new([0x37; SEED_BYTES])))
    }
    fn capture(&mut self, spec: &Spec, within: Option<Duration>) -> Result<Ran, command::Error> {
        let args: Vec<_> = spec.arguments.iter().map(|arg| std::str::from_utf8(arg).expect("fixture argv")).collect();
        if let Some(within) = within {
            self.source_calls.push((args.iter().map(|arg| (*arg).to_owned()).collect(), within));
            return self.source.pop_front().expect("one supplied outcome per source command");
        }
        let output = match args.get(4..).unwrap_or(&args) {
            ["--version"] => match spec.program.as_ref() {
                b"/programs/forgejo" => format!("forgejo version {FORGEJO_PIN}\r\nunused line").into_bytes(),
                b"/programs/runner" => format!("forgejo-runner version v{RUNNER_PIN}\n").into_bytes(),
                program => panic!("no capture for {program:?}"),
            },
            ["migrate"] => Vec::new(),
            ["admin", "user", "create", "--username", "temper-admin", ..] => {
                if self.fail_admin {
                    return Ok(capture(1, b"", b"broken\n"));
                }
                Vec::new()
            }
            ["admin", "user", "create", "--username", "ada", ..] => {
                b"generated random password is 'fixture-person-password'\n".to_vec()
            }
            ["admin", "user", "generate-access-token", ..] => b"fixture-token\n".to_vec(),
            ["forgejo-cli", "actions", "register", ..] => b"fixture-runner-uuid\n".to_vec(),
            args => panic!("unexpected captured command {args:?}"),
        };
        Ok(capture(0, &output, b""))
    }
    fn note(&mut self, line: &[u8]) -> io::Result<()> {
        self.notes.extend_from_slice(line);
        Ok(())
    }
}
