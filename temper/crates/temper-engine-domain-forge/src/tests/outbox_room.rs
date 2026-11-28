//! A change gives up its decision and number, not its place (forge/room.md, 3–4).
use super::*;
use jig_domain_contract as contract;
use skein_forge::api::{Answer, Check, Error, Op, Read, Write};
use skein_lib::List;
use temper_engine_domain_forge_change as change;

const FIRST: u64 = 52;
const SECOND: u64 = 51;
const FRESH: u64 = 101;

pub(super) fn full() -> Domain {
    let mut d = refusals::adopted_domain();
    for number in 1..=u64::from(env().limits.entries) {
        outputs(&mut d, Event::Enqueue { entry: refusals::entry(number) });
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
    outputs(d, opening_step(task, entry));
}

fn opening_step(task: u64, entry: u64) -> Event {
    let Event::StepChange { merge_job, heard, gates, queue_repair_active, pending, kept, .. } = fixtures::change_step()
    else {
        panic!("step fixture")
    };
    Event::StepChange { task, entry, merge_job, heard, gates, queue_repair_active, pending, kept }
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

pub(super) fn later(clock: &Env<Limits>, duration: Duration) -> Env<Limits> {
    let mut clock = clock.with(clock.limits);
    clock.now = clock.now.saturating_add(duration);
    clock.wall = Wall::from_nanos(clock.wall.as_nanos().saturating_add(duration.as_nanos()));
    clock
}

fn wait_in(d: &mut Domain, env: &Env<Limits>, task: u64, place: u64) {
    let mut row = row(task);
    row.room = Some(place);
    step(d, env, Event::Change { row }, &mut Queue::uncapped());
}

fn room_wait(d: &Domain, env: &Env<Limits>, task: u64) -> jig_domain_core_vocabulary::waits::Reported {
    room::waited(d, &env.limits, task, temper_client_protocol_wire::waits::CHANGE_OPENING).expect("blocked room wait")
}

#[test]
fn room_reports_follow_live_holders_then_the_latest_end_or_join() {
    use jig_domain_core_vocabulary::waits::{AtBound, ReportedBound, Unbounded};
    use temper_client_protocol_wire::waits::CHANGE_OPENING;
    const OLD: u64 = 71;
    const LIVE: u64 = 72;
    for reservation in [false, true] {
        let mut d = refusals::adopted_domain();
        let start = env();
        let limit = u64::from(start.limits.entries);
        let last = if reservation { limit - 1 } else { limit };
        for number in 1..=last {
            let mut entry = refusals::entry(number);
            entry.task = if !reservation && number == last { LIVE } else { OLD };
            outputs(&mut d, Event::Enqueue { entry });
        }
        if reservation {
            assert!(d.reserve_call(&start, Token::new(1), LIVE));
        }
        wait_in(&mut d, &start, FIRST, 1);
        assert_eq!(room_wait(&d, &start, FIRST).bound, ReportedBound::None(Unbounded::Behind(OLD)));
        let moved = outputs(&mut d, Event::ChangeEnding { task: OLD, ending: Ending::Cancelled });
        let [replaced] = wait_report(&moved, FIRST).expect("owner ending changes the report") else {
            panic!("room record")
        };
        assert_eq!(replaced.bound, ReportedBound::None(Unbounded::Behind(LIVE)));
        let old = replaced.object.clone();
        let ended = later(&start, Duration::from_secs(7));
        let mut out = Queue::uncapped();
        step(&mut d, &ended, Event::ChangeEnding { task: LIVE, ending: Ending::Cancelled }, &mut out);
        let moved = collected(out);
        let [replaced] = wait_report(&moved, FIRST).expect("last live holder's end replaces the report") else {
            panic!("room record")
        };
        assert_eq!(replaced, &room_wait(&d, &ended, FIRST));
        assert_eq!(replaced.object, old);
        assert_eq!(
            replaced.bound,
            ReportedBound::At { at: later(&ended, ended.limits.waits.room).wall, then: AtBound::Hold }
        );
        let joined = later(&ended, Duration::from_secs(3));
        wait_in(&mut d, &joined, SECOND, 2);
        let late_wait = room_wait(&d, &joined, SECOND);
        assert_eq!(late_wait.since, joined.wall);
        assert_eq!(
            late_wait.bound,
            ReportedBound::At { at: later(&joined, joined.limits.waits.room).wall, then: AtBound::Hold }
        );
        let mut settled = Queue::uncapped();
        step(&mut d, &joined, Event::Withdraw { entry: 1 }, &mut settled);
        let settled = collected(settled);
        assert_eq!(asks(&settled).as_ref(), &[FIRST]);
        let bound =
            ReportedBound::At { at: advanced_wall(start.wall, start.limits.change_policy.stall), then: AtBound::Hold };
        assert_eq!(wait_report(&settled, FIRST), Some([step_wait(CHANGE_OPENING, start.wall, bound, &[])].as_slice()));
        assert_eq!(wait_report(&settled, SECOND), Some([room_wait(&d, &joined, SECOND)].as_slice()));
        assert_eq!(room_wait(&d, &joined, SECOND).bound, ReportedBound::None(Unbounded::Behind(FIRST)));
    }
}

#[test]
fn restored_room_clocks_start_with_the_engine_not_the_catch_up() {
    use jig_domain_core_vocabulary::waits::{AtBound, ReportedBound};
    let joined = env();
    let catch_up = later(&joined, Duration::from_secs(7));
    let mut d = fixtures::connected_domain();
    d.restore_begin(joined.wall);
    let mut out = Queue::uncapped();
    step(&mut d, &joined, Event::Restore { record: Stored::Repository(adopted()) }, &mut out);
    let mut row = row(FIRST);
    row.room = Some(1);
    step(&mut d, &joined, Event::Restore { record: Stored::Change(Box::new(row)) }, &mut out);
    for number in 1..=u64::from(joined.limits.entries) {
        let entry = refusals::entry(number);
        step(&mut d, &joined, Event::Restore { record: Stored::Entry { entry, held: true } }, &mut out);
    }
    step(&mut d, &joined, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic }, &mut out);
    step(&mut d, &catch_up, Event::CatchUp, &mut out);
    assert_eq!(d.inspection_entry_owner_ended(1), Some(joined.wall));
    let waited = room_wait(&d, &catch_up, FIRST);
    assert_eq!(waited.since, joined.wall);
    assert_eq!(
        waited.bound,
        ReportedBound::At { at: later(&joined, joined.limits.waits.room).wall, then: AtBound::Hold }
    );
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
        let goal = projection.goal;
        if keep {
            let kept = outputs(&mut d, Event::KeepProjection { row: projection, entry });
            assert_eq!(entries(&kept).first().unwrap().number, FRESH);
            assert_eq!(d.entries.len(), env().limits.entries);
            assert_eq!(room::reserved(&d), 0);
            assert!(asks(&kept).is_empty());
            assert_eq!(asks(&outputs(&mut d, Event::Withdraw { entry: FRESH })).as_ref(), &[FIRST]);
        } else {
            let returned = outputs(&mut d, Event::DropProjection { entry: FRESH, goal });
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
                Request::ProjectionEffect { .. }
                    | Request::ProjectionFailed { .. }
                    | Request::Up(contract::Up::Projections(contract::projections::Up::Settled { .. }))
            ));
        }
    }
}

