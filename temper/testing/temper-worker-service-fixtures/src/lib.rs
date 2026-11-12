//! Public worker fixtures for composition tests (temper/docs/design/protocol/services.md, 8).
use temper_worker_service::settings;
use jig_domain_link::{Bounds, EndpointBound};
pub fn agent() -> settings::Agent {
    use skein_lib::Duration;
    let mut endpoints = skein_lib::List::with_capacity(1);
    endpoints
        .push(smith_protocol_channel::Endpoint { name: b"fake".as_slice().into(), number: 1, dialect: 3 })
        .unwrap();
    let host_limits = smith_host_domain::Limits {
        agents: 1,
        directories: 1,
        conflicts: 4,
        path_bytes: 255,
        name_bytes: 64,
        accounts: 2,
        charter_bytes: 65_536,
        transcript_bytes: 262_144,
        answered_bytes: 65_536,
        message_bytes: 65_536,
        messages: 2,
        calls: 1,
        call_bytes: 65_536,
        answer_bytes: smith_host_domain::Delivered::owned_bound(),
        turns: 1,
        turn_bytes: 65_536,
        unacknowledged_bytes: 65_536,
        fact_bytes: 65_536,
        outcome_bytes: 4096,
        detail_bytes: 64,
        spawn_timeout: Duration::from_secs(1),
        no_progress: Duration::from_secs(120),
        silence: Duration::from_secs(60),
        long_span: Duration::from_secs(60),
        long_open: 1,
        wall_time: Duration::from_secs(300),
        cancel_grace: Duration::from_secs(5),
        exit_grace: Duration::from_secs(5),
        teardown: Duration::from_secs(1),
        kill_after: Duration::from_secs(2),
        facts: 1,
    };
    let mut limits = smith_host_protocol::Limits {
        bodies: smith_channel::Limits {
            turn_body: 65_536,
            accepted_result: 4096,
            host_ask_input: 4096,
            message_text: 65_536,
            host_reply_text: 65_536,
            start_charter: 65_536,
            start_transcript: 262_144,
            start_answered: 4,
            start_grants: 2,
            start_messages: 2,
            ..smith_channel::CEILINGS
        },
        charter: smith_charter::CEILINGS,
        transcript: smith_transcript::CEILINGS,
        endpoints: 1,
        channel: skein_channel::Limits {
            chunk: 64,
            credential: 1,
            skip: 1_048_576,
            output_bytes: 1_048_576,
            output_frames: 8,
            kinds: 32,
        },
        start_bytes: 1_048_576,
        calls: 1,
    };
    limits.channel.output_bytes = smith_host_protocol::output_bytes(&limits).expect("host frames");
    settings::Agent {
        run_limits: smith_agent_world::LIMITS.run,
        tool_payload: 2048,
        programs: Box::new([]),
        endpoints: smith_protocol_channel::Endpoints::new(endpoints),
        limits,
        host_limits,
        bounds: Bounds {
            program_endpoints: Box::new([]),
            endpoints: Box::new([EndpointBound { endpoint: 1, window: 8192, output: 1024 }]),
            turns: 800,
            spend: 256,
            time: Duration::from_secs(28_800),
            waiting: Duration::from_secs(60),
            checks: Duration::from_secs(60),
            message_bytes: 65_536,
            inbox: 2,
            unacknowledged: 1,
            reply_bytes: None,
        },
    }
}
use temper_worker_domain::wire;

pub fn one_repository() -> wire::Workspace {
    wire::Workspace {
        repositories: Box::new([wire::Repository {
            save: None,
            git: None,
            tag: (1_u64 << 32) | 3,
            name: b"app".as_slice().into(),
            remote: b"https://forge.example/acme/app.git".as_slice().into(),
            start: wire::Start::Base { branch: b"main".as_slice().into() },
            access: wire::Access::Writable { push: b"temper/task-7".as_slice().into(), expected: None, landing: None },
            identity: 1,
            account: None,
        }]),
    }
}
pub fn identity(token: &[u8]) -> temper_worker_protocol_git::Identity {
    temper_worker_protocol_git::Identity {
        token: skein_lib::Secret::new(token.into()),
        login: b"temper-bot".as_slice().into(),
        name: b"Temper Bot".as_slice().into(),
        email: b"bot@example.test".as_slice().into(),
    }
}
pub fn assignment() -> jig_domain_host::Assignment {
    use jig_domain_host as host;
    use skein_lib::Token;
    host::Assignment {
        assignment: host::RunAssignment {
            live: host::LiveAsk::NONE,
            identity: [7; 16],
            run: Token::new(1),
            attempt: Token::new(2),
            workspace: Some(host::Workspace { workstream: 1, items: Token::new(9) }),
            save: false,
            charter: Box::new([]),
            grants: Box::new([]),
        },
        turns: Box::new([]),
        answered: Box::new([]),
        messages: Box::new([]),
    }
}

