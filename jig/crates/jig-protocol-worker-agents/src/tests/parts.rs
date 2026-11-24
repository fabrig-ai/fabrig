use crate::route;
use jig_domain_host as hub;
use jig_domain_link::{KeptPart, PartKind};
use jig_protocol_charter as charter;
use skein_lib::{Duration, List, Token};
use smith_domain_session::{llm, record};
use smith_host_domain as smith;
use smith_protocol_channel::{Endpoint, Endpoints};

fn endpoints() -> Endpoints {
    let mut entries = List::with_capacity(1);
    entries.push(Endpoint { name: b"primary".as_slice().into(), number: 1, dialect: 3 }).unwrap();
    Endpoints::new(entries)
}

fn charter() -> Box<[u8]> {
    let value = charter::Charter {
        mode: smith_domain_run::Mode::Finite,
        instructions: b"help".as_slice().into(),
        brief: Box::new([]),
        tools: Box::new([]),
        wait: true,
        agents: true,
        workspace: charter::WorkspaceTools { inspect: false, modify: false, shell: false },
        delivers: false,
        conventions: None,
        words: 512,
        contract: charter::Contract {
            report: Some(charter::TextRule { max: 512, fields: Box::new([]) }),
            failure: None,
            verdicts: Box::new([]),
            change: None,
        },
        budget: charter::Budget {
            turns: 3,
            spend: 9000,
            time: Duration::from_secs(60),
            reserve: charter::Reserve { turns: 1, time: Duration::from_secs(10) },
        },
        model: charter::Llm {
            model: charter::Model {
                prices: charter::Prices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1000 },
                dialect: 3,
                endpoint: 1,
                name: b"small".as_slice().into(),
                window: 8192,
                output: 1024,
                effort: None,
            },
            grant: Some(5),
        },
        models: Box::new([]),
        waiting: Duration::ZERO,
        resumes: true,
    };
    let value = charter::charter(value).unwrap();
    charter::encode(
        value,
        &[charter::EndpointName { number: 1, dialect: 3, name: b"primary".as_slice().into() }],
        &smith_charter::CEILINGS,
    )
    .unwrap()
}

fn body() -> smith::TurnValue {
    smith::TurnValue::new(
        record::Turn {
            model: b"small".as_slice().into(),
            effort: None,
            version: record::VERSION,
            endpoint: Some(llm::Endpoint(1)),
            dialect: 3,
            sequence: 1,
            kind: record::Kind::First(Box::new(record::Opener {
                system: None,
                affinity: llm::Affinity { key: [3; 16], thread: 2 },
                account: Some(5),
                backend: record::Backend::Direct,
            })),
            usage: llm::Usage {
                input_tokens: Some(1),
                output_tokens: Some(1),
                cache_read_tokens: Some(0),
                cache_write_tokens: Some(0),
                reasoning_tokens: Some(0),
            },
            spent: 2,
            jobs: 0,
            messages: Box::new([]),
        },
        u64::MAX,
    )
    .unwrap()
}

#[test]
fn first_turn_result_parts_encode_without_a_conversation_opening() {
    let limits = smith_host_protocol_fixtures::small();
    let endpoints = endpoints();
    let mut result = body().into_value();
    result.kind = record::Kind::Ordinary;
    result.messages = Box::new([llm::Message {
        role: llm::Role::User,
        content: Box::new([llm::Block::ToolResult {
            id: b"call-1".as_slice().into(),
            result: llm::Returned::Text { text: b"done".as_slice().into(), error: false, replay: None },
        }]),
    }]);
    let encoded = route::encode_part(
        smith::channel::ToldPart {
            number: 2,
            turn: 1,
            thread: 0,
            call: 1,
            kind: record::PartKind::Result,
            body: smith::TurnValue::new(result.clone(), u64::MAX).unwrap(),
        },
        &limits,
        &endpoints,
    )
    .expect("a first-turn result is a part, not a whole turn");
    assert_eq!(
        (encoded.number, encoded.turn, encoded.thread, encoded.call, encoded.kind),
        (2, 1, 0, 1, PartKind::Result)
    );
    assert_eq!(smith_protocol_translate::decode_part(&encoded.body, &limits.transcript, &endpoints).unwrap(), result);
}

#[test]
fn starts_translate_every_kept_part_kind_and_subagent_attempt() {
    let limits = smith_host_protocol_fixtures::small();
    let endpoints = endpoints();
    let host_limits = smith_agent_profile::host_limits(
        &smith_agent_profile::profile::standard(),
        &smith_agent_profile_fixtures::small(),
        &[],
    )
    .unwrap();
    let body = body();
    let bytes = smith_protocol_channel::encode_turn(body.value(), &limits.transcript, &endpoints).unwrap();
    for tell_parts in [true, false] {
        let mut kept = List::with_capacity(3);
        let mut expected = List::with_capacity(3);
        for (kind, translated, thread, call, attempt) in [
            (PartKind::Answer, record::PartKind::Answer, 0, 0, 7),
            (PartKind::Result, record::PartKind::Result, 0, 2, 8),
            (PartKind::Window, record::PartKind::Window, 3, 9, 9),
        ] {
            kept.push(KeptPart { attempt, turn: 5, thread, call, kind, body: bytes.clone() }).unwrap();
            if tell_parts {
                expected
                    .push(smith::channel::KeptPart {
                        activation: attempt,
                        turn: 5,
                        thread,
                        call,
                        kind: translated,
                        body: body.clone(),
                    })
                    .unwrap();
            }
        }
        let start = hub::Start {
            tell_parts,
            parts: kept.into_boxed(),
            live: hub::LiveAsk::NONE,
            workspace: None,
            charter: charter(),
            activation: 10,
            identity: [3; 16],
            turns: Box::new([]),
            answered: Box::new([]),
            messages: Box::new([]),
            grants: Box::new([]),
        };
        let decoded =
            route::decode(Token::new(17), start, &limits, &host_limits, &endpoints, route::Evidence::Text).unwrap();
        assert_eq!(decoded.tell_parts, tell_parts);
        assert_eq!(decoded.parts.as_ref(), expected.as_slice());
    }
}
