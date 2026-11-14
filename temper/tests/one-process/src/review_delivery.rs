//! Review delivery and cancellation through bytes (temper/docs/design/testing/README.md, 2.5).
use crate::{
    ctl::{Caller, Command},
    fixtures,
    referee::{self, Store},
    stories,
    world::{self, HarnessProcess, OneProcess, Proc, Story},
};
use serde_json::{Value, json};
use skein_io::kernel::Exit;
use skein_lib::Time;
use skein_shell::Host;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use temper_playground::{
    machine::{Machine, SignalHost},
    programs,
};

thread_local! { static ACTIVE: Cell<bool> = const { Cell::new(false) }; }
const STEPS: u32 = 20_000;
const DEADLINE: Time = Time::from_nanos(10_000_000_000);
// The chat fits; the review's fixed instructions and tools exceed this envelope.
const CHARTER_BYTES: u32 = 49_152;

/// Keep the simulated goal, review and native controls within their fixture bounds.
pub(crate) fn configure(value: &mut Value) {
    if !ACTIVE.with(Cell::get) {
        return;
    }
    for (name, count) in [
        ("tasks.tasks", 12),
        ("tasks.project_tasks", 12),
        ("tasks.tree_tasks", 12),
        ("tasks.delegates", 8),
        ("tasks.spec_bytes", 512),
        ("people.inbox_tasks", 24),
        ("tasks.authority_grants", 12),
        ("tasks.authority_bytes", 512),
        ("authority.grants", 16),
        ("people.operations", 9),
        ("notes.scopes", 32),
        ("tasks.executor_kinds", 4),
        ("core.policy_bytes", 40_000),
        ("core.run_bytes", 45_000),
        ("hub.charter_bytes", CHARTER_BYTES),
    ] {
        value["engine"]["limits"]["overrides"][name] = count.into();
    }
    value["workers"][0]["limits"]["overrides"]["hub.charter_bytes"] = CHARTER_BYTES.into();
    value["application"]["limits"]["overrides"]["root.journal.run_bytes"] = 45_000.into();
    value["application"]["limits"]["overrides"]["forge.client.window_ms"] = 100.into();
    value["application"]["limits"]["overrides"]["forge.issue_policy.interval_ms"] = 100.into();
    let grant = |kind| json!({"connector":1,"kind":kind,"pattern":{"segments":["forge","127.0.0.1:3000"],"last":"open","value":"acme"}});
    let authority = json!({"tools":3,"grants":[grant(3),grant(4),grant(7),grant(8),grant(9),grant(10),grant(11),grant(12)],
        "delegation":{"kinds":[{"kind":"charter","number":1},{"kind":"charter","number":4},{"kind":"procedure","number":2}],"tasks":8,"depth":5},
        "budget":{"spend":500,"deadline_unix_seconds":null},"notes":0,"note_resources":[]});
    let policy = &mut value["world_settings"]["policies"][0]["policy"];
    policy["ceiling"] = authority.clone();
    policy["roles"][0]["authority"] = authority;
    policy["period_spend"] = 1000.into();
    policy["roles"][0]["period_spend"] = 1000.into();
    policy["projections"] = json!([grant(3), grant(4), grant(7), grant(8), grant(9), grant(10), grant(11), grant(12)]);
    value["world_settings"]["landing"]["deployment"] = json!([{
        "connector":1,"kind":4,"pattern":{"segments":["forge","127.0.0.1:3000"],"last":"open","value":"acme"},
        "enforce":true,"ci":true,"up_to_date":true,"gates":[{"number":1,"blocking":true,"freshness":"exact"}],"approvals":[]
    }]);
}

