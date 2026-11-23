use super::*;
use std::{cell::RefCell, collections::BTreeMap, io::ErrorKind, os::unix::ffi::OsStringExt};

use skein_scratch::Scratch;

const USER: u32 = 1000;

#[derive(Debug, PartialEq, Eq)]
enum Call {
    Inspect(PathBuf),
    Make { path: PathBuf, parents: bool, mode: u32 },
}

#[derive(Default)]
struct Disk {
    entries: RefCell<BTreeMap<PathBuf, Entry>>,
    calls: RefCell<Vec<Call>>,
}

struct Entry {
    facts: Facts,
    bytes: &'static [u8],
}

impl Disk {
    fn lay(&self, path: &Path, kind: Kind, mode: u32, bytes: &'static [u8]) {
        self.entries.borrow_mut().insert(path.into(), Entry { facts: Facts { kind, owner: USER, mode }, bytes });
    }

    fn inspect(&self, path: &Path) -> std::io::Result<Facts> {
        self.calls.borrow_mut().push(Call::Inspect(path.into()));
        self.entries.borrow().get(path).map(|entry| entry.facts).ok_or_else(|| ErrorKind::NotFound.into())
    }

    fn prepare(&self, programs: &Path, source: &str) -> Result<PathBuf, Refusal> {
        prepare_with(
            programs,
            source,
            || USER,
            |path| self.inspect(path),
            |path, parents, mode| {
                self.calls.borrow_mut().push(Call::Make { path: path.into(), parents, mode });
                if self.entries.borrow().contains_key(path) {
                    return Err(ErrorKind::AlreadyExists.into());
                }
                if !parents && !self.entries.borrow().contains_key(path.parent().expect("absolute path")) {
                    return Err(ErrorKind::NotFound.into());
                }
                self.lay(path, Kind::Directory, mode, b"");
                Ok(())
            },
        )
    }

    fn calls(&self) -> Vec<Call> {
        self.calls.take()
    }
}

fn assert_unsafe(result: Result<PathBuf, Refusal>, path: &Path, why: Unsafe) {
    match result {
        Err(Refusal::Unsafe { path: refused, found }) => {
            assert_eq!(refused.as_os_str(), path.as_os_str(), "refusal names the supplied path");
            assert_eq!(found, why, "kind of refusal");
        }
        other => panic!("expected {why:?} for {}, got {other:?}", path.display()),
    }
}

#[test]
fn preparing_makes_both_directories_private() {
    let disk = Disk::default();
    let programs = Path::new("/fixture/missing/programs");
    let configuration = programs.join("source-1");
    assert!(
        !private_directory_with(programs, || USER, &mut |path| disk.inspect(path)).expect("absent directory"),
        "programs directory starts absent"
    );
    disk.calls();
    assert_eq!(disk.prepare(programs, "source-1").expect("prepare directories"), configuration, "source path");
    assert_eq!(
        disk.calls(),
        [
            Call::Inspect(programs.into()),
            Call::Inspect(configuration.clone()),
            Call::Make { path: programs.into(), parents: true, mode: 0o700 },
            Call::Inspect(programs.into()),
            Call::Make { path: configuration.clone(), parents: false, mode: 0o700 },
            Call::Inspect(configuration.clone()),
        ],
        "create private directories and recheck each"
    );
    let entries = disk.entries.borrow();
    for path in [programs, &configuration] {
        let facts = entries.get(path).expect("made directory").facts;
        assert!(matches!(facts.kind, Kind::Directory), "created a directory");
        assert_eq!(facts.mode, 0o700, "private mode");
        assert_eq!(facts.owner, USER, "owned by the effective user");
    }
}

#[test]
fn preparing_keeps_an_existing_private_directory() {
    let disk = Disk::default();
    let programs = Path::new("/fixture/programs");
    let configuration = disk.prepare(programs, "source").expect("prepare once");
    let kept = configuration.join("kept");
    disk.lay(&kept, Kind::Other, 0o600, b"kept");
    disk.calls();
    assert_eq!(disk.prepare(programs, "source").expect("prepare again"), configuration, "same source path");
    assert_eq!(
        disk.calls(),
        [programs, &configuration, programs, &configuration].map(|path| Call::Inspect(path.into())),
        "no creation or replacement on the second prepare"
    );
    assert_eq!(disk.entries.borrow().get(&kept).expect("source data").bytes, b"kept", "existing data untouched");
}

