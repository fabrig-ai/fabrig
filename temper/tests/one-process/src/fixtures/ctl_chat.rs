//! A hosted ctl and a recording provider on a disk that can be reopened after drain
//! (temper/docs/design/testing/tiers.md, 1.5).
mod drain;
pub use drain::receipts_across_drained_services;

use crate::{
    ctl::{self, Caller, Command},
    fixtures,
    referee::Store,
    stories,
    world::{self, HarnessProcess, ServiceObserver, Story},
};
use jig_domain_core as core;
use jig_domain_tasks::{Active, Phase, Stored};
use serde_json::{Value, json};
use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
use skein_fake_peers::llm::Observation;
use skein_io::kernel::{Exit, ServiceSignal};
use skein_lib::Time;
use skein_shell::Host;
use std::{cell::RefCell, net::SocketAddr, rc::Rc};
use temper_engine_domain::Record;
use temper_playground::{
    documents::{self, Configuration},
    fakes,
    machine::{self, Machine, SignalHost},
    programs::{self, Harness},
};

const WORDS: &[u8] = b"Hello";
const REPLY: &[u8] = fixtures::ANSWER;
const WAITING: u64 = 2_000_000;
const FIRST: SocketAddr = SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 7101);
const SECOND: SocketAddr = SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 7103);
// Synthetic fixture material; the caller only sees the path.
const FILE_KEY: &[u8] = b"private-file-smoke-llm-key";
const FILE: &[u8] = b"/setup/private-chat.key";

struct Chat;
type Proc = programs::Process<Chat>;
impl Harness for Chat {
    type Service = ServiceObserver;
    type Process = HarnessProcess;

    fn seeds(_: u64) -> temper_service::settings::Seeds {
        fixtures::seeds()
    }

    fn given(document: &Configuration) -> temper_service::stock::Given {
        let mut given = documents::given(document);
        let temper_service::stock::Given::Own { jig, .. } = &mut given else { panic!("fixture defaults") };
        jig.charters[0].policy.waiting = skein_lib::Duration::from_millis(1);
        jig.templates[0].flags.ended_by = jig_domain_tasks::EndedBy::Requester;
        given
    }

    fn agent_llm_address() -> Option<SocketAddr> {
        Some(
            programs::parameters().deployment["world_settings"]["llm"]["endpoints"][0]["address"]["socket"]
                .as_str()
                .expect("fixture dial")
                .parse()
                .expect("fixed socket address"),
        )
    }
}

#[derive(Clone, Copy)]
enum Start {
    FileKey,
    Before,
    After(u64),
}
impl Start {
    fn address(self) -> SocketAddr {
        if matches!(self, Self::After(_)) { SECOND } else { FIRST }
    }
    fn commands(self, caller: &mut Caller) {
        if let Self::After(chat) = self {
            command(caller, &[b"chat", b"show", chat.to_string().as_bytes()]);
            command(caller, &[b"settings", b"show", b"--json"]);
            command(caller, &[b"settings", b"show", b"charter", b"1", b"--json"]);
            return;
        }
        let file = if matches!(self, Self::FileKey) { FILE } else { b"/setup/llm.key" };
        command(caller, &[b"account", b"add-key", b"1", file, b"--deployment", b"--key", b"smoke-llm"]);
        // The providers-only store allocates source 0, then account 1.
        command(caller, &[b"account", b"rank", b"1", b"1"]);
        command(caller, &[b"account", b"list", b"--until", b"usable"]);
        if matches!(self, Self::Before) {
            command(caller, &[b"settings", b"show", b"charter", b"1", b"--json"]);
            command(
                caller,
                &[b"settings", b"apply", b"/setup/waiting.json", b"--against", b"1", b"--key", b"restart-settings"],
            );
            command(caller, &[b"settings", b"show", b"--json"]);
        }
        let name = if matches!(self, Self::FileKey) { b"key-chat".as_slice() } else { b"kept" };
        command(caller, &[b"chat", b"start", b"--project", b"1", WORDS, b"--key", name]);
    }
}

fn command(caller: &mut Caller, words: &[&[u8]]) {
    caller.commands.push_back(Command::new(words));
}

