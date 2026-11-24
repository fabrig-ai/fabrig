#[expect(clippy::module_inception, reason = "Keep the existing test paths under the suite file module.")]
mod opening {
    use jig_engine_workers_world::{
        World,
        fixtures::{
            self, APPLICATION_VERSION, OTHER_VERSION, SECRET_A, SECRET_B, SECRET_C, WORKER_A, WORKER_B, WORKER_C,
        },
    };
    use jig_protocol_worker_wire::Record;
    use skein_lib::Duration;
    fn world() -> World {
        World::new(&fixtures::limits(), &fixtures::workers(), 1)
    }
    fn connect_a(world: &mut World) -> skein_lib::Token {
        world.connect(fixtures::worker(WORKER_A, SECRET_A, APPLICATION_VERSION))
    }
    fn refused(world: &World, connection: skein_lib::Token) -> u16 {
        world.refusal(connection).expect("a refusal").0
    }
    fn stating(wires: skein_lib::List<jig_protocol_worker_wire::Wire>) -> skein_fake_channel::ScriptedPeer {
        stating_with_schema(wires, fixtures::schema())
    }
    fn stating_with_schema(
        wires: skein_lib::List<jig_protocol_worker_wire::Wire>,
        schema: skein_channel::Schema,
    ) -> skein_fake_channel::ScriptedPeer {
        use jig_protocol_worker_wire as wire;
        let value = wire::Credential {
            opening: jig_protocol_worker_wire::Opening::Link,
            name: WORKER_A.into(),
            secret: skein_lib::Secret::new(SECRET_A.into()),
            wires,
            output: fixtures::LIMITS.room,
        };
        let mut bytes = skein_lib::Writer::new(
            usize::try_from(value.measure(&jig_protocol_worker_wire::CEILINGS).expect("record within limits"))
                .expect("credential length"),
        );
        value.write(&jig_protocol_worker_wire::CEILINGS, &mut bytes).expect("credential");
        let channel = skein_channel::Limits {
            kinds: schema.version(wire::VERSION).expect("worker table").kinds.len(),
            ..fixtures::channel_limits()
        };
        skein_fake_channel::ScriptedPeer::new(
            schema,
            skein_channel::Role::Initiator,
            channel,
            wire::VERSION,
            bytes.finish(),
        )
        .expect("worker")
    }

    fn spoken() -> skein_lib::List<jig_protocol_worker_wire::Spoken> {
        skein_lib::List::from_boxed(fixtures::kept_wires().iter().chain(&fixtures::application()).cloned().collect())
    }

    fn whole(
        wires: &skein_lib::List<jig_protocol_worker_wire::Wire>,
        schema: &skein_channel::Schema,
    ) -> jig_protocol_engine_workers::Statement {
        use jig_protocol_engine_workers::{Stated, Statement};
        use jig_protocol_worker_wire::VERSION;
        use skein_channel::{Direction, Term};
        Statement {
            wires: skein_lib::List::from_boxed(
                wires
                    .iter()
                    .map(|wire| Stated {
                        name: core::str::from_utf8(&wire.name).expect("wire name").into(),
                        layout: wire.layout,
                        text: wire.text.clone(),
                    })
                    .collect(),
            ),
            output: fixtures::LIMITS.room,
            terms: skein_lib::List::from_boxed(
                schema
                    .version(VERSION)
                    .expect("current version")
                    .kinds
                    .iter()
                    .filter(|rule| rule.direction == Direction::FromResponder)
                    .map(|rule| Term { kind: rule.kind, largest: rule.largest })
                    .collect(),
            ),
        }
    }

    fn saved_judgement(
        statement: &jig_protocol_engine_workers::Statement,
    ) -> Result<skein_lib::List<Box<str>>, jig_protocol_engine_workers::Unfit> {
        let limits = jig_protocol_worker_wire::sizes::limits(&fixtures::SIZES).expect("codec");
        jig_protocol_engine_workers::judge(
            &spoken(),
            &skein_lib::List::with_capacity(0),
            fixtures::LIMITS.room,
            &limits,
            statement,
        )
    }

    fn accept(wires: skein_lib::List<jig_protocol_worker_wire::Wire>, schema: skein_channel::Schema, names: &[&str]) {
        use jig_protocol_engine_workers::Request;
        let expected = whole(&wires, &schema);
        let mut world = world();
        let connection = world.connect(stating_with_schema(wires, schema));
        world.run();
        assert!(world.ready(connection));
        assert_eq!(world.refusal(connection), None);
        let [Request::Opened { worker: 0, statement, converted }] = world.openings() else {
            panic!("one statement: {:?}", world.openings())
        };
        assert_eq!(statement, &expected);
        assert_eq!(converted.iter().map(AsRef::as_ref).collect::<Vec<&str>>(), names);
        assert_eq!(saved_judgement(&expected).expect("same judgement"), *converted);
        assert!(world.root().is_empty(), "the domain has heard no hello");
    }

