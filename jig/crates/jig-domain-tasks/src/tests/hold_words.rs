//! Held records explain themselves (domain/tasks/lifecycle-holds.md, 3).

use super::revisions::{Harness, task, to};
use crate::{
    Active, Bound, Circle, Class, Domain, Event, Exceeded, Hold, HoldSource, Holding, Key, Name, Phase, PlacedHold,
    Status, Stored, Was, hold_words,
};
use skein_lib::{Token, Wall};

const HELD: Wall = Wall::from_nanos(1_795_500_435_000_000_000);
const DEADLINE: Wall = Wall::from_nanos(1_795_500_000_000_000_000);
const LATER: Wall = Wall::from_nanos(1_795_501_200_000_000_000);
const SINCE: &str = "Held since 2026-11-24T06:07Z.";
const WAIT: &str = "Waited 24 h for news since 2026-11-23T06:07Z: subscriptions 3 and 5.";
const RESET: &str = "Release it for its executor to decide afresh.";
const DROP: &str = "Release it once access is back, or amend it to drop the resource.";

fn resource() -> Name {
    Name { connector: 7, path: Box::new([Box::from(b"build/artifact".as_slice())]) }
}

fn placed(number: u64, reason: Box<[u8]>) -> PlacedHold {
    PlacedHold {
        number,
        project: 1,
        owner: crate::Owner::Party { person: 5 },
        mode: crate::Mode::Stop,
        scope: crate::Scope::Subtree { task: 1 },
        reason,
        placed_at: 0,
        revision: 1,
        exempt: Box::new([]),
        lifted: false,
    }
}

fn connector_account(entry: Option<u64>, words: &[u8]) -> HoldSource {
    HoldSource::Account { connector: 7, entry, words: words.into() }
}

