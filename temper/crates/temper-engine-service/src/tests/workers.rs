use super::fixtures;
use jig_domain_core as core;
use jig_domain_host as host;
use jig_domain_link::{Form, Grant, JobKind, SourceName};
use jig_domain_tasks as tasks;
use jig_protocol_engine_workers::GrantValue;
use skein_lib::Time;
use skein_lib::Token;
use temper_engine_domain as root;
#[test]
fn each_repositorys_grant_carries_its_identitys_value() {
    let values = fixtures::values_with_identity(1, b"forge-token-1234");
    let filled = values.grants(&[fixtures::application_grant(1, 1)], Time::ZERO);
    let [Some(GrantValue::Application { secret })] = filled.as_ref() else { panic!("one application value") };
    let identity = fixtures::decoded_identity(secret);
    let temper_protocol_worker_wire::v1::Kind::Forgejo(token) = identity.kind() else { panic!("Forgejo identity") };
    assert_eq!(token.token().expose(), b"forge-token-1234");
    assert_eq!(identity.login(), b"temper");
    assert_eq!(identity.name(), b"temper");
    assert_eq!(identity.email(), b"temper@forge.example");
}
#[test]
fn a_missing_identity_or_generation_keeps_its_grant_slot_empty() {
    let values = fixtures::values_with_identity(1, b"forge-token-1234");
    let grants =
        [fixtures::application_grant(1, 2), fixtures::application_grant(2, 1), fixtures::application_grant(1, 1)];
    let filled = values.grants(&grants, Time::ZERO);
    assert_eq!(filled.len(), 3);
    for (index, present) in [(0, false), (1, false), (2, true)] {
        assert_eq!(filled[index].is_some(), present, "grant {index}");
    }
}
#[test]
fn application_grants_answer_either_retained_generation() {
    let mut values = fixtures::values_with_identity(1, b"first");
    let next = fixtures::values_with_identity(1, b"second");
    let (_, value) = next.identities.newest(1).unwrap();
    assert_eq!(values.identities.put(1, value.clone()), 2);
    let filled = values.grants(
        &[
            fixtures::application_grant(1, 1),
            fixtures::application_grant(1, 2),
            fixtures::application_grant(1, 0),
            fixtures::application_grant(1, 3),
        ],
        Time::ZERO,
    );
    for (index, token) in [(0, b"first".as_slice()), (1, b"second".as_slice())] {
        let Some(GrantValue::Application { secret }) = &filled[index] else { panic!("retained generation") };
        let identity = fixtures::decoded_identity(secret);
        let temper_protocol_worker_wire::v1::Kind::Forgejo(value) = identity.kind() else { panic!("Forgejo") };
        assert_eq!(value.token().expose(), token);
    }
    assert!(filled[2].is_none());
    assert!(filled[3].is_none());
}
#[test]
fn identities_are_announced_after_the_durable_start_wires() {
    let mut service = fixtures::started();
    service.engine.as_mut().expect("engine").store.hold_mark = true;
    assert_eq!(
        service.engine.as_ref().expect("engine").domain.next_deadline(&service.engine.as_ref().expect("engine").env),
        None
    );
    for _ in 0_u32..256 {
        fixtures::turn(&mut service);
    }
    let engine = service.engine.as_mut().expect("engine");
    assert_eq!(engine.domain.start_phase(), Some(core::start::Phase::Wires));
    assert!(engine.values.identities.numbers().is_empty(), "identities wait for the durable wires");
    assert!(engine.workers.forges.is_empty(), "runtime forges wait for the durable wires");
    assert!(!engine.forge_records.pending.is_empty(), "restored forges await the mark");
    let mark = engine.store.held_mark.take().expect("held startup wires");
    assert!(!engine.store.host.mark_durable());
    engine.store.hold_mark = false;
    assert!(engine.store.down(mark));
    assert!(!service.engine.as_ref().expect("engine").forge_records.done(), "secrets follow the mark");
    fixtures::restart(&mut service);
    let engine = service.engine.as_ref().expect("engine");
    assert_eq!(engine.values.identities.numbers().as_slice(), &[1]);
    assert!(engine.forge_records.pending.is_empty());
    assert!(engine.domain.ready());
    assert!(service.engine.as_ref().expect("engine").store.host.mark_durable());
}

#[test]
fn an_accounts_grant_uses_the_lent_generation_until_it_expires() {
    let mut values = fixtures::values_with_identity(1, b"forge-token-1234");
    values
        .accounts
        .granted(
            1,
            2,
            jig_protocol_accounts::table::Value {
                token: skein_lib::Secret::new(b"account-token".as_slice().into()),
                scheme: jig_protocol_accounts::table::Scheme::Bearer,
            },
            Time::from_nanos(9),
        )
        .unwrap();
    let mut grant = fixtures::application_grant(1, 2);
    grant.form = Form::Credential(jig_domain_link::Scheme::Bearer);
    for (now, generation, expected) in [(8, 2, true), (9, 2, false), (8, 1, false)] {
        grant.generation = generation;
        let filled = values.grants(&[grant], Time::from_nanos(now));
        assert_eq!(filled[0].is_some(), expected, "{now} {generation}");
        if let Some(value) = &filled[0] {
            let GrantValue::Credential { scheme: jig_protocol_engine_workers::Scheme::Bearer, token } = value else {
                panic!("a bearer grant")
            };
            assert_eq!(token.expose(), b"account-token", "{now} {generation}");
        }
    }
}

