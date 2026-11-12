//! Sized builders for protocol/git.md, sections 3 to 5.
use crate::Identity;
use alloc::boxed::Box;
use skein_lib::{Decimal, List, Token, Writer, base64};
use temper_worker_domain_checkout::git;

// protocol/git.md, 4.7: a hidden log at any depth is never staged.
const HIDDEN_LOG_EXCLUSION: &[u8] = b":(exclude,glob)**/.*.log";

/// Settings precede every command, in this order, each with `-c`.
pub const SETTINGS: [&[u8]; 13] = [
    b"core.hooksPath=/dev/null",
    b"core.fsmonitor=false",
    b"protocol.version=2",
    b"protocol.allow=never",
    b"protocol.https.allow=always",
    b"protocol.http.allow=always",
    b"credential.helper=",
    b"http.sslVerify=true",
    b"gc.auto=0",
    b"maintenance.auto=false",
    b"rerere.enabled=false",
    b"merge.conflictStyle=merge",
    b"submodule.recurse=false",
];

/// One invocation in an operation's fixed sequence.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Command {
    Format,
    SnapshotHead,
    SnapshotMerge { head: git::Commit, second: Option<git::Commit> },
    SnapshotFetch { remote: Box<[u8]>, head: git::Commit, merging: Option<git::Commit>, second: git::Commit },
    SnapshotReachable { head: git::Commit, merging: Option<git::Commit>, second: git::Commit },
    SnapshotReset { commit: git::Commit },
    Version,
    Origin { directory: Box<[u8]> },
    Clone { directory: Box<[u8]>, format: Format },
    Remote { remote: Box<[u8]> },
    Store { directory: Box<[u8]>, remote: Box<[u8]> },
    UpdateRef { into: git::Into, commit: git::Commit },
    Fetch { remote: Box<[u8]>, source: Source, into: git::Into, workspace: Token },
    Create { remote: Box<[u8]>, branch: Box<[u8]>, commit: git::Commit },
    CheckOut { commit: git::Commit },
    Clean,
    Merge { theirs: git::Commit },
    Unmerged,
    Add,
    DiffIndex { parent: git::Commit },
    DiffPaths { before: git::Commit, after: git::Commit },
    WriteTree,
    CommitTree { tree: git::Commit, parent: git::Commit, merging: Option<git::Commit> },
    MergeBases { head: git::Commit, upstream: git::Commit },
    BringsList { bases: Box<[git::Commit]>, base: git::Commit },
    IsAncestor { expected: git::Commit, commit: git::Commit },
    Push { remote: Box<[u8]>, commit: git::Commit, branch: Box<[u8]>, expected: Option<git::Commit> },
}

/// The item's program is carried by every operation except Make.
pub(crate) fn item_program(op: &git::Op) -> Option<&[u8]> {
    match op {
        git::Op::Make { .. } => None,
        git::Op::Store { git, .. } => git.as_deref(),
        git::Op::Clone { at, .. }
        | git::Op::Point { at, .. }
        | git::Op::Fetch { at, .. }
        | git::Op::Create { at, .. }
        | git::Op::CheckOut { at, .. }
        | git::Op::Look { at, .. }
        | git::Op::DiffPaths { at, .. }
        | git::Op::Brings { at, .. }
        | git::Op::IsAncestor { at, .. }
        | git::Op::Merge { at, .. }
        | git::Op::Commit { at, .. }
        | git::Op::Push { at, .. } => at.git.as_deref(),
    }
}

