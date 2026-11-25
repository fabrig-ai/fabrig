use super::fixtures;
use crate::{
    invocation::{self, Command, Source},
    limits::{self, Unfit},
};
use alloc::boxed::Box;
use skein_lib::{Token, bytes};
use temper_worker_domain_checkout::git;

// Literal settings make the expected vector independent of the builder.
const SETTINGS: &[&[u8]] = &[
    b"-c",
    b"core.hooksPath=/dev/null",
    b"-c",
    b"core.fsmonitor=false",
    b"-c",
    b"protocol.version=2",
    b"-c",
    b"protocol.allow=never",
    b"-c",
    b"protocol.https.allow=always",
    b"-c",
    b"protocol.http.allow=always",
    b"-c",
    b"credential.helper=",
    b"-c",
    b"http.sslVerify=true",
    b"-c",
    b"gc.auto=0",
    b"-c",
    b"maintenance.auto=false",
    b"-c",
    b"rerere.enabled=false",
    b"-c",
    b"merge.conflictStyle=merge",
    b"-c",
    b"submodule.recurse=false",
];
const ID: &[u8] = b"abababababababababababababababababababab";
#[expect(clippy::too_many_lines, reason = "the argument table covers both fetch destinations")]
fn rows() -> [(Command, &'static [&'static [u8]]); 33] {
    let remote: Box<[u8]> = Box::from(fixtures::REMOTE);
    let branch: Box<[u8]> = Box::from(&b"temper/7"[..]);
    let commit = fixtures::COMMIT;
    [
        (
            Command::Remote { remote: remote.clone(), format: invocation::Format::Sha1 },
            &[b"remote", b"add", b"origin", b"--", fixtures::REMOTE],
        ),
        (
            Command::Store { remote: remote.clone(), directory: Box::from(&b"making/1"[..]) },
            &[
                b"clone",
                b"--bare",
                b"--filter=blob:none",
                b"--single-branch",
                b"--no-tags",
                b"--quiet",
                b"--",
                fixtures::REMOTE,
                b"making/1",
            ],
        ),
        (Command::CountObjects { store: Some(Token::new(9)) }, &[b"count-objects", b"-v"]),
        (Command::CountObjects { store: None }, &[b"count-objects", b"-v"]),
        (
            Command::Repack { store: Some(Token::new(9)) },
            &[b"-c", b"remote.origin.promisor=true", b"repack", b"-A", b"-d", b"-l", b"-q", b"--no-write-bitmap-index"],
        ),
        (
            Command::Repack { store: None },
            &[b"-c", b"remote.origin.promisor=true", b"repack", b"-A", b"-d", b"-l", b"-q", b"--no-write-bitmap-index"],
        ),
        (Command::Format, &[b"rev-parse", b"--show-object-format"]),
        (
            Command::Clone { directory: Box::from(&b"app"[..]), format: invocation::Format::Sha256, bare: false },
            &[b"init", b"--quiet", b"--object-format=sha256", b"--", b"app"],
        ),
        (
            Command::Clone {
                directory: Box::from(b"gitdirs/0/fabrig".as_slice()),
                format: invocation::Format::Sha1,
                bare: true,
            },
            &[b"init", b"--quiet", b"--bare", b"--object-format=sha1", b"--", b"gitdirs/0/fabrig"],
        ),
        (Command::UpdateRef { into: git::Into::Fetched, commit }, &[b"update-ref", b"refs/temper/fetched", ID]),
        (Command::Version, &[b"version"]),
        (
            Command::Origin { directory: Box::from(&b"gitdirs/0/fabrig"[..]) },
            &[b"--git-dir=gitdirs/0/fabrig", b"config", b"--local", b"--get", b"remote.origin.url"],
        ),
        (
            Command::Clone { directory: Box::from(&b"app"[..]), format: invocation::Format::Sha1, bare: false },
            &[b"init", b"--quiet", b"--object-format=sha1", b"--", b"app"],
        ),
        (
            Command::Fetch {
                workspace: Token::new(7),
                into: git::Into::Fetched,
                remote: remote.clone(),
                source: Source::Branch(branch.clone()),
            },
            &[
                b"fetch",
                b"--porcelain",
                b"--verbose",
                b"--filter=blob:none",
                b"--no-tags",
                b"--no-write-fetch-head",
                b"--no-recurse-submodules",
                b"--",
                fixtures::REMOTE,
                b"+refs/heads/temper/7:refs/temper/7/fetched",
            ],
        ),
        (
            Command::Fetch {
                workspace: Token::new(7),
                into: git::Into::Landing,
                remote: remote.clone(),
                source: Source::Branch(branch.clone()),
            },
            &[
                b"fetch",
                b"--porcelain",
                b"--verbose",
                b"--filter=blob:none",
                b"--no-tags",
                b"--no-write-fetch-head",
                b"--no-recurse-submodules",
                b"--",
                fixtures::REMOTE,
                b"+refs/heads/temper/7:refs/temper/7/landing",
            ],
        ),
        (
            Command::Fetch {
                workspace: Token::new(7),
                into: git::Into::Fetched,
                remote: remote.clone(),
                source: Source::Commit(commit),
            },
            &[
                b"fetch",
                b"--porcelain",
                b"--verbose",
                b"--filter=blob:none",
                b"--no-tags",
                b"--no-write-fetch-head",
                b"--no-recurse-submodules",
                b"--",
                fixtures::REMOTE,
                b"+abababababababababababababababababababab:refs/temper/7/fetched",
            ],
        ),
        (
            Command::Fetch {
                workspace: Token::new(7),
                into: git::Into::Fetched,
                remote: remote.clone(),
                source: Source::Default,
            },
            &[
                b"fetch",
                b"--porcelain",
                b"--verbose",
                b"--filter=blob:none",
                b"--no-tags",
                b"--no-write-fetch-head",
                b"--no-recurse-submodules",
                b"--",
                fixtures::REMOTE,
                b"+HEAD:refs/temper/7/fetched",
            ],
        ),
        (
            Command::FetchCounts { workspace: Token::new(7), remote: remote.clone() },
            &[
                b"fetch",
                b"--porcelain",
                b"--verbose",
                b"--no-filter",
                b"--no-tags",
                b"--no-write-fetch-head",
                b"--no-recurse-submodules",
                b"--",
                fixtures::REMOTE,
                b"+refs/temper/counts:refs/temper/7/counts",
            ],
        ),
        (Command::UpdateCounts { commit }, &[b"update-ref", b"refs/temper/counts", ID]),
        (Command::DeleteCounts, &[b"update-ref", b"-d", b"refs/temper/counts"]),
        (
            Command::Create { remote: remote.clone(), branch: branch.clone(), commit },
            &[
                b"push",
                b"--porcelain",
                b"--force-with-lease=refs/heads/temper/7:",
                b"--",
                fixtures::REMOTE,
                b"abababababababababababababababababababab:refs/heads/temper/7",
            ],
        ),
        (
            Command::CheckOut { commit },
            &[b"-c", b"remote.origin.promisor=true", b"checkout", b"--force", b"--detach", ID],
        ),
        (Command::Clean, &[b"clean", b"-ffdxq"]),
        (
            Command::Merge { theirs: commit },
            &[b"-c", b"remote.origin.promisor=true", b"merge", b"--no-commit", b"--no-ff", b"--", ID],
        ),
        (
            Command::MergeHistory { theirs: commit },
            &[
                b"-c",
                b"remote.origin.promisor=true",
                b"log",
                b"--no-merges",
                b"-M",
                b"--diff-filter=R",
                b"--format=%H",
                b"HEAD...abababababababababababababababababababab",
                b"--",
            ],
        ),
        (Command::Unmerged, &[b"diff-files", b"--name-only", b"-z", b"--diff-filter=U"]),
        (Command::Add, &[b"add", b"--all", b"--", b".", b":(exclude,glob)**/.*.log"]),
        (Command::DiffIndex { parent: commit }, &[b"diff-index", b"--cached", b"--quiet", ID, b"--"]),
        (Command::WriteTree, &[b"write-tree"]),
        (
            Command::CommitTree { tree: commit, parent: commit, merging: None },
            &[b"commit-tree", ID, b"-p", ID, b"-F", b"-"],
        ),
        (
            Command::CommitTree { tree: commit, parent: commit, merging: Some(commit) },
            &[b"commit-tree", ID, b"-p", ID, b"-p", ID, b"-F", b"-"],
        ),
        (Command::IsAncestor { expected: commit, commit }, &[b"merge-base", b"--is-ancestor", ID, ID]),
        (
            Command::Push { remote, commit, branch, expected: None },
            &[
                b"push",
                b"--porcelain",
                b"--",
                fixtures::REMOTE,
                b"abababababababababababababababababababab:refs/heads/temper/7",
            ],
        ),
    ]
}
#[test]
fn each_command_s_arguments_are_git_md_s_vector() {
    for (command, tail) in rows().into_iter().chain([(
        Command::Push {
            remote: Box::from(fixtures::REMOTE),
            commit: fixtures::COMMIT,
            branch: Box::from(&b"temper/7"[..]),
            expected: Some(fixtures::COMMIT),
        },
        &[
            &b"push"[..],
            b"--porcelain",
            b"--force-with-lease=refs/heads/temper/7:abababababababababababababababababababab",
            b"--",
            fixtures::REMOTE,
            b"abababababababababababababababababababab:refs/heads/temper/7",
        ][..],
    )]) {
        let actual = invocation::arguments_for(&operation(&command), &command);
        let mut expected = skein_lib::List::uncapped();
        for word in SETTINGS {
            expected.push(Box::from(*word)).expect("uncapped");
        }
        let tail = if tail.starts_with(&[b"-c", b"remote.origin.promisor=true"]) {
            for word in tail.get(..2).expect("promisor") {
                expected.push(Box::from(*word)).expect("uncapped");
            }
            tail.get(2..).expect("command")
        } else {
            tail
        };
        let directories: &[&[u8]] = match command {
            Command::Version | Command::Clone { .. } | Command::Store { .. } | Command::Origin { .. } => &[],
            Command::CountObjects { store: Some(_) }
            | Command::Repack { store: Some(_) }
            | Command::Format
            | Command::Fetch { .. }
            | Command::FetchCounts { .. } => &[b"--git-dir=stores/9"],
            Command::Reds(_)
            | Command::CountObjects { store: None }
            | Command::Repack { store: None }
            | Command::SnapshotHead
            | Command::SnapshotMerge { .. }
            | Command::SnapshotFetch { .. }
            | Command::SnapshotReachable { .. }
            | Command::SnapshotReset { .. }
            | Command::Remote { .. }
            | Command::UpdateRef { .. }
            | Command::UpdateCounts { .. }
            | Command::DeleteCounts
            | Command::Create { .. }
            | Command::CheckOut { .. }
            | Command::Clean
            | Command::Merge { .. }
            | Command::MergeHistory { .. }
            | Command::Unmerged
            | Command::UpdateIndex { .. }
            | Command::Add
            | Command::DiffIndex { .. }
            | Command::DiffPaths { .. }
            | Command::WriteTree
            | Command::CommitTree { .. }
            | Command::MergeBases { .. }
            | Command::BringsList { .. }
            | Command::IsAncestor { .. }
            | Command::Push { .. } => &[b"--git-dir=gitdirs/0/fabrig", b"--work-tree=workspaces/0/fabrig"],
        };
        for word in directories.iter().chain(tail.iter()) {
            expected.push(Box::from(*word)).expect("uncapped");
        }
        assert_eq!(actual.as_ref(), expected.as_slice(), "{command:?}");
    }
    for (workspace, expected) in [
        (0, b"workspaces/0/app".as_slice()),
        (7, b"workspaces/7/app"),
        (1 << 32_u32, b"workspaces/4294967296/app"),
        (u64::MAX, b"workspaces/18446744073709551615/app"),
    ] {
        assert_eq!(
            &*invocation::place(&git::Place {
                git: None,
                workspace: Token::new(workspace),
                repository: Box::from(&b"app"[..])
            }),
            expected
        );
    }
}
fn operation(command: &Command) -> git::Op {
    let at = git::Place { git: None, workspace: Token::new(0), repository: Box::from(b"fabrig".as_slice()) };
    if let Command::Format = command {
        git::Op::Clone { at, store: Token::new(9), remote: Box::from(fixtures::REMOTE) }
    } else if let Command::Merge { theirs } | Command::MergeHistory { theirs } = command {
        git::Op::Merge { at, theirs: *theirs, identity: 1, account: None }
    } else if let Command::FetchCounts { .. } | Command::UpdateCounts { .. } | Command::DeleteCounts = command {
        git::Op::Counts { at, store: Token::new(9), remote: Box::from(fixtures::REMOTE), identity: 1, account: None }
    } else {
        git::Op::Fetch {
            at,
            store: Token::new(9),
            remote: Box::from(fixtures::REMOTE),
            want: git::Want::Default,
            into: git::Into::Fetched,
            identity: 1,
            account: None,
        }
    }
}

