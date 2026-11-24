//! Connect and install through the manifest, then exercise picker scope or delivery (temper/docs/design/testing/tiers.md, 1.5).
use super::{ACME, GIT_RECORDS, REPOSITORY, UMBRA, adopt, peer_with_readme, set_active};
use crate::{
    ctl::{Caller, Command},
    fixtures, m15,
    m15::observer,
    referee::{self, Store},
    stories,
    world::{self, HarnessProcess, OneProcess, Proc, Story},
};
use serde_json::{Value, json};
use skein_fake_forge_github::installations::{Kind, Reach, TOKEN_LIFE};
use skein_fake_machine::{How, Is, Machine as Disk, Opened, Refusal};
use skein_fake_peers::forge::{Forge, Kept, Observation, Peer};
use skein_io::kernel::Exit;
use skein_lib::{Duration, Time, Wall};
use skein_shell::Host;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use temper_playground::{
    machine::{Machine, SignalHost},
    programs,
};

const INSTALLATION: u64 = 1;
const STRAY: u64 = 2;
const README: &str = "# app\n";
const EDIT: &str = "Changed by temper.\n";
const CONNECT: usize = 1;
const VISIT: usize = 2;
const SETTLED: usize = 3;
const PICKER: usize = 4;
const ADOPTED: usize = 4;
const REFUSED: usize = 5;
const DELIVERED: usize = 10;
const PROCEDURE: u64 = 2;
const CODER: u64 = 3;
const PATIENCE: Time = Time::ZERO.saturating_add(Duration::from_secs(10));

thread_local! { static ACTIVE: Cell<bool> = const { Cell::new(false) }; }

pub(super) fn configure(document: &mut temper_playground::documents::Configuration) {
    if ACTIVE.with(Cell::get) {
        // The manifest conversion carries the App key in a decoded document.
        document.temper_over("forge_component.body_bytes", 16_384_u32);
    }
}

#[derive(Clone, Copy, Debug)]
enum Goal {
    Picker,
    Push,
}
impl Goal {
    fn calls(self) -> usize {
        match self {
            Self::Picker => REFUSED + 1,
            Self::Push => DELIVERED + 1,
        }
    }
    fn exit(self, index: usize) -> Exit {
        Exit::Code(if matches!(self, Self::Picker) && index == REFUSED { 2 } else { 0 })
    }
}

fn connect(caller: &mut Caller, link: Box<[u8]>) {
    let mut sign_in = Command::new(&[b"sign-in", b"-"]);
    sign_in.input = link;
    caller.commands.extend([sign_in, Command::new(&[b"forge", b"connect", b"github", b"--key", b"github-connect"])]);
}

fn continue_after_setup(caller: &mut Caller, goal: Goal) {
    caller.commands.push_back(Command::new(&[b"forge", b"list", b"--until", b"settled", b"--json"]));
    match goal {
        Goal::Picker => caller.commands.extend([
            Command::new(&[b"forge", b"repositories", b"1", b"--installation", b"1", b"--json"]),
            adopt(b"umbra", b"tool", b"adopt-umbra"),
        ]),
        Goal::Push => caller.commands.extend([
            adopt(b"acme", b"app", b"adopt-app"),
            Command::new(&[b"account", b"add-key", b"1", b"/setup/llm.key", b"--deployment", b"--key", b"github-llm"]),
            Command::new(&[b"account", b"list", b"--until", b"usable"]),
            Command::new(&[
                b"chat",
                b"start",
                b"--project",
                b"1",
                b"--key",
                b"github-change",
                b"@forge_chat Update README",
            ]),
            Command::new(&[b"task", b"show", b"1", b"--until", b"replied"]),
            // An idle procedure has durably made its coder before the next call watches it.
            Command::new(&[b"task", b"show", b"2", b"--until", b"phase:idle"]),
            Command::new(&[b"task", b"show", b"3", b"--until", b"ended", b"--json"]),
        ]),
    }
}

