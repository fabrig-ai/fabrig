//! A change gives up its decision and number, not its place (forge/room.md, 3–4).
use super::*;
use skein_forge::api::{Answer, Check, Error, Write};
use skein_lib::List;
use temper_engine_domain_forge_change as change;

const FIRST: u64 = 52;
const SECOND: u64 = 51;
const FRESH: u64 = 101;

pub(super) fn full() -> Domain {
    let mut d = refusals::adopted_domain();
    outputs(&mut d, Event::Client(skein_forge_client::Event::PauseOutbox));
    for number in 1..=u64::from(env().limits.entries) {
        outputs(&mut d, Event::Enqueue { entry: refusals::entry(number) });
        outputs(&mut d, Event::Committed { entry: number });
    }
    d
}

pub(super) fn row(task: u64) -> ChangeRow {
    let mut row = fixtures::opening_row();
    row.task = task;
    row.change.task = task;
    row.branch = format!("temper/{task}").into_bytes().into_boxed_slice();
    row.pending = None;
    row.effect = change::EffectResult::None;
    row.change.state = change::State::Opening { requested: false };
    row
}

fn start(d: &mut Domain, task: u64, entry: u64) {
    let Event::StepChange { merge_job, heard, gates, queue_repair_active, pending, kept, .. } = fixtures::change_step()
    else {
        panic!("step fixture")
    };
    outputs(d, Event::StepChange { task, entry, merge_job, heard, gates, queue_repair_active, pending, kept });
}

fn finish(d: &mut Domain) -> Box<[Request]> {
    fixtures::answer_next(d, Ok(Answer::Commit(fixtures::BASE)));
    fixtures::answer_next(d, Err(Error::Missing));
    fixtures::finish_missing(d)
}

pub(super) fn open(d: &mut Domain, task: u64, entry: u64) -> Box<[Request]> {
    start(d, task, entry);
    fixtures::answer_next(d, Ok(Answer::Commit(fixtures::HEAD)));
    finish(d)
}

fn waiters() -> Domain {
    let mut d = full();
    outputs(&mut d, Event::Change { row: row(FIRST) });
    outputs(&mut d, Event::Change { row: row(SECOND) });
    // SECOND starts first, but FIRST meets the full outbox first.
    start(&mut d, SECOND, 21);
    let (later, _) = fixtures::next_call(&mut d);
    let waited = open(&mut d, FIRST, 22);
    assert_wait(&waited, FIRST);
    fixtures::answer(&mut d, later, Ok(Answer::Commit(fixtures::HEAD)));
    let waited = finish(&mut d);
    assert_wait(&waited, SECOND);
    assert_eq!(d.change(FIRST).unwrap().room, Some(1));
    assert_eq!(d.change(SECOND).unwrap().room, Some(2));
    d
}

fn assert_wait(out: &[Request], task: u64) {
    let mut waited = false;
    for request in out {
        if let Request::ChangeDecision {
            task: owner,
            decision: change::Decision::Wait { until: None },
            entry: None,
            ..
        } = request
        {
            waited |= *owner == task;
        }
        assert!(
            !matches!(
                request,
                Request::ResourceRefused { .. }
                    | Request::Refused { .. }
                    | Request::ChangeDecision { decision: change::Decision::Hold(_), .. }
                    | Request::Save { record: Stored::Entry { .. }, .. }
            ),
            "{out:?}"
        );
    }
    assert!(waited, "{out:?}");
}

fn asks(out: &[Request]) -> Box<[u64]> {
    let mut asks = List::uncapped();
    for request in out {
        if let Request::WakeProcedure { task } = request {
            asks.push(*task).unwrap();
        }
    }
    asks.into_boxed()
}

fn entries(out: &[Request]) -> Box<[skein_forge_client::Entry]> {
    let mut entries = List::uncapped();
    for request in out {
        if let Request::Save { record: Stored::Entry { entry, .. }, .. } = request {
            entries.push(entry.clone()).unwrap();
        }
    }
    entries.into_boxed()
}

pub(super) fn free_one(d: &mut Domain) -> Box<[Request]> {
    outputs(d, Event::Withdraw { entry: 1 })
}

