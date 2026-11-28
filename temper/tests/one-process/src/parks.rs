//! Parks against wakes through real smith and committed jig rows (temper/docs/design/testing/tiers.md, 1.5).
use crate::{
    ctl::{self, Caller, Command},
    fixtures,
    referee::{AgentRecord, RunFrame, Store},
    stories,
    world::{self, HarnessProcess, OneProcess, ServiceObserver, Story},
};
use jig_core_kit::{
    cells::{self, Pairing, Park},
    parks,
};
use jig_domain_core as core;
use jig_domain_tasks::{Active, EndKind, Ending, MessageKind, Party, Phase, Stored, TaskRecord, TaskResult, Why};
use skein_fake_llm_domain::api::{Finish, Line, Part, Script, Turn};
use skein_fake_peers::llm::{Observation, Peer};
use skein_io::kernel::{Exit, ServiceSignal};
use skein_lib::{Reader, Time};
use skein_shell::Host;
use skein_world::faults::FaultLedger;
use smith_channel::{RunResult, Start};
use smith_transcript::{Block, Record as _};
use std::{cell::RefCell, rc::Rc};
use temper_engine_domain::{Record, Write};
use temper_playground::{
    documents::{self, Configuration},
    machine::{self, Machine, SignalHost},
    programs::{self, Harness},
};

const CHAT: &[u8] = b"@park-chat";
const CHILD: &[u8] = b"@park-delegate";
const NOTICE: &[u8] =
    b"Nothing more will come: your delegates have ended and no question waits. Finish with your result.";
const DEADLINE: Time = Time::from_nanos(10_000_000_000);

struct ParkStory;
type Proc = programs::Process<ParkStory>;
impl Harness for ParkStory {
    type Service = ServiceObserver;
    type Process = HarnessProcess;

    fn seeds(seed: u64) -> temper_service::settings::Seeds {
        OneProcess::seeds(seed)
    }
    fn agent_seed(seed: u64) -> u64 {
        OneProcess::agent_seed(seed)
    }
    fn prepared_schemas() -> Option<&'static [skein_codec::schema::Schema]> {
        OneProcess::prepared_schemas()
    }
    fn given(document: &Configuration) -> temper_service::stock::Given {
        let mut given = documents::given(document);
        let temper_service::stock::Given::Own { jig, .. } = &mut given else { panic!("fixture defaults") };
        jig.charters[0].policy.waiting = skein_lib::Duration::from_millis(1);
        // Leave part of the chat's allotment available for its delegate while the run is live.
        jig.charters[0].spend = 8;
        given
    }
}

