//! Saved branches join the stored release path (temper/docs/design/domain/forge/branches.md).
use super::*;
use skein_forge::api::{Answer, Error, Op, Write};

const SAVED_HOLDS: u32 = 1; // The fixture names one saved branch.
const TASK: u64 = 7;
const ROOT: u64 = 1;
const OTHER: u64 = 9;
const ENTRY: u64 = 20;
const NEXT: u64 = 21;
const SAVED: &[u8] = b"temper/1/s7";

fn name() -> Name {
    Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: What::Branch(Box::new([
            Box::from(b"temper".as_slice()),
            Box::from(b"1".as_slice()),
            Box::from(b"s7".as_slice()),
        ])),
    }
}

fn setup() -> Domain {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    d
}

fn saved(d: &mut Domain, task: u64) -> Box<[Request]> {
    outputs(d, Event::SavedWork { task, branches: Box::new([name()]) })
}

fn close(d: &mut Domain, task: u64, root: u64, ending: ReleaseEnding) -> Box<[Request]> {
    outputs(d, Event::Release { task, root, ending, reason: None, entry: ENTRY })
}

fn no_delete(requests: &[Request]) {
    for request in requests {
        if let Request::Save { record: Stored::Entry { .. }, .. } = request {
            unreachable!("this release leaves the branch alone: {request:?}");
        }
    }
}

fn deleted(d: &mut Domain, task: u64, missing: bool) {
    outputs(d, Event::Committed { entry: ENTRY });
    // The client's outbox searches for the cleanup effect before making it.
    fixtures::answer_next(d, Ok(Answer::Items { items: Box::new([]), more: false, now: env().now }));
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Write(Write::DeleteBranch { branch: SAVED.into() }));
    let result = if missing { Err(Error::Missing) } else { Ok(Answer::Done) };
    let answered = fixtures::answer(d, call, result);
    assert!(answered.contains(&Request::ContinueRelease { task }));
    let completed = outputs(d, Event::ContinueRelease { task, entry: NEXT });
    assert!(completed.contains(&Request::Released { task }));
    assert!(d.hold(&name()).is_none());
}

#[test]
fn a_closing_tasks_saved_work_is_deleted() {
    for ending in [ReleaseEnding::Done, ReleaseEnding::Cancelled] {
        let mut d = setup();
        assert_eq!(
            saved(&mut d, TASK).as_ref(),
            &[Request::Save {
                record: Stored::Hold(Hold {
                    design_writers: Box::new([]),
                    name: name(),
                    task: TASK,
                    writer: None,
                    drift: None
                }),
                release: None,
                read_afresh: None,
            }]
        );
        let closing = close(&mut d, TASK, ROOT, ending);
        assert!(!closing.contains(&Request::Released { task: TASK }), "delete precedes Released");
        deleted(&mut d, TASK, false);
    }
}

#[test]
fn a_saved_branch_already_gone_counts_as_deleted() {
    let mut d = setup();
    saved(&mut d, TASK);
    close(&mut d, TASK, ROOT, ReleaseEnding::Done);
    deleted(&mut d, TASK, true);
}

#[test]
fn a_failed_tasks_saved_work_stays_with_its_root_until_it_closes() {
    let mut d = setup();
    saved(&mut d, TASK);
    let closing = close(&mut d, TASK, ROOT, ReleaseEnding::Failed);
    assert!(closing.contains(&Request::Retained { task: TASK, root: ROOT, resource: name() }));
    assert!(closing.contains(&Request::Released { task: TASK }));
    assert_eq!(d.hold(&name()).expect("root retains saved branch").task, ROOT);
    no_delete(&closing);
    close(&mut d, ROOT, ROOT, ReleaseEnding::Done);
    deleted(&mut d, ROOT, false);
}

#[test]
fn a_failed_roots_saved_work_is_deleted_at_once() {
    let mut d = setup();
    saved(&mut d, TASK);
    close(&mut d, TASK, TASK, ReleaseEnding::Failed);
    deleted(&mut d, TASK, false);
}

#[test]
fn a_second_asking_takes_no_second_hold() {
    let mut d = setup();
    saved(&mut d, TASK);
    assert!(saved(&mut d, TASK).is_empty());
    assert_eq!(d.holds.len(), SAVED_HOLDS);
    close(&mut d, TASK, ROOT, ReleaseEnding::Done);
    deleted(&mut d, TASK, false);
}

#[test]
fn a_name_another_task_holds_is_left_alone() {
    let mut d = setup();
    saved(&mut d, OTHER);
    assert!(saved(&mut d, TASK).is_empty());
    let closed = close(&mut d, TASK, ROOT, ReleaseEnding::Done);
    assert!(closed.contains(&Request::Released { task: TASK }));
    assert_eq!(d.hold(&name()).expect("other task's branch").task, OTHER);
    no_delete(&closed);
}

#[test]
fn full_holds_leave_the_branch_and_the_close_completes() {
    let mut d = setup();
    for number in 0..LIMITS.holds {
        let resource = Name {
            what: What::Branch(Box::new([
                Box::from(b"temper".as_slice()),
                Box::from(b"other".as_slice()),
                Box::from(skein_lib::Decimal::of(u64::from(number)).as_bytes()),
            ])),
            ..name()
        };
        outputs(&mut d, Event::SavedWork { task: OTHER, branches: Box::new([resource]) });
    }
    assert_eq!(d.holds.len(), LIMITS.holds);
    assert!(saved(&mut d, TASK).is_empty());
    let closed = close(&mut d, TASK, ROOT, ReleaseEnding::Done);
    assert!(closed.contains(&Request::Released { task: TASK }));
    assert!(!release_failed(&closed, TASK));
    assert!(d.hold(&name()).is_none());
}

#[test]
fn a_restored_saved_hold_and_release_still_delete_the_branch() {
    let mut before = setup();
    let holding = saved(&mut before, TASK);
    let closing = close(&mut before, TASK, ROOT, ReleaseEnding::Done);
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    for request in holding.iter().chain(closing.iter()) {
        if let Request::Save { record, .. } = request
            && let Stored::Hold(_) | Stored::Release(ReleaseRow { pending: None, .. }) = record
        {
            outputs(&mut d, Event::Restore { record: record.clone() });
        }
    }
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    outputs(&mut d, Event::ContinueRelease { task: TASK, entry: ENTRY });
    deleted(&mut d, TASK, false);
}
