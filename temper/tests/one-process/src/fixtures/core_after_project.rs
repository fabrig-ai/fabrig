//! No core wait: the gate is open and the root ready; a two-membership client
//! rejects the mandatory `me` after project three (temper/docs/design/client/ctl/README.md, 9;
//! jig/docs/design/client/domain/contract/documents.md, 1), and meets those memberships again at restart.
use crate::{
    ctl::{Caller, Command},
    fixtures,
    referee::Store,
    stories,
    world::{self, HarnessProcess, Person, ServiceObserver, Story},
};
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use serde_json::{Value, json};
use skein_io::kernel::{Exit, ServiceSignal};
use skein_lib::Time;
use skein_shell::Host;
use std::{cell::RefCell, rc::Rc};
use temper_engine_domain as root;
use temper_playground::{
    machine::{Machine, SignalHost},
    programs::{self, Harness},
    service::{Observe, ServiceHost},
};

// Each native call keeps the shipped client's thirty-second deadline.
const ANSWER_BOUND: skein_lib::Duration = temper_client_shell_cli::limits::DEADLINE;
const LOGIN: &str = "personal";
const PROJECT: &str = "proof-5-clearer-words";
const ITEMS: usize = 10; // Rules, five charters, two templates and two policies.

struct Incident;
type Proc = programs::Process<Incident>;
impl Harness for Incident {
    type Service = ServiceObserver;
    type Process = HarnessProcess;
    fn seeds(_: u64) -> temper_service::settings::Seeds {
        fixtures::seeds()
    }
    fn files() -> temper_service::settings::Files {
        fixtures::files()
    }
    fn configure(value: &mut Value) {
        value["world_settings"] = fixtures::world_document()["world_settings"].clone();
        for (name, count) in [
            ("people.projects", 4),
            ("authority.projects", 3),
            ("people.holdings", 8),
            ("tasks.tasks", 12),
            ("tasks.project_tasks", 12),
            ("tasks.tree_tasks", 12),
            ("tasks.funders", 16),
            ("tasks.delegates", 8),
            ("people.inbox_tasks", 24),
            ("people.operations", 12),
            ("people.sign_ins", 8),
            ("tasks.spec_bytes", 512),
            ("authority.grants", 16),
            ("notes.scopes", 32),
            ("tasks.authority_grants", 16),
            ("tasks.authority_bytes", 512),
            ("tasks.executor_kinds", 12),
            ("core.policy_bytes", 40_000),
            ("core.run_bytes", 45_000),
            ("hub.charter_bytes", 65_536),
        ] {
            value["engine"]["limits"]["overrides"][name] = count.into();
        }
        value["workers"][0]["limits"]["overrides"]["hub.charter_bytes"] = 65_536.into();
        value["application"]["limits"]["overrides"]["root.journal.run_bytes"] = 45_000.into();
        value["application"]["limits"]["overrides"]["forge.judge_projects"] = 3.into();
        value["engine"]["llm"]["providers"][0]["sources"]["programs"] = json!(["claude_code"]);
        value["engine"]["llm"]["providers"][0]["manage_usage"] = "https://provider.example/usage".into();
        value["engine"]["acts"] = json!([{"connector":1,"may":["adopt"]}]);
        let grant = |kind| json!({"connector":1,"kind":kind,"pattern":{"segments":["forge","127.0.0.1:3000"],"last":"open","value":"acme"}});
        let tools = 3 | if crate::across_versions::active() { core::ToolFamilies::JIG.read.0 } else { 0 };
        let authority = json!({"tools":tools,"grants":[grant(3),grant(4),grant(7),grant(8),grant(9),grant(10),grant(11),grant(12)],"delegation":{"kinds":[{"kind":"charter","number":1},{"kind":"charter","number":4},{"kind":"procedure","number":2}],"tasks":8,"depth":5},"budget":{"spend":500,"deadline_unix_seconds":null},"notes":0,"note_resources":[]});
        let policy = &mut value["world_settings"]["policies"][0]["policy"];
        policy["ceiling"] = authority.clone();
        policy["roles"][0]["authority"] = authority;
        policy["period_spend"] = 1000.into();
        policy["roles"][0]["period_spend"] = 1000.into();
        policy["projections"] =
            json!([grant(3), grant(4), grant(7), grant(8), grant(9), grant(10), grant(11), grant(12)]);
    }
    fn given(document: &Value) -> temper_service::stock::Given {
        let mut given = temper_playground::documents::given_with(document, false, false);
        let temper_service::stock::Given::Own { jig, .. } = &mut given else { panic!("world defaults") };
        let mut policies = skein_lib::Map::with_capacity(2);
        let policy = jig.policies.get(&1).expect("world policy").clone();
        policies.insert(1, policy.clone()).expect("policy one");
        policies.insert(2, policy).expect("policy two");
        jig.policies = policies;
        given
    }
}

