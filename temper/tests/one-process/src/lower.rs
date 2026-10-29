//! Lower owners of service composition behaviors (temper/docs/design/domain/testing.md, 2.4).
pub mod github;
mod paging;
use crate::{
    ctl::{Caller, Command},
    fixtures,
    machine::{Machine, SignalHost},
    referee::{self, Store},
    stories,
    world::{self, Proc, Story},
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
    Done,
}
struct Judge {
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
    fn next(&mut self, caller: &mut Caller, words: &[&str], json: bool) {
        assert!(self.answers < 20, "bounded continuation calls");
        let mut words: Vec<_> = words.iter().map(|word| word.as_bytes()).collect();
        if json {
            words.push(b"--json");
        }
        caller.commands.push_back(Command::new(&words));
    }
    fn answer(&mut self, caller: &mut Caller) {
        let answer = &caller.answers[self.answers];
        referee::secrets::require(&answer.output, "paging output");
        referee::secrets::require(&answer.error, "paging diagnostic");
        assert!(
            answer.exit == Exit::Code(0),
            "paging call {:?}/{} accepted: {} {}",
            self.stage,
            self.answers,
            String::from_utf8_lossy(&answer.output),
            String::from_utf8_lossy(&answer.error)
        );
        assert!(answer.error.is_empty(), "paging call has no diagnostic");
        let output = answer.output.clone();
        self.answers += 1;
        match self.stage {
            Stage::Setup => {
                if self.answers == 3 {
                    self.stage = Stage::Settled;
                }
            }
            Stage::Task => {
                let task: Value = serde_json::from_slice(&output).expect("task JSON");
                self.activity_head = task["activity"].as_u64().expect("task activity");
                assert!(self.activity_head >= 3);
                self.stage = Stage::Activity;
                self.next(caller, &["task", "activity", "3"], true);
            }
            Stage::Activity => {
                let page: Value = serde_json::from_slice(&output).expect("activity JSON");
                let rows = page["rows"].as_array().expect("activity rows");
                assert!(!rows.is_empty() && rows.len() <= 2, "template row bound");
                let mut seen = self.seen.borrow_mut();
                seen.pages += 1;
                for row in rows {
                    let position = row["position"].as_u64().expect("activity position");
                    if let Some(previous) = seen.positions.last() {
                        assert_eq!(position + 1, *previous, "no missing activity");
                    } else {
                        assert_eq!(position, self.activity_head);
                    }
                    seen.positions.push(position);
                }
                drop(seen);
                if let Some(next) = page["read_on"].as_str() {
                    self.next(caller, &next.split_ascii_whitespace().collect::<Vec<_>>(), true);
                } else {
                    assert_eq!(self.seen.borrow().positions.last(), Some(&1));
                    self.stage = Stage::Tree;
                    self.next(caller, &["task", "show", "1", "--tree"], false);
                }
            }
            Stage::Tree => {
                let text = String::from_utf8(output).expect("tree text");
                let next = text
                    .lines()
                    .find_map(|line| {
                        line.strip_prefix("cut: ")
                            .and_then(|line| line.split_once("; read on with: ").map(|(_, next)| next))
                    })
                    .expect("tree continuation")
                    .to_owned();
                self.seen.borrow_mut().tree = text;
                self.stage = Stage::Delegates;
                self.next(caller, &next.split_ascii_whitespace().collect::<Vec<_>>(), false);
            }
            Stage::Delegates => {
                let text = std::str::from_utf8(&output).expect("delegates text");
                let tasks: Vec<u64> = text
                    .lines()
                    .filter_map(|line| line.split_once(" · ").and_then(|(number, _)| number.parse().ok()))
                    .collect();
                assert!(!tasks.is_empty() && tasks.len() <= 2, "template task page bound");
                self.seen.borrow_mut().delegates.extend(tasks);
                if let Some(next) = text.lines().find_map(|line| line.strip_prefix("read on with: ")) {
                    self.next(caller, &next.split_ascii_whitespace().collect::<Vec<_>>(), false);
                } else {
                    self.stage = Stage::Done;
                }
            }
            Stage::Settled | Stage::Done => panic!("unexpected paging answer"),
        }
    }
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Proc]) {
        let (first, others) = procs.split_at_mut(1);
        let Proc::Service(service) = &mut first[0] else { panic!("service first") };
        let Proc::Calls(caller) = &mut others[0] else { panic!("native caller second") };
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
            self.answer(caller);
        }
        if matches!(self.stage, Stage::Settled) && referee::terminals(&self.store.borrow()) == 2 {
            self.stage = Stage::Task;
            self.next(caller, &["task", "show", "3"], true);
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
    }
    fn observe(&mut self, _now: Time, procs: &[Proc]) {
        let Proc::Service(service) = &procs[0] else { panic!("service") };
        let applied = self.store.borrow().applied;
        if self.checked.replace(applied) != applied {
            referee::Referee::check_roots(service);
        }
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
        self.stopping
    }
}
/// Read the Small profile's two-row pages through shipped native ctl and simulated files/sockets.
#[must_use]
pub fn paging(draw: u64) -> Paging {
    ACTIVE.with(|active| active.set(true));
    stories::set(Story::Adopt);
    stories::setup_settings(world::Phase::Story);
    world::share(Some(stories::forge(Story::Adopt)));
    let store = Rc::new(RefCell::new(Store::new()));
    world::lower_store(Some(store.clone()));
    let seen = Rc::new(RefCell::new(Paging::default()));
    let mut disk = crate::machine::fresh();
    let setup = disk.lay(&[skein_fake_machine::Item::file(b"llm.key", fixtures::LLM_TOKEN).mode(0o600)]);
    disk.close(setup);
    let client = disk.reopen_root(2);
    let mut world = skein_world::World::new_controlled(
        draw,
        skein_sim::Config::calm(),
        skein_world::Memory::Unchecked,
        |controls| Judge {
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
    .with_machine(Machine(disk));
    world::host_programs(&mut world);
    world.spawn_restartable(world::service_roots(), world::service);
    world.spawn_root(skein_sim::Handle::new(client.raw()), |root| Proc::Calls(Box::new(Caller::scripted(root))));
    world.spawn(|| Proc::Peer(Box::new(SignalHost::new(fixtures::scripted_provider(paging::scripts()), None))));
    let outcome = world.run();
    assert!(outcome.procs.iter().all(Host::is_empty), "all paging processes drain");
    world::lower_store(None);
    world::share(None);
    ACTIVE.with(|active| active.set(false));
    Rc::try_unwrap(seen).expect("released paging referee").into_inner()
}
