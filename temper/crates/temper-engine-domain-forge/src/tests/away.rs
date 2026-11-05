//! Away and decision rules (temper/docs/design/domain/forge.md, 5.8, 5.9).
use super::*;
use jig_domain_people::ApplicationScope;
use skein_lib::ReplyTo;

const PARTY: u64 = 7;
const OTHER: u64 = 8;
const PROJECT: u32 = 1;
const REQUEST: Token = Token::new(90);
const ANSWER: Token = Token::new(91);
const NOW: Wall = Wall::from_nanos(100);
const LATER: Wall = Wall::from_nanos(200);
const KEY: NamedCall = NamedCall { task: 3, attempt: 4, completion: 5, position: 6 };

fn away() -> Away {
    Away { party: PARTY, project: PROJECT, since: Wall::EPOCH, until: None, kinds: 7, landed: 2, off: None }
}
fn decision(number: u32) -> Decided {
    Decided {
        project: PROJECT,
        number,
        party: PARTY,
        task: KEY.task,
        at: NOW,
        question: Box::from(b"Which?".as_slice()),
        options: Box::new([Box::from(b"first".as_slice()), Box::from(b"second".as_slice())]),
        took: 1,
        why: Box::from(b"Because".as_slice()),
        overturned: false,
    }
}
fn call(kind: DecisionKind) -> DecidedCall {
    let row = decision(1);
    DecidedCall { question: row.question, options: row.options, took: row.took, why: row.why, kind }
}
fn run(d: &mut Domain, wall: Wall, event: Event) -> Box<[Request]> {
    let mut env = env();
    env.wall = wall;
    let mut out = Queue::uncapped();
    step(d, &env, event, &mut out);
    let mut rows = skein_lib::List::with_capacity(out.len());
    for _ in 0..out.len() {
        rows.push(out.pop().unwrap()).unwrap();
    }
    rows.into_boxed()
}
fn with(rows: &[Stored]) -> Domain {
    let mut d = domain();
    for row in rows {
        outputs(&mut d, Event::Restore { record: row.clone() });
    }
    d
}
fn set(d: &mut Domain, wall: Wall, on: bool, until: Option<Wall>, kinds: u8) -> Box<[Request]> {
    run(d, wall, Event::SetAway { request: REQUEST, token: ANSWER, party: PARTY, project: PROJECT, on, until, kinds })
}
fn done() -> Request {
    Request::ApplicationDone { request: REQUEST, token: ANSWER }
}
fn mark(subject: u64) -> Request {
    Request::Marked { tasks: Box::new([subject]) }
}
fn saved(row: Stored) -> Request {
    Request::Save { record: row, release: None, read_afresh: None }
}
fn set_saved(d: &mut Domain, on: bool, until: Option<Wall>, kinds: u8, expected: Away) {
    assert_eq!(set(d, NOW, on, until, kinds).as_ref(), [saved(Stored::Away(expected)), mark(AWAY_SUBJECT), done()]);
    assert_eq!(d.aways.get(&(PARTY, PROJECT)), Some(&expected));
    assert_eq!(d.application_answer(ANSWER), Some(Answered::SetAway(SetAwayAnswer::Away)));
}
fn decide(d: &mut Domain, wall: Wall, call: DecidedCall) -> Box<[Request]> {
    run(
        d,
        wall,
        Event::Decided { to: ReplyTo::new(REQUEST), key: KEY, party: PARTY, project: PROJECT, task: KEY.task, call },
    )
}
fn reply(took: Option<&[u8]>) -> Request {
    let took = match took {
        Some(words) => Some(Box::from(words)),
        None => None,
    };
    Request::Decided { to: ReplyTo::new(REQUEST), key: KEY, took }
}
fn overturn(d: &mut Domain, party: u64, number: u32) -> Box<[Request]> {
    outputs(d, Event::Overturn { request: REQUEST, token: ANSWER, party, project: PROJECT, number })
}

