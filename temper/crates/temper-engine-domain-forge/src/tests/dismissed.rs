use super::*;

const EARLIER_PARTY: u64 = 0;
const PARTY: u64 = 7;
const OTHER_PARTY: u64 = 8;
// Eight times four member bindings times two projects (the shared LIMITS fixture).
const DISMISSALS: u32 = 64;

fn row(party: u64, project: u32) -> Stored {
    Stored::Dismissed { party, what: PersonFact::ToBegin { project } }
}
fn restore_rows(count: u32) -> Domain {
    let mut d = domain();
    for project in 0..count {
        outputs(&mut d, Event::Restore { record: row(PARTY, project) });
    }
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    d
}

#[test]
fn dismissals_are_the_partys_own_in_key_order() {
    let mut d = domain();
    for record in [row(OTHER_PARTY, 1), row(PARTY, u32::MAX), row(EARLIER_PARTY, 4), row(PARTY, 0), row(PARTY, 3)] {
        outputs(&mut d, Event::Restore { record });
    }
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    for (party, expected) in [
        (
            PARTY,
            Box::<[PersonFact]>::from([
                PersonFact::ToBegin { project: 0 },
                PersonFact::ToBegin { project: 3 },
                PersonFact::ToBegin { project: u32::MAX },
            ]),
        ),
        (OTHER_PARTY, Box::<[PersonFact]>::from([PersonFact::ToBegin { project: 1 }])),
        (u64::MAX, Box::<[PersonFact]>::from([])),
    ] {
        assert_eq!(d.dismissals(party).as_ref(), expected.as_ref());
    }
}

#[test]
fn dismissals_restore_at_the_bound() {
    let d = restore_rows(DISMISSALS);
    assert_eq!(LIMITS.dismissed(), DISMISSALS);
    assert_eq!(d.restore_fit_failure(), None);
    let mut expected = skein_lib::List::with_capacity(DISMISSALS);
    for project in 0..DISMISSALS {
        expected.push(PersonFact::ToBegin { project }).expect("expected dismissal room");
    }
    assert_eq!(d.dismissals(PARTY).as_ref(), expected.into_boxed().as_ref());
}

#[test]
fn a_dismissal_past_the_bound_is_refused_at_restart() {
    let d = restore_rows(DISMISSALS.checked_add(1).unwrap());
    assert_eq!(
        d.restore_fit_failure(),
        Some(&RestoreFit {
            family: Some(RecordKind::Dismissed),
            set: RestoreSet::Rows(RecordKind::Dismissed),
            held: u64::from(DISMISSALS),
            additional: 1,
            limit: u64::from(DISMISSALS),
        })
    );
    assert_eq!(d.dismissals(PARTY).len(), usize::try_from(DISMISSALS).unwrap());
}

const REQUEST: Token = Token::new(90);
const ANSWER: Token = Token::new(91);

fn dismiss(d: &mut Domain, party: u64, project: u32) -> Box<[Request]> {
    outputs(d, Event::Dismiss { request: REQUEST, token: ANSWER, party, what: PersonFact::ToBegin { project } })
}
fn mark() -> Request {
    Request::Marked { tasks: Box::new([DISMISSED_SUBJECT]) }
}
fn done() -> Request {
    Request::ApplicationDone { request: REQUEST, token: ANSWER }
}

#[test]
fn a_dismissal_saves_its_row_marks_and_answers() {
    let mut d = domain();
    assert_eq!(
        dismiss(&mut d, PARTY, 1).as_ref(),
        [Request::Save { record: row(PARTY, 1), release: None, read_afresh: None }, mark(), done(),]
    );
    assert_eq!(d.dismissals(PARTY).as_ref(), [PersonFact::ToBegin { project: 1 }]);
    assert_eq!(d.application_answer(ANSWER), Some(Answered::Dismiss(DismissAnswer::Dismissed)));
}

#[test]
fn a_second_dismissal_is_answered_as_the_first_and_writes_nothing() {
    let mut d = restore_rows(DISMISSALS.checked_sub(1).unwrap());
    let project = DISMISSALS.checked_sub(1).unwrap();
    dismiss(&mut d, PARTY, project);
    let first = d.application_answer(ANSWER);
    let request = Token::new(92);
    let token = Token::new(93);
    assert_eq!(
        outputs(&mut d, Event::Dismiss { request, token, party: PARTY, what: PersonFact::ToBegin { project } })
            .as_ref(),
        [Request::ApplicationDone { request, token }]
    );
    assert_eq!(first, Some(Answered::Dismiss(DismissAnswer::Dismissed)));
    assert_eq!(d.application_answer(token), first);
    assert_eq!(d.dismissals(PARTY).len(), usize::try_from(DISMISSALS).unwrap());
}

#[test]
fn too_large_at_the_family_bound() {
    let mut d = restore_rows(DISMISSALS);
    assert_eq!(dismiss(&mut d, OTHER_PARTY, 1).as_ref(), [done()]);
    assert_eq!(d.application_answer(ANSWER), Some(Answered::Dismiss(DismissAnswer::TooLarge)));
    assert!(d.dismissals(OTHER_PARTY).is_empty());
    assert_eq!(d.dismissals(PARTY).len(), usize::try_from(DISMISSALS).unwrap());
}