pub const TEMPER: &[u8] = b"/opt/temper/bin/temper";
pub const AGENT_JSON: &[u8] = b"/opt/temper/agent.json";
pub const WORKER_SECRET: &[u8] = b"worker-secret-value";
pub const NOW: skein_lib::Time = skein_lib::Time::ZERO;

pub fn worker() -> temper_worker_service::Worker {
    temper_worker_service::Worker::new(
        &worker_settings(),
        &worker_agent(),
        &worker_limits(),
        &worker_files(),
        temper_worker_service::WorkerRuntime { root: skein_io::kernel::Fd::new(9), path: b"/worker".as_slice().into() },
        7,
    )
    .expect("worker settings fit")
}




pub fn application_value(number: u32, generation: u64, token: &[u8]) -> jig_protocol_worker_link::Request {
    let value = identity(token);
    let record = temper_protocol_worker_wire::v1::Identity::new(
        &wire_limits(),
        temper_protocol_worker_wire::v1::IdentityParts {
            kind: temper_protocol_worker_wire::v1::Kind::Forgejo(
                temper_protocol_worker_wire::v1::Token::new(
                    &wire_limits(),
                    temper_protocol_worker_wire::v1::TokenParts { token: value.token },
                )
                .expect("token record"),
            ),
            login: value.login,
            name: value.name,
            email: value.email,
        },
    )
    .expect("identity record");
    jig_protocol_worker_link::Request::Value {
        account: number,
        generation,
        expires: skein_lib::Time::from_nanos(u64::MAX),
        value: jig_protocol_worker_link::GrantValue::Application {
            secret: temper_protocol_worker_wire::identity(&record),
        },
    }
}

pub fn worker_settings() -> temper_worker_service::settings::Worker {
    use jig_protocol_config as config;
    let mut limits = jig_protocol_config::profile(jig_protocol_config::Profile::Small);
    limits.core.call_answer_bytes = 2048;
    limits.hub = jig_domain_host::Limits {
        program_sources: 12,
        source_interval: skein_lib::Duration::from_secs(60),
        source_timeout: skein_lib::Duration::from_secs(10),
        slots: 1,
        accounts: 2,
        charter_bytes: 65_536,
        transcript_bytes: 262_144,
        delivery_evidence_bytes: 65_536,
        turn_bytes: 65_536,
        outcome_bytes: 4096,
        detail_bytes: 64,
        held: 2,
        inbox: 2,
        piece_bytes: 4096,
        event_bytes: 512,
        run_calls: 1,
        facts: 4,
        told: 4,
        turns: 1,
        turn_queue_bytes: 65_536,
    };
    temper_worker_service::settings::Worker {
        jig: config::Worker {
            check_interval: limits.hub.source_interval,
            engine: config::Address::Addr(skein_io::Addr::InProcess(skein_io::Name::new(b"workers").unwrap())),
            name: b"worker-a".as_slice().into(),
            secret: b"worker-secret".as_slice().into(),
            slots: 1,
            graces: config::Graces {
                grace: skein_lib::Duration::from_secs(10),
                agent_stop: skein_lib::Duration::from_secs(10),
                delivery: skein_lib::Duration::from_secs(10),
                save: skein_lib::Duration::from_secs(10),
            },
            stop_bound: skein_lib::Duration::from_secs(60),
            agent: config::Agent {
                program: TEMPER.into(),
                arguments: Box::new([b"internal".as_slice().into(), b"agent".as_slice().into(), AGENT_JSON.into()]),
                configuration: AGENT_JSON.into(),
            },
            limits,
            memory: None,
        },
        root: b"worker-root".as_slice().into(),
        git: Some(b"/usr/bin/git".as_slice().into()),
    }
}
pub fn worker_limits() -> temper_worker_service::worker_settings::Limits {
    let mut own = temper_worker_service::worker_settings::Limits::shared(temper_service_core::limits::profile(
        temper_service_core::limits::Profile::Small,
    ));
    own.checkout.workspaces = 1;
    own.checkout.repositories = 1;
    own.checkout.name_bytes = 64;
    own.checkout.message_bytes = 65_536;
    own.checkout.conflicts = 0;
    own.checkout.path_bytes = 64;
    own
}
pub fn worker_files() -> temper_worker_service::settings::Files {
    let mut entries = skein_lib::List::with_capacity(1);
    entries.push((b"worker-secret".as_slice().into(), skein_lib::Secret::new(WORKER_SECRET.into()))).unwrap();
    temper_worker_service::settings::Files { entries }
}
pub fn worker_agent() -> temper_worker_service::settings::Agent {
    let mut agent = agent();
    agent.bounds.message_bytes = 512;
    agent
}



pub fn wire_limits() -> temper_protocol_worker_wire::v1::Limits { temper_worker_service::worker_settings::payload_limits(&worker_limits()).unwrap() }