thread_local! { static PARAMETERS: RefCell<Option<programs::Parameters>> = const { RefCell::new(None) }; }
fn service(inherited: &skein_world::Inherited) -> Proc {
    let parameters = PARAMETERS.with(|parameters| parameters.borrow().as_ref().expect("incident parameters").clone());
    let root = inherited.roots.iter().find(|(name, _)| name.as_ref() == b"state").expect("state root").1;
    let worker = inherited.roots.iter().find(|(name, _)| name.as_ref() == b"worker").expect("worker root").1;
    let mut document: Value = serde_json::from_slice(&parameters.deployment).expect("deployment JSON");
    Incident::configure(&mut document);
    let given = Incident::given(&document);
    let parsed = temper_playground::documents::settings::configuration(&document);
    let codec = jig_protocol_store::codec::limits(&parsed.limits.core).expect("incident record bounds");
    let families = temper_protocol_store::bounds(
        &temper_service::limits::profile(temper_service::limits::Profile::Small).forge,
        parsed.limits.core.tasks.spec_bytes,
    )
    .expect("incident forge bounds");
    temper_playground::documents::settings::strip_models(&mut document);
    document.as_object_mut().expect("deployment object").remove("world_settings");
    let mut agent = temper_playground::documents::agent();
    agent.programs = Box::new([temper_service::settings::Program {
        adapter: temper_service::settings::Adapter::ClaudeCode,
        path: b"/usr/bin/claude".as_slice().into(),
        directory: b"/worker/programs".as_slice().into(),
    }]);
    let mut service = temper_service::startup::with_setup_given(
        &serde_json::to_vec(&document).expect("deployment bytes"),
        &jig_protocol_config::Directories::working(),
        &Incident::files(),
        Some(&agent),
        &Incident::seeds(parameters.seed),
        None,
        Some(temper_service::WorkerRuntime {
            wires: crate::across_versions::worker_wires(),
            root: worker,
            path: b"/worker".as_slice().into(),
            agent: temper_playground::documents::pinned_agent(),
        }),
        temper_playground::documents::forges(),
        None,
        None,
        None,
        &given,
        Box::new([]),
    )
    .unwrap_or_else(|problems| panic!("{}", String::from_utf8_lossy(&temper_service::startup::lines(&problems))));
    let parent =
        inherited.roots.iter().find(|(name, _)| name.as_ref() == b"programs_parent").expect("programs parent").1;
    let resources = temper_service::program_sources::configured(&agent.programs[0], Box::new([]), 1000, Some(parent))
        .expect("synthetic program resources");
    service.configure_program_sources(resources).expect("worker program source");
    service.adopt_store(root, 1000).expect("same service store");
    service.adopt_signals(inherited.signal).expect("service signal");
    Proc::Service(Box::new(ServiceHost::new(
        service,
        ServiceObserver::started(root, inherited).with_codec(&codec, &families),
        parameters,
    )))
}