/// The synthetic fatal output uses the URL this invocation would pass git.
pub(crate) fn no_program(op: &git::Op, command: &Command) -> Box<[u8]> {
    let remote = match command {
        Command::SnapshotFetch { remote, .. }
        | Command::Remote { remote }
        | Command::Store { remote, .. }
        | Command::Fetch { remote, .. }
        | Command::Create { remote, .. }
        | Command::Push { remote, .. } => Some(remote.as_ref()),
        Command::IsAncestor { .. } => match op {
            git::Op::Push { remote, .. } => Some(remote.as_ref()),
            git::Op::Look { .. } | git::Op::DiffPaths { .. } | git::Op::Brings { .. } | git::Op::IsAncestor { .. } => {
                None
            }
            git::Op::Store { .. }
            | git::Op::Make { .. }
            | git::Op::Clone { .. }
            | git::Op::Point { .. }
            | git::Op::Fetch { .. }
            | git::Op::Create { .. }
            | git::Op::CheckOut { .. }
            | git::Op::Merge { .. }
            | git::Op::Commit { .. } => unreachable!("ancestry is part of a push"),
        },
        Command::SnapshotHead
        | Command::SnapshotMerge { .. }
        | Command::SnapshotReachable { .. }
        | Command::SnapshotReset { .. }
        | Command::Origin { .. }
        | Command::Format
        | Command::Clone { .. }
        | Command::UpdateRef { .. }
        | Command::Version
        | Command::CheckOut { .. }
        | Command::Clean
        | Command::Merge { .. }
        | Command::Unmerged
        | Command::Add
        | Command::DiffPaths { .. }
        | Command::DiffIndex { .. }
        | Command::WriteTree
        | Command::MergeBases { .. }
        | Command::BringsList { .. }
        | Command::CommitTree { .. } => None,
    };
    let prefix = b"fatal: no git on this machine";
    if let Some(remote) = remote {
        let address = remote.strip_prefix(b"https://").or(remote.strip_prefix(b"http://")).unwrap_or(remote);
        let mut end = address.len();
        for (index, byte) in address.iter().enumerate() {
            if *byte == b'/' {
                end = index;
                break;
            }
        }
        let host = address.get(..end).expect("host within URL");
        joined(&[prefix, b" for ", host])
    } else {
        Box::from(prefix.as_slice())
    }
}

/// The source of a fetch refspec.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Source {
    Branch(Box<[u8]>),
    Commit(git::Commit),
    Default,
}

/// Concatenation measured before allocating, with no growing byte buffer.
pub(crate) fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded input lengths fit usize");
    }
    let mut out = Writer::new(length);
    for part in parts {
        out.put(part).expect("measured parts fit");
    }
    out.finish()
}
fn push(out: &mut List<Box<[u8]>>, bytes: &[u8]) {
    out.push(Box::from(bytes)).expect("fixed vector fits its capacity");
}
fn words(out: &mut List<Box<[u8]>>, bytes: &[&[u8]]) {
    for byte in bytes {
        push(out, byte);
    }
}

