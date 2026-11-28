use super::*;
use crate::landing::members::{hand, head, take};
use skein_lib::List;
use temper_engine_domain_forge_change as change;

const OLDER: u64 = 61;
const NEWER: u64 = 62;
const OTHER: u64 = fixtures::CHANGE + 1;
const ROUND: u16 = 3;
const MOVED: [u8; 32] =
    [5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

fn landing(task: u64) -> LandingRow {
    let mut row = merge_landing_row(false);
    row.landing.task = task;
    row.landing.preparation = ROUND;
    row
}

#[test]
fn restored_members_follow_the_oldest_live_row_until_they_leave_or_it_ends() {
    let mut d = domain();
    let mut older = landing(OLDER);
    let mut other = older.landing.members[0].clone();
    other.change = OTHER;
    older.landing.members = Box::new([older.landing.members[0].clone(), other]);
    let mut newer = landing(NEWER);
    newer.landing.members[0].landed = Some(fixtures::HEAD);
    for row in [&newer, &older] {
        outputs(&mut d, Event::Restore { record: Stored::Landing(row.clone()) });
    }
    d.fit_restored(&LIMITS).unwrap();
    assert_eq!(d.landing_of(fixtures::CHANGE), Some(OLDER), "rows restored newest first");
    assert_eq!(d.landing_of(OTHER), Some(OLDER));

    let mut out = Queue::uncapped();
    older.landing.members[0].left = Some(policy::Leave::Conflict);
    landing::save(&mut d, &env(), older, &mut out);
    assert_eq!(d.landing_of(fixtures::CHANGE), Some(NEWER), "a landed member is still in a live row");
    assert_eq!(d.landing_of(OTHER), Some(OLDER), "the other member stays");

    newer.landing.phase = policy::Phase::Landed;
    landing::save(&mut d, &env(), newer, &mut out);
    assert_eq!(d.landing_of(fixtures::CHANGE), None);
    assert_eq!(d.landing_of(OTHER), Some(OLDER));
}

const THIRD: u64 = OTHER + 1;

fn ready(d: &mut Domain, task: u64, base: &[u8]) {
    let mut row = d.change(fixtures::CHANGE).unwrap().clone();
    row.task = task;
    row.change.task = task;
    row.base = base.into();
    row.pull = Some(task);
    row.change.way = change::Way::Turn;
    row.change.state = change::State::Ready { head: fixtures::HEAD, ready_since: env().wall };
    row.change.ready_since = Some(env().wall);
    outputs(d, Event::Change { row });
}

fn form(d: &mut Domain, task: u64, base: &[u8], changes: &[(u64, u16)]) -> Box<[Request]> {
    let mut row = landing_row(task, policy::Phase::Waiting);
    row.landing.branch = base.into();
    let mut members = List::with_capacity(LIMITS.landing_members);
    for (change, unit) in changes {
        let mut member = d.forming_member(*change, LIMITS.gates).unwrap();
        member.unit = *unit;
        members.push(member).unwrap();
    }
    row.landing.members = members.into_boxed();
    outputs(d, Event::Landing { row })
}

fn cancel(d: &mut Domain, task: u64) -> Box<[Request]> {
    outputs(d, Event::SettleEffects { task, root: task, ending: ReleaseEnding::Cancelled })
}

#[test]
fn members_admit_at_the_bound_but_refuse_one_more() {
    let mut d = landing_fixture();
    assert_eq!(LIMITS.landing_members, 2);
    for task in [fixtures::CHANGE, OTHER, THIRD] {
        ready(&mut d, task, b"main");
    }
    assert_eq!(d.landing_admits(5, &[fixtures::CHANGE, OTHER, THIRD]), Err(LandingRefusal::Members));
    assert!(d.landing.rows.is_empty(), "refusing the selection makes no row");
    for task in [fixtures::CHANGE, OTHER, THIRD] {
        assert_eq!(d.landing_of(task), None);
    }

    assert_eq!(d.landing_admits(5, &[fixtures::CHANGE, OTHER]), Ok(()));
    let requests = form(&mut d, OLDER, b"main", &[(fixtures::CHANGE, 0), (OTHER, 1)]);
    assert_eq!(d.landing_of(fixtures::CHANGE), Some(OLDER));
    assert_eq!(d.landing_of(OTHER), Some(OLDER));
    assert_eq!(d.landing_of(THIRD), None);
    assert!(requests.contains(&Request::LandingNews { task: OLDER, member: 1, step: LandingNews::Joined }));
    assert_eq!(d.landing_admits(5, &[OTHER]), Err(LandingRefusal::Repository), "an owned change is refused as a value");
}

#[test]
fn member_cancellation_removes_its_unit_once_and_the_ready_mate_forms_again() {
    let mut d = landing_fixture();
    for task in [fixtures::CHANGE, OTHER] {
        ready(&mut d, task, b"main");
    }
    form(&mut d, OLDER, b"main", &[(fixtures::CHANGE, 0), (OTHER, 0)]);
    let requests = cancel(&mut d, fixtures::CHANGE);
    let left = d.landing(OLDER).unwrap().clone();
    assert_eq!(left.landing.members[0].left, Some(policy::Leave::Cancelled));
    assert_eq!(left.landing.members[1].left, Some(policy::Leave::Unit { change: fixtures::CHANGE }));
    assert_eq!(d.landing_of(fixtures::CHANGE), None);
    assert_eq!(d.landing_of(OTHER), None);
    let mut saved = false;
    for request in &requests {
        if let Request::Save { record: Stored::Landing(row), .. } = request {
            saved |= row.landing == left.landing;
        }
    }
    assert!(saved);
    let repeated = cancel(&mut d, fixtures::CHANGE);
    assert_eq!(d.landing(OLDER), Some(&left));
    assert!(!repeated.contains(&Request::WakeProcedure { task: OLDER }));
    for request in &repeated {
        assert!(!matches!(request, Request::Save { record: Stored::Landing(_), .. }));
    }

    cancel(&mut d, OLDER);
    assert_eq!(d.landing_admits(5, &[OTHER]), Ok(()));
    form(&mut d, NEWER, b"main", &[(OTHER, 0)]);
    assert_eq!(d.landing_of(OTHER), Some(NEWER));
    assert_eq!(d.landing_of(fixtures::CHANGE), None);
}

#[test]
fn cancelling_a_change_and_ending_a_landing_touch_only_their_own_memberships() {
    let mut d = landing_fixture();
    ready(&mut d, fixtures::CHANGE, b"main");
    for task in [OTHER, THIRD] {
        ready(&mut d, task, b"next");
    }
    form(&mut d, OLDER, b"main", &[(fixtures::CHANGE, 0)]);
    form(&mut d, NEWER, b"next", &[(OTHER, 0), (THIRD, 1)]);
    let before = d.landing(OLDER).unwrap().clone();
    for task in [OTHER, u64::MAX] {
        let requests = cancel(&mut d, task);
        assert_eq!(d.landing(OLDER), Some(&before));
        assert!(!requests.contains(&Request::WakeProcedure { task: OLDER }));
    }
    assert_eq!(d.landing_of(OTHER), None);
    assert_eq!(d.landing_of(THIRD), Some(NEWER));

    cancel(&mut d, OLDER);
    assert_eq!(d.landing_of(fixtures::CHANGE), None);
    assert_eq!(d.landing_of(THIRD), Some(NEWER));
    let next = NEWER + 1;
    assert_eq!(d.landing_admits(5, &[fixtures::CHANGE]), Ok(()));
    form(&mut d, next, b"main", &[(fixtures::CHANGE, 0)]);
    assert_eq!(d.landing_of(fixtures::CHANGE), Some(next));
}

#[test]
fn cancelling_a_landed_member_does_not_release_its_index_even_in_a_single_landing() {
    for single in [true, false] {
        let mut d = landing_fixture();
        let mut row = merge_landing_row(single);
        row.landing.members[0].landed = Some(fixtures::HEAD);
        outputs(&mut d, Event::Restore { record: Stored::Landing(row.clone()) });
        d.fit_restored(&LIMITS).unwrap();
        let requests = cancel(&mut d, fixtures::CHANGE);
        assert_eq!(d.landing(OLDER), Some(&row));
        assert_eq!(d.landing_of(fixtures::CHANGE), Some(OLDER));
        assert!(!requests.contains(&Request::WakeProcedure { task: OLDER }));

        cancel(&mut d, OLDER);
        assert_eq!(d.landing_of(fixtures::CHANGE), None);
        assert_eq!(d.landing(OLDER).unwrap().landing.members[0].landed, Some(fixtures::HEAD));
    }
}

fn judge() -> change::Ask {
    change::Ask::Judge(change::Judgement {
        landing: OLDER,
        round: ROUND,
        pull: 2,
        branch: b"prefix".as_slice().into(),
        base: b"main".as_slice().into(),
        prefix: fixtures::HEAD,
    })
}

fn repair(candidate: [u8; 32]) -> change::Ask {
    change::Ask::Repair { gate: 0, candidate, below: None }
}

#[test]
fn a_new_ask_of_a_kind_replaces_the_old_and_a_take_names_only_its_number() {
    let mut d = landing_fixture();
    let row = landing(OLDER);
    let mut out = Queue::uncapped();
    let judge = judge();
    hand(&mut d, &row, 0, judge.clone(), &mut out);
    hand(&mut d, &row, 0, repair(fixtures::BASE), &mut out);
    let candidate = fixtures::HEAD;
    hand(&mut d, &row, 0, repair(candidate), &mut out);
    let row = d.changes.get_mut(&fixtures::CHANGE).unwrap();
    let first = change::Handed { number: 1, landing: OLDER, round: ROUND, ask: judge };
    let third = change::Handed { number: 3, landing: OLDER, round: ROUND, ask: repair(candidate) };
    assert_eq!(row.handed, 3);
    assert_eq!(row.asks.as_ref(), &[first.clone(), third.clone()]);
    take(row, change::Took::Taken(2), &mut out);
    assert_eq!(row.asks.as_ref(), &[first, third.clone()], "a decided take cannot remove its replacement");
    take(row, change::Took::Dropped(1), &mut out);
    assert_eq!(row.asks.as_ref(), &[third]);
    assert_eq!(row.handed, 3);
}

fn departed_with_ask(mut landing: LandingRow, ask: change::Ask, why: policy::Leave, head: [u8; 32]) -> Domain {
    let mut d = landing_fixture();
    fixtures::connect(&mut d);
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::CatchUp);
    fixtures::settle_home_reads(&mut d, &env(), fixtures::BASE);
    let row = d.changes.get_mut(&fixtures::CHANGE).unwrap();
    row.change.way = change::Way::Turn;
    row.change.state = change::State::Ready { head: fixtures::HEAD, ready_since: env().wall };
    row.change.ready_since = Some(env().wall);
    row.pending = None;
    row.effect = change::EffectResult::None;

    assert!(d.landing.remember(landing.clone()).is_none());
    let mut out = Queue::uncapped();
    hand(&mut d, &landing, 0, ask, &mut out);
    landing.landing.members[0].left = Some(why);
    landing::save(&mut d, &env(), landing, &mut out);
    if head != fixtures::HEAD {
        admit_member_push(&mut d, head);
    }
    d
}

