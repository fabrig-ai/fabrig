use super::{
    Inputs, OwnedCommands, Path, Platform, Preparation, Routes, SetupFacts, SignIn, fetch_with, io,
    prepare_directory_with, prepare_fake_with, report_exit, setup_file_with, start_with_routes, test_fixtures,
};
use crate::configuration_tests::read_scripted;
use skein_io::kernel::{Kind, Stat};
use skein_lib::Secret;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    path::PathBuf,
};
use temper_config_shell::{File, Shape};
use temper_fetch_shell::testing::zstd;
use temper_forge_setup::{fetch as model, machine::BoxKind};
use temper_forge_setup_shell::{fetch::Failed, setup_file};
use temper_shell::directories::EnvironmentEntry;

#[path = "../../../../testing/forge_fetch.rs"]
mod forge_fetch;

const FAKE_DIRECTORY: &str = "/fixture/playground-1";
const FAKE_SETUP: &str = "/fixture/playground-1/setup";
const TOKEN: &str = "/fixture/playground-1/admin_token";
const USER: u32 = 1000;
const FAKE: &[u8] = br#"{"version":1,"kind":"fake","listen":{"socket":"127.0.0.1:3001"},"remote":"http://127.0.0.1:3001","git":"demo/bin/git"}"#;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Output {
    pub(crate) exit: u8,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: String,
}

impl Output {
    pub(crate) fn silent() -> Self {
        Self { exit: 0, stdout: Vec::new(), stderr: String::new() }
    }
}

pub(crate) struct Effects<'a> {
    pub(crate) stdout: &'a RefCell<Vec<u8>>,
    pub(crate) directory: &'a super::PrepareDirectory<'a>,
    pub(crate) fake: &'a super::PrepareFake<'a>,
    pub(crate) login: &'a super::Login<'a>,
    pub(crate) prepare: &'a super::PrepareOwned<'a>,
    pub(crate) sign_in: &'a SignIn<'a>,
    pub(crate) fetch: &'a super::FetchCommand<'a>,
    pub(crate) source: &'a super::Source<'a>,
}

impl<'a> Effects<'a> {
    pub(crate) fn recording(stdout: &'a RefCell<Vec<u8>>) -> Self {
        Self {
            stdout,
            directory: &|_, _| panic!("unexpected forge directory effect"),
            fake: &|_| panic!("unexpected fake token effect"),
            login: &|_| panic!("unexpected login lookup"),
            prepare: &|_, _, _| panic!("unexpected owned preparation"),
            sign_in: &|_, _, _, _, _| panic!("unexpected sign-in"),
            fetch: &|_, _, _| panic!("unexpected fetch"),
            source: &|_, _, _, _, _, _| panic!("unexpected source command"),
        }
    }
}

pub(crate) struct Invocation<'a> {
    pub(crate) number: u16,
    pub(crate) group: Option<BoxKind>,
    pub(crate) setup: &'a [u8],
    pub(crate) configuration: &'a [u8],
}

impl Invocation<'_> {
    pub(crate) fn call(
        &self,
        verb: &str,
        environment: &[EnvironmentEntry],
        roots: Option<PathBuf>,
        effects: &Effects<'_>,
    ) -> Output {
        let dirs = test_fixtures::directories();
        let mut args = vec![verb.into(), self.number.to_string().into()];
        if verb == "run" {
            args.push("forgejo".into());
        }
        let path = Path::new(test_fixtures::ROOT).join(format!("box/groups/{}", self.number));
        let groups = self.group.map(|kind| BTreeMap::from([(path.clone(), kind)]));
        let group_read = Cell::new(false);
        let group_stat = Cell::new(false);
        let setup_read = Cell::new(false);
        let kind = self.group.unwrap_or(BoxKind::Owned);
        let facts = SetupFacts {
            group_read: &|name, max| {
                assert_eq!((name, max), (path.as_path(), 16));
                assert!(!group_read.replace(true));
                groups.as_ref().and_then(|files| files.get(name)).map_or(Err(skein_shell::Unread::Absent), |kind| {
                    Ok(match kind {
                        BoxKind::Fake => b"fake\n".as_slice().into(),
                        BoxKind::Owned => b"owned\n".as_slice().into(),
                    })
                })
            },
            group_stat: &|name| {
                assert_eq!(name, path);
                assert!(group_read.get() && !group_stat.replace(true));
                assert!(groups.is_none(), "absence is only verified after an absent read");
                Err(io::ErrorKind::NotFound.into())
            },
            setup_present: &|name| {
                assert_eq!(verb, "prepare");
                assert_eq!(name, setup_file::path(&super::jig_directories(&dirs), self.number, kind));
                Ok(true)
            },
            setup_read: &|dirs, number, selected| {
                assert_eq!((number, selected), (self.number, kind));
                assert!(!setup_read.replace(true));
                setup_file::read_with(dirs, number, selected, |name, max| {
                    assert_eq!(name, setup_file::path(dirs, number, kind));
                    assert_eq!(max, 65_536);
                    Ok(Secret::new(self.setup.into()))
                })
            },
            setup_write: &|_, _, _| panic!("the setup and group tree stay unchanged"),
        };
        let inputs = Inputs {
            reader: crate::setup::check,
            listener_at: |_| panic!("no inherited listener"),
            setup: &|dirs, number, verb, flags| setup_file_with(dirs, number, verb, flags, &facts),
            configuration: &|_, shape| read_scripted(self.configuration, shape),
            sign_in: effects.sign_in,
        };
        let preparation = Preparation { directory: effects.directory, fake: effects.fake };
        let commands = OwnedCommands { login: effects.login, prepare: effects.prepare, run: &|_, _| panic!("no exec") };
        let routes = Routes { fetch: effects.fetch, source: effects.source };
        let answer = start_with_routes(
            "temper-box",
            Shape::EngineAlone,
            &dirs,
            &args,
            roots,
            environment,
            (&inputs, &preparation, &commands, &routes),
        );
        let mut stderr = String::new();
        let exit = report_exit(
            "temper-box",
            &args,
            answer,
            |line| {
                effects.stdout.borrow_mut().extend_from_slice(line.as_bytes());
                effects.stdout.borrow_mut().push(b'\n');
            },
            |line| stderr = line,
        );
        assert!(group_read.get() && setup_read.get());
        assert_eq!(group_stat.get(), groups.is_none());
        Output { exit, stdout: effects.stdout.take(), stderr }
    }
}

