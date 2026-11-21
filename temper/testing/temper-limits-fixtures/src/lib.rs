//! Small limits for tests alone (temper/docs/design/protocol/configuration/limits.md).
#![no_std]
extern crate alloc;
pub mod root_resume;
pub mod start_forge;
use skein_lib::Duration;
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;
use temper_engine_service_vocabulary::limits::{
    Checkout, Clients, Forges, GitComponent, Limits, Picker, Records, Root, Routes, SignIn,
};
use temper_service_core::limits::{FORGE_INSTALLATIONS, FORGE_TOKEN_BYTES, Peer, Shared};
const CHECKOUT: Checkout = Checkout {
    // Cached or held checkout workspaces, from the checkout world fixture.
    workspaces: 3,
    // One preparation at a time per worker: redeploys.md, 6.4.
    preparations: 1,
    // Held to the agent's two repository directories.
    repositories: 2,
    // Bytes in a checkout key, name, remote or branch.
    name_bytes: 256,
    // Holds the agent's largest commit-message argument.
    message_bytes: 4096,
    // Conflicted paths per repository, for a merge of a few hundred paths.
    conflicts: 512,
    // Bytes per conflicted path, with its NUL in the git component's stdout.
    path_bytes: 255,
    // Time for an operation reaching a forge, from the checkout world fixture.
    remote_timeout: Duration::from_secs(60),
    // Time for a local checkout operation, from the checkout world fixture.
    local_timeout: Duration::from_secs(10),
    reds_read_ms: 5_000,
    rerun_processes: 4,
    // Facts awaiting drain, from the checkout world fixture.
    facts: 256,
};
const GIT: GitComponent = GitComponent {
    // One operation per cached workspace, plus the component's startup and sweep.
    operations: 3,
    // At least every conflict path including its terminating NUL.
    stdout_bytes: 131_072,
    // The tail retained for classification and the checkout's diagnostic.
    stderr_bytes: 512,
    // Each stdout/stderr demand, bounded further by io's read size.
    read_bytes: 4096,
    // One page of trash entries held while sweeping.
    sweep_entries: 16,
    // Every name in that page at the kernel's longest name.
    sweep_bytes: 4080,
};

