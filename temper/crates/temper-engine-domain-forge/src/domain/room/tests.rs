use super::*;
use crate::tests::{domain, env, fixtures};
use crate::{Class, Ending, Event, Name, News, RefusalCause, Subscriber, Topic, What, topics};
use alloc::boxed::Box;
use skein_forge::api;

fn settled() -> ChangeRow {
    let mut row = fixtures::opening_row();
    row.ending = Some(Ending::Cancelled);
    row.change.state = change::State::Ended;
    row
}
fn full() -> (Domain, Env<Limits>) {
    let mut d = domain();
    let mut env = env();
    env.limits.changes = 1;
    d.changes.insert(fixtures::CHANGE, settled()).unwrap();
    (d, env)
}
fn dropped(d: &mut Domain, env: &Env<Limits>) -> Box<[Request]> {
    let mut out = Queue::uncapped();
    super::super::step(d, env, Event::DropChange { task: fixtures::CHANGE }, &mut out);
    let mut requests = List::uncapped();
    while let Some(request) = out.pop() {
        requests.push(request).unwrap();
    }
    requests.into_boxed()
}

#[test]
fn only_settled_rows_with_finished_releases_can_be_dropped() {
    for (state, ending, releasing, allowed) in [
        (change::State::Producing { requested: false }, true, false, false),
        (change::State::Ended, false, false, false),
        (change::State::Ended, true, true, false),
        (change::State::Ended, true, false, true),
        (change::State::Cleared { merge: fixtures::HEAD }, true, false, true),
        (change::State::NoChange { words: b"already done".as_slice().into() }, true, false, true),
    ] {
        let (mut d, env) = full();
        let row = d.changes.get_mut(&fixtures::CHANGE).unwrap();
        row.change.state = state.clone();
        row.ending = if ending { Some(Ending::Cancelled) } else { None };
        if releasing {
            let row = super::super::release_row(
                fixtures::CHANGE,
                fixtures::CHANGE,
                crate::ReleaseEnding::Done,
                None,
                true,
                true,
            );
            d.releases.insert(row.task, row).unwrap();
        }
        assert_eq!(d.droppable(fixtures::CHANGE), allowed, "{state:?}, ending {ending}, releasing {releasing}");
        assert!(!d.droppable_last(fixtures::CHANGE));
        let out = dropped(&mut d, &env);
        assert_eq!(d.change(fixtures::CHANGE).is_none(), allowed);
        assert_eq!(out.contains(&Request::Erase { key: Key::Change(fixtures::CHANGE) }), allowed);
    }
    let d = domain();
    assert!(!d.droppable(fixtures::CHANGE), "an unknown row is not droppable");
}

#[test]
fn ended_verifying_rows_give_room_only_after_release() {
    for state in [
        change::State::Uncleared { merge: fixtures::HEAD },
        change::State::Landed { merge: fixtures::HEAD, since: env().wall },
        change::State::Red { merge: fixtures::HEAD, since: env().wall },
    ] {
        for (ending, releasing, allowed) in [(false, false, false), (true, true, false), (true, false, true)] {
            let (mut d, env) = full();
            let row = d.changes.get_mut(&fixtures::CHANGE).unwrap();
            row.change.state = state.clone();
            row.ending = if ending { Some(Ending::Cancelled) } else { None };
            if releasing {
                let row = super::super::release_row(
                    fixtures::CHANGE,
                    fixtures::CHANGE,
                    crate::ReleaseEnding::Done,
                    None,
                    true,
                    true,
                );
                d.releases.insert(row.task, row).unwrap();
            }
            assert!(!d.droppable(fixtures::CHANGE));
            assert_eq!(
                d.droppable_last(fixtures::CHANGE),
                allowed,
                "{state:?}, ending {ending}, releasing {releasing}"
            );
            let out = dropped(&mut d, &env);
            assert_eq!(d.change(fixtures::CHANGE).is_none(), allowed);
            assert_eq!(out.contains(&Request::Erase { key: Key::Change(fixtures::CHANGE) }), allowed);
        }
    }
    assert!(!domain().droppable_last(fixtures::CHANGE), "an unknown row cannot give room last");
}