fn check_connection(caller: &Caller, peer: &Peer) {
    let [visit] = caller.browsed.as_slice() else { panic!("one manifest visit") };
    assert_eq!(visit.call, CONNECT);
    assert_eq!(visit.hops, [200, 200, 302, 302, 200], "manifest, create, redirect, install and setup");
    assert!(skein_lib::bytes::find(&visit.body, b"temper has the installation; you may close this tab.").is_some());
    let settled: Value = serde_json::from_slice(&caller.answers[SETTLED].output).expect("settled forges JSON");
    let slug = visit.app_slug.as_ref().expect("converted App's slug");
    let [forge] = settled["forges"]["forges"].as_array().expect("forge rows").as_slice() else {
        panic!("one settled forge")
    };
    assert_eq!(forge["state"], "connected");
    assert_eq!(forge["login"], format!("{slug}[bot]"), "connected as the converted App's bot");
    let installed = peer.github_layer().expect("GitHub layer").installations();
    let acme = installed.get(INSTALLATION).expect("acme stays installed");
    assert_eq!((acme.id, acme.account, acme.login.as_ref()), (INSTALLATION, ACME, b"acme".as_slice()));
    let umbra = installed.get(STRAY).expect("umbra still holds the distinct installation");
    assert_eq!((umbra.id, umbra.account, umbra.login.as_ref()), (STRAY, UMBRA, b"umbra".as_slice()));
    let mint = format!("/app/installations/{STRAY}/access_tokens");
    let heads = || {
        peer.observations().iter().filter_map(|record| match record {
            Observation::Head { call, .. } => Some(call),
            _ => None,
        })
    };
    assert_eq!(
        heads()
            .filter(|call| call.method == skein_http::Method::Post
                && call.target.starts_with(b"/app-manifests/")
                && call.target.ends_with(b"/conversions"))
            .count(),
        1,
        "the engine converted the App instead of pasting a key"
    );
    assert!(
        !heads().any(|call| call.target.starts_with(mint.as_bytes())),
        "no access-token request for the unaccepted installation"
    );
}

fn check_picker(caller: &Caller, peer: &Peer) {
    let page: Value = serde_json::from_slice(&caller.answers[PICKER].output).expect("repositories JSON");
    assert_eq!(
        page,
        json!({"rows":[{"owner":{"login":"acme","id":ACME},"name":"app","id":REPOSITORY,"private":true}],"more":false})
    );
    let refused: Value = serde_json::from_slice(&caller.answers[REFUSED].error).expect("refused adoption JSON");
    assert_eq!(caller.answers[REFUSED].exit, Exit::Code(2));
    assert_eq!(refused["forge_refused"]["reason"], "no_access");
    assert!(peer.git_calls().expect("git records").is_empty(), "no git call before the refusal");
}

fn check_delivery(caller: &Caller, peer: &Peer) {
    let adopted: Value = serde_json::from_slice(&caller.answers[ADOPTED].output).expect("adopted repository JSON");
    let adopted = &adopted["adopted"];
    assert_eq!(adopted["owner"], "acme");
    assert_eq!(adopted["name"], "app");
    assert_eq!(adopted["repository"], REPOSITORY);
    let delivered: Value = serde_json::from_slice(&caller.answers[DELIVERED].output).expect("coder's ended task JSON");
    let task = &delivered["task"]["chip"];
    let change = &task["details"]["owner"]["details"]["ending"]["done"]["change"];
    assert_eq!(task["task"], CODER);
    assert!(change.is_object(), "the ended coder delivered a change");
    assert_eq!(change["kind"], 2, "the coder's produced change");
    assert_eq!(change["part"]["key"]["number"], adopted["repository"]);

    let minted = peer.github_layer().expect("GitHub layer").installations().minted();
    assert!(minted > 0);
    let git = peer.git_calls().expect("git records");
    assert!(git.len() < GIT_RECORDS, "git observation ring did not fill");
    assert!(git.iter().any(|call| !call.writes && call.status == 200), "successful clone/read");
    assert_eq!(git.iter().filter(|call| call.writes).count(), 1, "exactly one push writes");
    for call in git {
        let identity = call.minted.expect("every git request authenticates with a minted token");
        assert_eq!(identity.installation, INSTALLATION);
        assert!((1..=minted).contains(&identity.mint));
        assert!(call.status == 200 || (!call.writes && call.status == 404), "{call:?}");
    }
}

