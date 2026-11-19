//! Real worker source commands, smith's host half and the hosted fake over the channel.
use crate::{
    processes::{self, Proc},
    program_sources::Files,
    referee::Rules,
    stories::Story,
    world::{self, Store},
};
use jig_domain_host::LinkDown;
use jig_domain_link::SourceName;
use jig_protocol_worker_wire as wire;
use skein_fake_machine::{How, Item};
use skein_io::kernel;
use skein_lib::{Duration, Reader, Time, Token};
use skein_world::{HostedProgram, StartupRoot};
use std::{cell::RefCell, rc::Rc};

pub const READ: u64 = 73;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Case {
    Read,
    Missing,
    Spent,
    Lost,
    Ordered,
}
#[derive(Default, Debug)]
pub struct Run {
    pub answers: Vec<wire::UsageRead>,
    pub records: Vec<u16>,
    pub observations: Vec<smith_fake_claude::Observation>,
    pub usage_started: bool,
    pub reconnected: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    New,
    Sources,
    SignOut,
    Asked,
    Cut,
    Done,
}
struct Judge {
    finding: Option<String>,
    shared: Rc<RefCell<Run>>,
    store: Store,
    rules: Rules,
    case: Case,
    phase: Phase,
    observed: usize,
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        if self.finding.is_none() {
            self.finding = self.check_act(now, procs).err();
        }
    }
    fn watch(&mut self, _now: Time, _procs: &[Proc]) -> Result<(), String> {
        self.finding.clone().map_or(Ok(()), Err)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now > Time::ZERO.saturating_add(Duration::from_secs(30)))
            .then(|| format!("usage read settles in {:?}: {:?}", self.phase, self.shared.borrow()))
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && self.phase == Phase::Done
    }
    fn last_word(&self) -> skein_world::LastWord {
        skein_world::LastWord::LastExpectation
    }
}
impl Judge {
    fn check_act(&mut self, now: Time, procs: &mut [Proc]) -> Result<(), String> {
        let ready = procs.iter().any(|proc| matches!(proc, Proc::Worker(worker) if worker.wire.frames.iter().any(|(_, kind, bytes)| *kind == wire::Kind::Logins.number() && wire::Logins::decode(&wire::CEILINGS, &mut Reader::new(bytes)).is_ok_and(|rows| rows.logins().len() == 1))));
        for proc in procs.iter_mut() {
            if let Proc::Engine(host) = proc {
                world::answer_store(host, &mut self.store, &mut self.rules, false, Story::Chat, now)?;
                let engine = host.service.engine().ok_or("engine")?;
                let source =
                    SourceName::new(if self.case == Case::Missing { b"absent" } else { b"alice" }).ok_or("source")?;
                if self.store.applied > 0 && self.phase == Phase::New {
                    engine.parties_in().push(jig_domain_core::Event::Fleet(jig_domain_fleet::Event::Sources {
                        sources: Box::new([source]),
                    }));
                    self.phase = Phase::Sources;
                }
                if ready && self.phase == Phase::Sources && self.case == Case::Ordered {
                    self.observed = host.wire.frames.len();
                    engine.parties_in().push(jig_domain_core::Event::Fleet(jig_domain_fleet::Event::SignOut {
                        to: skein_lib::ReplyTo::new(Token::new(700)),
                        channel: Token::new(1),
                        name: source,
                    }));
                    self.phase = Phase::SignOut;
                }
                let sign_out_sent = self.phase == Phase::SignOut
                    && host
                        .wire
                        .frames
                        .iter()
                        .skip(self.observed)
                        .any(|(_, kind, _)| *kind == wire::Kind::SignOut.number());
                if ready && self.phase == Phase::Sources || sign_out_sent {
                    if self.phase == Phase::Sources {
                        self.observed = host.wire.frames.len();
                    }
                    engine.send_worker(Token::new(1), LinkDown::ReadUsage { read: READ, source });
                    if self.case == Case::Ordered {
                        queue_list(engine, now);
                        engine.send_worker(Token::new(1), LinkDown::Check { name: source });
                    }
                    self.phase = Phase::Asked;
                }
                if self.case == Case::Lost && self.shared.borrow().usage_started && self.phase == Phase::Asked {
                    // Ensure network traffic while the usage child is silent, then reset it below.
                    engine.send_worker(Token::new(1), LinkDown::Check { name: source });
                    self.phase = Phase::Cut;
                }
                for (_, kind, bytes) in host.wire.frames.iter().skip(self.observed) {
                    self.shared.borrow_mut().records.push(*kind);
                    if *kind == wire::Kind::UsageRead.number() {
                        self.shared.borrow_mut().answers.push(
                            wire::UsageRead::decode(&wire::CEILINGS, &mut Reader::new(bytes))
                                .map_err(|why| format!("whole usage answer: {why:?}"))?,
                        );
                    }
                }
                self.observed = host.wire.frames.len();
            }
            if let Proc::Worker(host) = proc {
                let worker = host.service.worker(0).ok_or("worker")?;
                if worker.hosted() != 0 {
                    return Err("usage read takes no run slot".into());
                }
                if host.wire.hellos >= 2
                    && host
                        .wire
                        .frames
                        .iter()
                        .rev()
                        .take_while(|(_, kind, _)| *kind != wire::Kind::Hello.number())
                        .any(|(_, kind, _)| *kind == wire::Kind::Logins.number())
                {
                    self.shared.borrow_mut().reconnected = true;
                }
            }
        }
        let result = self.shared.borrow();
        let done = if self.case == Case::Lost {
            result.reconnected
        } else if self.case == Case::Ordered {
            result.records.contains(&wire::Kind::ListRead.number())
                && result.records.iter().filter(|kind| **kind == wire::Kind::Logins.number()).count() >= 1
        } else {
            !result.answers.is_empty()
        };
        drop(result);
        if done {
            self.phase = Phase::Done;
            for proc in procs {
                if let Proc::Engine(host) | Proc::Worker(host) = proc
                    && !host.stopping
                {
                    host.stopping = true;
                    host.service.shutdown();
                }
            }
        }
        Ok(())
    }
}
#[must_use]
pub fn run(case: Case) -> Run {
    let usage_read = match case {
        Case::Lost => smith_fake_claude::UsageReply::Silent,
        Case::Read | Case::Missing | Case::Spent | Case::Ordered => {
            smith_fake_claude::UsageReply::Result(serde_json::json!({
                "type":"result", "is_error":false,
                "result":"Current session\n34.5% used\nCurrent week (all models)\n100% used\n",
                "usage":{"input_tokens": u32::from(case == Case::Spent), "output_tokens":0},
                "total_cost_usd":0,
            }))
        }
    };
    let story = smith_fake_claude::Story { usage_read, logout: smith_fake_claude::Logout::Fails, ..Default::default() };
    let mut files = skein_fake_machine::Machine::new();
    let base = files.lay(&[
        Item::directory(b"state"),
        Item::directory(b"private").mode(0o700),
        Item::directory(b"private/home").mode(0o700),
        Item::directory(b"private/programs").mode(0o700),
        Item::directory(b"private/programs/alice").mode(0o700),
        Item::file(b"private/programs/alice/fake-claude.story", &story.encode()),
        Item::file(b"private/programs/alice/fake-claude.login", b"claude.ai"),
    ]);
    let shared = Rc::new(RefCell::new(Run::default()));
    let judge = Judge {
        finding: None,
        shared: shared.clone(),
        store: Store::default(),
        rules: Rules::default(),
        case,
        phase: Phase::New,
        observed: 0,
    };
    let config = skein_sim::Config::calm();
    let mut world =
        skein_world::World::new(914, config, judge, skein_world::Memory::Unchecked).with_machine(Files { files, base });
    let factory = smith_fake_claude::hosted(b"claude");
    world.host_startup(
        HostedProgram {
            program: factory.program,
            make: processes::auth,
            instances: factory.instances,
            operations: factory.operations,
        },
        |spawn| smith_fake_claude::config_root(spawn).into_iter().chain(smith_fake_claude::mcp_root(spawn)).collect(),
        |spawn| smith_fake_claude::observation_append(spawn).into_iter().collect(),
    );
    world.spawn_restartable(
        vec![StartupRoot { name: b"state".as_slice().into(), path: b"state".as_slice().into() }],
        processes::engine,
    );
    world.spawn_restartable(
        vec![
            StartupRoot { name: b"worker".as_slice().into(), path: b".".as_slice().into() },
            StartupRoot { name: b"parent".as_slice().into(), path: b"private".as_slice().into() },
        ],
        processes::worker_programs,
    );
    let mut reset = false;
    let mut outcome = world.run_with_faults(|_, _, submissions| {
        for submit in submissions {
            if let kernel::Op::Append { bytes, .. } = &submit.kind
                && bytes.windows(b"/usage".len()).any(|word| word == b"/usage")
            {
                shared.borrow_mut().usage_started = true;
            }
        }
        let cut = case == Case::Lost
            && !reset
            && shared.borrow().usage_started
            && submissions.iter().any(|submit| matches!(submit.kind, kernel::Op::Send { .. }));
        reset |= cut;
        Some(skein_sim::Faults { reset: if cut { 1000 } else { 0 }, ..config.faults })
    });
    let bytes = match outcome.machine.files.open(base, b"private/programs/alice/fake-claude.observations", How::Read) {
        Ok(fd) => {
            let bytes = outcome.machine.files.read(fd, 0, 65536).expect("observations");
            outcome.machine.files.close(fd);
            bytes
        }
        Err(skein_fake_machine::Refusal::NotFound) => Vec::new(),
        Err(error) => panic!("observations: {error:?}"),
    };
    outcome.machine.files.close(base);
    assert_eq!(outcome.machine.files.open_handles(), 0, "usage files settle");
    let mut result = std::mem::take(&mut *shared.borrow_mut());
    result.observations = smith_fake_claude::observations(&bytes);
    result
}

fn queue_list(engine: &mut jig_test_service::Engine, now: Time) {
    engine.put_account(
        1,
        1,
        jig_protocol_accounts::table::Value {
            token: skein_lib::Secret::new(b"usage-list-fixture".as_slice().into()),
            scheme: jig_protocol_accounts::table::Scheme::Key,
        },
        now.saturating_add(Duration::from_secs(1)),
    );
    engine.send_worker(
        Token::new(1),
        LinkDown::ReadList {
            read: READ + 1,
            endpoint: 1,
            grant: jig_domain_link::Grant {
                account: 1,
                generation: 1,
                form: jig_domain_link::Form::Credential(jig_domain_link::Scheme::Key),
                valid: Duration::from_secs(1),
            },
        },
    );
}