#[test]
fn no_command_spawns_anywhere_but_the_worker_root() {
    let commit = fixtures::COMMIT;
    let mut commands = skein_lib::List::uncapped();
    for (command, _) in rows() {
        commands.push(command).expect("uncapped commands");
    }
    for command in [
        Command::Reds(crate::reds::CheckCommand::Pin),
        Command::SnapshotHead,
        Command::SnapshotMerge { head: commit, second: None },
        Command::SnapshotFetch { remote: Box::from(fixtures::REMOTE), head: commit, merging: None, second: commit },
        Command::SnapshotReachable { head: commit, merging: None, second: commit },
        Command::SnapshotReset { commit },
        Command::DiffPaths { before: commit, after: commit },
        Command::MergeBases { head: commit, upstream: commit },
        Command::BringsList { bases: Box::new([commit]), base: commit },
    ] {
        commands.push(command).expect("uncapped commands");
    }
    for command in commands.into_boxed() {
        let mut below = fixtures::Below::new();
        let mut operation = crate::operation::Operation::new(
            fixtures::OWNER,
            fixtures::OWNER,
            fixtures::DEADLINE,
            Some(operation(&command)),
            Some(fixtures::identity()),
            command.clone(),
        );
        operation.spawn(fixtures::OWNER, command, &fixtures::settings(), &fixtures::limits(), &mut below.out);
        let Some(skein_io::Request::Spawn { spawn, .. }) = below.out.io.pop() else { panic!("spawn") };
        assert!(spawn.dir.is_empty(), "spawn dir: {:?}", spawn.dir);
    }
}