struct Judge {
    controls: skein_world::Controls,
    start: Start,
    store: Rc<RefCell<Store>>,
    answered: usize,
    target: usize,
    chat: Option<u64>,
    reading_reply: bool,
    stopping: bool,
    peer_stopped: bool,
    wake: Option<Time>,
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, _: Time, procs: &mut [Proc]) {
        let [Proc::Service(service), Proc::Other(HarnessProcess::Calls(caller)), Proc::Peer(peer), ..] = procs else {
            panic!("service, ctl caller, then provider")
        };
        self.wake = None;
        if self.stopping {
            if service.is_empty() && !self.peer_stopped {
                peer.shutdown();
                self.peer_stopped = true;
            }
            return;
        }
        if self.target == 0 {
            assert!(service.service.end().is_none(), "service before sign-in: {:?}", service.service.end());
            if let Some(link) = service.service.links().pop() {
                if !matches!(self.start, Start::After(_)) {
                    assert!(
                        !self.store.borrow().rows.values().any(|row| matches!(
                            row,
                            Record::Core(core::Record::Accounts(jig_domain_accounts::Stored::Account { .. }))
                        )),
                        "the fresh store has no account before ctl adds one"
                    );
                }
                let mut sign_in = Command::new(&[b"sign-in", b"-"]);
                sign_in.input = link;
                caller.commands.push_back(sign_in);
                self.start.commands(caller);
                self.target = caller.commands.len();
            }
            return;
        }
        for answer in &caller.answers[self.answered..] {
            self.answered += 1;
            assert_eq!(
                answer.exit,
                Exit::Code(0),
                "ctl call {}: {}",
                self.answered,
                String::from_utf8_lossy(&answer.error)
            );
        }
        if self.answered != self.target {
            return;
        }
        if self.chat.is_none() && !matches!(self.start, Start::After(_)) {
            let number = chat_number(&caller.answers[self.target - 1].output);
            self.chat = Some(number);
            let number = number.to_string();
            command(caller, &[b"task", b"show", number.as_bytes(), b"--until", b"replied"]);
            self.target += 1;
            return;
        }
        if self.chat.is_some_and(|number| !parked(&self.store.borrow().rows, number)) {
            return;
        }
        if matches!(self.start, Start::Before) && !self.reading_reply {
            let number = self.chat.expect("opened chat").to_string();
            command(caller, &[b"chat", b"show", number.as_bytes()]);
            self.target += 1;
            self.reading_reply = true;
            return;
        }
        caller.closed = true;
        self.controls.signal(0, ServiceSignal::Terminate);
        self.stopping = true;
    }
    fn watch(&mut self, _: Time, procs: &[Proc]) -> Result<(), String> {
        if let Proc::Service(service) = &procs[0]
            && let Some(why) = &service.observer.finding
        {
            return Err(why.clone());
        }
        Ok(())
    }
    fn idle(&mut self, now: Time, procs: &[Proc], _: bool) {
        // The last service completion can leave only the listening peer. Give
        // the referee a turn at this same tick to close it after service drain.
        if self.stopping && !self.peer_stopped && procs[0].is_empty() {
            self.wake = Some(now);
        }
    }
    fn next_deadline(&self) -> Option<Time> {
        self.wake
    }
    fn passed(&self) -> bool {
        self.peer_stopped
    }
}

fn chat_number(output: &[u8]) -> u64 {
    std::str::from_utf8(output)
        .expect("chat start output")
        .split_once("done: task ")
        .expect("created task")
        .1
        .split(|c: char| !c.is_ascii_digit())
        .next()
        .expect("task number")
        .parse()
        .expect("numeric task number")
}

fn parked(rows: &std::collections::BTreeMap<temper_engine_domain::Key, Record>, number: u64) -> bool {
    rows.values().any(|row| {
        matches!(row,
        Record::Core(core::Record::Tasks(Stored::Live(task)))
        if task.number == number && task.phase == Phase::Active(Active::Idle))
    })
}

struct Outcome {
    answers: Vec<ctl::Answer>,
    queries: usize,
    authenticated_chat: bool,
}

fn peer(start: Start) -> skein_fake_peers::llm::Peer {
    let key = if matches!(start, Start::FileKey) { FILE_KEY } else { fixtures::LLM_TOKEN };
    fakes::scripted_provider_at(
        start.address(),
        skein_llm::Credential::key(key.into()),
        Box::new([Script {
            cue: Box::default(),
            turns: Box::new([Turn {
                lines: Box::new([
                    Line::Text { text: REPLY.into() },
                    Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
                ]),
                finish: Finish::ToolCalls,
                tokens: 1,
            }]),
        }]),
        17,
    )
}

