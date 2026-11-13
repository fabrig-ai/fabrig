use crate::{CoreWait, Note, core_wait::Watch};
use jig_domain_core::start::Phase;
use skein_lib::{Duration, Time, Wall};

// protocol/README.md, 6.2 bounds the whole line, including its newline.
const LINE_BYTES: usize = 240;

fn at(ms: u64) -> Time {
    Time::ZERO.saturating_add(Duration::from_millis(ms))
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
        (CoreWait::Starting { step: Some(Phase::ReadAfresh { connector: 1 }) }, "starting: reading the forge afresh"),
        (
            CoreWait::Starting { step: Some(Phase::SettleOutbox { connector: 1 }) },
            "starting: settling the forge's outbox",
        ),
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