#[test]
fn live_states_never_give_room_even_with_an_ending() {
    let head = fixtures::HEAD;
    let base = fixtures::BASE;
    let ready_since = env().wall;
    for state in [
        change::State::Producing { requested: false },
        change::State::Opening { requested: false },
        change::State::Recreating { head },
        change::State::Reopening { requested: false, retarget: false },
        change::State::Checking { head },
        change::State::Gating { head, asked: Box::new([]) },
        change::State::CancellingGates { head },
        change::State::Ready { head, ready_since },
        change::State::Preparing { from: head, main: base, ready_since },
        change::State::Queued { head, ready_since },
        change::State::First { head, ready_since },
        change::State::Updating { from: head, base, ready_since },
        change::State::WaitingProducer { base },
        change::State::Resolving { requested: false, base },
        change::State::Repairing { requested: false, why: change::Repair::Ci },
        change::State::Landing { head },
        change::State::Held { was: Box::new(change::State::Producing { requested: false }), why: change::Hold::Failed },
    ] {
        let (mut d, env) = full();
        d.changes.get_mut(&fixtures::CHANGE).unwrap().change.state = state.clone();
        assert!(!d.droppable(fixtures::CHANGE), "{state:?}");
        assert!(!d.droppable_last(fixtures::CHANGE), "{state:?}");
        assert!(dropped(&mut d, &env).is_empty());
        assert!(d.change(fixtures::CHANGE).is_some());
    }
}

#[test]
fn a_root_selected_verifying_row_gives_room_beside_goal_pinned_settled_rows() {
    for state in [
        change::State::Uncleared { merge: fixtures::HEAD },
        change::State::Landed { merge: fixtures::HEAD, since: env().wall },
        change::State::Red { merge: fixtures::HEAD, since: env().wall },
    ] {
        let (mut d, mut env) = full();
        env.limits.changes = 2;
        d.changes.get_mut(&fixtures::CHANGE).unwrap().change.state = state;
        let mut other = settled();
        other.task = 52;
        other.change.task = 52;
        other.goal = Some(70);
        d.changes.insert(52, other).unwrap();
        assert!(d.droppable_last(fixtures::CHANGE));
        assert!(d.droppable(52));
        let out = dropped(&mut d, &env);
        assert!(out.contains(&Request::Erase { key: Key::Change(fixtures::CHANGE) }));
        assert!(d.change(fixtures::CHANGE).is_none());
        assert_eq!(d.change(52).unwrap().goal, Some(70), "the live goal's settled row stays");
        let (opening, _) = fixtures::opening();
        d.repositories = opening.repositories;
        let mut row = fixtures::opening_row();
        row.task = 99;
        row.change.task = 99;
        super::super::step(&mut d, &env, Event::Change { row }, &mut Queue::uncapped());
        assert!(d.change(99).is_some(), "the new change registers");
        assert_eq!(d.changes.len(), env.limits.changes);
    }
}

#[test]
fn drop_requires_a_full_table_and_no_landing_member_even_after_landing_ends() {
    use temper_engine_domain_forge_landing as landing;
    for state in [
        change::State::Ended,
        change::State::Uncleared { merge: fixtures::HEAD },
        change::State::Landed { merge: fixtures::HEAD, since: env().wall },
        change::State::Red { merge: fixtures::HEAD, since: env().wall },
    ] {
        let (mut d, mut env) = full();
        d.changes.get_mut(&fixtures::CHANGE).unwrap().change.state = state;
        env.limits.changes = 2;
        assert!(d.droppable(fixtures::CHANGE) || d.droppable_last(fixtures::CHANGE));
        assert!(dropped(&mut d, &env).is_empty());
        assert!(d.change(fixtures::CHANGE).is_some());
        env.limits.changes = 1;
        for phase in [landing::Phase::Waiting, landing::Phase::Landed, landing::Phase::Cancelled] {
            let row = crate::LandingRow {
                landing: landing::Landing {
                    round: temper_engine_domain_forge_landing::Round::new(Box::from(b"temper/1/l61".as_slice())),

                    task: 60,
                    project: 5,
                    phase,
                    members: Box::new([landing::Member {
                        left: None,
                        change: fixtures::CHANGE,
                        repository: landing::Repository { forge: 1, repository: 2 },
                        pull: 1,
                        head: None,
                        base: None,
                        candidate: None,
                        gates: Box::new([]),
                        asked: None,
                        landed: None,
                    }]),
                    preparation: 0,
                    stale_proofs: 0,
                    branch: Box::from(b"main".as_slice()),
                },
                asks: Box::new([None]),
                entries: Box::new([]),
            };
            d.landing.rows.insert(60, row).unwrap();
            assert!(!d.droppable(fixtures::CHANGE));
            assert!(!d.droppable_last(fixtures::CHANGE));
            assert!(dropped(&mut d, &env).is_empty());
            assert!(d.change(fixtures::CHANGE).is_some());
        }
    }
}