// Fetched objects live in stores; files in a run's gitdirs/.../objects are local commits.
fn local_commits(disk: &mut Disk, dir: Opened, found: &mut Vec<u64>) {
    loop {
        let entries = disk.list(dir, 16, 4096).expect("git directory entries");
        if entries.is_empty() {
            break;
        }
        for (kind, name) in entries {
            if kind != Is::Directory {
                continue;
            }
            let child = disk.open(dir, &name, How::DirectoryNoFollow).expect("git directory");
            if name.as_ref() == b"objects" {
                loop {
                    let objects = disk.list(child, 16, 4096).expect("local object entries");
                    if objects.is_empty() {
                        break;
                    }
                    for (kind, name) in objects {
                        if kind == Is::File {
                            found.push(
                                std::str::from_utf8(&name).expect("object name").parse().expect("counted object"),
                            );
                        }
                    }
                }
            } else {
                local_commits(disk, child, found);
            }
            disk.close(child);
        }
    }
}

fn ready_to_stop_after_push(service: &world::ServiceHost, rest: &[Proc]) -> Result<bool, String> {
    use jig_domain_tasks::{Active, Phase, Was};
    let engine = service.service.engine_domain().ok_or("live engine")?;
    let tasks = engine.tasks();
    if tasks.task(PROCEDURE).is_none() {
        return Err("the change procedure is left live at the coder's delivery".into());
    }
    for row in &tasks.view_tasks() {
        let task = tasks.task(row.number).ok_or("live task")?;
        let owed = match &task.phase {
            Phase::Active(active) | Phase::Held { was: Was::Active(active), .. } => {
                matches!(active, Active::Due | Active::Preparing | Active::Claimed { .. } | Active::Running { .. })
            }
            Phase::Closing(_) | Phase::Held { was: Was::Closing(_), .. } => {
                return Err(format!("task {} still closing at the coder's delivery", row.number));
            }
            _ => false,
        };
        if owed && row.number != PROCEDURE {
            if row.number == 1 {
                return Ok(false); // The chat parks again if the delivery woke it.
            }
            return Err(format!("unexpected live obligation on task {} after the push", row.number));
        }
    }
    let worker = service.service.worker_domain().ok_or("live worker")?;
    Ok(engine.pending_assignments().is_empty()
        && worker.hosting().is_empty()
        && !rest.iter().any(|proc| {
            matches!(proc, Proc::Agent(_) | Proc::Git(_) | Proc::GitRoute(_) | Proc::Shell(_)) && proc.exit().is_none()
        }))
}

struct Judge {
    goal: Goal,
    controls: skein_world::Controls,
    store: Rc<RefCell<Store>>,
    finding: Option<String>,
    answers: usize,
    stopping: bool,
}
impl Judge {
    fn step(&mut self, procs: &mut [Proc]) -> Result<(), String> {
        let [Proc::Service(service), Proc::Other(HarnessProcess::Calls(caller)), Proc::Forge(peer), rest @ ..] = procs
        else {
            return Err("service, caller and forge first".into());
        };
        if self.stopping {
            return Ok(());
        }
        if let Some(link) = service.service.links().pop() {
            connect(caller, link);
        }
        let before = self.answers;
        for (index, answer) in caller.answers.iter().enumerate().skip(before) {
            let surface = if index == VISIT { "browser setup page" } else { "ctl stdout" };
            observer::artifact(surface, &answer.output);
            observer::artifact("ctl stderr", &answer.error);
            if answer.exit != self.goal.exit(index) {
                return Err(format!(
                    "{index}: unexpected exit {:?}: {} {}",
                    answer.exit,
                    observer::public_output(&answer.output)?,
                    observer::public_output(&answer.error)?
                ));
            }
        }
        self.answers = caller.answers.len();
        if before <= CONNECT && self.answers > CONNECT {
            caller.commands.push_back(Command::browse(CONNECT));
        }
        if before <= VISIT && self.answers > VISIT {
            let installed = peer.github_layer_mut().ok_or("GitHub layer")?.installations_mut();
            if !installed.iter().map(|row| (row.id, row.account)).eq([(INSTALLATION, ACME)]) {
                return Err("only acme installed by the browser".into());
            }
            let id = installed
                .install(UMBRA, b"umbra", Kind::Organization, Reach::All, Wall::EPOCH)
                .map_err(|why| format!("seed unaccepted installation: {why:?}"))?;
            if id != STRAY {
                return Err("umbra has its own installation".into());
            }
            m15::checkpoint(service, &self.store.borrow())?;
            continue_after_setup(caller, self.goal);
        }
        if self.answers == self.goal.calls() {
            if matches!(self.goal, Goal::Push) && !ready_to_stop_after_push(service, rest)? {
                return Ok(());
            }
            m15::checkpoint(service, &self.store.borrow())?;
            caller.closed = true;
            self.controls.signal(0, skein_io::kernel::ServiceSignal::Terminate);
            peer.shutdown();
            for proc in rest {
                referee::shutdown_peer(proc);
            }
            self.stopping = true;
        }
        Ok(())
    }
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Proc]) {
        if self.finding.is_none() {
            self.finding = self.step(procs).err();
        }
    }
    fn watch(&mut self, _now: Time, procs: &[Proc]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }
        for proc in procs {
            match proc {
                Proc::Service(service) => {
                    if let Some(why) = &service.observer.finding {
                        return Err(why.clone());
                    }
                }
                Proc::Peer(peer) => observer::context(peer)?,
                _ => {}
            }
        }
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.stopping).then_some(PATIENCE)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.stopping && now >= PATIENCE)
            .then(|| format!("GitHub {:?}: {} of {} answers", self.goal, self.answers, self.goal.calls()))
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && self.stopping
    }
}

