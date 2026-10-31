#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "ordinary Rust step harness, programming-model.md 10.2"
)]
use super::fixtures::{backend, env, request};
use super::*;
use alloc::vec;

#[test]
fn a_late_spawn_after_grace_is_terminated_as_a_group() {
    let mut backend = backend();
    backend.phase = Phase::Starting;
    backend.spawning = true;
    let mut up = Queue::with_capacity(32);
    let mut below = Queue::with_capacity(64);
    backend.close(&env(0), &mut up, &mut below);
    backend.fire(&env(2), &mut up, &mut below);
    assert!(backend.spawning);
    assert!(backend.stopping > 0);
    backend.from_io(
        &env(3),
        BelowEvent::Io(io::Event::Spawned {
            owner: backend.token(4),
            child: Token::new(30),
            pipes: Box::new([Token::new(31), Token::new(32), Token::new(33)]),
        }),
        &mut up,
        &mut below,
    );
    assert!(below.iter().any(|request| {
        if let Below::Io(io::Request::Signal { child, to: kernel::Target::Group, signal: kernel::Signal::Terminate }) =
            request
        {
            *child == Token::new(30)
        } else {
            false
        }
    }));
    backend.fire(&env(4), &mut up, &mut below);
    assert!(below.iter().any(|request| if let Below::Io(io::Request::Close { entity }) = request {
        *entity == Token::new(30)
    } else {
        false
    }));
}