fn call(name: &str, arguments: &Value) -> Value {
    json!({"lines":[{"call":name,"arguments":arguments}],"finish":"tool_calls","tokens":1})
}
fn provider() -> skein_fake_peers::llm::Peer {
    let mut value: Value = serde_json::from_slice(temper_playground::stories::A.llm).expect("fixture provider");
    value["cued"] = json!([
        {"cue":"@incident-goal","turns":[
            call("change", &json!({"words":"@incident-edit"})), call("wait", &json!({})),
            call("finish", &json!({"form":"report","text":"Done: earlier goal ended","fields":{}}))]},
        {"cue":"@incident-edit","turns":[
            call("shell", &json!({"command":"echo fixed > conflict.txt"})),
            call("finish", &json!({"form":"change","text":"","fields":{"title":"Earlier change","body":"Fixed"}}))]},
        {"cue":"Review this change at the named head","turns":[
            call("finish", &json!({"form":"verdict","label":"1","text":"approved","fields":{},"items":[]}))]}
    ]);
    if crate::across_versions::active() {
        value["cued"][1]["turns"]
            .as_array_mut()
            .expect("change turns")
            .insert(0, call("read_task", &json!({"task":3})));
    }
    value["asked"] = json!([]);
    value["otherwise"] = json!([call("finish", &json!({"form":"report","text":"Done","fields":{}}))]);
    value["pace"]["reply_ms"] = 0.into();
    let document = temper_fake::llm::Document::decode(&serde_json::to_vec(&value).expect("provider bytes"))
        .expect("provider document");
    let configuration = temper_fake::llm::configuration(&document, fixtures::LLM_TOKEN, 17);
    if crate::across_versions::active() {
        temper_fake::llm::recording_peer(configuration).expect("recorded provider")
    } else {
        temper_fake::llm::peer(configuration).expect("provider")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Setup,
    Goal,
    PriorRevisions,
    Forget,
    ReadSettings,
    Apply,
    Declare,
    Make,
    Calls,
    Native,
    Done,
}
#[derive(Default, Debug)]
pub struct Outcome {
    pub calls: usize,
    pub fresh: usize,
    pub native: bool,
    pub legacy_rejected: bool,
    pub gate: String,
}
#[expect(clippy::struct_excessive_bools, reason = "the story records each independent client and shutdown handoff")]
struct Judge {
    controls: skein_world::Controls,
    store: Rc<RefCell<Store>>,
    disk: Rc<RefCell<skein_fake_machine::Machine>>,
    seen: Rc<RefCell<Outcome>>,
    warm: bool,
    whole: bool,
    legacy: bool,
    stage: Stage,
    target: usize,
    answered: usize,
    handed: bool,
    messaged: Option<(u64, u64)>,
    stopping: bool,
    first_link: Box<[u8]>,
    fresh_at: usize,
    fresh_handed: bool,
    native_started: bool,
    readings: usize,
    deadline: Time,
}
impl Judge {
    fn command(caller: &mut Caller, words: &[&str]) {
        caller.commands.push_back(Command::new(&words.iter().map(|word| word.as_bytes()).collect::<Vec<_>>()));
    }
    fn queue(&mut self, caller: &mut Caller, commands: &[&[&str]], next: Stage) {
        self.target = caller.answers.len() + commands.len();
        for words in commands {
            Self::command(caller, words);
        }
        self.stage = next;
    }
    fn sign_in(caller: &mut Caller, link: &[u8]) {
        let mut command = Command::new(&[b"sign-in", b"-"]);
        command.input = link.into();
        caller.commands.push_back(command);
    }
    fn calls(&mut self, caller: &mut Caller) {
        let project = "3";
        self.queue(
            caller,
            &[&["project", "show", project], &["me"], &["account", "list"], &["settings", "show"]],
            Stage::Calls,
        );
    }
    fn after_prior(&mut self, caller: &mut Caller) {
        let account = self.account();
        self.queue(caller, &[&["account", "forget", &account, "--key", "forget-program"]], Stage::Forget);
    }
    fn after_forget(&mut self, caller: &mut Caller) {
        self.readings = caller.answers.len();
        let mut commands = vec![vec!["settings", "show", "rules", "--json"]];
        for number in ["1", "2", "4", "9", "10"] {
            commands.push(vec!["settings", "show", "charter", number, "--json"]);
        }
        for kind in ["template", "policy"] {
            for number in ["1", "2"] {
                commands.push(vec!["settings", "show", kind, number, "--json"]);
            }
        }
        self.target = caller.answers.len() + commands.len();
        for words in commands {
            Self::command(caller, &words);
        }
        self.stage = Stage::ReadSettings;
    }
    fn after_apply(&mut self, caller: &mut Caller) {
        self.queue(
            caller,
            &[&["account", "declare", "1", "claude_code", LOGIN, "--key", "recovered-declaration"]],
            Stage::Declare,
        );
    }
    fn make(&mut self, caller: &mut Caller) {
        let mut commands = Vec::new();
        if self.whole {
            commands.push(["account", "list"].as_slice());
        }
        commands.push(["project", "make", PROJECT, "--key", "incident-project"].as_slice());
        self.queue(caller, &commands, Stage::Make);
    }
    fn account(&self) -> String {
        self.store
            .borrow()
            .rows
            .values()
            .find_map(|row| match row {
                root::Record::Core(core::Record::Accounts(jig_domain_accounts::Stored::Source {
                    account,
                    name,
                    ..
                })) if name.as_ref() == LOGIN.as_bytes() => Some(account.to_string()),
                _ => None,
            })
            .expect("declared program account")
    }
    fn ended_goal(&self) -> bool {
        self.store.borrow().rows.values().any(|row| matches!(row, root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(task))) if task.tracked.is_some() && matches!(task.phase, tasks::Phase::Ended(tasks::Ending::Done(_)))))
    }
    fn check_answers(&mut self, caller: &Caller, now: Time) {
        while self.answered < caller.answers.len() {
            let answer = &caller.answers[self.answered];
            assert_eq!(
                answer.exit,
                Exit::Code(0),
                "stage {:?}, call {}: {} {}",
                self.stage,
                self.answered,
                String::from_utf8_lossy(&answer.output),
                String::from_utf8_lossy(&answer.error)
            );
            self.answered += 1;
            self.deadline = now.saturating_add(ANSWER_BOUND);
        }
    }
    fn prepare_settings(&mut self, caller: &Caller) {
        let mut items = Vec::new();
        for answer in caller.answers.iter().skip(self.readings).take(ITEMS) {
            let value: Value = serde_json::from_slice(&answer.output).expect("whole setting JSON");
            items.push(json!({"setting":value["setting"],"set":value["value"]}));
        }
        assert_eq!(items.len(), ITEMS, "all incident settings read");
        for item in &mut items {
            if item["setting"]["kind"] == "charter" && item["setting"]["number"] == 1 {
                item["set"]["instructions"] = "Complete the earlier work carefully.".into();
            }
            if item["setting"]["kind"] == "template" && item["setting"]["number"] == 1 {
                item["set"]["name"]["bytes"] = "Conversation".into();
            }
        }
        let bytes = serde_json::to_vec(&json!({"items":items})).expect("incident settings file");
        temper_playground::machine::lay_handover(&mut self.disk.borrow_mut(), b"/setup/incident.json", &bytes);
    }
}
impl skein_world::Referee<Proc> for Judge {
    #[expect(clippy::too_many_lines, reason = "the referee keeps the incident acts in their required order")]
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        let (services, rest) = procs.split_at_mut(1);
        let Proc::Service(service) = &mut services[0] else { panic!("service first") };
        let (callers, rest) = rest.split_at_mut(1);
        let Proc::Other(HarnessProcess::Calls(caller)) = &mut callers[0] else { panic!("caller second") };
        let (people, rest) = rest.split_at_mut(1);
        let Proc::Other(HarnessProcess::Client(person)) = &mut people[0] else { panic!("native person third") };
        let (fresh_callers, peers) = rest.split_at_mut(1);
        let Proc::Other(HarnessProcess::Calls(fresh)) = &mut fresh_callers[0] else { panic!("fresh caller fourth") };
        if !self.handed
            && let Some(link) = service.service.links().pop()
        {
            self.first_link.clone_from(&link);
            if self.warm {
                self.calls(caller);
            } else {
                Self::sign_in(caller, &link);
                if self.whole {
                    let commands = [
                        vec!["account", "add-key", "1", "/setup/llm.key", "--deployment", "--key", "incident-key"],
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
                            "incident-forge",
                        ],
                        vec!["forge", "list", "--until", "connected:1"],
                        vec!["project", "make", "Earlier work", "--key", "earlier-project"],
                        vec!["account", "declare", "1", "claude_code", LOGIN, "--key", "program-account"],
                        vec![
                            "repository",
                            "adopt",
                            "--project",
                            "2",
                            "--owner",
                            "acme",
                            "--name",
                            "app",
                            "--landing",
                            "main",
                            "--key",
                            "incident-adoption",
                        ],
                        vec![
                            "goal",
                            "start",
                            "--project",
                            "2",
                            "--charter",
                            "1",
                            "--as-allowed",
                            "--deadline",
                            "2030-01-01T00:00Z",
                            "@incident-goal",
                            "--key",
                            "earlier-goal",
                        ],
                    ];
                    for words in &commands {
                        if crate::across_versions::active() && words.starts_with(&["account", "declare"]) {
                            continue;
                        }
                        Self::command(caller, words);
                    }
                    Self::command(caller, &["settings", "show", "charter", "1", "--json"]);
                    self.target = caller.commands.len();
                    self.stage = Stage::Goal;
                } else {
                    Self::command(caller, &["project", "make", "Earlier work", "--key", "earlier-project"]);
                    self.target = 2;
                    self.stage = Stage::Goal;
                }
            }
            self.handed = true;
        }
        if self.stage == Stage::Make {
            let domain = service.service.engine_domain().expect("engine root");
            self.seen.borrow_mut().gate = format!(
                "ready={}, start={:?}, projects={}; {:?}",
                domain.ready(),
                domain.start_phase(),
                domain.deployment().projects,
                service.service.pending(now)
            ) + &caller.pending_answer();
        }
        if crate::across_versions::active() && self.messaged.is_none() && self.stage == Stage::Goal {
            let running = self.store.borrow().rows.values().find_map(|row| match row {
                root::Record::Core(core::Record::Tasks(tasks::Stored::Live(task)))
                    if task.spec.words.windows(b"@incident-edit".len()).any(|part| part == b"@incident-edit")
                        && matches!(task.phase, tasks::Phase::Active(tasks::Active::Running { .. })) =>
                {
                    Some((task.number, task.attempt))
                }
                _ => None,
            });
            if let Some((task, attempt)) = running {
                Self::command(
                    caller,
                    &[
                        "task",
                        "message",
                        &task.to_string(),
                        crate::across_versions::MESSAGE,
                        "--key",
                        "bridged-message",
                    ],
                );
                self.target += 1;
                self.messaged = Some((task, attempt));
            }
        }
        self.check_answers(caller, now);
        if self.handed && caller.answers.len() >= self.target {
            match self.stage {
                Stage::Goal if !self.whole || self.ended_goal() => {
                    if crate::across_versions::active() {
                        let (number, attempt) = self.messaged.expect("a message was sent to the goal's running change");
                        assert!(self.store.borrow().rows.values().any(|row| matches!(row,
                            root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(task)))
                            if task.number == number && task.attempt == attempt && matches!(task.phase, tasks::Phase::Ended(tasks::Ending::Done(_)))
                        )), "the engine read that running attempt's successful result");
                        bridged_readback(peers);
                        crate::across_versions::tasks(&self.store.borrow());
                        assert!(self.store.borrow().rows.values().any(|row| matches!(row, root::Record::Forge { row } if matches!(row.as_ref(), temper_engine_domain_forge::Stored::Change(row) if matches!(row.change.state, temper_engine_domain_forge_change::State::Landed { .. })))), "the bridged goal's change landed");
                        self.stage = Stage::Done;
                    } else if self.whole {
                        let saved: Value =
                            serde_json::from_slice(&caller.answers.last().expect("initial charter answer").output)
                                .expect("initial charter");
                        for (file, instructions) in [
                            (b"/setup/prior-two.json".as_slice(), "Complete the earlier work."),
                            (b"/setup/prior-three.json".as_slice(), "Complete the earlier work fully."),
                        ] {
                            let mut charter = saved["value"].clone();
                            charter["instructions"] = instructions.into();
                            let prior = serde_json::to_vec(
                                &json!({"items":[{"setting":{"kind":"charter","number":1},"set":charter}]}),
                            )
                            .expect("prior settings");
                            temper_playground::machine::lay_handover(&mut self.disk.borrow_mut(), file, &prior);
                        }
                        assert!(self.store.borrow().rows.values().any(|row| matches!(row, root::Record::Forge { row } if matches!(row.as_ref(), temper_engine_domain_forge::Stored::Change(row) if matches!(row.change.state, temper_engine_domain_forge_change::State::Landed { .. })))), "earlier change landed");
                        self.queue(
                            caller,
                            &[
                                &[
                                    "settings",
                                    "apply",
                                    "/setup/prior-two.json",
                                    "--against",
                                    "1",
                                    "--key",
                                    "revision-two",
                                ],
                                &[
                                    "settings",
                                    "apply",
                                    "/setup/prior-three.json",
                                    "--against",
                                    "2",
                                    "--key",
                                    "revision-three",
                                ],
                            ],
                            Stage::PriorRevisions,
                        );
                    } else {
                        self.make(caller);
                    }
                }
                Stage::PriorRevisions => self.after_prior(caller),
                Stage::Forget => {
                    let forgotten = caller.answers.last().expect("forget terminal");
                    assert!(
                        String::from_utf8_lossy(&forgotten.output).contains("Signed out on every machine"),
                        "program forget: {} {}",
                        String::from_utf8_lossy(&forgotten.output),
                        String::from_utf8_lossy(&forgotten.error)
                    );
                    self.after_forget(caller);
                }
                Stage::ReadSettings => {
                    self.prepare_settings(caller);
                    self.queue(
                        caller,
                        &[&["settings", "apply", "/setup/incident.json", "--against", "3", "--key", "revision-four"]],
                        Stage::Apply,
                    );
                }
                Stage::Apply => self.after_apply(caller),
                Stage::Declare => self.make(caller),
                Stage::Make => {
                    let domain = service.service.engine_domain().expect("engine root");
                    assert!(
                        domain.ready(),
                        "gate after act 6: ready={}, start={:?}, root={domain:?}",
                        domain.ready(),
                        domain.start_phase()
                    );
                    self.seen.borrow_mut().gate = format!(
                        "ready={}, start={:?}; {:?}",
                        domain.ready(),
                        domain.start_phase(),
                        service.service.pending(now)
                    );
                    self.calls(caller);
                }
                Stage::Calls => {
                    let domain = service.service.engine_domain().expect("engine root");
                    assert!(domain.ready(), "root ready after answering the calls");
                    self.seen.borrow_mut().gate = format!(
                        "ready={}, start={:?}; {:?}",
                        domain.ready(),
                        domain.start_phase(),
                        service.service.pending(now)
                    );
                    self.seen.borrow_mut().calls = 4;
                    if !self.fresh_handed {
                        Self::sign_in(fresh, &self.first_link);
                        self.fresh_handed = true;
                    }
                    eprintln!("core calls answered: {}; gate {}", self.seen.borrow().calls, self.seen.borrow().gate);
                    self.stage = Stage::Native;
                }
                Stage::Native => {
                    if !fresh.answers.is_empty() && self.fresh_at == 0 {
                        if self.warm {
                            self.fresh_at = 1;
                            self.deadline = now.saturating_add(ANSWER_BOUND);
                            Self::command(fresh, &["me"]);
                        } else if let Some(link) = service.service.links().pop() {
                            Self::sign_in(fresh, &link);
                            self.fresh_at = 1;
                            self.deadline = now.saturating_add(ANSWER_BOUND);
                        }
                    }
                    let expected = 2;
                    if fresh.answers.len() >= expected {
                        let last = fresh.answers.last().expect("fresh sign-in answer");
                        assert_eq!(
                            last.exit,
                            Exit::Code(0),
                            "fresh sign-in: {} {}",
                            String::from_utf8_lossy(&last.output),
                            String::from_utf8_lossy(&last.error)
                        );
                        self.seen.borrow_mut().fresh = 1;
                        if !self.native_started {
                            if self.legacy {
                                let mut limits = stories::client_limits();
                                limits.client.client.domain.projects = 2;
                                person.saved_with_limits(&limits);
                            } else {
                                person.saved();
                            }
                            self.native_started = true;
                            self.deadline = now.saturating_add(ANSWER_BOUND);
                        }
                        if self.legacy && now >= self.deadline {
                            assert!(!person.finished, "two memberships cannot decode the mandatory me");
                            assert!(service.service.engine_domain().expect("engine root").ready());
                            self.seen.borrow_mut().legacy_rejected = true;
                            self.stage = Stage::Done;
                        } else if person.finished {
                            self.seen.borrow_mut().native = true;
                            self.stage = Stage::Done;
                        }
                    }
                }
                Stage::Setup | Stage::Goal | Stage::Done => {}
            }
        }
        if self.stage == Stage::Done && !self.stopping {
            caller.closed = true;
            fresh.closed = true;
            if crate::across_versions::active() {
                person.saved();
            }
            person.finish();
            self.controls.signal(0, ServiceSignal::Terminate);
            for proc in peers {
                match proc {
                    Proc::Peer(peer) => peer.shutdown(),
                    Proc::Forge(peer) => peer.shutdown(),
                    _ => {}
                }
            }
            self.stopping = true;
        }
        self.store.borrow_mut().pending.clear();
    }
    fn observe(&mut self, _: Time, _: &[Proc]) {}
    fn next_deadline(&self) -> Option<Time> {
        (!self.stopping).then_some(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.stopping && now >= self.deadline).then(|| {
            format!(
                "incident stage {:?}, answers {}/{}, fresh {}, store rows {}, gate {}",
                self.stage,
                self.answered,
                self.target,
                self.fresh_at,
                self.store.borrow().rows.len(),
                self.seen.borrow().gate,
            )
        })
    }
    fn passed(&self) -> bool {
        self.stopping
    }
}

