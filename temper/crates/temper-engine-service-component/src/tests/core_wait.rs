use crate::{CoreWait, Note, core_wait::Watch};
use jig_domain_core::start::Phase;
use skein_lib::{Duration, Time, Wall};

// protocol/service-process.md, 1.2 bounds the whole line, including its newline.
const LINE_BYTES: usize = 240;

fn at(ms: u64) -> Time {
    Time::ZERO.saturating_add(Duration::from_millis(ms))
}

#[test]
fn kept_settings_warn_once_after_start_and_none_warns_of_nothing() {
    use crate::tests::fixtures;
    use jig_domain_core::{
        CoreRecord, Record,
        settings::{self, Setting, Value},
    };
    use temper_engine_domain as root;
    let config = || {
        let mut settings = fixtures::engine();
        settings.jig.limits.core.tasks.charters = 14;
        settings.jig.limits.core.templates = 10;
        settings
    };
    for kept in [false, true] {
        let mut service = fixtures::start_configured(config(), &[(1, 100)], None, false, false);
        fixtures::restart(&mut service);
        let mut engine = service.engine.take().unwrap();
        if kept {
            let config = config();
            let charter = |number| {
                let mut value = config.jig.charters.first().expect("stock charter").clone();
                value.number = number;
                Value::Charter(Box::new(value))
            };
            let template = |number, executor| {
                let mut value = config.jig.templates.first().expect("stock template").clone();
                value.number = number;
                value.executor = jig_domain_people::TemplateExecutor::Charter(executor);
                Value::Template(Box::new(value))
            };
            let writes = [
                (Setting::Charter(17), charter(17)),
                (Setting::Charter(19), charter(19)),
                (Setting::Template(9), template(9, 19)),
                (Setting::Template(16), template(16, 17)),
            ]
            .map(|(setting, value)| {
                root::Write::Save(root::Record::Core(Record::Core(CoreRecord::Settings(settings::Record::Changed {
                    setting,
                    changed: settings::Changed { value: Some(value), revision: 1 },
                }))))
            });
            let commit = engine.domain.deployment().commits.checked_add(1).unwrap();
            fixtures::commit(&mut engine.store, commit, Box::new(writes));
            let Some(jig_protocol_store::Event::Committed { .. }) = engine.store.host.above.pop() else {
                panic!("fixture changes committed")
            };
        }
        let stored = fixtures::reopened_store(engine.store, &engine.env.limits);
        let mut service = fixtures::start_configured(config(), &[(1, 100)], Some(stored), false, false);
        assert!(!service.engine.as_ref().unwrap().domain.ready());
        for note in &service.engine.as_ref().unwrap().notes {
            if let Note::StartupWarning { line } = note {
                assert!(!line.starts_with(b"settings kept"));
            }
        }
        fixtures::restart(&mut service);
        fixtures::restart(&mut service);
        let expected = b"settings kept at stock numbers as a deployment's own (the stock's 1 to 32, a deployment's own 33 to 64): charter 17, charter 19, template 9, template 16; a release whose stock brings one of these numbers refuses its start until each moves to 33 to 64";
        let mut warnings = 0_u32;
        for note in &service.engine.as_ref().unwrap().notes {
            if let Note::StartupWarning { line } = note
                && line.starts_with(b"settings kept")
            {
                assert_eq!(line.as_ref(), expected);
                warnings = warnings.saturating_add(1);
            }
        }
        assert_eq!(warnings, u32::from(kept));
    }
}

#[test]
fn a_wait_names_its_reason_at_one_second_and_on_change() {
    let mut watch = Watch::new();
    let why = CoreWait::Starting { step: None };
    for (ms, expected) in [(0, false), (999, false), (1000, true), (2000, false)] {
        assert_eq!(watch.kept(at(ms), why), expected.then_some(Note::CoreWaiting { why }));
    }
    let why = CoreWait::ForgeRecords;
    assert_eq!(watch.kept(at(2000), why), Some(Note::CoreWaiting { why }));
    assert_eq!(watch.kept(at(3000), why), None);
}

#[test]
fn passing_reports_answers_once_only_after_a_wait_line() {
    for written in [false, true] {
        let mut watch = Watch::new();
        let why = CoreWait::Resolving;
        assert_eq!(watch.kept(at(0), why), None);
        if written {
            assert_eq!(watch.kept(at(1000), why), Some(Note::CoreWaiting { why }));
        }
        assert_eq!(watch.passed(), written.then_some(Note::CoreAnswers));
        assert_eq!(watch.passed(), None);
        assert_eq!(watch.kept(at(3000), why), None, "passing resets the wait's start");
    }
}