fn assert_dropped(requests: Box<[Request]>, words: &[u8]) {
    assert!(
        requests.contains(&Request::AskDropped { change: fixtures::CHANGE, words: words.into() }),
        "decision reports the dropped ask with the specified words",
    );
    assert!(saved_hot(requests).asks.is_empty(), "the drop decision saves the empty queue");
}

#[test]
fn a_single_landing_members_stale_prepare_no_longer_blocks_forming() {
    let mut d = departed_with_ask(
        merge_landing_row(true),
        change::Ask::Prepare { head: fixtures::HEAD, main: fixtures::BASE },
        policy::Leave::HeadMoved { head: MOVED },
        MOVED,
    );
    let requests = step_departed_change(&mut d, MOVED, false);
    assert_dropped(
        requests,
        b"change 51: prepare for 0303030303030303030303030303030303030303030303030303030303030303 \
          dropped: the head is now 0505050505050505050505050505050505050505",
    );
    let row = d.change(fixtures::CHANGE).unwrap();
    assert_eq!(row.change.state, change::State::Ready { head: MOVED, ready_since: env().wall });
    let waiting = d.waiting_changes(REPO, b"main");
    assert_eq!(waiting.first().expect("eligible to form").task, fixtures::CHANGE);
}

#[test]
fn a_departed_turn_member_drops_its_judgement_and_stays_ready() {
    let mut d = departed_with_ask(landing(OLDER), judge(), policy::Leave::EarlierConflict, fixtures::HEAD);
    let requests = step_departed_change(&mut d, fixtures::HEAD, false);
    assert_dropped(
        requests,
        b"change 51: judgement of landing 61, round 3, dropped: the change is in that landing no longer",
    );
    let row = d.change(fixtures::CHANGE).unwrap();
    assert_eq!(row.change.state, change::State::Ready { head: fixtures::HEAD, ready_since: env().wall });
    assert!(row.asks.is_empty());
}

