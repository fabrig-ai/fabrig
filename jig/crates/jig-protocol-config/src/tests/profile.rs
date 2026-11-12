use crate::{Profile, profile};

#[test]
fn supplied_profiles_have_valid_domain_limits() {
    for selected in [Profile::Small, Profile::Standard] {
        let limits = profile(selected);
        assert_eq!(limits.driver.redirect_bytes, 16_384, "{selected:?} redirect bound");
        let redirect_room = skein_oauth_accounts::redirect_room(&limits.driver).unwrap();
        assert!(limits.driver.redirect_bytes >= redirect_room, "{selected:?} redirect holds a full answer");
        assert_eq!(limits.driver.server.head, 24_593, "{selected:?} listener head");
        let listener_head = skein_oauth_accounts::listener_head(limits.driver.redirect_bytes).unwrap();
        assert!(limits.driver.server.head >= listener_head, "{selected:?} head holds a browser's request");
        assert_eq!(limits.core.tasks.charter_documents, 2);
        assert_eq!(limits.core.brief.sections, 10);
        assert!(jig_domain_core::valid(&limits.core), "{selected:?} core");
        assert!(jig_domain_host::valid_hub(&limits.hub), "{selected:?} hub");
        assert_eq!(crate::domain::capacity(&limits.core), None, "{selected:?} capacity");
        assert_eq!(crate::domain::check_message(&limits.core), Ok(()), "{selected:?} messages");
        assert_eq!(crate::check_effective_limits(&limits.core), Ok(()), "{selected:?} codecs");
        jig_protocol_store::sizes::check_commits(
            &limits.core,
            &jig_protocol_store::sizes::Application { version: 1, key: 0, value: 0 },
            &limits.store.commits,
            &limits.store.store,
            limits.store.orphan_erases,
        )
        .expect("profile store bounds");
        let writes = jig_domain_core::settings::writes_bound(&limits.core).expect("profile settings writes");
        assert!(writes <= limits.journal.writes, "{selected:?} journal");
        let mut problems = crate::Problems::empty();
        crate::clients::check_settings(&limits, &mut problems);
        assert!(problems.problems.is_empty(), "{selected:?} client wire: {problems:?}");
    }
}

#[test]
fn child_validity_keeps_semantic_and_count_refusals() {
    let limits = profile(Profile::Small).core;
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
    for (kind, request, page) in [(Profile::Small, 65_536, 3_421_331), (Profile::Standard, 262_144, 100_883_575)] {
        let configured = profile(kind);
        let codec = jig_protocol_store::codec::limits(&configured.core).expect("profile codec");
        assert_eq!(u64::from(configured.core.resume_bytes), request);
        assert_eq!(configured.store.store.page.bytes, page);
        assert_eq!(jig_protocol_store::core_v6::TurnRecord::encoded_bound(&codec.core).expect("body") + 12, MAX_FRAME);
        assert!(request.min(u64::from(page)) < MAX_FRAME);
    }
}