fn provider() -> Peer {
    fn call(name: &[u8], arguments: &[u8]) -> Turn {
        Turn {
            lines: Box::new([Line::Call { name: name.into(), arguments: arguments.into() }]),
            finish: Finish::ToolCalls,
            tokens: 1,
        }
    }
    fixtures::scripted_provider(Box::new([
        Script { cue: CHAT.into(), turns: Box::new([
            call(b"delegate", br#"{"batch":[{"executor":{"kind":"agent","charter":1},"spec":{"words":"@park-delegate"},"contract":{"kind":"report","words":40},"authority":{"tools":0,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},"budget":{"spend":12},"notes":0}}]}"#),
            call(b"wait", b"{}"),
            call(b"finish", br#"{"form":"report","text":"Delegate done","fields":{}}"#),
        ]) },
        Script { cue: CHILD.into(), turns: Box::new([
            call(b"wait", b"{}"),
            call(b"finish", br#"{"form":"report","text":"Nothing left to do","fields":{}}"#),
        ]) },
    ]))
}

#[derive(Default)]
struct Driver {
    pairing: Pairing,
    ledger: FaultLedger,
    park: Option<Park>,
    wait_id: Option<Box<[u8]>>,
    notice: Option<u64>,
}
impl Driver {
    fn smith(&mut self, task: u64, attempt: u64, start: &Start, frames: &[RunFrame], now: Time) {
        assert!(self.park.is_none(), "one armed park");
        assert!(start.transcript.is_empty() && start.messages.is_empty(), "first start has nothing queued");
        let [RunFrame::Turn(turn), RunFrame::Answer(RunResult::Parked)] = frames else {
            panic!("one whole turn before the parked answer: {frames:?}");
        };
        let decoded = smith_transcript::Turn::decode(&smith_transcript::CEILINGS, &mut Reader::new(&turn.body))
            .expect("decode the delegate's parked turn");
        let blocks: Vec<_> = decoded.messages.iter().flat_map(|message| message.blocks.iter()).collect();
        let mut calls = blocks.iter().filter_map(|block| match block {
            Block::Call(call) => Some(call),
            _ => None,
        });
        let wait = calls.next().expect("the parked turn tells its wait");
        assert_eq!(wait.name.as_ref(), b"wait");
        assert!(calls.next().is_none(), "only the wait was called");
        assert!(
            !blocks.iter().any(|block| matches!(block, Block::Result(_))),
            "the parked turn withholds the wait result"
        );
        let park = Park { task, attempt, number: 1 };
        assert!(
            self.pairing
                .fell(park, smith_faults::parks::WITHHELD_WAIT, now, &mut self.ledger)
                .expect("smith half can pair")
                .is_none()
        );
        self.wait_id = Some(wait.id.clone());
        self.park = Some(park);
    }

    fn jig(&mut self, commit: &crate::referee::durable::Commit, now: Time) {
        let Some(park) = self.park else { return };
        for write in &commit.writes {
            let Write::Save(Record::Core(core::Record::Tasks(Stored::Live(task)))) = write else { continue };
            if task.number != park.task || task.attempt != park.attempt || task.phase != Phase::Active(Active::Due) {
                continue;
            }
            let [notice] = task.inbox.as_ref() else { panic!("only the nothing-awaited notice wakes the delegate") };
            assert_eq!(notice.kind, MessageKind::NothingAwaited);
            assert_eq!(notice.words.as_ref(), NOTICE);
            assert_eq!(task.due_reason, Why::Message { message: notice.number });
            assert!(self.notice.replace(notice.number).is_none(), "one durable wake");
            let cell = self
                .pairing
                .fell(park, parks::PARKS_ON_NOTHING, now, &mut self.ledger)
                .expect("jig half can pair")
                .expect("both halves fell at this park");
            assert_eq!(cell.kind, cells::WITHHELD_WAIT_PARKS_ON_NOTHING);
        }
    }

    fn finished(&self, store: &Store, attempt: u64, start: &Start, frames: &[RunFrame], peer: &Peer) {
        let park = self.park.expect("smith park observed");
        let notice = self.notice.expect("committed wake observed");
        let [entry] = self.ledger.entries() else { panic!("one cell: {:?}", self.ledger) };
        assert_eq!(entry.kind, cells::WITHHELD_WAIT_PARKS_ON_NOTHING);
        assert_eq!(entry.struck.request.as_deref(), Some(format!("task {}", park.task).as_str()));
        assert!(attempt > park.attempt && !start.transcript.is_empty(), "new attempt resumes the saved turn");
        let [message] = start.messages.as_slice() else { panic!("the resumed Start delivers exactly one notice") };
        assert_eq!((message.name, message.text.as_ref()), (notice, NOTICE));
        let [RunFrame::Turn(turn), RunFrame::Answer(RunResult::Accepted(_))] = frames else {
            panic!("the resumed attempt finishes in one turn: {frames:?}");
        };
        assert_eq!(turn.last_read, Some(notice), "the resumed attempt read the notice");
        for (attempt, end) in [(park.attempt, EndKind::Parked), (attempt, EndKind::Finished)] {
            let key = temper_engine_domain::Key::Core(core::Key::Tasks(jig_domain_tasks::Key::Attempt {
                task: park.task,
                attempt,
            }));
            assert!(
                matches!(store.rows.get(&key), Some(Record::Core(core::Record::Tasks(Stored::Attempt(row)))) if row.end == Some(end)),
                "durable attempt {attempt} ended {end:?}"
            );
        }
        self.requests(peer);
    }

    fn requests(&self, peer: &Peer) {
        let requests: Vec<_> = peer
            .observations()
            .iter()
            .filter_map(|seen| match seen {
                Observation::Query { query, .. } if skein_lib::bytes::find(&query.system, CHILD).is_some() => {
                    Some(query)
                }
                _ => None,
            })
            .collect();
        let [before, resumed] = requests.as_slice() else {
            panic!("one request per delegate attempt: {}", requests.len())
        };
        assert!(!before.messages.iter().flat_map(|msg| &msg.parts).any(|part| matches!(part, Part::ToolOutput { .. })));
        let outputs: Vec<_> = resumed
            .messages
            .iter()
            .flat_map(|msg| &msg.parts)
            .filter_map(|part| match part {
                Part::ToolOutput { id, output, is_error } => Some((id.as_ref(), output.as_ref(), *is_error)),
                _ => None,
            })
            .collect();
        let [(id, output, false)] = outputs.as_slice() else {
            panic!("one successful tool result on resume: {outputs:?}")
        };
        assert_eq!(Some(*id), self.wait_id.as_deref(), "the result answers the parked wait");
        assert!(skein_lib::bytes::find(output, NOTICE).is_some(), "the wait result carries the waking words");
        let appearances = resumed
            .messages
            .iter()
            .flat_map(|msg| &msg.parts)
            .filter_map(|part| match part {
                Part::Text { text } => Some(text.as_ref()),
                Part::ToolOutput { output, .. } => Some(output.as_ref()),
                _ => None,
            })
            .flat_map(|words| words.windows(NOTICE.len()))
            .filter(|words| *words == NOTICE)
            .count();
        assert_eq!(appearances, 1, "the first resumed request presents the notice once");
    }
}

struct Judge {
    controls: skein_world::Controls,
    store: Rc<RefCell<Store>>,
    driver: Driver,
    submitted: bool,
    stopping: bool,
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        let [Proc::Service(service), Proc::Other(HarnessProcess::Calls(caller)), Proc::Peer(peer), ..] = procs else {
            panic!("service, ctl caller, then provider");
        };
        if !self.submitted
            && let Some(link) = service.service.links().pop()
        {
            service.observer.wire.keep_run_frames();
            let mut sign_in = Command::new(&[b"sign-in", b"-"]);
            sign_in.input = link;
            caller.commands.push_back(sign_in);
            caller.commands.extend([
                Command::new(&[
                    b"account",
                    b"add-key",
                    b"1",
                    b"/setup/llm.key",
                    b"--deployment",
                    b"--key",
                    b"park-account",
                ]),
                Command::new(&[b"account", b"rank", b"1", b"1"]),
                Command::new(&[b"chat", b"start", b"--project", b"1", CHAT, b"--key", b"park-chat"]),
            ]);
            self.submitted = true;
        }
        if self.stopping {
            return;
        }
        let store = self.store.borrow();
        let Some(chat) =
            tasks(&store).find(|task| task.flags.conversation && matches!(task.requester, Party::Person(_)))
        else {
            return;
        };
        let Some(delegate) = tasks(&store).find(|task| task.requester == Party::Task(chat.number)) else {
            assert!(!matches!(chat.phase, Phase::Ended(_)), "chat ended without making its delegate");
            return;
        };
        let task = delegate.number;
        let observer = &mut service.observer;
        let starts: Vec<_> = store
            .agent_records
            .iter()
            .filter_map(|record| match record {
                AgentRecord::Start { task: owner, attempt, input: Some(input), .. } if *owner == task => {
                    Some((*attempt, input.as_ref()))
                }
                _ => None,
            })
            .collect();
        if self.driver.park.is_none()
            && let Some((attempt, input)) = starts.first()
            && let Some(run) = observer.wire.run_frames(task, *attempt)
            && matches!(run.last(), Some(RunFrame::Answer(_)))
        {
            self.driver.smith(task, *attempt, input, run, now);
        }
        for commit in observer.commits.drain(..) {
            self.driver.jig(&commit, now);
        }
        if matches!(chat.phase, Phase::Ended(_)) && caller.answers.len() == 4 {
            assert!(
                matches!(&delegate.phase, Phase::Ended(Ending::Done(TaskResult::Report { words }))
                if words.as_ref() == b"Nothing left to do"),
                "delegate finished with its report: {:?}",
                delegate.phase
            );
            let [_, (attempt, start)] = starts.as_slice() else { panic!("two proof-correlated delegate starts") };
            let run = observer.wire.run_frames(task, *attempt).expect("resumed agent frames");
            self.driver.finished(&store, *attempt, start, run, peer);
            caller.closed = true;
            self.controls.signal(0, ServiceSignal::Terminate);
            peer.shutdown();
            self.stopping = true;
        }
    }

    fn watch(&mut self, _: Time, procs: &[Proc]) -> Result<(), String> {
        let [Proc::Service(service), Proc::Other(HarnessProcess::Calls(caller)), ..] = procs else {
            return Err("service and caller first".into());
        };
        if let Some(why) = &service.observer.finding {
            return Err(why.clone());
        }
        if let Some(answer) = caller.answers.iter().find(|answer| answer.exit != Exit::Code(0)) {
            return Err(format!("park story ctl: {answer:?}"));
        }
        if let Some(end) = service.service.end() {
            if !self.stopping || *end != temper_service::End::Engine(temper_service::EngineEnd::Drained) {
                return Err(format!("park story service ended: {end:?}"));
            }
            if service.is_empty() {
                crate::referee::Referee::check_roots(&service.service)?;
            }
        }
        Ok(())
    }

    fn next_deadline(&self) -> Option<Time> {
        (!self.stopping).then_some(DEADLINE)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.stopping && now >= DEADLINE).then(|| {
            let store = self.store.borrow();
            let tasks: Vec<_> = tasks(&store).map(|task| (task.number, &task.phase)).collect();
            format!("park story stalled: tasks {tasks:?}, park {:?}, notice {:?}", self.driver.park, self.driver.notice)
        })
    }
    fn passed(&self) -> bool {
        self.stopping && self.store.borrow().pending.is_empty()
    }
}

fn tasks(store: &Store) -> impl Iterator<Item = &TaskRecord> {
    store.rows.values().filter_map(|row| match row {
        Record::Core(core::Record::Tasks(Stored::Live(task) | Stored::Ended(task))) => Some(task.as_ref()),
        _ => None,
    })
}

/// A person's chat delegates an agent whose next attempt answers its withheld wait.
pub fn withheld_wait() {
    let scenario = Story::Chat { words: CHAT };
    stories::set(scenario);
    stories::setup_settings(world::Phase::Story);
    let store = Rc::new(RefCell::new(Store::new()));
    world::lower_store(Some(store.clone()));
    let mut disk = machine::fresh();
    machine::setup(&mut disk);
    let client = disk.reopen_root(2);
    let machine = Machine::new(disk);
    let mut parameters = world::parameters(&machine, stories::forge(scenario));
    let mut document = fixtures::world_document();
    document.jig_over_all([("tasks.project_tasks", 2), ("tasks.tree_tasks", 2)]);
    for project in document["world_settings"]["policies"].as_array_mut().expect("world policies") {
        for pointer in ["/policy/ceiling", "/policy/roles/0/authority"] {
            let authority = project.pointer_mut(pointer).expect("project authority");
            authority["tools"] = 1.into(); // Delegate; the existing policy names charter 1.
            authority["delegation"]["tasks"] = 2.into(); // The chat itself takes one place.
        }
    }
    parameters.deployment = document.into();
    let outcome = programs::with(parameters, || {
        let mut simulation = skein_world::World::new_controlled(
            fixtures::SEED,
            skein_sim::Config::calm(),
            skein_world::Memory::Unchecked,
            |controls| Judge { controls, store, driver: Driver::default(), submitted: false, stopping: false },
        )
        .with_machine(machine);
        for (program, roots) in programs::programs::<ParkStory>() {
            simulation.host_roots(program, roots);
        }
        simulation.host_roots(
            ctl::program(|spawn, inherited| Proc::Other(HarnessProcess::Ctl(Box::new(ctl::host(spawn, inherited))))),
            ctl::roots,
        );
        simulation.spawn_restartable(programs::service_roots(), programs::service::<ParkStory>);
        simulation.spawn_root(skein_sim::Handle::new(client.raw()), |root| {
            Proc::Other(HarnessProcess::Calls(Box::new(Caller::scripted(root))))
        });
        simulation.spawn(|| Proc::Peer(Box::new(SignalHost::new(provider(), None))));
        simulation.run()
    });
    world::lower_store(None);
    assert!(outcome.procs.iter().all(Host::is_empty), "park story drains every process");
    assert_eq!(outcome.procs[0].exit(), Some(Exit::Code(0)), "service drained successfully");
}