#[expect(clippy::too_many_lines, reason = "the world composes its service, clients and fake peers in one loop")]
fn phase(
    warm: bool,
    whole: bool,
    legacy: bool,
    mut disk: skein_fake_machine::Machine,
    store: Rc<RefCell<Store>>,
    transport: &temper_fake_git_hosted::Typed,
) -> (skein_fake_machine::Machine, Outcome) {
    world::lower_store(Some(store.clone()));
    if warm {
        store.borrow_mut().recovering();
    }
    let caller = disk.reopen_root(2);
    let native = disk.reopen_root(2);
    let fresh = disk.reopen_root(2);
    let machine = Machine::new(disk);
    let seen = Rc::new(RefCell::new(Outcome::default()));
    let parameters = world::parameters(&machine, transport.clone());
    PARAMETERS.with(|at| *at.borrow_mut() = Some(parameters.clone()));
    let outcome = programs::with(parameters, || {
        let mut simulation = skein_world::World::new_controlled(
            fixtures::SEED,
            skein_sim::Config::calm(),
            skein_world::Memory::Unchecked,
            |controls| Judge {
                controls,
                store,
                disk: machine.0.clone(),
                seen: seen.clone(),
                warm,
                whole,
                legacy,
                stage: Stage::Setup,
                target: 0,
                answered: 0,
                handed: false,
                messaged: None,
                stopping: false,
                first_link: Box::new([]),
                fresh_at: 0,
                fresh_handed: false,
                native_started: false,
                readings: 0,
                deadline: Time::ZERO.saturating_add(ANSWER_BOUND),
            },
        )
        .with_machine(machine);
        for (mut program, roots) in programs::programs::<Incident>() {
            program.instances = temper_playground::documents::AGENTS + 1;
            simulation.host_roots(program, roots);
        }
        let mut ctl = crate::ctl::program(|spawn, inherited| {
            Proc::Other(HarnessProcess::Ctl(Box::new(crate::ctl::host(spawn, inherited).capturing())))
        });
        ctl.instances = 2;
        simulation.host_roots(ctl, crate::ctl::roots);
        let mut roots = programs::service_roots();
        roots.push(skein_world::StartupRoot {
            name: b"programs_parent".as_slice().into(),
            path: b"worker".as_slice().into(),
        });
        simulation.spawn_restartable(roots, service);
        let fake = smith_fake_claude::hosted(b"/usr/bin/claude");
        simulation.host_roots(
            skein_world::HostedProgram {
                program: fake.program,
                instances: fake.instances,
                operations: fake.operations,
                make: |spawn, inherited| {
                    Proc::Other(HarnessProcess::ProgramSource(Box::new((smith_fake_claude::hosted(
                        b"/usr/bin/claude",
                    )
                    .make)(spawn, inherited))))
                },
            },
            |_| Vec::new(),
        );
        simulation.spawn_root(skein_sim::Handle::new(caller.raw()), |root| {
            Proc::Other(HarnessProcess::Calls(Box::new(Caller::scripted(root))))
        });
        simulation.spawn_root(skein_sim::Handle::new(native.raw()), |root| {
            let steps = vec![
                skein_fake_person::Step::Go { address: temper_client_domain::Address::Home { project: 3 } },
                skein_fake_person::Step::See {
                    find: skein_fake_person::Find::text(PROJECT.as_bytes()),
                    within: fixtures::PERSON_WAIT,
                },
            ];
            Proc::Other(HarnessProcess::Client(Box::new(SignalHost::new(Person::pages(root, steps.into()), None))))
        });
        simulation.spawn_root(skein_sim::Handle::new(fresh.raw()), |root| {
            Proc::Other(HarnessProcess::Calls(Box::new(Caller::at_directory(
                root,
                if warm { b"/client/restarted-private" } else { b"/client/fresh-private" },
            ))))
        });
        simulation.spawn(|| {
            Proc::Forge(Box::new(SignalHost::new(
                temper_playground::fakes::forge_peer(transport, fixtures::SEED),
                None,
            )))
        });
        simulation.spawn(|| Proc::Peer(Box::new(SignalHost::new(provider(), None))));
        simulation.run()
    });
    assert!(outcome.procs.iter().all(Host::is_empty), "incident world drains every process");
    world::lower_store(None);
    PARAMETERS.with(|at| *at.borrow_mut() = None);
    let disk = std::mem::replace(&mut *outcome.machine.0.borrow_mut(), temper_playground::machine::fresh());
    (disk, Rc::try_unwrap(seen).expect("released referee").into_inner())
}