#[test]
fn a_remote_operation_carries_the_token_only_in_its_header() {
    let identity = fixtures::identity();
    for kind in
        [git::Kind::Store, git::Kind::Fetch, git::Kind::Create, git::Kind::Push, git::Kind::CheckOut, git::Kind::Merge]
    {
        let environment = invocation::environment(fixtures::HOME, Some(kind), Some(&identity));
        assert_eq!(environment.len(), 12);
        assert!(environment.contains(&Box::from(b"GIT_COMMITTER_NAME=Temper Bot".as_slice())));
        assert!(environment.contains(&Box::from(b"GIT_COMMITTER_EMAIL=bot@example.test".as_slice())));
        for expected in [
            &b"GIT_CONFIG_COUNT=1"[..],
            b"GIT_CONFIG_KEY_0=http.extraHeader",
            b"GIT_CONFIG_VALUE_0=Authorization: Basic dGVtcGVyLWJvdDp0b2stMQ==",
        ] {
            assert!(environment.contains(&Box::from(expected)), "{kind:?}: {expected:?}");
        }
        for variable in &environment {
            assert_eq!(bytes::find(variable, b"tok-1"), None);
            assert!(!variable.starts_with(b"PATH="));
        }
    }
    for (command, _) in rows() {
        for argument in &invocation::arguments(&command) {
            assert_eq!(bytes::find(argument, b"tok-1"), None, "{command:?}");
        }
    }
}
#[test]
fn a_commit_is_authored_and_committed_as_the_identity() {
    let identity = fixtures::identity();
    let environment = invocation::environment(fixtures::HOME, Some(git::Kind::Commit), Some(&identity));
    let expected = [
        b"HOME=/srv/worker/home".as_slice(),
        b"LC_ALL=C",
        b"TZ=UTC",
        b"GIT_CONFIG_NOSYSTEM=1",
        b"GIT_CONFIG_GLOBAL=/dev/null",
        b"GIT_TERMINAL_PROMPT=0",
        b"GIT_ASKPASS=",
        b"GIT_COMMITTER_NAME=Temper Bot",
        b"GIT_COMMITTER_EMAIL=bot@example.test",
        b"GIT_AUTHOR_NAME=Temper Bot",
        b"GIT_AUTHOR_EMAIL=bot@example.test",
    ];
    assert_eq!(environment.len(), expected.len());
    for (actual, expected) in environment.iter().zip(expected) {
        assert_eq!(&**actual, expected);
    }
}
#[test]
fn an_operation_naming_no_identity_commits_as_the_placeholder() {
    let identity = fixtures::identity();
    for kind in
        [Some(git::Kind::Look), Some(git::Kind::Clone), None, Some(git::Kind::Make), Some(git::Kind::IsAncestor)]
    {
        let environment = invocation::environment(fixtures::HOME, kind, None);
        let expected = [
            b"HOME=/srv/worker/home".as_slice(),
            b"LC_ALL=C",
            b"TZ=UTC",
            b"GIT_CONFIG_NOSYSTEM=1",
            b"GIT_CONFIG_GLOBAL=/dev/null",
            b"GIT_TERMINAL_PROMPT=0",
            b"GIT_ASKPASS=",
            b"GIT_COMMITTER_NAME=temper",
            b"GIT_COMMITTER_EMAIL=temper@temper.invalid",
        ];
        assert_eq!(environment.len(), expected.len());
        for (actual, expected) in environment.iter().zip(expected) {
            assert_eq!(&**actual, expected, "{kind:?}");
        }
        assert_eq!(invocation::environment(fixtures::HOME, kind, Some(&identity)), environment);
    }
}
#[test]
fn the_message_is_title_blank_line_body_and_newline() {
    for (body, expected) in [(b"".as_slice(), b"Title\n".as_slice()), (b"Body", b"Title\n\nBody\n")] {
        assert_eq!(&*invocation::message(b"Title", body), expected, "{body:?}");
    }
}
#[test]
fn ids_of_forty_and_sixty_four_digits_convert_both_ways() {
    for (digits, expected) in [
        (ID, Some(fixtures::COMMIT)),
        (b"cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd", Some(fixtures::COMMIT_256)),
        (b"abababababababababababababababababababa", None),
        (b"ABABABABABABABABABABABABABABABABABABABAB", None),
        (b"abababababababababababababababababababag", None),
    ] {
        assert_eq!(invocation::object(digits), expected, "{digits:?}");
        if let Some(commit) = expected {
            assert_eq!(&*invocation::hex(commit), digits);
        }
    }
}
#[test]
fn limits_breaking_git_md_s_rules_are_refused_naming_the_limit() {
    let valid = fixtures::limits();
    assert_eq!(limits::check(&valid, &fixtures::checkout()), Ok(()));
    for (limits, expected) in [
        (crate::Limits { operations: 1, ..valid }, Unfit::Operations),
        (crate::Limits { stdout_bytes: 0, ..valid }, Unfit::StdoutBytes),
        (crate::Limits { stderr_bytes: 511, ..valid }, Unfit::StderrBytes),
        (crate::Limits { read_bytes: 0, ..valid }, Unfit::ReadBytes),
        (crate::Limits { sweep_entries: 0, ..valid }, Unfit::SweepEntries),
        (crate::Limits { sweep_bytes: 254, ..valid }, Unfit::SweepBytes),
    ] {
        assert_eq!(limits::check(&limits, &fixtures::checkout()), Err(expected), "{expected:?}");
    }
    assert_eq!(limits::largest_output(&fixtures::checkout()), Some(384));
    let mut checkout = fixtures::checkout();
    checkout.conflicts = u32::MAX;
    assert_eq!(limits::largest_output(&checkout), None);
    assert_eq!(limits::check(&valid, &checkout), Err(Unfit::StdoutBytes));
}
#[test]
fn check_accepts_limits_whose_payload_sum_exceeds_u64() {
    let limits = crate::Limits {
        operations: 1_000_000_000,
        stdout_bytes: u32::MAX,
        stderr_bytes: u32::MAX,
        read_bytes: u32::MAX,
        ..fixtures::limits()
    };
    let checkout = temper_worker_domain_checkout::Limits { message_bytes: u32::MAX, ..fixtures::checkout() };
    assert_eq!(limits::check(&limits, &checkout), Ok(()));
}
#[test]
fn the_identity_s_debug_shows_no_token_byte() {
    let printed = alloc::fmt::format(format_args!("{:?}", fixtures::identity()));
    assert!(printed.contains("Secret { len: 5 }"));
    assert!(!printed.contains("tok-1"));
    assert!(!printed.contains("116, 111, 107, 45, 49"));
}