#[test]
fn dropping_erases_task_accounts_and_late_reads_cannot_recreate_them() {
    let (mut d, _) = fixtures::opening();
    let task = fixtures::CHANGE;
    fixtures::step_change(&mut d);
    assert_eq!(d.steps.len(), 1, "a change step is in flight");
    let step_owner = *d.steps.iter().next().unwrap().0;
    let subscriber = Subscriber {
        task: 70,
        number: 1,
        topic: Topic::Landings { repository: fixtures::opening_row().repository, branch: b"main".as_slice().into() },
        own_change: None,
        goal_tasks: Some(Box::new([task])),
        paths: Box::new([]),
    };
    d.subscriptions.insert((70, subscriber.topic.clone()), subscriber).unwrap();
    d.changes.get_mut(&task).unwrap().pull = Some(1);
    topics::refresh(&mut d, &env(), task, &mut Queue::uncapped());
    assert_eq!(d.goal_file_reads.len(), 1, "a goal file read is in flight");
    let read_owner = *d.goal_file_reads.iter().next().unwrap().0;
    let mut row = settled();
    row.queue_repairs = 2;
    d.changes.insert(task, row).unwrap();
    d.panel_ci.insert(task, change::Ci { head: fixtures::HEAD, status: change::Status::Passed }).unwrap();
    d.settling.insert(task, crate::settle::Settle::NEW).unwrap();
    d.start_conflicts.insert(task, fixtures::BASE).unwrap();
    d.procedure_wakes.insert(task, true).unwrap();
    d.change_due.insert(task, env().wall).unwrap();
    d.goal_files.insert(task, topics::Files { head: fixtures::HEAD, paths: Some(Box::new([])) }).unwrap();
    d.landed.insert(fixtures::HEAD, task).unwrap();
    d.landed.insert(fixtures::BASE, task).unwrap();
    let orphan = [9; 32];
    d.landed.insert(orphan, 999).unwrap();
    let live = [10; 32];
    let mut other = fixtures::opening_row();
    other.task = 52;
    other.change.task = 52;
    d.changes.insert(52, other).unwrap();
    d.landed.insert(live, 52).unwrap();
    let mut env = env();
    env.limits.changes = 2;
    assert_eq!(d.queue_repairs(fixtures::opening_row().repository, b"main"), 2);
    let out = dropped(&mut d, &env);
    assert!(d.panel_ci.get(&task).is_none());
    assert!(d.settling.get(&task).is_none());
    assert!(d.start_conflicts.get(&task).is_none());
    assert!(d.procedure_wakes.get(&task).is_none());
    assert!(d.change_due.get(&task).is_none());
    assert!(d.goal_files.get(&task).is_none());
    assert!(d.steps.get(&step_owner).is_none());
    assert!(d.goal_file_reads.get(&read_owner).is_none());
    for commit in [fixtures::HEAD, fixtures::BASE, orphan] {
        assert!(!d.landed.contains_key(&commit));
        assert!(out.contains(&Request::Erase { key: Key::Landed(commit) }));
    }
    assert_eq!(d.landed.get(&live), Some(&52));
    assert_eq!(d.queue_repairs(fixtures::opening_row().repository, b"main"), 0);
    assert!(!d.change_tasks().contains(&task));
    assert!(!d.changes_for(fixtures::opening_row().repository, env.limits.changes).contains(&task));
    assert!(d.panel(task, jig_domain_people::ApplicationScope::Project(5), env.wall, &env.limits).is_none());
    let mut marked = false;
    for request in &out {
        if let Request::Marked { tasks } = request {
            marked |= tasks.contains(&task) && tasks.contains(&crate::views::changes_subject(5));
        }
    }
    assert!(marked);
    topics::answer(&mut d, &env, read_owner, Err(api::Error::Missing), &mut Queue::uncapped());
    super::super::step_read(&mut d, &env, step_owner, Err(api::Error::Missing), &mut Queue::uncapped());
    assert!(d.goal_files.get(&task).is_none());
    assert!(d.change(task).is_none());
    assert!(d.settling.get(&task).is_none());
}

