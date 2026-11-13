use super::fixtures;
use jig_domain_core as core;
use jig_domain_host as host;
use skein_lib::{Token, Writer};
use temper_engine_domain as root;
fn call(activation: u64, tool: &[u8], input: &[u8]) -> jig_domain_fleet::Call {
    let name = smith_channel::CallName::new(
        &smith_channel::CEILINGS,
        smith_channel::CallNameParts { activation, completion: 2, position: 1 },
    )
    .unwrap();
    let mut writer = Writer::new(16);
    name.encode(&mut writer).unwrap();
    jig_domain_fleet::Call {
        contract: 1,
        name: writer.finish(),
        tool: tool.into(),
        input: input.into(),
        writes: false,
        deadline: skein_lib::Duration::from_secs(1),
    }
}

#[test]
fn a_workers_call_carries_its_contract_to_application_decoding() {
    let (mut service, run) = fixtures::claimed_chat();
    let engine = service.engine.as_mut().expect("engine");
    let call = call(run.attempt, b"release", br#"{"target":5,"revision":1}"#);
    engine.step(root::Event::Worker(root::WorkerInput::Link {
        channel: Token::new(7),
        link: root::Link::Up {
            parts: root::Parts::none(),
            up: host::LinkUp::Call {
                contract: 3,
                run: Token::new(run.task),
                attempt: Token::new(run.attempt),
                call: call.name,
                tool: call.tool,
                input: call.input,
                writes: true,
                deadline: call.deadline,
            },
        },
    }));
    let mut decoded = false;
    for _ in 0_u32..256 {
        root::release(&mut engine.domain, &engine.env, &mut engine.requests);
        if engine.requests.is_empty() {
            engine.step(root::Event::Resume);
            root::release(&mut engine.domain, &engine.env, &mut engine.requests);
        }
        while let Some(request) = engine.requests.pop() {
            if let root::Request::Worker(root::WorkerRequest::Host(host)) = &request
                && let root::HostRequest::Decode { call, .. } = host.as_ref()
            {
                assert_eq!(call.contract, 3, "hub-link, fleet and core keep the charter's version");
                assert!(!decoded, "one decode hand-out");
                decoded = true;
            }
            crate::route::request(engine, request);
        }
        if decoded {
            break;
        }
    }
    assert!(decoded, "the live worker call reaches decode_configured");
    let event = engine.events.pop().expect("decoder's answer");
    let root::Event::Worker(root::WorkerInput::RenderedHostCall { answer, .. }) = event else {
        panic!("version error: {event:?}")
    };
    assert_eq!(
        answer,
        core::SettledAnswer::Host {
            error: true,
            body: b"this run's host tools are version 3; this engine answers 1..1".as_slice().into(),
        }
    );
}

fn answered_at_decode(
    service: &mut crate::Service,
    run: &root::Assignment,
    call: jig_domain_fleet::Call,
) -> Option<host::Relayed> {
    service.engine.as_mut().expect("engine").step(root::Event::Worker(root::WorkerInput::Link {
        channel: Token::new(7),
        link: root::Link::Up {
            parts: root::Parts::none(),
            up: host::LinkUp::Call {
                contract: call.contract,
                run: Token::new(run.task),
                attempt: Token::new(run.attempt),
                call: call.name.clone(),
                tool: call.tool,
                input: call.input,
                writes: false,
                deadline: call.deadline,
            },
        },
    }));
    let mut delivered = skein_lib::Queue::with_capacity(32);
    fixtures::settle_root(service, &mut delivered);
    let mut answer = None;
    for _ in 0..delivered.len() {
        let root::Delivery::Link { down: host::LinkDown::Relayed { call: name, answer: relayed, .. }, .. } =
            delivered.pop().unwrap()
        else {
            continue;
        };
        assert_eq!(name, call.name);
        assert!(answer.replace(relayed).is_none(), "one answer");
    }
    answer
}

#[test]
fn a_mismatched_activation_is_malformed_without_a_commit() {
    let (mut service, run) = fixtures::claimed_chat();
    let commits = service.engine.as_ref().expect("engine").domain.deployment().commits;
    let call = call(run.attempt + 1, b"release", br#"{"target":5,"revision":1}"#);
    let answer = answered_at_decode(&mut service, &run, call);
    assert_eq!(answer, Some(host::Relayed::Answer { error: true, text: b"malformed host call".as_slice().into() }));
    assert_eq!(service.engine.as_ref().expect("engine").domain.deployment().commits, commits);
    assert_eq!(service.end(), None);
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the test formats expected contract-version diagnostics")]
fn unsupported_host_contracts_are_relayed_as_errors_without_a_commit() {
    for version in [0, temper_engine_smith::wire::VERSION.checked_add(1).unwrap()] {
        let (mut service, run) = fixtures::claimed_chat();
        let commits = service.engine.as_ref().expect("engine").domain.deployment().commits;
        let mut call = call(run.attempt, b"read_docs", br#"{"search":"WireGuard"}"#);
        call.contract = version;
        let answer = answered_at_decode(&mut service, &run, call);
        let expected = format!(
            "this run's host tools are version {version}; this engine answers {}..{}",
            temper_engine_smith::wire::FLOOR,
            temper_engine_smith::wire::VERSION
        );
        assert_eq!(answer, Some(host::Relayed::Answer { error: true, text: expected.as_bytes().into() }));
        assert_eq!(service.engine.as_ref().expect("engine").domain.deployment().commits, commits);
        assert_eq!(service.end(), None);
    }
}

#[test]
fn an_unknown_tool_is_answered_an_error_with_no_commit() {
    let (mut service, run) = fixtures::claimed_chat();
    let commits = service.engine.as_mut().expect("engine").domain.deployment().commits;
    let call = call(run.attempt, b"no_such_tool", b"{}");
    let answer = answered_at_decode(&mut service, &run, call);
    assert_eq!(answer, Some(host::Relayed::Answer { error: true, text: b"unknown tool".as_slice().into() }));
    assert_eq!(service.engine.as_ref().expect("engine").domain.deployment().commits, commits);
    assert_eq!(service.end(), None);
}

#[test]
fn a_documentation_read_is_answered_at_decode_with_no_commit() {
    let (mut service, run) = fixtures::claimed_chat();
    let commits = service.engine.as_mut().expect("engine").domain.deployment().commits;
    let call = call(run.attempt, b"read_docs", br#"{"search":"WireGuard"}"#);
    let answer = answered_at_decode(&mut service, &run, call);
    let Some(host::Relayed::Answer { error, text }) = answer else { panic!("documentation answer") };
    assert!(!error);
    assert!(text.starts_with(b"server@"));
    assert_eq!(service.engine.as_ref().expect("engine").domain.deployment().commits, commits);
    assert_eq!(service.end(), None);
}

#[test]
#[expect(clippy::wildcard_enum_match_arm, reason = "unexpected test outputs fail immediately")]
fn owned_capacity_refusal_preserves_the_opaque_address_without_a_tool_effect() {
    let (mut service, run) = fixtures::claimed_chat();
    let engine = service.engine.as_mut().expect("engine");
    let commits = engine.domain.deployment().commits;
    let mut decoded = 0_u32;
    let mut busy = 0_u32;
    for (channel, name) in [(7, b"first".as_slice()), (7, b"second"), (8, b"foreign"), (7, b"opaque capacity name")] {
        engine.step(root::Event::Worker(root::WorkerInput::Link {
            channel: Token::new(channel),
            link: root::Link::Up {
                parts: root::Parts::none(),
                up: host::LinkUp::Call {
                    contract: 1,
                    run: Token::new(run.task),
                    attempt: Token::new(run.attempt),
                    call: name.into(),
                    tool: b"no_such_tool".as_slice().into(),
                    input: b"{}".as_slice().into(),
                    writes: true,
                    deadline: skein_lib::Duration::from_secs(1),
                },
            },
        }));
        assert!(engine.pending.is_none(), "inputs enter only after the preceding route settles");
        let mut observed = false;
        for _ in 0_u32..256 {
            root::release(&mut engine.domain, &engine.env, &mut engine.requests);
            if engine.requests.is_empty() {
                engine.step(root::Event::Resume);
                root::release(&mut engine.domain, &engine.env, &mut engine.requests);
            }
            while let Some(request) = engine.requests.pop() {
                if let root::Request::Worker(worker) = request {
                    observed = true;
                    match worker {
                        root::WorkerRequest::Host(host) => {
                            if let root::HostRequest::Decode { .. } = *host {
                                decoded += 1;
                            }
                        }
                        root::WorkerRequest::Deliver(root::Delivery::Link {
                            channel,
                            down: host::LinkDown::Relayed { run: task, attempt, call, answer },
                        }) => {
                            assert_eq!(
                                (channel, task, attempt),
                                (Token::new(7), Token::new(run.task), Token::new(run.attempt))
                            );
                            assert_eq!(call.as_ref(), b"opaque capacity name");
                            assert_eq!(answer, host::Relayed::Busy);
                            busy += 1;
                        }
                        _ => panic!("unexpected worker output: {worker:?}"),
                    }
                }
            }
            if observed {
                break;
            }
        }
        assert!(observed, "every input produces its own bounded route output");
    }
    assert_eq!((decoded, busy), (2, 1), "foreign calls stay discarded; capacity cannot decode a tool");
    assert_eq!(engine.domain.deployment().commits, commits, "no effect or commit from refusal");
}

