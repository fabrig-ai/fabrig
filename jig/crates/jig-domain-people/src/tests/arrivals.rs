//! A need's arrival (domain/people/README.md, section 8).
use super::*;

fn question(message: u64) -> Entry {
    Entry { task: 7, project: 1, whom: Whom::Person(1), kind: EntryKind::Question { message }, at: Wall::EPOCH }
}

fn waiting(test: &mut Test, entries: &[Entry]) -> Vec<Request> {
    test.send(Event::Waiting { task: 7, entries: entries.into() })
}

fn arrival(party: u64, needs: u32) -> Request {
    Request::NeedArrived { party, project: 1, task: 7, needs }
}

#[test]
fn questions_tell_once_per_number_and_stop_counting_when_answered() {
    let mut test = Test::new(LIMITS);
    test.member();
    let first = question(11);
    let second = question(12);
    assert_eq!(waiting(&mut test, &[first]), [arrival(1, 1)]);
    assert_eq!(waiting(&mut test, &[first, second]), [arrival(1, 2)]);
    let refreshed = Entry { at: Wall::from_nanos(20), ..first };
    assert!(waiting(&mut test, &[refreshed, second]).is_empty(), "timestamps do not identify needs");
    assert!(waiting(&mut test, &[second]).is_empty(), "an answered question tells nothing");
    assert_eq!(waiting(&mut test, &[second, question(13)]), [arrival(1, 2)]);
}

#[test]
fn role_arrivals_reach_each_current_holder_only() {
    let mut test = Test::new(LIMITS);
    test.member();
    test.signin(2, 20, identity(0, 2));
    let holdings = Box::new([
        Holding { person: 1, role: Role::Policy { role: 4 } },
        Holding { person: 2, role: Role::Policy { role: 4 } },
    ]);
    test.send(Event::Roles { project: 1, holdings: holdings.clone() });
    let entry = Entry { whom: Whom::Role { project: 1, role: 4 }, kind: EntryKind::PersonTask, ..question(11) };
    assert_eq!(waiting(&mut test, &[entry]), [arrival(1, 1), arrival(2, 1)]);
    assert_eq!(waiting(&mut test, &[entry]).len(), 0);
    let unheld = Entry { whom: Whom::Role { project: 1, role: 5 }, ..entry };
    assert!(waiting(&mut test, &[unheld]).is_empty(), "a role without holders tells nobody");
    let roles = test.send(Event::Roles {
        project: 1,
        holdings: Box::new([
            Holding { person: 1, role: Role::Policy { role: 5 } },
            Holding { person: 2, role: Role::Policy { role: 5 } },
        ]),
    });
    for request in roles {
        if let Request::NeedArrived { .. } = request {
            panic!("giving a role over existing needs is not an arrival");
        }
    }
    assert_eq!(waiting(&mut test, &[unheld]).len(), 0);
}

#[test]
fn a_named_person_without_project_membership_gets_no_arrival() {
    let mut test = Test::new(LIMITS);
    test.member();
    test.signin(2, 20, identity(0, 2));
    test.send(Event::Roles { project: 2, holdings: Box::new([Holding { person: 2, role: Role::Member }]) });
    assert_eq!(waiting(&mut test, &[Entry { whom: Whom::Person(2), ..question(11) }]).len(), 0);
    assert_eq!(test.d.project_needs(2, 1), 0);
}

#[test]
fn proposals_and_escalation_revisions_tell_but_updates_do_not() {
    let mut test = Test::new(LIMITS);
    test.member();
    let proposal = Entry { kind: EntryKind::Proposal { number: 3 }, ..question(11) };
    assert_eq!(waiting(&mut test, &[proposal]), [arrival(1, 1)]);
    assert_eq!(waiting(&mut test, &[proposal]).len(), 0);
    let escalation = Entry { kind: EntryKind::Escalation { revision: 1 }, ..proposal };
    assert_eq!(waiting(&mut test, &[escalation]), [arrival(1, 1)]);
    assert_eq!(waiting(&mut test, &[escalation]).len(), 0);
    assert_eq!(
        waiting(&mut test, &[Entry { kind: EntryKind::Escalation { revision: 2 }, ..escalation }]),
        [arrival(1, 1)]
    );
    assert_eq!(
        waiting(
            &mut test,
            &[
                Entry { kind: EntryKind::Reply { message: 20 }, ..proposal },
                Entry { kind: EntryKind::ChatReply { message: 21 }, ..proposal },
            ]
        ).len(), 0
    );
    test.d.remember_result(
        &LIMITS,
        1,
        ResultRef { task: 8, project: 1, at: Wall::EPOCH, position: 30, forgettable: None },
    );
    assert_eq!(test.d.project_needs(1, 1), 0, "results and replies are updates, not needs");
}

#[test]
fn arrival_counts_only_this_partys_project_needs_past_the_cache() {
    let mut test = Test::new(LIMITS);
    test.member();
    test.signin(2, 20, identity(0, 2));
    test.send(Event::Roles {
        project: 1,
        holdings: Box::new([Holding { person: 1, role: Role::Member }, Holding { person: 2, role: Role::Member }]),
    });
    test.send(Event::Roles { project: 2, holdings: Box::new([Holding { person: 1, role: Role::Member }]) });
    test.send(Event::Waiting { task: 8, entries: Box::new([Entry { task: 8, project: 2, ..question(11) }]) });
    test.send(Event::Waiting {
        task: 9,
        entries: Box::new([Entry { task: 9, whom: Whom::Person(2), ..question(11) }]),
    });
    let entries = [question(11), question(12), question(13)];
    assert_eq!(waiting(&mut test, &entries), [arrival(1, 3), arrival(1, 3), arrival(1, 3)]);
    assert_eq!(test.d.project_needs(1, 1), 3);
    assert_eq!(test.d.project_needs(1, 2), 1);
    assert_eq!(test.d.cached_waiting(&LIMITS, 1).unwrap().len(), 2);
}

#[test]
fn rebuilding_the_census_is_silent_and_preserves_need_identity() {
    let mut test = Test::new(LIMITS);
    test.d = Domain::new(&LIMITS, Box::new([]), 2, Box::new([]));
    test.d.start_restore(Wall::EPOCH);
    test.send(Event::Restore { record: Stored::Person { number: 1, identity: identity(0, 1) } });
    test.send(Event::Restore {
        record: Stored::Roles { project: 1, holdings: Box::new([Holding { person: 1, role: Role::Member }]) },
    });
    assert!(test.d.start_project(&LIMITS, RestoreProjection::Waiting { task: 7, entries: Box::new([question(11)]) }));
    assert!(test.complete());
    assert_eq!(waiting(&mut test, &[question(11)]).len(), 0);
    assert_eq!(waiting(&mut test, &[question(11), question(12)]), [arrival(1, 2)]);
}