/// Small checkout and git capacities for worlds, before source derivations.
#[must_use]
pub const fn shared() -> Shared {
    Shared {
        checkout: CHECKOUT,
        git_component: GIT,
        peer: Peer { forges: 3, installations: FORGE_INSTALLATIONS, token_bytes: FORGE_TOKEN_BYTES },
    }
}
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
    // Room for a real forge's merge, under Forgejo's 360 s git timeout
    // (temper/docs/design/protocol/configuration/limits.md, "The forge client's lifetime and the forge call's deadline").
    lifetime: Duration::from_secs(300),
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
    forges: shared().peer.forges,
    // Repositories retained, from the engine forge world fixture.
    repositories: 2,
    // Tasks participating in the forge, from the engine forge world fixture.
    tasks: 8,
    // Held task resources, from the engine forge world fixture.
    holds: 8,
    branch_reds: 64,
    red_key_bytes: 256,
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
    landings_pending: 64,
    changes_rows: 32,
    changes_landed: 8,
    gates: 2,
    receipts_per_gate: 4,
    ignored_paths: 4,
    // Entries of one tree a landing key reads: a real repository's root runs past the
    // fixture's 16 (domain/landing.md, 6.1); forge.key_entries overrides it.
    key_entries: 256,
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
        // Execution unknown to CI has a grace before it counts (domain/forge/change/stalls.md, 2).
        unknown_grace: Duration::from_secs(30 * 60),
        // Waiting or blocked for a runner at checking has its own bound.
        runner_wait: Duration::from_secs(4 * 60 * 60),
        // Gates retained by a change, from the engine forge world fixture.
        gates: 8,
        // Clean branch heads retained, from the engine forge world fixture.
        clean_heads: 8,
    },
    // Time to gather a landing queue, from the engine forge world fixture.
    queue_window: Duration::from_secs(30),
    sitting: temper_engine_domain_forge::SITTING,
};
const ROOT: Root = Root {
    transcript_memory: u64::MAX,
    journal: engine::JournalLimits {
        // Commits awaiting durability, from the root world.
        commits: 3,
        // At least deliveries * 3: the root's worst case.
        held: 1536,
        // A commit's unique writes: the store commit edge, checked by Decision::write.
        writes: 12288,
        // Outputs at the journal's door, counted in the default admission watermark.
        deliveries: 512,
        // A changes document: 576 document + 64 landing bytes;
        // 8 queues * (208 queue + 2,048 repository + 64 branch + 120 queued row
        // + 256 title + 3 * 256 provenance) = 27,712 bytes;
        // two 8-row histories * (160 row + 256 title + 3 * 256 provenance
        // + 8 * (64 sign-off + 256 name)) = 59,904 bytes; total 88,256.
        // Small's changes-document floor (82,112 without chats).
        // The 65,536-byte forge answer dominates small's deep task row,
        // including its four AuthorityExecutor values and 56-byte Option<OpenAttempt> (engine::row_bound).
        transcript_bytes: 88_256,
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
const CLIENTS: Clients = Clients {
    // Files of a bundle, the page and both forms among them (client/web/README.md, 2.2).
    bundle_files: 128,
    // Bytes of both forms in all, read into memory at start (protocol/configuration/limits.md).
    bundle_bytes: 32 << 20,
};
/// The application's small figures, without stock-agent derivations.
#[must_use]
pub fn small() -> Limits {
    let shared = shared();
    Limits {
        live_goals: 40,
        tool_payload: 63_740,
        picker: Picker { page: 30 },
        routes: Routes { query_bytes: 2048 },
        setup: temper_forge_setup::limits::host(),
        records: Records { value_bytes: 4096 },
        forges: Forges { installations: FORGE_INSTALLATIONS, log_origins: 20 },
        forge_component: temper_engine_service_vocabulary::forge::stock(&FORGE),
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

/// Small box figures.
#[must_use]
pub fn box_small() -> temper_box_machine::limits::Limits {
    temper_box_machine::limits::Limits {
        backoff: Duration::from_millis(500),
        backoff_max: Duration::from_secs(30),
        restarts: 5,
        window: Duration::from_secs(300),
        grace: Duration::from_secs(10),
        line_bytes: 256,
        log_bytes: 4_194_304,
        tail_lines: 10,
        groups: 8,
    }
}

/// Small setup plan figures.
#[must_use]
pub fn setup_small() -> skein_forge_setup::limits::Limits {
    // Legacy command default; checked startup supplies forge.client.repositories.
    let repositories = 2;
    skein_forge_setup::limits::Limits {
        orgs: temper_forge_setup::limits::ORGS,
        reachable: 8, // Legacy command default; checked startup supplies picker.page.
        files: 3,
        file_bytes: 4096,
        repositories,
        deadline: Duration::from_secs(120),
        copy_deadline: temper_forge_setup::limits::COPY_DEADLINE,
        backoff: Duration::from_millis(100),
        backoff_max: Duration::from_secs(5),
    }
}

/// Reference worlds' figures, passed to the start instead of document overrides.
pub fn reference_over(jig: &mut jig_protocol_config::Limits, own: &mut Limits) {
    let loaded = 18_431_842_u64
        .checked_add(u64::try_from(size_of::<engine::Record>()).expect("record size fits u64"))
        .expect("reference turn and loaded row fit u64");
    for (name, value) in [
        (b"tasks.tree_tasks".as_slice(), 1024_u64),
        (b"core.resume_bytes".as_slice(), 18_431_846_u64),
        (b"hub.transcript_bytes".as_slice(), 73_727_378_u64),
        (b"hub.turn_bytes".as_slice(), 18_431_842_u64),
        (b"hub.turn_queue_bytes".as_slice(), 147_454_736_u64),
        (b"tasks.inbox_bytes".as_slice(), 5_226_516_u64),
        (b"tasks.message_bytes".as_slice(), 63_738_u64),
        (b"tasks.result_bytes".as_slice(), 63_738_u64),
    ] {
        jig_protocol_config::overrides::apply(jig, name, value).expect("reference jig figure");
    }
    for (name, value) in [
        (b"forge.change_policy.questions".as_slice(), 4_u64),
        (b"checkout.message_bytes".as_slice(), 63_740_u64),
        (b"forge.client.op_bytes".as_slice(), 63_738_u64),
        (b"forge_component.body_bytes".as_slice(), 127_497_u64),
        (b"root.journal.transcript_bytes".as_slice(), 18_431_846_u64),
        (b"root.loads.bytes".as_slice(), loaded),
        (b"root.loads.read_bytes".as_slice(), 36_863_684_u64),
        (b"root.loads.reply_bytes".as_slice(), loaded),
        (b"root.loads.transcript_bytes".as_slice(), 18_431_842_u64),
        (b"forge.client.answer_bytes".as_slice(), 255_570_u64),
        (b"root.journal.run_bytes".as_slice(), 65_536),
    ] {
        temper_engine_service_vocabulary::limits::apply(own, name, value).expect("reference temper figure");
    }
}