fn restart(cold: &ChangeCold, hot: ChangeHot) -> Domain {
    let mut d = domain();
    for row in
        [Stored::Repository(adopted()), Stored::ChangeHot(Box::new(hot)), Stored::ChangeCold(Box::new(cold.clone()))]
    {
        outputs(&mut d, Event::Restore { record: row });
    }
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::CatchUp);
    d
}

fn saved_hot(out: impl IntoIterator<Item = Request>) -> ChangeHot {
    let mut hot = None;
    for request in out {
        if let Request::Save { record: Stored::ChangeHot(row), .. } = request {
            hot = Some(*row);
        }
    }
    hot.expect("decision saved the change")
}

#[test]
fn the_landed_ask_takes_priority_and_is_consumed_once_across_restarts() {
    let mut d = landing_fixture();
    let member = d.changes.get_mut(&fixtures::CHANGE).unwrap();
    member.change.way = change::Way::Turn;
    member.change.state = change::State::Ready { head: fixtures::HEAD, ready_since: env().wall };
    let (cold, _) = member.clone().split();
    let row = landing(OLDER);
    let mut out = Queue::uncapped();
    let prefix = fixtures::BASE;
    hand(&mut d, &row, 0, repair(fixtures::HEAD), &mut out);
    hand(&mut d, &row, 0, change::Ask::Landed { prefix }, &mut out);

    let mut restored = restart(&cold, saved_hot(collected(out)));
    let row = restored.change(fixtures::CHANGE).unwrap();
    assert_eq!(head(row).unwrap().number, 2);
    let decided = outputs(&mut restored, fixtures::change_step());
    let after = restart(&cold, saved_hot(decided));
    let row = after.change(fixtures::CHANGE).unwrap();
    assert!(matches!(row.change.state, change::State::Landed { merge, .. } if merge == prefix));
    assert_eq!(row.asks.len(), 1);
    assert_eq!(head(row).unwrap().number, 1);
    assert_eq!(row.asks[0].ask, repair(fixtures::HEAD));
    assert_eq!(row.handed, 2);
}

#[test]
fn cancelling_a_landing_removes_only_its_prepare_and_judgement() {
    let mut d = landing_fixture();
    let row = landing(OLDER);
    assert!(d.landing.remember(row.clone()).is_none());
    let other = landing(NEWER);
    let mut out = Queue::uncapped();
    let ask = repair(fixtures::HEAD);
    hand(&mut d, &other, 0, ask.clone(), &mut out);
    hand(&mut d, &row, 0, change::Ask::Prepare { head: fixtures::HEAD, main: fixtures::BASE }, &mut out);
    hand(&mut d, &row, 0, judge(), &mut out);
    outputs(&mut d, Event::SettleEffects { task: OLDER, root: OLDER, ending: ReleaseEnding::Cancelled });
    assert_eq!(d.landing_of(fixtures::CHANGE), None);
    let row = d.change(fixtures::CHANGE).unwrap();
    assert_eq!(row.asks.as_ref(), &[change::Handed { number: 1, landing: NEWER, round: ROUND, ask }]);
    assert_eq!(row.handed, 3);
}