/// Make only the review exceed the worker wire while keeping its charter encodable.
pub(crate) fn given(given: &mut temper_service::stock::Given) {
    if !ACTIVE.with(Cell::get) {
        return;
    }
    let temper_service::stock::Given::Own { jig, .. } = given else { panic!("own fixture") };
    let review = jig.charters.iter_mut().find(|charter| charter.number == 4).expect("review charter");
    // It fits Smith's charter codec and exceeds the worker wire's charter bound.
    review.policy.instructions = vec![b'x'; 38_000].into();
    jig.charters[0].policy.waiting = skein_lib::Duration::from_millis(100);
    jig.templates[0].budget = jig_domain_people::TemplateBudget::AsAllowed { cap: Some(500) };
    jig.templates[0].flags.ended_by = jig_domain_tasks::EndedBy::Requester;
}

fn call(name: &str, arguments: Value) -> Value {
    let mut turn = json!({"lines":[{"call":name}],"finish":"tool_calls","tokens":1});
    turn["lines"][0]["arguments"] = arguments;
    turn
}
fn provider(goal: bool) -> skein_fake_peers::llm::Peer {
    let chat = json!({"cue":"@delivery-chat","turns":[
        if goal { call("goal", json!({"words":"@delivery-goal","agree":false})) }
        else { call("change", json!({"words":"@delivery-edit"})) },
        call("wait", json!({})), call("wait", json!({}))
    ]});
    let recovery = json!({"cue":"@recovery","turns":[call("finish", json!({"form":"report","text":"host available","fields":{}}))]});
    let edit = json!({"cue":"@delivery-edit","turns":[
        call("shell", json!({"command":"echo fixed > conflict.txt"})),
        call("finish", json!({"form":"change","text":"","fields":{"title":"Fix","body":"Fixed"}}))
    ]});
    let work = json!({"cue":"@delivery-goal","turns":[
        call("change", json!({"words":"@delivery-edit"})), call("wait", json!({})),
        call("finish", json!({"form":"report","text":"change cancelled","fields":{}}))
    ]});
    let mut document: Value = serde_json::from_slice(temper_playground::stories::A.llm).expect("checked fixture");
    document["cued"] = json!([work, edit]);
    document["asked"] = json!([chat, recovery]);
    document["otherwise"] = json!([call("finish", json!({"form":"report","text":"no script","fields":{}}))]);
    document["pace"]["reply_ms"] = 0.into();
    let document = temper_fake::llm::Document::decode(&serde_json::to_vec(&document).expect("checked fixture"))
        .expect("checked fixture");
    temper_fake::llm::peer(temper_fake::llm::configuration(&document, fixtures::LLM_TOKEN, 17))
        .expect("checked fixture")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Setup,
    Review,
    Cancel,
    Close,
    Recovery,
    RecoveryClose,
    Done,
}
#[expect(clippy::struct_excessive_bools, reason = "the scenario records setup, exercise and teardown facts separately")]
struct Judge {
    controls: skein_world::Controls,
    store: Rc<RefCell<Store>>,
    warm: bool,
    goal: bool,
    exercise: bool,
    stage: Stage,
    answered: usize,
    handed: bool,
    stopping: bool,
    review: Option<u64>,
    change: Option<u64>,
    checked: u64,
    steps: u32,
    wake: Option<Time>,
}
impl Judge {
    fn new(controls: skein_world::Controls, store: Rc<RefCell<Store>>, warm: bool, goal: bool, exercise: bool) -> Self {
        Self {
            controls,
            store,
            warm,
            goal,
            exercise,
            stage: Stage::Setup,
            answered: 0,
            handed: false,
            stopping: false,
            review: None,
            change: None,
            checked: 0,
            steps: 0,
            wake: None,
        }
    }

