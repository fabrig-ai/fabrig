//! Limits for fabrig-scale development (jig/docs/design/protocol/limits.md).
//! Counts cover twice fabrig's own development (2026-11-16), retaining higher stock figures.
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
use skein_lib::Duration;
pub(super) const TASKS: tasks::Limits = tasks::Limits {
    // Twice fabrig's own development's 256 placed holds (2026-11-16).
    placed_holds: 512,
    // Twice fabrig's own development's 128 project holds (2026-11-16).
    project_placed_holds: 256,
    // Twice fabrig's own development's 64 party holds (2026-11-16).
    party_placed_holds: 128,
    // One exemption per live task at the stock's task count.
    exemptions: 768,
    // Twice fabrig's own development's 384 live tasks (2026-11-16).
    tasks: 768,
    // One funder per live task at the stock's task count.
    funders: 768,
    // Twice fabrig's own development's 384 project tasks (2026-11-16).
    project_tasks: 768,
    project_goals: 80,
    // Twice fabrig's own development's 1,024 tree tasks (2026-11-16).
    tree_tasks: 2_048,
    delegates: 80,
    resource_reports: 256,
    // One pool per live task at the stock's task count.
    pools: 768,
    // One hold waiter per live task at the stock's task count.
    hold_waiters: 768,
    // Twice fabrig's own development's 24 charters (2026-11-16).
    charters: 48,
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
    // Twice fabrig's own development's 32 executor kinds (2026-11-16).
    executor_kinds: 64,
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
    subscriptions: 4,
    hold_kinds: 8,
    hold_bytes: 1_024,
    grace: Duration::from_secs(600),
    hold_wait: Duration::from_secs(120),
    person_stall: Duration::from_secs(3600),
    proposal_stall: Duration::from_secs(3600),
    escalation_stall: Duration::from_secs(3600),
    retries: RETRIES,
};
pub(super) const PEOPLE: people::Limits = people::Limits {
    program_hosts: 65,
    operations: 81,
    batch: 4,
    people: 16,
    // Twice fabrig's own development's 768 inbox tasks (2026-11-16); two per live task.
    inbox_tasks: 1_536,
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
    inbox_entries: 8,
    waiters: 4,
    program_host_name_bytes: 32,
    act_grants: 0,
    identity_bytes: 1_024,
    sign_in_lifetime: Duration::from_secs(86400),
    request_retention: Duration::from_secs(86400),
};
pub(super) const FLEET: fleet::Limits = fleet::Limits {
    reads: ACCOUNTS.accounts,
    configured_workers: 64,
    workers: 8,
    program_sources: 12,
    source_detail_bytes: 4096,
    needs_bytes: 262_144,
    grants: 32,
    // The most the engine's u32 link cap holds at the stock's and the application's
    // derived figures, below twice fabrig's own development's 36 slots per worker (2026-11-16).
    slots: 40,
    workstreams: 64,
    attempts: 64,
    calls: 256,
    turns: 128,
    facts: 256,
    history: 64,
    worker_name_bytes: 32,
    engine_slots: 0,
    call_name_bytes: 64,
    progress_bytes: 1_024,
    // Above the stop bound on the link profile with a 300 s engine-away grace:
    // max(5 + 15 + 300, 180) + max(5, 1) + 1 = 326 s. The engine-away grace
    // dominates the bound; keep a margin (jig/docs/design/domain/engine/fleet.md).
    grace: Duration::from_secs(360),
    restart_wait: Duration::from_millis(1_200_000),
};
pub(super) const AUTHORITY: authority::Limits = authority::Limits {
    projects: 8,
    accounts: 16,
    roles: 8,
    requirements: 8,
    grants: 16,
    // Twice fabrig's own development's 32 executors (2026-11-16).
    executors: 64,
    segments: 8,
    batch: 32,
    writes: 8,
    facts: 16,
    segment_bytes: 256,
    implications: 8,
};
pub(super) const BRIEF: brief::Limits =
    brief::Limits { briefs: 32, sections: 10, read_bytes: 8_192, brief_bytes: 16_384 };