fn check_no_handover(disk: &mut Disk, peer: &Peer) {
    let counts = observer::finish_manifest(disk).unwrap_or_else(|finding| panic!("{finding}"));
    assert!(counts.tokens > 0, "the secrecy scan accompanies token minting");
    assert_eq!(
        u64::try_from(counts.tokens).expect("fixture token count"),
        peer.github_layer().expect("GitHub layer").installations().minted(),
        "the observer learned every minted token"
    );
    let setup = disk.reopen_root(3);
    assert!(
        matches!(disk.open(setup, b"github-app.pem", How::PathNoFollow), Err(Refusal::NotFound)),
        "no App key downloaded"
    );
    disk.close(setup);
    // Check the exclusion is jig's directory, without opening or reading any of its entries.
    let state = disk.reopen_root(0);
    let secrets = disk.open(state, b"generations/1/secrets", How::PathNoFollow).expect("jig's secrets directory");
    assert_eq!(disk.stat(secrets).is, Is::Directory);
    disk.close(secrets);
    disk.close(state);
}

fn check_pushed_tree(disk: &mut Disk, forge: &Forge) {
    let branches = forge.domain.branches(b"acme/app");
    assert_eq!(branches.len(), 2, "one pushed branch beside main");
    assert!(branches.get(b"main".as_slice()).is_some());
    let (_, head) = branches.iter().find(|(name, _)| name.as_ref() != b"main").expect("pushed branch");
    let object = forge.domain.object(*head).expect("pushed object");
    let (_, readme) = object.tree.iter().find(|(path, _)| path.as_ref() == b"README.md").expect("pushed README");
    assert_eq!(readme.as_ref(), [README, EDIT].concat().as_bytes());
    let worker = disk.reopen_root(1);
    let gitdirs = disk.open(worker, b"gitdirs", How::DirectoryNoFollow).expect("worker git directories");
    let mut commits = Vec::new();
    local_commits(disk, gitdirs, &mut commits);
    disk.close(gitdirs);
    disk.close(worker);
    assert_eq!(commits, [*head], "the pushed head is the coder's only local commit");
}

