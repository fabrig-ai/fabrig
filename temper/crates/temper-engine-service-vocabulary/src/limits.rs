//! The stock and checked field overrides (protocol/configuration/limits.md).
#[cfg(test)]
mod development;
mod names;
mod reference;
use alloc::boxed::Box;
use jig_domain_link::{JobBound, JobKind};
pub use names::{NAMES, Name, Unit, find};
pub use reference::{engine_reference, engine_reference_with, reference};
use skein_lib::{Duration, List};
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;
pub use temper_service_core::limits::{Checkout, GitComponent};

/// Root's own limits, beyond jig's child limits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Root {
    pub journal: engine::JournalLimits,
    pub loads: engine::loads::Limits,
    pub brief: engine::BriefLimits,
}
/// The clients' listener's own limits: the browser's bundle (protocol/configuration/limits.md).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Clients {
    /// The bundle's files, its page and each compressed form among them.
    pub bundle_files: u32,
    /// Both forms' bytes in all, held in memory while `temper serve` runs.
    pub bundle_bytes: u32,
}
/// People's sign-ins (protocol/sign-in.md, 6).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SignIn {
    pub members: u32,
    pub pending: u32,
    pub deadline: Duration,
    pub fetches: u32,
    pub body_bytes: u32,
    pub url_bytes: u32,
    pub secret_bytes: u32,
}
/// Repositories read in one picker page (protocol/configuration/limits.md).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Picker {
    pub page: u32,
}
/// Temper's domain limits, each under its owner's prefix.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub setup: temper_forge_setup::limits::Host,
    pub routes: Routes,
    pub records: Records,
    pub forges: Forges,
    pub forge_component: skein_forge_connection::Limits,
    pub root: Root,
    pub forge: forge::Limits,
    pub checkout: Checkout,
    pub git_component: GitComponent,
    pub clients: Clients,
    pub sign_in: SignIn,
    pub picker: Picker,
}
/// Forge routes' query bound (protocol/configuration/limits.md).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Routes {
    pub query_bytes: u32,
}
/// Routing bounds (protocol/configuration/limits.md).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Records {
    pub value_bytes: u32,
}