fn assignment_with_identity() -> temper_engine_domain::Assignment {
    let mut assignment = fixtures::assignment_with(fixtures::repository(7, b"acme", b"shop"));
    assignment.grants = Box::new([Grant {
        form: Form::Application,
        account: 1,
        generation: 1,
        valid: skein_lib::Duration::from_secs(30),
    }]);
    assignment.transcript = Box::new([b"turn".as_slice().into()]);
    assignment
}
#[test]
fn an_assignment_carries_its_items_encoded_charter_and_identity_value() {
    let settings = fixtures::engine();
    let encoding = fixtures::encoding(&settings);
    let assignment = assignment_with_identity();
    let values = fixtures::values_with_identity(1, b"forge-token-1234");
    let event = crate::assignment::down(
        assignment,
        &fixtures::forge_table(&fixtures::forges()),
        &encoding,
        &values,
        Token::new(8),
        Time::ZERO,
    )
    .unwrap();
    let jig_protocol_engine_workers::Event::Down {
        channel,
        down: jig_domain_host::LinkDown::Assign { assignment },
        parts,
        values,
    } = event
    else {
        panic!("assignment event")
    };
    assert_eq!(channel, Token::new(8));
    assert_eq!(assignment.assignment.run, Token::new(7));
    assert_eq!(assignment.assignment.attempt, Token::new(2));
    assert_eq!(
        assignment.assignment.identity,
        [118, 213, 170, 119, 186, 221, 240, 223, 121, 163, 222, 185, 198, 157, 28, 125]
    );
    assert!(assignment.assignment.save);
    let workspace = assignment.assignment.workspace.unwrap();
    assert_eq!(workspace.workstream, 7);
    assert_eq!(workspace.items, parts[0].token);
    let items =
        temper_protocol_worker_wire::v1::Items::decode(&encoding.limits, &mut skein_lib::Reader::new(&parts[0].bytes))
            .unwrap();
    assert_eq!(items.repositories().get(0).unwrap().remote(), b"https://forge.example/acme/shop.git");
    let charter = decoded_charter(&assignment.assignment.charter);
    assert!(charter.resume());
    let Some(GrantValue::Application { secret }) = &values[0] else { panic!("an application value") };
    let value = fixtures::decoded_identity(secret);
    let temper_protocol_worker_wire::v1::Kind::Forgejo(token) = value.kind() else { panic!("Forgejo identity") };
    assert_eq!(token.token().expose(), b"forge-token-1234");
}
#[test]
#[expect(clippy::disallowed_methods, reason = "ordinary tests assemble byte fixtures and inspect text")]
fn a_fresh_start_whose_tail_reaches_the_turns_headers_still_encodes() {
    let settings = fixtures::engine();
    let encoding = fixtures::encoding(&settings);
    let mut assignment = assignment_with_identity();
    assignment.run.policy.resume = false;
    // jig's tail copies the retained turns' encoded bytes, headers among them.
    let mut tail = b"\x00\x00\x01\x2aearlier words of a turn that came before this one, ".to_vec();
    tail.extend_from_slice(&[0xff, 0xfe, 0x00, 0x00, 0x00, 0x05]);
    tail.extend_from_slice(b"The README's last line is in.");
    let mut sections = assignment.sections.into_vec();
    sections.push(temper_engine_domain::BriefSection {
        kind: temper_engine_domain::BriefKind::Core(jig_domain_brief::Core::TranscriptTail),
        body: temper_engine_domain::BriefBody::Text(tail.into()),
    });
    assignment.sections = sections.into_boxed_slice();
    let values = fixtures::values_with_identity(1, b"forge-token-1234");
    let event = crate::assignment::down(
        assignment,
        &fixtures::forge_table(&fixtures::forges()),
        &encoding,
        &values,
        Token::new(8),
        Time::ZERO,
    )
    .unwrap();
    let jig_protocol_engine_workers::Event::Down { down: jig_domain_host::LinkDown::Assign { assignment }, .. } = event
    else {
        panic!("assignment event")
    };
    let charter = &assignment.assignment.charter;
    assert!(!decoded_charter(charter).resume());
    assert!(skein_lib::bytes::find(charter, b"[earlier bytes not text, cut]\nThe README's last line is in.").is_some());
}
#[test]
#[expect(clippy::disallowed_methods, reason = "ordinary tests assemble byte fixtures and inspect text")]
fn a_tail_keeps_its_text_marked_and_never_grows() {
    use crate::assignment::text_tail;
    const MARKER: &[u8] = b"[earlier bytes not text, cut]\n";
    // A text tail, its newlines and tabs among it, is unchanged.
    assert_eq!(text_tail(b"I appended the line.\n\tDone.").as_ref(), b"I appended the line.\n\tDone.");
    // A tail with no text after its last header is the marker alone, never its bytes.
    assert_eq!(text_tail(&[0xff; 40]).as_ref(), MARKER);
    assert_eq!(text_tail(&[0x00; 40]).as_ref(), MARKER);
    // The marker's room is cut from the text's start, at a character's edge.
    let mut long = b"\x00\x05".to_vec();
    long.extend_from_slice("it’s done: the README’s line’s in, all of it".as_bytes());
    let text = text_tail(&long);
    assert!(text.starts_with(MARKER), "{text:?}");
    assert!(::core::str::from_utf8(&text).is_ok(), "{text:?}");
    assert!(text.ends_with(b"all of it"));
    assert!(text.len() <= long.len());
    // A tail too short for the marker keeps its text alone.
    assert_eq!(text_tail(b"\x00\x05done").as_ref(), b"done");
}
#[test]
fn an_assignment_that_cannot_encode_its_items_is_refused() {
    let settings = fixtures::engine();
    let encoding = fixtures::encoding(&settings);
    let assignment = fixtures::assignment_with(fixtures::repository(7, b"acme", b"shop"));
    let values = fixtures::values_with_identity(1, b"forge-token-1234");
    assert!(
        crate::assignment::down(assignment, &fixtures::forge_table(&[]), &encoding, &values, Token::new(8), Time::ZERO)
            .is_none()
    );
}
#[test]
fn a_bad_or_missing_answer_part_discards_all_of_its_parts() {
    use jig_domain_host as host;
    use skein_lib::Token;
    let limits = fixtures::wire_limits();
    let resource = temper_protocol_worker_wire::v1::Preparation::Refused(
        temper_protocol_worker_wire::v1::Tag::new(&limits, temper_protocol_worker_wire::v1::TagParts { tag: 7 })
            .unwrap(),
    );
    let up = fixtures::answer(host::Ending::Failed {
        failure: host::Failure::Unprepared(host::Preparation::Permanent { resource: Some(Token::new(2)) }),
        detail: Box::new([]),
        work: host::Work { left: Some(Token::new(1)), saved: None },
    });
    for bytes in [b"".as_slice(), &[0, 0, 0, 0, 3], &[0, 0, 0, 1]] {
        let parts = [
            jig_protocol_worker_wire::Part { token: Token::new(1), bytes: bytes.into() },
            jig_protocol_worker_wire::Part {
                token: Token::new(2),
                bytes: temper_protocol_worker_wire::preparation(&resource),
            },
        ];
        assert_eq!(crate::workers::parts(&up, &parts, &limits), None, "{bytes:?}");
    }
    assert_eq!(crate::workers::parts(&up, &[], &limits), None, "missing part");
}
#[test]
fn a_named_preparation_part_reaches_the_roots_parts() {
    use jig_domain_host as host;
    use skein_lib::Token;
    let limits = fixtures::wire_limits();
    let resource = temper_protocol_worker_wire::v1::Preparation::Refused(
        temper_protocol_worker_wire::v1::Tag::new(&limits, temper_protocol_worker_wire::v1::TagParts { tag: 7 })
            .unwrap(),
    );
    let up = fixtures::answer(host::Ending::Failed {
        failure: host::Failure::Unprepared(host::Preparation::Permanent { resource: Some(Token::new(2)) }),
        detail: Box::new([]),
        work: host::Work { left: None, saved: None },
    });
    let parts = [jig_protocol_worker_wire::Part {
        token: Token::new(2),
        bytes: temper_protocol_worker_wire::preparation(&resource),
    }];
    assert_eq!(
        crate::workers::parts(&up, &parts, &limits).unwrap().preparation,
        Some(temper_engine_domain::Preparation::Refused { tag: 7 })
    );
}

