//! Child limits of the root's resume fixture, kept at the same tiny figures.
use jig_domain_accounts as accounts;
use jig_domain_authority as authority;
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::Duration;
use temper_engine_domain_forge as forge;

#[must_use]
pub fn documents(authority: &authority::Limits, tasks: &tasks::Limits) -> jig_domain_core::documents::DocumentLimits {
    jig_domain_core::documents::DocumentLimits {
        roles: 1,
        segments: 9,
        grants: authority.grants.max(tasks.authority_grants),
        executors: authority.executors.max(tasks.executor_kinds),
        implications: authority.implications,
        rows: 4,
        text_bytes: 4_096,
        chips: 4,
        lines: 4,
        versions: jig_domain_core::documents::Versions { minimum: 1, maximum: 1 },
    }
}

#[must_use]
pub fn tasks() -> tasks::Limits {
    let retry = tasks::Retry { retries: 1, base: Duration::from_millis(10), max: Duration::from_secs(1) };
    tasks::Limits {
        charter_parts: 8,
        tasks: 2,
        funders: 4,
        project_tasks: 2,
        project_goals: 2,
        grace: Duration::from_secs(5),
        tree_tasks: 2,
        depth: 1,
        delegates: 1,
        references: 2,
        subscriptions: 2,
        batch: 1,
        batch_bytes: 131_072,
        dependencies: 1,
        circle: 2,
        holdings: 2,
        hold_kinds: 2,
        placed_holds: 2,
        project_placed_holds: 2,
        party_placed_holds: 2,
        exemptions: 2,
        resource_reports: 64,
        pools: 2,
        hold_segments: 4,
        hold_bytes: 128,
        hold_waiters: 2,
        hold_wait: Duration::from_secs(10),
        inputs: 1,
        spec_bytes: 64,
        parameters: 1,
        result_bytes: 128,
        run_inbox: 4,
        inbox_messages: 4,
        inbox_bytes: 128,
        message_bytes: 64,
        person_stall: Duration::from_secs(3600),
        proposal_stall: Duration::from_millis(10),
        escalation_stall: Duration::from_secs(3600),
        saved_resources: 2,
        contract_choices: 1,
        charters: 1,
        charter_documents: 2,
        authority_grants: 1,
        authority_segments: 1,
        authority_bytes: 32,
        executor_kinds: 1,
        retries: tasks::Retries {
            transient: retry,
            permanent: retry,
            run: retry,
            agent: retry,
            lost: retry,
            invalid: retry,
            infrastructure: retry,
        },
        facts: 2,
        room_refusals: 2,
    }
}

#[must_use]
pub fn people() -> people::Limits {
    people::Limits {
        batch: 4,
        act_grants: 0,
        operations: 2,
        people: 2,
        program_hosts: 2,
        program_host_name_bytes: 32,
        inbox_entries: 2,
        inbox_tasks: 4,
        inbox_recipients: 16,
        sign_ins: 2,
        projects: 2, // The configured project and the deployment roster.
        holdings: 2,
        initial_owners: 1,
        requests: 4,
        pending: 2,
        waiters: 2,
        identity_bytes: 32,
        words: 64,
        spec_bytes: 64,
        amendment_bytes: 512,
        sign_in_lifetime: Duration::from_secs(60),
        request_retention: Duration::from_secs(120),
        facts: 2,
        room_refusals: 2,
    }
}

#[must_use]
pub fn fleet() -> fleet::Limits {
    fleet::Limits {
        reads: 1,
        configured_workers: 1,
        worker_name_bytes: 32,
        program_sources: 12,
        source_detail_bytes: 4096,
        workers: 1,
        needs_bytes: 65_536,
        engine_slots: 0,
        slots: 1,
        workstreams: 1,
        attempts: 2,
        grants: 2,
        calls: 1,
        call_name_bytes: 64,
        progress_bytes: 0,
        turns: 2,
        grace: Duration::from_secs(5),
        restart_wait: Duration::from_millis(1_200_000),
        facts: 2,
        history: 2,
    }
}

