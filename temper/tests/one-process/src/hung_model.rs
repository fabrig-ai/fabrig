//! The provider holds a response after receiving a shell result (temper/docs/design/testing/tiers.md, 1.5).
use crate::{
    ctl::{Caller, Command},
    fixtures,
    referee::Store,
    stories,
    world::{self, HarnessProcess, ServiceObserver, Story},
};
use jig_domain_tasks as tasks;
use serde_json::{Value, json};
use skein_fake_llm_domain::api::{Finish, Line, Part, Piece, Query, Script, Turn};
use skein_fake_peers::llm::{Observation, Peer};
use skein_io::kernel::{Exit, ServiceSignal};
use skein_lib::{Duration, Time};
use skein_shell::Host;
use std::{cell::RefCell, rc::Rc};
use temper_engine_domain::{Key, Record};
use temper_playground::{
    documents,
    machine::{self, Machine},
    programs::{self, Harness},
};

const HEAD: Duration = Duration::from_millis(250);
const IDLE: Duration = Duration::from_millis(300);
// Closing an expired transport is separate from the idle deadline and retry wait.
const CLOSE: Duration = Duration::from_millis(20);
const HOLD: &[u8] = b"hung-after-shell";
const MARKER: &[u8] = b"hung-shell-result";
const REPORT: &str = "The shell result reached the model after its retry.";

struct Hosted;
type Proc = programs::Process<Hosted>;
impl Harness for Hosted {
    type Service = ServiceObserver;
    type Process = HarnessProcess;

    fn seeds(_: u64) -> temper_service::settings::Seeds {
        fixtures::seeds()
    }
    fn files() -> temper_service::settings::Files {
        fixtures::files()
    }
    fn configure(document: &mut documents::Configuration) {
        document["world_settings"] = fixtures::world_document()["world_settings"].clone();
        document["workers"][0]["slots"] = 1.into();
        // Observe a failed run without another engine run after it.
        document.jig_over("tasks.retries.run.retries", 0);
    }
    fn given(document: &documents::Configuration) -> temper_service::stock::Given {
        documents::given_with(document, false, false)
    }
    fn agent_profile() -> smith_agent_service::profile::Profile {
        let mut profile = documents::agent_profile();
        profile.policy.close = CLOSE;
        profile
    }
    fn configure_agent(
        service: &mut smith_agent_service::Config,
        declarations: &mut smith_agent_service::profile::Configuration,
    ) {
        (service.domain.limits[0].head, service.domain.limits[0].idle) = (HEAD, IDLE);
        (declarations.endpoints[0].limits.head, declarations.endpoints[0].limits.idle) = (HEAD, IDLE);
    }
    fn agent_settings() -> temper_service::settings::Agent {
        settings().0
    }
    fn agent_seed(_: u64) -> u64 {
        fixtures::SEED
    }
    fn provider(_: u64) -> Peer {
        fixtures::scripted_provider(Box::new([Script {
            cue: Box::default(),
            turns: Box::new([
                Turn {
                    lines: Box::new([Line::Call {
                        name: b"shell".as_slice().into(),
                        arguments: br#"{"command":"printf 'hung-shell-result\\n'"}"#.as_slice().into(),
                    }]),
                    finish: Finish::ToolCalls,
                    tokens: 1,
                },
                Turn {
                    lines: Box::new([
                        Line::Pieces {
                            pieces: Box::new([Piece {
                                text: b"Recovered.".as_slice().into(),
                                hold: Some(HOLD.into()),
                            }]),
                        },
                        Line::Call {
                            name: b"finish".as_slice().into(),
                            arguments: serde_json::to_vec(&json!({"form":"report","text":REPORT,"fields":{}}))
                                .expect("finish arguments")
                                .into(),
                        },
                    ]),
                    finish: Finish::ToolCalls,
                    tokens: 1,
                },
            ]),
        }]))
    }
}

