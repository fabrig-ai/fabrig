//! Limits for fabrig-scale development (jig/docs/design/protocol/limits.md).
//! The profile holds project work with five times the largest figures seen.
use super::small;
use super::{Limits, Link};
use jig_domain_accounts as accounts;
use jig_domain_authority as authority;
use jig_domain_brief as brief;
use jig_domain_core_vocabulary as core;
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_domain_views as views;
pub(super) const TASKS: tasks::Limits = tasks::Limits {
    placed_holds: 16,
    project_placed_holds: 8,
    party_placed_holds: 4,
    exemptions: 128,

    tasks: 128,
    funders: 128,
    project_tasks: 128,
    project_goals: 80,
    tree_tasks: 384,
    delegates: 80,
    resource_reports: 256,
    pools: 128,
    hold_waiters: 128,
    charters: 16,
    charter_documents: 2,
    charter_parts: 8,
    facts: 128,
    room_refusals: 32,
    batch: 160,
    circle: 32,
    dependencies: 16,
    inputs: 16,
    references: 16,
    holdings: 8,
    parameters: 8,
    saved_resources: 8,
    authority_grants: 16,
    authority_bytes: 4_096,
    // Twenty-one executor places hold the development delegation.
    executor_kinds: 21,
    depth: 32,
    inbox_messages: 82,
    run_inbox: 82,
    spec_bytes: 131_072,
    result_bytes: 262_144,
    message_bytes: 262_144,
    inbox_bytes: 21_495_808,
    batch_bytes: 4_194_304,
    authority_segments: 8,
    hold_segments: 8,
    contract_choices: 4,
    ..small::TASKS
};
pub(super) const PEOPLE: people::Limits = people::Limits {
    program_hosts: 65,
    operations: 81,
    batch: 4,
    people: 16,
    inbox_tasks: 256,
    inbox_recipients: 176,
    words: 8_192,
    spec_bytes: TASKS.spec_bytes,
    // The whole spec and 4,096 bytes for the member's other owned fields.
    amendment_bytes: 135_168,
    sign_ins: 32,
    projects: 9,
    holdings: 32,
    initial_owners: 4,
    // A day: 16 people * 20 conversations * (creation + 3 messages + decision),
    // with fivefold room for deciding acts, refusals and pending reservations.
    requests: 16 * 20 * 5 * 5,
    room_refusals: 32,
    pending: 32,
    facts: 128,
    ..small::PEOPLE
};
pub(super) const FLEET: fleet::Limits = fleet::Limits {
    reads: ACCOUNTS.accounts,
    configured_workers: 64,
    workers: 8,
    program_sources: 12,
    source_detail_bytes: 4096,
    needs_bytes: 262_144,
    grants: 32,
    slots: 4,
    workstreams: 64,
    attempts: 64,
    calls: 256,
    turns: 128,
    facts: 256,
    history: 64,
    ..small::FLEET
};
pub(super) const AUTHORITY: authority::Limits = authority::Limits {
    projects: 8,
    accounts: 16,
    roles: 8,
    requirements: 8,
    grants: 16,
    // Twenty-one executor places hold the development delegation.
    executors: 21,
    segments: 8,
    batch: 32,
    writes: 8,
    ..small::AUTHORITY
};
pub(super) const BRIEF: brief::Limits = brief::Limits { briefs: 32, ..small::BRIEF };
pub(super) const ACCOUNTS: accounts::Limits =
    accounts::Limits { program_hosts: 65, accounts: 16, sign_ins: 4, credentials: 15, facts: 128, ..small::ACCOUNTS };
pub(super) const VIEWS: views::Limits = views::Limits { runs: 32, watchers: 16, facts: 256, ..small::VIEWS };
pub(super) const NOTES: notes::Limits =
    notes::Limits { scopes: 19, entries_per_scope: 64, body_bytes: 65_536, lines: 8, recalled: 8, ..small::NOTES };
