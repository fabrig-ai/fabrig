//! The small and standard profiles and checked field overrides (protocol/configuration/limits.md).
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
pub use temper_service_core::limits::{FORGE_INSTALLATIONS, FORGE_TOKEN_BYTES, Profile, Refused};
const CLIENT: skein_forge_client::Limits = skein_forge_client::Limits {
    // Logical calls retained, queued and in flight, from the engine forge world fixture.
    pending: 4,
    // Calls in flight, from the engine forge world fixture.
    calls: 2,
    // Requests allowed per rate window, from the engine forge world fixture.
    rate: 100,
    // Requests reserved for writes, from the engine forge world fixture.
    reserve: 0,
    // Rate window duration, from the engine forge world fixture.
    window: Duration::from_secs(10),
    // Owned bytes of one operation, from the engine forge world fixture.
    op_bytes: 512,
    // Decoded bytes of one answer: the small profile's 65,536-byte ceiling.
    answer_bytes: 65_536,
    // Rows in one answer page, from the engine forge world fixture.
    rows: 8,
    // Comment and review delivery stamps per resource, from the engine forge world fixture.
    inbox: 16,
    // Attempts of a read, from the engine forge world fixture.
    read_attempts: 3,
    // First read retry delay, from the engine forge world fixture.
    backoff: Duration::from_secs(1),
    // Maximum read retry delay, from the engine forge world fixture.
    backoff_max: Duration::from_secs(4),
    // Durable outbox entries, from the engine forge world fixture.
    entries: 8,
    // Attempts of a write, from the engine forge world fixture.
    write_attempts: 3,
    // Longest time a sent write may arrive, from the engine forge world fixture.
    lifetime: Duration::from_secs(10),
    // Allowance for clock movement, from the engine forge world fixture.
    clock_margin: Duration::from_secs(1),
    // Three chat branches, six change names, two followed tier branches and
    // four departing reads fit sixteen slots (domain/forge/keeping-up.md, 1).
    resources: 16,
    // Repositories retained, from the engine forge world fixture.
    repositories: 2,
    // Normal resource polling interval, from the engine forge world fixture.
    poll: Duration::from_secs(5),
    // Maximum resource polling interval, from the engine forge world fixture.
    poll_max: Duration::from_secs(30),
    // Delay before a hinted read, from the engine forge world fixture.
    hinted: Duration::from_secs(1),
    // Delay before a slow read, from the engine forge world fixture.
    slow: Duration::from_secs(40),
    // Bound Forgejo's settling of mergeability and hinted CI after a move.
    settle: Duration::from_secs(30),
    // Events first; timed reads fall back while deliveries arrive (forge/client.md, 4.1).
    fallback: Duration::from_secs(300),
    // Facts awaiting drain, from the engine forge world fixture.
    facts: 16,
};
const FORGE: forge::Limits = forge::Limits {
    // Forges kept at most (temper/docs/design/protocol/forge/README.md, 6).
    forges: temper_service_core::limits::profile(Profile::Small).peer.forges,
    // Repositories retained, from the engine forge world fixture.
    repositories: 2,
    // Tasks participating in the forge, from the engine forge world fixture.
    tasks: 8,
    // Held task resources, from the engine forge world fixture.
    holds: 8,
    // Subscriptions to forge topics, from the engine forge world fixture.
    subscriptions: 8,
    // Durable outbox entries, from the engine forge world fixture.
    entries: 8,
    // At least jig small core.call_records: the root's worst case.
    named_calls: 32,
    // Live changes, from the engine forge world fixture.
    changes: 8,
    // Live issue projections, from the engine forge world fixture.
    issues: 8,
    // A goal watches its held g and r branches and landings on main.
    resources_per_task: 3,
    // Paths retained by one subscription, from the engine forge world fixture.
    paths_per_subscription: 4,
    // Direct connector fixture; engine() takes the configured authority.segments.
    authority_segments: 8,
    // Names retained by the forge connector.
    name_bytes: 64,
    // Output records reserved per forge step, from the engine forge world fixture.
    output: 80,
    // Facts awaiting drain, from the engine forge world fixture.
    facts: 16,
    // Adoptions in flight, from the engine forge world fixture.
    adoptions: 2,
    makings: 32,
    // Product capacity: keyed adoption answers retained for the request retention.
    applications: 32,
    // protocol/sign-in.md, 5: small permits 64 durable member bindings.
    bindings: 64,
    // Collaborators kept by the forge, from the engine forge world fixture.
    collaborators: 8,
    // Live landing tasks (domain/landing.md, 12).
    landings: 2,
    landing_members: 2,
    landings_kept: 16,
    changes_rows: 32,
    changes_landed: 8,
    gates: 2,
    receipts_per_gate: 4,
    ignored_paths: 4,
    key_entries: 16,
    key_path_bytes: 1024,
    trees_in_flight: 2,
    tiers: temper_engine_domain_forge::TierLimits { tiers: 2, failing_tests: 16, test_id_bytes: 256 },
    // Project judge tables, from the engine forge world fixture.
    judge_projects: 2,
    decided_per_project: 1024,
    findings_per_project: 256,
    history_days: 14,
    history_events: 131_072,
    // Criteria in each judge table, from the engine forge world fixture.
    judge_criteria: 8,
    landing_rules: 4,
    follows: 2,
    dependencies: 4,
    follow_paths: 8,
    follow_queue: 16,
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
    brief_bytes: 4608,
    // Forge client calls, retries and outbox room.
    client: CLIENT,
    issue_policy: temper_engine_domain_forge_issues::Limits {
        // Items in an issue plan, from the engine forge world fixture.
        plan_items: 8,
        // Milestones in an issue projection, from the engine forge world fixture.
        milestones: 8,
        // Bytes in an issue title, from the engine forge world fixture.
        title_bytes: 128,
        // Bytes in an issue body, from the engine forge world fixture.
        body_bytes: 512,
        // Bytes in an issue comment, from the engine forge world fixture.
        comment_bytes: 256,
        // Interval between issue projection checks, from the engine forge world fixture.
        interval: Duration::from_secs(5),
    },
    change_policy: temper_engine_domain_forge_change::Limits {
        reruns: 1,
        // Repair attempts, from the engine forge world fixture.
        repairs: 3,
        // Resolution attempts, from the engine forge world fixture.
        resolutions: 3,
        // Update attempts, from the engine forge world fixture.
        updates: 3,
        // The reference's bound (draft/reference-code-production.md, section 8).
        questions: 4,
        // Small's producer, a review, and two repairs each followed by its review.
        tasks: 6,
        // Outlast ordinary CI; report a silent forge within the same sitting (domain/forge/change/repairs.md).
        stall: Duration::from_secs(30 * 60),
        // Gates retained by a change, from the engine forge world fixture.
        gates: 8,
        // Clean branch heads retained, from the engine forge world fixture.
        clean_heads: 8,
    },
    // Time to gather a landing queue, from the engine forge world fixture.
    queue_window: Duration::from_secs(30),
    sitting: temper_engine_domain_forge::SITTING,
};
// A project of fabrig's scale (temper/docs/design/protocol/configuration/stock.md).
const STANDARD_FORGE: forge::Limits = forge::Limits {
    follows: 8,
    dependencies: 16,
    follow_queue: 64,
    // One gathered section shares the root task section's read bound.
    brief_bytes: STANDARD_BRIEF.budgets.task,
    // Tasks following live changes in a project, 18 changes seen.
    tasks: 128,
    // Holds for live changes in a project, 18 seen.
    holds: 128,
    // Live changes in a project, 18 seen.
    changes: 128,
    // Named calls retained by jig standard core.call_records.
    named_calls: 128,
    // Live landings alongside the project's changes.
    landings: 32,
    client: skein_forge_client::Limits {
        // Watched resources, at least one for each held branch.
        resources: 128,
        // Whole task results, as jig-protocol-config/src/profile/standard.rs sets tasks.result_bytes.
        op_bytes: 262_144,
        ..CLIENT
    },
    change_policy: temper_engine_domain_forge_change::Limits {
        reruns: 1,
        // Questions per change or task, 15 seen.
        questions: 80,
        // Tasks across a change's runs and retries, 14 runs seen.
        tasks: 80,
        // Repair attempts across a change's runs, 14 seen.
        repairs: 80,
        // Resolution attempts across a change's runs, 14 seen.
        resolutions: 80,
        // Update attempts across a change's runs, 14 seen.
        updates: 80,
        ..FORGE.change_policy
    },
    issue_policy: temper_engine_domain_forge_issues::Limits {
        // Tasks in one goal's lifetime plan, 73 seen.
        plan_items: 384,
        ..FORGE.issue_policy
    },
    ..FORGE
};
/// Root room follows the domain world; byte carriers take jig's small profile
/// bounds where its children require more than the world's reduced fixtures.
const ROOT: Root = Root {
    journal: engine::JournalLimits {
        // Commits awaiting durability, from the root world.
        commits: 3,
        // At least deliveries * 3: the root's worst case.
        held: 1536,
        // A commit's unique writes: the store commit edge, checked by Decision::write.
        writes: 12288,
        // Outputs at the journal's door, counted in the default admission watermark.
        deliveries: 512,
        // The 65,536-byte forge answer dominates small's deep task row,
        // including its four AuthorityExecutor values (engine::row_bound).
        transcript_bytes: 65_536,
        // At least brief.brief_bytes and jig small people.words: the root's worst case.
        result_bytes: 14_592,
        // Holds the stock goal agent's charter and room for an overlay.
        run_bytes: 65_536,
    },
    loads: engine::loads::Limits {
        // Concurrent loads, from the root world.
        loads: 2,
        // Rows in one page, from the root world.
        rows: 2,
        // Kept page bytes in loads' worst case; jig small core.load_bytes.
        bytes: 65_536,
        // At least loads.bytes: loads' worst case; jig small core.load_bytes.
        reply_bytes: 65_536,
        // One authenticated read's owned bytes; jig small core.load_bytes.
        read_bytes: 65_536,
        // Deep row admission bound for the root's largest forge.client.answer_bytes.
        transcript_bytes: 65_536,
    },
    brief: engine::BriefLimits {
        // Concurrent gathers, from the root world.
        briefs: 2,
        // Core sections and two charter documents per gather.
        sections: 10,
        // Parts per section, from the root world.
        parts: 2,
        // One section's read bytes: at least the task section's.
        read_bytes: 4608,
        budgets: engine::BriefBudgets {
            // Bytes of each charter document section the forge prepares.
            document: 4096,
            // Task section's bytes: a whole jig small spec (4096) with its contract and
            // requester lines, so an agent always reads all of what it was asked.
            task: 4608,
            // Dependencies section's bytes, from the root world.
            dependencies: 128,
            // CI section: a failed job's last lines, with five failed checks sharing.
            ci: 4096,
            // Gate remarks share the task and pull sections' working room (domain/forge/change/repairs.md).
            reviews: 4608,
            // A review receives a pull's title, body, files and diff, within the
            // same read bound as the task section (domain/forge/change/gates.md, 1).
            pull: 4608,
            // Attempts section's bytes, from the root world.
            attempts: 128,
            // Plan section's bytes, from the root world.
            plan: 128,
            // Notes section's bytes, from the root world.
            notes: 64,
            preferences: 2048,
            now: 4096,
        },
        // Whole task, pull and review sections, with 768 bytes for other context.
        brief_bytes: 14_592,
        // Outlast sequential forge reads and a retry; bound a stuck connector (domain/root.md, 9).
        gather: Duration::from_secs(30),
    },
};
// Standard carries a whole jig standard spec (131,072) plus 512 bytes for its
// contract and requester, as small carries 4,096 plus 512.
const STANDARD_BRIEF: engine::BriefLimits = engine::BriefLimits {
    read_bytes: 131_584,
    budgets: engine::BriefBudgets {
        task: 131_584,
        // Pull and reviews share the task section's read bound, as in small.
        pull: 131_584,
        reviews: 131_584,
        ..ROOT.brief.budgets
    },
    // tests/stock_brief.rs derives charter 5's room (337,624) less 1,472
    // bytes for 64 titles and framing and 400 margin; below 395,520.
    brief_bytes: 335_752,
    ..ROOT.brief
};
const STANDARD_ROOT: Root = Root {
    loads: engine::loads::Limits {
        // A whole note: body 65,536 + description 256 + pattern bytes 256,
        // 256 pattern segment slots * 16 and 8 references * 8 = 70,208.
        transcript_bytes: 70_208,
        // The row's owned bytes plus its 248-byte record slot.
        bytes: 70_456,
        reply_bytes: 70_456,
        read_bytes: 131_072,
        ..ROOT.loads
    },
    brief: STANDARD_BRIEF,
    journal: engine::JournalLimits {
        // Deep task row at jig standard's bounds (engine::row_bound):
        // 22,467,664 bytes for the task, inbox, later choice and two reason sets,
        // plus 21 executor places * 8 bytes per AuthorityExecutor = 22,467,832.
        transcript_bytes: 22_467_832,
        result_bytes: STANDARD_BRIEF.brief_bytes,
        ..ROOT.journal
    },
};
const CLIENTS: Clients = Clients {
    // Files of a bundle, the page and both forms among them (client/web/README.md, 2.2).
    bundle_files: 128,
    // Bytes of both forms in all, read into memory at start (protocol/configuration/limits.md).
    bundle_bytes: 32 << 20,
};
/// The standalone process's socket admission holds standard's owned example with both listeners and a sign-in.
pub const SOCKETS: u32 = 256;