fn settings() -> (temper_service::settings::Agent, Timing) {
    let profile = Hosted::agent_profile();
    let mut service = documents::agent_configuration_for(&profile);
    Hosted::configure_agent(&mut service, &mut documents::agent_declarations_for(&profile));
    let endpoint = &service.domain.limits[0];
    let session = service.limits.domain.session;
    assert_eq!(endpoint.head.as_millis(), 250);
    assert_eq!(endpoint.idle.as_millis(), 300);
    assert_eq!(session.retries, 3, "stock session retries");
    let tries = u64::from(session.retries);
    let turn = endpoint.head.saturating_add(endpoint.idle);
    let timing = Timing {
        gap_min: endpoint.idle.saturating_add(Duration::from_nanos(session.backoff_base.as_nanos() / 2)),
        gap_max: turn.saturating_add(session.backoff_max).saturating_add(CLOSE),
        // One transport close per attempt; calm io does not inject latency.
        envelope: turn
            .saturating_add(CLOSE)
            .saturating_mul(tries + 1)
            .saturating_add(session.backoff_max.saturating_mul(tries)),
    };
    let worker = documents::agent_from_configuration(&profile, &service);
    assert!(worker.host_limits.no_progress >= session.tools.tool_deadline);
    assert!(worker.host_limits.no_progress > timing.envelope.saturating_mul(10));
    (worker, timing)
}

#[derive(Clone, Copy)]
struct Timing {
    gap_min: Duration,
    gap_max: Duration,
    envelope: Duration,
}

fn service(inherited: &skein_world::Inherited) -> Proc {
    let Proc::Service(mut host) = programs::service::<Hosted>(inherited) else { unreachable!("service factory") };
    host.observer.worker_notes = Some(Vec::new());
    Proc::Service(host)
}

fn open(caller: &mut Caller, link: Box<[u8]>) {
    let mut sign_in = Command::new(&[b"sign-in", b"-"]);
    sign_in.input = link;
    caller.commands.push_back(sign_in);
    for words in [
        [
            "forge",
            "connect",
            "forgejo-admin",
            "/setup/admin.token",
            "--api",
            "127.0.0.1:3000",
            "--git",
            "http://127.0.0.1:3000",
            "--key",
            "hung-forge",
        ]
        .as_slice(),
        &["forge", "list", "--until", "connected:1"],
        &[
            "repository",
            "adopt",
            "--project",
            "1",
            "--forge",
            "1",
            "--owner",
            "acme",
            "--name",
            "app",
            "--landing",
            "main",
            "--key",
            "hung-home",
        ],
        &["account", "add-key", "1", "/setup/llm.key", "--deployment", "--key", "hung-account"],
        &["account", "list", "--until", "usable"],
        &["chat", "start", "--project", "1", "--key", "hung-chat", "Hello"],
    ] {
        call(caller, words);
    }
}
const OPEN_ANSWERS: usize = 7;
fn call(caller: &mut Caller, words: &[&str]) {
    caller.commands.push_back(Command::new(&words.iter().map(|word| word.as_bytes()).collect::<Vec<_>>()));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    SignIn,
    OpenChat,
    WaitTask,
    ReadTask,
    EndHeldTask,
    Stop,
}
struct Judge {
    controls: skein_world::Controls,
    store: Rc<RefCell<Store>>,
    recover: bool,
    timing: Timing,
    stage: Stage,
    answers: usize,
    task: String,
    released: bool,
    deadline: Time,
}
impl Judge {
    fn try_release(&mut self, peer: &mut Peer, count: u32) {
        // is_held alone could refer to the detached first attempt.
        if !self.released && peer.is_held(HOLD) && peer.held_count(HOLD) == count {
            peer.release(HOLD).expect("release scripted response");
            self.released = true;
        }
    }