#[test]
fn a_projections_room_is_reserved_until_kept_or_dropped() {
    for keep in [true, false] {
        let mut d = full();
        free_one(&mut d);
        let projected =
            outputs(&mut d, Event::Project { entry: FRESH, repository: REPO, view: projection::goal_view() });
        let mut staged = None;
        for request in projected {
            if let Request::ProjectionEffect { row, entry, .. } = request {
                staged = Some((row, entry));
            }
        }
        let (projection, entry) = staged.expect("projection stages its entry");
        assert!(!d.reserve_call(&env(), Token::new(501), 1), "projection owns the last room");
        outputs(&mut d, Event::Change { row: row(FIRST) });
        assert_wait(&open(&mut d, FIRST, 22), FIRST);
        assert_eq!(room::reserved(&d), 1);
        if keep {
            let kept = outputs(&mut d, Event::KeepProjection { row: projection, entry });
            assert_eq!(entries(&kept).first().unwrap().number, FRESH);
            assert_eq!(d.entries.len(), env().limits.entries);
            assert_eq!(room::reserved(&d), 0);
            assert!(asks(&kept).is_empty());
            assert_eq!(asks(&outputs(&mut d, Event::Withdraw { entry: FRESH })).as_ref(), &[FIRST]);
        } else {
            let returned = outputs(&mut d, Event::DropProjection { entry: FRESH });
            assert_eq!(asks(&returned).as_ref(), &[FIRST]);
            assert!(!d.entries.contains_key(&FRESH));
            assert_eq!(room::reserved(&d), 1, "the ask owns the returned room");
        }
    }
}

#[test]
fn a_projection_stays_behind_an_unasked_or_asked_change() {
    for asked in [false, true] {
        let mut d = waiters();
        if asked {
            free_one(&mut d);
        }
        let projected =
            outputs(&mut d, Event::Project { entry: FRESH, repository: REPO, view: projection::goal_view() });
        assert!(projected.contains(&Request::ProjectAfter {
            goal: 7,
            when: Some(Wall::from_nanos(env().wall.as_nanos() + env().limits.issue_policy.interval.as_nanos())),
        }));
        assert!(entries(&projected).is_empty());
        assert!(d.full_outbox_met());
        for request in projected {
            assert!(!matches!(
                request,
                Request::ProjectionEffect { .. } | Request::ProjectionFailed { .. } | Request::ProjectionSettled { .. }
            ));
        }
    }
}

#[test]
fn two_change_effects_wait_in_encounter_order_and_keep_one_fresh_entry() {
    let mut d = waiters();
    assert_eq!(asks(&free_one(&mut d)).as_ref(), &[FIRST]);
    assert_eq!(room::reserved(&d), 1);
    assert!(ready(&mut d).is_empty());
    let kept = open(&mut d, FIRST, FRESH);
    let entries = entries(&kept);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries.first().unwrap().number, FRESH);
    assert!(matches!(entries.first().unwrap().effect.write, Write::OpenPull { .. }));
    assert_eq!(d.change(FIRST).unwrap().room, None);
    assert_eq!(d.change(SECOND).unwrap().room, Some(2));
    assert_eq!(d.entries.len(), env().limits.entries);
    assert_eq!(room::reserved(&d), 0);
    assert!(!d.entries.contains_key(&21));
    assert!(!d.entries.contains_key(&22));
}

#[test]
fn change_room_places_restart_without_the_decision_that_waited() {
    let before = waiters();
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    for task in [SECOND, FIRST] {
        let saved = before.change(task).unwrap().clone();
        let (cold, hot) = saved.split();
        outputs(&mut d, Event::Restore { record: Stored::ChangeCold(Box::new(cold)) });
        outputs(&mut d, Event::Restore { record: Stored::ChangeHot(Box::new(hot)) });
    }
    let mut ended = row(53);
    ended.room = Some(3);
    ended.change.state = change::State::Ended;
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(ended)) });
    for (_, entry) in &before.entries {
        outputs(&mut d, Event::Restore { record: Stored::Entry { entry: entry.entry.clone(), held: true } });
    }
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    for (task, place) in [(FIRST, 1), (SECOND, 2)] {
        let saved = d.change(task).unwrap();
        assert_eq!(saved.room, Some(place));
        assert_eq!(saved.change.state, change::State::Opening { requested: false });
        assert_eq!(saved.pending, None);
        assert_eq!(saved.effect, change::EffectResult::None);
    }
    assert_eq!(d.change(53).unwrap().room, None);
    assert_eq!(room::reserved(&d), 0);
    let freed = outputs(&mut d, Event::Resolve { entry: 1, again: false });
    assert_eq!(asks(&freed).as_ref(), &[FIRST]);
    let kept = open(&mut d, FIRST, FRESH);
    assert_eq!(entries(&kept).len(), 1);
    assert_eq!(entries(&kept).first().unwrap().number, FRESH);
    assert_eq!(d.change(FIRST).unwrap().room, None);
    assert_eq!(d.change(SECOND).unwrap().room, Some(2));
}

