//! A real engine, worker and agent list child over simulated pipes and a fake provider (protocol/channel/README.md, 7 and 11).
use crate::{
    fixtures,
    processes::{self, Proc},
    program_sources::Files,
    referee::{Roots, Rules},
    stories::Story,
    world::{self, Store},
};
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_link::{Form, Grant, Scheme};
use jig_protocol_worker_wire as wire;
use skein_fake_machine::Item;
use skein_lib::{Duration, Reader, Secret, Time};
use skein_world::{HostedProgram, Inherited, StartupRoot};
use std::{cell::RefCell, rc::Rc};

const READ: u64 = 73;
const ACCOUNT: u32 = 1;
const GENERATION: u64 = 4;
pub const KEY: &[u8] = b"list-only-fixture-key";
pub const BODY: &[u8] = br#"{"data":[{"id":"first-model","display_name":"First model","max_input_tokens":200000,"max_tokens":8192,"capabilities":{"effort":{"low":{"supported":true},"high":{"supported":true}}}},{"id":"second-model"}]}"#;

#[derive(Default, Debug)]
pub struct Run {
    pub answers: Vec<wire::ListRead>,
    pub requests: Vec<(skein_llm::Provider, Option<skein_llm::Scheme>, Option<u32>)>,
    pub values_after: Option<u32>,
    pub values_expired: Option<u32>,
}

struct Judge {
    finding: Option<String>,
    shared: Rc<RefCell<Run>>,
    store: Store,
    rules: Rules,
    scheme: Scheme,
    endpoint: u32,
    asked: bool,
    done: bool,
    observed: usize,
    expires: Time,
}

impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        if self.finding.is_none() {
            self.finding = self.check_act(now, procs).err();
        }
    }
    fn watch(&mut self, _now: Time, _procs: &[Proc]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }
        Ok(())
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now > Time::ZERO.saturating_add(Duration::from_secs(10))).then(|| "worker list read settles".into())
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && self.done
    }
    fn last_word(&self) -> skein_world::LastWord {
        skein_world::LastWord::LastExpectation
    }
}

fn configuration() -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "profile":{"name":"standard","declared":{"memory":2_147_483_648_u64,"tool_payload":256,"read_window":256}},
        "endpoints":[{"name":"fake","number":1,"dialect":3,"provider":"openai",
            "address":"127.0.0.1:7101","transport":"plaintext","connect":1000,"handshake":1000,
            "limits":{"window":65536,"output":512,"reasoning_item":64,"head":1000,"idle":1000}},
            {"name":"other","number":2,"dialect":2,"provider":"anthropic",
            "address":"127.0.0.1:7101","transport":"plaintext","connect":1000,"handshake":1000,
            "limits":{"window":65536,"output":512,"reasoning_item":64,"head":1000,"idle":1000}}],
        "environment":[]
    }))
    .expect("list agent configuration")
}

fn agent(spawn: &skein_io::kernel::Spawn, inherited: &Inherited) -> Proc {
    assert_eq!(spawn.program.as_ref(), b"agent-program");
    assert_eq!(spawn.args.as_ref(), &[Box::from(b"--list".as_slice()), Box::from(b"agent.json".as_slice())]);
    assert!(spawn.env.is_empty(), "the agent's ordinary environment holds no credential");
    let pipe = |number| inherited.pipes.iter().find(|(child, _)| *child == number).expect("list pipe").1;
    Proc::Auth(Box::new(
        smith_agent_shell::list::Process::new(
            smith_agent_shell::config::decode(&configuration()).expect("list configuration"),
            smith_agent_shell::list::Faces {
                stdin: pipe(0),
                stdout: pipe(1),
                stderr: Some(pipe(2)),
                signal: inherited.signal,
            },
        )
        .expect("list child"),
    ))
}

/// Read one fake list through both real channel halves and the shipped agent list process.
#[must_use]
pub fn run(status: Option<u16>, scheme: Scheme, endpoint: u32) -> Run {
    let mut files = skein_fake_machine::Machine::new();
    let base = files.lay(&[
        Item::directory(b"state"),
        Item::directory(b"private").mode(0o700),
        Item::directory(b"private/programs").mode(0o700),
        Item::file(b"agent.json", &configuration()),
    ]);
    let shared = Rc::new(RefCell::new(Run::default()));
    let judge = Judge {
        finding: None,
        shared: shared.clone(),
        store: Store::default(),
        rules: Rules::default(),
        scheme,
        endpoint,
        asked: false,
        done: false,
        observed: 0,
        expires: Time::ZERO,
    };
    let mut world = skein_world::World::new(913, skein_sim::Config::calm(), judge, skein_world::Memory::Unchecked)
        .with_machine(Files { files, base });
    world.host(HostedProgram {
        program: b"agent-program".as_slice().into(),
        make: agent,
        instances: 1,
        operations: smith_agent_shell::list::operations(),
    });
    world.spawn_restartable(
        vec![StartupRoot { name: b"state".as_slice().into(), path: b"state".as_slice().into() }],
        processes::engine,
    );
    world.spawn_restartable(
        vec![
            StartupRoot { name: b"worker".as_slice().into(), path: b".".as_slice().into() },
            StartupRoot { name: b"parent".as_slice().into(), path: b"private".as_slice().into() },
        ],
        processes::worker_lists,
    );
    world.spawn(|| {
        let credential = match scheme {
            Scheme::Key => skein_llm::Credential::key(KEY.into()),
            Scheme::Bearer => skein_llm::Credential::bearer(KEY.into()),
        };
        let mut peer = fixtures::provider_at(
            (std::net::Ipv4Addr::LOCALHOST, 7101).into(),
            Box::new([]),
            credential,
            Duration::ZERO,
            skein_fake_peers::Transport::Plaintext,
            if endpoint == 1 {
                skein_fake_llm_protocol::documents::Provider::OpenAi
            } else {
                skein_fake_llm_protocol::documents::Provider::Anthropic
            },
        );
        peer.set_list(Some(BODY.into()), status);
        Proc::Peer(Box::new(peer))
    });
    let mut outcome = world.run();
    let mut result = std::mem::take(&mut *shared.borrow_mut());
    for proc in &outcome.procs {
        if let Proc::Peer(peer) = proc {
            for observation in peer.observations() {
                match observation {
                    skein_fake_peers::llm::Observation::List { route, scheme, index, .. } => {
                        result.requests.push((*route, *scheme, *index));
                    }
                    skein_fake_peers::llm::Observation::Query { .. } => panic!("list read starts no completion"),
                    _ => {}
                }
            }
        }
    }
    outcome.machine.files.close(base);
    assert_eq!(outcome.machine.files.open_handles(), 0, "all list world files settle");
    result
}

