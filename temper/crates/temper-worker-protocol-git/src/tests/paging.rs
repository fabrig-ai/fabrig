//! Merge delivery's bounded pages leave the worker's index untouched until the scan ends.
use super::fixtures::{self, Started};
use crate::{
    Request,
    invocation::{self, Command},
};
use skein_io::{
    self as io,
    kernel::{Done, Error, Fd, Op, OpenHow},
};
use skein_lib::{List, stream};
use temper_worker_domain_checkout::git;

const DIR: Fd = Fd::new(20);
const SOURCE: Fd = Fd::new(21);
const COPY: Fd = Fd::new(22);
const FILE: Fd = Fd::new(23);
fn opened(started: &mut Started, root: Fd, path: &[u8], how: OpenHow, fd: Fd) {
    assert_eq!(started.file_operation(), Op::Open { root, path: path.into(), how });
    started.complete(Ok(Done::Fd(fd)));
}
fn closed(started: &mut Started, fd: Fd) {
    assert_eq!(started.file_operation(), Op::Close { fd });
    started.complete_ok();
}
fn start() -> Started {
    let mut started = Started::new();
    started.component.checkout.conflicts = 2;
    started.component.checkout.path_bytes = 2;
    started.down(Request::Io {
        owner: fixtures::OWNER,
        op: fixtures::commit_op(Some(fixtures::COMMIT)),
        deadline: fixtures::DEADLINE,
        identity: Some(fixtures::identity()),
    });
    started
}
fn answer(started: &mut Started, command: Command, private: bool, code: u8, stdout: &[u8]) {
    let request = started.below.out.io.pop().expect("next invocation");
    let io::Request::Spawn { spawn, .. } = &request else { panic!("spawn: {request:?}") };
    assert_eq!(spawn.args, invocation::arguments_at(&command, Some(b"gitdirs/7/app"), Some(b"workspaces/7/app")));
    let mut index = List::uncapped();
    for entry in &spawn.env {
        if entry.starts_with(b"GIT_INDEX_FILE=") {
            index.push(entry.clone()).expect("index environment");
        }
    }
    if private {
        assert_eq!(
            index.as_slice(),
            [Box::<[u8]>::from(b"GIT_INDEX_FILE=/srv/worker/gitdirs/7/app/temper-scan".as_slice())]
        );
    } else {
        assert!(index.is_empty(), "only page updates and listings may use the copy");
    }
    if let Command::UpdateIndex { .. } = &command {
        assert!(private, "the worker's index is never updated page by page");
        let mut tail = List::uncapped();
        for word in spawn.args.iter().rev().take(5).rev() {
            tail.push(word.as_ref()).expect("argv tail");
        }
        assert_eq!(
            tail.as_slice(),
            [b"update-index".as_slice(), b"--add", b"--remove", b"-z", b"--stdin"],
            "page plumbing"
        );
    }
    started.below.out.io.push(request);
    let (child, pipes) = started.spawned();
    if pipes.len() == 3 {
        let expected =
            if let Command::UpdateIndex { paths } = command { paths } else { invocation::message(b"Title", b"Body") };
        let mut sent = List::uncapped();
        while usize::try_from(sent.len()).expect("test stdin") < expected.len() {
            while started.below.out.io.pop().is_some() {}
            started.component.up(
                started.now,
                io::Event::Stream { owner: pipes[0], up: stream::Up::Room },
                &mut started.below.out,
            );
            while let Some(request) = started.below.out.io.pop() {
                if let io::Request::Stream { down: stream::Down::Send(bytes), .. } = request {
                    for byte in bytes {
                        sent.push(byte).expect("test stdin");
                    }
                }
            }
        }
        assert_eq!(sent.as_slice(), expected.as_ref(), "whole NUL-terminated page on stdin");
    }
    let output = usize::from(pipes.len() == 3);
    started.stdout(pipes[output], stdout);
    started.end(pipes[output]);
    started.end(pipes[output.checked_add(1).expect("stderr pipe")]);
    started.exited(child, code);
    started.closed(child, &pipes);
}
fn copy(started: &mut Started) {
    opened(started, Fd::new(3), b"gitdirs/7/app", OpenHow::DirectoryNoFollow, DIR);
    opened(started, DIR, b"index", OpenHow::ReadNoFollow, SOURCE);
    opened(started, DIR, b"temper-scan", OpenHow::CreateNoFollow { mode: Some(0o600) }, COPY);
    let mut at = 0;
    for bytes in [b"index first chunk".as_slice(), b"index last chunk"] {
        let Op::Read { fd, at: offset, .. } = started.file_operation() else { panic!("index read") };
        assert_eq!((fd, offset), (SOURCE, at));
        started.read(bytes);
        assert_eq!(started.file_operation(), Op::Write { fd: COPY, bytes: bytes.into(), from: 0, at });
        started.complete(Ok(Done::Count(3)));
        at = at.checked_add(3).expect("test index offset");
        assert_eq!(started.file_operation(), Op::Write { fd: COPY, bytes: bytes.into(), from: 3, at });
        started.complete_ok();
        at = at
            .checked_add(u64::try_from(bytes.len().checked_sub(3).expect("short write")).expect("small index"))
            .expect("test index offset");
    }
    let Op::Read { fd, at: offset, .. } = started.file_operation() else { panic!("index eof") };
    assert_eq!((fd, offset), (SOURCE, at));
    started.read(b"");
    closed(started, COPY);
    closed(started, SOURCE);
    closed(started, DIR);
}
fn scan(started: &mut Started, paths: &[&[u8]], marked: Option<&[u8]>) {
    for path in paths {
        let path_in_tree = invocation::joined(&[b"workspaces/7/app/", path]);
        opened(started, Fd::new(3), &path_in_tree, OpenHow::ReadNoFollow, FILE);
        let Op::Read { fd: FILE, at: 0, .. } = started.file_operation() else { panic!("scan read") };
        started.read(if Some(*path) == marked { b"<<<<<<< ours\n" } else { b"settled\n" });
        let Op::Read { fd: FILE, .. } = started.file_operation() else { panic!("scan eof") };
        started.read(b"");
        closed(started, FILE);
    }
}
fn removed(started: &mut Started) {
    opened(started, Fd::new(3), b"gitdirs/7/app", OpenHow::DirectoryNoFollow, DIR);
    assert_eq!(
        started.file_operation(),
        Op::Remove { dir: DIR, name: b"temper-scan".as_slice().into(), directory: false }
    );
    assert_eq!(started.done(), None, "cleanup precedes the terminal");
    assert!(started.below.out.io.is_empty(), "no staging before cleanup");
    started.complete_ok();
    closed(started, DIR);
}
fn committed(started: &mut Started) {
    answer(started, Command::Add, false, 0, b"");
    let line = invocation::joined(&[&invocation::hex(fixtures::COMMIT), b"\n"]);
    answer(started, Command::WriteTree, false, 0, &line);
    answer(
        started,
        Command::CommitTree { tree: fixtures::COMMIT, parent: fixtures::PARENT, merging: Some(fixtures::COMMIT) },
        false,
        0,
        &line,
    );
    super::view::empty(started);
    assert_eq!(started.done(), Some((fixtures::OWNER, git::Done::Committed { commit: fixtures::COMMIT })));
    assert!(started.below.out.kernel.is_empty());
    assert!(started.below.out.io.is_empty());
}
fn first_page(started: &mut Started) {
    answer(started, Command::Unmerged, false, 0, b"aa\0bb\0cc\0dd\0ee\0");
    copy(started);
    scan(started, &[b"aa", b"bb"], None);
}
#[test]
fn five_paths_are_scanned_in_private_index_pages() {
    let mut started = start();
    first_page(&mut started);
    answer(&mut started, Command::UpdateIndex { paths: b"aa\0bb\0".as_slice().into() }, true, 0, b"");
    answer(&mut started, Command::Unmerged, true, 0, b"cc\0dd\0ee\0");
    scan(&mut started, &[b"cc", b"dd"], None);
    answer(&mut started, Command::UpdateIndex { paths: b"cc\0dd\0".as_slice().into() }, true, 0, b"");
    answer(&mut started, Command::Unmerged, true, 0, b"ee\0");
    scan(&mut started, &[b"ee"], None);
    removed(&mut started);
    committed(&mut started);
}
#[test]
fn stdout_cut_pages_drop_the_last_incomplete_path() {
    for cap in [6, 8] {
        let mut started = start();
        started.component.limits.stdout_bytes = cap;
        first_page(&mut started); // The kept stdout ends before cc, or inside it without its NUL.
        answer(&mut started, Command::UpdateIndex { paths: b"aa\0bb\0".as_slice().into() }, true, 0, b"");
        answer(&mut started, Command::Unmerged, true, 0, b"cc\0dd\0ee\0");
        scan(&mut started, &[b"cc", b"dd"], None);
        answer(&mut started, Command::UpdateIndex { paths: b"cc\0dd\0".as_slice().into() }, true, 0, b"");
        answer(&mut started, Command::Unmerged, true, 0, b"ee\0");
        scan(&mut started, &[b"ee"], None);
        removed(&mut started);
        started.component.limits.stdout_bytes = fixtures::limits().stdout_bytes;
        committed(&mut started);
    }
}
#[test]
fn a_last_page_marker_refuses_without_staging_the_worker_index() {
    let mut started = start();
    first_page(&mut started);
    answer(&mut started, Command::UpdateIndex { paths: b"aa\0bb\0".as_slice().into() }, true, 0, b"");
    answer(&mut started, Command::Unmerged, true, 0, b"cc\0dd\0ee\0");
    scan(&mut started, &[b"cc", b"dd"], None);
    answer(&mut started, Command::UpdateIndex { paths: b"cc\0dd\0".as_slice().into() }, true, 0, b"");
    answer(&mut started, Command::Unmerged, true, 0, b"ee\0");
    scan(&mut started, &[b"ee"], Some(b"ee"));
    removed(&mut started);
    super::view::empty(&mut started);
    assert_eq!(
        started.done(),
        Some((fixtures::OWNER, git::Done::Conflicted { files: Box::new([b"ee".as_slice().into()]), cut: false }))
    );
    assert!(started.below.out.io.is_empty());
}
#[test]
fn a_whole_first_listing_uses_no_private_index() {
    let mut started = start();
    answer(&mut started, Command::Unmerged, false, 0, b"aa\0bb\0");
    scan(&mut started, &[b"aa", b"bb"], None);
    assert!(started.below.out.kernel.is_empty());
    committed(&mut started);
}
#[test]
fn a_cancel_or_deadline_during_copy_closes_and_removes_it() {
    for cancel in [true, false] {
        let mut started = start();
        answer(&mut started, Command::Unmerged, false, 0, b"aa\0bb\0cc\0");
        opened(&mut started, Fd::new(3), b"gitdirs/7/app", OpenHow::DirectoryNoFollow, DIR);
        opened(&mut started, DIR, b"index", OpenHow::ReadNoFollow, SOURCE);
        assert_eq!(
            started.file_operation(),
            Op::Open {
                root: DIR,
                path: b"temper-scan".as_slice().into(),
                how: OpenHow::CreateNoFollow { mode: Some(0o600) }
            }
        );
        if cancel {
            started.down(Request::CancelIo { owner: fixtures::OWNER });
        } else {
            started.fire(fixtures::DEADLINE);
        }
        started.complete(Ok(Done::Fd(COPY)));
        closed(&mut started, COPY);
        closed(&mut started, SOURCE);
        closed(&mut started, DIR);
        removed(&mut started);
        let Some((_, git::Done::FailedWithOutput { fault, .. })) = started.done() else { panic!("stopped") };
        assert_eq!(fault, if cancel { git::Fault::Cancelled } else { git::Fault::TimedOut });
        assert!(started.below.out.io.is_empty());
    }
}
#[test]
fn an_index_copy_or_page_command_failure_removes_the_copy() {
    for copying in [true, false] {
        let mut started = start();
        if copying {
            answer(&mut started, Command::Unmerged, false, 0, b"aa\0bb\0cc\0");
            opened(&mut started, Fd::new(3), b"gitdirs/7/app", OpenHow::DirectoryNoFollow, DIR);
            let Op::Open { how: OpenHow::ReadNoFollow, .. } = started.file_operation() else { panic!("source index") };
            started.complete_err(Error::TooManyLinks);
            closed(&mut started, DIR);
        } else {
            first_page(&mut started);
            answer(&mut started, Command::UpdateIndex { paths: b"aa\0bb\0".as_slice().into() }, true, 1, b"");
        }
        removed(&mut started);
        let Some((_, git::Done::FailedWithOutput { fault: git::Fault::Broken, .. })) = started.done() else {
            panic!("failed")
        };
        assert!(started.below.out.io.is_empty());
    }
}
