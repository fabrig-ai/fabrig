//! The box's fake git invocation on the hosted disk and wire (temper/docs/design/testing/box/README.md, 3.13).
mod wire;

use skein_lib::Secret;
use temper_box_shell::fake::git;
use temper_fake::{credential, forge};
use temper_worker_domain_checkout::git::Kind;
use temper_worker_protocol_git::{
    Identity,
    invocation::{self, Command, Format},
};
use wire::{Endpoint, Hosted};

const SEED: u64 = 23;
const WORK: &[u8] = b"/work";
const POOL: &[u8] = b"/work/pool";
const WIDGET: &[u8] = b"/work/widget";
const ADMIN: &[u8] = b"temper-admin";
const VERSION: &str = "git version 2.47.3\n";
// The seed supplies commit 1; commit-tree stores commit 2.
const BASE: &[u8] = b"910a2dec89025cc1000000000000000100000000";
const STORED: &str = "975835de1c9756ce000000000000000200000000";
// The FNV tree id of forge::FILES plus new.txt.
const TREE: &str = "a48b10c7a2c77a727ec978a5592769d800000000";
const MESSAGE: &[u8] = b"made by the process\n";
const EDIT: (&[u8], &[u8]) = (b"new.txt", b"new\n");
const ALTERNATES: (&[u8], &[u8]) = (b".git/objects/info/alternates", b"../../../pool/objects\n");
const NO_ROUTE: Endpoint = Endpoint::Reply { status: "404 Not Found", body: b"" };
// The fake LLM's unauthorized answer is not a named Git route error.
const REFUSED: Endpoint = Endpoint::Reply {
    status: "401 Unauthorized",
    body: br#"{"error":{"code":"invalid_api_key","message":"invalid_api_key"}}"#,
};

struct Step {
    command: Command,
    kind: Option<Kind>,
    at: &'static [u8],
    input: &'static [u8],
    edit: Option<(&'static [u8], &'static [u8])>,
    extra: Option<&'static [u8]>,
    exit: u8,
    stdout: String,
    stderr: &'static str,
}

fn step(command: Command, kind: Option<Kind>, at: &'static [u8]) -> Step {
    Step { command, kind, at, input: b"", edit: None, extra: None, exit: 0, stdout: String::new(), stderr: "" }
}

fn version() -> Step {
    Step { stdout: VERSION.into(), ..step(Command::Version, None, WORK) }
}

fn store(remote: &[u8]) -> Step {
    step(Command::Store { remote: remote.into(), directory: b"pool".as_slice().into() }, Some(Kind::Store), WORK)
}

fn checkout(remote: &[u8]) -> Vec<Step> {
    vec![
        store(remote),
        Step { stdout: "sha1\n".into(), ..step(Command::Format, None, POOL) },
        step(Command::Clone { directory: b"widget".as_slice().into(), format: Format::Sha1, bare: false }, None, WORK),
        Step {
            edit: Some(ALTERNATES),
            ..step(Command::Remote { remote: remote.into(), format: Format::Sha1 }, None, WIDGET)
        },
        step(
            Command::CheckOut { commit: invocation::object(BASE).expect("seed commit") },
            Some(Kind::CheckOut),
            WIDGET,
        ),
    ]
}

fn push(remote: &[u8]) -> Vec<Step> {
    let tree = invocation::object(TREE.as_bytes()).expect("fixture tree id");
    let parent = invocation::object(BASE).expect("seed commit");
    let commit = invocation::object(STORED.as_bytes()).expect("fixture commit id");
    let mut steps = checkout(remote);
    steps.extend([
        Step { edit: Some(EDIT), ..step(Command::Add, Some(Kind::Commit), WIDGET) },
        Step { stdout: format!("{TREE}\n"), ..step(Command::WriteTree, Some(Kind::Commit), WIDGET) },
        Step {
            input: MESSAGE,
            stdout: format!("{STORED}\n"),
            ..step(Command::CommitTree { tree, parent, merging: None }, Some(Kind::Commit), WIDGET)
        },
        Step {
            stdout: format!("To {remote}\n*\t{STORED}:refs/heads/work\t[new branch]\nDone\n", remote = text(remote)),
            ..step(
                Command::Push { remote: remote.into(), commit, branch: b"work".as_slice().into(), expected: None },
                Some(Kind::Push),
                WIDGET,
            )
        },
    ]);
    steps
}

type Words = Box<[Box<[u8]>]>;

#[derive(Clone, Copy)]
enum Entry {
    Link,
    Verb,
}

