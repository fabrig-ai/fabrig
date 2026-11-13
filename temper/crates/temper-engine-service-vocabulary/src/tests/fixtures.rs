use crate::{limits, settings};
use skein_lib::Token;
pub const DIALECT: u32 = 3;

pub const ENGINE: &[u8] = br#"{
"limits": {"profile": "small", "overrides": {"core.run_bytes":32768,"tasks.charters":13,"core.templates":6,"tasks.result_bytes":32,"tasks.run_inbox":2,"hub.turns":1,"people.amendment_bytes":64,"core.policy_bytes":8192,"authority.grants":16,"authority.executors":21,"tasks.executor_kinds":21,"notes.scopes":32,"tasks.spec_bytes":128,"tasks.message_bytes":512,"tasks.hold_bytes":64,"tasks.authority_bytes":64,"people.words":512,"core.call_answer_bytes":2048,"notes.recalled":1}},
"llm": {
"providers":[{"name":"fake-provider","number":1,"endpoint":"fake","sources":{"keys":true}}],
"endpoints": [{
"name": "fake",
"number": 1,
"dialect": 3,
"provider": "fake-provider",
"address": {"socket": "127.0.0.1:7101"},
"transport": "plaintext",
"headers": [],
"connect_ms": 10000,
"handshake_ms": 10000
}]
},
"hosts": {
"workers": [
{"name": "worker-a", "digest": "secrets/worker-a.digest"},
{"name": "worker-b", "digest": "secrets/worker-b.digest"}
],
"listener": [{"in_process": "workers"}]
},
"clients": {
"heartbeat_ms": 10000,
"link": true
},
"deployment": {"directory": "state", "seed": 7},
"owners": [{"provider": 1, "subject": "owner-a"}],
"providers": [{"number": 1, "kind": "link"}, {"number": 2, "kind": "deployment"}],
"connectors": [1],
"memory_bytes": 1152921504606846976,
"projects": [{"number": 1, "policy": 1}]
}"#;

pub const OWNER: Token = Token::new(91);

pub fn engine() -> settings::Engine {
    static ENGINE_SETTINGS: std::sync::OnceLock<jig_protocol_config::Engine> = std::sync::OnceLock::new();
    let jig = ENGINE_SETTINGS.get_or_init(|| parse_engine(ENGINE)).clone();
    // This typed fixture reaches the core's start; policy 1 needs 6074 bytes with the standing charter executors.

    let own = small_limits();
    let root = limits::engine(&jig.limits, &own).unwrap();
    let connector = *jig.connectors.first().unwrap();
    settings::Engine {
        kinds: crate::stock::kinds(&root, connector),
        period_span: crate::stock::PERIOD_SPAN,
        host_reply_bytes: None,
        agent_charter_bytes: None,
        machine: machine(),
        directories: jig_protocol_config::Directories::working(),
        jig,
        webhooks: None,
        sign_in: None,
        push: None,
        clients: crate::startup::application::Clients {
            listener: jig_protocol_config::Address::Addr(http::address()),
            bundle: None,
            public: None,
        },
        files: settings::Files { entries: skein_lib::List::with_capacity(0) },
        demo: None,
        welcome: false,
        program_environment: Box::new([]),
        host_part: None,
        ways_stock: crate::stock::ways(),
        kinds_stock: temper_engine_domain::landing_stock(&[], &skein_lib::Map::with_capacity(0), 1, 4),
        landings: skein_lib::Map::with_capacity(0),
        limits: own,
    }
}

fn small_limits() -> limits::Limits {
    let mut own = limits::profile(limits::Profile::Small);
    own.root.journal.run_bytes = 32_768;
    own
}

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

