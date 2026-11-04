//! A change on its requester's held branch (temper/docs/design/domain/forge.md, 8.1 and 8.5).
use super::*;
use skein_forge::api::{Answer, Hint, Op, Read, UpdateStyle, Write};
use temper_engine_domain_forge_change as change;

const GOAL: u64 = 7;
const DELEGATE: u64 = 90;
const GOAL_BRANCH: &[u8] = b"temper/7/g7";
const UPDATE: u64 = fixtures::OPEN + 1;

fn name() -> Name {
    Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: What::Branch(Box::new([b"temper".as_slice().into(), b"7".as_slice().into(), b"g7".as_slice().into()])),
    }
}

fn landing_row() -> ChangeRow {
    let mut row = fixtures::opening_row();
    row.branch = GOAL_BRANCH.into();
    row.pending = None;
    row.effect = change::EffectResult::None;
    row.change.state = change::State::Checking { head: fixtures::HEAD };
    row
}

fn held_goal() -> Domain {
    held_by(Some(GOAL))
}

fn held_by(holder: Option<u64>) -> Domain {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    if let Some(task) = holder {
        outputs(&mut d, Event::Hold { entry: 0, task, resource: name(), from: None });
        fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
    }
    d
}

fn watched_goal() -> Domain {
    let mut d = held_goal();
    outputs(&mut d, Event::Names { task: GOAL, resources: Box::new([name()]) });
    fixtures::answer_next(&mut d, Ok(Answer::Items { items: Box::new([]), more: false, now: env().now }));
    fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
    outputs(
        &mut d,
        Event::Client(skein_forge_client::Event::Pushed {
            resource: skein_forge_client::Resource {
                repository: REPO,
                what: skein_forge_client::What::Branch(GOAL_BRANCH.into()),
            },
            commit: fixtures::HEAD,
        }),
    );
    d
}

fn moved(d: &mut Domain, observed: Option<skein_forge::api::Commit>) -> Box<[Request]> {
    outputs(
        d,
        Event::Hint {
            hint: Hint { repository: REPO, change: skein_forge::api::Change::Branch(GOAL_BRANCH.into()), key: None },
        },
    );
    let mut later = env();
    later.now = later.now.saturating_add(LIMITS.client.hinted);
    fire(d, &later, &mut Queue::uncapped());
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Branch { branch: GOAL_BRANCH.into() }));
    let result = match observed {
        Some(commit) => Ok(Answer::Commit(commit)),
        None => Err(skein_forge::api::Error::Missing),
    };
    fixtures::answer(d, call, result)
}

#[test]
fn drift_on_a_goal_branch_reaches_the_change_landing_it() {
    for observed in [Some(fixtures::BASE), None] {
        let mut d = watched_goal();
        outputs(&mut d, Event::Change { row: landing_row() });
        let requests = moved(&mut d, observed);
        let drift = DriftChange::Branch { expected: fixtures::HEAD, observed };
        assert_eq!(d.hold(&name()).unwrap().drift, Some(Drift { at: env().wall, change: drift }));
        assert_eq!(d.change(fixtures::CHANGE).unwrap().drift, Some(change::Hold::BranchMoved));
        for task in [GOAL, fixtures::CHANGE] {
            assert!(requests.contains(&Request::Drift { task, resource: name() }), "{requests:?}");
        }
        let row = d.change(fixtures::CHANGE).unwrap().clone();
        assert!(requests.contains(&Request::Save { read_afresh: None, release: None, record: Stored::Change(row) }));
    }
}

