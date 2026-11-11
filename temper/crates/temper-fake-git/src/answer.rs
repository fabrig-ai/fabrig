//! Git's plumbing outputs over a checkout; no invocation starts a program.
use crate::{Checked, Transport, id::id};
use skein_fake_checkout::{
    Checkout,
    git::{self, Created, Fault, Pushed, Tree, Want, What},
    in_git,
};
use temper_worker_domain_checkout::git::{Commit, Into};
use temper_worker_protocol_git::invocation::{Command, Source};

/// Bytes written to the invocation's pipes and its terminal status.
#[derive(Debug, PartialEq, Eq)]
pub struct Answer {
    pub exit: u8,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
impl Answer {
    fn success(stdout: Vec<u8>) -> Self {
        Self { exit: 0, stdout, stderr: Vec::new() }
    }
    fn local_failure() -> Self {
        Self { exit: 128, stdout: Vec::new(), stderr: b"fatal: invalid object or repository\n".to_vec() }
    }
    #[must_use]
    pub fn usage() -> Self {
        Self {
            exit: 129,
            stdout: Vec::new(),
            stderr: b"usage: git [--version] [--help] [-C <path>] [-c <name>=<value>] <command> [<args>]\n".to_vec(),
        }
    }
}
/// Answer one admitted invocation using only the fake checkout and transport.
#[must_use]
#[expect(clippy::too_many_lines, reason = "the invocation answer table keeps every supported git command together")]
pub fn answer(checked: &Checked, checkout: &mut Checkout, transport: &mut impl Transport) -> Answer {
    transport.authenticate(checked.credential.as_ref());
    let at = checked.at.as_ref();
    match &checked.command {
        Command::SnapshotHead => {
            let Some(head) = checkout.content(&path(at, b".git/temper-head")).map(count_bytes) else {
                return Answer::local_failure();
            };
            let mut stdout = [id(head).as_slice(), b" "].concat();
            if let Some(parent) = transport.parent(head) {
                stdout.extend(id(parent));
            }
            if let Some(second) = transport.merge_parent(head) {
                stdout.push(b' ');
                stdout.extend(id(second));
            }
            stdout.push(b'\n');
            Answer::success(stdout)
        }
        Command::SnapshotMerge { .. } => match checkout.content(&path(at, b".git/MERGE_HEAD")) {
            Some(head) => Answer::success([id(count_bytes(head)).as_slice(), b"\n"].concat()),
            None => Answer { exit: 1, stdout: Vec::new(), stderr: Vec::new() },
        },
        Command::SnapshotFetch { remote, .. } => {
            let heads = match transport.heads(remote) {
                Ok(heads) => heads,
                Err(fault) => return failed(remote, fault),
            };
            checkout.remove(&path(at, b".git/temper-delivery-heads"));
            for head in heads {
                if let Err(fault) = git::fetch(transport, checkout, remote, at, Want::Commit(head)) {
                    return failed(remote, fault);
                }
                checkout.write(&path(at, format!(".git/temper-delivery-heads/{head}").as_bytes()), &head.to_le_bytes());
            }
            Answer::success(Vec::new())
        }
        Command::SnapshotReachable { second, .. } => {
            let head = checkout
                .tree(&path(at, b".git/temper-delivery-heads"))
                .into_values()
                .map(|bytes| count_bytes(&bytes))
                .find(|head| transport.is_ancestor(count(*second), *head));
            Answer::success(head.map_or_else(Vec::new, |head| [id(head).as_slice(), b"\n"].concat()))
        }
        Command::SnapshotReset { commit } => {
            checkout.write(&path(at, b".git/temper-head"), &count(*commit).to_le_bytes());
            checkout.remove(&path(at, b".git/MERGE_HEAD"));
            checkout.remove(&path(at, b".git/temper-conflicts"));
            write_index(checkout, at, transport.tree(count(*commit)));
            Answer::success(Vec::new())
        }
        Command::Version => Answer::success(b"git version 2.47.3\n".to_vec()),
        Command::Origin { directory } => {
            let directory = path(at, directory);
            if let Some(remote) = checkout.content(&path(&directory, b".git/temper-remote")) {
                Answer::success([remote, b"\n"].concat())
            } else if checkout.exists(&path(&directory, b".git")) {
                Answer { exit: 1, stdout: Vec::new(), stderr: Vec::new() }
            } else {
                Answer {
                    exit: 128,
                    stdout: Vec::new(),
                    stderr: b"fatal: --local can only be used inside a git repository\n".to_vec(),
                }
            }
        }
        Command::Clone { remote, directory } => {
            let destination = path(at, directory);
            match git::clone_repository(transport, checkout, remote, &destination) {
                Ok(()) => {
                    checkout.write(&path(&destination, b".git/temper-remote"), remote);
                    Answer::success(Vec::new())
                }
                Err(fault) => failed(remote, fault),
            }
        }
        Command::Fetch { remote, source, into } => fetch(checkout, at, transport, remote, source, *into),
        Command::Create { remote, branch, commit } => create(checked, transport, remote, branch, commit),
        Command::CheckOut { commit } => {
            let lock = path(at, b".git/index.lock");
            if checkout.exists(&lock) {
                return Answer {
                    exit: 128,
                    stdout: Vec::new(),
                    stderr: format!("fatal: Unable to create '{}': File exists.\n", text(&lock)).into_bytes(),
                };
            }
            match git::check_out(transport, checkout, at, count(*commit)) {
                Ok(()) => {
                    write_index(checkout, at, working(checkout, at));
                    Answer::success(Vec::new())
                }
                Err(_) => Answer::local_failure(),
            }
        }
        Command::Clean => {
            let index = staged(checkout, at);
            let mut tree = working(checkout, at);
            tree.retain(|name, _| index.contains_key(name));
            checkout.replace_tree(at, &tree);
            Answer::success(Vec::new())
        }
        Command::Merge { theirs } => merge(checkout, at, transport, count(*theirs)),
        Command::Unmerged => {
            let mut bytes = Vec::new();
            for file in checkout.tree(&path(at, b".git/temper-conflicts")).into_values() {
                bytes.extend(file);
                bytes.push(0);
            }
            Answer::success(bytes)
        }
        Command::Add => {
            stage(checkout, at);
            checkout.remove(&path(at, b".git/temper-conflicts"));
            Answer::success(Vec::new())
        }
        Command::DiffPaths { before, after } => {
            if !checkout.exists(&object(at, count(*before))) || !checkout.exists(&object(at, count(*after))) {
                return Answer::local_failure();
            }
            let before = transport.tree(count(*before));
            let after = transport.tree(count(*after));
            let paths: std::collections::BTreeSet<_> = before.keys().chain(after.keys()).collect();
            let mut bytes = Vec::new();
            for path in paths {
                if before.get(path) != after.get(path) {
                    bytes.extend(path);
                    bytes.push(0);
                }
            }
            Answer::success(bytes)
        }
        Command::DiffIndex { parent } => {
            if !checkout.exists(&object(at, count(*parent))) {
                return Answer::local_failure();
            }
            let mut out = Answer::success(Vec::new());
            out.exit = u8::from(staged(checkout, at) != transport.tree(count(*parent)));
            out
        }
        Command::WriteTree => Answer::success([id(tree_id(&staged(checkout, at))).as_slice(), b"\n"].concat()),
        Command::CommitTree { tree, parent, merging } => {
            let snapshot = staged(checkout, at);
            if count(*tree) != tree_id(&snapshot) {
                return Answer::local_failure();
            }
            let parent = count(*parent);
            if !checkout.exists(&object(at, parent))
                || merging.is_some_and(|second| !checkout.exists(&object(at, count(second))))
            {
                return Answer::local_failure();
            }
            let commit = transport.commit_tree(parent, merging.map(count), snapshot, &checked.message);
            checkout.write(&object(at, commit), b"");
            Answer::success([id(commit).as_slice(), b"\n"].concat())
        }
        Command::IsAncestor { expected, commit } => {
            let mut out = Answer::success(Vec::new());
            out.exit = u8::from(!transport.is_ancestor(count(*expected), count(*commit)));
            out
        }
        Command::Push { remote, commit, branch, expected } => {
            push(checkout, at, transport, remote, commit, branch, *expected)
        }
    }
}
fn merge(checkout: &mut Checkout, at: &[u8], transport: &mut impl Transport, theirs: u64) -> Answer {
    // temper-git-transcripts' merge-in-progress records this refusal before even an already-contained head.
    if checkout.exists(&path(at, b".git/MERGE_HEAD")) {
        return Answer {
            exit: 128,
            stdout: Vec::new(),
            stderr: b"fatal: You have not concluded your merge (MERGE_HEAD exists).\nPlease, commit your changes before you merge.\n".to_vec(),
        };
    }
    let Some(head) = checkout.content(&path(at, b".git/temper-head")).map(count_bytes) else {
        return Answer::local_failure();
    };
    if transport.is_ancestor(theirs, head) {
        return Answer::success(b"Already up to date.\n".to_vec());
    }
    match git::merge(transport, checkout, at, theirs) {
        Ok(merged) => {
            let mut out = Answer::success(Vec::new());
            out.exit = u8::from(!merged.conflicts.is_empty());
            for file in &merged.conflicts {
                out.stdout.extend(
                    format!("Auto-merging {}\nCONFLICT (content): Merge conflict in {}\n", text(file), text(file))
                        .as_bytes(),
                );
            }
            if out.exit == 1 {
                out.stdout.extend(b"Automatic merge failed; fix conflicts and then commit the result.\n");
            }
            out
        }
        Err(_) => Answer::local_failure(),
    }
}
fn pushed(remote: &[u8], branch: &[u8], commit: u64, flag: u8, summary: &str, exit: u8) -> Answer {
    Answer {
        exit,
        stdout: format!(
            "To {}\n{}\t{}:refs/heads/{}\t{}\nDone\n",
            text(remote),
            char::from(flag),
            text(&id(commit)),
            text(branch),
            summary
        )
        .into_bytes(),
        stderr: if exit == 0 {
            Vec::new()
        } else {
            format!("error: failed to push some refs to '{}'\n", text(remote)).into_bytes()
        },
    }
}
fn failed(remote: &[u8], fault: Fault) -> Answer {
    let stderr = match fault {
        Fault::Missing(What::Repository) => format!("fatal: repository '{}' not found\n", text(remote)),
        Fault::Missing(What::Branch) => "fatal: couldn't find remote ref missing\n".into(),
        Fault::Missing(What::Commit) => "fatal: remote error: upload-pack: not our ref missing\n".into(),
        Fault::Refused => {
            format!("fatal: unable to access '{}': The requested URL returned error: 401\n", text(remote))
        }
        Fault::Unreachable => format!("fatal: unable to access '{}': Failed to connect to forge\n", text(remote)),
    };
    Answer { exit: 128, stdout: Vec::new(), stderr: stderr.into_bytes() }
}
pub(crate) fn count(commit: Commit) -> u64 {
    skein_fake_forge_forgejo::routes::commit(&skein_forge::forgejo::id::ObjectId {
        format: skein_forge::forgejo::id::ObjectFormat::Sha1,
        bytes: commit.raw(),
    })
    .expect("the shared fake commit spelling")
}
fn count_bytes(bytes: &[u8]) -> u64 {
    u64::from_le_bytes(bytes.try_into().expect("stored eight-byte count"))
}
pub(crate) fn path(at: &[u8], name: &[u8]) -> Vec<u8> {
    if at.is_empty() { name.to_vec() } else { [at, b"/", name].concat() }
}
fn object(at: &[u8], commit: u64) -> Vec<u8> {
    path(at, format!(".git/objects/{commit}").as_bytes())
}
fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
fn abbreviated(commit: u64) -> String {
    text(&id(commit)[..16])
}
fn working(checkout: &Checkout, at: &[u8]) -> Tree {
    let mut tree = checkout.tree(at);
    tree.retain(|name, _| !in_git(name));
    tree
}
fn staged(checkout: &Checkout, at: &[u8]) -> Tree {
    checkout.tree(&path(at, b".git/temper-index"))
}
// protocol/git.md, 4.7: excluded logs retain the index's contents, even when deleted.
fn stage(checkout: &mut Checkout, at: &[u8]) {
    let mut tree = working(checkout, at);
    let prior = staged(checkout, at);
    let ignored = tree.get(b".gitignore".as_slice()).cloned().unwrap_or_default();
    tree.retain(|name, _| {
        !hidden_log(name)
            && (prior.contains_key(name)
                || !ignored
                    .split(|b| *b == b'\n')
                    .any(|line| !line.is_empty() && !line.starts_with(b"#") && glob(line, name)))
    });
    for (name, content) in prior {
        if hidden_log(&name) {
            tree.insert(name, content);
        }
    }
    write_index(checkout, at, tree);
}
fn hidden_log(path: &[u8]) -> bool {
    path.rsplit(|b| *b == b'/')
        .next()
        .is_some_and(|name| name.len() > b".log".len() && name.starts_with(b".") && name.ends_with(b".log"))
}
fn write_index(checkout: &mut Checkout, at: &[u8], tree: Tree) {
    let index = path(at, b".git/temper-index");
    checkout.remove(&index);
    checkout.mkdir(&index);
    for (name, content) in tree {
        checkout.write(&path(&index, &name), &content);
    }
}
fn glob(pattern: &[u8], name: &[u8]) -> bool {
    match pattern.split_first() {
        None => name.is_empty(),
        Some((b'*', rest)) => (0..=name.len()).any(|skip| glob(rest, &name[skip..])),
        Some((b'?', rest)) => !name.is_empty() && glob(rest, &name[1..]),
        Some((first, rest)) => name.first() == Some(first) && glob(rest, &name[1..]),
    }
}
fn tree_id(tree: &Tree) -> u64 {
    tree.iter()
        .flat_map(|(path, content)| path.iter().chain(content))
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3))
}