pub fn parse_engine(bytes: &[u8]) -> jig_protocol_config::Engine {
    let document = jig_protocol_config::document(
        bytes,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .unwrap();
    let mut jig = jig_protocol_config::engine(
        &document,
        0,
        &jig_protocol_config::Directories::working(),
        jig_protocol_config::Defaults::Passed,
    )
    .unwrap();
    let own = match jig.llm.endpoints.first() {
        Some(endpoint) => endpoint.name.as_ref() == b"fake",
        None => false,
    };
    let defaults = if own {
        crate::stock::checked_with(Box::new([small_model()]), jig.limits.core.tasks.result_bytes).unwrap()
    } else {
        crate::stock::defaults(&jig.llm, jig.limits.core.tasks.result_bytes)
    };
    jig_protocol_config::pass(&mut jig, defaults).unwrap();
    jig
}

pub fn machine() -> temper_forge_setup::machine::Machine {
    temper_forge_setup::machine::Machine { owned: None, fake: None, github: None }
}

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

pub fn forge_table(forges: &[settings::Forge]) -> crate::forges::Forges {
    let mut table = crate::forges::Forges::with_capacity(u32::try_from(forges.len()).unwrap());
    for forge in forges {
        assert!(table.insert(settings::Forge {
            log_origins: forge.log_origins.clone(),
            mode: forge.mode,
            number: forge.number,
            kind: forge.kind,
            api: forge.api.clone(),
            transport: match &forge.transport {
                settings::Transport::Plaintext => settings::Transport::Plaintext,
                settings::Transport::Tls { server_name, trust } =>
                    settings::Transport::Tls { server_name: server_name.clone(), trust: trust.clone() },
            },
            git: forge.git.clone(),
            git_program: forge.git_program.clone(),
            web: forge.web.clone(),
            app_read_only: forge.app_read_only,
            user: forge.user,
            identity: forge.identity,
        }));
    }
    table
}

pub fn routes() -> crate::store::route::Routes {
    let (root, _, _) = store_limits();
    let bounds = temper_protocol_store::bounds(&root.forge, root.tasks.spec_bytes).expect("family bounds");
    crate::store::route::Routes::new(2, &root, bounds)
}

#[expect(clippy::too_many_lines, reason = "the fixture supplies every retained task field")]
pub fn task_record(number: u64) -> jig_domain_tasks::TaskRecord {
    use jig_domain_tasks as tasks;
    use skein_lib::Wall;
    tasks::TaskRecord {
        session: None,
        talk: None,
        activation_open: false,
        closer: None,
        offered: jig_domain_tasks::Reasons::ALL,
        reasons: jig_domain_tasks::Reasons::FIRST,
        asked_requester: false,
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

pub fn store_limits() -> (temper_engine_domain::Limits, jig_domain_core::Limits, jig_protocol_store::Limits) {
    let settings = engine();
    store_limits_for(&settings)
}

fn store_limits_for(
    settings: &settings::Engine,
) -> (temper_engine_domain::Limits, jig_domain_core::Limits, jig_protocol_store::Limits) {
    let root = limits::engine(&settings.jig.limits, &settings.limits).expect("root limits");
    (root, settings.jig.limits.core, settings.jig.limits.store)
}

pub fn wire_limits() -> temper_protocol_worker_wire::v1::Limits {
    crate::translate::limits(&limits::profile(limits::Profile::Small)).unwrap()
}

pub fn repository(repository: u32, owner: &[u8], name: &[u8]) -> temper_engine_domain::ForgeRepository {
    temper_engine_domain::ForgeRepository {
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
        identity: 1,
    }
}

pub fn assignment_with(repository: temper_engine_domain::ForgeRepository) -> temper_engine_domain::Assignment {
    let settings = engine();
    assignment_with_settings(repository, &settings)
}

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
        answered: Box::new([]),
        settled: Box::new([]),
        grants: Box::new([]),
    }
}

pub fn encoding(settings: &settings::Engine) -> crate::assignment::Encoding {
    let mut entries = skein_lib::List::with_capacity(u32::try_from(settings.jig.llm.endpoints.len()).unwrap());
    for endpoint in &settings.jig.llm.endpoints {
        entries
            .push(smith_protocol_channel::Endpoint {
                name: endpoint.name.clone(),
                number: endpoint.number,
                dialect: endpoint.dialect,
            })
            .unwrap();
    }
    crate::assignment::Encoding {
        limits: wire_limits(),
        endpoints: smith_protocol_channel::Endpoints::new(entries),
        deployment: [7; 16],
        charter_bytes: crate::assignment::charter_bound(
            settings.jig.limits.hub.charter_bytes,
            settings.agent_charter_bytes,
        ),
    }
}

pub fn withdrawn_history(task: u64, proposal: u64) -> jig_domain_core::Record {
    use jig_domain_tasks as tasks;
    jig_domain_core::Record::Tasks(tasks::Stored::History(tasks::History {
        task,
        revision: 3,
        by: tasks::Party::Task(task),
        change: tasks::Change::ProposalWithdrawn,
        reason: Box::new([]),
        at: skein_lib::Wall::EPOCH,
        amendment: None,
        proposal: Some(Box::new(tasks::Proposal {
            number: proposal,
            proposer: task,
            project: 1,
            action: tasks::ProposalAction::Release { task },
            reason: Box::new([]),
            as_holder: false,
            state: tasks::ProposalState::Withdrawn,
        })),
    }))
}

mod http;