    #[test]
    fn matching_layouts_use_the_engines_table_without_parsing_any_worker_text() {
        let mut wires = fixtures::statement(APPLICATION_VERSION);
        for wire in wires.iter_mut() {
            wire.text = b"not a schema".as_slice().into();
        }
        let first = wires.swap_remove(0).expect("first wire");
        wires.push(first).expect("wire slot");
        wires
            .push(jig_protocol_worker_wire::Wire {
                name: b"ignored".as_slice().into(),
                layout: [0; 32],
                text: Box::new([]),
            })
            .expect("extra wire");
        accept(wires, fixtures::schema(), &[]);
    }

    #[test]
    fn derived_layouts_open_with_the_workers_table_and_name_the_converted_wires() {
        use jig_protocol_engine_workers::{Request, Unfit};
        use jig_protocol_worker_wire as wire;
        use skein_channel::Kind;
        let mut wires = fixtures::statement(APPLICATION_VERSION);
        let mut schema = fixtures::schema();
        // An additional responder kind is in the worker's terms, but not the engine's own table.
        let current = schema.versions.get_mut(0).expect("one version");
        let part_busy = current.kind(wire::Kind::PartBusy.number()).expect("part busy");
        let mut worker_kinds: Vec<_> = current.kinds.iter().copied().collect();
        worker_kinds.push(Kind { kind: part_busy.kind + 1, ..part_busy });
        current.kinds = skein_lib::List::from_boxed(worker_kinds.into_boxed_slice());
        for (index, old, new) in [
            (0, "part_busy: PartBusy", "part_busy: PartBusy, quiet: PartBusy"),
            (2, "versioned record Charter {", "versioned record Charter { test_note: u8 = 0"),
            (3, "versioned record Turn {", "versioned record Turn { test_note: u8 = 0"),
            (4, " note: u8 = 0", ""),
        ] {
            let changed = wires.get_mut(index).expect("stated wire");
            let text = core::str::from_utf8(&changed.text).expect("current text").replacen(old, new, 1);
            replace_text(changed, text.as_bytes());
        }
        let first = wires.swap_remove(0).expect("first wire");
        wires.push(first).expect("wire slot");
        accept(
            wires.clone(),
            schema.clone(),
            &["jig-worker-wire", "smith-charter", "smith-transcript", "test-worker-wire"],
        );

        let quiet = schema.versions.get_mut(0).expect("one version").kinds.iter_mut().last().expect("quiet kind");
        let text = quiet.kind.to_string().into_bytes().into_boxed_slice();
        quiet.largest -= 1;
        assert_eq!(saved_judgement(&whole(&wires, &schema)), Err(Unfit::Limits(text.clone())));
        let mut world = world();
        let connection = world.connect(stating_with_schema(wires, schema));
        world.run();
        assert_eq!(world.refusal(connection), Some((2, text.clone())));
        let [Request::Refused { reason: 2, text: heard, .. }] = world.openings() else { panic!("one term gap") };
        assert_eq!(heard, &text);
    }

    fn replace_text(wire: &mut jig_protocol_worker_wire::Wire, text: &[u8]) {
        wire.layout = skein_codec::schema::parse(text).expect("derived text").fingerprints().family().digest();
        wire.text = text.into();
    }