#[test]
fn absent_goal_changes_are_over_not_unknown_for_overlap() {
    let (mut d, env) = full();
    let subscriber = Subscriber {
        task: 70,
        number: 1,
        topic: Topic::Landings { repository: fixtures::opening_row().repository, branch: b"main".as_slice().into() },
        own_change: None,
        goal_tasks: Some(Box::new([fixtures::CHANGE])),
        paths: Box::new([]),
    };
    let news = News::Landing {
        before: fixtures::BASE,
        after: fixtures::HEAD,
        files: Some(Box::new([b"src/file".as_slice().into()])),
    };
    assert_eq!(topics::classify(&d, &subscriber, &news), Class::Kept);
    dropped(&mut d, &env);
    assert_eq!(topics::classify(&d, &subscriber, &news), Class::Kept);
}

#[test]
fn register_change_names_each_limit_and_unadopted_but_keeps_malformed_and_duplicate_bare() {
    for (branch, base, unadopted, mismatched, duplicate, count, cause) in [
        (
            129,
            4,
            false,
            false,
            false,
            1,
            Some(RefusalCause::Limit { member: "forge.name_bytes", configured: 128, needed: 129 }),
        ),
        (
            9,
            129,
            false,
            false,
            false,
            1,
            Some(RefusalCause::Limit { member: "forge.name_bytes", configured: 128, needed: 129 }),
        ),
        (9, 4, true, false, false, 1, Some(RefusalCause::Unadopted)),
        (9, 4, false, true, false, 1, None),
        (0, 4, false, false, false, 1, None),
        (9, 0, false, false, false, 1, None),
        (9, 4, false, false, true, 1, None),
        (9, 4, false, false, false, 8, Some(RefusalCause::Limit { member: "forge.changes", configured: 8, needed: 9 })),
    ] {
        let (mut d, _) = fixtures::opening();
        let mut row = fixtures::opening_row();
        let repository = row.repository;
        for number in 1..count {
            let mut retained = row.clone();
            retained.task = number;
            retained.change.task = number;
            d.changes.insert(number, retained).unwrap();
        }
        if !duplicate {
            row.task = 99;
            row.change.task = 99;
        }
        if mismatched {
            row.change.task = 100;
        }
        row.branch = alloc::vec![b'a'; branch].into_boxed_slice();
        row.base = alloc::vec![b'b'; base].into_boxed_slice();
        if unadopted {
            row.repository.repository = 3;
        }
        let task = row.task;
        let mut out = Queue::uncapped();
        super::super::step(&mut d, &env(), Event::Change { row: row.clone() }, &mut out);
        let expected = match cause {
            Some(cause) => Request::ResourceRefused {
                task,
                operation: b"register change",
                resource: Some(Name {
                    forge: row.repository.forge,
                    repository: row.repository.repository,
                    what: What::Repository,
                }),
                cause,
            },
            None => Request::Refused { task },
        };
        assert_eq!(out.pop(), Some(expected), "branch {branch}, base {base}, rows {count}");
        assert!(out.is_empty());
        assert_eq!(d.change(task).is_some(), duplicate);
        assert!(d.repository(repository).is_some());
    }
    let (mut d, _) = fixtures::opening();
    let mut row = fixtures::opening_row();
    row.task = 99;
    row.change.task = 99;
    row.branch = alloc::vec![b'a'; 128].into_boxed_slice();
    row.base = alloc::vec![b'b'; 128].into_boxed_slice();
    super::super::step(&mut d, &env(), Event::Change { row }, &mut Queue::uncapped());
    assert!(d.change(99).is_some(), "names at their byte bound register");
}
