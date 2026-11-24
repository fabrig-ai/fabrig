//! Scripted file steps of the disposable view (protocol/git-operations.md, 1.13).
use super::fixtures::{self, Started};
use crate::{Request, invocation};
use alloc::boxed::Box;
use skein_io::kernel::{Done, Error, Fd, Op, OpenHow};
use skein_lib::{List, Token};
use temper_worker_domain_checkout::git;

const RUN: Fd = Fd::new(10);
const SOURCE: Fd = Fd::new(11);
const TARGET: Fd = Fd::new(12);
const PARENT: Fd = Fd::new(13);
const DIRECTORY: Fd = Fd::new(14);

pub(super) fn invocation(s: &mut Started, stdout: &[u8], code: u8) {
    let (child, pipes) = s.spawned();
    s.stdout(pipes[0], stdout);
    s.end(pipes[0]);
    s.end(pipes[1]);
    s.exited(child, code);
    s.closed(child, &pipes);
}
fn request(s: &mut Started, op: git::Op) {
    s.down(Request::Io {
        owner: fixtures::OWNER,
        op,
        deadline: fixtures::DEADLINE,
        identity: Some(fixtures::identity()),
    });
}
fn at() -> git::Place {
    git::Place { workspace: Token::new(0), repository: Box::from(b"fabrig".as_slice()), git: None }
}
pub(super) fn empty(s: &mut Started) {
    assert!(!s.below.out.kernel.is_empty(), "a view is pending");
    copy(s, &[], false);
}
pub(super) fn copy(s: &mut Started, files: &[(&[u8], &[u8])], short: bool) -> List<Op> {
    let mut ops = List::uncapped();
    let mut content = &b""[..];
    let mut short_write = short;
    for _ in 0_u16..512 {
        if s.below.out.kernel.is_empty() {
            return ops;
        }
        let op = s.file_operation();
        match &op {
            Op::Open { path, how: OpenHow::DirectoryNoFollow, .. } => {
                let fd = if path.ends_with(b"/.git") {
                    RUN
                } else if path.as_ref() == b"refs" || path.as_ref() == b"objects" {
                    PARENT
                } else {
                    DIRECTORY
                };
                s.complete(Ok(Done::Fd(fd)));
            }
            Op::Open { path, how: OpenHow::ReadNoFollow, .. } => {
                let mut found = None;
                for (name, bytes) in files {
                    if path.ends_with(&invocation::joined(&[b"/", name])) {
                        found = Some(*bytes);
                    }
                }
                if let Some(bytes) = found {
                    content = bytes;
                    s.complete(Ok(Done::Fd(SOURCE)));
                } else {
                    s.complete_err(Error::NotFound);
                }
            }
            Op::Open { how: OpenHow::CreateNoFollow { mode: Some(0o600) }, .. } => s.complete(Ok(Done::Fd(TARGET))),
            Op::Read { fd, buf, at } => {
                assert_eq!(*fd, SOURCE);
                assert_eq!(buf.len(), usize::try_from(fixtures::limits().read_bytes).expect("read bytes"));
                let rest = content.get(usize::try_from(*at).expect("file offset")..).expect("read offset");
                s.read(&rest[..rest.len().min(buf.len())]);
            }
            Op::Write { .. } if short_write => {
                short_write = false;
                s.complete(Ok(Done::Count(1)));
            }
            Op::Write { .. } | Op::Close { .. } | Op::Rename { no_replace: false, .. } => s.complete_ok(),
            Op::Remove { .. } => s.complete_err(Error::NotFound),
            Op::MakeDirectory { mode: 0o700, .. } => s.complete_err(Error::Exists),
            other @ (Op::Open { .. }
            | Op::Rename { .. }
            | Op::MakeDirectory { .. }
            | Op::Socket { .. }
            | Op::Bind { .. }
            | Op::Listen { .. }
            | Op::Accept { .. }
            | Op::Connect { .. }
            | Op::Recv { .. }
            | Op::Send { .. }
            | Op::Shutdown { .. }
            | Op::Append { .. }
            | Op::Truncate { .. }
            | Op::Sync { .. }
            | Op::Lock { .. }
            | Op::Stat { .. }
            | Op::HardLink { .. }
            | Op::List { .. }
            | Op::Spawn { .. }
            | Op::Wait { .. }
            | Op::Signal { .. }
            | Op::Usage
            | Op::Window { .. }
            | Op::ReadSignal { .. }
            | Op::PipeRead { .. }
            | Op::PipeWrite { .. }
            | Op::Cancel { .. }) => panic!("unexpected view step {other:?}"),
        }
        ops.push(op).expect("uncapped steps");
    }
    panic!("view settles");
}
fn contains(ops: &List<Op>, expected: Op) {
    assert!(ops.as_slice().contains(&expected), "missing {expected:?}");
}