#[test]
fn an_assignment_preserves_its_ordered_messages_and_answered_calls() {
    let settings = fixtures::engine();
    let encoding = fixtures::encoding(&settings);
    let mut assignment = assignment_with_identity();
    assignment.messages = Box::new([
        word(3, tasks::Party::Person(8), b"first"),
        word(5, tasks::Party::Deployment { project: 1 }, b"second"),
    ]);
    assignment.settled = Box::new([core::Settled {
        key: core::CallKey { task: assignment.task, attempt: 1, completion: 2, position: 1 },
        call: core::SettledCall {
            serial: 6,
            tool: b"note".as_slice().into(),
            answer: core::SettledAnswer::Host { error: false, body: b"done".as_slice().into() },
        },
    }]);
    let values = fixtures::values_with_identity(1, b"forge-token-1234");
    let event = crate::assignment::down(
        assignment,
        &fixtures::forge_table(&fixtures::forges()),
        &encoding,
        &values,
        Token::new(8),
        Time::ZERO,
    )
    .unwrap();
    let jig_protocol_engine_workers::Event::Down { down: host::LinkDown::Assign { assignment }, .. } = event else {
        panic!("assignment")
    };
    assert_eq!(assignment.messages.len(), 2);
    for (index, number, sender, words) in
        [(0, 3, b"8".as_slice(), b"first".as_slice()), (1, 5, b"deployment".as_slice(), b"second".as_slice())]
    {
        assert_eq!(assignment.messages[index].name, Token::new(number), "message {index}");
        assert_eq!(assignment.messages[index].sender.as_ref(), sender, "message {index}");
        assert_eq!(assignment.messages[index].words.as_ref(), words, "message {index}");
    }
    assert_eq!(
        assignment.answered.as_ref(),
        &[host::AnsweredCall {
            name: jig_protocol_charter::call_name(&jig_protocol_charter::CallKey {
                activation: 1,
                completion: 2,
                position: 1,
            }),
            tool: b"note".as_slice().into(),
            answer: host::SettledAnswer::Host { error: false, body: b"done".as_slice().into() }
        }]
    );
    assert_eq!(assignment.turns.as_ref(), &[Box::from(b"turn".as_slice())]);
}

fn word(number: u64, from: jig_domain_tasks::Party, words: &[u8]) -> jig_domain_tasks::Word {
    jig_domain_tasks::Word {
        number,
        from,
        words: words.into(),
        kind: jig_domain_tasks::MessageKind::Words,
        at: skein_lib::Wall::EPOCH,
        hits: 0,
        eligible: true,
    }
}

fn decoded_charter(bytes: &[u8]) -> smith_charter::Charter {
    smith_charter::Charter::decode(&smith_charter::CEILINGS, &mut skein_lib::Reader::new(bytes)).unwrap()
}

#[test]
fn a_part_that_does_not_decode_gives_no_parts_and_is_counted() {
    let mut workers = fixtures::workers();
    let mut events = skein_lib::Queue::uncapped();
    for (bytes, expected) in [(b"".as_slice(), 1), (&[0, 0, 0, 1], 2)] {
        let up = fixtures::answer(host::Ending::Ended {
            outcome: Box::new([]),
            work: host::Work { left: Some(Token::new(1)), saved: None },
        });
        workers.received(
            Token::new(8),
            up,
            &[jig_protocol_worker_wire::Part { token: Token::new(1), bytes: bytes.into() }],
            &mut events,
        );
        let Some(temper_engine_domain::Event::Worker(temper_engine_domain::WorkerInput::Link {
            channel,
            link: temper_engine_domain::Link::Up { parts, up },
        })) = events.pop()
        else {
            panic!("answer reaches root")
        };
        assert_eq!(channel, Token::new(8), "{bytes:?}");
        assert_eq!(parts, temper_engine_domain::Parts::none(), "{bytes:?}");
        assert_eq!(
            up,
            fixtures::answer(host::Ending::Ended {
                outcome: Box::new([]),
                work: host::Work { left: Some(Token::new(1)), saved: None },
            }),
            "{bytes:?}"
        );
        assert_eq!(workers.bad_parts, expected, "{bytes:?}");
    }
}
#[test]
fn the_workers_listener_and_accepted_socket_have_their_own_prefixes() {
    let mut workers = fixtures::workers();
    let owner = Token::new(crate::tokens::WORKER_LISTENERS);
    let mut events = skein_lib::Queue::uncapped();
    let Some(skein_io::Request::Listen { owner: requested, addr: addr @ skein_io::Addr::InProcess(_) }) =
        workers.host.io.pop()
    else {
        panic!("configured in-process listener")
    };
    assert_eq!(requested, owner);
    assert!(workers.up(Time::ZERO, skein_io::Event::Listening { owner, listener: Token::new(3), addr }, &mut events));
    assert!(workers.host.opened());
    assert!(workers.up(
        Time::ZERO,
        skein_io::Event::Accepted { owner, socket: Token::new(4), peer: addr },
        &mut events
    ));
    let Some(skein_io::Request::Bind { socket, owner: connection }) = workers.host.io.pop() else {
        panic!("accepted socket bound")
    };
    assert_eq!(socket, Token::new(4));
    assert_eq!(connection, Token::new(crate::tokens::WORKER_CONNECTIONS + 1));
}
#[test]
fn a_listener_that_opens_during_closing_is_closed_immediately() {
    let mut workers = fixtures::workers();
    let _listen = workers.host.io.pop().unwrap();
    let mut events = skein_lib::Queue::uncapped();
    workers.close(Time::ZERO, &mut events);
    assert!(workers.up(
        Time::ZERO,
        skein_io::Event::Listening {
            owner: Token::new(crate::tokens::WORKER_LISTENERS),
            listener: Token::new(3),
            addr: skein_io::Addr::InProcess(skein_io::Name::new(b"workers").unwrap())
        },
        &mut events
    ));
    let Some(skein_io::Request::Close { entity }) = workers.host.io.pop() else { panic!("late listener close") };
    assert_eq!(entity, Token::new(3));
    assert!(!workers.host.closed());
    assert!(workers.up(
        Time::ZERO,
        skein_io::Event::Closed { owner: Token::new(crate::tokens::WORKER_LISTENERS) },
        &mut events
    ));
    assert!(workers.host.closed());
}
#[test]
fn an_assignment_that_cannot_encode_is_answered_invalid_through_the_link() {
    let mut workers = fixtures::workers();
    let mut events = skein_lib::Queue::uncapped();
    let mut assignment = fixtures::assignment_with(fixtures::repository(7, b"acme", b"shop"));
    assignment.workspace.repositories[0].provider.forge = 99;
    workers.assignment(
        Time::ZERO,
        Token::new(8),
        assignment,
        &fixtures::values_with_identity(1, b"forge-token-1234"),
        &mut events,
    );
    let Some(temper_engine_domain::Event::Worker(temper_engine_domain::WorkerInput::Link {
        channel,
        link: temper_engine_domain::Link::Up { up: host::LinkUp::Answer { run, attempt, answer }, parts },
    })) = events.pop()
    else {
        panic!("invalid assignment answer")
    };
    assert_eq!(channel, Token::new(8));
    assert_eq!((run, attempt), (Token::new(7), Token::new(2)));
    assert_eq!(parts, temper_engine_domain::Parts::none());
    assert_eq!(
        answer.ending,
        host::Ending::Refused(host::Refusal::Invalid(host::Invalid::Start {
            cause: host::StartCause::Other,
            why: b"assignment cannot encode".as_slice().into(),
        }))
    );
}