/// The vector after argv[0], which io supplies from the program's path.
#[must_use]
pub fn arguments(command: &Command) -> Box<[Box<[u8]>]> {
    let extra = match command {
        Command::BringsList { bases, .. } => u32::try_from(bases.len()).expect("bounded merge-base stdout"),
        Command::Format
        | Command::SnapshotHead
        | Command::SnapshotMerge { .. }
        | Command::SnapshotFetch { .. }
        | Command::SnapshotReachable { .. }
        | Command::SnapshotReset { .. }
        | Command::Version
        | Command::Origin { .. }
        | Command::Clone { .. }
        | Command::Remote { .. }
        | Command::Store { .. }
        | Command::UpdateRef { .. }
        | Command::Fetch { .. }
        | Command::Create { .. }
        | Command::CheckOut { .. }
        | Command::Clean
        | Command::Merge { .. }
        | Command::Unmerged
        | Command::Add
        | Command::DiffIndex { .. }
        | Command::DiffPaths { .. }
        | Command::WriteTree
        | Command::CommitTree { .. }
        | Command::MergeBases { .. }
        | Command::IsAncestor { .. }
        | Command::Push { .. } => 0,
    };
    let mut out = List::with_capacity(40_u32.checked_add(extra).expect("bounded argv"));
    for setting in SETTINGS {
        words(&mut out, &[b"-c", setting]);
    }
    match command {
        Command::SnapshotHead => words(&mut out, &[b"log", b"-1", b"--format=%H %P", b"HEAD", b"--"]),
        Command::SnapshotMerge { .. } => words(&mut out, &[b"rev-parse", b"--verify", b"--quiet", b"MERGE_HEAD"]),
        Command::SnapshotFetch { remote, .. } => words(
            &mut out,
            &[
                b"fetch",
                b"--quiet",
                b"--prune",
                b"--no-tags",
                b"--no-write-fetch-head",
                b"--no-recurse-submodules",
                b"--",
                remote,
                b"+refs/heads/*:refs/temper/delivery/heads/*",
                b"+refs/tags/*:refs/temper/delivery/tags/*",
            ],
        ),
        Command::SnapshotReachable { second, .. } => words(
            &mut out,
            &[
                b"for-each-ref",
                b"--count=1",
                b"--format=%(objectname)",
                b"--contains",
                &hex(*second),
                b"refs/temper/delivery/",
            ],
        ),
        Command::SnapshotReset { commit } => words(&mut out, &[b"reset", b"--mixed", b"-q", &hex(*commit)]),
        Command::Version => push(&mut out, b"version"),
        Command::Origin { directory } => {
            let git_dir = joined(&[b"--git-dir=", directory, b"/.git"]);
            words(&mut out, &[&git_dir, b"config", b"--local", b"--get", b"remote.origin.url"]);
        }
        Command::Format => words(&mut out, &[b"rev-parse", b"--show-object-format"]),
        Command::Clone { directory, format } => {
            words(&mut out, &[b"init", b"--quiet", &joined(&[b"--object-format=", format.name()]), b"--", directory]);
        }
        Command::Remote { remote } => words(&mut out, &[b"remote", b"add", b"origin", b"--", remote]),
        Command::Store { remote, directory } => words(
            &mut out,
            &[
                b"clone",
                b"--bare",
                b"--filter=blob:none",
                b"--single-branch",
                b"--no-tags",
                b"--quiet",
                b"--",
                remote,
                directory,
            ],
        ),
        Command::UpdateRef { into, commit } => words(&mut out, &[b"update-ref", reference(*into), &hex(*commit)]),
        Command::Fetch { remote, source, into, workspace } => {
            let from = match source {
                Source::Branch(branch) => joined(&[b"refs/heads/", branch]),
                Source::Commit(commit) => hex(*commit),
                Source::Default => Box::from(&b"HEAD"[..]),
            };
            let spec = joined(&[b"+", &from, b":", &store_reference(*workspace, *into)]);
            words(
                &mut out,
                &[
                    b"fetch",
                    b"--porcelain",
                    b"--verbose",
                    b"--filter=blob:none",
                    b"--no-tags",
                    b"--no-write-fetch-head",
                    b"--no-recurse-submodules",
                    b"--",
                    remote,
                    &spec,
                ],
            );
        }
        Command::Create { remote, branch, commit } => {
            let lease = joined(&[b"--force-with-lease=refs/heads/", branch, b":"]);
            let spec = joined(&[&hex(*commit), b":refs/heads/", branch]);
            words(&mut out, &[b"push", b"--porcelain", &lease, b"--", remote, &spec]);
        }
        Command::CheckOut { commit } => words(
            &mut out,
            &[b"-c", b"remote.origin.promisor=true", b"checkout", b"--force", b"--detach", &hex(*commit)],
        ),
        Command::Clean => words(&mut out, &[b"clean", b"-ffdxq"]),
        Command::Merge { theirs } => words(
            &mut out,
            &[b"-c", b"remote.origin.promisor=true", b"merge", b"--no-commit", b"--no-ff", b"--", &hex(*theirs)],
        ),
        Command::DiffPaths { before, after } => words(
            &mut out,
            &[
                b"diff-tree",
                b"--no-commit-id",
                b"--name-only",
                b"--no-renames",
                b"-r",
                b"-z",
                &hex(*before),
                &hex(*after),
                b"--",
            ],
        ),
        Command::Unmerged => words(&mut out, &[b"diff-files", b"--name-only", b"-z", b"--diff-filter=U"]),
        Command::Add => words(&mut out, &[b"add", b"--all", b"--", b".", HIDDEN_LOG_EXCLUSION]),
        Command::DiffIndex { parent } => {
            words(&mut out, &[b"diff-index", b"--cached", b"--quiet", &hex(*parent), b"--"]);
        }
        Command::WriteTree => push(&mut out, b"write-tree"),
        Command::CommitTree { tree, parent, merging } => {
            words(&mut out, &[b"commit-tree", &hex(*tree), b"-p", &hex(*parent)]);
            if let Some(merging) = merging {
                words(&mut out, &[b"-p", &hex(*merging)]);
            }
            words(&mut out, &[b"-F", b"-"]);
        }
        Command::MergeBases { head, upstream } => {
            words(&mut out, &[b"merge-base", b"--all", &hex(*head), &hex(*upstream)]);
        }
        Command::BringsList { bases, base } => {
            words(&mut out, &[b"rev-list", b"--max-count=1"]);
            for commit in bases {
                push(&mut out, &hex(*commit));
            }
            words(&mut out, &[&joined(&[b"^", &hex(*base)]), b"--"]);
        }
        Command::IsAncestor { expected, commit } => {
            words(&mut out, &[b"merge-base", b"--is-ancestor", &hex(*expected), &hex(*commit)]);
        }
        Command::Push { remote, commit, branch, expected } => {
            words(&mut out, &[b"push", b"--porcelain"]);
            if let Some(expected) = expected {
                let lease = joined(&[b"--force-with-lease=refs/heads/", branch, b":", &hex(*expected)]);
                push(&mut out, &lease);
            }
            let spec = joined(&[&hex(*commit), b":refs/heads/", branch]);
            words(&mut out, &[b"--", remote, &spec]);
        }
    }
    out.into_boxed()
}

