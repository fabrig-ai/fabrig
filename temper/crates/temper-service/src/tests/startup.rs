// Fixture parsing without a deployment keeps written paths.
#![expect(clippy::disallowed_macros, reason = "ordinary tests format configuration fixtures")]
#![expect(clippy::disallowed_methods, reason = "ordinary test code edits UTF-8 configuration fixtures")]
use super::fixtures;
use crate::settings;
use crate::startup::{Problems, What, application, lines};
use jig_protocol_config::{self as config, Path, Segment};
use skein_lib::List;

const REFUSED_TEMPLATE: u32 = 7;

fn transcript_document(bytes: u32) -> Box<[u8]> {
    let document = fixtures::document();
    let document = core::str::from_utf8(&document).unwrap();
    let previous = fixtures::startup_agent().limits.bodies.turn_body;
    let needle = format!("\"root.loads.transcript_bytes\":{previous}");
    assert_eq!(document.matches(&needle).count(), 1);
    let mut document = document.replace(&needle, &format!("\"root.loads.transcript_bytes\":{bytes}"));
    // A raised transcript needs startup's turn-plus-record and decoded read-row
    // room too; the hard reply bound must cover the kept load.
    let transcript = bytes.max(previous);
    let loaded = transcript.checked_add(u32::try_from(size_of::<temper_engine_domain::Record>()).unwrap()).unwrap();
    let read = u64::from(transcript)
        .checked_add(u64::try_from(size_of::<jig_domain_core::reads::Row>()).unwrap())
        .unwrap()
        .checked_add(u64::try_from(size_of::<jig_domain_core::Record>()).unwrap())
        .unwrap();
    let previous_loaded =
        previous.checked_add(u32::try_from(size_of::<temper_engine_domain::Record>()).unwrap()).unwrap();
    for (key, before, after) in [
        ("root.loads.bytes", u64::from(previous_loaded), u64::from(loaded)),
        ("root.loads.reply_bytes", u64::from(previous_loaded), u64::from(loaded)),
        ("root.loads.read_bytes", u64::from(previous).checked_mul(2).unwrap(), read),
    ] {
        let needle = format!("\"{key}\":{before}");
        assert_eq!(document.matches(&needle).count(), 1);
        document = document.replace(&needle, &format!("\"{key}\":{after}"));
    }
    document.into_bytes().into_boxed_slice()
}

#[test]
fn startup_admits_a_frame_exactly_at_the_profile_page_ceiling() {
    let document = transcript_document(1_606_799);
    crate::startup(
        &document,
        &config::Directories::working(),
        &fixtures::files(),
        &fixtures::startup_agent(),
        &fixtures::seeds(),
        None,
        fixtures::worker_runtime(),
        fixtures::forge_runtime(),
    )
    .expect("a whole frame exactly at Small's page ceiling starts");
    let opened =
        crate::store_limits(&document, &config::Directories::working(), Some(&fixtures::startup_agent())).unwrap();
    assert_eq!(opened.store.store.page.bytes, 1_606_863);
}

#[test]
fn startup_names_a_frame_one_byte_past_the_profile_page_ceiling() {
    let document = transcript_document(1_606_800);
    let expected = b"application.limits.overrides.root.loads.transcript_bytes: its turn's transcript frame, 1606864 bytes, needs a store page past the profile's ceiling of 1606863 bytes\n";
    let named = crate::named(&document, &config::Directories::working()).unwrap_err();
    assert_eq!(lines(&named).as_ref(), expected);
    let opened =
        crate::store_limits(&document, &config::Directories::working(), Some(&fixtures::startup_agent())).unwrap_err();
    assert_eq!(lines(&opened).as_ref(), expected);
}

#[test]
fn startup_names_a_transcript_past_the_codec_ceiling() {
    let document = transcript_document(8_388_609);
    let found = crate::named(&document, &config::Directories::working()).unwrap_err();
    assert_eq!(
        lines(&found).as_ref(),
        b"application.limits.overrides.root.loads.transcript_bytes: 8388609 bytes, past the transcript codec's ceiling of 8388608\n"
    );
}

#[test]
fn startup_names_map_sizing_without_inventing_a_records_bound() {
    let mut engine = fixtures::engine();
    engine.jig.limits.store.store.key = 0;
    let mut problems = Problems::empty();
    crate::startup::checks::records(&engine.jig, &engine.limits, &mut problems);
    assert_eq!(problems.problems.len(), 1);
    let What::StoreMap(jig_protocol_store::sizes::SizeRefusal::Key { needs, has: 0 }) =
        problems.problems.get(0).unwrap().what
    else {
        panic!("the map's key admission is the only cause: {problems:?}");
    };
    assert!(needs > 0);
    assert!(lines(&problems).starts_with(b"engine.limits.store: the store's map key, "));
}

#[test]
fn record_sizing_leaves_driver_admission_until_after_agent_checks() {
    let mut engine = fixtures::engine();
    engine.limits.root.loads.transcript_bytes = 1_000_000;
    engine.limits.root.loads.read_bytes = 1_000_000;
    let root = crate::limits::engine(&engine.jig.limits, &engine.limits).unwrap();
    let refused = crate::store::admit(&root, &root.forge, &engine.jig.limits.store, None).unwrap_err();
    let crate::store::Refusal::ReadRoom { read, needs } = refused else {
        panic!("the driver's page is the only refusal: {refused:?}");
    };
    assert!(read < needs);

    let mut problems = Problems::empty();
    crate::startup::checks::records(&engine.jig, &engine.limits, &mut problems);
    assert!(problems.is_empty(), "record sizing must leave the driver's check to store admission: {problems:?}");
}

#[test]
fn a_read_bound_set_to_the_refusals_number_starts() {
    const TRANSCRIPT: u32 = 1_000_000;
    // With a smaller agent turn the page drives the requirement; with a turn
    // as large as the transcript the decoded row drives it instead.
    for turn in [fixtures::startup_agent().limits.bodies.turn_body, TRANSCRIPT] {
        let mut agent = fixtures::startup_agent();
        agent.limits.bodies.turn_body = turn;
        let mut document = transcript_document(TRANSCRIPT);
        if turn == TRANSCRIPT {
            let previous = u64::from(fixtures::startup_agent().limits.bodies.turn_body);
            let mut text = core::str::from_utf8(&document).unwrap().to_string();
            for (key, before, after) in [
                ("hub.turn_bytes", previous, u64::from(turn)),
                ("core.resume_bytes", previous + 4, u64::from(turn) + 4),
                ("hub.transcript_bytes", (previous + 4) * 4, (u64::from(turn) + 4) * 4),
                ("hub.turn_queue_bytes", previous * 8, u64::from(turn) * 8),
                ("root.journal.transcript_bytes", previous + 4, u64::from(turn) + 4),
            ] {
                let needle = format!("\"{key}\":{before}");
                assert!(text.contains(&needle));
                text = text.replace(&needle, &format!("\"{key}\":{after}"));
            }
            document = text.into_bytes().into_boxed_slice();
        }
        let store = crate::store_limits(&document, &config::Directories::working(), Some(&agent)).unwrap().store;
        let page = u64::from(store.store.page.bytes);
        let map = u64::from(store.store.key) + u64::from(store.store.value);
        let frame = u64::from(jig_protocol_store::sizes::frame(&store.core, TRANSCRIPT).unwrap());
        let decoded = u64::from(turn)
            + u64::try_from(size_of::<jig_domain_core::reads::Row>()).unwrap()
            + u64::try_from(size_of::<jig_domain_core::Record>()).unwrap();
        assert!(frame > map);
        assert_eq!(page, frame);
        assert!(u64::from(store.store.chunk) < page);
        assert_eq!(decoded > page, turn == TRANSCRIPT);
        let required = page.max(decoded);
        // A read with no carrier room exercises both causes together; the
        // same complete figure must be named there and at one byte short.
        for read in [u64::from(turn), required - 1, required] {
            let text = core::str::from_utf8(&document).unwrap();
            let needle = format!("\"root.loads.read_bytes\":{}", store.core.load_bytes);
            assert_eq!(text.matches(&needle).count(), 1);
            let document = text.replace(&needle, &format!("\"root.loads.read_bytes\":{read}"));
            let started = crate::startup(
                document.as_bytes(),
                &config::Directories::working(),
                &fixtures::files(),
                &agent,
                &fixtures::seeds(),
                None,
                fixtures::worker_runtime(),
                fixtures::forge_runtime(),
            );
            if read == required {
                started.expect("the refusal's complete figure starts");
            } else {
                let refused = started.err().expect("short read refuses startup");
                assert_eq!(refused.problems.len(), 1);
                let expected = format!(
                    "application.limits.overrides.root.loads.read_bytes: {read} bytes, short of a read's room, {required} bytes\n"
                );
                assert_eq!(lines(&refused).as_ref(), expected.as_bytes());
            }
        }
    }
}

