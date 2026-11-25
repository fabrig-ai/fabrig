use crate::{limits, settings};
use skein_lib::Token;

// smith's protocol/agent.md, section 4: OpenAI's public Responses dialect.
pub const DIALECT: u32 = 3;

// The stock catalogue holds twelve charters.
// The closer procedure and close job make seventeen executor places in both limits.
pub const ENGINE: &[u8] = br#"{
"hosts": {
"listener": [{"in_process": "workers"}]
},
"clients": {
},
"owners": [{"provider": 1, "subject": "owner-a"}]
}"#;

#[must_use]
pub fn engine() -> settings::Engine {
    engine_with(engine_jig())
}

#[must_use]
pub fn engine_jig() -> jig_protocol_config::Engine {
    let mut jig = parse_engine(ENGINE);
    let policy = jig.policies.first().expect("engine fixture values fit their declared bounds");
    assert_eq!(policy.number, 1);
    jig.projects = Box::new([jig_protocol_config::Project {
        number: 1,
        name: "Project 1".into(),
        policy: policy.policy.clone(),
        roles: policy.roles.clone(),
        templates: policy.templates.clone(),
    }]);
    for owner in &mut jig.owners {
        owner.project = 1;
    }
    jig
}

#[must_use]
pub fn engine_with(jig: jig_protocol_config::Engine) -> settings::Engine {
    // This typed fixture reaches the core's start; policy 1 includes the train’s merge grants with the standing charter executors.

    let own = small_limits();
    let root = limits::engine(&jig.limits, &own).expect("engine fixture values fit their declared bounds");
    let connector = *jig.connectors.first().expect("engine fixture values fit their declared bounds");
    settings::Engine {
        store_limits: None,
        kinds: crate::stock::kinds(&root, connector),
        period_span: crate::stock::PERIOD_SPAN,
        host_reply_bytes: None,
        agent_charter_bytes: None,
        setups: Box::new([]),
        owned_forgejo: None,
        owned_runner: None,
        test_github: None,
        directories: jig_protocol_config::Directories::working(),
        jig,
        webhooks: None,
        sign_in: None,
        push: None,
        clients: crate::startup::application::Clients {
            listener: jig_protocol_config::Address::Addr(crate::address()),
            bundle: None,
            public: None,
        },
        files: settings::Files { entries: skein_lib::List::with_capacity(0) },
        demo: None,
        program_environment: Box::new([]),
        host_part: temper_engine_domain::room::HostPart {
            allowance: u64::MAX,
            ..temper_engine_domain::room::HostPart::EMPTY
        },
        ways_stock: crate::stock::ways(),
        kinds_stock: temper_engine_domain::landing_stock(&[], &skein_lib::Map::with_capacity(0), 1, 4),
        landing_held: false,
        limits: own,
    }
}

// The small worlds admit the stock's larger goal charter while keeping Small's store page.
fn small_limits() -> limits::Limits {
    let mut own = temper_limits_fixtures::small();
    own.root.journal.run_bytes = 32_768;
    own.root.transcript_memory = 1_152_921_504_606_846_976;
    own
}

#[must_use]
pub fn small_model() -> jig_domain_core::settings::CatalogueModel {
    jig_domain_core::settings::CatalogueModel {
        first_read: None,
        number: 1,
        model: jig_domain_core::Model {
            provider: 1,
            dialect: DIALECT,
            endpoint: 1,
            program: None,
            name: b"small".as_slice().into(),
            cache_life_ms: 0,
            window: 8192,
            output: 1024,
            input_price: 1,
            cached_price: 1,
            cache_write_price: None,
            output_price: 1,
            price_unit: 1000,
            effort: None,
            efforts: Box::default(),
        },
        shown: None,
        description: None,
        picker: true,
        withdrawn: false,
        source: jig_domain_core::settings::ModelSource::Stock,
        estimated: false,
    }
}

/// A parsed engine passed the stock, before any component is made.
#[must_use]
pub fn parse_engine(bytes: &[u8]) -> jig_protocol_config::Engine {
    parse_engine_over(bytes, &engine_figures::over(&jig_protocol_config_fixtures::small()))
}
#[path = "../../engine_limit_figures.rs"]
mod engine_figures;

