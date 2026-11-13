#[expect(clippy::module_inception, reason = "Keep the existing test paths under the suite file module.")]
mod opening {
    use jig_engine_workers_world::{
        World,
        fixtures::{self, OTHER_PAYLOAD, PAYLOAD, SECRET_A, SECRET_B, SECRET_C, WORKER_A, WORKER_B, WORKER_C},
    };
    use skein_lib::Duration;
    fn world() -> World {
        World::new(&fixtures::limits(), &fixtures::workers(), 1)
    }
    fn connect_a(world: &mut World) -> skein_lib::Token {
        world.connect(fixtures::worker(WORKER_A, SECRET_A, PAYLOAD))
    }
    fn refused(world: &World, connection: skein_lib::Token) -> u16 {
        world.refusal(connection).expect("a refusal").0
    }
    fn stating(wires: skein_lib::List<jig_protocol_worker_wire::Wire>) -> skein_fake_channel::ScriptedPeer {
        use jig_protocol_worker_wire as wire;
        let value = wire::Credential::new(&wire::CEILINGS, wire::CredentialParts {
            name: WORKER_A.into(), secret: skein_lib::Secret::new(SECRET_A.into()), wires, output: fixtures::LIMITS.room,
        }).expect("credential");
        let mut bytes = skein_lib::Writer::new(usize::try_from(value.measure()).expect("credential length"));
        value.encode(&mut bytes).expect("credential");
        skein_fake_channel::ScriptedPeer::new(fixtures::schema(), skein_channel::Role::Initiator, fixtures::channel_limits(), wire::VERSION, bytes.finish()).expect("worker")
    }

    #[test]
    fn a_workers_agreed_wires_reach_the_owner_by_name_in_statement_order() {
        use jig_protocol_engine_workers::Request;
        let mut world = world();
        let mut statement = fixtures::statement(PAYLOAD);
        let first = statement.swap_remove(0).expect("first wire");
        statement.push(first).expect("wire slot");
        let connection = world.connect(stating(statement.clone()));
        world.run();
        assert!(world.ready(connection));
        let [Request::Opened { worker: 0, wires }] = world.openings() else { panic!("one agreed statement") };
        assert_eq!(wires.len(), statement.len());
        for (agreed, stated) in wires.iter().zip(&statement) {
            assert_eq!(agreed.name.as_bytes(), stated.name());
            assert_eq!(agreed.version, stated.lowest());
        }
        assert!(world.root().is_empty(), "the domain has heard no hello");
    }