    fn read(&mut self, now: Time, caller: &mut Caller, peer: &mut Peer) {
        let answer = &caller.answers[OPEN_ANSWERS + 1];
        let read: Value = serde_json::from_slice(&answer.output).expect("task JSON");
        check_activity(&read, self.recover);
        check_queries(peer, self.recover, self.timing, now);
        if self.recover {
            assert!(self.released, "the retry reached the hold before it was released");
            let report = read
                .pointer("/task/result/value/report/bytes")
                .or_else(|| read.pointer("/task/result/ending/done/report/bytes"))
                .expect("accepted report in the live ending or reclaimed result");
            assert_eq!(report, REPORT, "accepted result: {read}");
        } else {
            let held = &read["task"]["hold"];
            assert_eq!(
                held["reason"],
                json!({"failures":{"class":"run"}}),
                "a run failure, not an agent failure: {read}"
            );
            let since = {
                let task = self.task.parse().expect("chat task number");
                let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(task)));
                let store = self.store.borrow();
                let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) = store.rows.get(&key)
                else {
                    panic!("held chat's durable task record");
                };
                row.held_since.expect("hold start in the durable record").utc_second()
            };
            let minute = std::str::from_utf8(&since[..16]).expect("UTC minute");
            assert_eq!(
                held["words"]["bytes"],
                format!(
                    "It ran out of tries for run failures: fake fake: timed out, status 200, response. \
                     Held since {minute}Z. Release it to try again; amend it first if its instructions failed it."
                )
            );
            assert_eq!(held["tries"]["run"], 1);
            assert_eq!(held["tries"]["agent"], 0);
            assert!(!self.released, "all attempts stayed held through the terminal");
            self.try_release(peer, 4);
            assert!(self.released, "all four attempts reached the hold");
            call(caller, &["task", "cancel", &self.task, "The failed run was observed", "--key", "end-hung-model"]);
            call(caller, &["task", "show", &self.task, "--until", "ended"]);
            self.stage = Stage::EndHeldTask;
        }
    }
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        let [Proc::Service(host), Proc::Other(HarnessProcess::Calls(caller)), Proc::Forge(forge), Proc::Peer(peer), ..] =
            procs
        else {
            panic!("service, caller, forge and provider first");
        };
        for answer in &caller.answers[self.answers..] {
            assert_eq!(answer.exit, Exit::Code(0), "{}", String::from_utf8_lossy(&answer.error));
            self.deadline = now.saturating_add(temper_client_shell_cli::limits::DEADLINE);
        }
        self.answers = caller.answers.len();
        if self.stage == Stage::SignIn
            && let Some(link) = host.service.links().pop()
        {
            open(caller, link);
            self.stage = Stage::OpenChat;
        }
        if self.stage == Stage::OpenChat && self.answers == OPEN_ANSWERS {
            let output = std::str::from_utf8(&caller.answers[OPEN_ANSWERS - 1].output).expect("chat start text");
            let number = output
                .split("done: task ")
                .nth(1)
                .and_then(|text| text.split(|c: char| !c.is_ascii_digit()).next())
                .expect("chat task number")
                .parse::<u64>()
                .expect("task number");
            self.task = number.to_string();
            let until = if self.recover { "ended" } else { "held" };
            call(caller, &["task", "show", &self.task, "--until", until]);
            self.stage = Stage::WaitTask;
        }
        if self.stage == Stage::WaitTask {
            if self.recover {
                self.try_release(peer, 2);
            }
            if self.answers == OPEN_ANSWERS + 1 {
                // The wait form does not take an activity selector.
                call(caller, &["task", "show", &self.task, "--activity", "32", "--json"]);
                self.stage = Stage::ReadTask;
            }
        }
        if self.stage == Stage::ReadTask && self.answers == OPEN_ANSWERS + 2 {
            self.read(now, caller, peer);
            if self.recover {
                self.stage = Stage::Stop;
            }
        }
        if self.stage == Stage::EndHeldTask && self.answers == OPEN_ANSWERS + 4 {
            self.stage = Stage::Stop;
        }
        if self.stage == Stage::Stop && !caller.closed {
            caller.closed = true;
            self.controls.signal(0, ServiceSignal::Terminate);
            peer.shutdown();
            forge.shutdown();
        }
    }
    fn watch(&mut self, _: Time, procs: &[Proc]) -> Result<(), String> {
        let Proc::Service(host) = &procs[0] else { return Err("service first".into()) };
        if let Some(why) = &host.observer.finding {
            return Err(why.clone());
        }
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        (self.stage != Stage::Stop).then_some(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (self.stage != Stage::Stop && now >= self.deadline)
            .then(|| format!("held model: {:?}, {} answers", self.stage, self.answers))
    }
    fn passed(&self) -> bool {
        self.stage == Stage::Stop
    }
}

fn check_activity(read: &Value, recover: bool) {
    let rows = read["activity"]["rows"].as_array().expect("chat activity rows");
    let ended: Vec<_> = rows
        .iter()
        .filter_map(|row| {
            let source = row.pointer("/event/owner/source")?;
            Some((&source["task"], source.pointer("/line/run_ended")?))
        })
        .collect();
    assert_eq!(ended.len(), 1, "exactly one run ended: {read}");
    let (task, run) = ended[0];
    assert_eq!(task, &read["task"]["chip"]["task"], "the ended run belongs to this chat");
    if !recover {
        assert_eq!(run["end"], json!({"failed":{"class":"run"}}));
    }
}