#[test]
fn the_roots_facts_are_drained_and_counted_after_each_step() {
    use temper_engine_domain as root;
    let (mut service, run) = fixtures::claimed_chat();
    let preparation = root::Preparation::Missing { tag: 3, missing: root::Missing::Branch };
    let up = host::LinkUp::Answer {
        run: Token::new(run.task),
        attempt: Token::new(run.attempt),
        answer: host::RunAnswer {
            read: None,
            refused: Box::new([]),
            turns: 0,
            spent: 1,
            ending: host::Ending::Failed {
                failure: host::Failure::Unprepared(host::Preparation::Permanent { resource: None }),
                detail: Box::new([]),
                work: host::Work { left: None, saved: None },
            },
        },
    };
    service.engine.as_mut().expect("engine").step(root::Event::Worker(root::WorkerInput::Link {
        channel: Token::new(7),
        link: root::Link::Up {
            up,
            parts: root::Parts { saved: None, pushed: Box::new([]), preparation: Some(preparation) },
        },
    }));
    fixtures::settle_root(&mut service, &mut skein_lib::Queue::uncapped());
    assert_eq!(service.engine.as_ref().expect("engine").facts, 1);
    assert_eq!(service.engine.as_mut().expect("engine").domain.pop_fact(), None);
    assert_eq!(service.engine.as_ref().expect("engine").domain.facts_lost(), 0);
    service.engine.as_mut().expect("engine").step(root::Event::Resume);
    assert_eq!(service.engine.as_ref().expect("engine").facts, 1);
}

#[test]
fn a_restored_assignment_uses_the_stored_deployment_identity_not_the_new_seed() {
    use temper_engine_domain as root;
    let previous = fixtures::ready().engine.unwrap();
    let stored_id = previous.domain.deployment().id;
    let mut seeds = fixtures::seeds();
    seeds.deployment = [9; 16];
    assert_ne!(seeds.deployment, stored_id);
    let mut settings = fixtures::engine();
    fixtures::fill_files(&mut settings);
    let mut service = crate::Service::engine(settings, &seeds, None, fixtures::forge_runtime()).unwrap();
    service.engine.as_mut().unwrap().store = previous.store;
    fixtures::restart(&mut service);
    let engine = service.engine.as_mut().unwrap();
    assert_eq!(engine.domain.deployment().id, stored_id);
    assert_eq!(engine.workers.encoding.deployment, seeds.deployment);
    engine.workers.forges = fixtures::forge_table(&fixtures::forges());
    let run = fixtures::assignment_with(fixtures::repository(7, b"acme", b"shop"));
    let expected = jig_protocol_charter::run_identity(&stored_id, run.task, run.conversation);
    crate::route::request(
        engine,
        root::Request::Worker(root::WorkerRequest::Deliver(root::Delivery::Assigned {
            channel: Token::new(8),
            assignment: run,
        })),
    );
    let run = fixtures::assignment_with(fixtures::repository(7, b"acme", b"shop"));
    let jig_protocol_engine_workers::Event::Down { down: host::LinkDown::Assign { assignment }, .. } =
        engine.workers.encoded_assignment(engine.env.now, Token::new(8), run, &engine.values).unwrap()
    else {
        panic!("assignment encodes")
    };
    assert_eq!(assignment.assignment.identity, expected, "the wire identity uses the restored deployment id");
}

#[test]
fn a_worker_assignment_uses_the_services_configured_forges() {
    use temper_engine_domain as root;
    let mut service = fixtures::started();
    let mut assignment = fixtures::assignment_with(fixtures::repository(7, b"acme", b"shop"));
    assignment.workspace.repositories[0].provider.forge = 99;
    crate::route::request(
        service.engine.as_mut().expect("engine"),
        root::Request::Worker(root::WorkerRequest::Deliver(root::Delivery::Assigned {
            channel: Token::new(8),
            assignment,
        })),
    );
    let Some(root::Event::Worker(root::WorkerInput::Link {
        channel,
        link: root::Link::Up { up: host::LinkUp::Answer { run, attempt, answer }, parts },
    })) = service.engine.as_mut().expect("engine").events.pop()
    else {
        panic!("worker assignment refused through its root input")
    };
    assert_eq!(channel, Token::new(8));
    assert_eq!((run, attempt), (Token::new(7), Token::new(2)));
    assert_eq!(
        answer.ending,
        host::Ending::Refused(host::Refusal::Invalid(host::Invalid::Start {
            cause: host::StartCause::Other,
            why: b"assignment cannot encode".as_slice().into(),
        }))
    );
    assert_eq!(parts, root::Parts::none());
    assert_eq!(service.end(), None);
}

#[test]
fn engine_values_preserve_slots_and_refuse_a_different_declared_form() {
    use jig_protocol_accounts::table::{Scheme, Value};
    let forms = [Form::Credential(jig_domain_link::Scheme::Bearer), Form::Credential(jig_domain_link::Scheme::Key)];
    for (scheme, actual) in [(Scheme::Bearer, forms[0]), (Scheme::Key, forms[1])] {
        let mut values = fixtures::values_with_identity(1, b"forge-token");
        values
            .accounts
            .granted(
                2,
                3,
                Value { scheme, token: skein_lib::Secret::new(b"llm-token".as_slice().into()) },
                Time::from_nanos(9),
            )
            .unwrap();
        let grants =
            forms.map(|form| Grant { form, account: 2, generation: 3, valid: skein_lib::Duration::from_secs(5) });
        let filled = values.grants(&grants, Time::from_nanos(8));
        for (grant, value) in grants.iter().zip(filled.iter()) {
            assert_eq!(value.is_some(), grant.form == actual);
        }
        for value in values.grants(&grants, Time::from_nanos(9)) {
            assert!(value.is_none());
        }
    }
}

