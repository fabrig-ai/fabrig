use crate::profile;

#[test]
fn supplied_profiles_have_valid_domain_limits() {
    let limits = crate::stock();
    assert_eq!(limits.driver.redirect_bytes, 16_384, "stock redirect bound");
    let redirect_room = skein_oauth_accounts::redirect_room(&limits.driver).unwrap();
    assert!(limits.driver.redirect_bytes >= redirect_room, "stock redirect holds a full answer");
    assert_eq!(limits.driver.server.head, 24_593, "stock listener head");
    let listener_head = skein_oauth_accounts::listener_head(limits.driver.redirect_bytes).unwrap();
    assert!(limits.driver.server.head >= listener_head, "stock head holds a browser's request");
    assert_eq!(limits.core.tasks.charter_documents, 2);
    assert_eq!(limits.core.brief.sections, 10);
    assert!(jig_domain_core_vocabulary::valid(&limits.core), "stock core");
    assert!(jig_domain_host::valid_hub(&limits.hub), "stock hub");
    assert_eq!(crate::domain::capacity(&limits.core), None, "stock capacity");
    assert_eq!(crate::domain::check_message(&limits.core), Ok(()), "stock messages");
    assert_eq!(crate::check_effective_limits(&limits.core), Ok(()), "stock codecs");
    let sizes = jig_protocol_store::sizes::check_commits(
        &limits.core,
        &jig_protocol_store::sizes::Application { version: 1, key: 0, value: 0 },
        &limits.store.commits,
        &limits.store.store,
        limits.store.orphan_erases,
    )
    .expect("profile store bounds");
    assert_eq!(sizes.value, limits.store.store.value, "stock's least value envelope");
    let writes = jig_domain_core_vocabulary::settings::writes_bound(&limits.core).expect("profile settings writes");
    assert!(writes <= limits.journal.writes, "stock journal");
    let mut problems = crate::Problems::empty();
    crate::clients::check_settings(&limits, &mut problems);
    assert!(problems.problems.is_empty(), "stock client wire: {problems:?}");
}

#[test]
fn child_validity_keeps_semantic_and_count_refusals() {
    let limits = crate::stock().core;
    let mut brief = limits.brief;
    brief.briefs = 0;
    assert!(!jig_domain_brief::valid(&brief));
    brief = limits.brief;
    brief.sections = 0;
    assert!(!jig_domain_brief::valid(&brief));
    brief = limits.brief;
    brief.read_bytes = 0;
    assert!(!jig_domain_brief::valid(&brief));
    brief = limits.brief;
    brief.brief_bytes = 0;
    assert!(!jig_domain_brief::valid(&brief));
    let mut notes = limits.notes;
    notes.scopes = 0;
    assert!(!jig_domain_notes::valid(&notes));
    notes = limits.notes;
    notes.entries_per_scope = 0;
    assert!(!jig_domain_notes::valid(&notes));
    notes = limits.notes;
    notes.party_entries = 0;
    assert!(!jig_domain_notes::valid(&notes));
    notes = limits.notes;
    notes.party_entries = notes.entries_per_scope.checked_add(1).unwrap();
    assert!(!jig_domain_notes::valid(&notes));
    notes = limits.notes;
    notes.party_bytes = notes.description_bytes.checked_add(1).unwrap();
    assert!(!jig_domain_notes::valid(&notes));
    notes = limits.notes;
    notes.load_rows = 0;
    assert!(!jig_domain_notes::valid(&notes));
    notes = limits.notes;
    notes.recalled = 0;
    assert!(!jig_domain_notes::valid(&notes));
    notes = limits.notes;
    notes.scopes = u32::MAX;
    assert!(!jig_domain_notes::valid(&notes));
    let mut views = limits.views;
    views.runs = 0;
    assert!(!jig_domain_views::valid(&views));
    views = limits.views;
    views.watchers = 0;
    assert!(!jig_domain_views::valid(&views));
    views = limits.views;
    views.watchers_per_party = 0;
    assert!(!jig_domain_views::valid(&views));
    views = limits.views;
    views.subjects_per_change = 0;
    assert!(!jig_domain_views::valid(&views));
    views = limits.views;
    views.text_bytes = 0;
    assert!(!jig_domain_views::valid(&views));
    views = limits.views;
    views.watchers = u32::MAX;
    assert!(!jig_domain_views::valid(&views));
    let mut fleet = limits.fleet;
    fleet.worker_name_bytes = jig_domain_fleet::WORKER_NAME + 1;
    assert!(!jig_domain_fleet::valid(&fleet, limits.settings.versions));
    fleet = limits.fleet;
    fleet.workers = u32::MAX;
    fleet.engine_slots = 1;
    assert!(!jig_domain_fleet::valid(&fleet, limits.settings.versions));
    fleet = limits.fleet;
    fleet.history = u32::MAX;
    assert!(!jig_domain_fleet::valid(&fleet, limits.settings.versions));
}

