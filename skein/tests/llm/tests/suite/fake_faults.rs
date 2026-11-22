//! Account strikes and whole HTTP bodies cut before provider completion (fakes/llm.md, 5).
use skein_fake_llm_domain::api::{self, Cut, Line, Strike};
use skein_lib::{Duration, Time, Wall};
use skein_llm::{Call, Delta, Failure, Provider, client};
use skein_llm_world::{fake::Exchange, limits};

fn input(provider: Provider) -> Call {
    let mut input = match provider {
        Provider::OpenAi => skein_llm_world::public_call(1),
        Provider::Anthropic => skein_llm_world::anthropic_call(1),
    };
    input.prompt.instructions = b"fault-script".as_slice().into();
    input
}
fn text() -> Line {
    Line::Text { text: b"answer".as_slice().into() }
}
fn exchange(provider: Provider, lines: Vec<Line>) -> Exchange {
    Exchange::new(
        input(provider),
        limits(),
        Box::new([api::Script {
            cue: b"fault-script".as_slice().into(),
            turns: Box::new([api::Turn { lines: lines.into(), finish: api::Finish::Stop, tokens: 1 }]),
        }]),
    )
}
fn failure(world: &Exchange) -> (Failure, &skein_llm::Report, client::Evidence) {
    assert!(
        !world.seen.iter().any(|event| matches!(event, client::Event::Completed { .. })),
        "a fault gives no completion"
    );
    world
        .seen
        .iter()
        .find_map(|event| match event {
            client::Event::Failed { failure, report, evidence, .. } => Some((*failure, report, *evidence)),
            _ => None,
        })
        .expect("the client's failure")
}
fn complete(world: &Exchange) {
    assert!(world.seen.iter().any(|event| matches!(event, client::Event::Completed { .. })), "{:?}", world.seen);
    assert!(!world.seen.iter().any(|event| matches!(event, client::Event::Failed { .. })));
}
fn again(world: &mut Exchange, input: Call) {
    world.machine = client::Client::prepare(input, &world.env.limits).expect("fresh client on the drained connection");
    world.seen.clear();
    world.responses.clear();
    world.start();
    world.run();
}
fn head(world: &Exchange) -> &[u8] {
    let end = world.responses.windows(4).position(|bytes| bytes == b"\r\n\r\n").expect("complete response head");
    &world.responses[..end]
}
fn contains(bytes: &[u8], text: &[u8]) -> bool {
    bytes.windows(text.len()).any(|window| window == text)
}

#[test]
fn account_strikes_reach_the_client_and_clearing_replays_the_turn() {
    for provider in [Provider::OpenAi, Provider::Anthropic] {
        for (strike, expected, status, code) in [
            (
                Strike::Spent,
                Failure::Exhausted { retry_after: None },
                if provider == Provider::OpenAi { 429 } else { 402 },
                if provider == Provider::OpenAi { b"insufficient_quota".as_slice() } else { b"billing_error" },
            ),
            (
                Strike::SignedOut,
                Failure::Unauthorized,
                401,
                if provider == Provider::OpenAi { b"token_expired".as_slice() } else { b"authentication_error" },
            ),
            (
                Strike::Unserved { model: input(provider).prompt.model },
                Failure::Invalid,
                404,
                if provider == Provider::OpenAi { b"model_not_found".as_slice() } else { b"not_found_error" },
            ),
        ] {
            let mut world = exchange(provider, vec![text()]);
            world.strike(0, strike.clone()).expect("account room");
            world.start();
            world.run();
            let (class, report, evidence) = failure(&world);
            assert_eq!(class, expected, "{provider:?} {strike:?}");
            assert_eq!(evidence, client::Evidence::Response { status });
            assert_eq!(report.code.as_ref(), code);
            assert_eq!(
                report.field.as_ref(),
                if expected == Failure::Invalid && provider == Provider::OpenAi { b"model".as_slice() } else { b"" }
            );
            assert!(!report.request.is_empty());
            assert!(!contains(head(&world), b"retry-after"));
            world.clear(0, &strike);
            again(&mut world, input(provider));
            complete(&world);
        }
    }
}