#[test]
fn a_program_grant_carries_its_name_without_a_credential_table_entry() {
    let values = crate::values::Values::new(0, 0);
    let name = SourceName::new(b"claude").unwrap();
    let grant =
        Grant { form: Form::Program(name), account: 1, generation: 2, valid: skein_lib::Duration::from_secs(60) };
    let filled = values.grants(&[grant], Time::ZERO);
    let [Some(GrantValue::Program { name: held })] = filled.as_ref() else { panic!("one named program value") };
    assert_eq!(*held, name);
}

#[test]
fn a_program_grants_charter_keeps_its_source_account() {
    let settings = fixtures::engine();
    let encoding = fixtures::encoding(&settings);
    let mut assignment = fixtures::assignment_with(fixtures::repository(7, b"acme", b"shop"));
    assignment.run.sources = Box::new([Some(17)]);
    assignment.grants = Box::new([Grant {
        form: Form::Program(SourceName::new(b"alice").unwrap()),
        account: 17,
        generation: 2,
        valid: skein_lib::Duration::from_secs(60),
    }]);
    let empty = crate::values::Values::new(0, 0);
    let jig_protocol_engine_workers::Event::Down { down: host::LinkDown::Assign { assignment: sent }, values, .. } =
        crate::assignment::down(
            assignment,
            &fixtures::forge_table(&fixtures::forges()),
            &encoding,
            &empty,
            Token::new(8),
            Time::ZERO,
        )
        .unwrap()
    else {
        panic!("assignment down")
    };
    let [grant] = sent.assignment.grants.as_ref() else { panic!("one program grant") };
    assert_eq!(grant.form, Form::Program(SourceName::new(b"alice").unwrap()));
    let [Some(GrantValue::Program { name })] = values.as_ref() else { panic!("one name without a table entry") };
    assert_eq!(name.as_bytes(), b"alice");
    let charter = decoded_charter(&sent.assignment.charter);
    assert_eq!(charter.main().account(), &Some(17));
    assert_eq!(charter.main().endpoint(), b"fake");
}

#[test]
fn a_first_attempt_requests_saves_from_its_charters_resume_policy() {
    let settings = fixtures::engine();
    let encoding = fixtures::encoding(&settings);
    let values = fixtures::values_with_identity(1, b"forge-token");
    for resume in [false, true] {
        let mut run = fixtures::assignment_with(fixtures::repository(7, b"acme", b"shop"));
        run.run.policy.resume = resume;
        let jig_protocol_engine_workers::Event::Down { down: host::LinkDown::Assign { assignment }, parts, .. } =
            crate::assignment::down(
                run,
                &fixtures::forge_table(&fixtures::forges()),
                &encoding,
                &values,
                Token::new(8),
                Time::ZERO,
            )
            .unwrap()
        else {
            panic!("assignment")
        };
        let items = temper_protocol_worker_wire::v1::Items::decode(
            &encoding.limits,
            &mut skein_lib::Reader::new(&parts[0].bytes),
        )
        .unwrap();
        assert_eq!(assignment.assignment.save, resume);
        assert_eq!(items.repositories().get(0).unwrap().save().is_some(), resume);
        assert!(!decoded_charter(&assignment.assignment.charter).resume(), "the first attempt has no transcript yet");
    }
}

fn refused_read_call(service: &mut crate::Service, run: &temper_engine_domain::Assignment) {
    use temper_engine_domain as root;
    let name = jig_protocol_charter::call_name(&jig_protocol_charter::CallKey {
        activation: run.attempt,
        completion: 0,
        position: 1,
    });
    service.engine.as_mut().unwrap().step(root::Event::Worker(root::WorkerInput::Link {
        channel: Token::new(7),
        link: root::Link::Up {
            up: host::LinkUp::Call {
                run: Token::new(run.task),
                attempt: Token::new(run.attempt),
                call: name,
                tool: b"read_branch".as_slice().into(),
                writes: false,
                input: br#"{"repository":"acme/shop","branch":"main"}"#.as_slice().into(),
                deadline: skein_lib::Duration::from_secs(10),
            },
            parts: root::Parts::none(),
        },
    }));
}

fn advance_refusal(service: &mut crate::Service) {
    use temper_engine_domain as root;
    let engine = service.engine.as_mut().unwrap();
    for _ in 0_u32..32 {
        if let Some(event) = engine.events.pop() {
            engine.step(event);
        }
        root::release(&mut engine.domain, &engine.env, &mut engine.requests);
        for _ in 0..engine.requests.len() {
            let request = engine.requests.pop().unwrap();
            crate::route::request(engine, request);
        }
        engine.store.iterate();
        crate::engine::answer(engine);
        engine.domain.reclaim();
        for note in &engine.notes {
            if let crate::Note::CallAdmissionRefused { .. } = note {
                return;
            }
        }
    }
}

#[test]
fn call_admission_refusals_reach_operator_notes_through_the_service() {
    let (mut service, run) = fixtures::claimed_chat();
    refused_read_call(&mut service, &run);
    advance_refusal(&mut service);
    let mut found = false;
    while let Some(note) = service.notes().pop() {
        if let crate::Note::CallAdmissionRefused { task, attempt, tool, layer, reason } = note {
            assert_eq!((task, attempt), (run.task, run.attempt));
            assert_eq!(tool.as_ref(), b"read_branch");
            assert_eq!((layer, reason), ("call-proof", "completion is zero"));
            found = true;
        }
    }
    assert!(found, "the shell receives the refusing layer and its reason");
}

#[test]
fn refusal_notes_resume_after_the_operator_sink_drains() {
    let (mut service, run) = fixtures::claimed_chat();
    while service.notes().room() > 0 {
        service.notes().push(crate::Note::ForgeLacking { number: 1, names: Box::new([]) });
    }
    refused_read_call(&mut service, &run);
    advance_refusal(&mut service);
    assert_eq!(service.engine.as_ref().unwrap().facts, 0, "a full note sink leaves the fact queued");
    while service.notes().pop().is_some() {}
    let engine = service.engine.as_mut().unwrap();
    assert!(engine.work_pending(Time::ZERO));
    crate::engine::clock(engine, Time::ZERO, skein_lib::Wall::EPOCH);
    assert_eq!(engine.facts, 1);
    assert_eq!(engine.domain.facts_lost(), 0);
    let Some(crate::Note::CallAdmissionRefused { task, attempt, .. }) = service.notes().pop() else {
        panic!("queued refusal reaches the operator");
    };
    assert_eq!((task, attempt), (run.task, run.attempt));
}

