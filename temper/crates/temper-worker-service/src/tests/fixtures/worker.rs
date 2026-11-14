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
pub const PINNED_TEMPER: &[u8] = b"/worker/agent/temper";
pub const PINNED_AGENT_JSON: &[u8] = b"/worker/agent/agent.json";
#[expect(clippy::disallowed_methods, reason = "ordinary fixture code copies the agent's argument list")]
pub fn pinned_agent(agent: &jig_protocol_config::Agent) -> jig_protocol_config::Agent {
    jig_protocol_config::Agent {
        program: PINNED_TEMPER.into(),
        configuration: PINNED_AGENT_JSON.into(),
        arguments: agent
            .arguments
            .iter()
            .map(|argument| if *argument == agent.configuration { PINNED_AGENT_JSON.into() } else { argument.clone() })
            .collect(),
    }
}
pub const WORKER_SECRET: &[u8] = b"worker-secret-value";
pub const NOW: skein_lib::Time = skein_lib::Time::ZERO;

pub fn worker() -> crate::Worker {
    crate::Worker::new(
        &worker_settings(),
        &worker_agent(),
        &worker_limits(),
        &worker_files(),
        crate::WorkerRuntime {
            wires: None,
            root: skein_io::kernel::Fd::new(9),
            path: b"/worker".as_slice().into(),
            agent: pinned_agent(&worker_settings().jig.agent),
        },
        7,
    )
    .expect("worker settings fit")
}
pub fn spawn(worker: &mut crate::Worker) -> (skein_lib::Token, skein_io::kernel::Spawn) {
    worker.iterate(NOW, skein_lib::Wall::EPOCH);
    for _ in 0..worker.io().len() {
        if let Some(skein_io::Request::Spawn { owner, spawn }) = worker.io().pop() {
            return (owner, spawn);
        }
    }
    panic!("decoded start asks for a process");
}

pub fn chat_assignment() -> jig_protocol_worker_link::Request {
    assigned(false)
}
pub fn repository_assignment() -> jig_protocol_worker_link::Request {
    assigned(true)
}
pub fn application_value(number: u32, generation: u64, token: &[u8]) -> jig_protocol_worker_link::Request {
    let value = identity(token);
    let record = temper_protocol_worker_wire::v1::Identity::new(
        &super::wire_limits(),
        temper_protocol_worker_wire::v1::IdentityParts {
            kind: temper_protocol_worker_wire::v1::Kind::Forgejo(
                temper_protocol_worker_wire::v1::Token::new(
                    &super::wire_limits(),
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

pub fn worker_settings() -> crate::settings::Worker {
    use jig_protocol_config as config;
    let mut limits = jig_protocol_config::profile(jig_protocol_config::Profile::Small);
    limits.core.call_answer_bytes = 2048;
    limits.hub = jig_domain_host::Limits {
        application_ask_bytes: 0,
        application_answer_bytes: 0,
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
    crate::settings::Worker {
        jig: config::Worker {
            check_interval: limits.hub.source_interval,
            engine: config::Address::Addr(skein_io::Addr::InProcess(skein_io::Name::new(b"workers").unwrap())),
            name: b"worker-a".as_slice().into(),
            secret: b"worker-secret".as_slice().into(),
            slots: 1,
            graces: config::Graces {
                grace: skein_lib::Duration::from_secs(10),
                drain: skein_lib::Duration::from_secs(10),
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
pub fn worker_limits() -> crate::worker_settings::Limits {
    let mut own = crate::worker_settings::Limits::shared(temper_service_core::limits::profile(
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
pub fn worker_files() -> crate::settings::Files {
    let mut entries = skein_lib::List::with_capacity(1);
    entries.push((b"worker-secret".as_slice().into(), skein_lib::Secret::new(WORKER_SECRET.into()))).unwrap();
    crate::settings::Files { entries }
}
pub fn worker_agent() -> crate::settings::Agent {
    let mut agent = super::agent();
    agent.bounds.message_bytes = 512;
    agent
}

fn assigned(repository: bool) -> jig_protocol_worker_link::Request {
    use jig_domain_host as host;
    use skein_lib::{List, Token};
    use temper_protocol_worker_wire::{self as payload, v1};
    let agent = worker_agent();
    let charter =
        smith_protocol_channel::encode_charter(&super::charter(), &agent.limits.charter, &agent.endpoints).unwrap();
    let mut repositories = List::with_capacity(1);
    if repository {
        repositories
            .push(
                v1::Repository::new(
                    &super::wire_limits(),
                    v1::RepositoryParts {
                        save: Some(b"temper/saved-7".as_slice().into()),
                        git: None,
                        tag: (1_u64 << 32) | 3,
                        name: b"app".as_slice().into(),
                        remote: b"https://forge.example/acme/app.git".as_slice().into(),
                        start: v1::Start::Base(
                            v1::Branch::new(
                                &super::wire_limits(),
                                v1::BranchParts { branch: b"main".as_slice().into() },
                            )
                            .unwrap(),
                        ),
                        access: v1::Access::Writable(
                            v1::Writable::new(
                                &super::wire_limits(),
                                v1::WritableParts {
                                    push: b"temper/task-7".as_slice().into(),
                                    expected: None,
                                    landing: None,
                                },
                            )
                            .unwrap(),
                        ),
                        identity: 1,
                        account: None,
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    let parts = if repository {
        Box::from([jig_protocol_worker_wire::Part {
            token: Token::new(1),
            bytes: payload::items(&v1::Items::new(&super::wire_limits(), v1::ItemsParts { repositories }).unwrap()),
        }])
    } else {
        Box::default()
    };
    jig_protocol_worker_link::Request::Down {
        down: host::LinkDown::Assign {
            assignment: host::Assignment {
                assignment: host::RunAssignment {
                    live: host::LiveAsk::NONE,
                    identity: [7; 16],
                    run: Token::new(7),
                    attempt: Token::new(2),
                    workspace: if repository {
                        Some(host::Workspace { workstream: 7, items: Token::new(1) })
                    } else {
                        None
                    },
                    save: repository,
                    charter,
                    grants: Box::default(),
                },
                turns: Box::default(),
                answered: Box::default(),
                messages: Box::default(),
            },
        },
        parts,
    }
}