#[test]
fn checkout_copies_in_chunks_by_rename_and_removes_absent_merge_state() {
    let mut s = Started::new();
    request(&mut s, git::Op::CheckOut { at: at(), commit: fixtures::COMMIT, clean: false, identity: 1, account: None });
    invocation(&mut s, b"", 0);
    assert_eq!(s.done(), None, "the terminal waits for its view");
    let head = [b'h'; 150];
    let ops = copy(&mut s, &[(b"HEAD", &head), (b"refs/temper/counts", b"counts\n")], true);
    assert_eq!(
        *ops.get(0).expect("first step"),
        Op::Open {
            root: Fd::new(3),
            path: b"workspaces/0/fabrig/.git".as_slice().into(),
            how: OpenHow::DirectoryNoFollow
        }
    );
    assert_eq!(
        *ops.get(1).expect("source step"),
        Op::Open { root: Fd::new(3), path: b"gitdirs/0/fabrig/HEAD".as_slice().into(), how: OpenHow::ReadNoFollow }
    );
    contains(
        &ops,
        Op::Open {
            root: RUN,
            path: b"temper-view.tmp".as_slice().into(),
            how: OpenHow::CreateNoFollow { mode: Some(0o600) },
        },
    );
    contains(&ops, Op::Write { fd: TARGET, bytes: Box::from([b'h'; 64].as_slice()), from: 1, at: 1 });
    contains(&ops, Op::Write { fd: TARGET, bytes: Box::from([b'h'; 64].as_slice()), from: 0, at: 64 });
    contains(&ops, Op::Write { fd: TARGET, bytes: Box::from([b'h'; 22].as_slice()), from: 0, at: 128 });
    contains(
        &ops,
        Op::Rename {
            from_dir: RUN,
            from: b"temper-view.tmp".as_slice().into(),
            to_dir: RUN,
            to: b"HEAD".as_slice().into(),
            no_replace: false,
        },
    );
    contains(&ops, Op::Remove { dir: RUN, name: b"MERGE_HEAD".as_slice().into(), directory: false });
    contains(&ops, Op::MakeDirectory { dir: RUN, name: b"refs".as_slice().into(), mode: 0o700 });
    contains(&ops, Op::MakeDirectory { dir: PARENT, name: b"temper".as_slice().into(), mode: 0o700 });
    contains(&ops, Op::Write { fd: TARGET, bytes: b"counts\n".as_slice().into(), from: 0, at: 0 });
    contains(
        &ops,
        Op::Rename {
            from_dir: DIRECTORY,
            from: b"temper-view.tmp".as_slice().into(),
            to_dir: DIRECTORY,
            to: b"counts".as_slice().into(),
            no_replace: false,
        },
    );
    contains(&ops, Op::MakeDirectory { dir: PARENT, name: b"info".as_slice().into(), mode: 0o700 });
    contains(
        &ops,
        Op::Write { fd: TARGET, bytes: b"../../../../../gitdirs/0/fabrig/objects\n".as_slice().into(), from: 0, at: 0 },
    );
    contains(
        &ops,
        Op::Rename {
            from_dir: DIRECTORY,
            from: b"temper-view.tmp".as_slice().into(),
            to_dir: DIRECTORY,
            to: b"alternates".as_slice().into(),
            no_replace: false,
        },
    );
    for op in &ops {
        if let Op::Open { path, .. } = op {
            assert_ne!(path.as_ref(), b"HEAD", "never opens the run's target, including a link");
        }
    }
    assert_eq!(s.done(), Some((fixtures::OWNER, git::Done::Succeeded)));
}

