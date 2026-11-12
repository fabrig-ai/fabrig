use super::fixtures;
use crate::{
    invocation::Command,
    sequence::{self, Next},
};
use alloc::boxed::Box;
use temper_worker_domain_checkout::git;
#[test]
fn each_operation_s_next_invocation_is_git_md_s() {
    standard_rows();
    let create = git::Op::Create {
        at: fixtures::at(),
        remote: Box::from(fixtures::REMOTE),
        branch: Box::from(&b"main"[..]),
        commit: fixtures::COMMIT,
        identity: 1,
        account: None,
    };
    let ran = Command::Create {
        remote: Box::from(fixtures::REMOTE),
        branch: Box::from(&b"main"[..]),
        commit: fixtures::COMMIT,
    };
    for (stdout, expected) in [
        (
            b"*\tabababababababababababababababababababab:refs/heads/main\t[new branch]\n".as_slice(),
            git::Done::Succeeded,
        ),
        (b"=\tabababababababababababababababababababab:refs/heads/main\t[up to date]\n", git::Done::Exists),
        (b"!\tabababababababababababababababababababab:refs/heads/main\t[rejected] (stale info)\n", git::Done::Exists),
        (
            b"!\tabababababababababababababababababababab:refs/heads/main\t[remote rejected] (hook declined)\n",
            git::Done::FailedWithOutput { fault: git::Fault::Refused, diagnostic: git::PushDiagnostic::empty() },
        ),
    ] {
        assert_eq!(sequence::after(&create, &ran, 1, stdout, b"", 0, &fixtures::checkout()), Next::End(expected));
    }
}

#[test]
fn a_point_succeeds_or_breaks_in_one_invocation() {
    let op = git::Op::Point { at: fixtures::at(), into: git::Into::Landing, commit: fixtures::COMMIT };
    let command = Command::UpdateRef { into: git::Into::Landing, commit: fixtures::COMMIT };
    assert_eq!(sequence::first(&op), Next::Run(command.clone()));
    assert!(!op.is_remote());
    for code in [0, 1, 128] {
        let expected = if code == 0 {
            git::Done::Succeeded
        } else {
            git::Done::FailedWithOutput { fault: git::Fault::Broken, diagnostic: git::PushDiagnostic::empty() }
        };
        assert_eq!(sequence::after(&op, &command, code, b"", b"", 0, &fixtures::checkout()), Next::End(expected));
    }
    assert_eq!(
        crate::invocation::environment(fixtures::HOME, Some(git::Kind::Point), Some(&fixtures::identity())),
        crate::invocation::environment(fixtures::HOME, None, None),
        "a point never carries an identity or an authorization header",
    );
}

