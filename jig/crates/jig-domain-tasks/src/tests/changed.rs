//! Last-change facts are owned by accepted row mutations, including cold restore.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "ordinary Rust task decision assertions"
)]
use super::revisions::{Harness, task, to};
use crate::{
    Cancel, Contract, EscalationHolder, Event, Executor, Funder, Hold, Key, Numbers, Party, RecurringOverlap,
    RecurringTemplate, Request, Stored,
};
use skein_lib::Wall;

#[test]
fn changed_tracks_creation_real_changes_backward_walls_restore_and_ending() {
    let mut test = Harness::new();
    test.env.wall = Wall::from_nanos(100);
    test.send(Event::Make { reply_to: to(), creator: Party::Person(9), batch: Box::new([task(1, &[])]) });
    assert_eq!(test.domain.task(1).expect("created").changed, test.env.wall);
    test.env.wall = Wall::from_nanos(200);
    test.send(Event::Hold { task: 1, why: Hold::Stopped });
    assert_eq!(test.domain.task(1).expect("held").changed, test.env.wall);
    test.env.wall = Wall::from_nanos(50);
    test.send(Event::Hold { task: 1, why: Hold::Budget });
    assert_eq!(test.domain.task(1).expect("changed hold").changed, test.env.wall);
    test.route(1, EscalationHolder::Person(9), 1);
    test.env.wall = Wall::from_nanos(300);
    test.restart();
    assert_eq!(test.domain.task(1).expect("restored").changed, Wall::from_nanos(50));
    test.send(Event::Cancel {
        reply_to: to(),
        by: Party::Person(9),
        operation: Cancel { task: 1, reason: Box::new([]), grace: None, message: None },
    });
    test.send(Event::EffectsSettled { task: 1 });
    test.env.wall = Wall::from_nanos(400);
    test.send(Event::Settled { task: 1 });
    let Stored::Ended(row) = test.stored.get(&Key::Ended(1)).expect("ended snapshot") else { panic!("ended record") };
    assert_eq!(row.changed, Wall::from_nanos(400));
    assert_eq!(row.ended_at, Some(row.changed));
}

#[test]
fn duplicate_live_publication_does_not_replace_the_last_change_wall() {
    let mut test = Harness::new();
    let mut master = task(100, &[]);
    master.executor = Executor::Procedure { connector: 1, code: 1 };
    master.contract = Contract::Report { words: 0 };
    master.numbers = Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 };
    master.recurring =
        Some(Box::new(RecurringTemplate { key: 1, batch: Box::new([task(1, &[])]), overlap: RecurringOverlap::Skip }));
    test.env.wall = Wall::from_nanos(100);
    test.send(Event::Make { reply_to: to(), creator: Party::Deployment { project: 1 }, batch: Box::new([master]) });
    test.send(Event::OpenPeriod { reply_to: to(), project: 1, period: 1, budget: 1000 });
    test.env.wall = Wall::from_nanos(200);
    test.send(Event::TickRecurring { task: 100, period: 1 });
    let before = test.domain.task(100).expect("due recurring").clone();
    assert_eq!(before.funder, Funder::Period { project: 1, period: 1 });
    test.env.wall = Wall::from_nanos(300);
    let out = test.send(Event::TickRecurring { task: 100, period: 1 });
    let mut saved = false;
    for request in out {
        if let Request::Save { record: Stored::Live(row) } = request {
            assert_eq!(row.as_ref(), &before);
            saved = true;
        }
    }
    assert!(saved, "the no-op path really publishes a live snapshot");
    test.restart();
    test.env.wall = Wall::from_nanos(400);
    test.send(Event::TickRecurring { task: 100, period: 1 });
    assert_eq!(test.domain.task(100).expect("restored recurring").changed, before.changed);
}

fn part_saves(out: &[Request], number: u64) -> Vec<Key> {
    let mut keys: Vec<_> = out
        .iter()
        .filter_map(|request| match request {
            Request::Save { record: Stored::Live(row) } if row.number == number => Some(Key::Live(number)),
            Request::Save { record: Stored::LiveCold { number: found, .. } } if *found == number => {
                Some(Key::LiveCold(number))
            }
            _ => None,
        })
        .collect();
    keys.sort();
    keys.dedup();
    keys
}

