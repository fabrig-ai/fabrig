use super::fixtures;
use crate::{Event, Request, Unusable};
use alloc::boxed::Box;
use skein_io::kernel::{self, Error, Fd, Op, OpenHow};
use temper_worker_domain_checkout::git;

fn directory(dir: Fd, name: &[u8]) -> Op {
    Op::MakeDirectory { dir, name: Box::from(name), mode: 0o700 }
}
fn open(path: &[u8], how: OpenHow) -> Op {
    Op::Open { root: Fd::new(3), path: Box::from(path), how }
}
fn rename(from_dir: Fd, from: &[u8], to: &[u8]) -> Op {
    Op::Rename { from_dir, from: Box::from(from), to_dir: fixtures::TRASH, to: Box::from(to), no_replace: false }
}
fn remove(dir: Fd, name: &[u8], directory: bool) -> Op {
    Op::Remove { dir, name: Box::from(name), directory }
}
fn unmerged(component: &mut fixtures::Started, files: &[u8]) {
    component.down(Request::Io {
        owner: fixtures::OWNER,
        op: fixtures::commit_op(Some(fixtures::COMMIT)),
        deadline: fixtures::DEADLINE,
        identity: Some(fixtures::identity()),
    });
    let (child, pipes) = component.spawned();
    component.stdout(pipes[0], files);
    component.end(pipes[0]);
    component.end(pipes[1]);
    component.exited(child, 0);
    component.closed(child, &pipes);
}
fn listed(component: &mut fixtures::Started, name: &[u8]) {
    let Op::List { .. } = component.file_operation() else { panic!("expected List") };
    component.listing(&[(name, kernel::Kind::Directory)]);
    let Op::Close { .. } = component.file_operation() else { panic!("expected Close") };
    component.complete_ok();
}

#[test]
fn the_start_moves_workspaces_and_home_into_trash_and_makes_them_afresh() {
    let mut component = fixtures::Started::version_checked();
    let expected = [
        directory(Fd::new(3), b"trash"),
        open(b"trash", OpenHow::DirectoryNoFollow),
        rename(Fd::new(3), b"workspaces", b"1000"),
        rename(Fd::new(3), b"home", b"1001"),
        directory(Fd::new(3), b"workspaces"),
        directory(Fd::new(3), b"home"),
        open(b"workspaces", OpenHow::DirectoryNoFollow),
    ];
    for operation in expected {
        assert_eq!(component.file_operation(), operation);
        assert_eq!(component.below.out.events.pop(), None);
        component.complete_ok();
    }
    assert_eq!(component.below.out.events.pop(), Some(Event::Started));
}

#[test]
fn make_renames_the_old_workspace_aside_and_makes_it_empty() {
    let mut component = fixtures::Started::laid_out();
    component.down(fixtures::make(fixtures::OWNER, fixtures::WORKSPACE));
    assert_eq!(component.file_operation(), rename(fixtures::WORKSPACES, b"7", b"1000"));
    component.complete_ok();
    assert_eq!(component.file_operation(), directory(fixtures::WORKSPACES, b"7"));
    component.complete_ok();
    assert_eq!(component.done(), Some((fixtures::OWNER, git::Done::Succeeded)));
}

#[test]
fn make_accepts_only_a_missing_source_after_a_failed_rename() {
    let rows = [(Error::NotFound, true), (Error::NotEmpty, false), (Error::Permission, false)];
    for (error, creates) in rows {
        let mut component = fixtures::Started::laid_out();
        component.down(fixtures::make(fixtures::OWNER, fixtures::WORKSPACE));
        assert_eq!(component.file_operation(), rename(fixtures::WORKSPACES, b"7", b"1000"));
        component.complete_err(error);
        if creates {
            assert_eq!(component.file_operation(), directory(fixtures::WORKSPACES, b"7"));
            component.complete_ok();
            assert_eq!(component.done(), Some((fixtures::OWNER, git::Done::Succeeded)), "{error:?}");
        } else {
            assert_eq!(
                component.done(),
                Some((fixtures::OWNER, git::Done::Failed { fault: git::Fault::Broken })),
                "{error:?}"
            );
            assert_eq!(component.file_operation(), open(b"trash", OpenHow::DirectoryNoFollow));
        }
    }
}

#[test]
fn a_merging_commit_with_markers_left_ends_conflicted_staging_nothing() {
    let mut component = fixtures::Started::laid_out();
    unmerged(&mut component, b"first\0second\0");
    for path in [b"workspaces/7/app/first".as_slice(), b"workspaces/7/app/second"] {
        assert_eq!(component.file_operation(), open(path, OpenHow::ReadNoFollow));
        component.complete_ok();
        for bytes in [b"one\n<<<<".as_slice(), b"<<< ours\n", b""] {
            let Op::Read { .. } = component.file_operation() else { panic!("expected Read") };
            component.read(bytes);
        }
        let Op::Close { .. } = component.file_operation() else { panic!("expected Close") };
        component.complete_ok();
    }
    assert_eq!(
        component.done(),
        Some((
            fixtures::OWNER,
            git::Done::Conflicted { files: Box::new([Box::from(&b"first"[..]), Box::from(&b"second"[..])]) }
        ))
    );
    assert!(component.below.out.io.is_empty(), "nothing staged");
}