/// temper/docs/design/domain/worker/jobs.md: jobs receive the run path's repository records.
#[test]
fn a_jobs_forge_workspace_becomes_worker_items() {
    let mut service = fixtures::started();
    let engine = service.engine.as_mut().expect("engine");
    engine.workers.forges = fixtures::forge_table(&fixtures::forges());
    engine.workers.encoding = fixtures::encoding(&fixtures::engine());
    let mut repository = fixtures::repository(7, b"acme", b"shop");
    repository.start =
        temper_engine_domain::ForgeStart::Merge { branch: b"temper/change".as_slice().into(), base: [4; 32] };
    repository.push = Some(b"temper/change".as_slice().into());
    let workspace = temper_engine_domain::ForgeWorkspace { repositories: Box::new([repository]) };
    let job = job_assignment(true);
    let event = crate::route::job_assignment(engine, Token::new(8), job, Some(&workspace)).expect("job encodes");
    let jig_protocol_engine_workers::Event::Down { down: host::LinkDown::AssignJob { job }, parts, .. } = event else {
        panic!("job assignment")
    };
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].token, job.workspace.unwrap().items);
    let items = temper_protocol_worker_wire::v1::Items::decode(
        &engine.workers.encoding.limits,
        &mut skein_lib::Reader::new(&parts[0].bytes),
    )
    .expect("worker items");
    assert_eq!(items.repositories().len(), 1);
    let item = items.repositories().get(0).unwrap();
    assert_eq!(item.remote(), b"https://forge.example/acme/shop.git");
    assert_eq!(item.name(), b"shop");
    assert_eq!(item.identity(), 1);
    let temper_protocol_worker_wire::v1::Start::Merge(start) = item.start() else { panic!("merge start") };
    assert_eq!(start.branch(), b"temper/change");
    assert_eq!(start.base(), &[4; 32]);
    let temper_protocol_worker_wire::v1::Access::Writable(access) = item.access() else { panic!("writable") };
    assert_eq!(access.push(), b"temper/change");
    assert!(item.save().is_none());
}

/// temper/docs/design/domain/worker/jobs.md: a job without repositories has no items part.
#[test]
fn a_job_without_a_forge_workspace_sends_no_part() {
    let mut service = fixtures::started();
    let engine = service.engine.as_mut().expect("engine");
    let event = crate::route::job_assignment(engine, Token::new(8), job_assignment(false), None).expect("job encodes");
    let jig_protocol_engine_workers::Event::Down { down: host::LinkDown::AssignJob { job }, parts, .. } = event else {
        panic!("job assignment")
    };
    assert!(job.workspace.is_none());
    assert!(parts.is_empty());
}

fn job_assignment(workspace: bool) -> host::JobAssignment {
    host::JobAssignment {
        run: Token::new(7),
        attempt: Token::new(2),
        kind: JobKind(3),
        spec: b"merge spec".as_slice().into(),
        deadline: skein_lib::Duration::from_secs(600),
        workspace: if workspace { Some(host::Workspace { workstream: 4, items: Token::new(9) }) } else { None },
        grants: Box::new([]),
    }
}

fn diagnostic_answer(
    run: &temper_engine_domain::Assignment,
    ending: host::Ending,
    spent: u64,
) -> temper_engine_domain::Event {
    use temper_engine_domain as root;
    root::Event::Worker(root::WorkerInput::Link {
        channel: Token::new(7),
        link: root::Link::Up {
            up: host::LinkUp::Answer {
                run: Token::new(run.task),
                attempt: Token::new(run.attempt),
                answer: host::RunAnswer { read: None, refused: Box::new([]), turns: 0, spent, ending },
            },
            parts: root::Parts::none(),
        },
    })
}

/// temper/docs/design/domain/worker/README.md, 4.3: a failed answer reaches the operator once.
#[test]
fn failed_answer_detail_reaches_operator_once() {
    for full in [false, true] {
        check_answer_diagnostic(
            host::Ending::Failed {
                failure: host::Failure::Run(host::RunFailure::Model),
                detail: b"program claude exit 1: bad stream".as_slice().into(),
                work: host::Work { left: None, saved: None },
            },
            47,
            full,
        );
    }
}

/// temper/docs/design/domain/worker/README.md, 4.3: refusal evidence survives pressure, once.
#[test]
fn refused_answer_reason_reaches_operator_once() {
    for reason in [
        host::Refusal::Invalid(host::Invalid::Charter),
        host::Refusal::Invalid(host::Invalid::Start {
            cause: host::StartCause::Other,
            why: b"charter".as_slice().into(),
        }),
        host::Refusal::Invalid(host::Invalid::Start {
            cause: host::StartCause::Other,
            why: b"assignment of 1300000 bytes past the link's room of 1048576 bytes".as_slice().into(),
        }),
    ] {
        for full in [false, true] {
            check_answer_diagnostic(host::Ending::Refused(reason.clone()), 0, full);
        }
    }
}

/// A restored job claim uses the same fencing and channel as the run fixture.
fn claimed_job() -> (crate::Service, temper_engine_domain::Assignment) {
    use temper_engine_domain::{Record, Write};
    let (mut service, mut run) = fixtures::claimed_chat();
    let engine = service.engine.as_mut().unwrap();
    let mut task = engine.domain.tasks().task(run.task).unwrap().clone();
    task.executor = tasks::Executor::Job { kind: 3 };
    task.flags = tasks::Flags::PLAIN;
    task.llm = None;
    task.spec.words = Box::new([]);
    task.spec.parameters = Box::new([]);
    task.spec.inputs = Box::new([]);
    task.spec.asks = false;
    let mut parent = fixtures::task_record(2);
    parent.executor = tasks::Executor::Procedure { connector: 1, code: 2 };
    parent.numbers.budget = 0;
    parent.authority.budget.spend = 0;
    parent.funder = task.funder;
    parent.delegates = Box::new([task.number]);
    parent.references = Box::new([task.number]);
    task.requester = tasks::Party::Task(parent.number);
    task.root = parent.number;
    task.depth = 1;
    let mut header = engine.domain.deployment();
    header.tasks = 2;
    header.commits = header.commits.checked_add(1).unwrap();
    task.contract = tasks::Contract::Job { bytes: 32 };
    task.spec.job =
        Some(Box::new(tasks::JobSpec { bytes: Box::new([]), deadline: skein_lib::Duration::from_secs(10) }));
    fixtures::commit(
        &mut engine.store,
        1,
        Box::new([
            Write::Save(Record::Core(core::Record::Tasks(tasks::Stored::Live(Box::new(task))))),
            Write::Save(Record::Core(core::Record::Tasks(tasks::Stored::Live(Box::new(parent))))),
            Write::Save(Record::Core(core::Record::Core(core::CoreRecord::Deployment(header)))),
        ]),
    );
    let Some(jig_protocol_store::Event::Committed { .. }) = engine.store.host.above.pop() else {
        panic!("fixture commit");
    };
    while engine.store.host.above.pop().is_some() {}
    let mut restored = fixtures::start_with_store(&[(1, 100)], Some(service.engine.take().unwrap().store));
    fixtures::restart(&mut restored);
    let jig_protocol_engine_workers::host::Output::Hello { mut hello, .. } = hello_for_loss_notes(0) else {
        unreachable!()
    };
    hello.jobs = Box::new([crate::limits::merge_bound()]);
    restored.engine.as_mut().unwrap().step(temper_engine_domain::Event::Worker(
        temper_engine_domain::WorkerInput::Link {
            channel: Token::new(7),
            link: temper_engine_domain::Link::Hello { worker: 0, hello },
        },
    ));
    let engine = restored.engine.as_mut().unwrap();
    crate::engine::clock(engine, Time::from_nanos(2_000_000_000), skein_lib::Wall::from_nanos(2_000_000_000));
    temper_engine_domain::fire(&mut engine.domain, &engine.env);
    let mut delivered = skein_lib::Queue::uncapped();
    fixtures::settle_root(&mut restored, &mut delivered);
    let mut assigned = None;
    while let Some(delivery) = delivered.pop() {
        if let temper_engine_domain::Delivery::AssignedJob { job, .. } = delivery {
            assigned = Some(job);
        }
    }
    run.attempt = assigned.expect("the restored job is assigned after its first refusal backs off").attempt.raw();
    (restored, run)
}