pub(crate) fn directory_stat(mode: u32) -> Stat {
    Stat { kind: Kind::Directory, size: 0, mode, owner: USER, links: 1 }
}

pub(crate) fn owned_directory(path: &Path, create: bool) -> Result<u32, String> {
    assert_eq!(path, Path::new(test_fixtures::DIRECTORY));
    assert!(create);
    prepare_directory_with(
        path,
        create,
        |_| true,
        |_, _| panic!("directory exists"),
        || USER,
        |_| Ok(directory_stat(0o700)),
    )
}

pub(crate) fn login(user: u32) -> Box<[u8]> {
    assert_eq!(user, USER);
    test_fixtures::LOGIN.into()
}

fn configuration() -> Box<[u8]> {
    temper_config_shell::render(
        &File { workers: Some(([127, 0, 0, 1], 7200).into()), ..File::default() },
        Shape::EngineAlone,
    )
}

#[derive(Debug, PartialEq, Eq)]
enum Node {
    Directory(u32),
    File(u32, Vec<u8>),
}

type Inventory = RefCell<BTreeMap<&'static str, Node>>;

fn fake_inventory(mode: u32) -> Inventory {
    RefCell::new(BTreeMap::from([
        (FAKE_DIRECTORY, Node::Directory(mode)),
        (FAKE_SETUP, Node::File(0o600, FAKE.to_vec())),
    ]))
}

fn fake_directory(path: &Path, create: bool, inventory: &Inventory, seen: &RefCell<Vec<&str>>) -> Result<u32, String> {
    assert!(create);
    assert_eq!(path, Path::new(FAKE_DIRECTORY));
    prepare_directory_with(
        path,
        create,
        |name| {
            assert_eq!(name, path);
            seen.borrow_mut().push("directory present");
            inventory.borrow().contains_key(FAKE_DIRECTORY)
        },
        |_, _| panic!("the setup's directory exists"),
        || {
            seen.borrow_mut().push("user");
            USER
        },
        |name| {
            assert_eq!(name, path);
            seen.borrow_mut().push("directory stat");
            let Node::Directory(mode) = inventory.borrow()[FAKE_DIRECTORY] else { panic!("a directory") };
            Ok(directory_stat(mode))
        },
    )
}

const DIRECTORY_EFFECTS: &[&str] = &["directory present", "user", "directory present", "directory stat"];