fn fetch(
    checkout: &mut Checkout,
    at: &[u8],
    transport: &mut impl Transport,
    remote: &[u8],
    source: &Source,
    into: Into,
) -> Answer {
    let reference = match into {
        Into::Fetched => b"refs/temper/fetched".as_slice(),
        Into::Landing => b"refs/temper/landing".as_slice(),
    };
    let target = path(at, &[b".git/".as_slice(), reference].concat());
    let previous = checkout.content(&target).map_or(0, count_bytes);
    let want = match source {
        Source::Branch(branch) => Want::Branch(branch),
        Source::Commit(commit) => Want::Commit(count(*commit)),
        Source::Default => Want::Default,
    };
    match git::fetch(transport, checkout, remote, at, want) {
        Ok(commit) => {
            checkout.write(&target, &commit.to_le_bytes());
            let flag = if previous == 0 {
                b'*'
            } else if previous == commit {
                b'='
            } else if transport.is_ancestor(previous, commit) {
                b' '
            } else {
                b'+'
            };
            // An absent previous head is git's zero marker, not the fake's count-zero id.
            let previous_id = if previous == 0 { [b'0'; 40] } else { id(previous) };
            Answer::success(
                format!("{} {} {} {}\n", char::from(flag), text(&previous_id), text(&id(commit)), text(reference))
                    .into_bytes(),
            )
        }
        Err(fault) => failed(remote, fault),
    }
}

