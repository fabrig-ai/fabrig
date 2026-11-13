//! The engine's refusal text and consecutive note suppression (protocol/cluster.md, 3).
use super::fixtures;
use jig_domain_fleet::{AgentShortfall as A, Figures, HelloRefusal as H, HelloRefused};
use skein_lib::Token;

fn agent(why: A, figures: Option<Figures>) -> HelloRefused {
    HelloRefused { why: H::Agent { why }, figures }
}

fn cases() -> Box<[(Option<HelloRefused>, &'static [u8])]> {
    let figures = Some(Figures { offered: 32, required: 64 });
    let mut rows = skein_lib::List::with_capacity(27);
    for row in [
        (None, b"not admitted".as_slice()),
        (Some(agent(A::Model { charter: 3, model: 2 }, None)), b"charter 3, model 2: not in the agent's configuration under the same name"),
        (Some(agent(A::Window { charter: 3, model: 2 }, figures)), b"charter 3, model 2: needs a context window of 64 tokens; the agent offers 32 tokens"),
        (Some(agent(A::Output { charter: 3, model: 2 }, figures)), b"charter 3, model 2: needs 64 output tokens; the agent offers 32 tokens"),
        (Some(agent(A::Turns { charter: 3 }, figures)), b"charter 3: needs 64 turns; the agent's profile.policy.max_turns allows 32"),
        (Some(agent(A::Time { charter: 3 }, figures)), b"charter 3: needs 64 ms of run time; the agent's profile.policy.max_time allows 32 ms"),
        (Some(agent(A::Waiting { charter: 3 }, figures)), b"charter 3: needs 64 ms of waiting; the agent's profile.policy.max_waiting allows 32 ms"),
        (Some(agent(A::Checks { charter: 3 }, figures)), b"charter 3: needs checks within 64 ms; the agent's profile.policy.check_timeout is 32 ms; raise charter 3's reserve_time (temper help settings) or lower the timeout"),
        (Some(agent(A::Spend, figures)), b"needs a spend allowance of 64; the agent's profile.policy.max_spend allows 32"),
        (Some(agent(A::Message, figures)), b"needs 64-byte messages (tasks.message_bytes); the agent offers 32 (tool_payload less 2)"),
        (Some(agent(A::Inbox, figures)), b"needs an inbox of 64 (tasks.run_inbox); the agent's profile.policy.inbox allows 32"),
        (Some(agent(A::Unacknowledged, figures)), b"needs an unacknowledged-turn allowance of 64 (hub.turns); the agent's profile.policy.unacknowledged allows 32"),
        (Some(HelloRefused { why: H::JobKinds, figures: None }), b"its job kinds are not ascending and distinct"),
        (Some(HelloRefused { why: H::ReservedChannel, figures: None }), b"channel zero is the engine's own host"),
        (Some(HelloRefused { why: H::Workers { limit: 7 }, figures: None }), b"no room: fleet.workers is 7"),
        (Some(HelloRefused { why: H::Position { worker: 4 }, figures: None }), b"worker position 4 is not configured or is held by a live channel"),
        (Some(HelloRefused { why: H::StopBound { grace: skein_lib::Duration::from_millis(99) }, figures: None }), b"its stop bound is not shorter than the engine's grace of 99 ms"),
    ] {
        rows.push(row).expect("one row per refusal with figures");
    }
    for (why, words) in [
        (A::Window { charter: 3, model: 2 }, b"charter 3, model 2: not admitted".as_slice()),
        (A::Output { charter: 3, model: 2 }, b"charter 3, model 2: not admitted"),
        (A::Turns { charter: 3 }, b"charter 3: not admitted"),
        (A::Time { charter: 3 }, b"charter 3: not admitted"),
        (A::Waiting { charter: 3 }, b"charter 3: not admitted"),
        (A::Checks { charter: 3 }, b"charter 3: not admitted"),
        (A::Spend, b"not admitted"),
        (A::Message, b"not admitted"),
        (A::Inbox, b"not admitted"),
        (A::Unacknowledged, b"not admitted"),
    ] {
        rows.push((Some(agent(why, None)), words)).expect("one row per shortfall without figures");
    }
    rows.into_boxed()
}

fn sent(engine: &mut crate::engine::Engine, channel: u64, hello: Option<HelloRefused>) -> Box<[u8]> {
    let jig_protocol_engine_workers::Event::NotAdmitted { channel: actual, text } =
        crate::route::refuse(engine, Token::new(channel), hello)
    else {
        panic!("a refusal sends NotAdmitted")
    };
    assert_eq!(actual, Token::new(channel));
    text
}

#[test]
fn a_check_reserve_refusal_names_the_setting_on_the_wire_and_in_the_line() {
    let mut service = fixtures::started();
    let engine = service.engine.as_mut().expect("engine");
    engine.notes.clear();
    let hello = Some(agent(A::Checks { charter: 2 }, Some(Figures { required: 60_000, offered: 90_000 })));
    let words = b"charter 2: needs checks within 60000 ms; the agent's profile.policy.check_timeout is 90000 ms; raise charter 2's reserve_time (temper help settings) or lower the timeout";
    assert_eq!(sent(engine, 7, hello).as_ref(), words);
    assert!(words.len() <= 256);
    assert_eq!(engine.notes.pop(), Some(crate::Note::HelloRefused { channel: 7, text: words.as_slice().into() }));
}

#[test]
fn each_refusal_says_its_words() {
    let mut service = fixtures::started();
    let engine = service.engine.as_mut().expect("engine");
    for (hello, expected) in cases() {
        engine.notes.clear();
        assert_eq!(sent(engine, 7, hello).as_ref(), expected, "{hello:?}");
    }
}

#[test]
fn no_refusal_passes_the_channels_bound() {
    let mut service = fixtures::started();
    let engine = service.engine.as_mut().expect("engine");
    for (mut hello, _) in cases() {
        if let Some(hello) = &mut hello {
            if let Some(figures) = &mut hello.figures {
                *figures = Figures { offered: u64::MAX, required: u64::MAX };
            }
            match &mut hello.why {
                H::Agent { why } => match why {
                    A::Model { charter, model } | A::Window { charter, model } | A::Output { charter, model } => {
                        *charter = u32::MAX;
                        *model = u32::MAX;
                    }
                    A::Turns { charter } | A::Time { charter } | A::Waiting { charter } | A::Checks { charter } => {
                        *charter = u32::MAX;
                    }
                    A::Spend | A::Message | A::Inbox | A::Unacknowledged => {}
                },
                H::Workers { limit } => *limit = u32::MAX,
                H::Position { worker } => *worker = u32::MAX,
                H::StopBound { grace } => *grace = skein_lib::Duration::from_nanos(u64::MAX),
                H::JobKinds | H::ReservedChannel => {}
            }
        }
        engine.notes.clear();
        let text = sent(engine, u64::MAX, hello);
        assert!(text.len() <= 256, "{hello:?}: {} bytes", text.len());
    }
}

#[test]
fn a_refused_hello_sends_its_words_and_notes_them() {
    let mut service = fixtures::started();
    let engine = service.engine.as_mut().expect("engine");
    engine.notes.clear();
    let hello = Some(agent(A::Turns { charter: 3 }, Some(Figures { required: 64, offered: 32 })));
    let words = b"charter 3: needs 64 turns; the agent's profile.policy.max_turns allows 32";
    assert_eq!(sent(engine, 7, hello).as_ref(), words);
    assert_eq!(engine.notes.pop(), Some(crate::Note::HelloRefused { channel: 7, text: words.as_slice().into() }));
    // The real delivery arm preserves the reason, including the fallback.
    crate::route::request(
        engine,
        temper_engine_domain::Request::Worker(temper_engine_domain::WorkerRequest::Deliver(
            temper_engine_domain::Delivery::Refuse {
                channel: Token::new(8),
                hello: Some(agent(A::Turns { charter: 3 }, Some(Figures { required: 65, offered: 32 }))),
            },
        )),
    );
    assert_eq!(
        engine.notes.pop(),
        Some(crate::Note::HelloRefused {
            channel: 8,
            text: b"charter 3: needs 65 turns; the agent's profile.policy.max_turns allows 32".as_slice().into(),
        })
    );
    crate::route::request(
        engine,
        temper_engine_domain::Request::Worker(temper_engine_domain::WorkerRequest::Deliver(
            temper_engine_domain::Delivery::Refuse { channel: Token::new(9), hello: None },
        )),
    );
    assert_eq!(
        engine.notes.pop(),
        Some(crate::Note::HelloRefused { channel: 9, text: b"not admitted".as_slice().into() })
    );
    assert_eq!(sent(engine, 10, None).as_ref(), b"not admitted");
    assert_eq!(engine.notes.pop(), None);
}

#[test]
fn a_repeat_of_the_same_words_notes_nothing() {
    let mut service = fixtures::started();
    let engine = service.engine.as_mut().expect("engine");
    engine.notes.clear();
    let first = Some(agent(A::Turns { charter: 3 }, Some(Figures { required: 64, offered: 32 })));
    let different = Some(agent(A::Turns { charter: 3 }, Some(Figures { required: 65, offered: 32 })));
    for (channel, hello, noted) in [(7, first, true), (8, first, false), (9, different, true), (10, first, true)] {
        let text = sent(engine, channel, hello);
        let expected = noted.then_some(crate::Note::HelloRefused { channel, text });
        assert_eq!(engine.notes.pop(), expected);
        assert_eq!(engine.notes.pop(), None);
    }
}

#[test]
fn a_full_note_queue_keeps_the_next_refusal_until_drained() {
    use temper_engine_domain::{Delivery, Request, WorkerRequest};
    let mut service = fixtures::started();
    let engine = service.engine.as_mut().expect("engine");
    engine.notes = skein_lib::Queue::with_capacity(1);
    for (channel, hello) in [(7, None), (8, Some(HelloRefused { why: H::JobKinds, figures: None }))] {
        engine
            .requests
            .push(Request::Worker(WorkerRequest::Deliver(Delivery::Refuse { channel: Token::new(channel), hello })));
    }
    crate::engine::release(engine);
    assert_eq!(engine.requests.len(), 1);
    assert_eq!(
        engine.notes.pop(),
        Some(crate::Note::HelloRefused { channel: 7, text: b"not admitted".as_slice().into() })
    );
    assert!(engine.work_pending(engine.env.now));
    crate::engine::release(engine);
    assert_eq!(
        engine.notes.pop(),
        Some(crate::Note::HelloRefused {
            channel: 8,
            text: b"its job kinds are not ascending and distinct".as_slice().into()
        })
    );
}