#[test]
fn cancelling_a_waiting_or_asked_change_gives_the_next_waiter_room() {
    for asked in [false, true] {
        let mut d = waiters();
        if asked {
            assert_eq!(asks(&free_one(&mut d)).as_ref(), &[FIRST]);
        }
        let cancelled = outputs(&mut d, Event::ChangeEnding { task: FIRST, ending: Ending::Cancelled });
        assert_eq!(d.change(FIRST).unwrap().room, None);
        if asked {
            assert_eq!(asks(&cancelled).as_ref(), &[SECOND]);
        } else {
            assert_eq!(asks(&free_one(&mut d)).as_ref(), &[SECOND]);
        }
        assert_eq!(room::reserved(&d), 1);
    }
}

#[test]
fn a_change_waiting_for_room_has_no_poll_and_never_stalls() {
    let mut d = waiters();
    let before = d.next_wall_deadline();
    d.wait_change(&env(), FIRST, None);
    assert_eq!(d.next_wall_deadline(), before);
    let mut clock = env();
    clock.wall = Wall::from_nanos(clock.wall.as_nanos() + clock.limits.change_policy.stall.as_nanos() + 1);
    let mut expired = Queue::uncapped();
    fire(&mut d, &clock, &mut expired);
    for request in &expired {
        assert!(
            !matches!(request, Request::ChangeDecision { decision: change::Decision::Hold(change::Hold::Stalled), .. }),
            "{expired:?}"
        );
    }
    assert_eq!(d.change(FIRST).unwrap().room, Some(1));
    let mut d = full();
    let mut first = row(FIRST);
    first.pull = Some(1);
    first.change.state = change::State::First { head: fixtures::HEAD, ready_since: env().wall };
    first.change.owns_turn = true;
    outputs(&mut d, Event::Change { row: first });
    assert_wait(&pull_step(&mut d, FIRST, 21, "merge", true), FIRST);
    assert_eq!(asks(&free_one(&mut d)).as_ref(), &[FIRST]);
    let out = pull_step_at(&mut d, FIRST, FRESH, "unknown", true, &clock);
    for request in &out {
        assert!(
            !matches!(request, Request::ChangeDecision { decision: change::Decision::Hold(change::Hold::Stalled), .. }),
            "{out:?}"
        );
    }
    assert_eq!(d.change(FIRST).unwrap().room, None);
}

#[test]
fn a_held_waiter_is_passed_over_and_keeps_its_place() {
    let mut d = waiters();
    assert_eq!(asks(&free_one(&mut d)).as_ref(), &[FIRST]);
    let passed = outputs(&mut d, Event::RoomUnavailable { task: FIRST });
    assert_eq!(asks(&passed).as_ref(), &[SECOND]);
    assert_eq!(d.change(FIRST).unwrap().room, Some(1));
    assert_eq!(d.change(SECOND).unwrap().room, Some(2));
    assert_eq!(room::reserved(&d), 1);
}