fn outputs(query: &Query) -> impl Iterator<Item = (&[u8], bool)> {
    query.messages.iter().flat_map(|message| message.parts.iter()).filter_map(|part| {
        if let Part::ToolOutput { output, is_error, .. } = part { Some((output.as_ref(), *is_error)) } else { None }
    })
}
fn queries(peer: &Peer) -> Vec<(Time, &Query)> {
    peer.observations()
        .iter()
        .filter_map(|event| if let Observation::Query { at, query, .. } = event { Some((*at, query)) } else { None })
        .collect()
}
fn check_queries(peer: &Peer, recover: bool, timing: Timing, read_at: Time) {
    let queries = queries(peer);
    let expected = if recover { 2 } else { 4 };
    assert_eq!(queries.len(), expected + 1, "one shell turn, then {expected} held attempts");
    assert!(outputs(queries[0].1).next().is_none(), "shell query precedes its result");
    let attempts = &queries[1..];
    for (_, query) in attempts {
        assert_eq!(*query, attempts[0].1, "retry input is unchanged");
        let outputs: Vec<_> = outputs(query).collect();
        assert_eq!(outputs.len(), 1, "the shell's result in every attempt");
        let (output, is_error) = outputs[0];
        assert!(
            !is_error
                && output.starts_with(b"exit code 0\n")
                && output.windows(MARKER.len()).any(|bytes| bytes == MARKER),
            "{}",
            String::from_utf8_lossy(output)
        );
    }
    for pair in attempts.windows(2) {
        let gap = pair[1].0.saturating_since(pair[0].0);
        assert!(gap >= timing.gap_min, "idle cut and backoff before retry: {gap:?}");
        assert!(gap < timing.gap_max, "retry within endpoint and backoff bounds: {gap:?}");
    }
    let elapsed = read_at.saturating_since(attempts[0].0);
    assert!(elapsed < timing.envelope, "held request ended in {elapsed:?}, bound {:?}", timing.envelope);
}

/// Run one post-shell hold story on the simulator's clock.
pub fn run(recover: bool) {
    stories::set(Story::Adopt);
    stories::setup_settings(world::Phase::Story);
    let store = Rc::new(RefCell::new(Store::new()));
    world::lower_store(Some(store.clone()));
    let mut disk = machine::fresh();
    machine::setup(&mut disk);
    let caller = disk.reopen_root(2);
    let machine = Machine::new(disk);
    let parameters = world::parameters(&machine, stories::forge(Story::Adopt));
    let outcome = programs::with(parameters, || {
        let mut simulation = skein_world::World::new_controlled(
            fixtures::SEED,
            skein_sim::Config::calm(),
            skein_world::Memory::Unchecked,
            |controls| Judge {
                controls,
                store,
                recover,
                timing: settings().1,
                stage: Stage::SignIn,
                answers: 0,
                task: String::new(),
                released: false,
                deadline: Time::ZERO.saturating_add(temper_client_shell_cli::limits::DEADLINE),
            },
        )
        .with_machine(machine);
        for (program, roots) in programs::programs::<Hosted>() {
            simulation.host_roots(program, roots);
        }
        simulation.host_roots(
            crate::ctl::program(|spawn, inherited| {
                Proc::Other(HarnessProcess::Ctl(Box::new(crate::ctl::host(spawn, inherited))))
            }),
            crate::ctl::roots,
        );
        simulation.spawn_restartable(programs::service_roots(), service);
        simulation.spawn_root(skein_sim::Handle::new(caller.raw()), |root| {
            Proc::Other(HarnessProcess::Calls(Box::new(Caller::scripted(root))))
        });
        simulation.spawn_restartable(Vec::new(), programs::forge::<Hosted>);
        simulation.spawn_restartable(Vec::new(), programs::llm::<Hosted>);
        simulation.run()
    });
    let [Proc::Service(host), _, _, Proc::Peer(peer), ..] = outcome.procs.as_slice() else {
        panic!("service and peer")
    };
    assert!(outcome.procs.iter().all(Host::is_empty), "the hold story drains every process");
    assert!(outcome.killed.is_empty(), "no agent was killed: {:?}", outcome.killed);
    assert_eq!(
        outcome.procs.iter().filter(|proc| matches!(proc, Proc::Agent(_))).count(),
        1,
        "the only worker hosted the chat once"
    );
    assert_eq!(
        outcome.procs.iter().filter(|proc| matches!(proc, Proc::Shell(_))).count(),
        1,
        "one hosted shell invocation"
    );
    assert!(
        !host
            .observer
            .worker_notes
            .as_ref()
            .expect("worker notes kept")
            .iter()
            .any(|note| matches!(note, temper_service::worker_alone::Note::AgentStopped { .. })),
        "no agent stop, including NoProgress"
    );
    assert_eq!(queries(peer).len(), if recover { 3 } else { 5 }, "no new model request after the terminal");
    world::lower_store(None);
}
