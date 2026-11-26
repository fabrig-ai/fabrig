//! The look commands' git argument lists on real git (temper/docs/design/client/journey-writer.md, 7).

use std::{
    collections::BTreeSet,
    ffi::OsStr,
    fs,
    os::unix::ffi::OsStrExt as _,
    path::Path,
    process::{Command, Output, Stdio},
};

use skein_scratch::{Scratch, paths};

const FIXTURE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../crates/temper-client-journeys-tool/src/look_git_lists.tsv"
));

type Arguments = Vec<Box<[u8]>>;

fn argument(field: &str) -> Box<[u8]> {
    let mut bytes = field.bytes();
    let mut argument = Vec::with_capacity(field.len());
    while let Some(byte) = bytes.next() {
        argument.push(if byte == b'\\' {
            match bytes.next() {
                Some(b'\\') => b'\\',
                Some(b't') => b'\t',
                Some(b'r') => b'\r',
                Some(b'n') => b'\n',
                _ => panic!("invalid escape in git argument {field:?}"),
            }
        } else {
            byte
        });
    }
    argument.into_boxed_slice()
}

#[test]
fn git_accepts_the_look_steps_argument_lists() {
    let lists: BTreeSet<Arguments> =
        FIXTURE.split_terminator('\n').map(|row| row.split('\t').skip(1).map(argument).collect()).collect();

    let scratch = Scratch::new_in(&paths::target_tmp(), "look-git");
    let root = scratch.path();
    let revisions = Revisions::prepare(root, &lists);
    // Even a failed fetch clears FETCH_HEAD; let the lists that read it run first.
    let (fetches, other): (Vec<_>, Vec<_>) =
        lists.iter().partition(|args| args.first().is_some_and(|command| command.as_ref() == b"fetch"));
    let mut refusals = Vec::new();
    for args in other.into_iter().chain(fetches) {
        let replay = revisions.resolve(args);
        let output = git(root).args(replay.iter().map(|word| OsStr::from_bytes(word))).output().expect("run git");
        let display: Vec<_> = args.iter().map(|word| OsStr::from_bytes(word)).collect();
        let stderr = String::from_utf8_lossy(&output.stderr);
        let first_line = stderr.lines().next().unwrap_or_default();
        if output.status.code() == Some(129) {
            refusals.push(format!("git {display:?}: {first_line}"));
        } else if !output.status.success() && !first_line.is_empty() {
            println!("git {display:?}: {}: {first_line}", output.status);
        }
    }
    println!("Ran {} distinct git lists", lists.len());
    assert!(refusals.is_empty(), "git refused these argument lists with exit 129:\n{}", refusals.join("\n"));
}

struct Revisions {
    parent: Vec<u8>,
    child: Vec<u8>,
}

impl Revisions {
    fn prepare(root: &Path, lists: &BTreeSet<Arguments>) -> Self {
        setup(root, &["init", "--quiet", "--template=", "--initial-branch=look-requests", "--object-format=sha1"]);
        let parent = commit(root, "Parent");
        let child = commit(root, "Request");
        let mut refs = BTreeSet::from([
            "refs/heads/tui-requests",
            "refs/remotes/origin/look-requests",
            "refs/remotes/origin/tui-requests",
        ]);
        for name in lists.iter().flatten().filter(|word| word.starts_with(b"refs/heads/docs/look/")) {
            refs.insert(std::str::from_utf8(name).expect("test request ref is text"));
        }
        let tip = std::str::from_utf8(&child).expect("git commit ID is text");
        for name in refs {
            setup(root, &["update-ref", name, tip]);
        }
        for name in ["FETCH_HEAD", "REBASE_HEAD"] {
            fs::write(root.join(".git").join(name), format!("{tip}\n")).expect("write git pseudoref");
        }
        setup(
            root,
            &["update-ref", "refs/remotes/origin/main", std::str::from_utf8(&parent).expect("git commit ID is text")],
        );
        for (file, bytes) in [
            ("parent", "old\n"),
            ("landed", "old\n"),
            ("requested", "new\n"),
            ("merged", "new\n"),
            ("merged-with-landing-conflicts", "new\n"),
        ] {
            fs::write(root.join(file), bytes).expect("write merge and diff input");
        }
        Self { parent, child }
    }

    fn resolve(&self, args: &[Box<[u8]>]) -> Vec<Vec<u8>> {
        // Keep range and peeling syntax intact. Repeated IDs in one list stay
        // equal; two different IDs (e.g. diff-tree's P and C) resolve to two commits.
        let mut first = None;
        let mut paths = false;
        args.iter()
            .map(|arg| {
                paths |= arg.as_ref() == b"--";
                if paths {
                    arg.to_vec()
                } else {
                    let mut word = Vec::new();
                    for part in arg.split_inclusive(|byte| revision_separator(*byte)) {
                        let (atom, separator) = match part.split_last() {
                            Some((last, rest)) if revision_separator(*last) => (rest, &part[rest.len()..]),
                            _ => (part, &[][..]),
                        };
                        if (7..=64).contains(&atom.len())
                            && atom.iter().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
                        {
                            let head: &[u8] = first.get_or_insert(atom);
                            let revision = if head.starts_with(atom) || atom.starts_with(head) {
                                &self.parent
                            } else {
                                &self.child
                            };
                            word.extend_from_slice(revision);
                        } else {
                            word.extend_from_slice(atom);
                        }
                        word.extend_from_slice(separator);
                    }
                    word
                }
            })
            .collect()
    }
}

fn revision_separator(byte: u8) -> bool {
    matches!(byte, b'.' | b'^' | b'~' | b':' | b'=')
}

fn git(root: &Path) -> Command {
    let mut command = Command::new("git");
    command
        .current_dir(root)
        .env_clear()
        .env("HOME", root)
        .env("LC_ALL", "C")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_ALLOW_PROTOCOL", "file")
        .env("GIT_SEQUENCE_EDITOR", "false")
        .env("GIT_EDITOR", "false")
        .env("GIT_AUTHOR_NAME", "Look tests")
        .env("GIT_AUTHOR_EMAIL", "look@example.invalid")
        .env("GIT_COMMITTER_NAME", "Look tests")
        .env("GIT_COMMITTER_EMAIL", "look@example.invalid")
        .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00Z")
        .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00Z")
        .stdin(Stdio::null());
    command
}

fn setup(root: &Path, args: &[&str]) -> Output {
    let output = git(root).args(args).output().expect("set up git in scratch");
    assert!(output.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&output.stderr));
    output
}

fn commit(root: &Path, message: &str) -> Vec<u8> {
    setup(root, &["commit", "--allow-empty", "-qm", message]);
    setup(root, &["rev-parse", "HEAD"]).stdout.trim_ascii_end().to_vec()
}