#[must_use]
pub fn sequence(whole: bool) -> [Outcome; 2] {
    run_sequence(whole, false)
}

/// The unchanged file store created while the shipped client could receive only two memberships.
#[must_use]
pub fn legacy_store() -> [Outcome; 2] {
    run_sequence(false, true)
}

fn run_sequence(whole: bool, legacy: bool) -> [Outcome; 2] {
    stories::set(Story::Adopt);
    stories::setup_settings(world::Phase::Restart);
    let transport = forge();
    let store = Rc::new(RefCell::new(Store::new()));
    let mut disk = temper_playground::machine::fresh();
    temper_playground::machine::setup(&mut disk);
    let (disk, before) = phase(false, whole, legacy, disk, store.clone(), &transport);
    let (_, after) = phase(true, whole, false, disk, store, &transport);
    [before, after]
}

/// Only the model's goal-through-landing phase, without its cold restart.
pub(crate) fn bridged_goal() {
    stories::set(Story::Adopt);
    stories::setup_settings(world::Phase::Restart);
    let transport = forge();
    let store = Rc::new(RefCell::new(Store::new()));
    let mut disk = temper_playground::machine::fresh();
    temper_playground::machine::setup(&mut disk);
    phase(false, true, false, disk, store, &transport);
}

fn bridged_readback(procs: &[Proc]) {
    use skein_fake_llm_domain::api::Part;
    let queries: Vec<_> =
        procs
            .iter()
            .filter_map(|proc| if let Proc::Peer(peer) = proc { Some(peer) } else { None })
            .flat_map(|peer| peer.observations())
            .filter_map(|seen| {
                if let skein_fake_peers::llm::Observation::Query { query, .. } = seen { Some(query) } else { None }
            })
            .collect();
    assert!(
        queries.iter().any(|query| query.messages.iter().flat_map(|message| message.parts.iter()).any(|part| {
            let bytes = match part {
                Part::Text { text } => text,
                Part::Opaque { bytes } => bytes,
                Part::ToolOutput { output, .. } => output,
                Part::ToolCall { .. } => return false,
            };
            bytes
                .windows(crate::across_versions::MESSAGE.len())
                .any(|part| part == crate::across_versions::MESSAGE.as_bytes())
        })),
        "the run read its message across the bridge"
    );
    assert!(queries.iter().any(|query| {
        let parts: Vec<_> = query.messages.iter().flat_map(|message| message.parts.iter()).collect();
        parts.iter().any(|part| if let Part::ToolCall { id, name, .. } = part {
            name.as_ref() == b"read_task" && parts.iter().any(|answer| matches!(answer, Part::ToolOutput { id: answered, is_error: false, output } if answered == id && !output.starts_with(b"Error:")))
        } else { false })
    }), "the run read its host call's answer across the bridge");
}