#[test]
fn a_conflicted_merge_copies_its_unmerged_index_and_merge_files() {
    let mut s = Started::new();
    request(&mut s, git::Op::Merge { at: at(), theirs: fixtures::COMMIT, identity: 1, account: None });
    invocation(&mut s, b"", 1);
    invocation(&mut s, b"", 0);
    invocation(&mut s, b"conflict\0", 0);
    let ops = copy(
        &mut s,
        &[
            (b"index", b"unmerged\0index"),
            (b"MERGE_HEAD", b"second\n"),
            (b"MERGE_MSG", b"message\n"),
            (b"MERGE_MODE", b""),
        ],
        false,
    );
    for (name, content) in [
        (b"index".as_slice(), b"unmerged\0index".as_slice()),
        (b"MERGE_HEAD", b"second\n"),
        (b"MERGE_MSG", b"message\n"),
    ] {
        contains(&ops, Op::Write { fd: TARGET, bytes: content.into(), from: 0, at: 0 });
        contains(
            &ops,
            Op::Rename {
                from_dir: RUN,
                from: b"temper-view.tmp".as_slice().into(),
                to_dir: RUN,
                to: name.into(),
                no_replace: false,
            },
        );
    }
    contains(
        &ops,
        Op::Rename {
            from_dir: RUN,
            from: b"temper-view.tmp".as_slice().into(),
            to_dir: RUN,
            to: b"MERGE_MODE".as_slice().into(),
            no_replace: false,
        },
    );
    assert_eq!(
        s.done(),
        Some((fixtures::OWNER, git::Done::Conflicted { files: Box::new([b"conflict".as_slice().into()]), cut: false }))
    );
}

#[test]
fn a_worker_read_failure_is_broken_named_view_without_remaking() {
    for error in [Error::Permission, Error::TooManyLinks] {
        let mut s = Started::new();
        request(&mut s, git::Op::Point { at: at(), into: git::Into::Landing, commit: fixtures::COMMIT });
        invocation(&mut s, b"", 0);
        s.file_operation();
        s.complete_ok();
        let Op::Open { how: OpenHow::ReadNoFollow, .. } = s.file_operation() else {
            panic!("worker read without following links")
        };
        s.complete_err(error);
        let Op::Close { .. } = s.file_operation() else { panic!("close before terminal") };
        s.complete_ok();
        let Some((_, git::Done::FailedWithOutput { fault, diagnostic })) = s.done() else { panic!("view fault") };
        assert_eq!(fault, git::Fault::Broken);
        assert_eq!(diagnostic.command(), Some(b"view".as_slice()));
        assert!(s.below.out.kernel.is_empty() && s.below.out.io.is_empty());
    }
}

#[test]
fn a_failed_checkout_writes_no_view() {
    let mut s = Started::new();
    request(&mut s, git::Op::CheckOut { at: at(), commit: fixtures::COMMIT, clean: true, identity: 1, account: None });
    invocation(&mut s, b"", 128);
    let Some((_, git::Done::FailedWithOutput { fault: git::Fault::Broken, .. })) = s.done() else {
        panic!("checkout failed")
    };
    assert!(s.below.out.kernel.is_empty());
}