fn standard_rows() {
    let commit = fixtures::commit_op(None);
    let merging = fixtures::commit_op(Some(fixtures::COMMIT));
    let push = fixtures::push_op(Some(fixtures::EXPECTED));
    let merge = git::Op::Merge { at: fixtures::at(), theirs: fixtures::COMMIT, identity: 1, account: None };
    let fetch = git::Op::Fetch {
        store: skein_lib::Token::new(9),
        into: git::Into::Fetched,
        at: fixtures::at(),
        remote: Box::from(fixtures::REMOTE),
        want: git::Want::Default,
        identity: 1,
        account: None,
    };
    let landing = git::Op::Fetch {
        store: skein_lib::Token::new(9),
        at: fixtures::at(),
        remote: Box::from(fixtures::REMOTE),
        want: git::Want::Default,
        into: git::Into::Landing,
        identity: 1,
        account: None,
    };
    let path_bytes = fixtures::checkout().path_bytes;
    let mut too_long = skein_lib::List::with_capacity(path_bytes.checked_add(2).expect("overlong path and NUL"));
    for _ in 0..=path_bytes {
        too_long.push(b'x').expect("overlong path fits the fixture");
    }
    too_long.push(0).expect("room for the NUL");
    let rows = [
        (&commit,Command::DiffIndex { parent:fixtures::PARENT },0,b"".as_slice(),Next::End(git::Done::Unchanged)),
        (&commit,Command::DiffIndex { parent:fixtures::PARENT },1,b"",Next::Run(Command::WriteTree)),
        (&push,Command::IsAncestor { expected:fixtures::EXPECTED,commit:fixtures::COMMIT },1,b"",Next::End(git::Done::Rejected)),
        (&merge,Command::Merge { theirs:fixtures::COMMIT },0,b"",Next::End(git::Done::Merged)),
        (&merge,Command::Merge { theirs:fixtures::COMMIT },1,b"",Next::Run(Command::Unmerged)),
        (&merge,Command::Unmerged,0,b"conflict\0",Next::End(git::Done::Conflicted { files:Box::from([Box::from(&b"conflict"[..])]) })),
        (&merge,Command::Unmerged,0,b"one\0two\0three\0four\0five\0",Next::End(git::Done::Conflicted { files:Box::from([Box::from(b"one".as_slice()),Box::from(b"two".as_slice()),Box::from(b"three".as_slice()),Box::from(b"four".as_slice())]) })),
        (&merging,Command::Unmerged,0,b"one\0two\0three\0four\0five\0",Next::End(git::Done::FailedWithOutput { fault:git::Fault::Broken,diagnostic:git::PushDiagnostic::empty() })),
        (&merge,Command::Unmerged,0,too_long.as_slice(),Next::End(git::Done::FailedWithOutput { fault:git::Fault::Broken,diagnostic:git::PushDiagnostic::empty() })),
        (&merge,Command::Unmerged,0,b"",Next::End(git::Done::FailedWithOutput { fault:git::Fault::Broken,diagnostic:git::PushDiagnostic::empty() })),
        (&fetch,Command::Fetch { workspace: fixtures::WORKSPACE, into: git::Into::Fetched,  remote:Box::from(fixtures::REMOTE),source:crate::invocation::Source::Default },0,b"* 0000000000000000000000000000000000000000 abababababababababababababababababababab refs/temper/7/fetched\n",Next::Run(Command::UpdateRef { into: git::Into::Fetched, commit:fixtures::COMMIT })),
        (&landing,Command::Fetch { workspace: fixtures::WORKSPACE, into: git::Into::Landing,  remote:Box::from(fixtures::REMOTE),source:crate::invocation::Source::Default },0,b"* 0000000000000000000000000000000000000000 abababababababababababababababababababab refs/temper/7/landing\n",Next::Run(Command::UpdateRef { into: git::Into::Landing, commit:fixtures::COMMIT })),
        (&landing,Command::Fetch { workspace: fixtures::WORKSPACE, into: git::Into::Landing,  remote:Box::from(fixtures::REMOTE),source:crate::invocation::Source::Default },0,b"* 0000000000000000000000000000000000000000 abababababababababababababababababababab refs/temper/7/fetched\n",Next::End(git::Done::FailedWithOutput { fault:git::Fault::Broken,diagnostic:git::PushDiagnostic::empty() })),
    ];
    for (op, ran, code, stdout, next) in rows {
        assert_eq!(sequence::after(op, &ran, code, stdout, b"", 0, &fixtures::checkout()), next, "{ran:?}: {code}");
    }
}

#[test]
fn standalone_ancestry_exits_are_booleans_and_faults_keep_their_diagnostic() {
    let op = git::Op::IsAncestor { at: fixtures::at(), ancestor: fixtures::PARENT, commit: fixtures::COMMIT };
    let command = Command::IsAncestor { expected: fixtures::PARENT, commit: fixtures::COMMIT };
    assert_eq!(sequence::first(&op), Next::Run(command.clone()));
    for (code, done) in [
        (0, git::Done::IsAncestor { contains: true }),
        (1, git::Done::IsAncestor { contains: false }),
        (
            128,
            git::Done::FailedWithOutput {
                fault: git::Fault::Broken,
                diagnostic: git::PushDiagnostic::new(b"bad object", 3),
            },
        ),
    ] {
        assert_eq!(sequence::after(&op, &command, code, b"", b"bad object", 3, &fixtures::checkout()), Next::End(done));
    }
}