#[test]
fn configured_read_room_is_smaller_than_the_maximum_frame() {
    // The frame bound is TurnRecord::encoded_bound + 12, including
    // version, optional read, transcript length, checksum and both lengths.
    const MAX_FRAME: u64 = 33_554_496;
    // The map pages include job specs, task/run revisions and placed references;
    // transcript framing and direct read room are unchanged.
    let (request, page) = (262_144, 103_389_955);
    let configured = crate::stock();
    let codec = jig_protocol_store::codec::limits(&configured.core).expect("profile codec");
    assert_eq!(u64::from(configured.core.resume_bytes), request);
    assert_eq!(configured.store.store.page.bytes, page);
    assert_eq!(jig_protocol_store::core_v6::TurnRecord::encoded_bound(&codec.core).expect("body") + 12, MAX_FRAME);
    assert!(request.min(u64::from(page)) < MAX_FRAME);
}

// jig/docs/design/protocol/limits.md: sized from fabrig's own development (2026-11-16).
#[test]
fn stock_figures_cover_development_and_read_cleanly() {
    let limits = crate::stock();
    for (name, expected) in [
        ("clients.streams", 64),
        ("clients_http.streams", 64),
        ("clients.exchanges", 68),
        ("clients_http.calls", 68),
        // These stock counts already exceed twice the development counts.
        ("fleet.configured_workers", 64),
        ("fleet.workers", 8),
        ("fleet.slots", 40),
        ("hub.slots", 80),
        ("tasks.tree_tasks", 2_048),
        ("tasks.party_placed_holds", 128),
        ("tasks.project_placed_holds", 256),
        ("tasks.placed_holds", 512),
        ("authority.executors", 64),
        ("tasks.executor_kinds", 64),
        ("tasks.charters", 48),
        ("tasks.tasks", 768),
        ("tasks.project_tasks", 768),
        ("people.inbox_tasks", 1_536),
        // The stream-queue derivation and the policy bound already cover development.
        ("clients.queued_bytes", 520_094_521),
        ("core.policy_bytes", 16_384),
        ("people.sign_in_lifetime_ms", 86_400_000),
    ] {
        let row = crate::names::find(name.as_bytes()).expect("named limit");
        assert_eq!((row.value)(&limits), expected, "{name}");
    }
    let document = super::fixtures::document(br#"{"overrides":{}}"#);
    let parsed = crate::overrides::parse_over(&document, 0, &crate::Path::root(), &limits);
    assert!(parsed.problems.is_empty(), "{:?}", parsed.problems);
    assert_eq!(parsed.limits, limits);
    let bytes = super::fixtures::with(super::fixtures::ENGINE_SMALL, br#""profile": "standard""#, br#""overrides":{}"#);
    let engine = super::fixtures::engine(&bytes).expect("stock engine passes every startup check");
    assert_eq!(engine.limits, limits);
}

// Development figures and their coupled limits.
#[test]
fn standard_holds_fabrig_scale() {
    let limits = crate::stock();
    let tasks = limits.core.tasks;
    assert_eq!(tasks.project_goals, 80);
    assert_eq!(tasks.depth, 32);
    assert_eq!(tasks.delegates, 80);
    assert_eq!(tasks.batch, 160);
    assert_eq!(tasks.inbox_messages, 82);
    assert_eq!(tasks.run_inbox, 82);
    assert_eq!(tasks.spec_bytes, 131_072);
    assert_eq!(tasks.result_bytes, 262_144);
    assert_eq!(tasks.message_bytes, 262_144);
    assert_eq!(tasks.authority_grants, 16);
    assert_eq!(tasks.authority_segments, 8);
    assert_eq!(tasks.hold_segments, 8);
    assert_eq!(tasks.contract_choices, 4);
    assert_eq!(tasks.inbox_bytes, 21_495_808);
    assert_eq!(tasks.batch_bytes, 4_194_304);
    assert_eq!(tasks.funders, tasks.tasks);
    assert_eq!(tasks.pools, tasks.tasks);
    assert_eq!(tasks.hold_waiters, tasks.tasks);
    assert_eq!(tasks.exemptions, tasks.tasks);
    assert_eq!(limits.core.people.operations, 81);
    assert_eq!(limits.core.people.inbox_tasks, tasks.tasks * 2);
    assert_eq!(limits.core.people.inbox_recipients, 176);
    assert_eq!(limits.core.people.words, 8_192);
    assert_eq!(limits.core.authority.grants, 16);
    assert_eq!(limits.core.authority.segments, 8);
    assert_eq!(limits.core.authority.batch, 32);
    assert_eq!(limits.hub.charter_bytes, 262_144);
    assert_eq!(limits.hub.outcome_bytes, 131_072);
    assert_eq!(limits.core.notes.scopes, 19);
    assert_eq!(limits.core.documents.lines, 16);
    assert_eq!(limits.core.documents.segments, profile::name_segments(&tasks, &limits.core.authority));
    assert_eq!(limits.hub.inbox, 82);
    let store = limits.store.store;
    assert_eq!(store.value, 103_387_876);
    let frame = jig_protocol_store::sizes::frame(&limits.core, 33_554_432).expect("smith's largest turn frame");
    assert_eq!(store.page.bytes, store.key.checked_add(store.value).unwrap().max(frame));
    assert_eq!(Some(store.chunk), skein_kv::chunk_bytes(store.key, store.value));
}

// The core refuses new party work at its fill mark; a map whose budget is under
// the mark refuses every commit that grows it first, and the engine stops.
#[test]
fn each_profiles_store_map_holds_its_fill_mark() {
    let limits = crate::stock();
    assert!(limits.store.store.budget >= limits.core.fill_mark, "stock map budget");
}

#[test]
fn standard_page_holds_smiths_largest_turn() {
    let limits = crate::stock();
    let frame = jig_protocol_store::sizes::frame(&limits.core, 33_554_432).expect("smith's largest turn frame");
    assert_eq!(frame, 33_554_501);
    assert!(limits.store.store.page.bytes >= frame);
}

#[test]
fn profiles_retain_a_day_of_deciding_answers_with_fivefold_room() {
    let people = crate::stock().core.people;
    // Twenty conversations each: create, three messages, decide. Fivefold headroom.
    assert_eq!(people.requests, people.people * 20 * 5 * 5);
    assert_eq!(people.request_retention, skein_lib::Duration::from_secs(86_400));
}

// jig/docs/design/domain/engine/briefs.md, 2: bodies and the calls and loads carrying them.
#[test]
fn profiles_hold_a_documents_body_and_carriers() {
    {
        let (body, escaped, load) = (65_536, 135_168, 131_072);
        let limits = crate::stock();
        assert_eq!(limits.core.notes.body_bytes, body, "stock body");
        assert_eq!(limits.core.call_answer_bytes, escaped, "stock call");
        assert_eq!(limits.hub.event_bytes, u64::from(escaped), "stock event");
        assert_eq!(limits.core.load_bytes, load, "stock load");
    }
}

#[test]
fn supplied_profiles_keep_four_exchanges_beside_every_stream() {
    const ORDINARY_EXCHANGES: u32 = 4;

    let limits = crate::stock();
    let streams = limits.clients.streams.max(limits.clients_http.streams);
    let minimum = streams.checked_add(ORDINARY_EXCHANGES).expect("profile carrier count");
    assert!(limits.clients_http.calls >= minimum, "stock HTTP calls carry streams and ordinary exchanges");
    assert!(limits.clients.exchanges >= limits.clients_http.calls, "stock component carries HTTP calls");
}

#[test]
fn profiles_give_infrastructure_its_own_allowance_and_backoff() {
    use jig_domain_tasks::{Class, Retry};
    use skein_lib::Duration;

    let retries = crate::stock().core.tasks.retries;
    assert_eq!(
        retries.infrastructure,
        Retry { retries: 12, base: Duration::from_millis(10_000), max: Duration::from_millis(300_000) }
    );
    for class in [Class::Transient, Class::Permanent, Class::Run, Class::Agent, Class::Lost, Class::Invalid] {
        assert_eq!(
            retries.of(class),
            Retry { retries: 3, base: Duration::from_millis(1_000), max: Duration::from_millis(30_000) }
        );
    }
}

// jig/docs/design/protocol/limits.md: first deliveries and the HTTP frame carrying each.
#[test]
fn standard_stream_edges_hold_every_first_delivery() {
    use jig_client_protocol_wire as w;
    let limits = crate::stock();
    let clients = limits.clients;
    let delivery = w::Delivery::encoded_bound(&clients.wire).expect("delivery bound") + 1;
    assert_eq!(clients.watches_per_stream, 31);
    assert_eq!(delivery, u64::from(clients.wire.owned_record) + 20);
    assert_eq!(
        u64::from(clients.queued_bytes),
        u64::from(clients.watches_per_stream) * delivery + w::Hello::encoded_bound(&clients.wire).expect("hello") + 1
    );
    assert_eq!(clients.queued_bytes, 520_094_521);
    let record =
        u32::try_from(w::StreamEvent::encoded_bound(&clients.wire).expect("stream bound")).expect("u32 record");
    let base64 = skein_lib::base64::encoded_len(skein_lib::base64::Alphabet::Standard, record).expect("base64");
    assert_eq!(limits.clients_http.event, base64 + 8);
    assert!(jig_protocol_engine_clients::valid(&clients));
    assert_eq!(jig_protocol_engine_clients::limits::stream_queue_bytes(u32::MAX, &clients.wire), None);
}

#[test]
fn standard_sse_edge_accepts_the_largest_stream_frame() {
    const PIECE: [u8; 1024] = [b'A'; 1024];
    use skein_http::sse::writer as sse;
    use skein_lib::{Env, Queue, Time, Wall};
    let limits = crate::stock();
    let record = jig_client_protocol_wire::StreamEvent::encoded_bound(&limits.clients.wire).expect("stream record");
    let data_bytes = skein_lib::base64::encoded_len(
        skein_lib::base64::Alphabet::Standard,
        u32::try_from(record).expect("u32 stream record"),
    )
    .expect("base64");
    for short in [false, true] {
        let mut data = skein_lib::Writer::new(usize::try_from(data_bytes).expect("data length"));
        for _ in 0..data_bytes / 1024 {
            data.put(&PIECE).expect("whole base64 pieces");
        }
        let remainder = usize::try_from(data_bytes % 1024).expect("piece remainder");
        data.put(PIECE.get(..remainder).expect("partial piece")).expect("last base64 piece");
        let env = Env {
            now: Time::ZERO,
            wall: Wall::EPOCH,
            limits: sse::Limits {
                event: limits.clients_http.event - u32::from(short),
                chunk: limits.clients_http.send,
            },
        };
        let mut writer = sse::Writer::new(&env.limits);
        let mut above = Queue::uncapped();
        let mut below = Queue::uncapped();
        sse::down(
            &mut writer,
            &env,
            sse::Request::Event(sse::Outgoing { name: Box::new([]), data: data.finish(), id: None, retry: None }),
            &mut above,
            &mut below,
        );
        if short {
            assert_eq!(above.pop(), Some(sse::Event::Refused(sse::Refusal::TooLong)));
            assert!(below.is_empty());
        } else {
            assert!(above.is_empty());
            assert!(below.pop().is_some(), "accepted frame asks for transport room");
        }
    }
}