fn run(goal: Goal) {
    ACTIVE.with(|active| active.set(true));
    set_active(true);
    observer::start_manifest();
    stories::set(Story::Adopt);
    stories::setup_settings(world::Phase::Story);
    let store = Rc::new(RefCell::new(Store::new()));
    world::lower_store(Some(store.clone()));
    let (forge, peer) = peer_with_readme(TOKEN_LIFE, false, README.as_bytes());
    let peer = peer.keeping(Kept { heads: true, ..Kept::NONE });
    let transport = temper_fake_git_hosted::Typed::new(forge.clone(), &stories::forge_table());
    let mut disk = temper_playground::machine::fresh();
    // No private App key is supplied. The fake LLM's input is the only setup file.
    let setup = disk.lay(&[skein_fake_machine::Item::file(b"llm.key", fixtures::LLM_TOKEN).mode(0o600)]);
    disk.close(setup);
    let client = disk.reopen_root(2);
    let machine = Machine::new(disk);
    let parameters = world::parameters(&machine, transport);
    let outcome = programs::with(parameters, || {
        let mut world = skein_world::World::new_controlled(
            fixtures::SEED,
            skein_sim::Config::calm(),
            skein_world::Memory::Unchecked,
            |controls| Judge { goal, controls, store, finding: None, answers: 0, stopping: false },
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
        world.spawn(|| Proc::Forge(Box::new(SignalHost::new(peer, None))));
        if matches!(goal, Goal::Push) {
            world.spawn(|| Proc::Peer(Box::new(SignalHost::new(push_provider(), None))));
        }
        world.run()
    });
    assert!(outcome.procs.iter().all(Host::is_empty), "manifest processes drain");
    let [_, Proc::Other(HarnessProcess::Calls(caller)), Proc::Forge(peer), ..] = outcome.procs.as_slice() else {
        panic!("caller and GitHub peer")
    };
    let mut disk = outcome.machine.0.borrow_mut();
    check_no_handover(&mut disk, peer);
    check_connection(caller, peer);
    match goal {
        Goal::Picker => check_picker(caller, peer),
        Goal::Push => {
            check_delivery(caller, peer);
            check_pushed_tree(&mut disk, &forge.borrow());
        }
    }
    world::lower_store(None);
    set_active(false);
    ACTIVE.with(|active| active.set(false));
}

/// Connect by manifest and check picker and adoption scope with a distinct unaccepted installation.
pub fn manifest_picker() {
    run(Goal::Picker);
}

/// Deliver a coder's README edit on the connected App's installation token, without landing.
pub fn manifest_push() {
    run(Goal::Push);
}

fn push_provider() -> skein_fake_peers::llm::Peer {
    use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
    let call = |name: &[u8], args: Value| Turn {
        lines: Box::new([Line::Call {
            name: name.into(),
            arguments: serde_json::to_vec(&args).expect("tool JSON").into(),
        }]),
        finish: Finish::ToolCalls,
        tokens: fixtures::CALL_TOKENS,
    };
    let grant = |kind| json!({"connector":fixtures::FORGE_CONNECTOR,"kind":kind,"segments":["forge",fixtures::FORGE.to_string()],"terminal":"open","last":"acme"});
    let delegate = json!({"batch":[{
        "executor":{"kind":"procedure","connector":fixtures::FORGE_CONNECTOR,"code":fixtures::PROCEDURE_CODE},
        "spec":{"words":"@edit Update README","parameters":[{"name":1,"kind":"resource","connector":fixtures::FORGE_CONNECTOR,"resource":fixtures::REPOSITORY_TAG},{"name":2,"kind":"bytes","value":"main"}]},
        "contract":{"kind":"change","connector":fixtures::FORGE_CONNECTOR,"change_kind":fixtures::CHANGE_KIND,"words":fixtures::CHANGE_WORDS},
        "authority":{"tools":fixtures::PRODUCER_TOOLS,"grants":m15::DELIVERY_KINDS.map(grant),"delegation":{"kinds":[{"kind":"agent","number":fixtures::AGENT_KIND}],"tasks":fixtures::PRODUCER_TASKS,"depth":fixtures::PRODUCER_DEPTH},"budget":{"spend":fixtures::PRODUCER_SPEND},"notes":fixtures::PRODUCER_NOTES}
    }]});
    fixtures::scripted_provider(Box::new([
        Script {
            cue: b"@forge_chat".as_slice().into(),
            turns: Box::new([
                call(b"delegate", delegate),
                Turn {
                    lines: Box::new([
                        Line::Text { text: b"I asked the coder to update README.".as_slice().into() },
                        Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
                    ]),
                    finish: Finish::ToolCalls,
                    tokens: fixtures::CALL_TOKENS,
                },
                call(b"wait", json!({})),
            ]),
        },
        Script {
            cue: b"@edit Update README".as_slice().into(),
            turns: Box::new([
                call(b"read", json!({"path":"README.md"})),
                call(b"edit", json!({"path":"README.md","edits":[{"old":README,"new":([README, EDIT].concat())}]})),
                call(
                    b"finish",
                    json!({"form":"change","fields":{"title":"Update README","body":"Changed by temper."}}),
                ),
            ]),
        },
    ]))
}