#[test]
fn changed_paths_compare_both_trees_without_rename_detection() {
    let args = invocation::arguments(&Command::DiffPaths { before: fixtures::PARENT, after: fixtures::COMMIT });
    let settings = invocation::SETTINGS.len().checked_mul(2).expect("two arguments per setting");
    let expected: &[&[u8]] = &[
        b"diff-tree",
        b"--no-commit-id",
        b"--name-only",
        b"--no-renames",
        b"-r",
        b"-z",
        &invocation::hex(fixtures::PARENT),
        &invocation::hex(fixtures::COMMIT),
        b"--",
    ];
    let tail = args.get(settings..).expect("settings prefix");
    assert_eq!(tail.len(), expected.len());
    for (arg, expected) in tail.iter().zip(expected) {
        assert_eq!(arg.as_ref(), *expected);
    }
}

#[test]
fn brings_invocations_use_each_merge_base_as_a_positive_and_exclude_the_base() {
    let head = fixtures::COMMIT;
    let upstream = fixtures::COMMIT_256;
    let base = head;
    let sha256 = b"cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd";
    let rows: [(Command, &[&[u8]]); 2] = [
        (Command::MergeBases { head, upstream }, &[b"merge-base", b"--all", ID, sha256]),
        (
            Command::BringsList { bases: Box::new([head, upstream]), base },
            &[b"rev-list", b"--max-count=1", ID, sha256, b"^abababababababababababababababababababab", b"--"],
        ),
    ];
    for (command, expected) in rows {
        let arguments = invocation::arguments(&command);
        let actual = arguments.get(SETTINGS.len()..).expect("fixed settings");
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(expected) {
            assert_eq!(actual.as_ref(), *expected);
        }
    }
    assert_eq!(
        invocation::environment(b"/home", Some(git::Kind::Brings), None),
        invocation::environment(b"/home", None, None)
    );
}
