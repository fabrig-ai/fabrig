use temper_worker_domain::wire;

pub fn one_repository() -> wire::Workspace {
    wire::Workspace {
        repositories: Box::new([wire::Repository {
            admit: false,
            save: None,
            git: None,
            tag: (1_u64 << 32) | 3,
            name: b"app".as_slice().into(),
            remote: b"https://forge.example/acme/app.git".as_slice().into(),
            start: wire::Start::Base { branch: b"main".as_slice().into() },
            access: wire::Access::Writable {
                push: b"temper/task-7".as_slice().into(),
                expected: None,
                landing: None,
                delivered: None,
                pin: None,
            },
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
        parts: Box::new([]),
        tell_parts: false,
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

pub const NOW: skein_lib::Time = skein_lib::Time::ZERO;

pub fn worker() -> crate::Worker {
    crate::Worker::new(
        &worker_settings(),
        &worker_agent(),
        &worker_limits(),
        &worker_files(),
        crate::WorkerRuntime {
            root: skein_io::kernel::Fd::new(9),
            path: b"/worker".as_slice().into(),
            agent: pinned_agent(worker_settings().jig.agent.as_ref().expect("checked worker agent")),
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
    let record = temper_protocol_worker_wire::v1::Identity {
        kind: temper_protocol_worker_wire::v1::Kind::Forgejo(temper_protocol_worker_wire::v1::Token {
            token: value.token,
        }),
        login: value.login,
        name: value.name,
        email: value.email,
    };
    jig_protocol_worker_link::Request::Value {
        account: number,
        generation,
        expires: skein_lib::Time::from_nanos(u64::MAX),
        value: jig_protocol_worker_link::GrantValue::Application {
            secret: temper_protocol_worker_wire::identity(&record),
        },
    }
}

pub(crate) use crate::testing::fixtures::WORKER_SECRET;
pub use crate::testing::fixtures::{
    AGENT_JSON, PINNED_AGENT_JSON, PINNED_TEMPER, pinned_agent, worker_files, worker_limits,
};

pub fn worker_settings() -> crate::settings::Worker {
    crate::testing::fixtures::worker_settings(jig_protocol_config_fixtures::small())
}
pub fn worker_agent() -> crate::settings::Agent {
    crate::testing::fixtures::worker_agent(smith_agent_world::LIMITS.run)
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
            .push(v1::Repository {
                admit: false,
                save: Some(b"temper/saved-7".as_slice().into()),
                git: None,
                tag: (1_u64 << 32) | 3,
                name: b"app".as_slice().into(),
                remote: b"https://forge.example/acme/app.git".as_slice().into(),
                start: v1::Start::Base(v1::Branch { branch: b"main".as_slice().into() }),
                access: v1::Access::Writable(v1::Writable {
                    push: b"temper/task-7".as_slice().into(),
                    expected: None,
                    landing: None,
                    delivered: None,
                    pin: None,
                }),
                identity: 1,
                account: None,
            })
            .unwrap();
    }
    let parts = if repository {
        Box::from([jig_protocol_worker_wire::Part {
            token: Token::new(1),
            bytes: payload::items(&v1::Items { repositories }),
        }])
    } else {
        Box::default()
    };
    jig_protocol_worker_link::Request::Down {
        down: host::LinkDown::Assign {
            assignment: host::Assignment {
                parts: Box::new([]),
                tell_parts: false,
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