fn forge() -> temper_fake_git_hosted::Typed {
    use skein_fake_forge_domain::{self as fake, api};
    let mut config = temper_playground::documents::hosted_git::config();
    // Two service calls and the hosted git can be in flight together.
    config.limits.calls = 4;
    config.limits.page_size = 8; // The service requests eight rows per forge page.
    // Forge API timestamps and the service's outbox positions share UTC.
    config.skew = fake::Skew::Ahead(skein_lib::Duration::from_nanos(skein_sim::Config::calm().wall.as_nanos()));
    let mut domain = fake::Domain::new(&config, fixtures::SEED);
    fake::repository(
        &mut domain,
        &config,
        api::Setup {
            id: fixtures::REPOSITORY,
            name: b"acme/app".as_slice().into(),
            default: b"main".as_slice().into(),
            tree: Box::new([api::File {
                path: b"conflict.txt".as_slice().into(),
                content: b"base\n".as_slice().into(),
            }]),
            workflows: Box::new([]),
            labels: Box::new([]),
            protection: None,
            hooked: false,
            checks: api::Checks {
                cues: Box::new([]),
                contexts: Box::new([b"check".as_slice().into()]),
                latency_min: skein_lib::Duration::ZERO,
                latency_max: skein_lib::Duration::ZERO,
                silent: 0,
                passes: 1000,
                reruns: 0,
                cue: None,
            },
        },
    );
    fake::grant(&mut domain, b"acme/app", fixtures::WORKER_USER, api::Permission::Write);
    let forge = Rc::new(RefCell::new(skein_fake_peers::forge::Forge::new(domain, config)));
    let mut transport = temper_fake_git_hosted::Typed::new(forge, &stories::forge_table());
    let identity = temper_playground::documents::hosted_git::identity();
    transport.authenticate(Some(&(identity.login, identity.token)));
    transport
}