#[must_use]
pub fn parse_engine_over(bytes: &[u8], base: &jig_protocol_config::Limits) -> jig_protocol_config::Engine {
    let document = jig_protocol_config::document(
        bytes,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .expect("engine fixture values fit their declared bounds");
    let mut jig = jig_protocol_config::engine_over(
        &document,
        0,
        &jig_protocol_config::Directories::working(),
        jig_protocol_config::Defaults::PassedProviders,
        &crate::startup::fixed(&jig_protocol_config::Directories::working())
            .expect("engine fixture values fit their declared bounds"),
        base,
    )
    .expect("engine fixture values fit their declared bounds");
    let running = temper_worker_service_fixtures::providers();
    let mut defaults = crate::stock::checked_with(Box::new([small_model()]), jig.limits.core.tasks.result_bytes)
        .expect("engine fixture values fit their declared bounds");
    defaults.providers.clone_from(&running);
    jig_protocol_config::pass_providers(&mut jig, defaults, &running, Box::new([]))
        .expect("engine fixture values fit their declared bounds");
    jig
}
#[must_use]
pub fn forges() -> Box<[settings::Forge]> {
    Box::new([settings::Forge {
        log_origins: None,
        mode: temper_engine_domain_forge::Mode::Connected,
        number: 1,
        kind: settings::Kind::Forgejo,
        api: jig_protocol_config::Address::Host { name: Box::from(&b"forge.example"[..]), port: 3000 },
        transport: settings::Transport::Plaintext,
        git: Box::from(&b"https://forge.example"[..]),
        git_program: None,
        web: None,
        app_read_only: false,
        user: 42,
        identity: 1,
    }])
}

#[expect(clippy::too_many_lines, reason = "the fixture supplies every retained task field")]
#[must_use]
pub fn task_record(number: u64) -> jig_domain_tasks::TaskRecord {
    use jig_domain_tasks as tasks;
    use skein_lib::Wall;
    tasks::TaskRecord {
        concluded: None,
        session: None,
        talk: None,
        activation_open: false,
        closer: None,
        offered: jig_domain_tasks::Reasons::ALL,
        reasons: jig_domain_tasks::Reasons::FIRST,
        asked_requester: false,
        question_woken: 0,
        words_woken: 0,
        unrecorded: Box::new([]),
        open_attempt: None,
        suspension: None,
        suspended: 0,
        restarted: false,
        settings: tasks::SettingsRevision { task: 1, run: None },
        added: Box::new([]),
        hold_bound_since: None,
        label: 0,
        flags: tasks::Flags::PLAIN,
        last_activity: Wall::EPOCH,
        ended_below: tasks::Endings::ZERO,
        ended_delegates: tasks::Endings::ZERO,
        question_lines: Box::new([]),
        hold: 0,
        placed: None,
        proposal_rows: Box::new([]),
        instructions: None,
        llm: None,
        kept_account: None,
        refused: Box::new([]),
        grace: None,
        created_at: Wall::EPOCH,
        changed: Wall::EPOCH,
        phase_since: Wall::EPOCH,
        held_since: None,
        run_since: None,
        retry_notice: None,
        ended_at: None,
        revision: 0,
        revisions: tasks::Revisions { hold: 0, spec: 0, person: 0 },
        milestone: 0,
        activity: 0,
        due_reason: tasks::Why::Made,
        narrowing: false,
        llm_changed: false,
        result_position: 0,
        last_reply: 0,
        replies: Box::new([]),
        escalation: tasks::Escalation::Unheld { revision: 0 },
        proposal: None,
        result_proposal: false,
        number,
        project: 1,
        requester: tasks::Party::Deployment { project: 1 },
        run_spent: 0,
        run_reserved: 0,
        root: number,
        depth: 0,
        executor: tasks::Executor::Agent { charter: 1 },
        taken_by: None,
        person_answer: None,
        person_wait: None,
        result_followups: Box::new([]),
        tracked: None,
        recurring: None,
        spec: tasks::Spec {
            job: None,
            asks: false,
            words: Box::new([]),
            parameters: Box::new([]),
            inputs: Box::new([]),
        },
        contract: tasks::Contract::Report { words: 32 },
        authority: tasks::Authority {
            tools: tasks::Tools(0),
            models: tasks::Models::NONE,
            grants: Box::new([]),
            delegation: tasks::Delegation { kinds: Box::new([]), tasks: 1, depth: 0 },
            budget: tasks::Budget { spend: 1, deadline: None },
            notes: tasks::Scopes(0),
            note_resources: Box::new([]),
        },
        numbers: tasks::Numbers { budget: 1, spent: 0, spent_below: 0, reserved: 0 },
        funder: tasks::Funder::Period { project: 1, period: 1 },
        waiting_on: Box::new([]),
        holdings: Box::new([]),
        holds_taken: false,
        hold_wait_since: None,
        room_wait: None,
        dependencies: Box::new([]),
        started_from: Box::new([]),
        delegates: Box::new([]),
        references: Box::new([]),
        questions: Box::new([]),
        subscriptions: Box::new([]),
        wake: tasks::WakePolicy::DEFAULT,
        turn: 0,
        last_message: 0,
        inbox: Box::new([]),
        saved: Box::new([]),
        ever_turned: false,
        made: 1,
        attempt: 0,
        last_answer: None,
        tries: tasks::Tries::NONE,
        invalid_result: None,
        refusals: 0,
        phase: tasks::Phase::Waiting,
    }
}

#[must_use]
pub fn wire_limits() -> temper_protocol_worker_wire::v1::Limits {
    temper_engine_service::translate::limits(&temper_limits_fixtures::small())
        .expect("engine fixture values fit their declared bounds")
}

#[must_use]
pub fn repository(repository: u32, owner: &[u8], name: &[u8]) -> temper_engine_domain::ForgeRepository {
    temper_engine_domain::ForgeRepository {
        prefixes: Box::new([]),
        delivers: false,
        save: Some(b"temper/7/s7".as_slice().into()),
        tag: temper_engine_domain_forge::tag(1, repository),
        provider: skein_forge::api::Repository { forge: 1, repository },
        name: name.into(),
        host: b"forge.example".as_slice().into(),
        owner: owner.into(),
        start: temper_engine_domain::ForgeStart::Base(b"main".as_slice().into()),
        push: Some(b"temper/task-7".as_slice().into()),
        landing: None,
        delivered: None,
        pin: None,
        identity: 1,
    }
}

#[must_use]
pub fn assignment_with_settings(
    repository: temper_engine_domain::ForgeRepository,
    settings: &settings::Engine,
) -> temper_engine_domain::Assignment {
    let task = task_record(7);
    temper_engine_domain::Assignment {
        check_timeout: None,
        charter: 1,
        task: 7,
        attempt: 2,
        conversation: 2,
        live: jig_domain_host::LiveAsk::NONE,
        run: Box::new(temper_engine_domain::RunCharter {
            revision: 1,
            ended_by: jig_domain_tasks::EndedBy::Result,
            sources: Box::new([Some(1)]),
            accounts: Box::new([jig_domain_authority::AccountBinding { provider: 1, account: 1, source: 1 }]),
            placement: jig_domain_fleet::PlacementConstraint {
                channel: Token::new(1),
                kind: jig_domain_fleet::HostKind::Worker,
            },
            policy: settings.jig.charters[0].policy.clone(),
            contract: task.contract,
            authority: task.authority,
            budget: 20,
        }),
        sections: Box::new([]),
        messages: Box::new([]),
        saved: Box::new([]),
        workstream: 7,
        workspace: temper_engine_domain::ForgeWorkspace { repositories: Box::new([repository]) },
        transcript: Box::new([]),
        parts: Box::new([]),
        answered: Box::new([]),
        settled: Box::new([]),
        grants: Box::new([]),
    }
}

#[must_use]
pub fn encoding(settings: &settings::Engine) -> crate::assignment::Encoding {
    let mut entries = skein_lib::List::with_capacity(
        u32::try_from(settings.jig.llm.endpoints.len()).expect("engine fixture values fit their declared bounds"),
    );
    for endpoint in &settings.jig.llm.endpoints {
        entries
            .push(smith_protocol_channel::Endpoint {
                name: endpoint.name.clone(),
                number: endpoint.number,
                dialect: endpoint.dialect,
            })
            .expect("engine fixture values fit their declared bounds");
    }
    crate::assignment::Encoding {
        limits: wire_limits(),
        endpoints: smith_protocol_channel::Endpoints::new(entries),
        deployment: [7; 16],
        reply_bytes: settings.jig.limits.core.call_answer_bytes,
        forge_reads: temper_engine_smith::ForgeReadBounds {
            rows: settings.limits.forge.client.rows,
            answer_bytes: settings.limits.forge.client.answer_bytes,
        },
        charter_bytes: crate::assignment::charter_bound(
            settings.jig.limits.hub.charter_bytes,
            settings.agent_charter_bytes,
        ),
    }
}

#[must_use]
pub fn worker_runtime() -> crate::WorkerRuntime {
    crate::WorkerRuntime {
        root: skein_io::kernel::Fd::new(9),
        path: b"/worker".as_slice().into(),
        agent: super::pinned_agent(super::worker_settings().jig.agent.as_ref().expect("checked worker agent")),
    }
}