#[test]
fn live_saves_put_cold_only_at_creation_or_a_cold_amendment_and_erase_both_at_end() {
    let mut h = Harness::new();
    h.env.limits.spec_bytes = 4096;
    let mut made = task(7, &[]);
    made.spec.words = vec![b'w'; 4096].into();
    let out = h.send(Event::Make { reply_to: to(), creator: Party::Person(9), batch: Box::new([made]) });
    assert_eq!(part_saves(&out, 7), [Key::Live(7), Key::LiveCold(7)]);
    let out = h.send(Event::Prepare { reply_to: to(), task: 7 });
    assert_eq!(part_saves(&out, 7), [Key::Live(7)]);
    let out = h.send(Event::Claim {
        reply_to: to(),
        task: 7,
        attempt: 1,
        resumed: false,
        budget: 10,
        account: None,
        writes: Box::new([]),
        reasons: crate::Reasons::FIRST,
        offered: crate::Reasons::ALL,
        session_end: None,
    });
    assert_eq!(part_saves(&out, 7), [Key::Live(7)]);
    let out = h.send(Event::Message {
        reply_to: to(),
        project: 1,
        operation: crate::Message {
            task: 7,
            word: crate::Word {
                number: 1,
                from: Party::Person(9),
                kind: crate::MessageKind::Words,
                words: Box::from(b"message".as_slice()),
                at: h.env.wall,
                hits: 1,
                eligible: false,
            },
        },
    });
    assert_eq!(part_saves(&out, 7), [Key::Live(7)]);
    let out = h.send(Event::Hold { task: 7, why: Hold::Stopped });
    assert_eq!(part_saves(&out, 7), [Key::Live(7)]);
    for field in 0_u8..3 {
        let mut amendment = super::revisions::amendment();
        match field {
            0 => {
                let mut spec = h.domain.task(7).unwrap().spec.clone();
                spec.words = Box::from(b"s".as_slice());
                amendment.spec = Some(spec);
            }
            1 => {
                let mut authority = h.domain.task(7).unwrap().authority.clone();
                authority.tools = crate::Tools(1);
                amendment.authority = Some(authority);
            }
            _ => amendment.instructions = Some(Box::from(b"instructions".as_slice())),
        }
        let out = h.send(Event::Amend {
            reply_to: to(),
            by: Party::Person(9),
            funder: None,
            places: Box::new([]),
            operation: crate::Amend {
                task: 7,
                message: h.domain.task(7).unwrap().last_message.checked_add(1).unwrap(),
                stop_run: false,
                amendment: Box::new(amendment),
            },
        });
        assert_eq!(part_saves(&out, 7), [Key::Live(7), Key::LiveCold(7)], "field {field}: {out:?}");
    }
    h.send(Event::Activation {
        reply_to: to(),
        task: 7,
        attempt: 1,
        cause: crate::Cause::Priced { cumulative: 0 },
        end: crate::End::Parked,
        saved: None,
    });
    h.send(Event::Cancel {
        reply_to: to(),
        by: Party::Person(9),
        operation: Cancel { task: 7, reason: Box::new([]), grace: None, message: None },
    });
    h.send(Event::EffectsSettled { task: 7 });
    let out = h.send(Event::Settled { task: 7 });
    assert!(out.iter().any(|r| match r {
        Request::Erase { key: Key::Live(7) } => true,
        _ => false,
    }));
    assert!(out.iter().any(|r| match r {
        Request::Erase { key: Key::LiveCold(7) } => true,
        _ => false,
    }));
    assert!(out.iter().any(|r| match r {
        Request::Save { record: Stored::Ended(row) } if row.number == 7 => true,
        _ => false,
    }));
}

#[test]
fn procedure_words_and_funding_transfers_save_the_cold_part() {
    let mut h = Harness::new();
    let mut procedure = task(1, &[]);
    procedure.executor = Executor::Procedure { connector: 1, code: 1 };
    h.make(vec![procedure]);
    let mut decision = crate::ProcedureDecision::new(crate::ProcedureEnd::Wait);
    decision.words = Some(Box::from(b"procedure words".as_slice()));
    let out = h.send(Event::Procedure { reply_to: to(), task: 1, step: 1, read: None, decision });
    assert_eq!(part_saves(&out, 1), [Key::Live(1), Key::LiveCold(1)]);
    h.make(vec![task(2, &[])]);
    h.send(Event::Prepare { reply_to: to(), task: 2 });
    h.send(Event::Claim {
        reply_to: to(),
        task: 2,
        attempt: 1,
        resumed: false,
        budget: 10,
        account: None,
        writes: Box::new([]),
        reasons: crate::Reasons::FIRST,
        offered: crate::Reasons::ALL,
        session_end: None,
    });
    h.send(Event::Activation {
        reply_to: to(),
        task: 2,
        attempt: 1,
        cause: crate::Cause::Priced { cumulative: 3 },
        end: crate::End::Parked,
        saved: None,
    });
    h.send(Event::CarvePool { reply_to: to(), project: 1, person: 9, period: 0, budget: 100 });
    let out = h.send(Event::Amend {
        reply_to: to(),
        by: Party::Person(9),
        places: Box::new([]),
        funder: Some(Funder::Pool { project: 1, person: 9, period: 0 }),
        operation: crate::Amend {
            task: 2,
            message: 1,
            stop_run: false,
            amendment: Box::new(super::revisions::amendment()),
        },
    });
    assert_eq!(h.domain.task(2).unwrap().authority.budget.spend, 97);
    assert_eq!(part_saves(&out, 2), [Key::Live(2), Key::LiveCold(2)]);
}