#[test]
fn preparing_refuses_a_directory_open_to_others() {
    let disk = Disk::default();
    let programs = Path::new("/fixture/programs");
    disk.lay(programs, Kind::Directory, 0o755, b"");
    assert_unsafe(disk.prepare(programs, "source"), programs, Unsafe::Mode(0o055));
    assert_eq!(disk.calls(), [Call::Inspect(programs.into())], "nothing made after refusing the parent");
    assert_eq!(disk.entries.borrow().len(), 1, "programs directory still empty");
}

#[test]
fn preparing_refuses_a_link() {
    let disk = Disk::default();
    let programs = Path::new("/fixture/programs");
    let target = Path::new("/fixture/target/source");
    disk.lay(target, Kind::Directory, 0o700, b"");
    disk.lay(programs, Kind::Link, 0o777, target.as_os_str().as_bytes());
    for path in [programs, &programs.join(""), &programs.join(".")] {
        assert_unsafe(disk.prepare(path, "source"), path, Unsafe::Link);
        let calls = disk.calls();
        let [Call::Inspect(inspected)] = calls.as_slice() else {
            panic!("expected only a no-follow inspection, got {calls:?}");
        };
        assert_eq!(inspected.as_os_str(), programs.as_os_str(), "lookup strips trailing separators and dots");
    }
    assert_eq!(disk.entries.borrow().len(), 2, "link target still empty");
}

#[test]
fn preparing_refuses_a_file() {
    let disk = Disk::default();
    let programs = Path::new("/fixture/programs");
    disk.lay(programs, Kind::Other, 0o600, b"file");
    assert_unsafe(disk.prepare(programs, "source"), programs, Unsafe::NotADirectory);
    assert_eq!(disk.calls(), [Call::Inspect(programs.into())], "nothing made after refusing a file");
    assert_eq!(disk.entries.borrow().get(programs).expect("untouched file").bytes, b"file", "bytes preserved");
}

#[test]
fn preparing_checks_the_existing_source_directory() {
    let disk = Disk::default();
    let programs = Path::new("/fixture/programs");
    let configuration = disk.prepare(programs, "source").expect("source directory");
    disk.calls();
    for (kind, mode, why) in [(Kind::Directory, 0o755, Unsafe::Mode(0o055)), (Kind::Link, 0o777, Unsafe::Link)] {
        disk.lay(&configuration, kind, mode, b"");
        assert_unsafe(disk.prepare(programs, "source"), &configuration, why);
        assert_eq!(
            disk.calls(),
            [Call::Inspect(programs.into()), Call::Inspect(configuration.clone())],
            "both directories checked before making anything"
        );
    }
}

#[test]
fn preparing_refuses_a_bad_name() {
    let scratch = Scratch::new("program-name");
    let programs = scratch.path().join("programs");
    for name in ["", &"a".repeat(33), "A", "..", "a/b"] {
        assert!(matches!(prepare(&programs, name), Err(Refusal::Name)), "bad source {name:?}");
        assert!(!programs.exists(), "bad name makes nothing");
    }
    let longest = "a".repeat(32);
    assert_eq!(prepare(&programs, &longest).expect("maximum source name"), programs.join(longest));
}

#[test]
fn preparing_refuses_a_relative_directory() {
    assert!(
        matches!(prepare(Path::new("relative"), "source"), Err(Refusal::Relative(_))),
        "relative directory refused"
    );
    assert!(matches!(prepare(Path::new("relative"), ".."), Err(Refusal::Name)), "name checked first");
}

