use super::*;
use crate::landing::members::{hand, head, take};
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
    let row = d.changes.get_mut(&fixtures::CHANGE).unwrap();
    row.change.way = change::Way::Turn;
    row.change.state = change::State::Ready { head: fixtures::HEAD, ready_since: env().wall };
    row.change.last_head = Some(head);
    row.change.ready_since = Some(env().wall);
    row.pending = None;
    row.effect = change::EffectResult::None;

    d.landing.remember(landing.clone()).unwrap();
    let mut out = Queue::uncapped();
    hand(&mut d, &landing, 0, ask, &mut out);
    landing.landing.members[0].left = Some(why);
    landing::save(&mut d, &env(), landing, &mut out);
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
    d.landing.remember(row.clone()).unwrap();
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