#[test]
fn a_full_outbox_does_not_hide_an_oversize_or_duplicate_change_effect() {
    for oversized in [false, true] {
        let mut d = full();
        outputs(&mut d, Event::Change { row: row(FIRST) });
        start(&mut d, FIRST, if oversized { FRESH } else { 1 });
        fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
        fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::BASE)));
        fixtures::answer_next(&mut d, Err(Error::Missing));
        fixtures::answer_next(&mut d, Ok(fixtures::checks(Check::Passed)));
        fixtures::answer_next(&mut d, Ok(fixtures::checks(Check::Passed)));
        let (call, _) = fixtures::next_call(&mut d);
        let mut limits = env();
        if oversized {
            limits.limits.client.op_bytes = 1;
        }
        let out =
            fixtures::answer_at(&mut d, &limits, call, Ok(Answer::Commits { commits: Box::new([]), more: false }));
        let mut refused = false;
        for request in &out {
            refused |= if oversized {
                matches!(
                    request,
                    Request::ChangeDecision { decision: change::Decision::Hold(change::Hold::Failed), .. }
                )
            } else {
                matches!(request, Request::Refused { task: FIRST })
            };
        }
        assert!(refused, "{out:?}");
        assert_eq!(d.change(FIRST).unwrap().room, None);
        assert_eq!(d.entries.len(), env().limits.entries);
        assert_eq!(room::reserved(&d), 0);
    }
}

fn pull_step(d: &mut Domain, task: u64, entry: u64, kind: &str, green: bool) -> Box<[Request]> {
    pull_step_at(d, task, entry, kind, green, &env())
}

fn pull_step_at(d: &mut Domain, task: u64, entry: u64, kind: &str, green: bool, clock: &Env<Limits>) -> Box<[Request]> {
    use skein_forge::api::{Op, Read};
    start(d, task, entry);
    fixtures::answer_next(d, Ok(Answer::Commit(fixtures::HEAD)));
    fixtures::answer_next(d, Ok(Answer::Commit(fixtures::BASE)));
    let mut pull = fixtures::open_pull();
    pull.head = d.change(task).unwrap().branch.clone();
    if kind == "reopen" {
        pull.state = skein_forge::api::State::Closed;
    }
    fixtures::answer_next(d, Ok(Answer::Pull(pull)));
    for _ in 0_u32..6 {
        let (call, op) = fixtures::next_call(d);
        #[expect(
            clippy::wildcard_enum_match_arm,
            reason = "the scripted gather rejects every unrequested read or write"
        )]
        let answer = match op {
            Op::Read(Read::Checks { commit }) => {
                fixtures::checks(if green || commit == fixtures::BASE { Check::Passed } else { Check::Pending })
            }
            Op::Read(Read::Commits { .. }) => {
                let commits: Box<[skein_forge::api::CommitMessage]> = if kind == "update" {
                    Box::new([skein_forge::api::CommitMessage {
                        id: fixtures::BASE,
                        parents: Box::new([]),
                        message: Box::new([]),
                    }])
                } else {
                    Box::new([])
                };
                Answer::Commits { commits, more: kind == "unknown" }
            }
            Op::Read(Read::Reviews { .. }) => Answer::Reviews { reviews: Box::new([]), more: false, cut: false },
            unexpected => panic!("change's CI, containment or reviews: {unexpected:?}"),
        };
        let mut out = fixtures::answer_at(d, clock, call, Ok(answer));
        d.reclaim();
        let mut saved = List::uncapped();
        while let Some(request) = out.pop() {
            saved.push(request).unwrap();
        }
        let out = saved.into_boxed();
        for request in &out {
            if let Request::ChangeDecision { .. } = request {
                return out;
            }
        }
    }
    panic!("change decided")
}

#[test]
fn merge_update_and_reopen_effects_wait_and_are_admitted_once() {
    for kind in ["merge", "update", "reopen"] {
        let mut d = full();
        let mut row = row(FIRST);
        row.pull = Some(1);
        row.change.state = if kind == "reopen" {
            change::State::Reopening { requested: false, retarget: false }
        } else {
            change::State::First { head: fixtures::HEAD, ready_since: env().wall }
        };
        row.change.owns_turn = kind != "reopen";
        outputs(&mut d, Event::Change { row });
        let prior = d.change(FIRST).unwrap().clone();
        assert_wait(&pull_step(&mut d, FIRST, 21, kind, true), FIRST);
        assert_eq!(d.change(FIRST).unwrap().change, prior.change, "{kind}: undecided policy");
        assert_eq!(d.change(FIRST).unwrap().history, prior.history, "{kind}: no milestone for a write not kept");
        assert_eq!(asks(&free_one(&mut d)).as_ref(), &[FIRST]);
        let kept = pull_step(&mut d, FIRST, FRESH, kind, true);
        let entries = entries(&kept);
        assert_eq!(entries.len(), 1, "{kind}");
        assert_eq!(entries.first().unwrap().number, FRESH);
        assert!(
            match kind {
                "merge" => matches!(entries.first().unwrap().effect.write, Write::Merge { .. }),
                "update" => matches!(entries.first().unwrap().effect.write, Write::Update { .. }),
                "reopen" => matches!(entries.first().unwrap().effect.write, Write::Reopen { .. }),
                _ => unreachable!(),
            },
            "{kind}: {entries:?}"
        );
        assert_eq!(d.change(FIRST).unwrap().room, None);
    }
}