#[test]
fn job_answer_evidence_reaches_operator_once() {
    use temper_engine_domain::{JobDiagnostic, JobDiagnosticEnding};
    for (ending, diagnostic) in [
        (
            host::JobEnding::Failed {
                failure: host::JobFailure::Unprepared(host::Preparation::Transient),
                detail: b"fetch base: Broken".as_slice().into(),
                work: host::Work { left: None, saved: None },
            },
            JobDiagnosticEnding::Failed {
                failure: host::JobFailure::Unprepared(host::Preparation::Transient),
                detail: b"fetch base: Broken".as_slice().into(),
            },
        ),
        (
            host::JobEnding::Refused(host::JobRefusal::Invalid(host::JobInvalid::Start {
                why: b"job spec".as_slice().into(),
            })),
            JobDiagnosticEnding::Refused(host::JobRefusal::Invalid(host::JobInvalid::Start {
                why: b"job spec".as_slice().into(),
            })),
        ),
    ] {
        for full in [false, true] {
            let (mut service, run) = claimed_job();
            while service.notes().pop().is_some() {}
            let expected = crate::Note::JobAnswered(JobDiagnostic {
                task: run.task,
                run: run.task,
                attempt: run.attempt,
                kind: 3,
                ending: match &diagnostic {
                    JobDiagnosticEnding::Failed { failure, detail } => {
                        JobDiagnosticEnding::Failed { failure: *failure, detail: detail.clone() }
                    }
                    JobDiagnosticEnding::Refused(reason) => JobDiagnosticEnding::Refused(reason.clone()),
                },
            });
            for delivery in 0_u32..2 {
                if full {
                    while service.notes().room() > 0 {
                        service.notes().push(crate::Note::ForgeLacking { number: 1, names: Box::new([]) });
                    }
                }
                service.engine.as_mut().unwrap().step(temper_engine_domain::Event::Worker(
                    temper_engine_domain::WorkerInput::Link {
                        channel: Token::new(7),
                        link: temper_engine_domain::Link::Up {
                            up: host::LinkUp::JobAnswer {
                                run: Token::new(run.task),
                                attempt: Token::new(run.attempt),
                                answer: ending.clone(),
                            },
                            parts: temper_engine_domain::Parts::none(),
                        },
                    },
                ));
                advance_refusal(&mut service);
                if full {
                    while let Some(note) = service.notes().pop() {
                        assert_eq!(note, crate::Note::ForgeLacking { number: 1, names: Box::new([]) });
                    }
                }
                let engine = service.engine.as_mut().unwrap();
                crate::engine::clock(engine, engine.env.now, engine.env.wall);
                fixtures::settle_root(&mut service, &mut skein_lib::Queue::uncapped());
                let mut found = 0;
                while let Some(note) = service.notes().pop() {
                    if let crate::Note::JobAnswered(_) = note {
                        assert_eq!(note, expected);
                        found += 1;
                    }
                }
                assert_eq!(found, u32::from(delivery == 0), "once per accepted job answer");
            }
        }
    }
}

fn expected_run_note(run: &temper_engine_domain::Assignment, ending: &host::Ending) -> crate::Note {
    use temper_engine_domain::{RunDiagnostic, RunRefusalDiagnostic};
    match ending {
        host::Ending::Failed { failure, detail, .. } => crate::Note::RunFailed(RunDiagnostic {
            task: run.task,
            run: run.task,
            attempt: run.attempt,
            failure: *failure,
            detail: detail.clone(),
        }),
        host::Ending::Refused(reason) => crate::Note::RunRefused(RunRefusalDiagnostic {
            task: run.task,
            run: run.task,
            attempt: run.attempt,
            reason: reason.clone(),
        }),
        host::Ending::Ended { .. } | host::Ending::Parked { .. } => panic!("diagnostic ending"),
    }
}

fn check_answer_diagnostic(ending: host::Ending, spent: u64, full: bool) {
    let (mut service, run) = fixtures::claimed_chat();
    let expected = expected_run_note(&run, &ending);
    while service.notes().pop().is_some() {}
    for delivery in 0_u32..2 {
        if full {
            while service.notes().room() > 0 {
                service.notes().push(crate::Note::ForgeLacking { number: 1, names: Box::new([]) });
            }
        }
        let event = diagnostic_answer(&run, ending.clone(), spent);
        service.engine.as_mut().unwrap().step(event);
        if full {
            advance_refusal(&mut service);
            while let Some(note) = service.notes().pop() {
                assert_eq!(note, crate::Note::ForgeLacking { number: 1, names: Box::new([]) });
            }
            let engine = service.engine.as_mut().unwrap();
            crate::engine::clock(engine, engine.env.now, engine.env.wall);
        }
        fixtures::settle_root(&mut service, &mut skein_lib::Queue::uncapped());
        let mut found = 0_u32;
        while let Some(note) = service.notes().pop() {
            if let crate::Note::RunFailed(_) | crate::Note::RunRefused(_) = note {
                assert_eq!(note, expected);
                found = found.saturating_add(1);
            }
        }
        assert_eq!(found, u32::from(delivery == 0), "one diagnostic per answer; full={full}; {ending:?}");
    }
}