#[test]
fn a_map_driven_page_needs_no_larger_read_bound() {
    let document = fixtures::document();
    let store = crate::store_limits(&document, &config::Directories::working(), Some(&fixtures::startup_agent()))
        .unwrap()
        .store;
    let map = store.store.key.checked_add(store.store.value).unwrap();
    let frame =
        jig_protocol_store::sizes::frame(&store.core, fixtures::startup_agent().limits.bodies.turn_body).unwrap();
    assert!(map > frame);
    assert_eq!(store.store.page.bytes, map);
    assert!(store.store.chunk >= store.store.page.bytes);
    assert_eq!(store.core.load_bytes, 131_072);
    assert!(store.core.load_bytes < u64::from(store.store.page.bytes));
    crate::startup(
        &document,
        &config::Directories::working(),
        &fixtures::files(),
        &fixtures::startup_agent(),
        &fixtures::seeds(),
        None,
        fixtures::worker_runtime(),
        fixtures::forge_runtime(),
    )
    .expect("the chunk covers the map-driven page without raising the read bound");
}

#[test]
fn startup_names_a_whole_file_bound_below_the_manifest() {
    let found = crate::startup::checks::store_refusal(crate::store::Refusal::Map(
        jig_protocol_store::sizes::SizeRefusal::Whole { needs: 5464, has: 5463 },
    ));
    assert_eq!(
        lines(&found).as_ref(),
        b"engine.limits.store: the store's map whole-file read, 5464 bytes, past its bound of 5463 bytes\n"
    );
}

#[test]
fn startup_checks_the_effective_codec_envelope_before_generic_sizing() {
    let jig = config::profile(config::Profile::Small);
    let mut own = crate::limits::profile(crate::limits::Profile::Small);
    // Parsed core limits remain valid. Deliberately corrupt the effective root
    // answer bound to exercise this check independently of parsed admission;
    // journal capacity no longer widens the configured answer bound (R3-114).
    own.root.journal.transcript_bytes = u32::MAX;
    let mut root = crate::limits::engine_candidate(&jig, &own);
    root.call_answer_bytes = u32::MAX;
    let expected = config::check_effective_limits(&temper_engine_domain::core_limits(&root))
        .expect_err("effective answer exceeds the codec");
    let mut problems = Problems::empty();
    crate::startup::checks::engine_envelope(&root, &mut problems);
    assert_eq!(problems.problems.len(), 1);
    let message = expected.codec_message().expect("named codec diagnostic");
    let mut expected_line = skein_lib::Writer::new(b"engine.".len() + message.len() + b"\n".len());
    expected_line.put(b"engine.").unwrap();
    expected_line.put(&message).unwrap();
    expected_line.put(b"\n").unwrap();
    assert_eq!(lines(&problems), expected_line.finish());
}

#[test]
fn startup_checks_a_single_policy_row_at_its_bound_and_one_byte_past_it() {
    let jig = config::profile(config::Profile::Small);
    let own = crate::limits::profile(crate::limits::Profile::Small);
    let mut root = crate::limits::engine_candidate(&jig, &own);
    let bound = temper_engine_domain::policy_row_bound(&root).expect("policy row bound");
    for (capacity, expected) in [
        (bound, b"".as_slice()),
        (
            bound - 1,
            b"application.limits.overrides.root.journal.transcript_bytes: must agree with engine.limits.core.policy_bytes plus its policy row carrier\n",
        ),
    ] {
        root.journal.transcript_bytes = u32::try_from(capacity).unwrap();
        let mut problems = Problems::empty();
        crate::startup::checks::engine_envelope(&root, &mut problems);
        assert_eq!(lines(&problems).as_ref(), expected);
    }
}

#[test]
fn startup_refuses_policy_row_arithmetic_overflow_by_name() {
    let jig = config::profile(config::Profile::Small);
    let own = crate::limits::profile(crate::limits::Profile::Small);
    let mut root = crate::limits::engine_candidate(&jig, &own);
    root.policy_bytes = u64::MAX;
    assert_eq!(temper_engine_domain::policy_row_bound(&root), None);
    let mut problems = Problems::empty();
    crate::startup::checks::engine_envelope(&root, &mut problems);
    assert!(lines(&problems).ends_with(
        b"application.limits.overrides.root.journal.transcript_bytes: must agree with engine.limits.core.policy_bytes plus its policy row carrier\n"
    ));
}

#[test]
fn startup_names_the_policy_row_before_its_general_refusal() {
    let document = fixtures::with(
        &fixtures::document(),
        b"application.limits.overrides",
        br#"{"root.journal.transcript_bytes":1}"#,
    );
    let found = crate::store_limits(&document, &config::Directories::working(), Some(&fixtures::startup_agent()))
        .expect_err("short policy row");
    assert_eq!(
        lines(&found).as_ref(),
        b"application.limits.overrides.root.journal.transcript_bytes: must agree with engine.limits.core.policy_bytes plus its policy row carrier\n"
    );
}

#[test]
fn the_template_shaped_demo_reads_into_its_public_fields() {
    let (members, problems) = application(&fixtures::demo_application(), &fixtures::engine().jig);
    assert_eq!(&*lines(&problems), b"");
    let demo = members.unwrap().demo.unwrap();
    assert_eq!(demo.provider, 3);
    assert!(demo.key_file.ends_with(b"llm.key"));
}
#[test]
fn a_null_demo_reads_as_none() {
    let (members, problems) = application(fixtures::APPLICATION, &fixtures::engine().jig);
    assert_eq!(&*lines(&problems), b"");
    assert!(members.unwrap().demo.is_none());
}
#[test]
fn an_unanswerable_demo_is_refused_at_its_field() {
    use temper_client_protocol_wire::v1::CEILINGS;
    let input = fixtures::demo_application();
    let input = core::str::from_utf8(&input).unwrap();
    for (row, from, to, expected) in [
        (
            "cue count at bound",
            "\"asked\":[",
            ["\"asked\":[", &"{\"cue\":\"extra\",\"turns\":[\"x\"]},".repeat(5)].concat(),
            "",
        ),
        (
            "cue count past bound",
            "\"asked\":[",
            ["\"asked\":[", &"{\"cue\":\"extra\",\"turns\":[\"x\"]},".repeat(6)].concat(),
            "application.demo.llm.asked: past its bound\n",
        ),
        (
            "cue at bound",
            "Add CSV export to widget",
            "x".repeat(usize::try_from(CEILINGS.playground_answer_cues_item).unwrap()),
            "",
        ),
        (
            "cue past bound",
            "Add CSV export to widget",
            "x".repeat(usize::try_from(CEILINGS.playground_answer_cues_item + 1).unwrap()),
            "application.demo.llm.asked[0].cue: past its bound\n",
        ),
    ] {
        let input = input.replace(from, &to);
        let (members, problems) = application(input.as_bytes(), &fixtures::engine().jig);
        assert_eq!(members.is_some(), expected.is_empty(), "{row}");
        assert_eq!(&*lines(&problems), expected.as_bytes(), "{row}");
    }
}

#[test]
fn the_clients_listener_is_read_from_the_application_member() {
    let jig = fixtures::engine().jig;
    let (members, problems) = application(fixtures::APPLICATION, &jig);
    assert_eq!(&*lines(&problems), b"", "example accepted");
    let config::Address::Addr(skein_io::Addr::Ip(listener)) = members.unwrap().clients.listener else {
        panic!("a socket address")
    };
    assert_eq!(listener, "127.0.0.1:7100".parse::<core::net::SocketAddr>().unwrap());
}
#[test]
fn a_missing_clients_member_is_named_by_its_path() {
    let jig = fixtures::engine().jig;
    let input = core::str::from_utf8(fixtures::APPLICATION).unwrap();
    let input = input
        .replace("\"clients\":{\"listener\":{\"socket\":\"127.0.0.1:7100\"},\"public\":null,\"bundle\":null},", "");
    let (members, problems) = application(input.as_bytes(), &jig);
    assert!(members.is_none());
    assert_eq!(&*lines(&problems), b"application.clients: missing\n");
}
#[test]
fn an_in_process_clients_listener_is_refused_as_a_form() {
    let jig = fixtures::engine().jig;
    let input = core::str::from_utf8(fixtures::APPLICATION).unwrap();
    let input = input.replace("\"socket\":\"127.0.0.1:7100\"", "\"in_process\":\"clients\"");
    let (members, problems) = application(input.as_bytes(), &jig);
    assert!(members.is_none());
    assert_eq!(&*lines(&problems), b"application.clients.listener: invalid form\n");
}