#[test]
fn an_asked_change_deciding_no_write_returns_its_reservation() {
    let mut d = waiters();
    assert_eq!(asks(&free_one(&mut d)).as_ref(), &[FIRST]);
    let decided = pull_step(&mut d, FIRST, FRESH, "merge", false);
    assert_eq!(d.change(FIRST).unwrap().room, None);
    assert_eq!(asks(&decided).as_ref(), &[SECOND]);
    assert_eq!(entries(&decided).len(), 0);
    assert_eq!(room::reserved(&d), 1);
}

fn making(d: &mut Domain) {
    outputs(d, Event::Hold { entry: 30, task: 7, resource: goal_branch::name(b"temper/7/g7"), from: None });
    fixtures::answer_next(d, Err(Error::Missing));
    let waited = fixtures::answer_next(d, Ok(Answer::Commit(fixtures::BASE)));
    assert_eq!(entries(&waited).len(), 0);
}

#[test]
fn changes_and_goal_branch_makings_share_one_line_by_place() {
    for making_first in [false, true] {
        let mut d = full();
        outputs(&mut d, Event::Change { row: row(FIRST) });
        if making_first {
            making(&mut d);
        }
        assert_wait(&open(&mut d, FIRST, 22), FIRST);
        if !making_first {
            making(&mut d);
        }
        let freed = free_one(&mut d);
        if making_first {
            assert!(freed.contains(&Request::ContinueGoalBranch { task: 7 }));
            assert_eq!(asks(&freed).as_ref(), &[]);
        } else {
            assert_eq!(asks(&freed).as_ref(), &[FIRST]);
            assert!(!freed.contains(&Request::ContinueGoalBranch { task: 7 }));
        }
        assert_eq!(room::reserved(&d), 1);
        let next = outputs(&mut d, Event::Withdraw { entry: 2 });
        if making_first {
            assert_eq!(asks(&next).as_ref(), &[FIRST]);
        } else {
            assert!(next.contains(&Request::ContinueGoalBranch { task: 7 }));
        }
    }
}

#[test]
fn an_unreserved_enqueue_cannot_take_an_asked_changes_room() {
    let mut d = full();
    for task in [FIRST, SECOND] {
        outputs(&mut d, Event::Change { row: row(task) });
    }
    assert_wait(&open(&mut d, FIRST, 21), FIRST);
    assert_eq!(asks(&free_one(&mut d)).as_ref(), &[FIRST]);
    start(&mut d, FIRST, FRESH);
    let (reading, _) = fixtures::next_call(&mut d);
    assert_wait(&open(&mut d, SECOND, 22), SECOND);
    making(&mut d);
    let refused = outputs(&mut d, Event::Enqueue { entry: refusals::entry(102) });
    assert_eq!(
        refused.as_ref(),
        &[Request::Outcome {
            entry: 102,
            task: 7,
            outcome: skein_forge_client::Outcome::Failed(Error::Busy),
            account: None,
        }]
    );
    fixtures::answer(&mut d, reading, Ok(Answer::Commit(fixtures::HEAD)));
    assert_eq!(entries(&finish(&mut d)).first().unwrap().number, FRESH);
    assert_eq!(d.change(FIRST).unwrap().room, None);
    assert_eq!(d.change(SECOND).unwrap().room, Some(2));
    assert_eq!(room::reserved(&d), 0);
    let freed = outputs(&mut d, Event::Withdraw { entry: FRESH });
    assert_eq!(asks(&freed).as_ref(), &[SECOND]);
    assert!(!freed.contains(&Request::ContinueGoalBranch { task: 7 }));
}