/// Constant profile values in each owner's limits.
#[must_use]
pub fn profile(profile: Profile) -> Limits {
    let shared = temper_service_core::limits::profile(profile);
    let mut limits = match profile {
        Profile::Standard => Limits { root: STANDARD_ROOT, forge: STANDARD_FORGE, ..self::profile(Profile::Small) },
        Profile::Small => Limits {
            picker: Picker { page: 30 },
            routes: Routes { query_bytes: 2048 },
            setup: temper_forge_setup::limits::host(),
            records: Records { value_bytes: 4096 },
            forges: Forges { installations: FORGE_INSTALLATIONS, log_origins: 20 },
            forge_component: crate::forge::small(&FORGE),
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
        },
    };
    limits.checkout = shared.checkout;
    limits.git_component = shared.git_component;
    limits.forge.forges = shared.peer.forges;
    limits.forges.installations = shared.peer.installations;
    limits.forge_component.token_bytes = shared.peer.token_bytes;
    limits
}
/// Set one dotted field name byte for byte.
pub fn apply(limits: &mut Limits, name: &[u8], value: u64) -> Result<(), Refused> {
    names::apply(limits, name, value)?;
    if name == b"forge.client.calls" || name == b"forge.client.answer_bytes" {
        limits.forge_component.kept_bytes = u64::from(limits.forge.client.calls)
            .checked_mul(u64::from(limits.forge.client.answer_bytes))
            .expect("two u32 limits fit u64");
    }
    Ok(())
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

/// Server connections admit the mount's calls, with at least small's eight connections.
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
    fn forge_fallback_defaults_and_dotted_overrides() {
        for selected in [Profile::Small, Profile::Standard] {
            let mut limits = profile(selected);
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
        for selected in [Profile::Small, Profile::Standard] {
            assert_eq!(profile(selected).forge.makings, 32);
        }
    }
    #[test]
    fn making_limits_parse_at_both_bounds() {
        let mut limits = profile(Profile::Small);
        for count in [1, 64] {
            assert_eq!(apply(&mut limits, b"forge.makings", count), Ok(()));
            assert_eq!(u64::from(limits.forge.makings), count);
            assert!(forge::valid(&limits.forge));
        }
    }
    #[test]
    fn making_limits_refuse_both_sides_outside_the_bound() {
        let mut limits = profile(Profile::Small);
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