#[test]
fn two_change_effects_wait_in_encounter_order_and_keep_one_fresh_entry() {
    const NEWCOMER: u64 = 53;
    let mut d = waiters();
    assert_eq!(asks(&free_one(&mut d)).as_ref(), &[FIRST]);
    assert_eq!(room::reserved(&d), 1);
    assert!(ready(&mut d).is_empty());
    outputs(&mut d, Event::Change { row: row(NEWCOMER) });
    assert_wait(&open(&mut d, NEWCOMER, 23), NEWCOMER);
    assert_eq!(d.change(NEWCOMER).unwrap().room, Some(3));
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
    outputs(&mut d, Event::Restore { record: Stored::Names { task: 99, resources: Box::new([branch()]) } });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::CatchUp);
    assert!(d.room.ready, "the line is ready at CatchUp");
    let (fresh, _) = fixtures::next_call(&mut d);
    assert!(!d.fresh_reads_in(), "the watched branch still owes its fresh read");

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
    assert!(!d.fresh_reads_in(), "free room is asked before the fresh reads end");
    fixtures::answer(&mut d, fresh, Ok(Answer::Commit(fixtures::HEAD)));
    assert!(d.fresh_reads_in());
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
        assert!(!d.room_asked(FIRST));
        if asked {
            assert_eq!(asks(&cancelled).as_ref(), &[SECOND]);
        } else {
            assert_eq!(asks(&free_one(&mut d)).as_ref(), &[SECOND]);
        }
        assert_eq!(room::reserved(&d), 1);
        assert_eq!(d.inspection_reservations().as_ref(), &[SECOND]);
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
            !matches!(
                request,
                Request::ChangeDecision { decision: change::Decision::Hold(change::Cause::Stalled), .. }
            ),
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
            !matches!(
                request,
                Request::ChangeDecision { decision: change::Decision::Hold(change::Cause::Stalled), .. }
            ),
            "{out:?}"
        );
    }
    assert_eq!(d.change(FIRST).unwrap().room, None);
}