fn application(input: &[u8], jig: &config::Engine) -> (Option<application::Members>, Problems) {
    let document =
        config::document(input, &config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 }).unwrap();
    let mut problems = Problems::empty();
    let members = application::parse(&document, 0, jig, &config::Directories::working(), &mut problems);
    (members, problems)
}
#[test]
fn the_standalone_example_reads_into_its_members() {
    let jig = fixtures::engine().jig;
    let (members, problems) = application(fixtures::APPLICATION, &jig);
    assert_eq!(&*lines(&problems), b"", "example accepted");
    let members = members.unwrap();
    assert_eq!(members.workers[0].git.as_deref(), Some(b"/usr/bin/git".as_slice()));
    assert_eq!(members.charter, 1);
    assert_eq!(application::files(&members).len(), 0, "all private files");
    let resolved = application::resolve(&members, &fixtures::files(), &mut Problems::empty()).unwrap();
    assert!(resolved.webhooks.is_none());
    assert!(!fixtures::document().is_empty(), "complete example bytes");
    let document = fixtures::parsed();
    assert!(!document.is_empty(), "full standalone example is JSON");
}

#[test]
fn the_example_engine_keeps_jigs_chat_and_goal_templates() {
    let jig = fixtures::engine().jig;
    let labels: Box<[u32]> = jig.templates.iter().map(|template| template.label).collect();
    assert_eq!(*labels, [1, 2], "jig's chat and goal templates, in order");
    assert_eq!(jig.templates[0].number, 1);
    assert_eq!(jig.templates[0].flags.ended_by, jig_domain_tasks::EndedBy::Requester);
    assert_eq!(jig.templates[1].flags.ended_by, jig_domain_tasks::EndedBy::Result);
}
#[test]
fn a_chats_member_is_refused_as_unknown_naming_its_path() {
    let input = fixtures::document();
    let input = core::str::from_utf8(&input).unwrap().replacen("\"engine\":{", "\"engine\":{\"chats\":[],", 1);
    let problems = crate::named(input.as_bytes(), &jig_protocol_config::Directories::working()).unwrap_err();
    assert_eq!(&*lines(&problems), b"engine.chats: unknown member\n");
}
fn refused(input: &[u8]) -> Box<[u8]> {
    let (members, problems) = application(input, &fixtures::engine().jig);
    assert!(members.is_none());
    lines(&problems)
}
#[test]
fn an_unknown_profile_is_refused_naming_its_path() {
    let input = core::str::from_utf8(fixtures::APPLICATION).unwrap().replace("\"small\"", "\"large\"");
    assert_eq!(&*refused(input.as_bytes()), b"application.limits.profile: the only profile is \"small\"\n");
}
#[test]
#[expect(clippy::disallowed_macros, reason = "ordinary tests format input and diagnostics")]
fn an_override_by_dotted_name_is_set_or_refused() {
    let jig = fixtures::engine().jig;
    let input = core::str::from_utf8(fixtures::APPLICATION).unwrap();
    for (override_value, expected) in [
        (
            "\"overrides\":{\"forge.repositories\":3,\"forges.log_origins\":3,\"forge_component.log_origins\":7,\"forge_component.log_poll_ms\":7123,\"forge_component.log_catch_up_ms\":9321,\"forge_component.log_pages\":13,\"forge_component.delivery_bytes\":12345}",
            "",
        ),
        (
            "\"overrides\":{\"forge.nothing\":3}",
            "application.limits.overrides.forge.nothing: no limit of temper's has this name\n",
        ),
        (
            "\"overrides\":{\"checkout.name_bytes\":4294967296}",
            "application.limits.overrides.checkout.name_bytes: outside its type\n",
        ),
    ] {
        let (members, problems) = application(input.replace("\"overrides\":{}", override_value).as_bytes(), &jig);
        assert_eq!(&*lines(&problems), expected.as_bytes(), "row {override_value}");
        if let Some(members) = members {
            assert_eq!(members.limits.forge.repositories, 3, "set {override_value}");
            assert_eq!((members.limits.forges.log_origins, members.limits.forge_component.log_origins), (3, 7));
            assert_eq!(members.limits.forge_component.log_poll, skein_lib::Duration::from_millis(7123));
            assert_eq!(members.limits.forge_component.log_catch_up, skein_lib::Duration::from_millis(9321));
            assert_eq!(members.limits.forge_component.log_pages, 13);
            assert_eq!(members.limits.forge_component.delivery_bytes, 12_345);
        }
    }
    for (name, value) in [
        ("forges.log_origins", u64::from(u32::MAX) + 1),
        ("forge_component.log_origins", u64::from(u32::MAX) + 1),
        ("forge_component.log_poll_ms", u64::MAX / 1_000_000 + 1),
        ("forge_component.log_catch_up_ms", u64::MAX / 1_000_000 + 1),
        ("forge_component.log_pages", u64::from(u32::MAX) + 1),
        ("forge_component.delivery_bytes", u64::from(u32::MAX) + 1),
    ] {
        let override_value = format!("\"overrides\":{{\"{name}\":{value}}}");
        let (_, problems) = application(input.replace("\"overrides\":{}", &override_value).as_bytes(), &jig);
        assert_eq!(
            lines(&problems).as_ref(),
            format!("application.limits.overrides.{name}: outside its type\n").as_bytes()
        );
    }
}
fn application_with_landing_requirements() -> Box<[u8]> {
    let input = fixtures::with(
        fixtures::APPLICATION,
        b"landing.projects[0].rules[0].gates",
        br#"[{"number":3,"blocking":true,"freshness":"exact"}]"#,
    );
    fixtures::with(&input, b"landing.projects[0].rules[0].approvals", br#"[{"role":0,"people":1,"freshness":"clean"}]"#)
}
#[test]
fn a_landing_rule_reads_jigs_pattern_form() {
    let jig = fixtures::engine().jig;
    let (members, problems) = application(&application_with_landing_requirements(), &jig);
    assert_eq!(&*lines(&problems), b"", "landing accepted");
    let members = members.unwrap();
    let rule = &members.landing_rules.projects.get(&1).unwrap()[0];
    assert_eq!(rule.pattern.segments[0].as_ref(), b"forge");
    assert_eq!(rule.pattern.last, jig_domain_authority::Last::Open(b"main".as_slice().into()));
    assert_eq!(rule.gates[0].number, 3);
    assert_eq!(rule.approvals[0].role, 0);
    assert_eq!(rule.approvals[0].freshness, temper_engine_domain::Freshness::Clean);
}
#[test]
fn problems_print_one_line_each_with_their_path() {
    let mut problems = Problems::empty();
    let forge = Path::root().member(b"application").member(b"forges").child(Segment::Index(0));
    problems.push(forge.member(b"user"), What::Jig(config::What::Missing));
    problems.push(Path::root().member(b"application").member(b"webhooks"), What::NoWebhooks);
    assert_eq!(
        &*lines(&problems),
        b"application.forges[0].user: missing\napplication.webhooks: needs application.webhooks\n".as_slice()
    );
    for _ in 0..config::PROBLEMS {
        problems.push(Path::root(), What::NoWebhooks);
    }
    assert_eq!(problems.past, 2);
    assert!(lines(&problems).ends_with(b"and 2 more\n"), "capped diagnostics name overflow");
}
#[test]
fn jig_refusals_print_words() {
    for (what, expected) in [
        (config::What::Json, b"at: invalid JSON\n".as_slice()),
        (config::What::Unknown, b"at: unknown member\n"),
        (config::What::Missing, b"at: missing\n"),
        (config::What::Duplicate, b"at: duplicate\n"),
        (config::What::Kind, b"at: wrong value kind\n"),
        (config::What::Bound, b"at: past its bound\n"),
        (config::What::Form, b"at: invalid form\n"),
        (config::What::Unique, b"at: must be unique\n"),
        (config::What::Refers, b"at: unknown reference\n"),
        (config::What::Plaintext, b"at: plaintext requires loopback or in-process\n"),
        (config::What::Digest, b"at: invalid digest\n"),
        (config::What::Reserve { charter: 7 }, b"at: reserve does not fit charter 7\n"),
        (config::What::Budget { charter: 7 }, b"at: budget does not fit charter 7\n"),
        (config::What::Grace { template: REFUSED_TEMPLATE }, b"at: grace does not fit template 7\n"),
        (
            config::What::Operations { operations: 4, delegates: 4, goals: 8 },
            b"at: operations do not fit compound requests: operations 4, delegates 4, goals 8\n",
        ),
        (config::What::Message, b"at: message bounds do not agree\n"),
        (config::What::Silence, b"at: heartbeat does not fit silence bound\n"),
        (config::What::StopBound, b"at: stop bound does not fit grace\n"),
        (config::What::Override, b"at: invalid override\n"),
        (config::What::Unreadable, b"at: not read\n"),
        (config::What::Relative, b"at: relative path must be non-empty with no .. component\n"),
    ] {
        let mut problems = Problems::empty();
        problems.push(Path::root().member(b"at"), What::Jig(what));
        assert_eq!(&*lines(&problems), expected, "row {what:?}");
    }
}
#[test]
fn members_and_references_are_checked() {
    let jig = fixtures::engine().jig;
    let input = application_with_landing_requirements();
    let input = core::str::from_utf8(&input).unwrap();
    for (from, to, expected) in [
        ("\"charter\":1", "\"charter\":3", "application.charter: unknown reference\n"),
        ("\"role\":0", "\"role\":1", "application.landing.projects[0].rules[0].approvals[0].role: unknown reference\n"),
        ("\"project\":1", "\"project\":2", "application.landing.projects[0].project: unknown reference\n"),
        ("\"git\":\"/usr/bin/git\"", "\"git\":\"git\"", "application.workers[0].git: must be an absolute path\n"),
        ("\"git\":\"/usr/bin/git\"", "\"git\":null", ""),
    ] {
        let (members, problems) = application(input.replace(from, to).as_bytes(), &jig);
        assert_eq!(members.is_some(), expected.is_empty(), "row {from}: {to}");
        assert_eq!(&*lines(&problems), expected.as_bytes(), "row {from}");
        if expected.is_empty() {
            assert_eq!(members.unwrap().workers[0].git, None);
        }
    }
}
#[test]
fn named_lists_each_file_with_its_bound_and_privacy() {
    let named = crate::named(&fixtures::document(), &jig_protocol_config::Directories::working()).unwrap();
    assert_eq!(named.files.len(), 3, "all files listed");
    for (name, path, max, private) in [
        (b"secrets/worker-a.digest".as_slice(), b"engine.hosts.workers[0].digest".as_slice(), 65, true),
        (b"worker-secret", b"workers[0].secret", 257, true),
        (fixtures::AGENT_JSON, b"workers[0].agent.configuration", 1_048_576, false),
    ] {
        let file = named.files.iter().find(|file| file.path.as_ref() == name).expect("listed file");
        assert_eq!(file.max, max, "bound {name:?}");
        assert_eq!(file.private, private, "privacy {name:?}");
        let mut problems = Problems::empty();
        problems.push(file.at.clone(), What::Jig(config::What::Missing));
        let expected = [path, b": missing\n"].concat();
        assert_eq!(&*lines(&problems), expected.as_slice(), "path {name:?}");
    }
}
#[test]
fn named_returns_the_worker_root_and_the_agent_configuration() {
    let named = crate::named(&fixtures::document(), &jig_protocol_config::Directories::working()).unwrap();
    assert_eq!(named.deployment.as_ref(), b"state");
    assert_eq!(named.worker.as_ref().expect("standalone worker").root.as_ref(), b"worker-root");
    assert_eq!(named.worker.as_ref().expect("standalone worker").agent.as_ref(), fixtures::AGENT_JSON);
}
#[test]
fn a_file_named_but_not_given_refuses_the_start_naming_it() {
    let mut files = fixtures::files();
    files.entries = fixtures::without(files.entries, b"worker-secret");
    let refused = crate::startup(
        &fixtures::document(),
        &jig_protocol_config::Directories::working(),
        &files,
        &fixtures::startup_agent(),
        &fixtures::seeds(),
        None,
        fixtures::worker_runtime(),
        fixtures::forge_runtime(),
    );
    let Err(problems) = refused else { panic!("a missing token refuses the start") };
    assert_eq!(&*lines(&problems), b"workers[0].secret: not read\n".as_slice());
}
#[test]
fn jigs_problems_keep_their_paths_under_engine() {
    let input = fixtures::document();
    let input = core::str::from_utf8(&input)
        .unwrap()
        .replace("\"memory_bytes\": 1152921504606846976,", "\"memory_bytes\": \"bad\",");
    let problems = crate::named(input.as_bytes(), &jig_protocol_config::Directories::working()).unwrap_err();
    assert_eq!(&*lines(&problems), b"engine.memory_bytes: wrong value kind\n".as_slice());
}
#[test]
fn a_zero_byte_budget_refuses_the_start_naming_its_member() {
    let input = fixtures::document();
    let input = core::str::from_utf8(&input).unwrap();
    let needle = "\"memory_bytes\": 1152921504606846976,";
    assert_eq!(input.matches(needle).count(), 1);
    let zero = input.replace(needle, "\"memory_bytes\": 0,");
    let problems = crate::named(zero.as_bytes(), &config::Directories::working()).unwrap_err();
    assert_eq!(lines(&problems).as_ref(), b"engine.memory_bytes: past its bound\n");
    let one = input.replace(needle, "\"memory_bytes\": 1,");
    crate::named(one.as_bytes(), &config::Directories::working()).expect("a positive byte budget is valid");
}

#[test]
fn startup_makes_the_one_process_from_the_example() {
    let mut service = match crate::startup(
        &fixtures::document(),
        &jig_protocol_config::Directories::working(),
        &fixtures::files(),
        &fixtures::startup_agent(),
        &fixtures::seeds(),
        None,
        fixtures::worker_runtime(),
        fixtures::forge_runtime(),
    ) {
        Ok(service) => service,
        Err(problems) => panic!("startup: {}", core::str::from_utf8(&lines(&problems)).unwrap()),
    };
    service.engine.as_mut().expect("engine").store = fixtures::memory_store();
    let mut stand = fixtures::system::SystemStand { service, kernel: fixtures::Kernel::new() };
    fixtures::until_hosting(&mut stand);
    let (channel, hello) = fixtures::hello(&stand.service).expect("worker linked through bytes");
    assert_eq!(channel, skein_lib::Token::new(1));
    assert_eq!(hello.slots, 1);
    for _ in 0_u16..256 {
        if stand.service.engine.as_mut().expect("engine").domain.ready() {
            break;
        }
        stand.turn();
    }
    assert!(stand.service.engine.as_ref().expect("engine").domain.ready(), "engine restored its store");
    assert_eq!(stand.service.end(), None);
}
#[test]
fn a_workers_missing_or_oversized_secret_keeps_its_absolute_path() {
    let document = fixtures::document();
    let agent = fixtures::startup_agent();
    let seeds = fixtures::seeds();
    for (row, bytes, expected) in [
        ("missing", None, b"workers[0].secret: not read\n".as_slice()),
        ("past", Some(Box::from([b'x'; 257])), b"workers[0].secret: past its bound\n"),
    ] {
        let mut files = fixtures::files();
        files.entries = fixtures::without(files.entries, b"worker-secret");
        if let Some(bytes) = bytes {
            files.entries.push((b"worker-secret".as_slice().into(), skein_lib::Secret::new(bytes))).unwrap();
        }
        let found = match crate::startup(
            &document,
            &jig_protocol_config::Directories::working(),
            &files,
            &agent,
            &seeds,
            None,
            fixtures::worker_runtime(),
            fixtures::forge_runtime(),
        ) {
            Ok(service) => {
                drop(service);
                Problems::empty()
            }
            Err(problems) => problems,
        };
        assert_eq!(&*lines(&found), expected, "row {row}");
    }
}
#[test]
fn a_worker_secret_at_its_bound_excludes_its_final_newline() {
    let mut worker = fixtures::worker_settings();
    worker.jig.limits.link.secret = 4;
    let mut entries = List::with_capacity(1);
    entries.push((b"worker-secret".as_slice().into(), skein_lib::Secret::new(b"four\n".as_slice().into()))).unwrap();
    let made = crate::Worker::new(
        &worker,
        &fixtures::worker_agent(),
        &fixtures::worker_limits(),
        &settings::Files { entries },
        fixtures::worker_runtime(),
        7,
    );
    assert!(made.is_ok(), "four secret bytes plus their final newline fit four");
}

#[test]
fn a_missing_setup_or_demo_is_refused_naming_its_path() {
    for (member, expected) in [
        ("\"machine\":{},", b"application.machine: missing\n".as_slice()),
        ("\"demo\":null,", b"application.demo: missing\n".as_slice()),
    ] {
        let input = core::str::from_utf8(fixtures::APPLICATION).unwrap().replace(member, "");
        let (members, problems) = application(input.as_bytes(), &fixtures::engine().jig);
        assert!(members.is_none());
        assert_eq!(&*lines(&problems), expected);
    }
}

#[test]
fn a_setup_or_box_override_is_handed_on_with_its_value() {
    for (override_value, repositories) in [("", 2_u64), (",\"forge.client.repositories\":3", 3)] {
        let input = core::str::from_utf8(fixtures::APPLICATION).unwrap().replace(
            "\"overrides\":{}",
            &["\"overrides\":{\"setup.orgs\":2,\"box.steps\":3", override_value, "}"].concat(),
        );
        let (members, problems) = application(input.as_bytes(), &fixtures::engine().jig);
        assert_eq!(&*lines(&problems), b"");
        assert_eq!(
            members.unwrap().overrides.as_ref(),
            &[
                (b"setup.orgs".as_slice().into(), 2),
                (b"box.steps".as_slice().into(), 3),
                (b"forge.client.repositories".as_slice().into(), repositories),
            ]
        );
    }
}

// protocol/sign-in.md, 9: a configured provider is admitted, and startup reads its key and
// client secret as private files, refusing either one missing by its path.
#[test]
fn a_clients_provider_key_and_secret_are_private_files_named_by_path() {
    let document = core::str::from_utf8(&fixtures::document()).unwrap().replace(
        "\"clients\": {",
        r#""clients": {"provider":{"key":"provider.pem","clients":[{"id":"app","secret":"app.secret","redirect_uri":"https://app.example.com/cb"}],"addresses":[]},"#,
    );
    let named = crate::named(document.as_bytes(), &jig_protocol_config::Directories::working()).unwrap();
    for (name, expected) in [
        (b"provider.pem".as_slice(), b"engine.clients.provider.key: missing\n".as_slice()),
        (b"app.secret", b"engine.clients.provider.clients[0].secret: missing\n"),
    ] {
        let file = named.files.iter().find(|file| file.path.as_ref() == name).unwrap();
        assert!(file.private);
        let mut problems = Problems::empty();
        problems.push(file.at.clone(), What::Jig(config::What::Missing));
        assert_eq!(&*lines(&problems), expected);
    }
}
#[test]
fn null_webhook_members_are_read_as_none() {
    let (members, problems) = application(fixtures::APPLICATION, &fixtures::engine().jig);
    assert_eq!(&*lines(&problems), b"");
    let members = members.unwrap();
    assert!(members.webhooks.is_none());
}
#[test]
fn a_missing_webhooks_member_is_refused_by_path() {
    let input = core::str::from_utf8(fixtures::APPLICATION).unwrap().replace("\"webhooks\":null,", "");
    assert_eq!(&*refused(input.as_bytes()), b"application.webhooks: missing\n");
}
#[test]
fn an_in_process_webhook_listener_is_refused() {
    let input =
        fixtures::with(&fixtures::document(), b"application.webhooks", br#"{"listener":{"in_process":"hooks"}}"#);
    assert_eq!(
        &*lines(&crate::named(&input, &jig_protocol_config::Directories::working()).unwrap_err()),
        b"application.webhooks.listener: invalid form\n"
    );
}
#[test]
fn the_configured_bundle_is_named_for_the_binary() {
    let document = fixtures::with(&fixtures::document(), b"application.clients.bundle", br#""/bundle""#);
    let named = crate::named(&document, &jig_protocol_config::Directories::working()).expect("named");
    assert_eq!(named.bundle.as_deref(), Some(b"/bundle".as_slice()));
    assert_eq!(named.bundle_limits, crate::limits::Clients { bundle_files: 128, bundle_bytes: 16 << 20 });
}
#[test]
fn a_missing_bundle_is_named_by_its_path() {
    let input = core::str::from_utf8(fixtures::APPLICATION).unwrap().replace(",\"bundle\":null}", "}");
    assert_eq!(&*refused(input.as_bytes()), b"application.clients.bundle: missing\n");
}
#[test]
fn a_webhook_listener_with_a_bundle_is_refused() {
    let document = fixtures::with(
        &fixtures::webhook_document(),
        b"application.webhooks",
        br#"{"listener":{"socket":"127.0.0.1:7200"},"bundle":null}"#,
    );
    assert_eq!(
        &*lines(&crate::named(&document, &jig_protocol_config::Directories::working()).unwrap_err()),
        b"application.webhooks.bundle: unknown member\n"
    );
}

#[test]
fn an_old_agent_command_is_refused_with_its_fix() {
    let document =
        fixtures::with(&fixtures::document(), b"workers[0].agent.arguments", br#"["agent", "/c/agent.json"]"#);
    let refused = crate::named(&document, &config::Directories::working()).unwrap_err();
    assert_eq!(
        lines(&refused).as_ref(),
        b"workers[0].agent.arguments: \"agent\" is now \"internal\", \"agent\": write [\"internal\", \"agent\", PATH], PATH agent.configuration's\n"
    );
    let document =
        fixtures::with(&document, b"workers[0].agent.arguments", br#"["internal", "agent", "/c/agent.json"]"#);
    crate::named(&document, &config::Directories::working()).expect("the replacement agent command passes");
}

#[test]
fn paths_are_resolved_by_kind_and_absolute_only_members_refuse_relative_values() {
    let directories = config::Directories::given(b"/configuration", b"/data", b"/cache").unwrap();
    for (member, input, expected, refused) in [
        (b"application.workers[0].root".as_slice(), br#""root""#.as_slice(), b"/data/root".as_slice(), false),
        (b"application.workers[0].root", br#""/kept/root""#, b"/kept/root", false),
        (b"engine.deployment.directory", br#""state""#, b"/data/state", false),
        (b"engine.deployment.directory", br#""/kept/state""#, b"/kept/state", false),
        (b"workers[0].secret", br#""secret""#, b"/data/secret", false),
        (b"workers[0].secret", br#""/kept/secret""#, b"/kept/secret", false),
        (b"engine.hosts.workers[0].digest", br#""digest""#, b"/data/digest", false),
        (b"engine.hosts.workers[0].digest", br#""/kept/digest""#, b"/kept/digest", false),
        (b"application.workers[0].git", br#""git""#, b"", true),
        (b"application.clients.bundle", br#""bundle""#, b"", true),
        (b"application.clients.bundle", br#""/kept/bundle""#, b"/kept/bundle", false),
        (b"workers[0].agent.program", br#""temper""#, b"", true),
        (b"workers[0].agent.configuration", br#""agent.json""#, b"", true),
        (b"workers[0].agent.arguments", br#"["internal", "agent","agent.json"]"#, b"", true),
    ] {
        let document = fixtures::with(&fixtures::document(), member, input);
        let result = crate::named(&document, &directories);
        if refused {
            let lines = lines(&result.unwrap_err());
            let expected = [member, b": must be an absolute path\n"].concat();
            assert_eq!(lines.as_ref(), expected, "{}", core::str::from_utf8(member).expect("test path"));
        } else {
            let named = result.unwrap();
            let value = match member {
                b"application.workers[0].root" => &*named.worker.as_ref().expect("standalone worker").root,
                b"engine.deployment.directory" => &*named.deployment,
                b"application.clients.bundle" => named.bundle.as_deref().unwrap(),
                _ => &*named.files.iter().find(|file| file.path.as_ref() == expected).expect("resolved file key").path,
            };
            assert_eq!(value, expected);
        }
    }
}

#[test]
fn admission_model_words_use_tokens_and_missing_names_make_no_amount_claim() {
    use jig_domain_fleet::AgentShortfall;
    for (why, expected) in [
        (
            AgentShortfall::Model { charter: 7, model: 0 },
            "model: not in the agent's configuration under the same name\n",
        ),
        (
            AgentShortfall::Window { charter: 7, model: 0 },
            "model: needs a context window of 8192 tokens; the agent offers 1024 tokens\n",
        ),
        (
            AgentShortfall::Output { charter: 7, model: 0 },
            "model: needs 8192 output tokens; the agent offers 1024 tokens\n",
        ),
    ] {
        let mut problems = Problems::empty();
        problems.push(Path::root().member(b"model"), What::Admission { why, offered: 1024, required: 8192 });
        assert_eq!(&*lines(&problems), expected.as_bytes());
    }
}

#[test]
fn a_store_reader_derives_serves_limits_or_their_problem_path() {
    let document = fixtures::document();
    let directories = config::Directories::working();
    let service = crate::startup(
        &document,
        &directories,
        &fixtures::files(),
        &fixtures::startup_agent(),
        &fixtures::seeds(),
        None,
        fixtures::worker_runtime(),
        fixtures::forge_runtime(),
    )
    .unwrap();
    assert_eq!(
        service.engine.as_ref().unwrap().store.opening_limits().core,
        temper_engine_domain::core_limits(&service.engine.as_ref().unwrap().env.limits),
    );
    let refused = core::str::from_utf8(&document).unwrap().replacen(
        "\"overrides\":{\"checkout.name_bytes\"",
        "\"overrides\":{\"root.loads.rows\":0,\"checkout.name_bytes\"",
        1,
    );
    for (name, input, expected) in [
        (
            "serve",
            document.as_ref(),
            Ok((
                service.engine.as_ref().expect("engine").store.opening_limits(),
                service.engine.as_ref().expect("engine").env.limits.forge,
            )),
        ),
        ("limits", refused.as_bytes(), Err(b"engine.limits: past its bound\n".as_slice())),
    ] {
        let found = crate::store_limits(input, &directories, Some(&fixtures::startup_agent()));
        let found = found.map(|limits| (limits.store, limits.forge)).map_err(|problems| lines(&problems));
        assert_eq!(found.as_ref().map_err(AsRef::as_ref), expected.as_ref().map_err(AsRef::as_ref), "{name}");
    }
}

#[test]
fn an_engine_alone_prepares_without_a_worker_or_its_files_and_agent() {
    let input = fixtures::engine_alone_document();
    let named = crate::named(&input, &config::Directories::working()).unwrap();
    assert!(named.worker.is_none());
    let mut files = fixtures::files();
    for path in [b"worker-secret".as_slice(), fixtures::AGENT_JSON] {
        files.entries = fixtures::without(files.entries, path);
    }
    let service = crate::startup::with_setup(
        &input,
        &config::Directories::working(),
        &files,
        None,
        &fixtures::seeds(),
        None,
        None,
        fixtures::forge_runtime(),
        None,
        None,
    )
    .unwrap();
    assert!(service.worker.is_none());
}

#[test]
fn a_checked_worker_alone_has_its_shape_and_refuses_engine_application_members() {
    let input = fixtures::worker_alone_document();
    assert_eq!(crate::startup::shape(&input, &config::Directories::working()), Ok(crate::startup::Shape::WorkerAlone));
    let text = core::str::from_utf8(&input).unwrap().replace("\"application\":{", "\"application\":{\"forges\":[],");
    let found = crate::startup::shape(text.as_bytes(), &config::Directories::working()).unwrap_err();
    assert_eq!(found.problems.len(), 1);
    assert_eq!(found.problems.get(0).unwrap().path, Path::root().member(b"application").member(b"forges"));
    assert_eq!(found.problems.get(0).unwrap().what, What::WorkerAlone);
}

#[test]
fn a_worker_alone_checks_its_application_paths_and_its_git_and_channel_limits() {
    let input = fixtures::worker_alone_document();
    for (member, value, path, what) in [
        (
            b"application.workers[0].root".as_slice(),
            b"12".as_slice(),
            Path::root().member(b"application").member(b"workers").child(Segment::Index(0)).member(b"root"),
            What::Jig(config::What::Kind),
        ),
        (
            b"application.workers[0].root",
            br#""../escape""#,
            Path::root().member(b"application").member(b"workers").child(Segment::Index(0)).member(b"root"),
            What::Jig(config::What::Relative),
        ),
        (
            b"application.workers[0].git",
            br#""git""#,
            Path::root().member(b"application").member(b"workers").child(Segment::Index(0)).member(b"git"),
            What::Absolute,
        ),
        (
            b"workers[0].agent.configuration",
            br#""temper""#,
            Path::root().member(b"workers").child(Segment::Index(0)).member(b"agent").member(b"configuration"),
            What::Absolute,
        ),
        (
            b"application.limits.overrides",
            br#"{"git_component.operations":1}"#,
            Path::root()
                .member(b"application")
                .member(b"limits")
                .member(b"overrides")
                .member(b"git_component.operations"),
            What::Against { other: b"checkout.workspaces" },
        ),
        (
            b"application.limits.overrides",
            br#"{"checkout.repositories":4294967295}"#,
            Path::root().member(b"application").member(b"limits").member(b"overrides").member(b"checkout.repositories"),
            What::Past {
                bytes: 4_294_967_295,
                bound: u64::from(temper_protocol_worker_wire::v1::CEILINGS.items_repositories),
            },
        ),
    ] {
        let input = fixtures::with(&input, member, value);
        let found = crate::startup::shape(&input, &config::Directories::working()).unwrap_err();
        assert_eq!(found.problems.len(), 1, "{}: {:?}", core::str::from_utf8(member).unwrap(), found);
        assert_eq!((found.problems.get(0).unwrap().path.clone(), found.problems.get(0).unwrap().what), (path, what));
    }
}

#[test]
fn a_worker_alone_with_the_owned_examples_worker_fits_one_ring() {
    // Linux's IORING_MAX_ENTRIES, the largest submission ring.
    const IORING_MAX_ENTRIES: u32 = 32_768;
    let input = fixtures::owned_worker_alone_document();
    let directories = config::Directories::working();
    let agent = fixtures::owned_worker_agent();
    let part = crate::startup::worker_part(&input, &directories).unwrap();
    let operations = crate::system::worker_alone_operations(
        &part.worker.jig.limits.io,
        crate::worker_settings::operations(&part.limits.git_component).unwrap(),
    )
    .unwrap();
    let mut files = settings::Files { entries: List::with_capacity(2) };
    for file in &part.files {
        files
            .entries
            .push((file.path.clone(), skein_lib::Secret::new(b"fixture-worker-secret".as_slice().into())))
            .unwrap();
    }
    let service = crate::startup::worker_alone(part, &files, &agent, fixtures::worker_runtime(), 7).unwrap();
    assert!(service.operations() <= IORING_MAX_ENTRIES);
    assert_eq!(service.operations(), operations, "the start's check and the ring agree");
}

#[test]
fn clients_public_keeps_the_parsed_origin_and_jig_name() {
    let jig = fixtures::engine().jig;
    let input = core::str::from_utf8(fixtures::APPLICATION).unwrap();
    let (members, problems) = application(input.as_bytes(), &jig);
    assert!(lines(&problems).is_empty());
    assert!(members.unwrap().clients.public.is_none());
    for (origin, scheme, host, port, name) in [
        ("https://temper.example.com", application::Scheme::Https, "temper.example.com", None, "temper.example.com"),
        (
            "https://temper.example.com:8443",
            application::Scheme::Https,
            "temper.example.com",
            Some(8443),
            "temper.example.com:8443",
        ),
        ("http://127.0.0.1:9000", application::Scheme::Http, "127.0.0.1", Some(9000), "127.0.0.1:9000"),
        ("http://localhost", application::Scheme::Http, "localhost", None, "localhost"),
        ("http://[::1]:9000", application::Scheme::Http, "[::1]", Some(9000), "[::1]:9000"),
    ] {
        let input = input.replace("\"public\":null", &format!("\"public\":\"{origin}\""));
        let (members, problems) = application(input.as_bytes(), &jig);
        assert!(lines(&problems).is_empty(), "{origin}: {:?}", lines(&problems));
        let public = members.unwrap().clients.public.unwrap();
        assert_eq!(
            (public.scheme, public.host.as_ref(), public.port, public.name.as_ref()),
            (scheme, host.as_bytes(), port, name.as_bytes())
        );
        assert_eq!(public.base().as_ref(), format!("{origin}/").as_bytes());
    }
}
#[test]
fn clients_public_refusals_name_the_public_member() {
    let jig = fixtures::engine().jig;
    let input = core::str::from_utf8(fixtures::APPLICATION).unwrap();
    let missing = input.replace("\"public\":null,", "");
    let (_, problems) = application(missing.as_bytes(), &jig);
    assert_eq!(lines(&problems).as_ref(), b"application.clients.public: missing: null for none\n");
    for origin in [
        "https://temper.example.com:443",
        "https://Temper.example.com",
        "https://temper.example.com/path",
        "https://temper.example.com/",
        "https://temper.example.com?query",
        "https://temper.example.com#fragment",
        "https://user@temper.example.com",
        "http://temper.example.com",
        "https://-bad.example.com",
        "https://bad-.example.com",
        "https://bad..example.com",
        "https://bad_label.example.com",
        "https://temper.example.com:",
        "https://temper.example.com:0",
        "https://temper.example.com:65536",
        "https://temper.example.com:+8443",
        "https://",
        "ftp://temper.example.com",
        "http://127.0.0.1:9000/path",
        "http://192.0.2.1:9000",
        "http://[::2]:9000",
        "https://127.0.0.1",
        "https://temper.example.com.",
    ] {
        let input = input.replace("\"public\":null", &format!("\"public\":\"{origin}\""));
        let (_, problems) = application(input.as_bytes(), &jig);
        assert_eq!(lines(&problems).as_ref(), b"application.clients.public: invalid form\n", "{origin}");
    }
    for value in ["true", "8", "{}"] {
        let input = input.replace("\"public\":null", &format!("\"public\":{value}"));
        let (_, problems) = application(input.as_bytes(), &jig);
        assert_eq!(lines(&problems).as_ref(), b"application.clients.public: wrong value kind\n", "{value}");
    }
}
#[test]
fn webhooks_public_is_an_unknown_member() {
    let input = core::str::from_utf8(fixtures::APPLICATION)
        .unwrap()
        .replace("\"webhooks\":null", "\"webhooks\":{\"listener\":{\"socket\":\"127.0.0.1:7200\"},\"public\":null}");
    let (_, problems) = application(input.as_bytes(), &fixtures::engine().jig);
    assert_eq!(lines(&problems).as_ref(), b"application.webhooks.public: unknown member\n");
}

#[test]
fn clients_public_bounds_dns_labels_and_the_host() {
    let jig = fixtures::engine().jig;
    let input = core::str::from_utf8(fixtures::APPLICATION).unwrap();
    let longest = format!("{}.{}.{}.{}", "a".repeat(63), "b".repeat(63), "c".repeat(63), "d".repeat(61));
    assert_eq!(longest.len(), 253);
    for (host, admitted) in [(longest.clone(), true), (format!("{longest}d"), false), ("a".repeat(64), false)] {
        let origin = format!("https://{host}");
        let input = input.replace("\"public\":null", &format!("\"public\":\"{origin}\""));
        let (members, problems) = application(input.as_bytes(), &jig);
        if admitted {
            assert!(lines(&problems).is_empty());
            assert_eq!(members.unwrap().clients.public.unwrap().host.as_ref(), host.as_bytes());
        } else {
            assert_eq!(lines(&problems).as_ref(), b"application.clients.public: invalid form\n");
        }
    }
}

fn turn_projects(project: u32) -> Box<[u8]> {
    format!(
        r#"[{{"project":{project},"rules":[],"by":"turn","gates":[{{"forge":1,"repository":2,"gates":[{{"name":"gate","workflow":"gate.yml","ignored":["docs"],"blocking":true}}]}}]}}]"#
    )
    .into_bytes()
    .into_boxed_slice()
}
fn turn_configuration(project: u32) -> (Option<application::Members>, Problems) {
    let jig = fixtures::engine().jig;
    let input = fixtures::with(fixtures::APPLICATION, b"landing.projects", &turn_projects(project));
    application(&input, &jig)
}

#[test]
fn turn_configuration_can_name_a_project_adopted_after_the_seed() {
    const ADOPTED: u32 = 2;
    let (members, problems) = turn_configuration(ADOPTED);
    assert!(problems.is_empty(), "{:?}", lines(&problems));
    let members = members.unwrap();
    let landing = members.landings.get(&ADOPTED).unwrap();
    assert_eq!(landing.by, temper_engine_domain::Way::Turn);
    assert_eq!(landing.repositories[0].gates[0].workflow.as_ref(), b"gate.yml");
    assert!(!members.landing_rules.projects.contains_key(&ADOPTED));
    let document = fixtures::with(&fixtures::document(), b"application.landing.projects", &turn_projects(ADOPTED));
    let started = crate::startup(
        &document,
        &config::Directories::working(),
        &fixtures::files(),
        &fixtures::startup_agent(),
        &fixtures::seeds(),
        None,
        fixtures::worker_runtime(),
        fixtures::forge_runtime(),
    );
    if let Err(problems) = started {
        panic!("restored turn startup: {}", core::str::from_utf8(&lines(&problems)).unwrap());
    }
}

#[test]
fn landing_configuration_reads_the_way_and_bounded_repository_gates() {
    let jig = fixtures::engine().jig;
    let (members, problems) = turn_configuration(1);
    assert!(problems.is_empty(), "{:?}", lines(&problems));
    let members = members.unwrap();
    let landing = members.landings.get(&1).unwrap();
    assert_eq!(landing.by, temper_engine_domain::Way::Turn);
    assert_eq!(landing.repositories[0].repository.repository, 2);
    let check = &landing.repositories[0].gates[0];
    assert_eq!(check.name.as_ref(), b"gate");
    assert_eq!(check.workflow.as_ref(), b"gate.yml");
    assert_eq!(check.ignored.as_ref(), &[Box::from(b"docs".as_slice())]);
    assert!(check.blocking);
    assert!(landing.repositories[0].tiers.is_empty());
    assert_eq!(
        landing.after,
        temper_engine_domain::AfterLandingCheck { tier_retries: 2, verification_bound_ms: 7_200_000 }
    );
    let (members, problems) = application(fixtures::APPLICATION, &jig);
    assert!(problems.is_empty());
    let members = members.unwrap();
    let landing = members.landings.get(&1).unwrap();
    assert_eq!(landing.by, temper_engine_domain::Way::Queue);
    assert!(landing.repositories.is_empty());
    assert_eq!(
        landing.after,
        temper_engine_domain::AfterLandingCheck { tier_retries: 2, verification_bound_ms: 7_200_000 }
    );
}

const TIER: &str = r#"{"name":"slow-1_A","workflow":"slow.yml","ignored":["docs","fixtures/data"],"artifact":"failures","format":"junit","branch":"release/next"}"#;

fn tier_projects(tiers: &str, settings: &str) -> Box<[u8]> {
    format!(
        r#"[{{"project":1,"rules":[],{settings}"gates":[{{"forge":1,"repository":2,"gates":[{{"name":"gate","workflow":"gate.yml","blocking":true}}],"tiers":{tiers}}}]}}]"#
    )
    .into_bytes()
    .into_boxed_slice()
}

#[test]
fn landing_configuration_reads_tiers_and_project_settings_by_queue_and_turn() {
    use temper_engine_domain::{AfterLandingCheck, TierCheck, Way};
    use temper_engine_domain_forge_tiers::Format;
    let longest = "a".repeat(32);
    let tiers =
        format!(r#"[{TIER},{{"name":"{longest}","workflow":"lines.yml","artifact":"{longest}","format":"lines"}}]"#);
    for (by, way) in [("queue", Way::Queue), ("turn", Way::Turn)] {
        let projects =
            tier_projects(&tiers, &format!(r#""by":"{by}","tier_retries":7,"verification_bound_ms":123456,"#));
        let input = fixtures::with(fixtures::APPLICATION, b"landing.projects", &projects);
        let (members, problems) = application(&input, &fixtures::engine().jig);
        assert!(problems.is_empty(), "{:?}", lines(&problems));
        let members = members.unwrap();
        let landing = members.landings.get(&1).unwrap();
        assert_eq!(landing.by, way);
        assert_eq!(landing.after, AfterLandingCheck { tier_retries: 7, verification_bound_ms: 123_456 });
        assert_eq!(landing.repositories[0].gates[0].name.as_ref(), b"gate");
        assert_eq!(
            landing.repositories[0].tiers.as_ref(),
            &[
                TierCheck {
                    name: Box::from(b"slow-1_A".as_slice()),
                    workflow: Box::from(b"slow.yml".as_slice()),
                    ignored: Box::new([Box::from(b"docs".as_slice()), Box::from(b"fixtures/data".as_slice())]),
                    artifact: Box::from(b"failures".as_slice()),
                    format: Format::Junit,
                    branch: Some(Box::from(b"release/next".as_slice())),
                },
                TierCheck {
                    name: longest.as_bytes().into(),
                    workflow: Box::from(b"lines.yml".as_slice()),
                    ignored: Box::new([]),
                    artifact: longest.as_bytes().into(),
                    format: Format::Lines,
                    branch: None,
                },
            ]
        );
        let document = fixtures::with(&fixtures::document(), b"application.landing.projects", &projects);
        if let Err(problems) = crate::startup(
            &document,
            &config::Directories::working(),
            &fixtures::files(),
            &fixtures::startup_agent(),
            &fixtures::seeds(),
            None,
            fixtures::worker_runtime(),
            fixtures::forge_runtime(),
        ) {
            panic!("configured tiers start: {}", core::str::from_utf8(&lines(&problems)).unwrap());
        }
    }
}

#[test]
fn landing_configuration_defaults_tier_settings_when_tiers_are_present() {
    let projects = tier_projects(&format!("[{TIER}]"), "");
    let input = fixtures::with(fixtures::APPLICATION, b"landing.projects", &projects);
    let (members, problems) = application(&input, &fixtures::engine().jig);
    assert!(problems.is_empty(), "{:?}", lines(&problems));
    let members = members.unwrap();
    let landing = members.landings.get(&1).unwrap();
    assert_eq!(landing.by, temper_engine_domain::Way::Queue);
    assert_eq!(landing.repositories[0].tiers.len(), 1);
    assert_eq!(
        landing.after,
        temper_engine_domain::AfterLandingCheck { tier_retries: 2, verification_bound_ms: 7_200_000 }
    );
}

#[test]
fn landing_configuration_tier_refusals_name_the_member() {
    let projects = tier_projects(&format!("[{TIER}]"), "");
    let input = fixtures::with(fixtures::APPLICATION, b"landing.projects", &projects);
    for (member, value, words) in [
        ("name", r#""""#, "invalid form"),
        ("name", r#""slow.tier""#, "invalid form"),
        ("name", r#""slöw""#, "invalid form"),
        ("name", r#""aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa""#, "past its bound"),
        ("name", r#""gate""#, "must be unique"),
        ("artifact", r#""""#, "invalid form"),
        ("artifact", r#""reports/failures""#, "invalid form"),
        ("artifact", r#""aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa""#, "past its bound"),
        ("format", r#""json""#, "invalid form"),
        ("branch", r#""""#, "invalid form"),
        ("branch", "null", "wrong value kind"),
    ] {
        let path = format!("landing.projects[0].gates[0].tiers[0].{member}");
        let changed = fixtures::with(&input, path.as_bytes(), value.as_bytes());
        let (members, problems) = application(&changed, &fixtures::engine().jig);
        assert!(members.is_none(), "{member}={value}");
        assert_eq!(lines(&problems).as_ref(), format!("application.{path}: {words}\n").as_bytes(), "{value}");
    }
}

#[test]
fn landing_configuration_tier_names_are_unique_and_the_count_is_bounded() {
    for (tiers, path, words) in [
        (format!("[{TIER},{TIER}]"), "tiers[1].name", "must be unique"),
        (format!("[{TIER},{TIER},{TIER}]"), "tiers", "past its bound"),
    ] {
        let projects = tier_projects(&tiers, "");
        let input = fixtures::with(fixtures::APPLICATION, b"landing.projects", &projects);
        let (members, problems) = application(&input, &fixtures::engine().jig);
        assert!(members.is_none());
        assert_eq!(
            lines(&problems).as_ref(),
            format!("application.landing.projects[0].gates[0].{path}: {words}\n").as_bytes()
        );
    }
}

#[test]
fn landing_configuration_tiers_use_the_configured_limits() {
    use temper_engine_domain_forge::TierLimits;
    let projects = tier_projects(&format!("[{TIER}]"), "");
    let input = fixtures::with(fixtures::APPLICATION, b"landing.projects", &projects);
    let (members, problems) = application(&input, &fixtures::engine().jig);
    assert!(problems.is_empty());
    assert_eq!(members.unwrap().limits.forge.tiers, TierLimits { tiers: 2, failing_tests: 16, test_id_bytes: 256 });
    let overrides = br#"{"forge.tiers.tiers":1,"forge.tiers.failing_tests":8,"forge.tiers.test_id_bytes":128}"#;
    let input = fixtures::with(&input, b"limits.overrides", overrides);
    let (members, problems) = application(&input, &fixtures::engine().jig);
    assert!(problems.is_empty(), "{:?}", lines(&problems));
    assert_eq!(members.unwrap().limits.forge.tiers, TierLimits { tiers: 1, failing_tests: 8, test_id_bytes: 128 });
    let tiers =
        format!(r#"[{TIER},{{"name":"another","workflow":"slow.yml","artifact":"failures","format":"lines"}}]"#);
    let changed = fixtures::with(&input, b"landing.projects[0].gates[0].tiers", tiers.as_bytes());
    let (members, problems) = application(&changed, &fixtures::engine().jig);
    assert!(members.is_none());
    assert_eq!(lines(&problems).as_ref(), b"application.landing.projects[0].gates[0].tiers: past its bound\n");
}

#[test]
fn landing_configuration_tier_settings_keep_their_integer_bounds() {
    use temper_engine_domain::AfterLandingCheck;
    for (settings, expected, refusal) in [
        (
            r#""tier_retries":0,"verification_bound_ms":0,"#,
            Some(AfterLandingCheck { tier_retries: 0, verification_bound_ms: 0 }),
            "",
        ),
        (
            r#""tier_retries":255,"verification_bound_ms":18446744073709551615,"#,
            Some(AfterLandingCheck { tier_retries: u8::MAX, verification_bound_ms: u64::MAX }),
            "",
        ),
        (r#""tier_retries":256,"#, None, "tier_retries: past its bound"),
        (r#""tier_retries":-1,"#, None, "tier_retries: past its bound"),
        (r#""tier_retries":"2","#, None, "tier_retries: wrong value kind"),
        (r#""verification_bound_ms":18446744073709551616,"#, None, "verification_bound_ms: past its bound"),
        (r#""verification_bound_ms":-1,"#, None, "verification_bound_ms: past its bound"),
        (r#""verification_bound_ms":"7200000","#, None, "verification_bound_ms: wrong value kind"),
    ] {
        let projects = tier_projects("[]", settings);
        let input = fixtures::with(fixtures::APPLICATION, b"landing.projects", &projects);
        let (members, problems) = application(&input, &fixtures::engine().jig);
        match expected {
            Some(after) => {
                assert!(problems.is_empty(), "{:?}", lines(&problems));
                assert_eq!(members.unwrap().landings.get(&1).unwrap().after, after);
            }
            None => {
                assert!(members.is_none());
                assert_eq!(
                    lines(&problems).as_ref(),
                    format!("application.landing.projects[0].{refusal}\n").as_bytes()
                );
            }
        }
    }
}

#[test]
fn a_document_without_demo_forge_and_project_is_read() {
    let input = fixtures::demo_application();
    let input = core::str::from_utf8(&input).unwrap().replace(",\n\"forge\":1,\"project\":1", "");
    let (members, problems) = application(input.as_bytes(), &fixtures::engine().jig);
    assert_eq!(&*lines(&problems), b"");
    let demo = members.unwrap().demo.unwrap();
    assert_eq!(demo.provider, 3);
    assert_eq!(demo.owner.as_ref(), b"playground");
    assert_eq!(demo.name.as_ref(), b"widget");
    assert_eq!(demo.landing.as_ref(), b"main");
    assert!(demo.key_file.ends_with(b"demo/llm.key"));
    assert_eq!(demo.cues.len(), 3);
}

#[test]
fn a_document_naming_forges_is_refused_by_path() {
    let input = core::str::from_utf8(&fixtures::document())
        .unwrap()
        .replace("\"application\":{", "\"application\":{\"forges\":[],");
    let found = crate::named(input.as_bytes(), &config::Directories::working()).unwrap_err();
    assert_eq!(lines(&found).as_ref(), b"application.forges: unknown member\n");
}
#[test]
fn identities_and_setup_are_unknown_members() {
    for member in ["identities", "setup"] {
        let input = core::str::from_utf8(&fixtures::document())
            .unwrap()
            .replace("\"application\":{", &format!("\"application\":{{\"{member}\":[],"));
        let found = crate::named(input.as_bytes(), &config::Directories::working()).unwrap_err();
        assert_eq!(lines(&found).as_ref(), format!("application.{member}: unknown member\n").as_bytes());
    }
}

#[test]
fn demo_refuses_forge_and_project_members() {
    for member in ["forge", "project"] {
        let input = fixtures::demo_application();
        let input = core::str::from_utf8(&input).unwrap().replace("\"demo\":{", &format!("\"demo\":{{\"{member}\":1,"));
        let (members, problems) = application(input.as_bytes(), &fixtures::engine().jig);
        assert!(members.is_none());
        assert_eq!(lines(&problems).as_ref(), format!("application.demo.{member}: unknown member\n").as_bytes());
    }
}

#[test]
fn startup_requires_room_for_the_core_and_two_charter_documents() {
    for sections in [7_u32, 8] {
        let input = fixtures::document();
        let overrides = fixtures::value(&input, b"application.limits.overrides");
        let overrides = core::str::from_utf8(&overrides).unwrap().trim_end().strip_suffix('}').unwrap();
        let overrides = format!("{overrides},\"root.brief.sections\":{sections}}}");
        let document = fixtures::with(&input, b"application.limits.overrides", overrides.as_bytes());
        let started = crate::startup(
            &document,
            &config::Directories::working(),
            &fixtures::files(),
            &fixtures::startup_agent(),
            &fixtures::seeds(),
            None,
            fixtures::worker_runtime(),
            fixtures::forge_runtime(),
        );
        if sections == 7 {
            let Err(problems) = started else { panic!("seven sections cannot hold the core and two documents") };
            assert_eq!(lines(&problems).as_ref(), b"application.limits.overrides.root.brief.sections: must agree with engine.limits.overrides.tasks.charter_documents and the core sections\n");
        } else {
            started.expect("eight sections hold the core and two documents");
        }
    }
}