#[must_use]
pub fn accounts() -> accounts::Limits {
    accounts::Limits {
        accounts: 1,
        move_deadline: Duration::from_secs(60),
        holder_accounts: 1,
        sources: 3,
        sign_ins: 1,
        credentials: 1,
        program_names: 3,
        program_hosts: 2,
        backoff_base: Duration::from_millis(10),
        backoff_max: Duration::from_secs(1),
        retry: Duration::from_millis(10),
        spent_attention: Duration::from_secs(1),
        facts: 2,
    }
}

#[must_use]
pub fn authority_limits() -> authority::Limits {
    authority::Limits {
        projects: 1,
        roles: 1,
        requirements: 2,
        facts: 2,
        grants: 1,
        executors: 1,
        segments: 1,
        segment_bytes: 32,
        implications: 0,
        batch: 1,
        accounts: 2,
        writes: 1,
    }
}

#[must_use]
pub fn settings() -> jig_domain_core::settings::Limits {
    jig_domain_core::settings::Limits {
        revisions: 4,
        versions: 8,
        history: 64,
        alternatives: 16,
        permissions: 32,
        proposals: 4,
    }
}

#[must_use]
pub fn notes() -> jig_domain_notes::Limits {
    jig_domain_notes::Limits {
        scopes: 4,
        entries_per_scope: 1,
        party_entries: 1,
        party_bytes: 64,
        pattern_bytes: 64,
        description_bytes: 64,
        body_bytes: 256,
        references: 1,
        load_rows: 1,
        lines: 1,
        recalled: 1,
    }
}

const CLIENT: skein_forge_client::Limits = skein_forge_client::Limits {
    pending: 4,
    calls: 2,
    rate: 100,
    reserve: 0,
    window: Duration::from_secs(10),
    op_bytes: 512,
    answer_bytes: 4096,
    rows: 8,
    inbox: 16,
    read_attempts: 3,
    backoff: Duration::from_secs(1),
    backoff_max: Duration::from_secs(4),
    entries: 8,
    write_attempts: 3,
    lifetime: Duration::from_secs(10),
    clock_margin: Duration::from_secs(1),
    resources: 8,
    repositories: 2,
    poll: Duration::from_secs(5),
    poll_max: Duration::from_secs(30),
    hinted: Duration::from_secs(1),
    slow: Duration::from_secs(40),
    settle: Duration::from_secs(30),
    fallback: Duration::ZERO,
    facts: 16,
};
pub const FORGE_LIMITS: forge::Limits = forge::Limits {
    forges: 4,
    repositories: 2,
    tasks: 8,
    holds: 8,
    branch_reds: 64,
    red_key_bytes: 256,
    subscriptions: 8,
    entries: 8,
    named_calls: 16,
    changes: 8,
    issues: 8,
    resources_per_task: 4,
    paths_per_subscription: 4,
    authority_segments: 8,
    name_bytes: 128,
    output: 80,
    facts: 16,
    adoptions: 2,
    makings: 2,
    applications: 4,
    bindings: 4,
    collaborators: 8,
    landings: 8,
    landing_members: 2,
    landings_kept: 16,
    landings_pending: 64,
    changes_rows: 32,
    changes_landed: 8,
    gates: 2,
    receipts_per_gate: 4,
    ignored_paths: 4,
    key_entries: 16,
    key_path_bytes: 1024,
    trees_in_flight: 2,
    tiers: temper_engine_domain_forge::TierLimits { tiers: 2, failing_tests: 16, test_id_bytes: 256 },
    judge_projects: 2,
    decided_per_project: 64,
    history_days: 14,
    history_events: 131_072,
    findings_per_project: 256,
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
    brief_sections: 32,
    brief_bytes: 4096,
    client: CLIENT,
    issue_policy: temper_engine_domain_forge_issues::Limits {
        plan_items: 8,
        milestones: 8,
        title_bytes: 128,
        body_bytes: 512,
        comment_bytes: 256,
        interval: Duration::from_secs(5),
    },
    waits: forge::STANDALONE_WAIT_FIGURES,
    change_policy: temper_engine_domain_forge_change::Limits {
        reruns: 1,
        repairs: 3,
        resolutions: 3,
        updates: 3,
        questions: 4,
        tasks: 6,
        stall: Duration::from_secs(30),
        unknown_grace: Duration::from_secs(30),
        runner_wait: Duration::from_secs(240),
        gates: 8,
        clean_heads: 8,
    },
    queue_window: Duration::from_secs(30),
    sitting: temper_engine_domain_forge::SITTING,
};