    fn command(caller: &mut Caller, words: &[&str]) {
        caller.commands.push_back(Command::new(&words.iter().map(|word| word.as_bytes()).collect::<Vec<_>>()));
    }
    fn setup(&mut self, service: &mut world::ServiceHost, caller: &mut Caller) {
        if self.handed {
            return;
        }
        let Some(link) = service.service.links().pop() else {
            return;
        };
        if self.warm {
            Self::command(caller, &["sign-out"]);
        }
        let mut sign_in = Command::new(&[b"sign-in", b"-"]);
        sign_in.input = link;
        caller.commands.push_back(sign_in);
        if !self.warm {
            for words in [
                vec!["account", "add-key", "1", "/setup/llm.key", "--deployment", "--key", "delivery-account"],
                vec![
                    "forge",
                    "connect",
                    "forgejo-admin",
                    "/setup/admin.token",
                    "--api",
                    "127.0.0.1:3000",
                    "--git",
                    "http://127.0.0.1:3000",
                    "--key",
                    "delivery-forge",
                ],
                vec!["forge", "list", "--until", "connected:1"],
                vec![
                    "repository",
                    "adopt",
                    "--project",
                    "1",
                    "--owner",
                    "acme",
                    "--name",
                    "app",
                    "--landing",
                    "main",
                    "--key",
                    "delivery-adopt",
                ],
            ] {
                Self::command(caller, &words);
            }
        }
        self.handed = true;
    }
    fn tasks(&self) -> Vec<jig_domain_tasks::TaskRecord> {
        self.store
            .borrow()
            .rows
            .values()
            .filter_map(|row| match row {
                temper_engine_domain::Record::Core(jig_domain_core::Record::Tasks(
                    jig_domain_tasks::Stored::Live(task) | jig_domain_tasks::Stored::Ended(task),
                )) => Some(task.as_ref().clone()),
                _ => None,
            })
            .collect()
    }
    fn review(&mut self, caller: &mut Caller) {
        let tasks = self.tasks();
        let Some(review) = tasks.iter().find(|task| task.executor == jig_domain_tasks::Executor::Agent { charter: 4 })
        else {
            return;
        };
        if self.goal {
            assert!(
                tasks.iter().any(|task| task.label == 2 && task.tracked.is_some()),
                "a tracked goal delegates the change"
            );
        }
        self.review = Some(review.number);
        assert_eq!(review.tries.invalid, 0, "an assignment refused before its first turn has no result");
        if review.tries.permanent == 0 {
            return;
        }
        assert!(
            matches!(
                review.phase,
                jig_domain_tasks::Phase::Active(_)
                    | jig_domain_tasks::Phase::Held { .. }
                    | jig_domain_tasks::Phase::Ended(_)
            ),
            "review starts or reaches terminal refusal within its bound: {:?}",
            review.phase
        );
        assert_eq!(review.run_reserved, 0, "refusal releases the reservation");
        assert!(
            !self.store.borrow().agent_records.iter().any(|record| matches!(record,
            referee::AgentRecord::Start { task, .. } if *task == review.number)),
            "unencodable review starts no process"
        );
        let change = tasks
            .iter()
            .find(|task| matches!(task.executor, jig_domain_tasks::Executor::Procedure { .. }))
            .expect("change");
        self.change = Some(change.number);
        Self::command(
            caller,
            &["task", "cancel", &change.number.to_string(), "Regression cancellation", "--key", "delivery-cancel"],
        );
        self.stage = Stage::Cancel;
    }
    fn can_progress(&self) -> bool {
        match self.stage {
            Stage::Cancel => {
                let tasks = self.tasks();
                [self.change.expect("checked fixture"), self.review.expect("checked fixture")].iter().all(|number| {
                    tasks
                        .iter()
                        .any(|task| task.number == *number && matches!(task.phase, jig_domain_tasks::Phase::Ended(_)))
                })
            }
            Stage::Close | Stage::RecoveryClose => {
                self.tasks().iter().all(|task| matches!(task.phase, jig_domain_tasks::Phase::Ended(_)))
            }
            Stage::Recovery => self.tasks().iter().any(|task| {
                task.spec.words.as_ref() == b"@recovery"
                    && task.ever_turned
                    && matches!(task.phase, jig_domain_tasks::Phase::Active(jig_domain_tasks::Active::Idle))
            }),
            Stage::Setup | Stage::Review | Stage::Done => true,
        }
    }
    fn progress(&mut self, caller: &mut Caller) {
        if !self.can_progress() {
            return;
        }
        match self.stage {
            Stage::Setup => {
                let expected = if self.warm { 2 } else { 5 };
                if self.answered != expected {
                    return;
                }
                if self.exercise {
                    Self::command(
                        caller,
                        &["chat", "start", "--project", "1", "--key", "delivery-chat", "@delivery-chat"],
                    );
                    self.stage = Stage::Review;
                } else {
                    self.stage = Stage::Done;
                }
            }
            Stage::Review => self.review(caller),
            Stage::Cancel => {
                let tasks = self.tasks();
                let chat = tasks.iter().find(|task| task.flags.conversation).expect("chat");
                Self::command(caller, &["chat", "close", &chat.number.to_string(), "--key", "delivery-close"]);
                self.stage = Stage::Close;
            }
            Stage::Close => {
                Self::command(caller, &["chat", "start", "--project", "1", "--key", "delivery-recovery", "@recovery"]);
                self.stage = Stage::Recovery;
            }
            Stage::Recovery => self.recovery(caller),
            Stage::RecoveryClose => {
                self.stage = Stage::Done;
            }
            Stage::Done => {}
        }
    }
    fn recovery(&mut self, caller: &mut Caller) {
        let tasks = self.tasks();
        let task = tasks.iter().find(|task| task.spec.words.as_ref() == b"@recovery").expect("ready recovery");
        Self::command(caller, &["chat", "close", &task.number.to_string(), "--key", "recovery-close"]);
        self.stage = Stage::RecoveryClose;
    }
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Proc]) {
        self.wake = None;
        self.steps += 1;
        assert!(self.steps < STEPS, "stage {:?}, {} answers", self.stage, self.answered);
        let (first, others) = procs.split_at_mut(1);
        let Proc::Service(service) = &mut first[0] else { panic!("service first") };
        let Proc::Other(HarnessProcess::Calls(caller)) = &mut others[0] else { panic!("caller second") };
        self.setup(service, caller);
        while self.answered < caller.answers.len() {
            let answer = &caller.answers[self.answered];
            assert_eq!(
                answer.exit,
                Exit::Code(0),
                "ctl: {} {}",
                String::from_utf8_lossy(&answer.output),
                String::from_utf8_lossy(&answer.error)
            );
            self.answered += 1;
        }
        self.progress(caller);
        if self.stage == Stage::Done && !self.stopping {
            caller.closed = true;
            self.controls.signal(0, skein_io::kernel::ServiceSignal::Terminate);
            for proc in others {
                match proc {
                    Proc::Peer(peer) => peer.shutdown(),
                    Proc::Forge(peer) => peer.shutdown(),
                    _ => {}
                }
            }
            self.stopping = true;
        }
    }
    fn observe(&mut self, _now: Time, procs: &[Proc]) {
        let applied = self.store.borrow().applied;
        if self.checked == applied {
            return;
        }
        self.checked = applied;
        let Proc::Service(service) = &procs[0] else { panic!("service") };
        referee::Referee::check_roots(&service.service);
    }
    fn idle(&mut self, now: Time, _procs: &[Proc], _settled: bool) {
        // Referee::next_deadline includes when the judge will act, even if
        // the product has no work left to wake the world (testing-strategy.md, 7).
        self.wake = (!self.stopping
            && matches!(self.stage, Stage::Cancel | Stage::Close | Stage::Recovery | Stage::RecoveryClose)
            && self.can_progress())
        .then_some(now);
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.stopping).then(|| self.wake.map_or(DEADLINE, |wake| wake.min(DEADLINE)))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.stopping && now >= DEADLINE).then(|| {
            format!(
                "stage {:?}; {} answers; agent starts {:?}; agent answers {:?}; tasks {:?}",
                self.stage,
                self.answered,
                self.store
                    .borrow()
                    .agent_records
                    .iter()
                    .filter_map(|record| match record {
                        referee::AgentRecord::Start { task, attempt, .. } => Some((*task, *attempt)),
                        referee::AgentRecord::Answer { .. } => None,
                    })
                    .collect::<Vec<_>>(),
                self.store
                    .borrow()
                    .agent_records
                    .iter()
                    .filter_map(|record| match record {
                        referee::AgentRecord::Answer { task, attempt, .. } => Some((*task, *attempt)),
                        referee::AgentRecord::Start { .. } => None,
                    })
                    .collect::<Vec<_>>(),
                self.tasks()
                    .iter()
                    .map(|task| (
                        task.number,
                        task.executor,
                        &task.phase,
                        task.ever_turned,
                        task.run_reserved,
                        &task.retry_notice
                    ))
                    .collect::<Vec<_>>()
            )
        })
    }
    fn passed(&self) -> bool {
        self.stopping
    }
}