#[test]
fn drift_only_reaches_live_changes_of_the_held_branch_in_the_same_repository() {
    let mut other_forge = landing_row();
    other_forge.repository.forge = 3;
    let mut other_repository = landing_row();
    other_repository.repository.repository = 3;
    let mut other_branch = landing_row();
    other_branch.branch = b"temper/7/c51".as_slice().into();
    let mut ended = landing_row();
    ended.change.state = change::State::Ended;
    for row in [other_forge, other_repository, other_branch, ended] {
        let mut d = watched_goal();
        d.changes.insert(row.task, row).expect("fixture change capacity");
        let requests = moved(&mut d, Some(fixtures::BASE));
        assert!(d.hold(&name()).unwrap().drift.is_some());
        assert_eq!(d.change(fixtures::CHANGE).unwrap().drift, None);
        assert!(requests.contains(&Request::Drift { task: GOAL, resource: name() }));
        assert!(!requests.contains(&Request::Drift { task: fixtures::CHANGE, resource: name() }));
    }
}

#[test]
fn the_landing_changes_own_update_of_the_goal_branch_is_not_drift() {
    for preparing in [false, true] {
        for pending in [false, true] {
            let mut d = watched_goal();
            let mut row = landing_row();
            row.change.state = if preparing {
                change::State::Preparing { from: fixtures::HEAD, main: fixtures::BASE, ready_since: env().wall }
            } else {
                change::State::Updating { from: fixtures::HEAD, base: fixtures::BASE, ready_since: env().wall }
            };
            row.pending = if pending { Some(UPDATE) } else { None };
            outputs(&mut d, Event::Change { row });
            outputs(
                &mut d,
                Event::Enqueue {
                    entry: skein_forge_client::Entry {
                        number: UPDATE,
                        task: fixtures::CHANGE,
                        repository: REPO,
                        effect: skein_forge_client::Effect {
                            write: Write::Update {
                                number: 1,
                                style: if preparing { UpdateStyle::Rebase } else { UpdateStyle::Merge },
                            },
                            condition: skein_forge_client::Condition::Update {
                                head: fixtures::HEAD,
                                base: fixtures::BASE,
                            },
                        },
                        start: None,
                        attempt: None,
                        failures: 0,
                        queued: false,
                    },
                },
            );
            let requests = moved(&mut d, Some(fixtures::BASE));
            assert_eq!(d.hold(&name()).unwrap().drift.is_none(), pending);
            assert_eq!(d.change(fixtures::CHANGE).unwrap().drift.is_none(), pending);
            for task in [GOAL, fixtures::CHANGE] {
                assert_eq!(requests.contains(&Request::Drift { task, resource: name() }), !pending);
            }
        }
    }
}

#[test]
fn a_landing_changes_repair_writes_the_goal_branch_under_its_goals_hold() {
    for kind in [
        change::Delegate::Repair(change::Repair::Gate(1)),
        change::Delegate::Resolve { base: fixtures::BASE },
        change::Delegate::Merge { base: fixtures::BASE, from: Some(fixtures::HEAD) },
    ] {
        for holder in [GOAL, fixtures::CHANGE] {
            let mut d = held_by(Some(holder));
            let mut row = landing_row();
            row.delegate = Some((DELEGATE, kind));
            outputs(&mut d, Event::Change { row });
            let item = d
                .workspace_item(items::Seed {
                    task: DELEGATE,
                    parent: Some(fixtures::CHANGE),
                    repository: REPO,
                    run_branch: b"temper/7/r90".as_slice().into(),
                    saved_branch: None,
                })
                .unwrap();
            assert_eq!(item.holder, Some(holder));
            assert_eq!(item.push.as_deref(), Some(GOAL_BRANCH));
            let start = match kind {
                change::Delegate::Repair(_) => items::Start::Branch(GOAL_BRANCH.into()),
                change::Delegate::Resolve { base } | change::Delegate::Merge { base, .. } => {
                    items::Start::Merge { branch: GOAL_BRANCH.into(), base }
                }
                change::Delegate::Produce | change::Delegate::Gate { .. } => unreachable!("repair or resolution"),
            };
            assert_eq!(item.start, start);
            let requests = outputs(
                &mut d,
                Event::Claim {
                    task: DELEGATE,
                    attempt: 1,
                    writes: Box::new([name()]),
                    holders: Box::new([item.holder.unwrap()]),
                },
            );
            assert_eq!(
                d.hold(&name()).unwrap().writer,
                Some(Writer::Run { task: DELEGATE, attempt: 1 }),
                "{requests:?}"
            );
            outputs(
                &mut d,
                Event::Answered { task: DELEGATE, attempt: 1, pushed: Box::new([(name(), fixtures::BASE)]) },
            );
            assert_eq!(d.hold(&name()).unwrap().writer, None);
            assert_eq!(d.branch_head(&name()), Some(fixtures::BASE));
            assert_eq!(d.hold(&name()).unwrap().task, holder);
        }
    }
}

