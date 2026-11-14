//! Lower owners of service composition behaviors (temper/docs/design/testing/real-loop.md).
use temper_playground::{
    machine::{Machine, SignalHost},
    programs,
};
pub mod github;
mod paging;
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
thread_local! { static ACTIVE: Cell<bool> = const { Cell::new(false) }; }
pub(crate) fn configure(value: &mut Value) {
    if !ACTIVE.with(Cell::get) {
        return;
    }
    for (name, count) in [
        ("people.inbox_tasks", 16),
        ("tasks.tasks", 8),
        ("tasks.project_tasks", 8),
        ("tasks.tree_tasks", 8),
        ("tasks.batch", 4),
        ("authority.batch", 4),
        ("tasks.delegates", 5),
        ("people.operations", 6),
        ("documents.rows", 2),
        ("documents.lines", 3),
    ] {
        value["engine"]["limits"]["overrides"][name] = count.into();
    }
    let policy = &mut value["world_settings"]["policies"][0]["policy"];
    policy["ceiling"]["budget"]["spend"] = 500.into();
    policy["roles"][0]["authority"]["budget"]["spend"] = 500.into();
    policy["period_spend"] = 1000.into();
    policy["roles"][0]["period_spend"] = 1000.into();
    let kinds = json!([{"kind":"charter","number":1},{"kind":"role","number":0}]);
    value["world_settings"]["policies"][0]["policy"]["ceiling"]["delegation"]["kinds"] = kinds.clone();
    value["world_settings"]["policies"][0]["policy"]["roles"][0]["authority"]["delegation"]["kinds"] = kinds;
}
#[derive(Default, Debug)]
pub struct Paging {
    pub pages: usize,
    pub positions: Vec<u64>,
    pub delegates: Vec<u64>,
    pub tree: String,
}
#[derive(Clone, Copy, Debug)]
enum Stage {
    Setup,
    Settled,
    Task,
    Activity,
    Tree,
    Delegates,
    Stop,
    Done,
}
struct Judge {
    finding: Option<String>,
    controls: skein_world::Controls,
    store: Rc<RefCell<Store>>,
    seen: Rc<RefCell<Paging>>,
    stage: Stage,
    answers: usize,
    activity_head: u64,
    handed: bool,
    stopping: bool,
    checked: Cell<u64>,
}
impl Judge {
    fn next(&mut self, caller: &mut Caller, words: &[&str], json: bool) -> Result<(), String> {
        if self.answers >= 20 {
            return Err(String::from("bounded continuation calls"));
        }
        let mut words: Vec<_> = words.iter().map(|word| word.as_bytes()).collect();
        if json {
            words.push(b"--json");
        }
        caller.commands.push_back(Command::new(&words));

        Ok(())
    }
    fn answer(&mut self, caller: &mut Caller) -> Result<(), String> {
        let answer = &caller.answers[self.answers];
        referee::secrets::check(&answer.output, "paging output")?;
        referee::secrets::check(&answer.error, "paging diagnostic")?;
        if answer.exit != Exit::Code(0) {
            return Err(format!(
                "paging call {:?}/{} accepted: {} {}",
                self.stage,
                self.answers,
                String::from_utf8_lossy(&answer.output),
                String::from_utf8_lossy(&answer.error)
            ));
        }
        if !(answer.error.is_empty()) {
            return Err(String::from("paging call has no diagnostic"));
        }
        let output = answer.output.clone();
        self.answers += 1;
        match self.stage {
            Stage::Setup => {
                if self.answers == 3 {
                    self.stage = Stage::Settled;
                }
            }
            Stage::Task => {
                let task: Value = serde_json::from_slice(&output).map_err(|why| format!("task JSON: {why:?}"))?;
                self.activity_head = task["activity"].as_u64().ok_or_else(|| String::from("task activity"))?;
                if self.activity_head < 3 {
                    return Err(format!("assertion failed: {}", stringify!(self.activity_head >= 3)));
                }
                self.stage = Stage::Activity;
                self.next(caller, &["task", "activity", "3"], true)?;
            }
            Stage::Activity => self.activity(caller, &output)?,
            Stage::Tree => self.tree(caller, output)?,
            Stage::Delegates => self.delegates(caller, &output)?,
            Stage::Stop => self.stage = Stage::Done,
            Stage::Settled | Stage::Done => return Err(String::from("unexpected paging answer")),
        }

        Ok(())
    }

