//! Away and decision rules (temper/docs/design/domain/forge.md, 5.8, 5.9).
use super::*;
use jig_domain_people::ApplicationScope;
use skein_lib::ReplyTo;
use temper_protocol_reaches::{Class, Classes, Item, Kind, PathSet, Reach, Reason, Subject};

const PARTY: u64 = 7;
const OTHER: u64 = 8;
const PROJECT: u32 = 1;
const REQUEST: Token = Token::new(90);
const ANSWER: Token = Token::new(91);
const NOW: Wall = Wall::from_nanos(100);
const LATER: Wall = Wall::from_nanos(200);
const KEY: NamedCall = NamedCall { task: 3, attempt: 4, completion: 5, position: 6 };

fn away() -> Away {
    Away { party: PARTY, project: PROJECT, since: Wall::EPOCH, until: None, landed: 2, off: None }
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
        took: Some(1),
        why: Box::from(b"Because".as_slice()),
        overturned: false,
        reach: reach(Kind::Retry, Class::Decide, false, Reason::Kind),
    }
}
fn call(kind: Kind) -> DecidedCall {
    let row = decision(1);
    DecidedCall {
        question: row.question,
        options: row.options,
        took: 1,
        why: row.why,
        kind,
        paths: Box::new([]),
        under: 0,
    }
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
fn set(d: &mut Domain, wall: Wall, on: bool, until: Option<Wall>) -> Box<[Request]> {
    run(d, wall, Event::SetAway { request: REQUEST, token: ANSWER, party: PARTY, project: PROJECT, on, until })
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
fn set_saved(d: &mut Domain, on: bool, until: Option<Wall>, expected: Away) {
    assert_eq!(set(d, NOW, on, until).as_ref(), [saved(Stored::Away(expected)), mark(AWAY_SUBJECT), done()]);
    assert_eq!(d.aways.get(&(PARTY, PROJECT)), Some(&expected));
    assert_eq!(d.application_answer(ANSWER), Some(Answered::SetAway(SetAwayAnswer::Away)));
}
fn decide(d: &mut Domain, wall: Wall, call: DecidedCall) -> Box<[Request]> {
    run(
        d,
        wall,
        Event::Decided {
            to: ReplyTo::new(REQUEST),
            key: KEY,
            party: PARTY,
            project: PROJECT,
            task: KEY.task,
            always: false,
            call,
        },
    )
}
fn reply(took: Option<&[u8]>, reach: Option<Reach>) -> Request {
    let took = match took {
        Some(words) => Some(Box::from(words)),
        None => None,
    };
    Request::Decided { to: ReplyTo::new(REQUEST), key: KEY, took, reach }
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
            Away { since: NOW, until: Some(LATER), landed: landed.unwrap_or(0), ..away() },
        );
    }
}
#[test]
fn on_again_keeps_the_spell() {
    let mut d = with(&[Stored::Away(away())]);
    set_saved(&mut d, true, Some(LATER), Away { until: Some(LATER), ..away() });
}
#[test]
fn off_writes_when_it_went() {
    let mut d = with(&[Stored::Away(away())]);
    set_saved(&mut d, false, Some(LATER), Away { until: Some(LATER), off: Some(NOW), ..away() });
}
#[test]
fn past_until_it_is_not_on_and_the_spell_is_unchanged() {
    let row = Away { until: Some(NOW), ..away() };
    assert!(row.on_at(Wall::EPOCH));
    assert!(!row.on_at(NOW));
    assert!(!row.on_at(LATER));
    let mut d = with(&[Stored::Away(row)]);
    let reach = reach(Kind::Implementation, Class::Ask, false, Reason::Kind);
    let expected = asked(Decided { reach, ..decision(1) });
    assert_eq!(
        decide(&mut d, NOW, call(Kind::Implementation)).as_ref(),
        [saved(Stored::Decided(expected)), mark(decided_subject(PROJECT)), reply(None, Some(reach))]
    );
    assert_eq!(d.aways.get(&(PARTY, PROJECT)), Some(&row));
}
#[test]
fn on_past_until_is_a_new_spell() {
    for row in [Away { until: Some(NOW), ..away() }, Away { off: Some(Wall::EPOCH), ..away() }] {
        let mut d = with(&[Stored::Away(row)]);
        set_saved(&mut d, true, None, Away { since: NOW, landed: 0, ..away() });
    }
}
#[test]
fn off_past_until_keeps_the_end_at_until() {
    let mut d = with(&[Stored::Away(Away { until: Some(Wall::EPOCH), ..away() })]);
    set_saved(&mut d, false, Some(LATER), Away { until: Some(LATER), off: Some(Wall::EPOCH), ..away() });
}
#[test]
fn off_with_no_row_keeps_the_choices() {
    set_saved(
        &mut domain(),
        false,
        Some(LATER),
        Away { since: NOW, until: Some(LATER), landed: 0, off: Some(NOW), ..away() },
    );
}
#[test]
fn the_same_again_writes_nothing() {
    for row in [away(), Away { off: Some(Wall::EPOCH), ..away() }] {
        let mut d = with(&[Stored::Away(row)]);
        assert_eq!(set(&mut d, NOW, row.off.is_none(), None).as_ref(), [done()]);
        assert_eq!(d.application_answer(ANSWER), Some(Answered::SetAway(SetAwayAnswer::Away)));
    }
}
#[test]
fn past_the_bound_a_new_row_is_too_large() {
    let mut d = domain();
    for project in 2..(LIMITS.bindings * LIMITS.judge_projects) + 2 {
        outputs(&mut d, Event::Restore { record: Stored::Away(Away { project, ..away() }) });
    }
    assert_eq!(set(&mut d, NOW, true, None).as_ref(), [done()]);
    assert_eq!(d.application_answer(ANSWER), Some(Answered::SetAway(SetAwayAnswer::TooLarge)));
    assert!(d.aways.get(&(PARTY, PROJECT)).is_none());
    // A replacement still fits when the family is full.
    assert_eq!(
        run(
            &mut d,
            NOW,
            Event::SetAway { request: REQUEST, token: ANSWER, party: PARTY, project: 2, on: false, until: None }
        )
        .len(),
        3
    );
}
fn reach(kind: Kind, class: Class, away: bool, reason: Reason) -> Reach {
    Reach { kind, class, reason, away, under: 0, sets: 0, matched: 0 }
}
fn asked(mut row: Decided) -> Decided {
    row.took = None;
    row.options = Box::new([]);
    row.why = Box::new([]);
    row
}
#[test]
fn classes_judge_every_call_and_keep_its_reach() {
    for (kind, on, custom, class, reason) in [
        (Kind::Wording, false, false, Class::Decide, Reason::Kind),
        (Kind::Implementation, false, false, Class::Ask, Reason::Kind),
        (Kind::Implementation, true, false, Class::Ask, Reason::Kind),
        (Kind::Money, true, false, Class::Wait, Reason::Kind),
        (Kind::Other, true, false, Class::Wait, Reason::Kind),
        (Kind::Dependency, true, true, Class::AskAway, Reason::Kind),
        (Kind::Wording, true, true, Class::AskAway, Reason::Set(1)),
    ] {
        let mut d = domain();
        if on {
            outputs(&mut d, Event::Restore { record: Stored::Away(away()) });
        }
        if custom {
            let classes = Classes {
                items: Box::new([
                    Item { subject: Subject::Kind(Kind::Dependency), words: Box::new([]), class: Class::AskAway },
                    Item { subject: Subject::Kind(Kind::Wording), words: Box::new([]), class: Class::Decide },
                    Item { subject: Subject::Set(1), words: Box::new([]), class: Class::AskAway },
                ]),
                sets: Box::new([PathSet {
                    number: 1,
                    name: Box::from(b"Core".as_slice()),
                    patterns: Box::new([Box::from(b"src/**".as_slice())]),
                }]),
            };
            let size = temper_protocol_reaches::classes_record(&classes).unwrap().measure();
            outputs(
                &mut d,
                Event::Restore {
                    record: Stored::Setting(crate::settings::SettingRow {
                        key: crate::settings::SettingKey {
                            kind: 8,
                            scope: PROJECT,
                            number: 1,
                            slot: crate::settings::SettingSlot::Change,
                        },
                        edit: crate::settings::Edit::Set(crate::settings::SizedValue {
                            value: crate::settings::Value::Classes(classes),
                            size,
                        }),
                    }),
                },
            );
        }
        let mut call = call(kind);
        let mut reach = reach(kind, class, on, reason);
        if reason == Reason::Set(1) {
            call.paths = Box::new([Box::from(b"src/lib.rs".as_slice())]);
            reach.sets = 1;
            reach.matched = 1;
        }
        let mut row = Decided { reach, ..decision(1) };
        let took = if reach.decided() {
            Some(b"second".as_slice())
        } else {
            row = asked(row);
            None
        };
        assert_eq!(
            decide(&mut d, NOW, call).as_ref(),
            [saved(Stored::Decided(row.clone())), mark(decided_subject(PROJECT)), reply(took, Some(reach))],
            "{kind:?}, away {on}, custom {custom}"
        );
        assert_eq!(d.decisions.get(&(PROJECT, 1)), Some(&row));
    }
}
#[test]
fn an_option_past_the_list_answers_ask() {
    let mut d = with(&[Stored::Away(away())]);
    let mut call = call(Kind::Retry);
    call.took = 2;
    assert_eq!(decide(&mut d, NOW, call).as_ref(), [reply(None, None)]);
    assert!(d.decisions.is_empty());
}
#[test]
fn the_bound_keeps_a_day_sweeps_seven_days_and_numbers_rise() {
    for age in [DECIDED_DAY_NS, DECIDED_DAY_NS + 1, DECIDED_KEPT_NS, DECIDED_KEPT_NS + 1] {
        let mut env = env();
        env.limits.decided_per_project = 4;
        env.wall = Wall::from_nanos(DECIDED_KEPT_NS + 1000);
        let mut d = domain_with_limits(&env.limits);
        let oldest_at = Wall::from_nanos(env.wall.as_nanos() - age);
        let reach = reach(Kind::Retry, Class::Decide, false, Reason::Kind);
        for number in 1..=4 {
            let at = if number == 1 { oldest_at } else { env.wall };
            outputs(&mut d, Event::Restore { record: Stored::Decided(Decided { at, ..decision(number) }) });
        }
        outputs(&mut d, Event::Restore { record: Stored::Decided(Decided { project: 2, ..decision(1) }) });
        let mut out = Queue::uncapped();
        crate::away::decided(
            &mut d,
            &env,
            ReplyTo::new(REQUEST),
            KEY,
            PARTY,
            PROJECT,
            KEY.task,
            false,
            call(Kind::Retry),
            &mut out,
        );
        if age == DECIDED_DAY_NS {
            assert_eq!(out.pop(), Some(reply(None, Some(reach))));
            assert!(d.decisions.contains_key(&(PROJECT, 1)), "inside its day is never erased");
            assert!(!d.decisions.contains_key(&(PROJECT, 5)));
        } else {
            assert_eq!(out.pop(), Some(Request::Erase { key: Key::Decided { project: PROJECT, number: 1 } }));
            assert_eq!(out.pop(), Some(saved(Stored::Decided(Decided { at: env.wall, ..decision(5) }))));
            assert_eq!(out.pop(), Some(mark(decided_subject(PROJECT))));
            assert_eq!(out.pop(), Some(reply(Some(b"second"), Some(reach))));
            assert!(!d.decisions.contains_key(&(PROJECT, 1)));
            assert!(d.decisions.contains_key(&(PROJECT, 5)));
        }
        assert!(out.is_empty());
        assert_eq!(d.decisions.len(), 5);
        assert!(d.decisions.contains_key(&(2, 1)));
        // A later write sweeps every expired row, but still advances past the highest number.
        env.wall = Wall::from_nanos(env.wall.as_nanos() + DECIDED_KEPT_NS + 1);
        crate::away::decided(
            &mut d,
            &env,
            ReplyTo::new(REQUEST),
            KEY,
            PARTY,
            PROJECT,
            KEY.task,
            false,
            call(Kind::Retry),
            &mut out,
        );
        assert!(d.decisions.contains_key(&(PROJECT, if age == DECIDED_DAY_NS { 5 } else { 6 })));
        assert_eq!(d.decisions.len(), 2);
        assert!(d.decisions.contains_key(&(2, 1)));
    }
}
fn decide_at_a_full_day(with_expired: bool) -> (Domain, Queue<Request>) {
    let mut env = env();
    env.limits.decided_per_project = 4;
    env.wall = Wall::from_nanos(DECIDED_KEPT_NS + 1);
    let mut d = domain_with_limits(&env.limits);
    for number in 2..=5 {
        outputs(&mut d, Event::Restore { record: Stored::Decided(Decided { at: env.wall, ..decision(number) }) });
    }
    if with_expired {
        // Seed the sweep's excess row directly: restart refuses rows past the bound.
        d.decisions.insert((PROJECT, 1), Decided { at: Wall::EPOCH, ..decision(1) }).unwrap();
    }
    let mut out = Queue::uncapped();
    crate::away::decided(
        &mut d,
        &env,
        ReplyTo::new(REQUEST),
        KEY,
        PARTY,
        PROJECT,
        KEY.task,
        false,
        call(Kind::Retry),
        &mut out,
    );
    (d, out)
}
#[test]
fn a_sweep_at_the_bound_marks_before_answering_ask() {
    let (d, mut out) = decide_at_a_full_day(true);
    assert_eq!(out.pop(), Some(Request::Erase { key: Key::Decided { project: PROJECT, number: 1 } }));
    assert_eq!(out.pop(), Some(mark(decided_subject(PROJECT))));
    assert_eq!(out.pop(), Some(reply(None, Some(reach(Kind::Retry, Class::Decide, false, Reason::Kind)))));
    assert!(out.is_empty());
    assert!(!d.decisions.contains_key(&(PROJECT, 1)));
    assert_eq!(d.decisions.len(), 4);
}
#[test]
fn a_full_day_without_a_sweep_only_answers_ask() {
    let (d, mut out) = decide_at_a_full_day(false);
    assert_eq!(out.pop(), Some(reply(None, Some(reach(Kind::Retry, Class::Decide, false, Reason::Kind)))));
    assert!(out.is_empty());
    assert_eq!(d.decisions.len(), 4);
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
    let mut d = with(&[Stored::Decided(decision(1)), Stored::Decided(asked(decision(3)))]);
    for (party, number) in [(OTHER, 1), (PARTY, 2), (PARTY, 3)] {
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
    let d = with(&[
        Stored::Decided(decision(1)),
        Stored::Decided(Decided { took: Some(0), overturned: true, ..decision(3) }),
    ]);
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
                    overturned: false,
                    reach: decision(1).reach,
                },
                DecisionView {
                    number: 3,
                    task: KEY.task,
                    at: NOW,
                    question: decision(3).question,
                    took: Box::from(b"first".as_slice()),
                    why: decision(3).why,
                    overturned: true,
                    reach: decision(3).reach,
                },
            ]),
            asked: Box::new([]),
            cut: false,
        }))
    );

    for (decided_count, asked_count, cut) in [(128, 128, false), (129, 128, true), (128, 129, true)] {
        let limits = Limits { decided_per_project: 300, ..LIMITS };
        let mut d = domain_with_limits(&limits);
        d.restore_begin(Wall::EPOCH);
        let env = Env { limits, ..env() };
        let mut out = Queue::uncapped();
        for n in 1..=decided_count {
            step(&mut d, &env, Event::Restore { record: Stored::Decided(decision(n * 2)) }, &mut out);
        }
        for n in 1..=asked_count {
            step(&mut d, &env, Event::Restore { record: Stored::Decided(asked(decision(n * 2 + 1))) }, &mut out);
        }
        let view = d.decisions_view(PROJECT, PARTY);
        assert_eq!(view.cut, cut);
        assert_eq!(view.rows.len(), DECISIONS_SHOWN);
        assert_eq!(view.asked.len(), DECISIONS_SHOWN);
        assert_eq!(view.rows.first().unwrap().number, if decided_count == 129 { 4 } else { 2 });
        assert_eq!(view.asked.first().unwrap().number, if asked_count == 129 { 5 } else { 3 });
        assert_eq!(view.rows.last().unwrap().number, decided_count * 2);
        assert_eq!(view.asked.last().unwrap().number, asked_count * 2 + 1);
        for rows in view.rows.windows(2) {
            assert!(rows[0].number < rows[1].number);
        }
        for rows in view.asked.windows(2) {
            assert!(rows[0].number < rows[1].number);
        }
    }
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
                Stored::Decided(Decided {
                    project: n / LIMITS.decided_per_project,
                    ..decision(n % LIMITS.decided_per_project + 1)
                })
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
    for n in 1..=LIMITS.decided_per_project {
        outputs(&mut d, Event::Restore { record: Stored::Decided(decision(n)) });
    }
    outputs(&mut d, Event::Restore { record: Stored::Decided(decision(LIMITS.decided_per_project)) }); // replacement fits
    assert!(d.restore_fit_failure().is_none());
    outputs(&mut d, Event::Restore { record: Stored::Decided(decision(LIMITS.decided_per_project + 1)) });
    assert_eq!(
        d.restore_fit_failure(),
        Some(&RestoreFit {
            family: Some(RecordKind::Decided),
            set: RestoreSet::Rows(RecordKind::Decided),
            held: u64::from(LIMITS.decided_per_project),
            additional: 1,
            limit: u64::from(LIMITS.decided_per_project)
        })
    );
    assert!(!d.decisions.contains_key(&(PROJECT, 65)));
    assert_eq!(Limits { judge_projects: u32::MAX, ..LIMITS }.decided(), u32::MAX);
}
