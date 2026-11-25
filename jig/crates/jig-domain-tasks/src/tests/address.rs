//! Person address replacements through the task owner's public entrance.
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "unit tests build fixtures and inspect selected child events"
)]
use super::revisions::{Harness, task, to};
use crate::{AddressChange, Amendment, Event, Executor, Key, Party, PersonAddress, Refusal, Request, Stored};
use alloc::vec::Vec;
use skein_lib::{Time, Wall};

fn change(revision: u64, address: PersonAddress) -> Amendment {
    Amendment {
        tracked: None,
        addressed: Some(AddressChange { revision, address }),
        release: None,
        instructions: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        llm: None,
        reason: b"new addressee".as_slice().into(),
    }
}
fn amend(h: &mut Harness, change: Amendment) -> Vec<Request> {
    h.send(Event::Amend {
        operation: crate::Amend {
            funder: None,
            task: 1,
            message: h.domain.task(1).unwrap().last_message.checked_add(1).unwrap(),
            stop_run: false,
            amendment: Box::new(change),
        },
        places: Box::new([]),
        reply_to: to(),
        by: Party::Person(9),
    })
}

#[test]
fn readdressing_releases_the_taker_moves_only_the_person_revision_and_keeps_history_after_restart() {
    let mut h = Harness::new();
    let mut member = task(1, &[]);
    member.executor = Executor::Person(PersonAddress::Role(4));
    h.make(Vec::from([member]));
    h.send(Event::TakePerson { reply_to: to(), task: 1, person: 7 });
    assert_eq!(h.domain.task(1).unwrap().taken_by, Some(7));
    let wait = h.domain.task(1).unwrap().person_wait.expect("active person wait");
    h.env.now = Time::from_nanos(100);
    h.env.wall = Wall::from_nanos(100);
    let out = amend(&mut h, change(0, PersonAddress::Person(8)));
    assert!(
        out.iter().any(|r| match r {
            Request::Done { .. } => true,
            _ => false,
        }),
        "{out:?}"
    );
    let row = h.domain.task(1).unwrap();
    assert_eq!(row.executor, Executor::Person(PersonAddress::Person(8)));
    assert_eq!(row.taken_by, None);
    assert_eq!(row.revisions.person, 1);
    assert_eq!(row.revisions.spec, 0);
    let fresh = row.person_wait.expect("the new addressee has a wait");
    assert_eq!(fresh.since, h.env.wall);
    assert!(fresh.since > wait.since);
    assert_eq!(fresh.offer, None);
    let Stored::History(history) = &h.stored[&Key::History { task: 1, revision: 1 }] else { panic!("history") };
    let settings = history.amendment.as_ref().unwrap();
    assert_eq!(settings.before.addressed, Some(PersonAddress::Role(4)));
    assert_eq!(settings.before.taken_by, Some(7));
    assert_eq!(settings.after.addressed, Some(PersonAddress::Person(8)));
    assert_eq!(settings.after.taken_by, None);
    let history = history.clone();
    h.restart();
    assert_eq!(h.domain.task(1).unwrap().executor, Executor::Person(PersonAddress::Person(8)));
    assert_eq!(h.domain.task(1).unwrap().revisions.person, 1);
    assert_eq!(h.stored[&Key::History { task: 1, revision: 1 }], Stored::History(history));
    amend(&mut h, change(1, PersonAddress::Role(u32::MAX)));
    assert_eq!(h.domain.task(1).unwrap().executor, Executor::Person(PersonAddress::Role(u32::MAX)));
    assert_eq!(h.domain.task(1).unwrap().revisions.person, 2);
    let wait = h.domain.task(1).unwrap().person_wait;
    h.env.now = Time::from_nanos(200);
    h.env.wall = Wall::from_nanos(200);
    amend(&mut h, change(2, PersonAddress::Role(u32::MAX)));
    let unchanged = h.domain.task(1).unwrap();
    assert_eq!(unchanged.revisions.person, 2, "equal unclaimed address keeps its semantic revision");
    assert_eq!(unchanged.person_wait, wait, "no new addressee, no new wait");
}

#[test]
fn stale_invalid_and_nonperson_addresses_refuse_the_whole_amendment_before_mutation() {
    for (executor, revision, address, expected) in [
        (
            Executor::Person(PersonAddress::Role(4)),
            1,
            PersonAddress::Person(8),
            Refusal::PersonRevision { revision: 0 },
        ),
        (Executor::Person(PersonAddress::Role(4)), 0, PersonAddress::Person(0), Refusal::Executor),
        (Executor::Agent { charter: 1 }, 0, PersonAddress::Person(8), Refusal::Executor),
        (Executor::Procedure { connector: 1, code: 1 }, 0, PersonAddress::Person(8), Refusal::Executor),
    ] {
        let mut h = Harness::new();
        let mut member = task(1, &[]);
        member.executor = executor;
        h.make(Vec::from([member]));
        let before = h.domain.task(1).unwrap().clone();
        let mut edit = change(revision, address);
        let mut spec = before.spec.clone();
        spec.words = b"must not land".as_slice().into();
        edit.spec = Some(spec);
        let out = amend(&mut h, edit);
        assert!(
            out.iter().any(|r| match r {
                Request::Refused { problem, .. } => problem.why == expected,
                _ => false,
            }),
            "{out:?}"
        );
        assert_eq!(h.domain.task(1).unwrap(), &before);
        assert!(!out.iter().any(|r| match r {
            Request::Save { .. } | Request::Relay { .. } => true,
            _ => false,
        }));
    }
}