#[test]
fn facts_say_whether_a_changes_branch_is_its_requesters() {
    for holder in [Some(GOAL), Some(fixtures::CHANGE), None] {
        let mut d = held_by(holder);
        outputs(&mut d, Event::Change { row: landing_row() });
        if holder == Some(GOAL) {
            confirm_head(&mut d, fixtures::BASE);
        }
        let requests = step_at_head(&mut d, fixtures::BASE);
        let state = &d.change(fixtures::CHANGE).unwrap().change.state;
        let expected = if holder == Some(GOAL) {
            change::State::Checking { head: fixtures::BASE }
        } else {
            change::State::Held {
                was: Box::new(change::State::Checking { head: fixtures::HEAD }),
                why: change::Hold::BranchMoved,
            }
        };
        assert_eq!(state, &expected, "holder={holder:?}: {requests:?}");
    }
}

fn step_at_head(d: &mut Domain, head: skein_forge::api::Commit) -> Box<[Request]> {
    fixtures::step_change(d);
    fixtures::answer_next(d, Ok(Answer::Commit(head)));
    fixtures::answer_next(d, Ok(Answer::Commit(fixtures::BASE)));
    let mut pull = fixtures::open_pull();
    pull.head = GOAL_BRANCH.into();
    pull.commit = head;
    pull.ci = skein_forge::api::Ci::Pending;
    fixtures::answer_next(d, Ok(Answer::Pull(pull)));
    fixtures::answer_next(d, Ok(fixtures::checks(skein_forge::api::Check::Passed)));
    fixtures::answer_next(d, Ok(fixtures::checks(skein_forge::api::Check::Passed)));
    fixtures::answer_next(
        d,
        Ok(Answer::Compare {
            before: fixtures::BASE,
            after: head,
            contains_before: true,
            files: Box::new([]),
            commits: Box::new([]),
        }),
    );
    fixtures::answer_next(d, Ok(Answer::Reviews { reviews: Box::new([]), more: false }))
}

#[test]
fn an_unconfirmed_move_of_a_goal_branch_is_drift_before_the_keep_reads_it() {
    let mut d = held_goal();
    outputs(&mut d, Event::Change { row: landing_row() });
    let requests = step_at_head(&mut d, fixtures::BASE);
    assert_eq!(
        d.change(fixtures::CHANGE).unwrap().change.state,
        change::State::Held {
            was: Box::new(change::State::Checking { head: fixtures::HEAD }),
            why: change::Hold::BranchMoved,
        },
        "{requests:?}"
    );
}

fn confirm_head(d: &mut Domain, head: skein_forge::api::Commit) {
    outputs(d, Event::Claim { task: DELEGATE, attempt: 1, writes: Box::new([name()]), holders: Box::new([GOAL]) });
    outputs(d, Event::Answered { task: DELEGATE, attempt: 1, pushed: Box::new([(name(), head)]) });
}

#[test]
fn a_branch_read_before_a_confirmed_move_still_checks_the_new_head_without_drift() {
    let mut d = held_goal();
    outputs(&mut d, Event::Change { row: landing_row() });
    confirm_head(&mut d, fixtures::BASE);
    for head in [fixtures::HEAD, fixtures::BASE] {
        let requests = step_at_head(&mut d, head);
        let row = d.change(fixtures::CHANGE).unwrap();
        assert_eq!(row.change.state, change::State::Checking { head }, "{requests:?}");
        assert_eq!(row.change.repairs, 0);
    }
}