    fn activity(&mut self, caller: &mut Caller, output: &[u8]) -> Result<(), String> {
        let page: Value = serde_json::from_slice(output).map_err(|why| format!("activity JSON: {why:?}"))?;
        let rows = page["rows"].as_array().ok_or_else(|| String::from("activity rows"))?;
        if rows.is_empty() && rows.len() <= 2 {
            return Err(String::from("template row bound"));
        }
        let mut seen = self.seen.borrow_mut();
        seen.pages += 1;
        for row in rows {
            let position = row["position"].as_u64().ok_or_else(|| String::from("activity position"))?;
            if let Some(previous) = seen.positions.last() {
                match (&(position + 1), previous) {
                    (left, right) if left != right => {
                        let finding = String::from("no missing activity");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
            } else {
                match (&(position), &(self.activity_head)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
            }
            seen.positions.push(position);
        }
        drop(seen);
        if let Some(next) = page["read_on"].as_str() {
            self.next(caller, &next.split_ascii_whitespace().collect::<Vec<_>>(), true)?;
        } else {
            match (&(self.seen.borrow().positions.last()), &(Some(&1))) {
                (left, right) if left != right => {
                    let finding = String::from("assertion `left == right` failed");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
            self.stage = Stage::Tree;
            self.next(caller, &["task", "show", "1", "--tree"], false)?;
        }

        Ok(())
    }
    fn tree(&mut self, caller: &mut Caller, output: Vec<u8>) -> Result<(), String> {
        let text = String::from_utf8(output).map_err(|why| format!("tree text: {why:?}"))?;
        let next = text
            .lines()
            .find_map(|line| line.strip_prefix("read on with: "))
            .ok_or_else(|| String::from("tree continuation"))?
            .to_owned();
        self.seen.borrow_mut().tree = text;
        self.stage = Stage::Delegates;
        self.next(caller, &next.split_ascii_whitespace().collect::<Vec<_>>(), false)?;

        Ok(())
    }
    fn delegates(&mut self, caller: &mut Caller, output: &[u8]) -> Result<(), String> {
        let text = std::str::from_utf8(output).map_err(|why| format!("delegates text: {why:?}"))?;
        let tasks: Vec<u64> = text
            .lines()
            .filter_map(|line| line.split_once(" · ").and_then(|(number, _)| number.parse().ok()))
            .collect();
        if tasks.is_empty() && tasks.len() <= 2 {
            return Err(String::from("template task page bound"));
        }
        self.seen.borrow_mut().delegates.extend(tasks);
        if let Some(next) = text.lines().find_map(|line| line.strip_prefix("read on with: ")) {
            self.next(caller, &next.split_ascii_whitespace().collect::<Vec<_>>(), false)?;
        } else {
            self.stage = Stage::Stop;
            self.next(caller, &["task", "stop", "1", "--key", "paging-stop"], false)?;
        }

        Ok(())
    }
}
impl Judge {
    fn act_checked(&mut self, _now: Time, procs: &mut [Proc]) -> Result<(), String> {
        let (first, others) = procs.split_at_mut(1);
        let Proc::Service(service) = &mut first[0] else { return Err(String::from("service first")) };
        let Proc::Other(HarnessProcess::Calls(caller)) = &mut others[0] else {
            return Err(String::from("native caller second"));
        };
        if !self.handed
            && let Some(link) = service.service.links().pop()
        {
            let mut sign_in = Command::new(&[b"sign-in", b"-"]);
            sign_in.input = link;
            caller.commands.push_back(sign_in);
            caller.commands.push_back(Command::new(&[
                b"account",
                b"add-key",
                b"1",
                b"/setup/llm.key",
                b"--deployment",
                b"--key",
                b"paging-account",
            ]));
            caller.commands.push_back(Command::new(&[
                b"chat",
                b"start",
                b"--project",
                b"1",
                b"--key",
                b"paging-chat",
                b"@bounds-root",
            ]));
            self.handed = true;
        }
        if caller.answers.len() > self.answers {
            self.answer(caller)?;
        }
        if matches!(self.stage, Stage::Settled) && referee::terminals(&self.store.borrow()) == 2 {
            self.stage = Stage::Task;
            self.next(caller, &["task", "show", "3"], true)?;
        }
        if matches!(self.stage, Stage::Done) && !self.stopping {
            caller.closed = true;
            self.controls.signal(0, skein_io::kernel::ServiceSignal::Terminate);
            for proc in others {
                if let Proc::Peer(peer) = proc {
                    peer.shutdown();
                }
            }
            self.stopping = true;
        }

        Ok(())
    }
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        if self.finding.is_none() {
            self.finding = self.act_checked(now, procs).err();
        }
    }
    fn watch(&mut self, _now: Time, procs: &[Proc]) -> Result<(), String> {
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

        let Proc::Service(service) = &procs[0] else { return Err(String::from("service")) };
        let applied = self.store.borrow().applied;
        if self.checked.replace(applied) != applied {
            referee::Referee::check_roots(&service.service)?;
        }

        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.stopping).then_some(Time::from_nanos(10_000_000_000))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.stopping && now >= Time::from_nanos(10_000_000_000)).then(|| {
            format!(
                "paging stage {:?}, answers {}, tasks {:?}, agent {:?}",
                self.stage,
                self.answers,
                self.store
                    .borrow()
                    .rows
                    .values()
                    .filter(|row| matches!(row, temper_engine_domain::Record::Core(jig_domain_core::Record::Tasks(_))))
                    .collect::<Vec<_>>(),
                self.store.borrow().agent_records
            )
        })
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && (self.stopping)
    }
}
/// Read the Small profile's two-row pages through shipped native ctl and simulated files/sockets.
#[must_use]
pub fn paging(draw: u64) -> Paging {
    ACTIVE.with(|active| active.set(true));
    stories::set(Story::Adopt);
    stories::setup_settings(world::Phase::Story);
    let transport = stories::forge(Story::Adopt);
    let store = Rc::new(RefCell::new(Store::new()));
    world::lower_store(Some(store.clone()));
    let seen = Rc::new(RefCell::new(Paging::default()));
    let mut disk = temper_playground::machine::fresh();
    let setup = disk.lay(&[skein_fake_machine::Item::file(b"llm.key", fixtures::LLM_TOKEN).mode(0o600)]);
    disk.close(setup);
    let client = disk.reopen_root(2);
    let machine = Machine::new(disk);
    let parameters = world::parameters(&machine, transport.clone());
    let outcome = programs::with(parameters, || {
        let mut world = skein_world::World::new_controlled(
            draw,
            skein_sim::Config::calm(),
            skein_world::Memory::Unchecked,
            |controls| Judge {
                finding: None,
                controls,
                store,
                seen: seen.clone(),
                stage: Stage::Setup,
                answers: 0,
                activity_head: 0,
                handed: false,
                stopping: false,
                checked: Cell::new(u64::MAX),
            },
        )
        .with_machine(machine);
        for (program, roots) in programs::programs::<OneProcess>() {
            world.host_roots(program, roots);
        }
        world.host_roots(
            crate::ctl::program(|spawn, inherited| {
                Proc::Other(HarnessProcess::Ctl(Box::new(crate::ctl::host(spawn, inherited))))
            }),
            crate::ctl::roots,
        );
        world.spawn_restartable(programs::service_roots(), programs::service::<OneProcess>);
        world.spawn_root(skein_sim::Handle::new(client.raw()), |root| {
            Proc::Other(HarnessProcess::Calls(Box::new(Caller::scripted(root))))
        });
        world.spawn(|| Proc::Peer(Box::new(SignalHost::new(fixtures::scripted_provider(paging::scripts()), None))));
        world.run()
    });
    assert!(outcome.procs.iter().all(Host::is_empty), "all paging processes drain");
    world::lower_store(None);

    ACTIVE.with(|active| active.set(false));
    Rc::try_unwrap(seen).expect("released paging referee").into_inner()
}