fn create(checked: &Checked, transport: &mut impl Transport, remote: &[u8], branch: &[u8], commit: &Commit) -> Answer {
    let commit = count(*commit);
    let old = transport.branch(remote, branch);
    match git::create(transport, remote, branch, commit) {
        Ok(Created::Created) => pushed(remote, branch, commit, b'*', "[new branch]", 0),
        Ok(Created::Exists) if old == Some(commit) => pushed(remote, branch, commit, b'=', "[up to date]", 0),
        Ok(Created::Exists) => pushed(remote, branch, commit, b'!', "[rejected] (stale info)", 1),
        Err(Fault::Refused) if checked.credential.is_some() && transport.authenticated() => {
            pushed(remote, branch, commit, b'!', "[remote rejected] (pre-receive hook declined)", 1)
        }
        Err(fault) => failed(remote, fault),
    }
}

fn push(
    checkout: &Checkout,
    at: &[u8],
    transport: &mut impl Transport,
    remote: &[u8],
    commit: &Commit,
    branch: &[u8],
    expected: Option<Commit>,
) -> Answer {
    let commit = count(*commit);
    if !checkout.exists(&object(at, commit)) {
        return Answer::local_failure();
    }
    let old = transport.branch(remote, branch);
    match transport.push(remote, branch, commit, expected.map(count)) {
        Ok(Pushed::Pushed) if old == Some(commit) => pushed(remote, branch, commit, b'=', "[up to date]", 0),
        Ok(Pushed::Pushed) if old.is_none() => pushed(remote, branch, commit, b'*', "[new branch]", 0),
        Ok(Pushed::Pushed) => pushed(
            remote,
            branch,
            commit,
            b' ',
            &format!("{}..{}", abbreviated(old.expect("existing branch")), abbreviated(commit)),
            0,
        ),
        Ok(Pushed::Rejected) => pushed(
            remote,
            branch,
            commit,
            b'!',
            if expected.is_some() && expected.map(count) != old {
                "[rejected] (stale info)"
            } else {
                "[rejected] (non-fast-forward)"
            },
            1,
        ),
        Err(Fault::Refused) if transport.authenticated() => {
            pushed(remote, branch, commit, b'!', "[remote rejected] (pre-receive hook declined)", 1)
        }
        Err(fault) => failed(remote, fault),
    }
}
