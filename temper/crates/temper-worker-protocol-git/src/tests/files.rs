use super::fixtures::{self, SWEEP_CURSOR};
use crate::{Event, Request, Unusable};
use alloc::boxed::Box;
use skein_io::kernel::{self, Error, Fd, Op, OpenHow};
use skein_lib::{Decimal, Duration, Time};
use temper_worker_domain_checkout::git;

// Completions before opening kept workspaces or making fresh ones.
const KEPT_BEFORE_OPEN: u8 = 9;
const FRESH_BEFORE_WORKSPACES: u8 = 4;

fn directory(dir: Fd, name: &[u8]) -> Op {
    Op::MakeDirectory { dir, name: Box::from(name), mode: 0o700 }
}
fn open(path: &[u8], how: OpenHow) -> Op {
    Op::Open { root: Fd::new(3), path: Box::from(path), how }
}
fn rename(from_dir: Fd, from: &[u8], to: &[u8]) -> Op {
    Op::Rename { from_dir, from: Box::from(from), to_dir: fixtures::TRASH, to: Box::from(to), no_replace: true }
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
#[test]
fn the_start_keeps_stores_and_replaces_workspaces_making_and_home() {
    for existing in [true, false] {
        let mut component = fixtures::Started::version_checked();
        let exists = if existing { Some(Error::Exists) } else { None };
        let missing = if existing { None } else { Some(Error::NotFound) };
        for (operation, error) in [
            (directory(Fd::new(3), b"trash"), exists),
            (open(b"trash", OpenHow::DirectoryNoFollow), None),
            (rename(Fd::new(3), b"making", b"1000"), missing),
            (rename(Fd::new(3), b"home", b"1001"), missing),
            (directory(Fd::new(3), b"workspaces"), None),
            (directory(Fd::new(3), b"gitdirs"), None),
            (directory(Fd::new(3), b"stores"), exists),
            (directory(Fd::new(3), b"making"), None),
            (directory(Fd::new(3), b"home"), None),
            (open(b"workspaces", OpenHow::DirectoryNoFollow), None),
            (open(b"gitdirs", OpenHow::DirectoryNoFollow), None),
            (open(b"stores", OpenHow::DirectoryNoFollow), None),
            (open(b"making", OpenHow::DirectoryNoFollow), None),
        ] {
            assert_eq!(component.file_operation(), operation);
            assert_eq!(component.below.out.events.pop(), None);
            if let Some(error) = error {
                component.complete_err(error);
            } else {
                component.complete_ok();
            }
        }
        component.finish_layout();
        assert_eq!(component.below.out.events.pop(), Some(Event::Started));
    }
}

#[test]
fn a_start_skips_a_trash_name_left_by_an_earlier_start() {
    let mut component = fixtures::Started::version_checked();
    component.file_operation();
    component.complete_err(Error::Exists);
    component.file_operation();
    component.complete_ok();
    assert_eq!(component.file_operation(), rename(Fd::new(3), b"making", b"1000"));
    component.complete_err(Error::Exists);
    assert_eq!(component.file_operation(), rename(Fd::new(3), b"making", b"1001"));
    component.complete_ok();
    assert_eq!(component.file_operation(), rename(Fd::new(3), b"home", b"1002"));
}

#[test]
fn make_renames_the_old_workspace_aside_and_makes_it_empty() {
    let mut component = fixtures::Started::laid_out();
    component.down(fixtures::make(fixtures::OWNER, fixtures::WORKSPACE));
    assert_eq!(component.file_operation(), rename(fixtures::WORKSPACES, b"7", b"1000"));
    component.complete_ok();
    assert_eq!(component.file_operation(), rename(fixtures::GITDIRS, b"7", b"1001"));
    component.complete_ok();
    assert_eq!(component.file_operation(), directory(fixtures::WORKSPACES, b"7"));
    component.complete_ok();
    assert_eq!(component.file_operation(), directory(fixtures::GITDIRS, b"7"));
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
            assert_eq!(component.file_operation(), rename(fixtures::GITDIRS, b"7", b"1001"));
            component.complete_err(Error::NotFound);
            assert_eq!(component.file_operation(), directory(fixtures::WORKSPACES, b"7"));
            component.complete_ok();
            assert_eq!(component.file_operation(), directory(fixtures::GITDIRS, b"7"));
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
    super::view::empty(&mut component);
    assert_eq!(
        component.done(),
        Some((
            fixtures::OWNER,
            git::Done::Conflicted {
                files: Box::new([Box::from(&b"first"[..]), Box::from(&b"second"[..])]),
                cut: false
            }
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
        assert!(spawn.args.contains(&Box::from(b"add".as_slice())), "{error:?}");
        assert_eq!(component.done(), None);
    }
}

const PARENT_CURSOR: Fd = Fd::new(31);
const MINUTE: Duration = Duration::from_secs(60);

fn sweep_file(component: &mut fixtures::Started, name: &[u8], result: Result<kernel::Done, Error>) {
    component.sweep_page(SWEEP_CURSOR, &[(name, kernel::Kind::File)]);
    assert_eq!(component.file_operation(), remove(fixtures::TRASH, name, false));
    component.complete(result);
}
fn open_parent(component: &mut fixtures::Started, name: &[u8], path: &[u8]) {
    component.sweep_page(SWEEP_CURSOR, &[(name, kernel::Kind::Directory)]);
    assert_eq!(component.file_operation(), remove(fixtures::TRASH, name, false));
    component.complete_err(Error::IsADirectory);
    assert_eq!(component.file_operation(), open(path, OpenHow::DirectoryNoFollow));
    component.complete(Ok(kernel::Done::Fd(PARENT_CURSOR)));
}
fn close_parent(component: &mut fixtures::Started, name: &[u8]) {
    component.sweep_page(PARENT_CURSOR, &[]);
    assert_eq!(component.file_operation(), Op::Close { fd: PARENT_CURSOR });
    component.complete_ok();
    assert_eq!(component.file_operation(), remove(fixtures::TRASH, name, true));
}
fn report(component: &mut fixtures::Started, path: &[u8], error: Error) {
    assert_eq!(component.below.out.events.pop(), Some(Event::Unswept { path: path.into(), error }));
}
fn fail_make(component: &mut fixtures::Started) {
    component.down(fixtures::make(fixtures::OWNER, fixtures::WORKSPACE));
    let Op::Rename { .. } = component.file_operation() else { panic!("expected Make rename") };
    component.complete_err(Error::Permission);
    assert_eq!(component.done(), Some((fixtures::OWNER, git::Done::Failed { fault: git::Fault::Broken })));
    component.component.reclaim();
}

#[test]
fn the_sweep_removes_files_and_moves_subdirectories_up_into_trash() {
    let mut component = fixtures::Started::sweep_pending();
    component.open_sweep();
    open_parent(&mut component, b"old", b"trash/old");
    component.sweep_page(PARENT_CURSOR, &[(b"nested", kernel::Kind::Directory)]);
    assert_eq!(component.file_operation(), remove(PARENT_CURSOR, b"nested", false));
    component.complete_err(Error::IsADirectory);
    assert_eq!(component.file_operation(), rename(PARENT_CURSOR, b"nested", b"1000"));
    component.complete_err(Error::Exists);
    assert_eq!(component.file_operation(), rename(PARENT_CURSOR, b"nested", b"1001"));
    component.complete_ok();
    close_parent(&mut component, b"old");
    component.complete_ok();
    sweep_file(&mut component, b"file", Ok(kernel::Done::Nothing));
    component.end_sweep();
    component.empty_sweep();
    assert!(component.below.out.kernel.is_empty());
}

#[test]
fn the_sweep_sets_aside_an_entry_and_continues_on_the_same_cursor() {
    let mut component = fixtures::Started::sweep_pending();
    component.open_sweep();
    sweep_file(&mut component, b"1", Err(Error::Permission));
    sweep_file(&mut component, b"2", Ok(kernel::Done::Nothing));
    sweep_file(&mut component, b"3", Ok(kernel::Done::Nothing));
    component.end_sweep();
    report(&mut component, b"trash/1", Error::Permission);
    assert_eq!(component.below.out.events.pop(), None);
    component.open_sweep();
}

#[test]
fn the_sweep_reports_the_stuck_child_not_its_nonempty_parent() {
    let mut component = fixtures::Started::sweep_pending();
    component.open_sweep();
    open_parent(&mut component, b"4", b"trash/4");
    component.sweep_page(PARENT_CURSOR, &[(b"target", kernel::Kind::Directory), (b"a", kernel::Kind::File)]);
    assert_eq!(component.file_operation(), remove(PARENT_CURSOR, b"target", false));
    component.complete_err(Error::IsADirectory);
    assert_eq!(component.file_operation(), rename(PARENT_CURSOR, b"target", b"1000"));
    component.complete_err(Error::Permission);
    assert_eq!(component.file_operation(), remove(PARENT_CURSOR, b"a", false));
    component.complete_ok();
    close_parent(&mut component, b"4");
    component.complete_err(Error::NotEmpty);
    sweep_file(&mut component, b"5", Err(Error::NotFound));
    component.end_sweep();
    report(&mut component, b"trash/4/target", Error::Permission);
    assert_eq!(component.below.out.events.pop(), None);
}

#[test]
fn a_set_aside_path_is_reported_again_only_on_change_or_return() {
    let mut component = fixtures::Started::sweep_pending();
    component.open_sweep();
    for (error, reported) in [(Error::Permission, true), (Error::Permission, false), (Error::ReadOnly, true)] {
        sweep_file(&mut component, b"1", Err(error));
        component.end_sweep();
        if reported {
            report(&mut component, b"trash/1", error);
        }
        assert_eq!(component.below.out.events.pop(), None);
        component.fire(component.now.saturating_add(MINUTE));
        component.open_sweep();
    }
    sweep_file(&mut component, b"1", Ok(kernel::Done::Nothing));
    component.end_sweep();
    component.open_sweep();
    sweep_file(&mut component, b"1", Err(Error::ReadOnly));
    component.end_sweep();
    report(&mut component, b"trash/1", Error::ReadOnly);
    assert_eq!(component.below.out.events.pop(), None);
}

#[test]
fn a_full_report_table_makes_room_only_at_the_end_of_a_pass() {
    let mut component = fixtures::Started::sweep_pending();
    component.open_sweep();
    for number in 1..=64 {
        let name = Decimal::of(number);
        sweep_file(&mut component, name.as_bytes(), Err(Error::Permission));
        let path = crate::invocation::joined(&[b"trash/", name.as_bytes()]);
        report(&mut component, &path, Error::Permission);
    }
    sweep_file(&mut component, b"65", Err(Error::Permission));
    component.end_sweep();
    assert_eq!(component.below.out.events.pop(), None);
    component.fire(component.now.saturating_add(MINUTE));
    component.open_sweep();
    sweep_file(&mut component, b"1", Ok(kernel::Done::Nothing));
    for number in 2..=64 {
        sweep_file(&mut component, Decimal::of(number).as_bytes(), Err(Error::Permission));
    }
    sweep_file(&mut component, b"65", Err(Error::Permission));
    assert_eq!(component.below.out.events.pop(), None);
    component.end_sweep();
    component.open_sweep();
    sweep_file(&mut component, b"65", Err(Error::Permission));
    component.end_sweep();
    report(&mut component, b"trash/65", Error::Permission);
    assert_eq!(component.below.out.events.pop(), None);
}

#[test]
fn a_stalled_sweep_retries_a_minute_after_ending_then_repeats_on_progress() {
    let mut component = fixtures::Started::sweep_pending();
    component.open_sweep();
    sweep_file(&mut component, b"1", Err(Error::Permission));
    assert_eq!(component.component.next_deadline(), None);
    let end = Time::from_nanos(17);
    component.now = end;
    component.end_sweep();
    let due = end.saturating_add(MINUTE);
    let before = Time::from_nanos(due.as_nanos().checked_sub(1).expect("nonzero deadline"));
    assert_eq!(component.component.next_deadline(), Some(due));
    assert!(!component.component.is_due(before));
    component.fire(before);
    assert!(component.below.out.kernel.is_empty());
    assert!(component.component.is_due(due));
    component.fire(due);
    assert_eq!(component.component.next_deadline(), None);
    component.open_sweep();
    sweep_file(&mut component, b"1", Ok(kernel::Done::Nothing));
    component.end_sweep();
    component.empty_sweep();
    assert_eq!(component.component.next_deadline(), None);
    assert!(component.below.out.kernel.is_empty());
}

#[test]
fn a_make_ending_during_an_empty_pass_requests_another_pass() {
    let mut component = fixtures::Started::laid_out();
    fail_make(&mut component);
    component.open_sweep();
    let list = component.below.out.kernel.pop().expect("pass listing in flight");
    fail_make(&mut component);
    assert!(component.below.out.kernel.is_empty(), "one pass at a time");
    component.below.out.kernel.push(list);
    component.end_sweep();
    component.empty_sweep();
    assert!(component.below.out.kernel.is_empty());
    assert_eq!(component.component.next_deadline(), None);
}

#[test]
fn failure_to_open_trash_is_reported_and_retried() {
    let mut component = fixtures::Started::sweep_pending();
    assert_eq!(component.file_operation(), open(b"trash", OpenHow::DirectoryNoFollow));
    component.complete_err(Error::Permission);
    report(&mut component, b"trash", Error::Permission);
    assert_eq!(component.below.out.events.pop(), None);
    assert!(component.below.out.kernel.is_empty());
    assert_eq!(component.component.next_deadline(), Some(component.now.saturating_add(MINUTE)));
}

#[test]
fn an_unremovable_trash_entry_holds_back_no_other() {
    let mut component = fixtures::Started::sweep_pending();
    component.open_sweep();
    sweep_file(&mut component, b"first", Err(Error::Permission));
    sweep_file(&mut component, b"later", Ok(kernel::Done::Nothing));
    component.end_sweep();
    report(&mut component, b"trash/first", Error::Permission);
    assert_eq!(component.component.sweep_state(), (true, 1, true, Some(1000)));
    component.open_sweep();
    sweep_file(&mut component, b"first", Err(Error::Permission));
    component.end_sweep();
    assert_eq!(component.component.sweep_state(), (true, 2, true, Some(1000)));
    assert_eq!(component.component.next_deadline(), Some(component.now.saturating_add(MINUTE)));
    assert!(component.below.out.kernel.is_empty());
}

#[test]
fn each_refused_trash_removal_is_counted_across_passes() {
    let mut component = fixtures::Started::sweep_pending();
    component.open_sweep();
    sweep_file(&mut component, b"first", Err(Error::Permission));
    sweep_file(&mut component, b"second", Err(Error::Permission));
    sweep_file(&mut component, b"third", Ok(kernel::Done::Nothing));
    component.end_sweep();
    assert_eq!(component.component.sweep_state(), (true, 2, true, Some(1000)));
    component.open_sweep();
    sweep_file(&mut component, b"first", Err(Error::Permission));
    sweep_file(&mut component, b"second", Err(Error::Permission));
    component.end_sweep();
    assert_eq!(component.component.sweep_state(), (true, 4, true, Some(1000)));
}

#[test]
#[ignore = "worker disk: the sweep starts a pass only at a Make, a start or its retry, so what a Store or a run's git directory made again sets aside waits for the next (temper/docs/design/protocol/git-sweep.md, 4)"]
fn what_a_store_sets_aside_is_swept_without_a_make() {
    let mut component = fixtures::Started::laid_out();
    assert_eq!(component.component.sweep_state(), (false, 0, false, Some(1000)));
    component.down(Request::Io {
        owner: fixtures::OWNER,
        op: git::Op::Store {
            store: skein_lib::Token::new(9),
            aside: Some(skein_lib::Token::new(8)),
            remote: Box::from(fixtures::REMOTE),
            identity: 1,
            account: None,
            git: None,
        },
        deadline: fixtures::DEADLINE,
        identity: Some(fixtures::identity()),
    });
    assert_eq!(component.file_operation(), rename(fixtures::STORES, b"8", b"1000"));
    component.complete_ok();
    assert_eq!(
        component.file_operation(),
        Op::Open { root: fixtures::STORES, path: Box::from(b"9".as_slice()), how: OpenHow::DirectoryNoFollow }
    );
    component.complete_ok();
    assert_eq!(component.file_operation(), Op::Close { fd: fixtures::WORKSPACES });
    component.complete_ok();
    assert_eq!(component.done(), Some((fixtures::OWNER, git::Done::Succeeded)));
    assert!(!component.below.out.kernel.is_empty(), "Store's aside must start a sweep without Make or start");
    component.open_sweep();
    open_parent(&mut component, b"1000", b"trash/1000");
    close_parent(&mut component, b"1000");
    component.complete_ok();
    component.end_sweep();
    component.empty_sweep();
    assert_eq!(component.component.sweep_state(), (false, 0, false, Some(1001)));
}

#[test]
fn no_sweep_pass_starts_after_shutdown() {
    let mut component = fixtures::Started::sweep_pending();
    component.open_sweep();
    open_parent(&mut component, b"4", b"trash/4");
    let Op::List { fd: PARENT_CURSOR, .. } = component.file_operation() else { panic!("expected child list") };
    component.down(Request::Shutdown);
    component.complete_err(Error::Permission);
    for fd in [
        PARENT_CURSOR,
        SWEEP_CURSOR,
        fixtures::MAKING,
        fixtures::STORES,
        fixtures::GITDIRS,
        fixtures::WORKSPACES,
        fixtures::TRASH,
    ] {
        assert_eq!(component.file_operation(), Op::Close { fd });
        component.complete_ok();
    }
    assert!(component.below.out.kernel.is_empty());
    assert_eq!(component.below.out.events.pop(), None);
    assert!(component.component.is_empty());
    assert_eq!(component.component.next_deadline(), None);
}

#[test]
fn an_armed_sweep_does_not_keep_the_component_alive_after_shutdown() {
    let mut component = fixtures::Started::sweep_pending();
    component.open_sweep();
    sweep_file(&mut component, b"1", Err(Error::Permission));
    component.end_sweep();
    let due = component.component.next_deadline().expect("armed retry");
    component.shutdown();
    assert_eq!(component.component.next_deadline(), None);
    assert!(component.component.is_empty());
    component.fire(due);
    assert!(component.below.out.kernel.is_empty());
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
            git::Done::FailedWithOutput {
                fault: git::Fault::Cancelled,
                diagnostic: git::PushDiagnostic::empty().named(b"scan")
            }
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

#[test]
fn a_kept_start_replaces_making_and_home_and_keeps_workspaces_and_stores() {
    for existing in [true, false] {
        let mut component = fixtures::Started::version_checked_keeping();
        for (operation, error) in [
            (directory(Fd::new(3), b"trash"), None),
            (open(b"trash", OpenHow::DirectoryNoFollow), None),
            (rename(Fd::new(3), b"making", b"1000"), if existing { None } else { Some(Error::NotFound) }),
            (rename(Fd::new(3), b"home", b"1001"), if existing { None } else { Some(Error::NotFound) }),
            (directory(Fd::new(3), b"workspaces"), if existing { Some(Error::Exists) } else { None }),
            (directory(Fd::new(3), b"gitdirs"), if existing { Some(Error::Exists) } else { None }),
            (directory(Fd::new(3), b"stores"), if existing { Some(Error::Exists) } else { None }),
            (directory(Fd::new(3), b"making"), None),
            (directory(Fd::new(3), b"home"), None),
            (open(b"workspaces", OpenHow::DirectoryNoFollow), None),
            (open(b"gitdirs", OpenHow::DirectoryNoFollow), None),
            (open(b"stores", OpenHow::DirectoryNoFollow), None),
            (open(b"making", OpenHow::DirectoryNoFollow), None),
        ] {
            assert_eq!(component.file_operation(), operation);
            if let Some(error) = error {
                component.complete_err(error);
            } else {
                component.complete_ok();
            }
        }
        component.finish_layout();
        assert_eq!(component.below.out.events.pop(), Some(Event::Started));
    }
}

#[test]
fn a_kept_start_refuses_workspaces_reached_through_a_link() {
    let mut component = fixtures::Started::version_checked_keeping();
    for _ in 0..KEPT_BEFORE_OPEN {
        component.file_operation();
        component.complete_ok();
    }
    assert_eq!(component.file_operation(), open(b"workspaces", OpenHow::DirectoryNoFollow));
    component.complete_err(Error::TooManyLinks);
    assert_eq!(
        component.below.out.events.pop(),
        Some(Event::Unusable { why: Unusable::Layout { error: Error::TooManyLinks } })
    );
}

#[test]
fn an_unset_start_opens_existing_workspaces_for_the_selective_sweep() {
    let mut component = fixtures::Started::version_checked();
    for _ in 0..FRESH_BEFORE_WORKSPACES {
        component.file_operation();
        component.complete_ok();
    }
    assert_eq!(component.file_operation(), directory(Fd::new(3), b"workspaces"));
    component.complete_err(Error::Exists);
    component.finish_layout();
    assert_eq!(component.below.out.events.pop(), Some(Event::Started));
}

#[test]
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the layout observer rejects any operation other than named directory operations"
)]
fn the_layout_never_names_the_pinned_agent_or_its_files() {
    for keep in [false, true] {
        let mut component =
            if keep { fixtures::Started::version_checked_keeping() } else { fixtures::Started::version_checked() };
        let mut started = false;
        for _ in 0_u8..40 {
            let operation = component.file_operation();
            let name = match &operation {
                Op::MakeDirectory { name, .. } | Op::Remove { name, .. } => name,
                Op::Open { path, .. } => path,
                Op::Rename { from, .. } => from,
                Op::List { .. } | Op::Close { .. } => b"".as_slice(),
                _ => panic!("unexpected layout operation: {operation:?}"),
            };
            assert!(name != b"agent" && !name.starts_with(b"agent/"), "pinned agent touched: {operation:?}");
            component.complete_ok();
            if component.below.out.events.pop() == Some(Event::Started) {
                started = true;
                break;
            }
        }
        assert!(started, "the layout must finish without touching the pinned files");
    }
}