fn argv(s: &Started, suffix: &[&[u8]]) {
    let mut expected = List::uncapped();
    for setting in invocation::SETTINGS {
        expected.push(Box::from(b"-c".as_slice())).expect("args");
        expected.push(Box::from(setting)).expect("args");
    }
    for word in suffix {
        expected.push(Box::from(*word)).expect("args");
    }
    let mut found = false;
    for request in &s.below.out.io {
        if let skein_io::Request::Spawn { spawn, .. } = request {
            assert_eq!(spawn.args.as_ref(), expected.as_slice());
            assert_eq!(spawn.env, invocation::environment(fixtures::HOME, None, None));
            assert!(spawn.dir.is_empty());
            found = true;
        }
    }
    assert!(found, "view making invokes git");
}

#[test]
fn a_broken_run_skips_occupied_trash_names_and_uses_the_workers_format() {
    let mut s = Started::new();
    request(&mut s, git::Op::Point { at: at(), into: git::Into::Landing, commit: fixtures::COMMIT });
    invocation(&mut s, b"", 0);
    let Op::Open { how: OpenHow::DirectoryNoFollow, .. } = s.file_operation() else { panic!("view directory") };
    s.complete_err(Error::NotADirectory);
    assert_eq!(
        s.file_operation(),
        Op::Open { root: Fd::new(3), path: b"workspaces/0/fabrig".as_slice().into(), how: OpenHow::DirectoryNoFollow }
    );
    s.complete(Ok(Done::Fd(Fd::new(22))));
    for (name, error) in [(b"1000".as_slice(), Some(Error::Exists)), (b"1001", Some(Error::Exists)), (b"1002", None)] {
        assert_eq!(
            s.file_operation(),
            Op::Rename {
                from_dir: Fd::new(22),
                from: b".git".as_slice().into(),
                to_dir: fixtures::TRASH,
                to: name.into(),
                no_replace: true
            }
        );
        if let Some(error) = error {
            s.complete_err(error);
        } else {
            s.complete_ok();
        }
    }
    assert_eq!(s.file_operation(), Op::Close { fd: Fd::new(22) });
    s.complete_ok();
    argv(&s, &[b"--git-dir=gitdirs/0/fabrig", b"rev-parse", b"--show-object-format"]);
    invocation(&mut s, b"sha256\n", 0);
    argv(&s, &[b"init", b"--quiet", b"--object-format=sha256", b"--", b"workspaces/0/fabrig"]);
    invocation(&mut s, b"", 0);
    empty(&mut s);
    assert_eq!(s.done(), Some((fixtures::OWNER, git::Done::Succeeded)));
}

#[test]
fn brings_paths_and_reds_end_without_reading_a_worker_view() {
    for (op, code, expected) in [
        (
            git::Op::Brings { at: at(), head: fixtures::COMMIT, base: fixtures::PARENT, upstream: fixtures::PARENT },
            1,
            git::Done::Brings { brings: false },
        ),
        (
            git::Op::DiffPaths { at: at(), before: fixtures::PARENT, after: fixtures::COMMIT, markdown: false },
            0,
            git::Done::Paths { files: Box::new([]) },
        ),
    ] {
        let mut s = Started::new();
        request(&mut s, op);
        invocation(&mut s, b"", code);
        assert_eq!(s.done(), Some((fixtures::OWNER, expected)));
        assert!(s.below.out.kernel.is_empty());
    }
    let mut s = Started::new();
    request(&mut s, git::Op::Reds { at: at(), action: git::Reds::Clear });
    for _ in 0_u8..16 {
        if s.below.out.kernel.is_empty() {
            break;
        }
        if let Op::Open { path, .. } = s.file_operation() {
            assert!(!path.starts_with(b"gitdirs/"), "reds writes no view");
        }
        s.complete_ok();
    }
    assert_eq!(s.done(), Some((fixtures::OWNER, git::Done::Succeeded)));
    assert!(s.below.out.kernel.is_empty());
}