fn phase(
    mut disk: skein_fake_machine::Machine,
    store: &Rc<RefCell<Store>>,
    start: Start,
) -> (skein_fake_machine::Machine, Outcome) {
    let scenario = Story::Chat { words: WORDS };
    let mut document = fixtures::world_document();
    document["world_settings"]["llm"]["endpoints"][0]["address"]["socket"] = start.address().to_string().into();
    let caller = disk.reopen_root(2);
    let machine = Machine::new(disk);
    let mut parameters = world::parameters(&machine, stories::forge(scenario));
    parameters.deployment = document.into();
    world::lower_store(Some(store.clone()));
    let mut outcome = programs::with(parameters, || {
        let mut simulation = skein_world::World::new_controlled(
            fixtures::SEED,
            skein_sim::Config::calm(),
            skein_world::Memory::Unchecked,
            |controls| Judge {
                controls,
                start,
                store: store.clone(),
                answered: 0,
                target: 0,
                chat: None,
                reading_reply: false,
                stopping: false,
                peer_stopped: false,
                wake: None,
            },
        )
        .with_machine(machine);
        for (program, roots) in programs::programs::<Chat>() {
            simulation.host_roots(program, roots);
        }
        simulation.host_roots(
            ctl::program(|spawn, inherited| Proc::Other(HarnessProcess::Ctl(Box::new(ctl::host(spawn, inherited))))),
            ctl::roots,
        );
        simulation.spawn_restartable(programs::service_roots(), programs::service::<Chat>);
        simulation.spawn_root(skein_sim::Handle::new(caller.raw()), |root| {
            let directory =
                if matches!(start, Start::After(_)) { b"/client/cold".as_slice() } else { b"/client/private" };
            Proc::Other(HarnessProcess::Calls(Box::new(Caller::at_directory(root, directory))))
        });
        simulation.spawn(|| Proc::Peer(Box::new(SignalHost::new(peer(start), None))));
        simulation.run()
    });
    assert!(outcome.procs.iter().all(Host::is_empty), "service, caller and peer drain");
    assert_eq!(outcome.procs[0].exit(), Some(Exit::Code(0)), "service drained successfully");
    world::lower_store(None);
    let disk = std::mem::replace(&mut *outcome.machine.0.borrow_mut(), machine::fresh());
    let [_, Proc::Other(HarnessProcess::Calls(caller)), Proc::Peer(peer), ..] = &mut outcome.procs[..] else {
        panic!("ctl caller and recording provider");
    };
    let observations = peer.observations();
    let queries: Vec<_> = observations
        .iter()
        .filter_map(|seen| match seen {
            Observation::Query { connection, .. } => Some(*connection),
            _ => None,
        })
        .collect();
    let authenticated_chat = queries.iter().any(|connection| {
        observations.contains(&Observation::Credential {
            connection: *connection,
            route: skein_llm::Provider::OpenAi,
            scheme: Some(skein_llm::Scheme::Key),
            index: Some(0),
        })
    });
    (disk, Outcome { answers: std::mem::take(&mut caller.answers), queries: queries.len(), authenticated_chat })
}

/// The distinct private file's key authenticates the claimed chat, not just a provider probe.
pub fn private_file_key() {
    stories::set(Story::Chat { words: WORDS });
    let mut disk = machine::fresh();
    machine::setup(&mut disk);
    machine::lay_handover(&mut disk, FILE, &[FILE_KEY, b"\n"].concat());
    let (_, ran) = phase(disk, &Rc::new(RefCell::new(Store::new())), Start::FileKey);
    assert_eq!(ran.answers.len(), 6, "all six ctl calls succeeded");
    assert_eq!(ran.queries, 1, "the chat made one model query");
    assert!(ran.authenticated_chat, "the chat's own query authenticated with the stripped private key");
}

fn occurrences(bytes: &[u8]) -> usize {
    bytes.windows(REPLY.len()).filter(|part| *part == REPLY).count()
}
fn json_answer(answer: &ctl::Answer) -> Value {
    serde_json::from_slice(&answer.output).expect("settings JSON")
}

/// Start a fresh service on the drained service's disk with a different provider dial.
pub fn cold_read() {
    stories::set(Story::Chat { words: WORDS });
    let mut disk = machine::fresh();
    machine::setup(&mut disk);
    let mut charter = documents::settings::stock(&fixtures::world_document())["charter"].clone();
    charter["waiting"] = WAITING.into();
    let settings = json!({"items": [documents::settings::item("charter", 1, charter)]});
    machine::lay_handover(&mut disk, b"/setup/waiting.json", &serde_json::to_vec(&settings).expect("settings file"));
    let store = Rc::new(RefCell::new(Store::new()));
    let (disk, first) = phase(disk, &store, Start::Before);
    let [_, _, _, _, original, _, changed, opened, _, replied] = first.answers.as_slice() else {
        panic!("first service answered sign-in, account setup, settings change and chat");
    };
    assert_eq!(json_answer(original)["value"]["waiting"], 1_000_000);
    assert_eq!(json_answer(changed)["newest"], 2);
    assert_eq!(occurrences(&replied.output), 1, "the first service printed the original reply");
    let chat = chat_number(&opened.output);
    assert!(parked(&store.borrow().rows, chat), "the replied chat settled before stop");
    store.borrow_mut().recovering();
    let (_, second) = phase(disk, &store, Start::After(chat));
    let [_, shown, inventory, charter] = second.answers.as_slice() else {
        panic!("second service answered sign-in, chat and both settings reads");
    };
    let store = store.borrow();
    assert!(parked(store.restored.as_ref().expect("cold recovery observed"), chat));
    assert_eq!((shown.exit, occurrences(&shown.output)), (Exit::Code(0), 1));
    assert_eq!(second.queries, 0, "the restarted service did not ask the model again");
    assert_eq!(json_answer(inventory)["newest"], 3, "the provider's rebind released new defaults");
    assert_eq!(json_answer(charter)["value"]["waiting"], WAITING, "the saved value overrides the 1 ms default");
}