#[test]
fn a_child_ignoring_termination_reaches_a_bounded_close() {
    let mut backend = backend();
    backend.phase = Phase::Running;
    backend.child = Some(Token::new(30));
    let mut up = Queue::with_capacity(32);
    let mut below = Queue::with_capacity(64);
    backend.close(&env(0), &mut up, &mut below);
    backend.fire(&env(1), &mut up, &mut below);
    assert!(below.iter().any(|request| {
        if let Below::Io(io::Request::Signal { to: kernel::Target::Group, signal, .. }) = request {
            *signal == kernel::Signal::Terminate
        } else {
            false
        }
    }));
    backend.fire(&env(2), &mut up, &mut below);
    assert!(below.iter().any(|request| {
        if let Below::Io(io::Request::Signal { to: kernel::Target::Group, signal, .. }) = request {
            *signal == kernel::Signal::Kill
        } else {
            false
        }
    }));
}
#[test]
fn a_renewed_grant_survives_its_first_lapse() {
    let mut backend = backend();
    let name = GrantName { account: 0, generation: 1 };
    backend
        .grant(Grant { name, source: b"source".as_slice().into(), lapses: env(2).now })
        .expect("bounded test fixture");
    backend
        .grant(Grant { name, source: b"source".as_slice().into(), lapses: env(4).now })
        .expect("bounded test fixture");
    assert!(backend.grant.as_ref().expect("bounded test fixture").lapses > env(3).now);
    // Drive the admission after the first grant expired: SignedOut would be
    // immediate if renewal had been dropped; the missing root is separate.
    backend.settings.directory = Some(Root { file: Token::new(40), descriptor: kernel::Fd::new(41) });
    let request = request(&backend);
    let mut up = Queue::with_capacity(32);
    let mut below = Queue::with_capacity(32);
    backend.from_domain(&env(3), request, &mut up, &mut below);
    assert_eq!(backend.phase, Phase::Checking);
    assert!(up.is_empty());
    assert!(below.iter().any(|request| if let Below::Io(io::Request::Spawn { owner, .. }) = request {
        *owner == backend.token(3)
    } else {
        false
    }));
}
#[test]
fn an_accepted_init_uses_the_completion_head_deadline() {
    let mut backend = backend();
    backend.phase = Phase::Starting;
    backend.deadline = Some(env(30).now);
    backend.owner = Some(Token::new(80));
    backend.request = Some(request(&backend));
    backend.child = Some(backend.token(4));
    backend.pipes = [Some(Token::new(31)), Some(Token::new(32)), Some(Token::new(33))];
    let mut up = Queue::with_capacity(32);
    let mut below = Queue::with_capacity(64);
    backend.from_io(&env(1),BelowEvent::Io(io::Event::Stream { owner: Token::new(32),up: skein_lib::stream::Up::Bytes(br#"{"type":"system","subtype":"init","mcp_servers":[{"name":"smith","status":"connected"}],"tools":["mcp__smith__permission_prompt"]}
"#.as_slice().into()) }),&mut up,&mut below);
    assert_eq!(backend.phase, Phase::Running);
    assert_eq!(backend.next_deadline(), Some(env(2).now));
    backend.fire(&env(2), &mut up, &mut below);
    let Some(ToDomain::Failed { failure, .. }) = up.pop() else { panic!("completion head deadline") };
    assert_eq!(failure, llm::Failure::TimedOut { phase: llm::Phase::Head });
    assert_eq!(backend.phase, Phase::Running);
    assert!(backend.switching, "stops the expired invocation while retaining retry storage");
    assert!(backend.stop == stop::Stop::Closing);
    assert!(backend.owner.is_none());
}
#[test]
fn private_diagnostics_redact_known_private_values() {
    let backend = backend();
    let secret = mcp::secret_text(backend.names.mcp.expose().try_into().expect("bounded test fixture"));
    let bytes = mcp::json::join(&[b"stderr ", &secret, b" ", &backend.run_path(b"mcp.json"), b" tail"]);
    let text = backend.private_diagnostic(&bytes);
    assert!(!text.windows(secret.len()).any(|window| window == secret));
    assert!(!text.windows(backend.names.run.len()).any(|window| window == backend.names.run));
    assert!(text.ends_with(b" tail"));
}
#[test]
fn native_input_is_written_in_whole_chunks_within_its_room() {
    let mut backend = backend();
    backend.phase = Phase::Running;
    backend.pipes[0] = Some(Token::new(31));
    backend.outgoing = Some(vec![b'x'; 10001].into());
    backend.send_input();
    let mut count = 0;
    let mut chunks = 0_u32;
    let mut up = Queue::with_capacity(32);
    let mut below = Queue::with_capacity(64);
    for _ in 0_u32..3 {
        backend.flush(&mut below);
        let room = below.pop().expect("bounded test fixture");
        let Below::Io(io::Request::Output { down: skein_lib::stream::OutputDown::Room { bytes, .. }, .. }) = room
        else {
            panic!("input room");
        };
        assert!(bytes <= 4096);
        backend.from_io(
            &env(0),
            BelowEvent::Io(io::Event::Output {
                owner: Token::new(31),
                up: skein_lib::stream::OutputUp::Settled {
                    right: Token::new(1),
                    outcome: skein_lib::stream::OutputOutcome::Granted,
                },
            }),
            &mut up,
            &mut below,
        );
        let Below::Io(io::Request::Output { down: skein_lib::stream::OutputDown::Send { bytes, .. }, .. }) =
            below.pop().expect("bounded test fixture")
        else {
            panic!("input chunk");
        };
        count += bytes.len();
        chunks += 1;
    }
    assert_eq!(count, 10001);
    assert_eq!(chunks, 3);
    assert!(backend.outgoing.is_none());
}

#[test]
fn an_input_line_can_exceed_the_output_declaration() {
    let backend = backend();
    let FromDomain::Complete { mut prompt, .. } = request(&backend) else {
        panic!("completion");
    };
    prompt.messages = Box::new([llm::Message {
        role: llm::Role::User,
        content: Box::new([llm::Block::Text { text: vec![1; 2000].into(), replay: None }]),
    }]);
    assert!(stream::user_message(&prompt, backend.settings.line_bytes).is_none());
    let input = stream::user_message(&prompt, backend.settings.input_bytes).expect("independent input cap");
    assert!(input.len() > 4096);
}
#[test]
fn lower_debug_keeps_only_the_store_length() {
    let backend = backend();
    let secret = mcp::secret_text(backend.names.mcp.expose().try_into().expect("bounded test fixture"));
    let effect = Below::File {
        request: file::Request::Store {
            owner: Token::new(1),
            root: Token::new(2),
            path: backend.run_path(b"mcp.json"),
            bytes: secret.as_slice().into(),
            expected: file::Expect::Absent,
            no_follow: true,
        },
        deadline: Time::ZERO,
    };
    let text = alloc::format!("{effect:?}");
    assert_eq!(text, "File { payload_bytes: 64, .. }");
}

#[test]
fn the_input_line_bound_includes_its_newline() {
    let backend = backend();
    let FromDomain::Complete { prompt, .. } = request(&backend) else {
        panic!("completion");
    };
    let expected = b"{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":[]}}\n";
    let exact = u32::try_from(expected.len()).expect("fixed user line");
    assert!(stream::user_message(&prompt, exact.checked_sub(1).expect("one less byte")).is_none());
    assert_eq!(stream::user_message(&prompt, exact).expect("exact user line").as_ref(), expected);
}