/// Admit a moved file on jig's wire; setup still checks temper's values against
/// `records.value_bytes` (protocol/configuration/limits.md; jig's protocol/client/README.md, 3).
#[must_use]
pub fn client_value_bytes(jig: &jig_protocol_config::Limits, own: &Limits) -> u32 {
    let value = u64::from(own.records.value_bytes).max(crate::clients::translation::moved_bytes(jig));
    u32::try_from(u64::from(jig.clients.wire.keyed_value_some).min(value)).expect("bounded by the wire ceiling")
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Forges {
    pub installations: u32,
    /// Maximum configured job-log origins per forge (protocol/configuration/limits.md).
    pub log_origins: u32,
}
pub use temper_service_core::limits::{FORGE_INSTALLATIONS, FORGE_TOKEN_BYTES, Refused};
const CLIENT: skein_forge_client::Limits = skein_forge_client::Limits {
    // Two calls in flight and one queued replacement per call.
    pending: 4,
    // Two simultaneous forge requests, shared across the six retained repositories.
    calls: 2,
    // Ten requests per second over the ten-second rate window.
    rate: 100,
    // No reserved write quota; reads and writes share the rate window.
    reserve: 0,
    // A ten-second rate window smooths the stock’s two concurrent requests.
    window: Duration::from_secs(10),
    // Jig’s stock tasks.result_bytes before payload derivation (derived.md, 5.1).
    op_bytes: 262_144,
    // Decoded answer bytes: fabrig’s development of 2026-11-16 uses 1,048,576.
    answer_bytes: 1_048_576,
    // Eight rows per read page, bounding work per request independently of retained resources.
    rows: 8,
    // Two pages of eight delivery stamps per resource before polling catches up.
    inbox: 16,
    // One read and two retries before reporting a failed request.
    read_attempts: 3,
    // One second before the first of the two read retries.
    backoff: Duration::from_secs(1),
    // The read retry delay doubles to at most four seconds.
    backoff_max: Duration::from_secs(4),
    // Eight durable writes, four times the two calls in flight.
    entries: 8,
    // One write and two retries before its resource is held.
    write_attempts: 3,
    // Room for a real forge's merge, under Forgejo's 360 s git timeout
    // (temper/docs/design/protocol/configuration/limits.md, "The forge client's lifetime and the forge call's deadline").
    lifetime: Duration::from_secs(300),
    // One second of clock movement around a forge write’s lifetime.
    clock_margin: Duration::from_secs(1),
    // Fabrig’s development of 2026-11-16: twice dev’s 1,024.
    resources: 2_048,
    // Fabrig’s development of 2026-11-16: twice dev’s 3.
    repositories: 6,
    // Five-second reads while a watched resource is active.
    poll: Duration::from_secs(5),
    // Thirty seconds between reads of a quiet resource.
    poll_max: Duration::from_secs(30),
    // One second to coalesce deliveries before their hinted read.
    hinted: Duration::from_secs(1),
    // Forty seconds between slow resource reads, above the normal polling maximum.
    slow: Duration::from_secs(40),
    // Bound Forgejo's settling of mergeability and hinted CI after a move.
    settle: Duration::from_secs(30),
    // Events first; timed reads fall back while deliveries arrive (forge/client.md, 4.1).
    fallback: Duration::from_secs(300),
    // Two eight-row pages of facts between drains.
    facts: 16,
};
const FORGE: forge::Limits = forge::Limits {
    // Forges kept at most (temper/docs/design/protocol/forge/README.md, 6).
    forges: temper_service_core::limits::stock().peer.forges,
    // Fabrig’s development of 2026-11-16: twice dev’s 3.
    repositories: 6,
    // Room for the project’s 128 participating tasks (configuration/stock.md).
    tasks: 128,
    // Fabrig’s development of 2026-11-16: 32 live landings times 5 members.
    holds: 160,
    branch_reds: 64,
    red_key_bytes: 256,
    // Fabrig’s development of 2026-11-16: twice dev’s 64.
    subscriptions: 128,
    // Eight durable writes, four times the two calls in flight.
    entries: 8,
    // Named calls retained by jig’s stock core.call_records.
    named_calls: 128,
    // Fabrig’s development of 2026-11-16: twice dev’s 379 rounded up.
    changes: 768,
    // Eight active issue projections beside the project’s changes.
    issues: 8,
    // Fabrig’s development of 2026-11-16: twice dev’s 5.
    resources_per_task: 10,
    // Four watched paths per subscription, independent of the 128 subscriptions.
    paths_per_subscription: 4,
    // engine() takes the configured authority.segments.
    authority_segments: 8,
    // Names retained by the forge connector.
    name_bytes: 64,
    // Fabrig’s development of 2026-11-16: twice dev’s 82 rounded up.
    output: 192,
    // Two eight-row pages of facts between drains.
    facts: 16,
    // One adoption per concurrent forge request.
    adoptions: 2,
    makings: 32,
    // Product capacity: keyed adoption answers retained for the request retention.
    applications: 32,
    // protocol/sign-in.md, 5: room for 64 durable member bindings.
    bindings: 64,
    // Eight collaborators per repository read, one page of the forge client.
    collaborators: 8,
    // Live landing tasks (domain/landing.md, 12).
    landings: 32,
    // Fabrig’s development of 2026-11-16: dev’s 4, capped by the worker wire’s five prefixes.
    landing_members: 5,
    landings_kept: 16,
    changes_rows: 32,
    changes_landed: 8,
    gates: 2,
    receipts_per_gate: 4,
    ignored_paths: 4,
    // Entries of a repository root read to make one landing key (domain/landing.md, 6.1).
    key_entries: 256,
    key_path_bytes: 1024,
    trees_in_flight: 2,
    tiers: temper_engine_domain_forge::TierLimits {
        tiers: 2,
        failing_tests: 16,
        // Twice the longest failing test’s id today, about 221 bytes, rounded up.
        test_id_bytes: 512,
    },
    // Fabrig’s development of 2026-11-16: twice dev’s 8.
    judge_projects: 16,
    decided_per_project: 1024,
    findings_per_project: 256,
    history_days: 14,
    history_events: 131_072,
    // Eight judge criteria in each project’s table.
    judge_criteria: 8,
    landing_rules: 4,
    follows: 8,
    dependencies: 16,
    follow_paths: 8,
    follow_queue: 64,
    follow_merge: 8,
    follow_commits: 8,
    follow_files: 32,
    follow_message_bytes: 512,
    follow_skips: 8,
    follow_reason_bytes: 256,
    roles: 4,
    setting_proposals: 4,
    setting_history: 4,
    // Two gathers of ten sections, each with a second fetch while cutting.
    brief_sections: 40,
    // Bytes of one gathered forge section: at least the root's brief.read_bytes.
    brief_bytes: BRIEF.budgets.task,
    // Forge client calls, retries and outbox room.
    client: CLIENT,
    issue_policy: temper_engine_domain_forge_issues::Limits {
        // Three plan items for each of the stock's 128 participating forge tasks.
        plan_items: 384,
        // Eight milestones per active issue projection.
        milestones: 8,
        // A 128-byte title for each issue projection.
        title_bytes: 128,
        // A 512-byte issue body preview; full bodies remain in the forge.
        body_bytes: 512,
        // A 256-byte issue comment preview; full comments remain in the forge.
        comment_bytes: 256,
        // Five seconds between an issue projection’s checks, the client’s active polling interval.
        interval: Duration::from_secs(5),
    },
    change_policy: temper_engine_domain_forge_change::Limits {
        reruns: 1,
        // One repair per task in the change's eighty-task lifetime.
        repairs: 80,
        // One resolution per task in the change's eighty-task lifetime.
        resolutions: 80,
        // One update per task in the change's eighty-task lifetime.
        updates: 80,
        // One question per task in the change's eighty-task lifetime (configuration/limits.md).
        questions: 80,
        // A change’s task lifetime at the stock’s policy count.
        tasks: 80,
        // Outlast ordinary CI; report a silent forge within the same sitting (domain/forge/change/repairs.md).
        stall: Duration::from_secs(30 * 60),
        // Execution unknown to CI has a grace before it counts (domain/forge/change/stalls.md, 2).
        unknown_grace: Duration::from_secs(30 * 60),
        // Waiting or blocked for a runner at checking has its own bound.
        runner_wait: Duration::from_secs(4 * 60 * 60),
        // Eight gates retained per change, independent of the number of live changes.
        gates: 8,
        // Eight checked heads retained per change before its oldest is replaced.
        clean_heads: 8,
    },
    // Thirty seconds to gather compatible changes into one landing.
    queue_window: Duration::from_secs(30),
    sitting: temper_engine_domain_forge::SITTING,
};
const ROOT: Root = Root {
    journal: engine::JournalLimits {
        // Three journal commit stages: queued, syncing and durable.
        commits: 3,
        // At least deliveries * 3: the root's worst case.
        held: 1536,
        // A commit's unique writes: the store commit edge, checked by Decision::write.
        writes: 12288,
        // Outputs at the journal's door, counted in the default admission watermark.
        deliveries: 512,
        // The stock’s deep task row before source derivation; derive() supplies the resume carrier.
        transcript_bytes: 22_468_264,
        // At least brief.brief_bytes and jig stock people.words: the root's worst case.
        result_bytes: BRIEF.brief_bytes,
        // Holds the stock goal agent's charter and room for an overlay.
        run_bytes: 65_536,
    },
    loads: engine::loads::Limits {
        // One owner read beside one gather’s load.
        loads: 2,
        // Two rows per load page, bounding each owner read’s work.
        rows: 2,
        // The whole note's 70,208 owned bytes and its 248-byte temper record slot.
        bytes: 70_456,
        // The same whole note and record slot fit the reply before source derivation.
        reply_bytes: 70_456,
        // One authenticated read's owned bytes; jig stock core.load_bytes.
        read_bytes: 131_072,
        // A whole stock note: body 65,536, description and pattern 256 each,
        // 256 segment places of 16 bytes, and eight references of eight bytes.
        transcript_bytes: 70_208,
    },
    brief: BRIEF,
};
const BRIEF: engine::BriefLimits = engine::BriefLimits {
    // Two simultaneous gathers; their ten sections account for forge.brief_sections.
    briefs: 2,
    // Core sections and two charter documents per gather.
    sections: 10,
    // Two source parts per brief section.
    parts: 2,
    // One section's read bytes: at least the task section's.
    read_bytes: 131_584,
    budgets: engine::BriefBudgets {
        // Bytes of each charter document section the forge prepares.
        document: 4096,
        // Task section’s bytes: a whole jig stock spec (131,072) with its contract and
        // requester lines, so an agent always reads all of what it was asked.
        task: 131_584,
        // A 128-byte dependency preview; complete dependencies stay in their document.
        dependencies: 128,
        // CI section: a failed job's last lines, with five failed checks sharing.
        ci: 4096,
        // Gate remarks share the task and pull sections' working room (domain/forge/change/repairs.md).
        reviews: 131_584,
        // A review receives a pull's title, body, files and diff, within the
        // same read bound as the task section (domain/forge/change/gates.md, 1).
        pull: 131_584,
        // A 128-byte attempt preview; complete history stays in the forge.
        attempts: 128,
        // A 128-byte plan preview; complete plans stay in their document.
        plan: 128,
        // A 64-byte notes preview; complete notes stay readable by their names.
        notes: 64,
        preferences: 2048,
        now: 4096,
    },
    // Three 131,584-byte task, pull and review sections, cut within charter 5’s
    // 319,844-byte stock host room after 64 titles and framing, with 66 bytes spare.
    brief_bytes: 319_844 - 64 * (16 + 7) - 66,
    // Fabrig’s development of 2026-11-16: 300 seconds to gather sequential forge reads.
    gather: Duration::from_secs(300),
};
const CLIENTS: Clients = Clients {
    // Files of a bundle, the page and both forms among them (client/web/README.md, 2.2).
    bundle_files: 128,
    // Bytes of both forms in all, read into memory at start (protocol/configuration/limits.md).
    bundle_bytes: 32 << 20,
};
/// The standalone process's socket admission holds standard's owned example with both listeners and a sign-in.
pub const SOCKETS: u32 = 256;

/// The stock before source derivations and document overrides.
#[must_use]
pub fn stock() -> Limits {
    let shared = temper_service_core::limits::stock();
    Limits {
        picker: Picker { page: 30 },
        routes: Routes { query_bytes: 2048 },
        setup: temper_forge_setup::limits::host(),
        records: Records { value_bytes: 4096 },
        forges: Forges { installations: FORGE_INSTALLATIONS, log_origins: 20 },
        forge_component: crate::forge::stock(&FORGE),
        root: ROOT,
        forge: FORGE,
        checkout: shared.checkout,
        git_component: shared.git_component,
        clients: CLIENTS,
        sign_in: SignIn {
            members: 64,
            pending: 64,
            deadline: Duration::from_secs(600),
            fetches: 8,
            body_bytes: 65_536,
            url_bytes: 2048,
            secret_bytes: 512,
        },
    }
}
/// Apply temper's carriers before the document's overrides (configuration/derived.md, 5.1).
pub fn derive(limits: &mut Limits, sources: temper_service_core::derived::Sources) -> Option<()> {
    use temper_service_core::derived;
    limits.checkout.message_bytes = sources.payload;
    limits.root.journal.transcript_bytes = derived::resume(sources.turn)?;
    limits.root.loads.transcript_bytes = sources.turn;
    let record = u32::try_from(size_of::<engine::Record>()).ok()?;
    limits.root.loads.bytes = derived::loaded(sources.turn, record)?;
    limits.root.loads.reply_bytes = derived::loaded(sources.turn, record)?;
    limits.root.loads.read_bytes = derived::read(
        sources.turn,
        u64::try_from(size_of::<jig_domain_core::reads::Row>()).ok()?,
        u64::try_from(size_of::<jig_domain_core::Record>()).ok()?,
    )?;
    limits.forge.client.op_bytes = derived::message(sources.payload)?;
    limits.forge_component.body_bytes = derived::forge_body(limits.forge.client.op_bytes)?;
    let answer = derived::forge_answer(&limits.forge_component, limits.forge.client.answer_bytes)?;
    apply(limits, b"forge.client.answer_bytes", u64::from(answer)).ok()?;
    Some(())
}
/// Set one dotted field name byte for byte.
pub fn apply(limits: &mut Limits, name: &[u8], value: u64) -> Result<(), Refused> {
    let lifetime = limits.forge.client.lifetime;
    names::apply(limits, name, value)?;
    if name == b"forge.client.lifetime_ms" && limits.forge_component.deadline == crate::forge::deadline(lifetime) {
        limits.forge_component.deadline = crate::forge::deadline(limits.forge.client.lifetime);
    }
    if name == b"forge.client.calls" || name == b"forge.client.answer_bytes" {
        limits.forge_component.kept_bytes = u64::from(limits.forge.client.calls)
            .checked_mul(u64::from(limits.forge.client.answer_bytes))
            .expect("two u32 limits fit u64");
    }
    Ok(())
}
/// Every applied limit by dotted name (temper/docs/design/protocol/configuration/figures.md, 2).
#[must_use]
pub fn list(limits: &Limits) -> List<(&'static str, u64)> {
    let shared = temper_service_core::limits::Shared {
        checkout: limits.checkout,
        git_component: limits.git_component,
        peer: temper_service_core::limits::Peer {
            forges: limits.forge.forges,
            installations: limits.forges.installations,
            token_bytes: limits.forge_component.token_bytes,
        },
    };
    let mut values = temper_service_core::limits::list(&shared);
    for name in &NAMES {
        if !temper_service_core::limits::shared_name(name.name.as_bytes()) {
            values.push((name.name, (name.value)(limits))).expect("uncapped named limits");
        }
    }
    values
}

/// Build the root's limits from jig's children and temper's own room.
#[must_use]
pub fn engine(jig: &jig_protocol_config::Limits, limits: &Limits) -> Option<engine::Limits> {
    let root = engine_candidate(jig, limits);
    if !engine::valid(&root) {
        return None;
    }
    Some(root)
}

/// The same effective projection, before sizing can refuse it anonymously.
#[must_use]
pub fn engine_candidate(jig: &jig_protocol_config::Limits, limits: &Limits) -> engine::Limits {
    engine::Limits {
        streams: jig.clients.streams,
        routes: jig.routes,
        // The budget is the engine's own memory_bytes, set where it starts.
        memory_bytes: u64::MAX,
        watermark: engine::default_watermark(&limits.root.journal),
        public_watermark: engine::default_watermark(&limits.root.journal).checked_div(2).expect("half the watermark"),
        public_reads: jig.clients.public.reads,
        public: jig.core.public,
        retention: jig.core.retention,
        journal: limits.root.journal,
        loads: limits.root.loads,
        brief: limits.root.brief,
        tasks: jig.core.tasks,
        authority: jig.core.authority,
        people: jig.core.people,
        fleet: jig.core.fleet,
        accounts: jig.core.accounts,
        notes: jig.core.notes,
        documents: jig.core.documents,
        templates: jig.core.templates,
        policy_bytes: jig.core.policy_bytes,
        call_answer_bytes: jig.core.call_answer_bytes,
        host_reply_bytes: None,
        settings: jig.core.settings,
        call_records: jig.core.call_records,
        forge: forge::Limits {
            authority_segments: jig.core.authority.segments,
            roles: jig.core.authority.roles,
            setting_proposals: jig.core.settings.proposals,
            setting_history: jig.core.settings.history,
            ..limits.forge
        },
    }
}

/// Limits decoded by the setup and box owners, kept with their values.
#[must_use]
pub fn handed_on(name: &[u8]) -> bool {
    (name.starts_with(b"setup.") && name != b"setup.runs" && name != b"setup.states" && name != b"setup.state_life_ms")
        || name.starts_with(b"box.")
}
/// Setup bounds consumed by the engine's host.
#[must_use]
pub fn host_setup(name: &[u8]) -> bool {
    [
        b"setup.deadline_ms".as_slice(),
        b"setup.backoff_ms",
        b"setup.backoff_max_ms",
        b"setup.runs",
        b"setup.states",
        b"setup.state_life_ms",
    ]
    .contains(&name)
}

/// Server connections admit the mount's calls, with at least the stock’s eight connections.
#[must_use]
pub fn server(jig: &jig_protocol_config::Limits) -> crate::server::Limits {
    crate::server::Limits {
        connections: jig.clients_http.calls.max(8),
        http: skein_http::server::Limits {
            head: 4096,
            headers: 32,
            body: u64::from(jig.clients_http.body),
            read: jig.clients_http.read,
            response: 4096,
            send: jig.clients_http.send,
        },
        stall: Duration::from_secs(30),
        peer_end: crate::server::PeerEnd::Connection,
    }
}

pub use temper_service_core::limits::{
    CLOSE_DEADLINE, CLOSE_RESULT_BYTES, CLOSE_SPEC_BYTES, MERGE_DEADLINE, MERGE_KIND, MERGE_RESULT_BYTES,
    MERGE_SPEC_BYTES, TOOLING_PATH, merge_bound, merge_jobs, prepare_bound,
};

/// Declare prepare only where the tasks child holds its bounds
/// (temper/docs/design/domain/worker/jobs.md).
#[must_use]
pub fn prepare_jobs(jig: &jig_protocol_config::Limits) -> Box<[JobBound]> {
    let bound = prepare_bound();
    if bound.spec <= jig.core.tasks.spec_bytes && bound.result <= jig.core.tasks.result_bytes {
        Box::new([bound])
    } else {
        Box::new([])
    }
}

/// The close job's kind and bounds (temper/docs/design/domain/worker/jobs.md).
#[must_use]
pub fn close_bound() -> JobBound {
    JobBound {
        kind: JobKind(temper_engine_domain_forge::CLOSE_KIND),
        spec: CLOSE_SPEC_BYTES,
        result: CLOSE_RESULT_BYTES,
        deadline: CLOSE_DEADLINE,
    }
}

/// Declare close only where the tasks child holds its bounds
/// (temper/docs/design/domain/worker/jobs.md).
#[must_use]
pub fn close_jobs(jig: &jig_protocol_config::Limits) -> Box<[JobBound]> {
    let bound = close_bound();
    if bound.spec <= jig.core.tasks.spec_bytes && bound.result <= jig.core.tasks.result_bytes {
        Box::new([bound])
    } else {
        Box::new([])
    }
}

/// The engine's declared kinds, ascending (temper/docs/design/domain/worker/jobs.md).
#[must_use]
pub fn jobs(jig: &jig_protocol_config::Limits) -> Box<[JobBound]> {
    let mut jobs = List::with_capacity(3);
    for bound in prepare_jobs(jig).iter().chain(merge_jobs(jig).iter()).chain(close_jobs(jig).iter()) {
        jobs.push(*bound).expect("three built job kinds");
    }
    jobs.into_boxed()
}

#[cfg(test)]
mod making_tests {
    #[test]
    fn retention_overrides_reach_the_effective_core_limits() {
        let mut jig = jig_protocol_config::stock();
        for (name, value) in [
            (b"retention.turns_ms".as_slice(), 13),
            (b"retention.records_ms".as_slice(), 23),
            (b"retention.summaries_ms".as_slice(), 37),
        ] {
            jig_protocol_config::overrides::apply(&mut jig, name, value).unwrap();
        }
        let root = engine(&jig, &stock()).expect("coherent projected limits");
        assert_eq!(root.retention, jig.core.retention);
        assert_eq!(engine::core_limits(&root).retention, jig.core.retention);
    }

    #[test]
    fn standard_forge_calls_have_room_for_a_merge() {
        let limits = stock();
        assert_eq!(limits.forge.client.lifetime, Duration::from_secs(300));
        assert_eq!(limits.forge_component.deadline, Duration::from_secs(270));
    }

    #[test]
    fn a_shorter_forge_lifetime_moves_the_derived_deadline() {
        let mut limits = stock();
        assert_eq!(apply(&mut limits, b"forge.client.lifetime_ms", 100_000), Ok(()));
        assert_eq!(limits.forge.client.lifetime, Duration::from_secs(100));
        assert_eq!(limits.forge_component.deadline, Duration::from_secs(100));
    }

    #[test]
    fn a_longer_forge_lifetime_keeps_the_github_deadline_cap() {
        let mut limits = stock();
        assert_eq!(apply(&mut limits, b"forge.client.lifetime_ms", 600_000), Ok(()));
        assert_eq!(limits.forge.client.lifetime, Duration::from_secs(600));
        assert_eq!(limits.forge_component.deadline, Duration::from_secs(270));
    }

    #[test]
    fn a_forge_deadline_override_before_the_lifetime_stays() {
        let mut limits = stock();
        assert_eq!(apply(&mut limits, b"forge_component.deadline_ms", 60_000), Ok(()));
        assert_eq!(apply(&mut limits, b"forge.client.lifetime_ms", 120_000), Ok(()));
        assert_eq!(limits.forge.client.lifetime, Duration::from_secs(120));
        assert_eq!(limits.forge_component.deadline, Duration::from_secs(60));
    }

    #[test]
    fn a_forge_deadline_override_after_the_lifetime_stays() {
        let mut limits = stock();
        assert_eq!(apply(&mut limits, b"forge.client.lifetime_ms", 120_000), Ok(()));
        assert_eq!(apply(&mut limits, b"forge_component.deadline_ms", 60_000), Ok(()));
        assert_eq!(limits.forge.client.lifetime, Duration::from_secs(120));
        assert_eq!(limits.forge_component.deadline, Duration::from_secs(60));
    }

    #[test]
    fn checking_clock_limits_default_parse_and_are_named() {
        let maximum = u64::MAX / 1_000_000;
        {
            let stock = stock();
            assert_eq!(stock.forge.change_policy.unknown_grace, Duration::from_secs(1800));
            assert_eq!(stock.forge.change_policy.runner_wait, Duration::from_secs(14_400));
            for (name, default) in [
                (b"forge.change_policy.unknown_grace_ms".as_slice(), 1_800_000),
                (b"forge.change_policy.runner_wait_ms".as_slice(), 14_400_000),
            ] {
                let entry = find(name).expect("checking limit named");
                assert_eq!((entry.value)(&stock), default);
                assert_eq!(entry.unit, Unit::Milliseconds);
                assert!(temper_service_core::limits::known(name));
                for millis in [0, 1, maximum] {
                    let mut limits = stock;
                    assert_eq!(apply(&mut limits, name, millis), Ok(()));
                    let policy = limits.forge.change_policy;
                    let field = if name == b"forge.change_policy.unknown_grace_ms" {
                        policy.unknown_grace
                    } else {
                        policy.runner_wait
                    };
                    assert_eq!(field, Duration::from_millis(millis));
                    assert_eq!((entry.value)(&limits), millis);
                }
                for millis in [maximum + 1, u64::MAX] {
                    let mut limits = stock;
                    assert_eq!(apply(&mut limits, name, millis), Err(Refused::Range));
                    assert_eq!(limits.forge.change_policy, stock.forge.change_policy);
                }
            }
        }
    }

    #[test]
    fn forge_fallback_defaults_and_dotted_overrides() {
        {
            let mut limits = stock();
            assert_eq!(limits.forge.client.fallback, Duration::from_secs(300));
            for millis in [0, 1, 300_000] {
                assert_eq!(apply(&mut limits, b"forge.client.fallback_ms", millis), Ok(()));
                assert_eq!(limits.forge.client.fallback, Duration::from_millis(millis));
            }
        }
    }
    use super::*;
    #[test]
    fn making_limits_default_to_thirty_two() {
        {
            assert_eq!(stock().forge.makings, 32);
        }
    }
    #[test]
    fn making_limits_parse_at_both_bounds() {
        let mut limits = stock();
        for count in [1, 64] {
            assert_eq!(apply(&mut limits, b"forge.makings", count), Ok(()));
            assert_eq!(u64::from(limits.forge.makings), count);
            assert!(forge::valid(&limits.forge));
        }
    }
    #[test]
    fn making_limits_refuse_both_sides_outside_the_bound() {
        let mut limits = stock();
        let before = limits.forge.makings;
        for count in [0, 65, u64::MAX] {
            assert_eq!(apply(&mut limits, b"forge.makings", count), Err(Refused::Range));
            assert_eq!(limits.forge.makings, before);
        }
        for count in [0, 65] {
            let mut limits = limits.forge;
            limits.makings = count;
            assert!(!forge::valid(&limits));
        }
    }
}

#[cfg(test)]
mod key_entry_tests {
    use super::*;
    #[test]
    fn a_landing_key_reads_a_real_root_and_takes_a_dotted_override() {
        {
            let limits = stock();
            assert_eq!(limits.forge.key_entries, 256);
            for value in [1, 17, 4096] {
                let mut changed = limits;
                apply(&mut changed, b"forge.key_entries", value).unwrap();
                assert_eq!((find(b"forge.key_entries").unwrap().value)(&changed), value);
                assert!(forge::valid(&changed.forge));
            }
            let mut changed = limits;
            assert_eq!(apply(&mut changed, b"forge.key_entries", 0), Err(Refused::Range));
            assert_eq!(changed.forge.key_entries, 256);
        }
    }
}

#[cfg(test)]
mod red_tests {
    use super::*;
    #[test]
    fn goal_red_limits_have_stock_values_and_bounded_dotted_overrides() {
        {
            let limits = stock();
            assert_eq!((limits.forge.branch_reds, limits.forge.red_key_bytes), (64, 256));
            for (name, most) in [(b"forge.branch_reds".as_slice(), 256), (b"forge.red_key_bytes".as_slice(), 512)] {
                for value in [1, most] {
                    let mut changed = limits;
                    apply(&mut changed, name, value).unwrap();
                    assert_eq!((find(name).unwrap().value)(&changed), value);
                    assert!(forge::valid(&changed.forge));
                }
                for value in [0, most + 1, u64::MAX] {
                    let mut changed = limits;
                    assert_eq!(apply(&mut changed, name, value), Err(Refused::Range));
                    assert_eq!((find(name).unwrap().value)(&changed), (find(name).unwrap().value)(&limits));
                }
            }
            for value in [0, 257] {
                assert!(!forge::valid(&forge::Limits { branch_reds: value, ..limits.forge }));
            }
            for value in [0, 513] {
                assert!(!forge::valid(&forge::Limits { red_key_bytes: value, ..limits.forge }));
            }
            assert!(!forge::valid(&forge::Limits { holds: u32::MAX, ..limits.forge }));
        }
    }
}