// jig/docs/design/protocol/limits.md: development figures and their coupled limits.
#[test]
fn standard_holds_fabrig_scale() {
    let limits = profile(Profile::Standard);
    let tasks = limits.core.tasks;
    assert_eq!(tasks.tasks, 128);
    assert_eq!(tasks.project_tasks, 128);
    assert_eq!(tasks.project_goals, 80);
    assert_eq!(tasks.tree_tasks, 384);
    assert_eq!(tasks.depth, 32);
    assert_eq!(tasks.delegates, 80);
    assert_eq!(tasks.batch, 160);
    assert_eq!(tasks.inbox_messages, 82);
    assert_eq!(tasks.run_inbox, 82);
    assert_eq!(tasks.spec_bytes, 131_072);
    assert_eq!(tasks.result_bytes, 262_144);
    assert_eq!(tasks.message_bytes, 262_144);
    assert_eq!(tasks.charters, 12);
    // The development delegation needs eighteen executor places.
    assert_eq!(tasks.executor_kinds, 18);
    assert_eq!(tasks.authority_grants, 16);
    assert_eq!(tasks.authority_segments, 8);
    assert_eq!(tasks.hold_segments, 8);
    assert_eq!(tasks.contract_choices, 4);
    assert_eq!(tasks.inbox_bytes, 21_495_808);
    assert_eq!(tasks.batch_bytes, 4_194_304);
    assert_eq!(tasks.funders, 128);
    assert_eq!(tasks.pools, 128);
    assert_eq!(tasks.hold_waiters, 128);
    assert_eq!(tasks.exemptions, 128);
    assert_eq!(limits.core.people.operations, 81);
    assert_eq!(limits.core.people.inbox_tasks, 256);
    assert_eq!(limits.core.people.inbox_recipients, 176);
    assert_eq!(limits.core.people.words, 8_192);
    assert_eq!(limits.core.authority.grants, 16);
    // The development delegation needs eighteen executor places.
    assert_eq!(limits.core.authority.executors, 18);
    assert_eq!(limits.core.authority.segments, 8);
    assert_eq!(limits.core.authority.batch, 32);
    assert_eq!(limits.hub.charter_bytes, 262_144);
    assert_eq!(limits.hub.outcome_bytes, 131_072);
    assert_eq!(limits.core.notes.scopes, 19);
    assert_eq!(limits.core.documents.lines, 16);
    assert_eq!(limits.core.documents.segments, profile::name_segments(&tasks, &limits.core.authority));
    assert_eq!(limits.hub.inbox, 82);
    let store = limits.store.store;
    assert_eq!(store.value, 100_881_496);
    let frame = jig_protocol_store::sizes::frame(&limits.core, 33_554_432).expect("smith's largest turn frame");
    assert_eq!(store.page.bytes, store.key.checked_add(store.value).unwrap().max(frame));
    assert_eq!(Some(store.chunk), skein_kv::chunk_bytes(store.key, store.value));
}

#[test]
fn standard_page_holds_smiths_largest_turn() {
    let limits = profile(Profile::Standard);
    let frame = jig_protocol_store::sizes::frame(&limits.core, 33_554_432).expect("smith's largest turn frame");
    assert_eq!(frame, 33_554_496);
    assert!(limits.store.store.page.bytes >= frame);
}

#[test]
fn small_is_unchanged() {
    let limits = profile(Profile::Small);
    let tasks = limits.core.tasks;
    assert_eq!(tasks.tasks, 8);
    assert_eq!(tasks.batch, 2);
    assert_eq!(tasks.spec_bytes, 4_096);
    assert_eq!(tasks.result_bytes, 512);
    assert_eq!(limits.store.store.value, 3_419_252);
}

#[test]
fn profiles_retain_a_day_of_deciding_answers_with_fivefold_room() {
    for selected in [Profile::Small, Profile::Standard] {
        let people = profile(selected).core.people;
        // Twenty conversations each: create, three messages, decide. Fivefold headroom.
        assert_eq!(people.requests, people.people * 20 * 5 * 5);
        assert_eq!(people.request_retention, skein_lib::Duration::from_secs(86_400));
    }
}

// jig/docs/design/domain/engine/briefs.md, 2: bodies and the calls and loads carrying them.
#[test]
fn profiles_hold_a_documents_body_and_carriers() {
    for (kind, body, escaped, load) in
        [(Profile::Small, 32_768, 69_632, 65_536), (Profile::Standard, 65_536, 135_168, 131_072)]
    {
        let limits = profile(kind);
        assert_eq!(limits.core.notes.body_bytes, body, "{kind:?} body");
        assert_eq!(limits.core.call_answer_bytes, escaped, "{kind:?} call");
        assert_eq!(limits.hub.event_bytes, u64::from(escaped), "{kind:?} event");
        assert_eq!(limits.core.load_bytes, load, "{kind:?} load");
    }
}

#[test]
fn supplied_profiles_keep_four_exchanges_beside_every_stream() {
    const ORDINARY_EXCHANGES: u32 = 4;
    for selected in [Profile::Small, Profile::Standard] {
        let limits = profile(selected);
        let streams = limits.clients.streams.max(limits.clients_http.streams);
        let minimum = streams.checked_add(ORDINARY_EXCHANGES).expect("profile carrier count");
        assert!(limits.clients_http.calls >= minimum, "{selected:?} HTTP calls carry streams and ordinary exchanges");
        assert!(limits.clients.exchanges >= limits.clients_http.calls, "{selected:?} component carries HTTP calls");
    }
}

#[test]
fn profiles_give_infrastructure_its_own_allowance_and_backoff() {
    use jig_domain_tasks::{Class, Retry};
    use skein_lib::Duration;
    for selected in [Profile::Small, Profile::Standard] {
        let retries = profile(selected).core.tasks.retries;
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
}