    #[test]
    fn wire_mismatches_are_refused_with_the_same_words_to_the_worker_and_owner() {
        use jig_protocol_engine_workers::{Request, Unfit};
        use std::fmt::Write as _;
        let ordinary = fixtures::statement(APPLICATION_VERSION);
        let mut missing = ordinary.clone();
        missing.truncate(4);
        let mut twice = ordinary.clone();
        twice.push(ordinary.get(4).expect("application wire").clone()).expect("wire slot");
        let mut pieces = ordinary.clone();
        let pieces_wire = pieces.get_mut(1).expect("pieces wire");
        let mut hex = String::new();
        for byte in pieces_wire.layout.iter().take(8) {
            write!(hex, "{byte:02x}").expect("hex layout");
        }
        pieces_wire.layout = [2; 32];
        let pieces_text = format!(
            "wire smith-pieces: the worker's layout 0202020202020202 is not the engine's {hex}, and pieces cross as written"
        );
        let malformed = fixtures::statement(OTHER_VERSION);
        let mut foreign = ordinary.clone();
        replace_text(foreign.get_mut(2).expect("charter"), &ordinary.get(4).expect("app").text);
        let mut not_layout = ordinary.clone();
        not_layout.get_mut(2).expect("charter").layout = [1; 32];
        let mut no_plan = ordinary.clone();
        let charter = no_plan.get_mut(2).expect("charter");
        let retyped = core::str::from_utf8(&charter.text).expect("current text").replacen(
            "instructions: text 262144",
            "instructions: u32",
            1,
        );
        replace_text(charter, retyped.as_bytes());
        for (wires, text) in [
            (missing, b"wire test-worker-wire: the worker states none".as_slice()),
            (twice, b"wire test-worker-wire: stated twice"),
            (pieces, pieces_text.as_bytes()),
            (malformed, b"wire test-worker-wire: the worker's text does not parse at line 1"),
            (foreign, b"wire smith-charter: the worker's text is of test-worker-wire"),
            (not_layout, b"wire smith-charter: the worker's text is not its layout 0101010101010101"),
            (no_plan, b"wire smith-charter Charter: no plan to the worker: retyped at Charter.instructions"),
        ] {
            let mut world = world();
            let whole = whole(&wires, &fixtures::schema());
            let connection = world.connect(stating(wires));
            world.run();
            assert_eq!(world.refusal(connection), Some((257, text.into())));
            let [Request::Refused { worker: Some(0), reason: 257, text: heard }] = world.openings() else {
                panic!("one named refusal")
            };
            assert_eq!(heard.as_ref(), text);
            assert_eq!(saved_judgement(&whole), Err(Unfit::Wires(text.into())));
            assert!(world.root().is_empty());
        }
    }
    #[test]
    fn unequal_output_rooms_are_refused_with_both_values_and_the_setting() {
        use jig_protocol_engine_workers::{Loss, Request};
        use jig_protocol_worker_wire as wire;
        const ENGINE: u32 = 1_048_576;
        for (output, secret, payload, reason) in [
            (ENGINE, SECRET_A, APPLICATION_VERSION, None),
            (ENGINE - 1, SECRET_A, APPLICATION_VERSION, Some(2)),
            (ENGINE + 1, SECRET_A, APPLICATION_VERSION, Some(2)),
            (18_432_634, SECRET_A, APPLICATION_VERSION, Some(2)),
            (u32::MAX, SECRET_A, APPLICATION_VERSION, Some(2)),
            (18_432_634, b"wrong".as_slice(), APPLICATION_VERSION, Some(260)),
            (18_432_634, SECRET_A, OTHER_VERSION, Some(257)),
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
                    Box::from(b"wire test-worker-wire: the worker's text does not parse at line 1".as_slice())
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
    fn a_wrong_secret_is_refused_as_not_admitted() {
        let mut world = world();
        let connection = world.connect(fixtures::worker(WORKER_A, b"wrong", APPLICATION_VERSION));
        world.run();
        assert_eq!(refused(&world, connection), 260);
        assert!(world.root().is_empty());
    }
    #[test]
    fn a_third_worker_past_the_fleet_limit_is_refused_as_busy() {
        let mut world = world();
        connect_a(&mut world);
        world.connect(fixtures::worker(WORKER_B, SECRET_B, APPLICATION_VERSION));
        world.run();
        let third = world.connect(fixtures::worker(WORKER_C, SECRET_C, APPLICATION_VERSION));
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
        let other = 15; // An old channel is refused before reading its credential.
        schema.versions.truncate(1);
        schema.versions.get_mut(0).expect("one version").version = other;
        let mut world = world();
        let peer = skein_fake_channel::ScriptedPeer::new(
            schema,
            skein_channel::Role::Initiator,
            fixtures::channel_limits(),
            other,
            b"not a credential".as_slice().into(),
        )
        .expect("old worker");
        let connection = world.connect(peer);
        world.run();
        let current = jig_protocol_worker_wire::VERSION;
        let expected = format!("speaks {current}..{current}, offered {other}..{other}");
        assert_eq!(world.refusal(connection), Some((1, expected.as_bytes().into())));
        let [jig_protocol_engine_workers::Request::Refused { worker: None, reason: 1, text }] = world.openings() else {
            panic!("precredential refusal")
        };
        assert_eq!(text.as_ref(), expected.as_bytes());
        assert!(world.root().is_empty());
    }
    #[test]
    fn terms_below_the_largest_assignment_are_refused_as_limits() {
        let mut schema = fixtures::schema();
        for version in schema.versions.iter_mut() {
            if version.version != jig_protocol_worker_wire::VERSION {
                continue;
            }
            for kind in version.kinds.iter_mut() {
                if kind.kind == 0x0180 {
                    kind.largest = 1;
                }
            }
        }
        let mut world = world();
        let connection = world.connect(fixtures::peer(
            schema,
            jig_protocol_worker_wire::VERSION,
            WORKER_A,
            SECRET_A,
            APPLICATION_VERSION,
        ));
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