fn add_placed(domain: &mut Domain, number: u64, reason: &[u8]) {
    // Older placed holds may have no reason. This is their retained row, not a new place request.
    domain.placed.insert(number, placed(number, reason.into())).expect("test hold room");
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table pins each independent cause and way out")]
fn every_cause_names_its_object_since_and_way_out() {
    let mut h = Harness::new();
    let name = resource();
    h.send(Event::Resource { name: name.clone(), hold: crate::HoldKind::Exclusive { taken: crate::Taken::Waits } });
    let holding = Holding::Write { resource: name.clone(), kind: 1 };
    let mut blocker = task(20, &[]);
    blocker.holdings = Box::new([holding.clone()]);
    h.make([task(1, &[]), blocker].into());
    let mut row = h.domain.task(1).expect("held row fixture").clone();
    assert!(hold_words(Some(&h.domain), &row).is_none());
    row.held_since = Some(HELD);
    row.phase_since = LATER;
    row.authority.budget.deadline = Some(DEADLINE);
    row.numbers.spent = 12;
    row.numbers.spent_below = 5;
    row.hold_wait_since = Some(DEADLINE);
    row.holdings = Box::new([holding]);
    h.domain.stubs.insert(6, crate::Stub { task: 6, phase: Status::Cancelled, result: Token::new(7) }).unwrap();
    add_placed(&mut h.domain, 12, b"migrating");
    add_placed(&mut h.domain, 13, b"");
    let wait = Some(HoldSource::Wait { words: WAIT.as_bytes().into() });
    let account = Some(connector_account(Some(42), b"permission denied on build/artifact"));
    for (why, source, cause, exit) in [
        (
            Hold::Exceeded(Exceeded { bound: Bound::Window, value: 2048 }),
            None,
            "It reached its model window bound of 2048 tokens.",
            "Lighten what was asked, or raise the allowance, then release it.",
        ),
        (
            Hold::Failures(Class::Permanent),
            Some(HoldSource::Preparation { words: b"workspace missing".as_slice().into() }),
            "It ran out of tries for permanent failures: workspace missing.",
            "Release it to try again; amend it first if its instructions failed it.",
        ),
        (Hold::Stalled, None, "Its executor made no progress within its stall bound.", "Release it to run again."),
        (
            Hold::Dependency(6),
            None,
            "Dependency task 6 was cancelled.",
            "Amend it to drop the dependency, then release it.",
        ),
        (
            Hold::Dependency(9),
            None,
            "Dependency task 9 failed or was cancelled.",
            "Amend it to drop the dependency, then release it.",
        ),
        (Hold::Stopped, None, "It was stopped.", "Release it."),
        (
            Hold::Placed { hold: 12 },
            None,
            "Hold 12 was placed by person 5: migrating.",
            "Person 5 lifts the hold or releases this task from it.",
        ),
        (
            Hold::Placed { hold: 13 },
            None,
            "Hold 13 was placed with no reason by person 5.",
            "Person 5 lifts the hold or releases this task from it.",
        ),
        (
            Hold::Drift,
            Some(HoldSource::Resource(name)),
            "A resource drifted: connector 7/build/artifact.",
            "Release it as the connector directs.",
        ),
        (
            Hold::Effects,
            Some(HoldSource::Effect { connector: 7, entry: 42 }),
            "An effect failed for good at entry 42.",
            RESET,
        ),
        (
            Hold::EffectFailed,
            account.clone(),
            "An effect failed for good at entry 42: permission denied on build/artifact.",
            RESET,
        ),
        (
            Hold::Uncertain { entry: 27 },
            None,
            "Effect entry 27 may or may not have happened on its system.",
            "Look at the system, then release it, making the effect again or not.",
        ),
        (
            Hold::Budget,
            None,
            "It spent 17; its next run or effect cannot be funded.",
            "Widen its budget, then release it.",
        ),
        (Hold::Deadline, None, "Its deadline 2026-11-24T06:00Z passed.", "Move its deadline, then release it."),
        (
            Hold::Deadline,
            wait.clone(),
            "Its deadline 2026-11-24T06:00Z passed. Waited 24 h for news since 2026-11-23T06:07Z: subscriptions 3 and 5.",
            "Move its deadline, then release it.",
        ),
        (
            Hold::HoldsWaited,
            None,
            "Its resource queue reached its bound after queuing since 2026-11-24T06:00Z: resource connector 7/build/artifact (held by task 20).",
            "Release it to queue again, or amend it to drop the holdings.",
        ),
        (
            Hold::ResourceUnavailable,
            account.clone(),
            "A resource is unavailable: permission denied on build/artifact.",
            DROP,
        ),
        (Hold::ResourceAccess, account, "Access to a resource was refused: permission denied on build/artifact.", DROP),
        (Hold::PoolLost { pool: 8 }, None, "It lost its allocation in pool 8.", "Release it to wait for a slot."),
        (
            Hold::Procedure,
            Some(connector_account(None, b"review needs a grant")),
            "Its procedure asked for a decision: review needs a grant.",
            "Release it to reset its counts.",
        ),
        (
            Hold::Circle,
            Some(HoldSource::Circle(Circle { tasks: Box::new([9, 4, 5]), cut: true })),
            "A circle of waits among tasks 9 -> 4 -> 5…",
            "It lifts once the circle breaks; amend a task on it to drop a dependency.",
        ),
        (Hold::Waited, wait, WAIT, "Release it to wait again, or amend or cancel it."),
    ] {
        let source = match source {
            Some(source) => Some(Box::new(source)),
            None => None,
        };
        row.phase = Phase::Held { was: Was::Waiting, why, source };
        let words = hold_words(Some(&h.domain), &row).expect("held words");
        let after_cause = words.bytes.strip_prefix(cause.as_bytes()).expect("held cause");
        let after_since = after_cause.strip_prefix(b" ").unwrap().strip_prefix(SINCE.as_bytes()).expect("held start");
        assert_eq!(after_since.strip_prefix(b" "), Some(exit.as_bytes()), "{why:?}");
        assert_eq!(words.cut, why == Hold::Circle, "{why:?}");
    }
}

#[test]
fn a_256_kib_placed_reason_is_cut_at_a_character_before_the_start_and_exit() {
    const REASON: usize = 256 * 1024;
    const HEAD: &str = "Hold 12 was placed by person 5: ";
    const LETTER: &str = "🧪";
    const TAIL: &str = "… Held since 2026-11-24T06:07Z. Person 5 lifts the hold or releases this task from it.";
    let mut h = Harness::new();
    h.env.wall = HELD;
    h.env.limits.message_bytes = u32::try_from(REASON).unwrap();
    h.make([task(1, &[])].into());
    let mut reason = skein_lib::Writer::new(REASON);
    reason.put(b"x").unwrap();
    for _ in 0..(REASON - 4) / LETTER.len() {
        reason.put(LETTER.as_bytes()).unwrap();
    }
    reason.put(b"end").unwrap();
    h.send(Event::Place { reply_to: to(), hold: placed(12, reason.finish()) });
    let words = hold_words(Some(&h.domain), h.domain.task(1).unwrap()).expect("placed hold words");
    assert!(skein_lib::bytes::is_utf8(&words.bytes));
    let reason_text = words.bytes.strip_prefix(HEAD.as_bytes()).unwrap().strip_suffix(TAIL.as_bytes()).unwrap();
    assert!(words.cut);
    assert!(words.bytes.len() <= crate::HOLD_WORDS);
    let (letters, remainder) = reason_text.strip_prefix(b"x").unwrap().as_chunks::<{ LETTER.len() }>();
    assert_eq!(remainder, []);
    for letter in letters {
        assert_eq!(letter.as_slice(), LETTER.as_bytes());
    }
    assert!(words.bytes.len() + LETTER.len() > crate::HOLD_WORDS, "keep room for as much of the reason as fits");
}

#[test]
fn a_row_without_held_since_keeps_its_latest_hold_time_across_a_phase_move() {
    let mut h = Harness::new();
    h.make([task(1, &[])].into());
    h.env.wall = DEADLINE;
    h.send(Event::Hold { task: 1, why: Hold::Drift });
    h.env.wall = HELD;
    h.send(Event::HoldFrom {
        task: 1,
        why: Hold::Drift,
        source: Box::new(HoldSource::Resource(Name { connector: 7, path: Box::new([b"other".as_slice().into()]) })),
    });
    let Stored::Live(row) = h.stored.get_mut(&Key::Live(1)).unwrap() else { panic!("live row") };
    row.held_since = None;
    let position = row.hold;
    let Some(Stored::Hold(line)) = h.stored.get(&Key::Hold { task: 1, position }) else { panic!("hold line") };
    assert_eq!(line.at, HELD);
    h.restart();
    let row = h.domain.task(1).unwrap();
    assert_eq!(row.held_since, None);
    let words = hold_words(Some(&h.domain), row).unwrap();
    assert!(skein_lib::bytes::find(&words.bytes, SINCE.as_bytes()).is_some());
    h.env.wall = LATER;
    h.send(Event::HoldFrom { task: 1, why: Hold::Drift, source: Box::new(HoldSource::Resource(resource())) });
    let row = h.domain.task(1).unwrap();
    assert_eq!(
        row.phase,
        Phase::Held {
            why: Hold::Drift,
            was: Was::Active(Active::Due),
            source: Some(Box::new(HoldSource::Resource(resource())))
        }
    );
    assert_eq!(row.phase_since, LATER);
    let words = hold_words(Some(&h.domain), row).unwrap();
    assert!(skein_lib::bytes::find(&words.bytes, SINCE.as_bytes()).is_some());
}