pub(super) const ACCOUNTS: accounts::Limits = accounts::Limits {
    program_hosts: 65,
    accounts: 16,
    sign_ins: 4,
    credentials: 15,
    facts: 128,
    holder_accounts: 4,
    program_names: 12,
    sources: 3,
    backoff_base: Duration::from_secs(2),
    backoff_max: Duration::from_secs(60),
    retry: Duration::from_secs(30),
    move_deadline: Duration::from_secs(60),
    spent_attention: Duration::from_secs(300),
};
pub(super) const VIEWS: views::Limits = views::Limits {
    runs: 32,
    watchers: 16,
    facts: 256,
    watchers_per_party: 4,
    subjects_per_change: 32,
    text_bytes: 65_536,
};
pub(super) const NOTES: notes::Limits = notes::Limits {
    scopes: 19,
    entries_per_scope: 64,
    body_bytes: 65_536,
    lines: 8,
    recalled: 8,
    party_entries: 12,
    party_bytes: 120,
    pattern_bytes: 256,
    description_bytes: 256,
    references: 8,
    load_rows: 4,
};
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
    versions: core::documents::Versions { minimum: 1, maximum: 1 },
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
    retention: core::RetentionLimits {
        turns: Duration::from_secs(14 * 86_400),
        records: Duration::from_secs(30 * 86_400),
        summaries: Duration::from_secs(365 * 86_400),
    },
    policy_bytes: 16_384,
    escalation_reason_bytes: 512,
    brief_parts: 8,
    brief_core_budgets: BUDGETS,
};
pub(super) const HUB: host::Limits = host::Limits {
    charter_bytes: 262_144,
    outcome_bytes: 131_072,
    event_bytes: 135_168,
    inbox: 82,
    // Twice fabrig's own development's 36 slots per worker (2026-11-16), rounded.
    slots: 80,
    accounts: 32,
    facts: 256,
    told: 256,
    application_ask_bytes: 64,
    application_answer_bytes: 64,
    program_sources: 12,
    source_interval: Duration::from_secs(60),
    source_timeout: Duration::from_secs(10),
    transcript_bytes: 262_144,
    delivery_evidence_bytes: 4_096,
    turn_bytes: 65_536,
    detail_bytes: 4_096,
    held: 8,
    run_calls: 8,
    turns: 8,
    turn_queue_bytes: 524_288,
    piece_bytes: 4096,
};
pub(super) const LINK: Link = Link {
    names: 64,
    opening: 16,
    facts: 4470,
    chunk: 16_384,
    name: 32,
    secret: 256,
    handshake: Duration::from_secs(10),
    hello: Duration::from_secs(10),
    ping: Duration::from_secs(5),
    silence: Duration::from_secs(15),
    stall: Duration::from_secs(30),
    backoff: Duration::from_millis(250),
    ceiling: Duration::from_secs(30),
    proved: Duration::from_secs(60),
    busy: Duration::from_secs(2),
};
// Twice fabrig's own development's 32 streams (2026-11-16).
const STREAMS: u32 = 64;

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
        io: IO,
        file_stall: Duration::from_secs(30),
        lock_wait: Duration::from_secs(1),
        // A full answer at the client's bounds, 13,407 bytes (redirect_room),
        // with room to grow (jig's protocol/limits.md).
        redirect_bytes: REDIRECT,
        server: skein_http::server::Limits {
            // The request line at the redirect bound, BROWSER_FIELDS for a browser's fields
            // and its loopback host's cookies (not port-scoped), and the blank line.
            head: skein_oauth_accounts::listener_head(REDIRECT).expect("a browser's redirect at the bound"),
            headers: 32,
            body: 1024,
            read: 1024,
            response: 4096,
            send: 1024,
        },
        refresh_lead: Duration::from_secs(60),
        client: skein_oauth::ClientLimits {
            document: skein_oauth::Limits {
                document_bytes: 65536,
                string_bytes: 16384,
                token_bytes: 7934,
                client_bytes: 1024,
                detail_bytes: 1024,
                record_bytes: 65536,
                // Account sign-in: up to one KiB of issuer-issued client id (skein skein/docs/design/oauth/README.md 5).
                issued_client_bytes: 1024,
                granted_bytes: 4096,
                // Account sign-in: one KiB of OpenID subject (skein skein/docs/design/oauth/README.md 5).
                subject_bytes: 1024,
                id_token_bytes: 7934,
                depth: 16,
                tokens: 4096,
            },
            uri_bytes: 4096,
            scope_bytes: 4096,
            state_bytes: 64,
            code_bytes: 4096,
            url_bytes: jig_domain_accounts::ADDRESS,
            request_bytes: 65536,
            // Account sign-in terms (skein skein/docs/design/oauth/README.md, 5, skein/docs/design/oauth/accounts.md, 7): host id 17 + 45,
            // agent_name_hint 15 + up to 256 bytes.
            parameter_bytes: 62 + 15 + 256,
            // One URI's 4096 bytes for the account sign-in resource.
            resource_bytes: 4096,
            // 32 random bytes, above the OAuth client's 16-byte minimum.
            nonce_bytes: 32,
            // Account sign-in discovery and key-set documents: 64 KiB each.
            metadata_bytes: 65_536,
            key_set_bytes: 65_536,
            // Four rotating issuer keys (skein skein/docs/design/oauth/README.md 5).
            keys: 4,
            modulus_bits: 2048,
            skew: Duration::ZERO,
            sign_in_time: Duration::from_secs(300),
            request_time: Duration::from_secs(30),
            backoff_base: Duration::from_secs(1),
            backoff_ceiling: Duration::from_secs(30),
            max_attempts: 5,
        },
        http: skein_http::client::Limits { request: 65536, head: 16384, headers: 32, read: 16384, send: 16384 },
        tls: skein_tls::client::Limits { read: 16384, send: 65536, records: 131_072 },
    },
    clients: CLIENTS,
    clients_http: jig_protocol_engine_clients_http::Limits {
        public_calls: 0,
        // The stock's stream bound plus four ordinary calls.
        calls: CLIENTS.exchanges,
        // Twice fabrig's own development's 32 HTTP streams (2026-11-16).
        streams: CLIENTS.streams,
        body: CLIENTS.message_bytes,
        read: 16384,
        send: 16384,
        event: 87416,
    },
    io: IO,
    journal: JOURNAL,
    routes: 2048,
};