#[test]
fn fake_prepare_token_rows() {
    let config = configuration();
    let command = Invocation { number: 1, group: Some(BoxKind::Fake), setup: FAKE, configuration: &config };
    for existing in [None, Some(b"existing token\n".as_slice())] {
        let inventory = fake_inventory(0o700);
        if let Some(bytes) = existing {
            inventory.borrow_mut().insert(TOKEN, Node::File(0o600, bytes.to_vec()));
        }
        let seen = RefCell::new(Vec::new());
        let output = command.call(
            "prepare",
            &[],
            None,
            &Effects {
                directory: &|path, create| fake_directory(path, create, &inventory, &seen),
                fake: &|path| {
                    prepare_fake_with(
                        path,
                        |name| {
                            assert_eq!(name, Path::new(TOKEN));
                            seen.borrow_mut().push("token stat");
                            inventory.borrow().contains_key(TOKEN).then_some(()).ok_or(io::ErrorKind::NotFound.into())
                        },
                        || {
                            seen.borrow_mut().push("draw");
                            let random = std::array::from_fn::<_, 32, _>(|i| u8::try_from(i).expect("seed index"));
                            Ok(Secret::new(Box::new(random)))
                        },
                        |name, mode, bytes| {
                            assert_eq!(name, Path::new(TOKEN));
                            seen.borrow_mut().push("exclusive write");
                            assert_eq!(inventory.borrow_mut().insert(TOKEN, Node::File(mode, bytes.into())), None);
                            Ok(())
                        },
                    )
                },
                ..Effects::recording(&RefCell::new(Vec::new()))
            },
        );
        assert_eq!(output, Output::silent());
        let mut expected = DIRECTORY_EFFECTS.to_vec();
        expected.push("token stat");
        if existing.is_none() {
            expected.extend(["draw", "exclusive write"]);
        }
        assert_eq!(seen.into_inner(), expected);
        let token = existing.unwrap_or(b"000102030405060708090a0b0c0d0e0f10111213");
        assert_eq!(
            inventory.into_inner(),
            BTreeMap::from([
                (FAKE_DIRECTORY, Node::Directory(0o700)),
                (FAKE_SETUP, Node::File(0o600, FAKE.to_vec())),
                (TOKEN, Node::File(0o600, token.to_vec())),
            ])
        );
    }
}

#[test]
fn fake_verb_refusals() {
    let config = configuration();
    let command = Invocation { number: 1, group: Some(BoxKind::Fake), setup: FAKE, configuration: &config };
    for (verb, reason, expected) in [
        (
            "prepare",
            "/fixture/playground-1: must be a private directory of this user's, not reached through a link",
            DIRECTORY_EFFECTS,
        ),
        ("run", "it is fake, not owned", &[]),
    ] {
        let inventory = fake_inventory(0o755);
        let seen = RefCell::new(Vec::new());
        let output = command.call(
            verb,
            &[],
            None,
            &Effects {
                directory: &|path, create| fake_directory(path, create, &inventory, &seen),
                ..Effects::recording(&RefCell::new(Vec::new()))
            },
        );
        assert_eq!(
            output,
            Output { exit: 1, stdout: Vec::new(), stderr: format!("temper-box forge {verb}: forge 1: {reason}\n") }
        );
        assert_eq!(seen.into_inner(), expected);
        assert_eq!(inventory.into_inner(), fake_inventory(0o755).into_inner());
    }
}

#[test]
fn forge_fetch_rows() {
    use forge_fetch::{ROOTS, Row, Script};
    use skein_forge::forgejo::pin::{self, Binary};
    let config = configuration();
    let paths = test_fixtures::setup_bytes(false, false);
    let fetched = String::from_utf8(paths.to_vec()).expect("setup text").replace(
        "\"/programs/forgejo\"",
        "\"fetched\",\"mirror\":{\"forgejo\":\"https://mirror.example.invalid\",\"runner\":\"https://mirror.example.invalid\"}",
    );
    let pin = pin::digest(Binary::Forgejo, Platform::Linux, "amd64").expect("Linux Forgejo pin");
    for (name, group, setup, row, reason) in [
        ("fake", Some(BoxKind::Fake), FAKE, None, "it is fake, not owned"),
        ("paths", Some(BoxKind::Owned), paths.as_ref(), None, "neither of its binaries is \"fetched\""),
        ("digest", Some(BoxKind::Owned), fetched.as_bytes(), Some(Row::Digest), ""),
        ("present", Some(BoxKind::Owned), fetched.as_bytes(), Some(Row::Present), ""),
        ("present-no-group", None, fetched.as_bytes(), Some(Row::Present), ""),
        ("missing-roots", Some(BoxKind::Owned), fetched.as_bytes(), Some(Row::MissingRoots), ""),
    ] {
        let host = Script::new(row);
        let stdout = RefCell::new(Vec::new());
        let output = Invocation { number: 3, group, setup, configuration: &config }.call(
            "fetch",
            &[],
            Some(ROOTS.into()),
            &Effects {
                fetch: &|owned, bound, roots| {
                    fetch_with(owned, bound, Platform::Linux, "x86_64", |machine| {
                        temper_forge_setup_shell::fetch::run_with(
                            machine,
                            || host.roots(roots.as_deref().expect("SSL_CERT_FILE")),
                            |(), request| Ok(host.answer(request)),
                            &mut *stdout.borrow_mut(),
                        )
                    })
                },
                ..Effects::recording(&stdout)
            },
        );
        let diagnostic = row.map_or_else(|| reason.to_owned(), |row| row.diagnostic(pin));
        assert_eq!(
            output,
            Output {
                exit: 1,
                stdout: row.map_or_else(Vec::new, Row::stdout),
                stderr: format!("temper-box forge fetch: forge 3: {diagnostic}\n"),
            },
            "{name}"
        );
        host.check();
    }
}