#[test]
fn a_new_spell_takes_the_wall_and_the_landed_count() {
    for landed in [None, Some(9)] {
        let mut d = domain();
        if let Some(landed) = landed {
            outputs(
                &mut d,
                Event::Restore {
                    record: Stored::ProjectFacts(ProjectFacts {
                        project: PROJECT,
                        landed,
                        last_landed: Wall::EPOCH,
                        brief: None,
                        running: None,
                    }),
                },
            );
        }
        set_saved(
            &mut d,
            true,
            Some(LATER),
            15,
            Away { since: NOW, until: Some(LATER), kinds: 15, landed: landed.unwrap_or(0), ..away() },
        );
    }
}
#[test]
fn on_again_keeps_the_spell() {
    let mut d = with(&[Stored::Away(away())]);
    set_saved(&mut d, true, Some(LATER), 8, Away { until: Some(LATER), kinds: 8, ..away() });
}
#[test]
fn off_writes_when_it_went() {
    let mut d = with(&[Stored::Away(away())]);
    set_saved(&mut d, false, Some(LATER), 8, Away { until: Some(LATER), kinds: 8, off: Some(NOW), ..away() });
}
#[test]
fn past_until_it_is_not_on_and_nothing_is_written() {
    let row = Away { until: Some(NOW), ..away() };
    assert!(row.on_at(Wall::EPOCH));
    assert!(!row.on_at(NOW));
    assert!(!row.on_at(LATER));
    let mut d = with(&[Stored::Away(row)]);
    assert_eq!(decide(&mut d, NOW, call(DecisionKind::Retry)).as_ref(), [reply(None)]);
    assert_eq!(d.aways.get(&(PARTY, PROJECT)), Some(&row));
}
#[test]
fn on_past_until_is_a_new_spell() {
    for row in [Away { until: Some(NOW), ..away() }, Away { off: Some(Wall::EPOCH), ..away() }] {
        let mut d = with(&[Stored::Away(row)]);
        set_saved(&mut d, true, None, 7, Away { since: NOW, landed: 0, ..away() });
    }
}
#[test]
fn off_past_until_keeps_the_end_at_until() {
    let mut d = with(&[Stored::Away(Away { until: Some(Wall::EPOCH), ..away() })]);
    set_saved(&mut d, false, Some(LATER), 8, Away { until: Some(LATER), kinds: 8, off: Some(Wall::EPOCH), ..away() });
}
#[test]
fn off_with_no_row_keeps_the_choices() {
    set_saved(
        &mut domain(),
        false,
        Some(LATER),
        8,
        Away { since: NOW, until: Some(LATER), kinds: 8, landed: 0, off: Some(NOW), ..away() },
    );
}
#[test]
fn the_same_again_writes_nothing() {
    for row in [away(), Away { off: Some(Wall::EPOCH), ..away() }] {
        let mut d = with(&[Stored::Away(row)]);
        assert_eq!(set(&mut d, NOW, row.off.is_none(), None, 7).as_ref(), [done()]);
        assert_eq!(d.application_answer(ANSWER), Some(Answered::SetAway(SetAwayAnswer::Away)));
    }
}
#[test]
fn past_the_bound_a_new_row_is_too_large() {
    let mut d = domain();
    for project in 2..(LIMITS.bindings * LIMITS.judge_projects) + 2 {
        outputs(&mut d, Event::Restore { record: Stored::Away(Away { project, ..away() }) });
    }
    assert_eq!(set(&mut d, NOW, true, None, 7).as_ref(), [done()]);
    assert_eq!(d.application_answer(ANSWER), Some(Answered::SetAway(SetAwayAnswer::TooLarge)));
    assert!(d.aways.get(&(PARTY, PROJECT)).is_none());
    // A replacement still fits when the family is full.
    assert_eq!(
        run(
            &mut d,
            NOW,
            Event::SetAway {
                request: REQUEST,
                token: ANSWER,
                party: PARTY,
                project: 2,
                on: false,
                until: None,
                kinds: 7
            }
        )
        .len(),
        3
    );
}
#[test]
fn decided_with_the_switch_on_saves_a_row_and_answers_its_words() {
    for (kind, bit) in [
        (DecisionKind::Implementation, 1),
        (DecisionKind::Retry, 2),
        (DecisionKind::Wording, 4),
        (DecisionKind::Dependency, 8),
    ] {
        assert_eq!(kind.bit(), Some(bit));
        let mut d = with(&[Stored::Away(Away { kinds: bit, ..away() })]);
        assert_eq!(
            decide(&mut d, NOW, call(kind)).as_ref(),
            [saved(Stored::Decided(decision(1))), mark(decided_subject(PROJECT)), reply(Some(b"second"))]
        );
        assert_eq!(d.decisions.get(&(PROJECT, 1)), Some(&decision(1)));
    }
}
#[test]
fn decided_with_the_switch_off_answers_ask_and_writes_nothing() {
    for row in [None, Some(Away { off: Some(Wall::EPOCH), ..away() })] {
        let mut d = domain();
        if let Some(row) = row {
            outputs(&mut d, Event::Restore { record: Stored::Away(row) });
        }
        assert_eq!(decide(&mut d, NOW, call(DecisionKind::Retry)).as_ref(), [reply(None)]);
        assert!(d.decisions.is_empty());
    }
}
#[test]
fn a_kind_not_ticked_answers_ask() {
    let mut d = with(&[Stored::Away(Away { kinds: 0, ..away() })]);
    assert_eq!(decide(&mut d, NOW, call(DecisionKind::Retry)).as_ref(), [reply(None)]);
}
#[test]
fn other_never_decides_whatever_the_bits() {
    assert_eq!(DecisionKind::Other.bit(), None);
    let mut d = with(&[Stored::Away(Away { kinds: 15, ..away() })]);
    assert_eq!(decide(&mut d, NOW, call(DecisionKind::Other)).as_ref(), [reply(None)]);
}
#[test]
fn an_option_past_the_list_answers_ask() {
    let mut d = with(&[Stored::Away(away())]);
    let mut call = call(DecisionKind::Retry);
    call.took = 2;
    assert_eq!(decide(&mut d, NOW, call).as_ref(), [reply(None)]);
    assert!(d.decisions.is_empty());
}
#[test]
fn the_sixty_fifth_erases_the_lowest_and_numbers_rise() {
    let mut d = with(&[Stored::Away(away()), Stored::Decided(Decided { project: 2, ..decision(1) })]);
    for n in 1..=64 {
        decide(&mut d, NOW, call(DecisionKind::Retry));
        assert!(d.decisions.contains_key(&(PROJECT, n)));
    }
    for n in 65..=66 {
        assert_eq!(
            decide(&mut d, NOW, call(DecisionKind::Retry)).as_ref(),
            [
                Request::Erase { key: Key::Decided { project: PROJECT, number: n - 64 } },
                saved(Stored::Decided(decision(n))),
                mark(decided_subject(PROJECT)),
                reply(Some(b"second"))
            ]
        );
    }
    assert_eq!(d.decisions.len(), 65);
    assert!(d.decisions.contains_key(&(2, 1)));
}
#[test]
fn overturn_marks_once_then_refuses() {
    let mut d = with(&[Stored::Decided(decision(1))]);
    assert_eq!(
        overturn(&mut d, PARTY, 1).as_ref(),
        [saved(Stored::Decided(Decided { overturned: true, ..decision(1) })), mark(decided_subject(PROJECT)), done()]
    );
    assert_eq!(d.application_answer(ANSWER), Some(Answered::Overturn(OverturnAnswer::Overturned)));
    assert_eq!(overturn(&mut d, PARTY, 1).as_ref(), [done()]);
    assert_eq!(d.application_answer(ANSWER), Some(Answered::Overturn(OverturnAnswer::Already)));
}
#[test]
fn overturn_of_anothers_row_is_unknown() {
    let mut d = with(&[Stored::Decided(decision(1))]);
    for (party, number) in [(OTHER, 1), (PARTY, 2)] {
        assert_eq!(overturn(&mut d, party, number).as_ref(), [done()]);
        assert_eq!(d.application_answer(ANSWER), Some(Answered::Overturn(OverturnAnswer::Unknown)));
        assert_eq!(d.decisions.get(&(PROJECT, 1)), Some(&decision(1)));
    }
}
fn document(d: &Domain, subject: u64, scope: ApplicationScope, party: u64) -> Option<Document> {
    d.document(subject, scope, party, NOW, &LIMITS)
}
#[test]
fn the_documents_are_the_asking_partys_own() {
    let d = with(&[
        Stored::Away(Away { party: OTHER, ..away() }),
        Stored::Away(Away { project: 2, ..away() }),
        Stored::Away(away()),
        Stored::Decided(Decided { party: OTHER, ..decision(2) }),
        Stored::Decided(Decided { project: 2, ..decision(1) }),
        Stored::Decided(decision(1)),
    ]);
    assert_eq!(
        document(&d, AWAY_SUBJECT, ApplicationScope::Own, PARTY),
        Some(Document::Aways(AwaysView { rows: Box::new([away(), Away { project: 2, ..away() }]) }))
    );
    assert_eq!(
        document(&d, AWAY_SUBJECT, ApplicationScope::Own, OTHER),
        Some(Document::Aways(AwaysView { rows: Box::new([Away { party: OTHER, ..away() }]) }))
    );
    let Document::Decisions(view) =
        document(&d, decided_subject(PROJECT), ApplicationScope::Project(PROJECT), PARTY).unwrap()
    else {
        panic!("decisions")
    };
    assert_eq!(view.rows.len(), 1);
    assert_eq!(view.rows[0].number, 1);
    let Document::Decisions(view) =
        document(&d, decided_subject(PROJECT), ApplicationScope::Project(PROJECT), OTHER).unwrap()
    else {
        panic!("decisions")
    };
    assert_eq!(view.rows.len(), 1);
    assert_eq!(view.rows[0].number, 2);
    assert!(document(&d, decided_subject(PROJECT), ApplicationScope::Own, PARTY).is_none());
    assert!(document(&d, decided_subject(PROJECT), ApplicationScope::Project(2), PARTY).is_none());
    assert!(document(&d, AWAY_SUBJECT, ApplicationScope::Deployment, PARTY).is_none());
}
#[test]
fn decisions_say_the_options_words() {
    let d =
        with(&[Stored::Decided(decision(1)), Stored::Decided(Decided { took: 0, overturned: true, ..decision(3) })]);
    assert_eq!(
        document(&d, decided_subject(PROJECT), ApplicationScope::Project(PROJECT), PARTY),
        Some(Document::Decisions(DecisionsView {
            project: PROJECT,
            rows: Box::new([
                DecisionView {
                    number: 1,
                    task: KEY.task,
                    at: NOW,
                    question: decision(1).question,
                    took: Box::from(b"second".as_slice()),
                    why: decision(1).why,
                    overturned: false
                },
                DecisionView {
                    number: 3,
                    task: KEY.task,
                    at: NOW,
                    question: decision(3).question,
                    took: Box::from(b"first".as_slice()),
                    why: decision(3).why,
                    overturned: true
                },
            ])
        }))
    );
}
#[test]
fn both_are_erased_with_the_project() {
    let mut d = with(&[
        Stored::Away(away()),
        Stored::Away(Away { project: 2, ..away() }),
        Stored::Decided(decision(1)),
        Stored::Decided(Decided { project: 2, ..decision(1) }),
    ]);
    assert_eq!(
        outputs(&mut d, Event::ReleaseProject { project: PROJECT, request: REQUEST }).as_ref(),
        [
            Request::Erase { key: Key::Away { party: PARTY, project: PROJECT } },
            mark(AWAY_SUBJECT),
            Request::Erase { key: Key::Decided { project: PROJECT, number: 1 } },
            mark(decided_subject(PROJECT)),
            Request::ProjectReleased { request: REQUEST }
        ]
    );
    assert_eq!(d.aways.len(), 1);
    assert_eq!(d.decisions.len(), 1);
    assert_eq!(
        outputs(&mut d, Event::ReleaseProject { project: PROJECT, request: REQUEST }).as_ref(),
        [Request::ProjectReleased { request: REQUEST }]
    );
}
#[test]
fn rows_past_their_bound_are_refused_at_restart() {
    for family in [RecordKind::Away, RecordKind::Decided] {
        let mut d = domain();
        let limit = if family == RecordKind::Away { LIMITS.bindings * LIMITS.judge_projects } else { LIMITS.decided() };
        for n in 0..=limit {
            if n == limit {
                assert!(d.restore_fit_failure().is_none());
            }
            let record = if family == RecordKind::Away {
                Stored::Away(Away { project: n, ..away() })
            } else {
                Stored::Decided(Decided { project: n / 64, ..decision(n % 64 + 1) })
            };
            outputs(&mut d, Event::Restore { record });
        }
        assert_eq!(
            d.restore_fit_failure(),
            Some(&RestoreFit {
                family: Some(family),
                set: RestoreSet::Rows(family),
                held: u64::from(limit),
                additional: 1,
                limit: u64::from(limit)
            })
        );
    }
    let mut d = domain();
    for n in 1..=64 {
        outputs(&mut d, Event::Restore { record: Stored::Decided(decision(n)) });
    }
    outputs(&mut d, Event::Restore { record: Stored::Decided(decision(64)) }); // replacement fits
    assert!(d.restore_fit_failure().is_none());
    outputs(&mut d, Event::Restore { record: Stored::Decided(decision(65)) });
    assert_eq!(
        d.restore_fit_failure(),
        Some(&RestoreFit {
            family: Some(RecordKind::Decided),
            set: RestoreSet::Rows(RecordKind::Decided),
            held: 64,
            additional: 1,
            limit: 64
        })
    );
    assert!(!d.decisions.contains_key(&(PROJECT, 65)));
    assert_eq!(Limits { judge_projects: u32::MAX, ..LIMITS }.decided(), u32::MAX);
}