#[test]
fn a_held_waiter_is_passed_over_and_keeps_its_place() {
    const THIRD: u64 = 53;
    let mut d = waiters();
    outputs(&mut d, Event::Change { row: row(THIRD) });
    assert_wait(&open(&mut d, THIRD, 23), THIRD);
    assert_eq!(asks(&free_one(&mut d)).as_ref(), &[FIRST]);
    let passed = outputs(&mut d, Event::RoomUnavailable { task: FIRST });
    assert_eq!(asks(&passed).as_ref(), &[SECOND]);
    assert_eq!(d.change(FIRST).unwrap().room, Some(1));
    assert_eq!(d.change(SECOND).unwrap().room, Some(2));
    assert_eq!(room::reserved(&d), 1);
    // The release steps FIRST again, but SECOND's ask still holds the only room.
    assert_wait(&open(&mut d, FIRST, FRESH), FIRST);
    let offered = outputs(&mut d, Event::Withdraw { entry: 2 });
    assert_eq!(asks(&offered).as_ref(), &[FIRST]);
    assert_eq!(d.change(THIRD).unwrap().room, Some(3));
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
                    Request::ChangeDecision { decision: change::Decision::Hold(change::Cause::Failed), .. }
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

fn making(d: &mut Domain, task: u64) {
    let branch = format!("temper/{task}/g{task}");
    outputs(d, Event::Hold { entry: 30, task, resource: goal_branch::name(branch.as_bytes()), from: None });
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
            making(&mut d, 7);
        }
        assert_wait(&open(&mut d, FIRST, 22), FIRST);
        if !making_first {
            making(&mut d, 7);
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
fn closing_a_goal_returns_its_makings_ask_before_the_fresh_tip_arrives() {
    const GOAL: u64 = 70;
    let mut d = full();
    making(&mut d, GOAL);
    outputs(&mut d, Event::Change { row: row(FIRST) });
    assert_wait(&open(&mut d, FIRST, 22), FIRST);
    let asked = free_one(&mut d);
    assert!(asked.contains(&Request::ContinueGoalBranch { task: GOAL }));
    outputs(&mut d, Event::ContinueGoalBranch { task: GOAL, entry: FRESH });
    let (tip, _) = fixtures::next_call(&mut d);

    let ended = outputs(&mut d, Event::SettleEffects { task: GOAL, root: GOAL, ending: ReleaseEnding::Cancelled });
    assert_eq!(asks(&ended).as_ref(), &[FIRST], "closing returns the making's reservation before the read settles");
    let late = fixtures::answer(&mut d, tip, Ok(Answer::Commit(fixtures::BASE)));
    assert!(entries(&late).is_empty());
    assert!(asks(&late).is_empty(), "the late read neither keeps nor frees the reservation again");
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
    making(&mut d, 7);
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

const WRITER: u64 = 7;
const STATUS: u64 = 1;
const LAST_PLACE: u64 = 2;
const WAITING_EFFECT: u64 = 3;
// One retained status and one more write reach the outbox's bound.
const TWO_PLACES: u32 = 2;

fn two_places() -> (Domain, Env<Limits>) {
    let mut clock = env();
    clock.limits.entries = TWO_PLACES;
    let mut d = domain_with_limits(&clock.limits);
    d.restore_begin(clock.wall);
    for event in [
        Event::Connection {
            forge: REPO.forge,
            state: Connection::Connected { id: WRITER, login: b"bot".as_slice().into() },
        },
        Event::Restore { record: Stored::Repository(adopted()) },
        Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic },
        Event::CatchUp,
        Event::Enqueue { entry: status(STATUS, WRITER) },
    ] {
        step_at(&mut d, &clock, event);
    }
    (d, clock)
}

fn step_at(d: &mut Domain, clock: &Env<Limits>, event: Event) -> Box<[Request]> {
    let mut out = Queue::uncapped();
    step(d, clock, event, &mut out);
    collected(out)
}

fn open_at(d: &mut Domain, clock: &Env<Limits>, task: u64, entry: u64) -> Box<[Request]> {
    step_at(d, clock, opening_step(task, entry));
    fixtures::answer_next_at(d, clock, Ok(Answer::Commit(fixtures::HEAD)));
    fixtures::answer_next_at(d, clock, Ok(Answer::Commit(fixtures::BASE)));
    fixtures::answer_next_at(d, clock, Err(Error::Missing));
    fixtures::finish_missing_at(d, clock)
}

fn send_status(d: &mut Domain, clock: &Env<Limits>) -> (Token, Box<[Request]>) {
    step_at(d, clock, Event::Committed { entry: STATUS });
    let (call, op) = fixtures::next_at(d, clock);
    assert!(matches!(op, Op::Read(Read::Items { .. })), "write's clock: {op:?}");
    fixtures::answer_at(
        d,
        clock,
        call,
        Ok(Answer::Items { items: Box::new([]), more: false, cut: false, now: clock.now }),
    );
    let mut sending = Queue::uncapped();
    resume(d, clock, &mut sending);
    let sending = collected(sending);
    for request in &sending {
        if let Request::Call { call, op, .. } = request {
            assert_eq!(op, &Op::Write(status(STATUS, WRITER).effect.write));
            return (*call, sending);
        }
    }
    panic!("status write: {sending:?}");
}

#[test]
fn the_last_place_passes_to_the_waiting_change() {
    let (mut d, clock) = two_places();
    for task in [FIRST, SECOND] {
        step_at(&mut d, &clock, Event::Change { row: row(task) });
    }
    let kept = entries(&open_at(&mut d, &clock, FIRST, LAST_PLACE));
    let [entry] = kept.as_ref() else { panic!("one kept entry: {kept:?}") };
    assert_eq!((entry.number, entry.task), (LAST_PLACE, FIRST));
    assert_eq!(room::free(&d, &clock.limits), 0);

    let waiting = open_at(&mut d, &clock, SECOND, WAITING_EFFECT);
    assert_wait(&waiting, SECOND);
    assert_eq!(d.change(SECOND).unwrap().room, Some(1));
    assert!(asks(&waiting).is_empty());
    let (call, _) = send_status(&mut d, &clock);
    let freed = collected(fixtures::answer_at(&mut d, &clock, call, Ok(Answer::Done)));
    assert!(freed.contains(&Request::Erase { key: Key::Entry(STATUS) }));
    assert_eq!(asks(&freed).as_ref(), &[SECOND]);
    assert_eq!(room::reserved(&d), 1);

    let kept = entries(&open_at(&mut d, &clock, SECOND, FRESH));
    let [entry] = kept.as_ref() else { panic!("one kept entry: {kept:?}") };
    assert_eq!((entry.number, entry.task), (FRESH, SECOND));
    assert_eq!(d.change(SECOND).unwrap().room, None);
    assert_eq!(room::reserved(&d), 0);
    assert_eq!(room::free(&d, &clock.limits), 0);
}

#[test]
fn a_full_outbox_saves_progress_in_place() {
    let (mut d, clock) = two_places();
    step_at(&mut d, &clock, Event::Enqueue { entry: status(LAST_PLACE, WRITER) });
    wait_in(&mut d, &clock, FIRST, 1);
    let (_, progress) = send_status(&mut d, &clock);
    let saved = entries(&progress);
    let [entry] = saved.as_ref() else { panic!("one progress save: {progress:?}") };
    assert_eq!((entry.number, entry.task), (STATUS, WRITER));
    assert!(entry.attempt.is_some());
    assert_eq!(&d.entries.get(&STATUS).unwrap().entry, entry);
    assert_eq!(room::free(&d, &clock.limits), 0);
    assert_eq!(room::reserved(&d), 0);
    assert!(asks(&progress).is_empty());
    for request in &progress {
        assert!(!matches!(
            request,
            Request::Refused { .. } | Request::ResourceRefused { .. } | Request::Outcome { .. }
        ));
    }
}

#[test]
fn a_repeated_outbox_outcome_leaves_the_same_room_and_line() {
    use skein_forge_client::{Made, Outcome};
    let (mut d, clock) = two_places();
    step_at(&mut d, &clock, Event::Enqueue { entry: status(LAST_PLACE, WRITER) });
    wait_in(&mut d, &clock, FIRST, 1);
    wait_in(&mut d, &clock, SECOND, 2);
    let (call, _) = send_status(&mut d, &clock);
    let made = Outcome::Made { made: Made::Set, found: false };
    let terminal = Request::Outcome { entry: STATUS, task: WRITER, outcome: made, account: None };
    let settled = collected(fixtures::answer_at(&mut d, &clock, call, Ok(Answer::Done)));
    assert!(settled.contains(&terminal));
    assert_eq!(asks(&settled).as_ref(), &[FIRST]);
    let first = room::Waiter::Change(FIRST);
    let second = room::Waiter::Change(SECOND);
    assert_eq!(room::reserved(&d), 1);
    assert_eq!(room::free(&d, &clock.limits), 0);
    assert!(room::asked(&d, first));
    assert!(!room::asked(&d, second));

    let mut out = Queue::uncapped();
    domain::scripted_outcome(&mut d, &clock, STATUS, WRITER, made, &mut out);
    resume(&mut d, &clock, &mut out);
    let late = collected(out);
    for request in &late {
        assert!(!matches!(request, Request::Outcome { .. }), "no second terminal: {late:?}");
    }
    assert!(asks(&late).is_empty());
    assert_eq!(d.entries.len(), 1);
    assert_eq!(room::reserved(&d), 1);
    assert_eq!(room::free(&d, &clock.limits), 0);
    assert_eq!(room::place(&d, first), Some(1));
    assert_eq!(room::place(&d, second), Some(2));
    assert!(room::asked(&d, first));
    assert!(!room::asked(&d, second));
}