    #[test]
    fn wire_mismatches_are_refused_with_the_same_words_to_the_worker_and_owner() {
        use jig_protocol_engine_workers::Request;
        use jig_protocol_worker_wire as wire;
        let mut layout = fixtures::statement(PAYLOAD);
        let mut parts = layout.get(4).expect("application wire").clone().into_parts();
        *parts.layouts.get_mut(0).expect("layout") = [4; 32];
        *layout.get_mut(4).expect("application wire") = wire::Wire::new(&wire::CEILINGS, parts).expect("wire");
        let mut missing = fixtures::statement(PAYLOAD);
        missing.truncate(4);
        let mut twice = fixtures::statement(PAYLOAD);
        twice.push(twice.get(4).expect("application wire").clone()).expect("wire slot");
        for (statement, text) in [
            (layout, b"wire test-worker-wire version 3: the engine's layout 0303030303030303, the worker's 0404040404040404".as_slice()),
            (fixtures::statement(OTHER_PAYLOAD), b"wire test-worker-wire: the engine speaks 3..3, the worker 4..4"),
            (missing, b"wire test-worker-wire: the worker states none"),
            (twice, b"wire test-worker-wire: stated twice"),
        ] {
            let mut world = world();
            let connection = world.connect(stating(statement));
            world.run();
            assert_eq!(world.refusal(connection), Some((257, text.into())));
            let [Request::Refused { worker: Some(0), reason: 257, text: heard }] = world.openings() else { panic!("one named refusal") };
            assert_eq!(heard.as_ref(), text);
            assert!(world.root().is_empty());
        }
    }
    #[test]
    fn unequal_output_rooms_are_refused_with_both_values_and_the_setting() {
        use jig_protocol_engine_workers::{Loss, Request};
        use jig_protocol_worker_wire as wire;
        const ENGINE: u32 = 1_048_576;
        for (output, secret, payload, reason) in [
            (ENGINE, SECRET_A, PAYLOAD, None),
            (ENGINE - 1, SECRET_A, PAYLOAD, Some(2)),
            (ENGINE + 1, SECRET_A, PAYLOAD, Some(2)),
            (18_432_634, SECRET_A, PAYLOAD, Some(2)),
            (u32::MAX, SECRET_A, PAYLOAD, Some(2)),
            (18_432_634, b"wrong".as_slice(), PAYLOAD, Some(256)),
            (18_432_634, SECRET_A, OTHER_PAYLOAD, Some(257)),
        ] {
            let limits = jig_protocol_engine_workers::Limits { room: ENGINE, ..fixtures::limits() };
            let mut world = World::new(&limits, &fixtures::workers(), 1);
            let peer = fixtures::peer_with_output(fixtures::schema(), wire::VERSION, WORKER_A, secret, payload, output);
            let connection = world.connect(peer);
            world.run();
            if let Some(reason) = reason {
                let text: Box<[u8]> = if reason == 2 {
                    format!("engine io.output {ENGINE} bytes differs from worker io.output {output} bytes; change io.output to agree")
                        .into_bytes()
                        .into_boxed_slice()
                } else if reason == 257 {
                    Box::from(b"wire test-worker-wire: the engine speaks 3..3, the worker 4..4".as_slice())
                } else {
                    Box::new([])
                };
                assert_eq!(world.refusal(connection), Some((reason, text)), "worker room {output}");
                assert!(!world.ready(connection));
                if reason == 2 {
                    let [Request::Ended { worker, channel, why, .. }] = world.ended() else {
                        panic!("one opening diagnostic")
                    };
                    assert_eq!(
                        (*worker, *channel, *why),
                        (Some(0), None, Loss::OutputRoom { engine: ENGINE, worker: output })
                    );
                }
            } else {
                assert_eq!(world.refusal(connection), None);
                assert!(world.ready(connection));
            }
            assert!(world.root().is_empty(), "no hello reaches the domain");
        }
    }
    #[test]
    fn a_wrong_secret_is_refused_as_unauthorized() {
        let mut world = world();
        let connection = world.connect(fixtures::worker(WORKER_A, b"wrong", PAYLOAD));
        world.run();
        assert_eq!(refused(&world, connection), 256);
        assert!(world.root().is_empty());
    }
    #[test]
    fn a_third_worker_past_the_fleet_limit_is_refused_as_busy() {
        let mut world = world();
        connect_a(&mut world);
        world.connect(fixtures::worker(WORKER_B, SECRET_B, PAYLOAD));
        world.run();
        let third = world.connect(fixtures::worker(WORKER_C, SECRET_C, PAYLOAD));
        world.run();
        assert_eq!(refused(&world, third), 258);
        assert!(world.root().is_empty());
    }
    #[test]
    fn a_worker_dialling_again_replaces_its_open_channel() {
        let mut world = world();
        let older = connect_a(&mut world);
        world.run();
        let newer = connect_a(&mut world);
        world.run();
        assert_eq!(refused(&world, older), 259);
        assert!(world.ready(newer));
        assert!(world.gone(older));
    }
    #[test]
    fn another_version_is_refused_by_skein_as_version() {
        let mut schema = fixtures::schema();
        let other = 9; // The previous channel is refused before reading its credential.
        schema.versions.get_mut(0).expect("one version").version = other;
        let mut world = world();
        let peer = skein_fake_channel::ScriptedPeer::new(schema, skein_channel::Role::Initiator, fixtures::channel_limits(), other, b"not a credential".as_slice().into()).expect("old worker");
        let connection = world.connect(peer);
        world.run();
        assert_eq!(world.refusal(connection), Some((1, b"speaks 10..10, offered 9..9".as_slice().into())));
        let [jig_protocol_engine_workers::Request::Refused { worker: None, reason: 1, text }] = world.openings() else { panic!("precredential refusal") };
        assert_eq!(text.as_ref(), b"speaks 10..10, offered 9..9");
        assert!(world.root().is_empty());
    }
    #[test]
    fn terms_below_the_largest_assignment_are_refused_as_limits() {
        let mut schema = fixtures::schema();
        let kinds = &mut schema.versions.get_mut(0).expect("one version").kinds;
        for index in 0..kinds.len() {
            let kind = kinds.get_mut(index).expect("kind slot");
            if kind.kind == 0x0180 {
                kind.largest = 1;
            }
        }
        let mut world = world();
        let connection =
            world.connect(fixtures::peer(schema, jig_protocol_worker_wire::VERSION, WORKER_A, SECRET_A, PAYLOAD));
        world.run();
        assert_eq!(world.refusal(connection), Some((2, Box::from(b"384".as_slice()))));
        assert!(world.root().is_empty());
    }
    #[test]
    fn an_opening_past_the_handshake_deadline_is_closed() {
        let mut world = world();
        let connection = connect_a(&mut world);
        world.withhold_write_room(connection);
        world.run();
        world.advance(Duration::from_secs(2));
        assert!(world.gone(connection));
        assert!(world.root().is_empty());
    }
    #[test]
    fn connections_past_workers_and_openings_are_turned_away() {
        let mut world = world();
        for _ in 0..3 {
            connect_a(&mut world);
        }
        let fourth = connect_a(&mut world);
        assert!(world.gone(fourth));
        assert!(!world.ready(fourth));
    }
}