#[test]
fn a_look_requires_the_exact_remote_and_handles_absent_repositories() {
    let op = git::Op::Look { at: fixtures::at(), remote: Box::from(fixtures::REMOTE) };
    let command = Command::Origin { directory: Box::from(&b"workspaces/7/app"[..]) };
    assert_eq!(sequence::first(&op), Next::Run(command.clone()));
    let matching = crate::invocation::joined(&[fixtures::REMOTE, b"\n"]);
    let extra = crate::invocation::joined(&[&matching, b"\n"]);
    for (code, stdout, holds) in [
        (0, matching.as_ref(), true),
        (0, fixtures::REMOTE, false),
        (0, b"https://other.example.test/app\n", false),
        (0, b"\n", false),
        (0, b"", false),
        (0, extra.as_ref(), false),
        (1, matching.as_ref(), false),
        (128, matching.as_ref(), false),
    ] {
        assert_eq!(
            sequence::after(&op, &command, code, stdout, b"", 0, &fixtures::checkout()),
            Next::End(git::Done::Looked { holds }),
            "{code}: {stdout:?}"
        );
    }
    assert_eq!(
        sequence::after(&op, &command, 2, b"", b"bad config", 3, &fixtures::checkout()),
        Next::End(git::Done::FailedWithOutput {
            fault: git::Fault::Broken,
            diagnostic: git::PushDiagnostic::new(b"bad config", 3)
        })
    );
}

#[test]
fn an_always_commit_writes_a_tree_without_comparing_the_parent() {
    for always in [false, true] {
        let mut op = fixtures::commit_op(None);
        let git::Op::Commit { always: flag, .. } = &mut op else { unreachable!("commit fixture") };
        *flag = always;
        let expected = if always { Command::WriteTree } else { Command::DiffIndex { parent: fixtures::PARENT } };
        assert_eq!(sequence::after(&op, &Command::Add, 0, b"", b"", 0, &fixtures::checkout()), Next::Run(expected));
    }
}

#[test]
fn changed_paths_keep_the_first_code_path_without_a_conflict_count_limit() {
    let op = git::Op::DiffPaths { at: fixtures::at(), before: fixtures::PARENT, after: fixtures::COMMIT };
    let command = Command::DiffPaths { before: fixtures::PARENT, after: fixtures::COMMIT };
    assert_eq!(sequence::first(&op), Next::Run(command.clone()));
    let valid: &[(&[u8], &[&[u8]])] =
        &[(b"", &[]), (b"docs/a.md\0src/lib.rs\0", &[b"src/lib.rs"]), (b"one\0two\0three\0four\0five\0", &[b"one"])];
    for (stdout, paths) in valid {
        let mut files = skein_lib::List::with_capacity(u32::try_from(paths.len()).expect("tiny list"));
        for path in *paths {
            files.push(Box::from(*path)).expect("room for expected paths");
        }
        let files = files.into_boxed();
        assert_eq!(
            sequence::after(&op, &command, 0, stdout, b"", 0, &fixtures::checkout()),
            Next::End(git::Done::Paths { files })
        );
    }
    for stdout in [b"docs/a.md".as_slice(), b"\0"] {
        assert_eq!(
            sequence::after(&op, &command, 0, stdout, b"", 0, &fixtures::checkout()),
            Next::End(git::Done::FailedWithOutput {
                fault: git::Fault::Broken,
                diagnostic: git::PushDiagnostic::empty(),
            }),
            "an incomplete or empty path cannot hide a code path"
        );
    }
}