#[test]
fn a_conflicted_path_gone_a_directory_or_a_link_holds_no_marker() {
    for error in [Error::NotFound, Error::IsADirectory, Error::TooManyLinks, Error::NotAFile] {
        let mut component = fixtures::Started::laid_out();
        unmerged(&mut component, b"first\0");
        assert_eq!(component.file_operation(), open(b"workspaces/7/app/first", OpenHow::ReadNoFollow));
        if error == Error::IsADirectory {
            component.complete_ok();
            let Op::Read { at: 0, .. } = component.file_operation() else { panic!("expected first Read") };
            component.complete_err(error);
            let Op::Close { .. } = component.file_operation() else { panic!("expected Close") };
            component.complete_ok();
        } else {
            component.complete_err(error);
        }
        let request = component.below.out.io.pop().expect("add spawn");
        let skein_io::Request::Spawn { spawn, .. } = request else { panic!("expected Add") };
        assert_eq!(&**spawn.args.last().expect("argument"), b"--all", "{error:?}");
        assert_eq!(component.done(), None);
    }
}

#[test]
fn the_sweep_removes_files_and_moves_subdirectories_up_into_trash() {
    let mut component = fixtures::Started::version_checked();
    for _ in 0_u8..7 {
        component.file_operation();
        component.complete_ok();
    }
    assert_eq!(component.below.out.events.pop(), Some(Event::Started));
    assert_eq!(component.file_operation(), open(b"trash", OpenHow::DirectoryNoFollow));
    component.complete_ok();
    listed(&mut component, b"old");
    assert_eq!(component.file_operation(), remove(fixtures::TRASH, b"old", false));
    component.complete_err(Error::IsADirectory);
    assert_eq!(component.file_operation(), open(b"trash/old", OpenHow::DirectoryNoFollow));
    component.complete_ok();
    listed(&mut component, b"nested");
    assert_eq!(component.file_operation(), open(b"trash/old", OpenHow::DirectoryNoFollow));
    component.complete_ok();
    assert_eq!(component.file_operation(), remove(fixtures::WORKSPACES, b"nested", false));
    component.complete_err(Error::IsADirectory);
    assert_eq!(component.file_operation(), rename(fixtures::WORKSPACES, b"nested", b"1002"));
    component.complete_ok();
    assert_eq!(component.file_operation(), Op::Close { fd: fixtures::WORKSPACES });
    component.complete_ok();
    assert_eq!(component.file_operation(), open(b"trash", OpenHow::DirectoryNoFollow));
    component.complete_ok();
    listed(&mut component, b"file");
    assert_eq!(component.file_operation(), remove(fixtures::TRASH, b"file", false));
    component.complete_ok();
    component.empty_sweep();
}

#[test]
fn no_sweep_pass_starts_after_shutdown() {
    let mut component = fixtures::Started::version_checked();
    for _ in 0_u8..7 {
        component.file_operation();
        component.complete_ok();
    }
    assert_eq!(component.below.out.events.pop(), Some(Event::Started));
    assert_eq!(component.file_operation(), open(b"trash", OpenHow::DirectoryNoFollow));
    component.down(Request::Shutdown);
    component.complete_ok();
    for fd in [fixtures::TRASH, fixtures::WORKSPACES, fixtures::TRASH] {
        assert_eq!(component.file_operation(), Op::Close { fd });
        component.complete_ok();
    }
    assert!(component.below.out.kernel.is_empty());
    assert!(component.component.is_empty());
}

#[test]
fn a_scan_cancel_closes_the_open_file_before_its_terminal() {
    let mut component = fixtures::Started::laid_out();
    unmerged(&mut component, b"first\0");
    component.file_operation();
    component.down(Request::CancelIo { owner: fixtures::OWNER });
    assert_eq!(component.done(), None);
    component.complete_ok();
    assert_eq!(component.file_operation(), Op::Close { fd: fixtures::WORKSPACES });
    assert_eq!(component.done(), None);
    component.complete_ok();
    assert_eq!(
        component.done(),
        Some((
            fixtures::OWNER,
            git::Done::FailedWithOutput { fault: git::Fault::Cancelled, diagnostic: git::PushDiagnostic::empty() }
        ))
    );
    assert!(component.below.out.io.is_empty());
}

#[test]
fn a_layout_failure_closes_descriptors_and_makes_the_start_unusable() {
    let mut component = fixtures::Started::version_checked();
    for _ in 0_u8..2 {
        component.file_operation();
        component.complete_ok();
    }
    component.file_operation();
    component.complete_err(Error::Permission);
    assert_eq!(
        component.below.out.events.pop(),
        Some(Event::Unusable { why: Unusable::Layout { error: Error::Permission } })
    );
    assert_eq!(component.file_operation(), Op::Close { fd: fixtures::TRASH });
    component.complete_ok();
    assert!(component.component.is_empty());
}