pub(super) const CLIENTS: jig_protocol_engine_clients::Limits = jig_protocol_engine_clients::Limits {
    public: jig_protocol_engine_clients::PublicLimits::OFF,
    // Twice fabrig's own development's 32 streams (2026-11-16).
    streams: STREAMS,
    // A full web page holds this many watches (protocol/limits.md).
    watches_per_stream: 31,
    // The stock's streams plus four ordinary exchanges (protocol/limits.md).
    exchanges: STREAMS.checked_add(4).expect("stock stream carriers fit u32"),
    mints: 16,
    provider_codes: 256,
    provider_tokens: 1024,
    queued_bytes: 65536,
    identity_bytes: 1024,
    families: 32,
    providers: 32,
    exchanges_per_client: 2,
    // One full moved file and its keyed account-change envelope.
    message_bytes: 131_328,
    wire: jig_client_protocol_wire::CEILINGS,
    link_span: Duration::from_secs(7200),
};

// One extra Sources output at hello adds eight core route positions.
// Close cancellation adds one output per pending sign-in: four in Standard (protocol/llm.md, 5.2).
// Account moves count 56 core route positions for their emitted outputs.
// Retained-revision release adds 2 * 8 heads + 16 versions + one extent = 33.
// Placed holds: tightened core room plus 18 connector slots needs 151,673 writes.
const JOURNAL: super::JournalSizes = super::JournalSizes { commits: 3, writes: 151_673, door: 1024, release: 1024 };
// check_commits sets value to the largest map value, 103,273,060 bytes at these limits (protocol/store/records.md, 4).
// Settings history includes provider settings and the values a revision replaced.
// Page bytes are key + value = 2,079 + 103,273,060 = 103,275,139, above the 33,554,496-byte
// TurnRecord frame for smith's 33,554,432-byte turn (protocol/store/transcripts.md, 1).
// Chunk bytes are skein_kv::chunk_bytes(key, value) = 103,275,159, including framing.
// The map's budget is the core's fill mark, 1 GiB: a project of fabrig's scale fills a team's
// 256 MiB, since its tasks keep every milestone in the map, and past its budget the map refuses
// every commit that grows it, before the fill mark refuses new party work.
const STORE_KV: skein_kv::Limits = skein_kv::Limits {
    value: 103_273_060,
    ops: 455_024,
    // 1 GiB values + 455,024 operations * (2,079 key + 9 framing) + 26 frame bytes.
    commit: 2_023_831_962,
    // Three queued commits; segment holds two, snapshot follows one.
    queued_bytes: 6_071_495_886,
    budget: CORE.fill_mark,
    segment: 4_047_663_924,
    snapshot_after: 2_023_831_962,
    chunk: 103_275_159,
    page: skein_kv::Page { rows: 2, bytes: 103_275_139 },
    record: 103_275_139,
    logs: 64,
    key: 2079,
    queued: 3,
    log: 1 << 40,
    deadline: Duration::from_secs(30),
};
const STORE: jig_protocol_store::Limits = jig_protocol_store::Limits {
    core: CORE,
    store: STORE_KV,
    secrets: STORE_KV,
    commits: jig_protocol_store::sizes::Commits { in_flight: JOURNAL.commits, writes: JOURNAL.writes },
    held: CLIENTS.exchanges.checked_mul(2).expect("clients carried capacity"),
    joins: 16,
    fetch: 64 * 1024 * 1024,
    reads: 48,
    queue: 3,
    files: 8,
    checkpoints: jig_protocol_store::checkpoints::Checkpoints { keep: 2 },
    orphan_erases: 1,
};