pub(super) const DOCUMENTS: core::documents::DocumentLimits = core::documents::DocumentLimits {
    roles: AUTHORITY.roles,
    grants: super::authority_grants(&TASKS, &AUTHORITY),
    executors: super::authority_executors(&TASKS, &AUTHORITY),
    implications: AUTHORITY.implications,
    rows: 8,
    text_bytes: 8_192,
    chips: 8,
    lines: 16,
    segments: super::name_segments(&TASKS, &AUTHORITY),
    ..small::DOCUMENTS
};
pub(super) const CORE: core::Limits = core::Limits {
    public: core::PublicIndexLimits::OFF,
    // The sum of child route bounds when this configured hand-off bound was set.
    routes: 151_655,
    settings: core::settings::Limits {
        revisions: 8,
        versions: 16,
        history: 64,
        proposals: 8,
        alternatives: 63,
        permissions: 32,
    },
    templates: 16,
    fill_mark: 1_073_741_824,
    connectors: 4,
    load_slots: 16,
    call_records: 128,
    tasks: TASKS,
    authority: AUTHORITY,
    people: PEOPLE,
    fleet: FLEET,
    brief: BRIEF,
    accounts: ACCOUNTS,
    notes: NOTES,
    views: VIEWS,
    load_bytes: 131_072,
    call_answer_bytes: 135_168,
    resume_bytes: 262_144,
    run_bytes: 65_536,
    documents: DOCUMENTS,
    ..small::CORE
};
pub(super) const HUB: host::Limits = host::Limits {
    charter_bytes: 262_144,
    outcome_bytes: 131_072,
    event_bytes: 135_168,
    inbox: 82,
    slots: 4,
    accounts: 32,
    facts: 256,
    told: 256,
    ..small::HUB
};
pub(super) const LINK: Link = Link { names: 64, opening: 16, facts: 4470, ..small::LINK };
pub(super) const LIMITS: Limits = Limits {
    core: CORE,
    store: STORE,
    hub: HUB,
    link: LINK,
    // Smith keeps the two-byte id length and up to 256 id bytes in its 8192-byte value.
    token: 7934,
    driver: skein_oauth_accounts::Limits {
        accounts: super::driver_places(&CORE.accounts).expect("all sources and pending account sign-ins"),
        exchanges: 8,
        listeners: CORE.accounts.sign_ins.checked_mul(2).expect("a listener and a settling one per sign-in"),
        io: skein_io::Limits { sockets: 64, names: 64, ..small::IO },
        ..small::DRIVER
    },
    clients: CLIENTS,
    clients_http: jig_protocol_engine_clients_http::Limits {
        public_calls: 0,
        calls: 16,
        streams: 8,
        body: CLIENTS.message_bytes,
        read: 16384,
        send: 16384,
        event: 87416,
    },
    io: skein_io::Limits { sockets: 64, names: 64, ..small::IO },
    journal: JOURNAL,
    routes: 2048,
};

pub(super) const CLIENTS: jig_protocol_engine_clients::Limits = jig_protocol_engine_clients::Limits {
    public: jig_protocol_engine_clients::PublicLimits::OFF,
    streams: 8,
    // A full web page holds this many watches (protocol/limits.md).
    watches_per_stream: 31,
    exchanges: 32,
    mints: 16,
    provider_codes: 256,
    provider_tokens: 1024,
    ..small::CLIENTS
};

// One extra Sources output at hello adds eight core route positions.
// Close cancellation adds one output per pending sign-in: four in Standard (protocol/llm.md, 5.2).
// Account moves count 56 core route positions for their emitted outputs.
// Retained-revision release adds 2 * 8 heads + 16 versions + one extent = 33.
// Placed holds: tightened core room plus 18 connector slots needs 151,673 writes.
const JOURNAL: super::JournalSizes = super::JournalSizes { commits: 3, writes: 151_673, door: 1024, release: 1024 };
// check_commits sets value to the largest map value, 100,881,496 bytes at these limits (protocol/store/records.md, 4).
// RunPolicy.tools is priced in the largest task row.
// Page bytes are key + value = 2,079 + 100,881,496 = 100,883,575, above the 33,554,496-byte
// TurnRecord frame for smith's 33,554,432-byte turn (protocol/store/transcripts.md, 1).
// Chunk bytes are skein_kv::chunk_bytes(key, value) = 100,883,595, including framing.
// The map's budget is the core's fill mark, 1 GiB: a project of fabrig's scale fills a team's
// 256 MiB, since its tasks keep every milestone in the map, and past its budget the map refuses
// every commit that grows it, before the fill mark refuses new party work.
const STORE_KV: skein_kv::Limits = skein_kv::Limits {
    value: 100_881_496,
    ops: 303_350,
    // 1 GiB values + 303,350 operations * (2,079 key + 9 framing) + 26 frame bytes.
    commit: 1_707_136_650,
    // Three queued commits; segment holds two, snapshot follows one.
    queued_bytes: 5_121_409_950,
    budget: CORE.fill_mark,
    segment: 3_414_273_300,
    snapshot_after: 1_707_136_650,
    chunk: 100_883_595,
    page: skein_kv::Page { rows: 2, bytes: 100_883_575 },
    ..small::STORE_KV
};
const STORE: jig_protocol_store::Limits = jig_protocol_store::Limits {
    core: CORE,
    store: STORE_KV,
    secrets: STORE_KV,
    commits: jig_protocol_store::sizes::Commits { in_flight: JOURNAL.commits, writes: JOURNAL.writes },
    held: CLIENTS.exchanges.checked_mul(2).expect("clients carried capacity"),
    joins: 16,
    reads: 48,
    ..small::STORE
};

/// Size stream edges from the wire, without reserving their byte ceilings.
pub(super) fn limits() -> Limits {
    let mut limits = LIMITS;
    limits.clients.queued_bytes = jig_protocol_engine_clients::limits::stream_queue_bytes(
        limits.clients.watches_per_stream,
        &limits.clients.wire,
    )
    .expect("standard's first deliveries fit a u32 byte bound");
    let record = jig_client_protocol_wire::v21::StreamEvent::encoded_bound(&limits.clients.wire)
        .expect("standard stream record");
    let record = u32::try_from(record).expect("standard stream record fits u32");
    // Base64 data has one `data: ` prefix and two line feeds.
    limits.clients_http.event = skein_lib::base64::encoded_len(skein_lib::base64::Alphabet::Standard, record)
        .expect("standard base64 record")
        .checked_add(8)
        .expect("standard SSE frame");
    limits
}