/// Whole environment; no PATH or inherited variable reaches git.
#[must_use]
pub fn environment(home: &[u8], kind: Option<git::Kind>, identity: Option<&Identity>) -> Box<[Box<[u8]>]> {
    environment_for(home, kind, identity, true)
}
pub(crate) fn environment_for(
    home: &[u8],
    kind: Option<git::Kind>,
    identity: Option<&Identity>,
    credentials: bool,
) -> Box<[Box<[u8]>]> {
    let identity = match kind {
        Some(
            git::Kind::Store
            | git::Kind::Fetch
            | git::Kind::Create
            | git::Kind::Commit
            | git::Kind::Push
            | git::Kind::CheckOut
            | git::Kind::Merge,
        ) => identity,
        None
        | Some(
            git::Kind::Point
            | git::Kind::Clone
            | git::Kind::Look
            | git::Kind::DiffPaths
            | git::Kind::Brings
            | git::Kind::IsAncestor
            | git::Kind::Make,
        ) => None,
    };
    let mut out = List::with_capacity(14);
    let home = joined(&[b"HOME=", home]);
    words(
        &mut out,
        &[
            &home,
            b"LC_ALL=C",
            b"TZ=UTC",
            b"GIT_CONFIG_NOSYSTEM=1",
            b"GIT_CONFIG_GLOBAL=/dev/null",
            b"GIT_TERMINAL_PROMPT=0",
            b"GIT_ASKPASS=",
        ],
    );
    let (name, email): (&[u8], &[u8]) = match identity {
        Some(value) => (&value.name, &value.email),
        None => (b"temper", b"temper@temper.invalid"),
    };
    let name_variable = joined(&[b"GIT_COMMITTER_NAME=", name]);
    let email_variable = joined(&[b"GIT_COMMITTER_EMAIL=", email]);
    words(&mut out, &[&name_variable, &email_variable]);
    match kind {
        Some(git::Kind::Commit) => {
            let author = joined(&[b"GIT_AUTHOR_NAME=", name]);
            let email = joined(&[b"GIT_AUTHOR_EMAIL=", email]);
            words(&mut out, &[&author, &email]);
        }
        Some(
            git::Kind::Store
            | git::Kind::Fetch
            | git::Kind::Create
            | git::Kind::Push
            | git::Kind::CheckOut
            | git::Kind::Merge,
        ) => {
            if let Some(identity) = identity
                && credentials
            {
                let raw = joined(&[&identity.login, b":", identity.token.expose()]);
                let size = u32::try_from(raw.len()).expect("channel identity length fits u32");
                let size =
                    base64::encoded_len(base64::Alphabet::Standard, size).expect("bounded identity encoding fits u32");
                let mut encoded = Writer::new(skein_lib::usize_of(size));
                base64::encode(base64::Alphabet::Standard, &raw, &mut encoded).expect("measured encoding fits");
                let header = joined(&[b"GIT_CONFIG_VALUE_0=Authorization: Basic ", &encoded.finish()]);
                words(&mut out, &[b"GIT_CONFIG_COUNT=1", b"GIT_CONFIG_KEY_0=http.extraHeader", &header]);
            }
        }
        None
        | Some(
            git::Kind::Point
            | git::Kind::Clone
            | git::Kind::Look
            | git::Kind::DiffPaths
            | git::Kind::Brings
            | git::Kind::IsAncestor
            | git::Kind::Make,
        ) => {}
    }
    out.into_boxed()
}