#[test]
fn erase_project_erases_both_kinds() {
    let mut d = domain();
    for record in [row(PARTY, 1), goal_row(PARTY, 1), row(OTHER_PARTY, 1), row(PARTY, 2)] {
        outputs(&mut d, Event::Restore { record });
    }
    let released = outputs(&mut d, Event::ReleaseProject { project: 1, request: REQUEST });
    assert_eq!(
        released.as_ref(),
        [
            Request::Erase { key: Key::Dismissed { party: PARTY, what: PersonFact::ToBegin { project: 1 } } },
            Request::Erase { key: Key::Dismissed { party: PARTY, what: GOAL_FROM } },
            Request::Erase { key: Key::Dismissed { party: OTHER_PARTY, what: PersonFact::ToBegin { project: 1 } } },
            mark(),
            Request::ProjectReleased { request: REQUEST },
        ]
    );
    assert_eq!(d.dismissals(PARTY).as_ref(), [PersonFact::ToBegin { project: 2 }]);
    assert!(d.dismissals(OTHER_PARTY).is_empty());
}

#[test]
fn a_release_with_no_dismissals_marks_nothing() {
    let mut d = restore_rows(1);
    assert_eq!(
        outputs(&mut d, Event::ReleaseProject { project: 1, request: REQUEST }).as_ref(),
        [Request::ProjectReleased { request: REQUEST }]
    );
    assert_eq!(d.dismissals(PARTY).as_ref(), [PersonFact::ToBegin { project: 0 }]);
}

const GOAL_FROM: PersonFact = PersonFact::GoalFrom { project: 1, chat: 7, goal: 11, at: 0 };
fn goal_row(party: u64, project: u32) -> Stored {
    Stored::Dismissed { party, what: PersonFact::GoalFrom { project, chat: 7, goal: 11, at: 0 } }
}
fn dismiss_goal(d: &mut Domain) -> Box<[Request]> {
    outputs(d, Event::Dismiss { request: REQUEST, token: ANSWER, party: PARTY, what: GOAL_FROM })
}
#[test]
fn goal_from_is_saved_and_marked() {
    let mut d = domain();
    assert_eq!(
        dismiss_goal(&mut d).as_ref(),
        [Request::Save { record: goal_row(PARTY, 1), release: None, read_afresh: None }, mark(), done()]
    );
    assert_eq!(d.dismissals(PARTY).as_ref(), [GOAL_FROM]);
    assert_eq!(d.application_answer(ANSWER), Some(Answered::Dismiss(DismissAnswer::Dismissed)));
}
#[test]
fn a_second_goal_from_writes_nothing() {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: goal_row(PARTY, 1) });
    assert_eq!(dismiss_goal(&mut d).as_ref(), [done()]);
    assert_eq!(d.application_answer(ANSWER), Some(Answered::Dismiss(DismissAnswer::Dismissed)));
}
#[test]
fn too_large_at_the_partys_own_bound() {
    let mut limits = LIMITS;
    // Room for two whole party documents, with an additional project per member.
    limits.bindings = 2;
    limits.judge_projects = temper_client_protocol_wire::DISMISSED / 8 + 1;
    let mut d = domain_with_limits(&limits);
    for goal in 0..u64::from(temper_client_protocol_wire::DISMISSED.checked_sub(1).unwrap()) {
        outputs(
            &mut d,
            Event::Restore {
                record: Stored::Dismissed {
                    party: PARTY,
                    what: PersonFact::GoalFrom { project: 1, chat: 7, goal, at: 0 },
                },
            },
        );
    }
    // An earlier party's row must not be counted against this party.
    outputs(&mut d, Event::Restore { record: goal_row(EARLIER_PARTY, 1) });
    // The last admitted row is ToBegin, before every GoalFrom in the party's range.
    assert_eq!(
        dismiss(&mut d, PARTY, 2).as_ref(),
        [Request::Save { record: row(PARTY, 2), release: None, read_afresh: None }, mark(), done()]
    );
    assert_eq!(dismiss(&mut d, PARTY, 1).as_ref(), [done()]);
    assert_eq!(d.application_answer(ANSWER), Some(Answered::Dismiss(DismissAnswer::TooLarge)));
    assert_eq!(d.dismissals(PARTY).len(), usize::try_from(temper_client_protocol_wire::DISMISSED).unwrap());
    assert_eq!(
        dismiss(&mut d, OTHER_PARTY, 1).as_ref(),
        [Request::Save { record: row(OTHER_PARTY, 1), release: None, read_afresh: None }, mark(), done()]
    );
}
#[test]
fn another_partys_facts_are_not_answered() {
    let mut d = domain();
    for record in [goal_row(EARLIER_PARTY, 1), row(PARTY, 1), goal_row(PARTY, 1), goal_row(OTHER_PARTY, 1)] {
        outputs(&mut d, Event::Restore { record });
    }
    assert_eq!(d.dismissals(PARTY).as_ref(), [PersonFact::ToBegin { project: 1 }, GOAL_FROM]);
}