fn phase(
    warm: bool,
    exercise: bool,
    goal: bool,
    mut disk: skein_fake_machine::Machine,
    store: Rc<RefCell<Store>>,
    transport: &temper_fake_git_hosted::Typed,
) -> skein_fake_machine::Machine {
    world::lower_store(Some(store.clone()));
    let client = disk.reopen_root(2);
    let machine = Machine::new(disk);
    let parameters = world::parameters(&machine, transport.clone());
    let outcome = programs::with(parameters, || {
        let mut simulation = skein_world::World::new_controlled(
            fixtures::SEED,
            skein_sim::Config::calm(),
            skein_world::Memory::Unchecked,
            |controls| Judge::new(controls, store, warm, goal, exercise),
        )
        .with_machine(machine);
        for (mut program, roots) in programs::programs::<OneProcess>() {
            program.instances = temper_playground::documents::AGENTS + 1;
            simulation.host_roots(program, roots);
        }
        simulation.host_roots(
            crate::ctl::program(|spawn, inherited| {
                Proc::Other(HarnessProcess::Ctl(Box::new(crate::ctl::host(spawn, inherited))))
            }),
            crate::ctl::roots,
        );
        simulation.spawn_restartable(programs::service_roots(), programs::service::<OneProcess>);
        simulation.spawn_root(skein_sim::Handle::new(client.raw()), |root| {
            Proc::Other(HarnessProcess::Calls(Box::new(Caller::scripted(root))))
        });
        simulation.spawn(|| Proc::Peer(Box::new(SignalHost::new(provider(goal), None))));
        simulation.spawn(|| {
            Proc::Forge(Box::new(SignalHost::new(
                temper_playground::fakes::forge_peer(transport, fixtures::SEED),
                None,
            )))
        });
        simulation.run()
    });
    assert!(outcome.procs.iter().all(Host::is_empty), "every process drains");
    world::lower_store(None);
    std::mem::replace(&mut outcome.machine.0.borrow_mut(), temper_playground::machine::fresh())
}

/// Exercise a fresh goal or a change after two restarts, refuse its review, cancel,
/// close the conversation and prove another run can use the host.
pub fn walk(restarts: u32) {
    ACTIVE.with(|active| active.set(true));
    stories::set(Story::Adopt);
    stories::setup_settings(world::Phase::Restart);
    let transport = stories::forge(Story::Adopt);
    let store = Rc::new(RefCell::new(Store::new()));
    let mut disk = temper_playground::machine::fresh();
    temper_playground::machine::setup(&mut disk);
    for start in 0..restarts {
        disk = phase(start > 0, false, false, disk, store.clone(), &transport);
    }
    let _disk = phase(restarts > 0, true, restarts == 0, disk, store, &transport);
    ACTIVE.with(|active| active.set(false));
}