/// Title, optional blank line and body, and the final newline.
#[must_use]
pub fn message(title: &[u8], body: &[u8]) -> Box<[u8]> {
    if body.is_empty() { joined(&[title, b"\n"]) } else { joined(&[title, b"\n\n", body, b"\n"]) }
}

/// SHA-1's forty digits, or SHA-256's sixty-four digits.
#[must_use]
pub fn hex(commit: git::Commit) -> Box<[u8]> {
    let raw = commit.raw();
    let mut size = 20_usize;
    for byte in raw.get(20..).expect("fixed id") {
        if *byte != 0 {
            size = 32;
        }
    }
    let mut out = Writer::new(size.checked_mul(2).expect("fixed id"));
    for byte in raw.get(..size).expect("fixed id") {
        for digit in [byte >> 4_u32, byte & 15] {
            let byte = if digit < 10 {
                b'0'.checked_add(digit)
            } else {
                b'a'.checked_add(digit.checked_sub(10).expect("hex letter"))
            }
            .expect("hex digit");
            out.put(&[byte]).expect("measured hex fits");
        }
    }
    out.finish()
}

/// Only complete lowercase ids are accepted.
#[must_use]
pub fn object(digits: &[u8]) -> Option<git::Commit> {
    if digits.len() != 40 && digits.len() != 64 {
        return None;
    }
    let mut raw = [0; 32];
    for (&[high, low], byte) in digits.as_chunks::<2>().0.iter().zip(raw.iter_mut()) {
        *byte = nibble(high)?.checked_mul(16)?.checked_add(nibble(low)?)?;
    }
    Some(git::Commit::new(raw))
}
fn nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => byte.checked_sub(b'0'),
        b'a'..=b'f' => byte.checked_sub(b'a')?.checked_add(10),
        _ => None,
    }
}

/// The spawn's directory, beneath the service's root.
#[must_use]
pub fn place(at: &git::Place) -> Box<[u8]> {
    let workspace = Decimal::of(at.workspace.raw());
    joined(&[b"workspaces/", workspace.as_bytes(), b"/", &at.repository])
}

/// The local destination of a fetch (temper/docs/design/protocol/git.md, 4.3).
pub(crate) const fn reference(into: git::Into) -> &'static [u8] {
    match into {
        git::Into::Fetched => b"refs/temper/fetched",
        git::Into::Landing => b"refs/temper/landing",
    }
}

/// Workspace references in a shared store (protocol/git.md, 4.3).
#[must_use]
pub fn store_reference(workspace: Token, into: git::Into) -> Box<[u8]> {
    let name = reference(into).strip_prefix(b"refs/temper/").expect("fixed reference prefix");
    joined(&[b"refs/temper/", Decimal::of(workspace.raw()).as_bytes(), b"/", name])
}

/// The store and its borrowing repository have one object format (protocol/git.md, 4.2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Format {
    Sha1,
    Sha256,
}
impl Format {
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::Sha1 => b"sha1",
            Self::Sha256 => b"sha256",
        }
    }
}
