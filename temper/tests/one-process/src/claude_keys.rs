//! Claude's key-only, login-then-key and owner deployment accounts through the shipped ctl
//! (temper/docs/design/draft/claude-api.md, 6).
use crate::{
    ctl::{Caller, Command},
    fixtures,
    referee::Store,
    stories,
    world::{self, HarnessProcess, ServiceObserver, Story},
};
use serde_json::{Value, json};
use skein_io::kernel::{Exit, ServiceSignal};
use skein_lib::{List, Time};
use skein_shell::Host;
use std::{cell::RefCell, rc::Rc};
use temper_playground::{
    machine::Machine,
    programs::{self, Harness},
    service::{Observe, ServiceHost},
};

struct Accounts;
type Proc = programs::Process<Accounts>;
impl Harness for Accounts {
    type Service = ServiceObserver;
    type Process = HarnessProcess;
    fn seeds(_: u64) -> temper_service::settings::Seeds {
        fixtures::seeds()
    }
    fn files() -> temper_service::settings::Files {
        fixtures::files()
    }
    fn configure(value: &mut Value) {
        let llm = &mut value["engine"]["llm"];
        let template: Value = serde_json::from_slice(include_bytes!("../../../crates/temper-init/template/bare.json"))
            .expect("init template");
        let mut provider = template["engine"]["llm"]["providers"][1].clone();
        provider["models"] = json!(["claude-fixture"]);
        llm["providers"].as_array_mut().expect("providers").push(provider);
        let mut model = llm["models"][0].clone();
        model["name"] = "claude-fixture".into();
        model["endpoint"] = "anthropic".into();
        llm["models"].as_array_mut().expect("world models").push(model);
        for name in ["anthropic", "anthropic-api"] {
            let mut endpoint = template["engine"]["llm"]["endpoints"]
                .as_array()
                .expect("init endpoints")
                .iter()
                .find(|endpoint| endpoint["name"] == name)
                .expect("Claude endpoint")
                .clone();
            endpoint["address"] = json!({"socket":"127.0.0.1:3001"});
            endpoint["transport"] = "plaintext".into();
            endpoint.as_object_mut().expect("endpoint").remove("server_name");
            llm["endpoints"].as_array_mut().expect("endpoints").push(endpoint);
        }
    }
    fn given(document: &Value) -> temper_service::stock::Given {
        temper_playground::documents::given_with(document, false, false)
    }
}
thread_local! { static PARAMETERS: RefCell<Option<programs::Parameters>> = const { RefCell::new(None) }; }
fn service(inherited: &skein_world::Inherited) -> Proc {
    let parameters = PARAMETERS.with(|at| at.borrow().as_ref().expect("account parameters").clone());
    let root = inherited.roots.iter().find(|(name, _)| name.as_ref() == b"state").expect("state root").1;
    let worker = inherited.roots.iter().find(|(name, _)| name.as_ref() == b"worker").expect("worker root").1;
    let mut document = fixtures::world_document();
    Accounts::configure(&mut document);
    let given = Accounts::given(&document);
    temper_playground::documents::settings::strip_models(&mut document);
    document.as_object_mut().expect("deployment").remove("world_settings");
    let mut agent = temper_playground::documents::agent();
    agent.programs = Box::new([temper_service::settings::Program {
        adapter: temper_service::settings::Adapter::ClaudeCode,
        path: b"/usr/bin/claude".as_slice().into(),
        directory: b"/worker/programs".as_slice().into(),
    }]);
    let mut endpoints = List::with_capacity(3);
    for (name, number, dialect) in [("fake", 0, 3), ("anthropic", 2, 2), ("anthropic-api", 4, 2)] {
        endpoints
            .push(smith_protocol_channel::Endpoint { name: name.as_bytes().into(), number, dialect })
            .expect("three endpoints");
    }
    agent.endpoints = smith_protocol_channel::Endpoints::new(endpoints);
    let mut service = temper_service::startup::with_setup_given(
        &serde_json::to_vec(&document).expect("deployment JSON"),
        &jig_protocol_config::Directories::working(),
        &Accounts::files(),
        Some(&agent),
        &Accounts::seeds(parameters.seed),
        None,
        Some(temper_service::WorkerRuntime {
            wires: None,
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
        .expect("program resources");
    service.configure_program_sources(resources).expect("program sources");
    service.adopt_store(root, 1000).expect("account store");
    service.adopt_signals(inherited.signal).expect("service signal");
    Proc::Service(Box::new(ServiceHost::new(service, ServiceObserver::started(root, inherited), parameters)))
}

struct Judge {
    finding: Option<String>,
    controls: skein_world::Controls,
    store: Rc<RefCell<Store>>,
    stage: u32,
    target: usize,
    answered: usize,
    deadline: Time,
}
fn command(caller: &mut Caller, words: &[&str]) {
    caller.commands.push_back(Command::new(&words.iter().map(|word| word.as_bytes()).collect::<Vec<_>>()));
}
fn listed(caller: &Caller) -> Result<Value, String> {
    Ok({
        let document: Value =
            serde_json::from_slice(&caller.answers.last().ok_or_else(|| String::from("account list"))?.output)
                .map_err(|why| format!("accounts JSON: {why:?}"))?;
        document["accounts"].clone()
    })
}
fn assert_account(account: &Value, kinds: &[&str]) -> Result<(), String> {
    match (&(account["provider"]), &(2)) {
        (left, right) if left != right => {
            let finding = String::from("assertion `left == right` failed");
            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
        }
        _ => {}
    }
    match (&(account["provider_name"]["bytes"]), &("anthropic")) {
        (left, right) if left != right => {
            let finding = String::from("assertion `left == right` failed");
            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
        }
        _ => {}
    }
    let sources = account["sources"].as_array().ok_or_else(|| String::from("sources"))?;
    match (
        &(sources
            .iter()
            .map(|source| source["kind"].as_str().ok_or_else(|| String::from("source kind")))
            .collect::<Result<Vec<_>, _>>()?),
        &(kinds),
    ) {
        (left, right) if left != right => {
            let finding = String::from("assertion `left == right` failed");
            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
        }
        _ => {}
    }

    Ok(())
}
impl Judge {
    #[expect(clippy::too_many_lines, reason = "the ordered referee checks return their findings at each step")]
    fn act_checked(&mut self, now: Time, procs: &mut [Proc]) -> Result<(), String> {
        let (services, rest) = procs.split_at_mut(1);
        let Proc::Service(service) = &mut services[0] else { return Err(String::from("service first")) };
        let Proc::Other(HarnessProcess::Calls(caller)) = &mut rest[0] else {
            return Err(String::from("caller second"));
        };
        if self.stage == 0
            && let Some(link) = service.service.links().pop()
        {
            let mut sign_in = Command::new(&[b"sign-in", b"-"]);
            sign_in.input = link;
            caller.commands.push_back(sign_in);
            command(caller, &["account", "add-key", "2", "/setup/llm.key", "--key", "claude-key"]);
            command(caller, &["account", "list", "--json"]);
            self.target = 3;
            self.stage = 1;
        }
        while self.answered < caller.answers.len() {
            let answer = &caller.answers[self.answered];
            match (&(answer.exit), &(Exit::Code(0))) {
                (left, right) if left != right => {
                    let finding = format!(
                        "account call {}: {} {}",
                        self.answered,
                        String::from_utf8_lossy(&answer.output),
                        String::from_utf8_lossy(&answer.error)
                    );
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
            self.answered += 1;
            self.deadline = now.saturating_add(temper_client_shell_cli::limits::DEADLINE);
        }
        if self.stage != 0 && caller.answers.len() == self.target {
            match self.stage {
                1 => {
                    let list = listed(caller)?;
                    let party = list["party"].as_array().ok_or_else(|| String::from("party accounts"))?;
                    match (&(party.len()), &(1)) {
                        (left, right) if left != right => {
                            let finding = String::from("assertion `left == right` failed");
                            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                        }
                        _ => {}
                    }
                    assert_account(&party[0], &["key"])?;
                    if !(party[0]["holder"]["party"].is_number()) {
                        return Err(format!(
                            "assertion failed: {}",
                            stringify!(party[0]["holder"]["party"].is_number())
                        ));
                    }
                    command(caller, &["account", "declare", "2", "claude_code", "personal", "--key", "claude-login"]);
                    command(caller, &["account", "list", "--json"]);
                    self.target += 2;
                    self.stage = 2;
                }
                2 => {
                    let list = listed(caller)?;
                    let account = list["party"]
                        .as_array()
                        .ok_or_else(|| String::from("party accounts"))?
                        .iter()
                        .find(|account| account["sources"][0]["kind"] == "program")
                        .ok_or_else(|| String::from("declared login"))?;
                    assert_account(account, &["program"])?;
                    let number = account["number"].as_u64().ok_or_else(|| String::from("account number"))?.to_string();
                    command(
                        caller,
                        &[
                            "account",
                            "add-key",
                            "2",
                            "/setup/llm.key",
                            "--account",
                            &number,
                            "--key",
                            "claude-fallback",
                        ],
                    );
                    command(
                        caller,
                        &["account", "add-key", "2", "/setup/llm.key", "--deployment", "--key", "claude-deployment"],
                    );
                    command(caller, &["account", "list", "--json"]);
                    self.target += 3;
                    self.stage = 3;
                }
                3 => {
                    let list = listed(caller)?;
                    let party = list["party"].as_array().ok_or_else(|| String::from("party accounts"))?;
                    match (&(party.len()), &(2)) {
                        (left, right) if left != right => {
                            let finding = String::from("assertion `left == right` failed");
                            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                        }
                        _ => {}
                    }
                    assert_account(&party[0], &["key"])?;
                    assert_account(&party[1], &["program", "key"])?;
                    match (&(party[1]["holder"]), &(party[0]["holder"])) {
                        (left, right) if left != right => {
                            let finding = String::from("assertion `left == right` failed");
                            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                        }
                        _ => {}
                    }
                    match (&(party[1]["sources"][0]["name"]), &("personal")) {
                        (left, right) if left != right => {
                            let finding = String::from("assertion `left == right` failed");
                            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                        }
                        _ => {}
                    }
                    let deployment =
                        list["deployment"].as_array().ok_or_else(|| String::from("owner sees deployment accounts"))?;
                    let account = deployment
                        .iter()
                        .find(|account| account["provider"] == 2)
                        .ok_or_else(|| String::from("deployment Claude key"))?;
                    assert_account(account, &["key"])?;
                    match (&(account["holder"]["deployment"]), &(true)) {
                        (left, right) if left != right => {
                            let finding = String::from("assertion `left == right` failed");
                            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                        }
                        _ => {}
                    }
                    caller.closed = true;
                    self.controls.signal(0, ServiceSignal::Terminate);
                    self.stage = 4;
                }
                _ => {}
            }
        }
        self.store.borrow_mut().pending.clear();

        Ok(())
    }
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        if self.finding.is_none() {
            self.finding = self.act_checked(now, procs).err();
        }
    }
    fn watch(&mut self, _: Time, procs: &[Proc]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }
        for proc in procs {
            if let Proc::Service(service) = proc
                && let Some(why) = &service.observer.finding
            {
                return Err(why.clone());
            }
        }

        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        (self.stage < 4).then_some(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (self.stage < 4 && now >= self.deadline)
            .then(|| format!("Claude accounts stage {}, answers {}/{}", self.stage, self.answered, self.target))
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && (self.stage == 4)
    }
}

/// Runs all three additions with the real client, engine, secrets store and worker.
pub fn walk() {
    stories::set(Story::Adopt);
    stories::setup_settings(world::Phase::Restart);
    let store = Rc::new(RefCell::new(Store::new()));
    world::lower_store(Some(store.clone()));
    let mut disk = temper_playground::machine::fresh();
    temper_playground::machine::setup(&mut disk);
    let caller = disk.reopen_root(2);
    let machine = Machine::new(disk);
    let parameters = world::parameters(&machine, stories::forge(Story::Adopt));
    PARAMETERS.with(|at| *at.borrow_mut() = Some(parameters.clone()));
    let outcome = programs::with(parameters, || {
        let mut simulation = skein_world::World::new_controlled(
            fixtures::SEED,
            skein_sim::Config::calm(),
            skein_world::Memory::Unchecked,
            |controls| Judge {
                finding: None,
                controls,
                store,
                stage: 0,
                target: 0,
                answered: 0,
                deadline: Time::ZERO.saturating_add(temper_client_shell_cli::limits::DEADLINE),
            },
        )
        .with_machine(machine);
        for (program, roots) in programs::programs::<Accounts>() {
            simulation.host_roots(program, roots);
        }
        simulation.host_roots(
            crate::ctl::program(|spawn, inherited| {
                Proc::Other(HarnessProcess::Ctl(Box::new(crate::ctl::host(spawn, inherited).capturing())))
            }),
            crate::ctl::roots,
        );
        let mut roots = programs::service_roots();
        roots.push(skein_world::StartupRoot {
            name: b"programs_parent".as_slice().into(),
            path: b"worker".as_slice().into(),
        });
        simulation.spawn_restartable(roots, service);
        simulation.spawn_root(skein_sim::Handle::new(caller.raw()), |root| {
            Proc::Other(HarnessProcess::Calls(Box::new(Caller::scripted(root))))
        });
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
        simulation.run()
    });
    assert!(outcome.procs.iter().all(Host::is_empty), "account world drains every process");
    world::lower_store(None);
    PARAMETERS.with(|at| *at.borrow_mut() = None);
}
