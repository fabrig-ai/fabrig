use super::*;
use jig_client::contract as c;
use skein_lib::{List, Token};

const CLIENT: ClientId = ClientId(0);
const ATTEMPT: Token = Token::new(91);

fn text(value: &[u8]) -> c::Text {
    c::Text::new(value.into(), crate::fixtures::domain_limits().client.domain.text).expect("fixture name")
}
fn amendment() -> c::Amendment {
    c::Amendment {
        addressed: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        instructions: None,
        tracked: None,
        llm: None,
    }
}
fn choice(model: &[u8]) -> c::LlmChoice {
    c::LlmChoice {
        account: Some(17),
        main: c::ModelEffort { model: text(model), effort: None },
        subagents: List::with_capacity(0),
    }
}
fn request(change: c::Amendment) -> Request {
    Request::Client {
        request: jig_client::Request::Down {
            down: Down::Send {
                attempt: ATTEMPT,
                key: [31; 16],
                request: Ask::Jig(Box::new(c::Ask::Amend {
                    project: 1,
                    task: 7,
                    revision: Some(c::Revision { value: 3 }),
                    change: Box::new(change),
                    reason: text(b"retarget"),
                })),
                value: None,
            },
        },
    }
}
fn transport() -> Transport {
    let mut transport = Transport::new(c::VersionRange {
        first: jig_client_protocol_wire::VERSION,
        last: jig_client_protocol_wire::VERSION,
    });
    transport.models = temper_engine_domain_world::walking::models();
    transport.bind(
        CLIENT,
        crate::fixtures::PARTY,
        people::Identity {
            key: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
            login: b"person".as_slice().into(),
            name: b"Person".as_slice().into(),
        },
    );
    transport.signed_in(CLIENT, crate::fixtures::PARTY, 23);
    transport
}
#[test]
fn chosen_model_reaches_the_engine() {
    let mut transport = transport();
    let mut change = amendment();
    change.llm = Some(choice(b"scripted"));
    let (events, _) =
        super::request(&mut transport, CLIENT, &mut request(change), &crate::fixtures::domain_limits().client.domain);
    let [engine::Event::Party(PartyInput::Ask { ask: people::Ask::Amend { amendment, .. }, .. })] = events.as_slice()
    else {
        panic!("one amended task input")
    };
    assert_eq!(
        amendment.llm,
        Some(people::LlmChoice {
            account: Some(17),
            main: people::ModelEffort { model: 0, effort: None },
            subagents: Box::new([]),
        })
    );
}
#[test]
fn authority_model_names_reach_the_engine() {
    let mut transport = transport();
    let mut change = amendment();
    let mut names = List::with_capacity(1);
    names.push(Box::<[u8]>::from(b"scripted".as_slice())).expect("one selector");
    change.authority = Some(c::OwnerAuthority {
        models: Some(c::AuthorityModels::Named(names)),
        tools: 0,
        grants: List::with_capacity(0),
        delegation: c::OwnerDelegation { kinds: List::with_capacity(0), tasks: 0, depth: 0 },
        spend: 0,
        deadline: None,
        notes: 0,
        note_resources: List::with_capacity(0),
    });
    let (events, _) =
        super::request(&mut transport, CLIENT, &mut request(change), &crate::fixtures::domain_limits().client.domain);
    let [engine::Event::Party(PartyInput::Ask { models, .. })] = events.as_slice() else {
        panic!("one named authority input")
    };
    assert_eq!(models.as_ref(), &[Box::<[u8]>::from(b"scripted".as_slice())]);
}
#[test]
fn unconfigured_model_uses_its_original_keyed_route() {
    let mut transport = transport();
    let mut change = amendment();
    change.llm = Some(choice(b"missing"));
    let (events, refusal) =
        super::request(&mut transport, CLIENT, &mut request(change), &crate::fixtures::domain_limits().client.domain);
    let Some(packets::Incoming::Callback {
        client,
        event:
            jig_client_test_domain::Event::Client {
                event: jig_client::Event::Up { up: jig_client::Up::Answer { attempt, answer, .. } },
            },
    }) = refusal
    else {
        panic!("immediate keyed refusal")
    };
    assert_eq!(
        (events.len(), client, attempt, answer),
        (
            0,
            CLIENT,
            ATTEMPT,
            Some(jig_client::Answered::Jig(c::Answer::Refused {
                refusal: c::Refusal::Llm { model: text(b"missing"), why: c::LlmWhy::Unconfigured },
                at: None,
            }))
        )
    );
}