#[test]
fn each_wait_names_the_gate_that_holds_it() {
    for (why, words) in [
        (CoreWait::Starting { step: None }, "starting: loading the core's records"),
        (CoreWait::Starting { step: Some(Phase::LoadCore) }, "starting: loading the core's records"),
        (CoreWait::Starting { step: Some(Phase::LoadInbox) }, "starting: loading the inbox"),
        (
            CoreWait::Starting { step: Some(Phase::RestoreConnector { connector: 1 }) },
            "starting: loading the forge's records",
        ),
        (CoreWait::Starting { step: Some(Phase::Link) }, "starting: linking the forge's records"),
        (CoreWait::Starting { step: Some(Phase::Sweep) }, "starting: sweeping"),
        (CoreWait::Starting { step: Some(Phase::Switch) }, "starting: switching the store"),
        (CoreWait::Starting { step: Some(Phase::Wires) }, "starting: committing the wires"),
        (CoreWait::Starting { step: Some(Phase::AdoptRuns) }, "starting: adopting runs"),
        (CoreWait::Starting { step: Some(Phase::Open) }, "starting: opening decisions"),
        (CoreWait::ForgeRecords, "loading the forges' records"),
        (CoreWait::FirstPeriods, "opening the first periods"),
        (CoreWait::Resolving, "resolving the forges' names"),
        (
            CoreWait::Store { oldest: 7, in_flight: 3 },
            "waiting for the store to make commit 7 durable, the oldest of 3 in flight",
        ),
        (CoreWait::Watermark { backlog: 9, watermark: 8 }, "at its watermark: 9 of 8 in flight"),
        (
            CoreWait::Store { oldest: u64::MAX, in_flight: u64::MAX },
            "waiting for the store to make commit 18446744073709551615 durable, the oldest of 18446744073709551615 in flight",
        ),
        (
            CoreWait::Watermark { backlog: u64::MAX, watermark: u32::MAX },
            "at its watermark: 18446744073709551615 of 4294967295 in flight",
        ),
    ] {
        assert_eq!(why.words().as_ref(), words.as_bytes());
        assert!(b"temper serve: the core is not answering: ".len() + why.words().len() < LINE_BYTES);
    }
}

fn input() -> temper_engine_domain::Event {
    temper_engine_domain::Event::Party(temper_engine_domain::PartyInput::Ask {
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(72)),
        models: Box::new([]),
        sign_in: 99,
        key: [7; 16],
        ask: jig_domain_people::Ask::SetRoles { project: 1, holdings: Box::new([]) },
    })
}

#[test]
fn a_start_step_reports_through_the_service_without_party_input() {
    let mut service = super::fixtures::started();
    let engine = service.engine.as_mut().expect("engine");
    assert_eq!(engine.domain.next_deadline(&engine.env), Some(at(30_000)));
    for ms in [29_000, 30_000, 45_000, 60_000] {
        crate::engine::iterate(engine, at(ms), Wall::EPOCH);
        if ms == 30_000 || ms == 60_000 {
            let age = if ms == 30_000 { b"30 s".as_slice() } else { b"60 s" };
            assert_eq!(
                engine.notes.pop(),
                Some(Note::StartWaiting {
                    line: skein_lib::bytes::concat(&[
                        b"starting: loading the core's records for ",
                        age,
                        b": 1 store page (the core's records)"
                    ]),
                })
            );
        }
        assert!(engine.notes.is_empty(), "no client asked and each doubling writes once");
    }
}

#[test]
fn a_service_names_starting_after_a_second_of_refused_inputs() {
    let mut service = super::fixtures::started();
    let engine = service.engine.as_mut().expect("engine");
    for ms in [0, 999, 1000, 2000] {
        engine.clients.early.push(input());
        crate::engine::iterate(engine, at(ms), Wall::EPOCH);
    }
    assert_eq!(engine.notes.pop(), Some(Note::CoreWaiting { why: CoreWait::Starting { step: Some(Phase::LoadCore) } }));
    assert!(engine.notes.is_empty());
}

#[test]
fn a_me_held_for_one_second_names_the_start_gate() {
    use temper_engine_domain::{Event, PartyInput};
    let mut service = super::fixtures::started();
    let engine = service.engine.as_mut().expect("engine");
    engine.clients.early.push(Event::Party(PartyInput::Watch {
        watcher: skein_lib::Token::new(72),
        sign_in: 99,
        subject: jig_domain_core::documents::Watched::Frame,
        window: jig_domain_core::documents::Window { rows: 1, after: None },
    }));
    for ms in [0, 999] {
        crate::engine::iterate(engine, at(ms), Wall::EPOCH);
        assert!(engine.notes.is_empty(), "no line before one second");
    }
    crate::engine::iterate(engine, at(1000), Wall::EPOCH);
    assert_eq!(engine.notes.pop(), Some(Note::CoreWaiting { why: CoreWait::Starting { step: Some(Phase::LoadCore) } }));
    crate::engine::iterate(engine, at(2000), Wall::EPOCH);
    assert!(engine.notes.is_empty(), "no second line for the same reason");
}

#[test]
fn a_journal_stop_ends_the_service_with_its_reason() {
    let mut service = super::fixtures::ready();
    let engine = service.engine.as_mut().expect("engine");
    engine.step(temper_engine_domain::Event::Store(temper_engine_domain::StoreInput::Committed {
        number: u64::MAX,
        fill: 0,
    }));
    crate::engine::release(engine);
    let why = engine.domain.journal_stop().expect("refused store answer");
    assert_eq!(engine.end, Some(crate::End::Stopped(crate::Stop::Journal { why })));
}

#[test]
fn a_held_party_input_names_the_watermark_then_answers_again() {
    let mut service = super::fixtures::ready();
    let engine = service.engine.as_mut().expect("engine");
    // Each unauthenticated call retains one refusal behind the journal.
    let watermark = engine.env.limits.watermark;
    for _ in engine.domain.backlog()..u64::from(watermark) {
        engine.step(input());
    }
    let backlog = u64::from(watermark);
    crate::engine::clock(engine, at(0), Wall::EPOCH);
    engine.step(input());
    assert!(engine.blocked);
    crate::engine::clock(engine, at(1000), Wall::EPOCH);
    let held = engine.pending.take().expect("held call");
    engine.step(*held);
    assert_eq!(engine.notes.pop(), Some(Note::CoreWaiting { why: CoreWait::Watermark { backlog, watermark } }));
    crate::engine::release(engine);
    let held = engine.pending.take().expect("still held at the watermark");
    engine.step(*held);
    assert!(engine.pending.is_none());
    assert_eq!(engine.notes.pop(), Some(Note::CoreAnswers));
    assert!(engine.notes.is_empty());
}