impl Entry {
    fn select(self, arguments: Vec<Box<[u8]>>, working: &[u8]) -> (&'static [u8], Words) {
        match self {
            Self::Link => {
                let program = "/bin/git";
                assert!(git::is_git(std::ffi::OsStr::new(program)));
                (program.as_bytes(), arguments.into_boxed_slice())
            }
            Self::Verb => {
                let mut words = vec![Box::from(b"fake".as_slice()), Box::from(b"git".as_slice())];
                words.extend(arguments);
                let temper_box_shell::Command::FakeGit { arguments } =
                    temper_box_shell::configure_with(&words, &[], working).expect("fake git verb")
                else {
                    panic!("fake git verb must select Git");
                };
                (b"temper-box", arguments)
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Tree {
    Unchanged,
    Checkout,
    Pushed,
}

struct Row {
    name: &'static str,
    endpoint: Endpoint,
    entry: Entry,
    steps: Vec<Step>,
    missing: Option<&'static [u8]>,
    tree: Tree,
}

fn row(name: &'static str, steps: Vec<Step>) -> Row {
    Row { name, endpoint: Endpoint::Forge, entry: Entry::Link, steps, missing: None, tree: Tree::Unchanged }
}

fn wire_failure(name: &'static str, endpoint: Endpoint, stderr: &'static str) -> Row {
    let clone = Step { exit: 128, stderr, ..store(endpoint.remote().as_bytes()) };
    Row { endpoint, ..row(name, vec![clone]) }
}

#[test]
fn git_run() {
    let remote = Endpoint::Forge.remote();
    let usage = Step {
        extra: Some(b"--unsupported"),
        exit: 129,
        stderr: "usage: git [--version] [--help] [-C <path>] [-c <name>=<value>] <command> [<args>]\n",
        ..step(Command::Version, None, WORK)
    };
    let environment = Step {
        exit: 125,
        stderr: "temper fake git: broken: fixed environment value\n",
        ..step(Command::Version, None, WORK)
    };
    for row in [
        row("version", vec![version()]),
        Row { tree: Tree::Checkout, ..row("clone", checkout(remote.as_bytes())) },
        Row { tree: Tree::Pushed, ..row("push", push(remote.as_bytes())) },
        row("usage", vec![usage]),
        Row { missing: Some(b"LC_ALL"), ..row("environment", vec![environment]) },
        wire_failure("no_route", NO_ROUTE, "fatal: {address} serves no fake git route\n"),
        wire_failure("refused", REFUSED, "fatal: unable to access '{remote}': The requested URL returned error: 401\n"),
        wire_failure("closed", Endpoint::Closed, "fatal: unable to access '{remote}': Failed to connect to forge\n"),
        Row { entry: Entry::Verb, ..row("verb", vec![version()]) },
    ] {
        check_row(row);
    }
}

fn check_row(row: Row) {
    let token = credential::credential(SEED, credential::ADMIN_TOKEN);
    let identity = Identity {
        token: Secret::new(token.as_slice().into()),
        login: ADMIN.into(),
        name: b"QA".as_slice().into(),
        email: b"qa@temper.invalid".as_slice().into(),
    };
    let mut hosted = Hosted::new(&token);
    let remote = row.endpoint.remote();
    for step in row.steps {
        if let Some((path, content)) = step.edit {
            hosted.write(step.at, path, content);
        }
        let mut arguments = invocation::arguments(&step.command).into_vec();
        arguments.extend(step.extra.map(Box::from));
        let (program, arguments) = row.entry.select(arguments, step.at);
        let environment: Vec<_> = invocation::environment(b"/home", step.kind, Some(&identity))
            .iter()
            .map(|entry| entry.split_at(entry.iter().position(|b| *b == b'=').expect("environment entry")))
            .filter(|(name, _)| Some(*name) != row.missing)
            .map(|(name, value)| (Box::from(name), Box::from(&value[1..])))
            .collect();
        let answer = hosted
            .run(row.endpoint, step.input, |root| git::invocation(&arguments, program, &environment, step.at, root));
        let stderr = step.stderr.replace("{remote}", &remote).replace("{address}", &row.endpoint.address().to_string());
        assert_eq!(
            answer.exit,
            step.exit,
            "{}: {:?}: {}",
            row.name,
            step.command,
            String::from_utf8_lossy(&answer.stderr)
        );
        assert_eq!(answer.stdout, step.stdout.as_bytes(), "{}: {:?}: stdout", row.name, step.command);
        assert_eq!(answer.stderr, stderr.as_bytes(), "{}: {:?}: stderr", row.name, step.command);
    }
    match row.tree {
        Tree::Unchanged => assert!(hosted.work_is_empty(), "{}: nothing written", row.name),
        Tree::Checkout | Tree::Pushed => {
            for (path, content) in
                forge::FILES.into_iter().chain([(b".git/temper-remote".as_slice(), remote.as_bytes())])
            {
                assert_eq!(hosted.read(WIDGET, path), content, "{}: {}", row.name, text(path));
            }
            assert_eq!(hosted.read(WIDGET, ALTERNATES.0), ALTERNATES.1);
        }
    }
    if let Tree::Pushed = row.tree {
        assert_eq!(hosted.read(WIDGET, EDIT.0), EDIT.1);
        assert_eq!(hosted.branch(&token), STORED);
        hosted.check_message(MESSAGE);
    }
    hosted.finish();
}

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("fixture text")
}
