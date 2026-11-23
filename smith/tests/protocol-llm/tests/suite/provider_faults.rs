//! Open provider failures at the real client/session boundary.
//! Contract: temper/docs/design/draft/failure-worlds.md, sections 3.3 and 7.
use skein_fake_llm_domain::api::{Cut, Line, Strike};
use skein_lib::Duration;
use smith_domain::{llm, run, tools};
use smith_protocol_llm_world::{
    Boundary, World,
    fixture::{self, fault_finish as finish},
};

const ROUNDS: u32 = 100_000;
const READING: &[u8] = b"reading the complete reply";

fn world(configuration: fixture::Configuration, turns: Box<[skein_fake_llm_domain::api::Turn]>) -> World {
    fixture::provider_fault_world(127, configuration, turns, true, Box::new([]))
}

#[test]
fn a_key_sign_out_ends_without_retry() {
    let mut world = world(fixture::Dialect::Anthropic.configuration(), Box::new([finish()]));
    world.record_fault_runs("run 1", "a");
    world.strike(0, Strike::SignedOut).expect("account fits");
    world.run(ROUNDS);
    world.judge_faults().expect("a key cannot refresh");
    assert_eq!(world.protocol_calls().len(), 1);
    assert_eq!(world.unauthorized().len(), 1);
    assert_eq!(smith_faults::ended(world.answer()), smith_faults::Ended::Unauthorized);
}

#[test]
fn a_bearer_sign_out_retries_after_the_host_renews() {
    let mut world = world(fixture::Dialect::OpenAi.configuration(), Box::new([finish()]));
    world.record_fault_runs("run 1", "a");
    world.strike(0, Strike::SignedOut).expect("account fits");
    for _ in 0..ROUNDS {
        if !world.unauthorized().is_empty() {
            break;
        }
        assert!(!world.drive(1), "the bearer awaits renewal");
    }
    assert_eq!(world.unauthorized().len(), 1);
    world.renew_sign_in(2);
    world.run(ROUNDS);
    world.judge_faults().expect("one retry on the new bearer");
    assert_eq!(world.protocol_calls().len(), 2);
    assert!(matches!(world.answer(), run::Answer::Accepted { .. }));
    assert!(world.fault_ledger().entries()[0].until.is_some(), "the sign-out ends with the renewal");
}

#[test]
#[ignore = "provider failures: read the provider reset headers before retrying a rate window"]
fn a_rate_window_is_waited_out() {
    let mut findings = Vec::new();
    for configuration in fixture::configurations() {
        let provider = configuration.endpoint.provider;
        let mut world = world(
            configuration,
            Box::new([fixture::says(Box::new([Line::Text { text: b"first call".as_slice().into() }]), 3), finish()]),
        );
        let length = Duration::from_secs(10);
        let reset = world.now().saturating_add(length);
        world.strike(0, Strike::Window { tokens: 1, length }).expect("one account window");
        world.run(ROUNDS);
        let calls = world.protocol_calls();
        assert!(calls.len() >= 2, "the first reply must provoke a later call: {:?}", world.answer());
        assert!(matches!(calls[1].failure, Some(llm::Failure::RateLimited { .. })), "the second call meets the window");
        let waited = calls.get(2).and_then(|call| call.received_at).is_some_and(|at| at >= reset);
        if !waited || !matches!(world.answer(), run::Answer::Accepted { .. }) {
            findings.push(format!(
                "{provider:?}: reset={reset:?}, peer arrivals={:?}, answer={:?}",
                calls.iter().map(|call| call.received_at).collect::<Vec<_>>(),
                world.answer(),
            ));
        }
    }
    assert!(findings.is_empty(), "{}", findings.join("\n"));
}

fn cut_turn(place: Cut) -> skein_fake_llm_domain::api::Turn {
    let mut lines = vec![
        Line::Text { text: READING.into() },
        Line::Call { name: b"read".as_slice().into(), arguments: br#"{"path":"src/lib.rs"}"#.as_slice().into() },
    ];
    let at = match place {
        Cut::BeforeOutput => 0,
        Cut::MidText => 1,
        Cut::MidCall => 2,
    };
    lines.insert(at, Line::Cut { times: 1 });
    fixture::calls(lines.into(), 11)
}

#[test]
#[ignore = "provider failures: retry incomplete streams with backoff without committing their partial output"]
fn a_cut_stream_is_retried_not_failed() {
    let mut findings = Vec::new();
    for place in [Cut::BeforeOutput, Cut::MidText, Cut::MidCall] {
        for configuration in fixture::configurations() {
            let provider = configuration.endpoint.provider;
            let mut world = world(configuration, Box::new([cut_turn(place), finish()]));
            world.run(ROUNDS);
            let calls = world.protocol_calls();
            let retried = calls.first().zip(calls.get(1)).is_some_and(|(first, second)| {
                first.terminal.is_some_and(|(at, _, _)| second.received_at.is_some_and(|next| next > at))
            });
            if !retried || !matches!(world.answer(), run::Answer::Accepted { .. }) {
                findings.push(format!(
                    "{provider:?} {place:?}: attempts={}, failures={:?}, answer={:?}",
                    calls.len(),
                    calls.iter().map(|call| call.failure).collect::<Vec<_>>(),
                    world.answer(),
                ));
                continue;
            }
            assert_eq!(world.prompts()[0].messages, world.prompts()[1].messages, "no partial reply on retry");
            assert_eq!(world.turns().len(), 2, "only whole replies commit turns");
            let texts: Vec<_> = world
                .turns()
                .iter()
                .flat_map(|turn| &turn.messages)
                .filter(|message| message.role == smith_domain::session::llm::Role::Assistant)
                .flat_map(|message| &message.content)
                .filter_map(|block| match block {
                    smith_domain::session::llm::Block::Text { text, .. } => Some(text.as_ref()),
                    _ => None,
                })
                .collect();
            assert_eq!(texts, [READING], "only the complete reply's text enters history");
            assert_eq!(
                world
                    .boundaries()
                    .iter()
                    .filter(|(_, boundary)| matches!(boundary, Boundary::Io { op: tools::Op::Load { .. } }))
                    .count(),
                1,
                "only the completed read call runs",
            );
            let first_terminal = calls[0].terminal.expect("cut terminal").0;
            assert!(
                world
                    .boundaries()
                    .iter()
                    .filter(|(_, boundary)| matches!(boundary, Boundary::Io { .. }))
                    .all(|(at, _)| *at > first_terminal)
            );
            let run::Answer::Accepted { outcome: run::outcome::Declared::Report(report), .. } = world.answer() else {
                panic!("report accepted")
            };
            assert_eq!(report.text.as_ref(), b"done", "the answer contains no cut text");
        }
    }
    assert!(findings.is_empty(), "{}", findings.join("\n"));
}