#[test]
fn shared_window_headers_are_evidence_but_the_client_reports_zero_delay() {
    for provider in [Provider::OpenAi, Provider::Anthropic] {
        let mut world = exchange(provider, vec![text()]);
        let wall = Wall::from_nanos(1_000_200_000_000);
        world.at(Time::from_nanos(10), wall);
        world.strike(0, Strike::Window { tokens: 1, length: Duration::from_secs(2) }).expect("window room");
        world.start();
        world.run();
        complete(&world);
        let mut next = input(provider);
        next.prompt.messages = Box::new([skein_llm::Message {
            role: skein_llm::Role::User,
            content: Box::new([skein_llm::Block::Text {
                text: b"another conversation".as_slice().into(),
                replay: None,
            }]),
        }]);
        again(&mut world, next);
        let (class, report, evidence) = failure(&world);
        assert_eq!(class, Failure::RateLimited { retry_after: Duration::ZERO });
        assert_eq!(evidence, client::Evidence::Response { status: 429 });
        assert_eq!(
            report.code.as_ref(),
            if provider == Provider::OpenAi { b"rate_limit_exceeded".as_slice() } else { b"rate_limit_error" }
        );
        let head = head(&world);
        assert!(!contains(head, b"retry-after"));
        for header in if provider == Provider::OpenAi {
            [
                b"x-ratelimit-limit-tokens: 1".as_slice(),
                b"x-ratelimit-remaining-tokens: 0",
                b"x-ratelimit-reset-tokens: 2s",
            ]
        } else {
            [
                b"anthropic-ratelimit-tokens-limit: 1".as_slice(),
                b"anthropic-ratelimit-tokens-remaining: 0",
                b"anthropic-ratelimit-tokens-reset: 1970-01-01T00:16:43Z",
            ]
        } {
            assert!(contains(head, header), "{provider:?}: {}", String::from_utf8_lossy(head));
        }
        world.at(Time::from_nanos(2_000_000_010), wall);
        again(&mut world, input(provider));
        complete(&world);
    }
}

#[test]
fn every_cut_delivers_its_deltas_without_completion_then_the_retry_completes() {
    for provider in [Provider::OpenAi, Provider::Anthropic] {
        for place in [Cut::BeforeOutput, Cut::MidText, Cut::MidCall] {
            let mut lines = match place {
                Cut::BeforeOutput => vec![],
                Cut::MidText => vec![Line::Pieces {
                    pieces: Box::new([
                        api::Piece { text: b"ans".as_slice().into(), hold: None },
                        api::Piece { text: b"wer".as_slice().into(), hold: None },
                    ]),
                }],
                Cut::MidCall => {
                    vec![Line::Call { name: b"tool".as_slice().into(), arguments: br#"{"x":123}"#.as_slice().into() }]
                }
            };
            lines.push(Line::Cut { times: 1 });
            lines.push(text());
            let mut world = exchange(provider, lines);
            world.start();
            world.run();
            assert_eq!(failure(&world).0, Failure::Protocol, "{provider:?} {place:?}");
            assert!(world.responses.starts_with(b"HTTP/1.1 200"));
            assert!(world.responses.ends_with(b"0\r\n\r\n"), "the body ends whole, not a transport truncation");
            let mut deltas = Vec::new();
            for event in &world.seen {
                if let client::Event::Delta { delta, .. } = event {
                    match delta {
                        Delta::Text { text, .. } => deltas.extend_from_slice(text),
                        Delta::ToolArguments { delta, .. } => deltas.extend_from_slice(delta),
                        Delta::ToolStart { .. } | Delta::Reasoning { .. } => {}
                    }
                }
            }
            assert_eq!(
                deltas.as_slice(),
                match place {
                    Cut::BeforeOutput => b"".as_slice(),
                    Cut::MidText => b"answer",
                    Cut::MidCall => br#"{"x""#,
                },
                "{provider:?} {place:?}"
            );
            again(&mut world, input(provider));
            complete(&world);
        }
    }
}

#[test]
fn a_struck_credential_and_model_leave_the_other_calls_alone() {
    for provider in [Provider::OpenAi, Provider::Anthropic] {
        let mut world = exchange(provider, vec![text()]);
        let other = || {
            if provider == Provider::OpenAi {
                skein_llm::Credential::bearer(b"other".as_slice().into())
            } else {
                skein_llm::Credential::key(b"other".as_slice().into())
            }
        };
        assert_eq!(world.accept(other()), Ok(1));
        world.strike(0, Strike::Spent).expect("account room");
        world.machine =
            client::Client::prepare(Call { credential: Some(other()), ..input(provider) }, &world.env.limits)
                .expect("other credential");
        world.start();
        world.run();
        complete(&world);
        world.clear(0, &Strike::Spent);
        world.strike(0, Strike::Unserved { model: b"not-this-model".as_slice().into() }).expect("model room");
        again(&mut world, input(provider));
        complete(&world);
    }
}