#[test]
fn opening_agreements_and_each_refusals_text_reach_shell_notes() {
    use jig_protocol_engine_workers::host::Output;
    let mut service = fixtures::started();
    let workers = &mut service.engine.as_mut().unwrap().workers;
    let worker: Box<[u8]> = b"worker-a".as_slice().into();
    let wires =
        skein_lib::List::from_boxed(Box::new([jig_protocol_worker_wire::Agreed { name: "example-wire", version: 7 }]));
    let mut events = skein_lib::Queue::uncapped();
    let statement = jig_protocol_engine_workers::Statement {
        wires: skein_lib::List::from_boxed(Box::new([jig_protocol_engine_workers::Stated {
            name: "example-wire".into(),
            lowest: 7,
            layouts: skein_lib::List::from_boxed(Box::new([[5; 32]])),
        }])),
        output: 4096,
        terms: skein_lib::List::from_boxed(Box::new([skein_channel::Term { kind: 12, largest: 34 }])),
    };
    workers.route(Output::Opened { worker: worker.clone(), wires: wires.clone(), statement }, &mut events);
    assert_eq!(workers.notes.pop(), Some(crate::Note::WorkerOpened { worker: worker.clone(), wires }));
    let Some(root::Event::Worker(root::WorkerInput::Opened { worker: name, statement })) = events.pop() else {
        panic!("accepted opening reaches the root");
    };
    assert_eq!(name.as_ref(), "worker-a");
    assert_eq!(
        statement,
        root::WorkerStatement {
            wires: Box::new([root::OpenedWire {
                name: "example-wire".into(),
                lowest: 7,
                layouts: Box::new([[5; 32]])
            }]),
            output: 4096,
            terms: Box::new([root::OpenedTerm { kind: 12, largest: 34 }]),
        }
    );
    let text: Box<[u8]> = b"wire example-wire: the worker states none".as_slice().into();
    for _ in 0_u8..2 {
        workers.route(Output::Refused { worker: Some(worker.clone()), reason: 257, text: text.clone() }, &mut events);
        assert_eq!(
            workers.notes.pop(),
            Some(crate::Note::WorkerOpeningRefused { worker: Some(worker.clone()), reason: 257, text: text.clone() })
        );
    }
    assert!(events.is_empty(), "operator notes reach no domain");
    assert!(workers.notes.is_empty());
}

#[test]
fn worker_opening_ends_are_suppressed_by_name_and_reason_until_hello() {
    use jig_protocol_engine_workers::{Loss, host::Output};
    for (worker, position) in
        [(Some(b"worker-a".as_slice()), Some(0)), (Some(b"worker-b".as_slice()), Some(1)), (None, None)]
    {
        let mut service = fixtures::started();
        let workers = &mut service.engine.as_mut().unwrap().workers;
        let worker = match worker {
            Some(name) => Some(Box::from(name)),
            None => None,
        };
        let mut events = skein_lib::Queue::uncapped();
        let why = Loss::Refused { reason: 256 };
        for _ in 0_u8..2 {
            workers.route(Output::Ended { worker: worker.clone(), channel: None, why }, &mut events);
        }
        assert_eq!(workers.notes.pop(), Some(crate::Note::WorkerOpeningEnded { worker: worker.clone(), why }));
        assert!(workers.notes.is_empty(), "the same opening refusal prints once");
        let other = u32::from(position == Some(0));
        workers.route(hello_for_loss_notes(other), &mut events);
        workers.route(Output::Ended { worker: worker.clone(), channel: None, why }, &mut events);
        assert!(workers.notes.is_empty(), "another worker's hello does not reset this name");
        if let Some(position) = position {
            workers.route(hello_for_loss_notes(position), &mut events);
            workers.route(Output::Ended { worker: worker.clone(), channel: None, why }, &mut events);
            assert_eq!(workers.notes.pop(), Some(crate::Note::WorkerOpeningEnded { worker: worker.clone(), why }));
        }
        let changed = Loss::Refused { reason: 257 };
        workers.route(Output::Ended { worker: worker.clone(), channel: None, why: changed }, &mut events);
        assert_eq!(workers.notes.pop(), Some(crate::Note::WorkerOpeningEnded { worker: worker.clone(), why: changed }));
        // A channel that said hello prints on every end, including the same cause.
        for channel in [Token::new(7), Token::new(8)] {
            workers.route(Output::Ended { worker: worker.clone(), channel: Some(channel), why }, &mut events);
            assert_eq!(workers.notes.pop(), Some(crate::Note::WorkerLost { worker: worker.clone(), channel, why }));
        }
        assert!(workers.notes.is_empty());
    }
}

/// temper/docs/design/protocol/cluster.md, 3: every unsent record has one operator note.
#[test]
fn each_unsent_record_reaches_the_operator_once() {
    use jig_protocol_engine_workers::{Bound, Unsent, host::Output};
    let mut service = fixtures::started();
    let workers = &mut service.engine.as_mut().unwrap().workers;
    let mut events = skein_lib::Queue::uncapped();
    let worker: Box<[u8]> = b"worker-a".as_slice().into();
    let channel = Token::new(7);
    let unsent =
        Unsent { kind: jig_protocol_worker_wire::Kind::Assignment, bytes: 1_300_000, bound: Bound::Room(1_048_576) };
    workers.route(Output::Unsent { worker: worker.clone(), channel, unsent }, &mut events);
    assert_eq!(workers.notes.len(), 1, "one unsent record produces one operator note");
    assert_eq!(workers.notes.pop(), Some(crate::Note::WorkerUnsent { worker, channel, unsent }));
    assert!(workers.notes.is_empty());
    assert!(events.is_empty(), "an operator note is outside the domain");
}

fn hello_for_loss_notes(worker: u32) -> jig_protocol_engine_workers::host::Output {
    jig_protocol_engine_workers::host::Output::Hello {
        worker,
        channel: Token::new(7),
        hello: jig_domain_link::Hello {
            jobs: Box::new([]),
            slots: 1,
            stop_bound: skein_lib::Duration::from_millis(100),
            workstreams: Box::new([]),
            hosting: Box::new([]),
            bounds: fixtures::worker_agent().bounds,
        },
    }
}

#[test]
fn worker_loss_notes_wait_for_the_operator_sink_without_blocking_the_link() {
    use jig_protocol_engine_workers::{Loss, host::Output};
    let mut service = fixtures::started();
    let engine = service.engine.as_mut().unwrap();
    for _ in 0..engine.notes.capacity() {
        engine.notes.push(crate::Note::ForgeLacking { number: 1, names: Box::new([]) });
    }
    let worker = Some(b"worker-a".as_slice().into());
    let channel = Token::new(7);
    engine.workers.route(
        Output::Ended { worker: worker.clone(), channel: Some(channel), why: Loss::Silence },
        &mut engine.events,
    );
    engine.workers.route(Output::Ended { worker: None, channel: None, why: Loss::NoRoom }, &mut engine.events);
    engine.drain_worker_notes();
    assert_eq!(engine.workers.notes.len(), 2, "a full diagnostic sink retains both lines");
    while engine.notes.pop().is_some() {}
    crate::engine::clock(engine, Time::ZERO, skein_lib::Wall::EPOCH);
    assert_eq!(engine.notes.pop(), Some(crate::Note::WorkerLost { worker, channel, why: Loss::Silence }));
    assert_eq!(engine.notes.pop(), Some(crate::Note::WorkerOpeningEnded { worker: None, why: Loss::NoRoom }));
    assert!(engine.workers.notes.is_empty());
    assert!(engine.notes.is_empty());
}