/// Size stream edges from the wire, without reserving their byte ceilings.
pub(super) fn limits() -> Limits {
    let mut limits = LIMITS;
    limits.clients.queued_bytes = jig_protocol_engine_clients::limits::stream_queue_bytes(
        limits.clients.watches_per_stream,
        &limits.clients.wire,
    )
    .expect("standard's first deliveries fit a u32 byte bound");
    let record =
        jig_client_protocol_wire::StreamEvent::encoded_bound(&limits.clients.wire).expect("standard stream record");
    let record = u32::try_from(record).expect("standard stream record fits u32");
    // Base64 data has one `data: ` prefix and two line feeds.
    limits.clients_http.event = skein_lib::base64::encoded_len(skein_lib::base64::Alphabet::Standard, record)
        .expect("standard base64 record")
        .checked_add(8)
        .expect("standard SSE frame");
    limits
}

const RETRY: tasks::Retry = tasks::Retry { retries: 3, base: Duration::from_secs(1), max: Duration::from_secs(30) };

const RETRIES: tasks::Retries = tasks::Retries {
    transient: RETRY,
    permanent: RETRY,
    run: RETRY,
    agent: RETRY,
    lost: RETRY,
    invalid: RETRY,
    infrastructure: tasks::Retry { retries: 12, base: Duration::from_secs(10), max: Duration::from_secs(300) },
};

pub(super) const BUDGETS: core::CoreBriefBudgets = core::CoreBriefBudgets {
    tail: 2_048,
    task: 4_096,
    dependencies: 2_048,
    attempts: 1_024,
    plan: 2_048,
    notes: 2_048,
    preferences: 2_048,
    now: 4_096,
};

const REDIRECT: u32 = 16_384;

const IO: skein_io::Limits = skein_io::Limits {
    sockets: 64,
    names: 64,
    refusals: 8,
    intake: 1_048_576,
    receive: 16_384,
    output: 1_048_576,
    sends: 8,
    accepts: 4,
    backlog: 16,
    close_timeout: Duration::from_secs(30),
    retry: Duration::from_millis(250),
};