#[test]
fn sign_in_builds_auth_login_in_the_filtered_source_environment() {
    let program = Path::new("/usr/bin/claude");
    let configuration = Path::new("/private/programs/source");
    let home_and_key = [("HOME", "/private/home"), ("ANTHROPIC_API_KEY", "synthetic-key")];
    let with_overrides = [
        ("HOME", "/private/home"),
        ("XDG_CONFIG_HOME", "/private/home/.config"),
        ("ANTHROPIC_API_KEY", "key"),
        ("ANTHROPIC_CONFIG_DIR", "/home/u/.config/anthropic"),
        ("ANTHROPIC_AUTH_TOKEN", "token"),
        ("CLAUDE_CODE_OAUTH_TOKEN", "token"),
        ("CLAUDE_CODE_USE_BEDROCK", "1"),
        ("AWS_BEARER_TOKEN_BEDROCK", "token"),
        ("CLAUDE_CONFIG_DIR", "/person/config"),
    ];
    for (base, xdg) in [(&home_and_key[..], None), (&with_overrides[..], Some("/private/home/.config"))] {
        let command = sign_in(program, configuration, base.iter().map(|&(name, value)| (name.into(), value.into())));
        assert_eq!(command.get_program(), program, "sign-in executable");
        assert_eq!(command.get_args().collect::<Vec<_>>(), ["auth", "login"], "sign-in arguments");
        let anthropic = configuration.join("anthropic");
        let mut expected = vec![
            (OsStr::new("ANTHROPIC_CONFIG_DIR"), Some(anthropic.as_os_str())),
            (OsStr::new("CLAUDE_CONFIG_DIR"), Some(configuration.as_os_str())),
            (OsStr::new("HOME"), Some(OsStr::new("/private/home"))),
        ];
        if let Some(value) = xdg {
            expected.push((OsStr::new("XDG_CONFIG_HOME"), Some(OsStr::new(value))));
        }
        assert_eq!(
            command.get_envs().collect::<Vec<_>>(),
            expected,
            "only source configuration and allowed environment"
        );
    }
}

#[test]
fn the_sign_in_s_environment_drops_a_named_profile() {
    let base = [("ANTHROPIC_PROFILE", "work"), ("HOME", "/h")];
    let command = sign_in(
        Path::new("/usr/bin/claude"),
        Path::new("/programs/a"),
        base.map(|(name, value)| (name.into(), value.into())),
    );
    assert_eq!(
        command.get_envs().collect::<Vec<_>>(),
        [
            (OsStr::new("ANTHROPIC_CONFIG_DIR"), Some(OsStr::new("/programs/a/anthropic"))),
            (OsStr::new("CLAUDE_CONFIG_DIR"), Some(OsStr::new("/programs/a"))),
            (OsStr::new("HOME"), Some(OsStr::new("/h"))),
        ]
    );
}

#[test]
fn the_sign_in_s_environment_drops_federation_variables() {
    let rule = ("ANTHROPIC_FEDERATION_RULE_ID", "rule");
    let organization = ("ANTHROPIC_ORGANIZATION_ID", "organization");
    for credentials in [std::slice::from_ref(&rule), std::slice::from_ref(&organization), &[rule, organization]] {
        let base = credentials.iter().copied().chain([("HOME", "/h")]);
        let command = sign_in(
            Path::new("/usr/bin/claude"),
            Path::new("/programs/a"),
            base.map(|(name, value)| (name.into(), value.into())),
        );
        assert_eq!(
            command.get_envs().collect::<Vec<_>>(),
            [
                (OsStr::new("ANTHROPIC_CONFIG_DIR"), Some(OsStr::new("/programs/a/anthropic"))),
                (OsStr::new("CLAUDE_CONFIG_DIR"), Some(OsStr::new("/programs/a"))),
                (OsStr::new("HOME"), Some(OsStr::new("/h"))),
            ]
        );
    }
}

#[test]
fn the_sign_in_s_environment_drops_an_endpoint() {
    let base = [("ANTHROPIC_BASE_URL", "https://example.com"), ("HOME", "/h")];
    let command = sign_in(
        Path::new("/usr/bin/claude"),
        Path::new("/programs/a"),
        base.map(|(name, value)| (name.into(), value.into())),
    );
    assert_eq!(
        command.get_envs().collect::<Vec<_>>(),
        [
            (OsStr::new("ANTHROPIC_CONFIG_DIR"), Some(OsStr::new("/programs/a/anthropic"))),
            (OsStr::new("CLAUDE_CONFIG_DIR"), Some(OsStr::new("/programs/a"))),
            (OsStr::new("HOME"), Some(OsStr::new("/h"))),
        ]
    );
}

#[test]
fn the_terminal_environment_has_no_hosted_spawn_bound() {
    let mut base: Vec<_> = (0..100).map(|index| (format!("VARIABLE_{index}").into(), "x".repeat(100).into())).collect();
    base.push((OsString::from_vec(vec![b'N', 0xff]), OsString::from_vec(vec![0xfe])));
    let command = sign_in(Path::new("/claude"), Path::new("/programs/source"), base);
    assert_eq!(command.get_envs().count(), 103);
}