impl Judge {
    fn check_act(&mut self, now: Time, procs: &mut [Proc]) -> Result<(), String> {
        let ready = procs.iter().any(|proc| {
            matches!(proc, Proc::Worker(worker) if worker.wire.frames.iter().any(|(_, kind, _)| *kind == wire::Kind::Logins.number()))
        });
        for proc in procs.iter_mut() {
            match proc {
                Proc::Engine(host) => {
                    world::answer_store(host, &mut self.store, &mut self.rules, false, Story::Chat, now)?;
                    let engine = host.service.engine().ok_or("engine")?;
                    if ready && !self.asked {
                        self.expires = now.saturating_add(Duration::from_secs(1));
                        engine.put_account(
                            ACCOUNT,
                            GENERATION,
                            jig_protocol_accounts::table::Value {
                                token: Secret::new(KEY.into()),
                                scheme: match self.scheme {
                                    Scheme::Key => jig_protocol_accounts::table::Scheme::Key,
                                    Scheme::Bearer => jig_protocol_accounts::table::Scheme::Bearer,
                                },
                            },
                            self.expires,
                        );
                        engine.parties_in().push(core::Event::Fleet(fleet::Event::ReadList {
                            read: READ,
                            endpoint: self.endpoint,
                            grant: Grant {
                                account: ACCOUNT,
                                generation: GENERATION,
                                form: Form::Credential(self.scheme),
                                valid: Duration::from_secs(1),
                            },
                        }));
                        self.asked = true;
                    }
                    for (_, kind, bytes) in host.wire.frames.iter().skip(self.observed) {
                        if *kind == wire::Kind::ListRead.number() {
                            let answer = wire::ListRead::decode(&wire::CEILINGS, &mut Reader::new(bytes))
                                .map_err(|why| format!("engine receives whole list read: {why:?}"))?;
                            {
                                let (observed_value, wanted) = (&(answer.read()), &(READ));
                                if observed_value != wanted {
                                    return Err(format!(
                                        "read number returns through the channel; assertion `left == right` failed: left: {observed_value:?}, right: {wanted:?}"
                                    ));
                                }
                            }
                            self.shared.borrow_mut().answers.push(answer);
                        }
                    }
                    self.observed = host.wire.frames.len();
                }
                Proc::Worker(_)
                | Proc::Agent { .. }
                | Proc::Auth(_)
                | Proc::Client(_)
                | Proc::Peer(_)
                | Proc::Probe(_) => {}
            }
        }
        if !self.shared.borrow().answers.is_empty() {
            for proc in procs.iter_mut() {
                if let Proc::Worker(host) = proc {
                    let count = host.service.worker(0).ok_or("worker")?.value_count();
                    self.shared.borrow_mut().values_after.get_or_insert(count);
                    // The receiver's clock starts validity when the grant arrives.
                    if self.scheme == Scheme::Key || now > self.expires.saturating_add(Duration::from_millis(10)) {
                        self.shared.borrow_mut().values_expired = Some(count);
                        self.done = true;
                    }
                }
            }
        }
        for proc in procs {
            match proc {
                Proc::Engine(host) | Proc::Worker(host) => {
                    self.rules.roots(&format!("{:?}", Roots(&host.service)), &[KEY]);
                    if self.done && !host.stopping {
                        host.stopping = true;
                        host.service.shutdown();
                    }
                }
                Proc::Peer(peer) if self.done => peer.shutdown(),
                Proc::Peer(_) | Proc::Agent { .. } | Proc::Auth(_) | Proc::Client(_) | Proc::Probe(_) => {}
            }
        }
        {
            let (observed_value, wanted) = (&(self.rules.verdict()), &(skein_world::domain::Verdict::Passed));
            if observed_value != wanted {
                return Err(format!(
                    "root isolation; assertion `left == right` failed: left: {observed_value:?}, right: {wanted:?}"
                ));
            }
        }

        Ok(())
    }
}
