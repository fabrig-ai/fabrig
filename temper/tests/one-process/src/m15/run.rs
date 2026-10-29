//! A read, edit, renewal and authenticated delivery exercise credential boundaries (temper/docs/design/protocol/credentials.md, 4–7).
use super::{checkpoint, observer, pages, paste};
use crate::{
    ctl::{Caller, Command},
    fixtures,
    lower::github,
    machine::{Machine, SignalHost},
    referee::{self, Store},
    stories,
    world::{self, Person, Proc, Story},
};
use serde_json::json;
use skein_fake_forge_domain as forge;
use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
use skein_fake_peers::forge::Forge;
use skein_io::kernel::Exit;
use skein_lib::{Duration, Time};
use skein_shell::Host;
use std::{cell::RefCell, rc::Rc};
// Five seconds until renewal under the component's unchanged ten-minute lead.
const TOKEN_LIFE: Duration = Duration::from_secs(605);
const PATIENCE: Time = Time::from_nanos(30_000_000_000);
fn call(name: &[u8], args: &[u8]) -> Turn {
    Turn {
        lines: Box::new([Line::Call { name: name.into(), arguments: args.into() }]),
        finish: Finish::ToolCalls,
        tokens: fixtures::CALL_TOKENS,
    }
}
fn provider(renew: bool) -> skein_fake_peers::llm::Peer {
    let grant = |kind| json!({"connector":fixtures::FORGE_CONNECTOR,"kind":kind,"segments":["forge",fixtures::FORGE.to_string()],"terminal":"open","last":"acme"});
    let delegate = json!({"batch":[{
        "executor":{"kind":"procedure","connector":fixtures::FORGE_CONNECTOR,"code":fixtures::PROCEDURE_CODE},
        "spec":{"words":"@edit Fix","parameters":[{"name":fixtures::PARAMETER_NAMES[0],"kind":"resource","connector":fixtures::FORGE_CONNECTOR,"resource":fixtures::REPOSITORY_TAG},{"name":fixtures::PARAMETER_NAMES[1],"kind":"bytes","value":"main"}]},
        "contract":{"kind":"change","connector":fixtures::FORGE_CONNECTOR,"change_kind":fixtures::CHANGE_KIND,"words":fixtures::CHANGE_WORDS},
        "authority":{"tools":fixtures::PRODUCER_TOOLS,"grants":temper_engine_domain_forge::CHANGE_KINDS.map(grant),"delegation":{"kinds":[{"kind":"agent","number":fixtures::AGENT_KIND}],"tasks":fixtures::PRODUCER_TASKS,"depth":fixtures::PRODUCER_DEPTH},"budget":{"spend":fixtures::PRODUCER_SPEND},"notes":fixtures::PRODUCER_NOTES}
    }]});
    let mut edits =
        vec![call(b"read", br#"{"path":"README.md"}"#), call(b"shell", br#"{"command":"echo fixed > conflict.txt"}"#)];
    if renew {
        edits.push(call(b"wait", b"{}"));
    }
    edits.push(call(
        b"finish",
        br#"{"form":"change","text":"","fields":{"title":"Fix the file","body":"The file is fixed."}}"#,
    ));
    fixtures::scripted_provider(Box::new([
        Script {
            cue: b"@plan".as_slice().into(),
            turns: Box::new([
                call(b"delegate", &serde_json::to_vec(&delegate).expect("delegate fixture")),
                call(b"wait", b"{}"),
                call(b"wait", b"{}"),
            ]),
        },
        Script { cue: b"@edit".as_slice().into(), turns: edits.into_boxed_slice() },
    ]))
}
fn commands(caller: &mut Caller, link: Box<[u8]>, app: u64) -> usize {
    let mut sign_in = Command::new(&[b"sign-in", b"-"]);
    sign_in.input = link;
    caller.commands.push_back(sign_in);
    caller.commands.push_back(paste(app, b"secrecy-app"));
    caller.commands.push_back(Command::new(&[b"forge", b"list", b"--until", b"connected:1", b"--json"]));
    caller.commands.push_back(Command::new(&[b"forge", b"install", b"1", b"--key", b"secrecy-install"]));
    caller.commands.push_back(Command::browse(3));
    caller.commands.push_back(Command::new(&[b"forge", b"list", b"--until", b"settled", b"--json"]));
    caller.commands.push_back(Command::new(&[
        b"repository",
        b"adopt",
        b"--project",
        b"1",
        b"--owner",
        b"acme",
        b"--name",
        b"app",
        b"--landing",
        b"main",
        b"--key",
        b"secrecy-adopt",
        b"--json",
    ]));
    caller.commands.push_back(Command::new(&[
        b"account",
        b"add-key",
        b"1",
        b"/setup/llm.key",
        b"--deployment",
        b"--key",
        b"secrecy-account",
    ]));
    caller.commands.push_back(Command::new(&[b"account", b"list", b"--until", b"usable"]));
    caller.commands.push_back(Command::new(&[
        b"chat",
        b"start",
        b"--project",
        b"1",
        b"--key",
        b"secrecy-chat",
        b"@plan Fix the adopted repository",
        b"--json",
    ]));
    10
}
struct Judge {
    controls: skein_world::Controls,
    store: Rc<RefCell<Store>>,
    forge: Rc<RefCell<Forge>>,
    app: u64,
    renew: bool,
    calls: usize,
    answers: usize,
    handed: bool,
    initial: bool,
    refreshed: bool,
    browsing: bool,
    stopping: bool,
    read: bool,
    pushed: bool,
    landed: bool,
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        let (first, other) = procs.split_at_mut(1);
        let Proc::Service(service) = &mut first[0] else { panic!("service first") };
        let Proc::Calls(caller) = &mut other[0] else { panic!("caller second") };
        if !self.handed
            && let Some(link) = service.service.links().pop()
        {
            self.calls = commands(caller, link, self.app);
            self.handed = true;
        }
        for (index, answer) in caller.answers.iter().enumerate().skip(self.answers) {
            assert!(
                answer.exit == Exit::Code(0),
                "run call {index} succeeds: {} {}",
                observer::public_output(&answer.output),
                observer::public_output(&answer.error)
            );
            observer::artifact("ctl stdout", &answer.output);
            observer::artifact("ctl stderr", &answer.error);
        }
        self.answers = caller.answers.len();
        if !self.handed || self.answers != self.calls || self.stopping {
            return;
        }
        let peer = other
            .iter()
            .find_map(|proc| if let Proc::Forge(peer) = proc { Some(peer) } else { None })
            .expect("GitHub peer");
        let minted = peer.github_layer().expect("GitHub").installations().minted();
        let git = peer.git_calls().expect("git observation");
        if !self.initial && self.read {
            assert_eq!(minted, 2, "both initial scopes were issued");
            assert_eq!(observer::tokens() as u64, minted, "every initial token byte was observed");
            checkpoint(service, &self.store.borrow());
            self.initial = true;
        }
        if self.renew && self.initial && !self.refreshed && minted >= 4 && observer::tokens() as u64 == minted {
            assert!(now >= Time::from_nanos(5_000_000_000), "the component's renewal timer fired");
            assert_eq!(observer::tokens() as u64, minted, "every refreshed token byte was observed");
            checkpoint(service, &self.store.borrow());
            let Proc::Calls(caller) = &mut other[0] else { unreachable!() };
            caller.commands.push_back(Command::new(&[
                b"task",
                b"message",
                b"3",
                b"Continue after renewal",
                b"--key",
                b"secrecy-continue",
            ]));
            self.calls += 1;
            self.refreshed = true;
            return;
        }
        if self.initial && self.pushed && self.landed && (!self.renew || self.refreshed) && !self.browsing && self.store.borrow().rows.values().any(|row| matches!(row, temper_engine_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Ended(task))) if task.number == 2)) {
            assert_eq!(observer::tokens() as u64, minted, "the observer holds every issued token");
            assert!(git.iter().any(|call| call.writes), "a real git delivery used a token");
            assert!(
                git.iter().any(|call| !call.writes && call.minted.is_some_and(|minted| minted.mint <= 2)),
                "initial credential exercised checkout"
            );
            for request in git {
                let identity = request.minted.expect("authenticated git request");
                assert_eq!(identity.installation, 1);
                assert!(request.status == 200 || (!request.writes && request.status == 404));
                if self.renew && request.writes {
                    assert!(identity.mint > 2, "delivery uses the refreshed credential");
                }
            }
            checkpoint(service, &self.store.borrow());
            for proc in other.iter_mut() {
                if let Proc::Client(person) = proc {
                    person.saved();
                }
            }
            self.browsing = true;
        }
        if self.browsing && other.iter().any(|proc| matches!(proc, Proc::Client(person) if person.finished)) {
            checkpoint(service, &self.store.borrow());
            let Proc::Calls(caller) = &mut other[0] else { unreachable!() };
            caller.closed = true;
            self.controls.signal(0, skein_io::kernel::ServiceSignal::Terminate);
            for proc in other {
                referee::shutdown_peer(proc);
            }
            self.stopping = true;
        }
    }
    fn observe(&mut self, _now: Time, procs: &[Proc]) {
        for proc in procs {
            if let Proc::Peer(peer) = proc {
                observer::context(peer);
                self.read |= peer.observations().iter().any(|seen| matches!(seen, skein_fake_peers::llm::Observation::Query { query, .. } if query.messages.iter().flat_map(|message| message.parts.iter()).any(|part| matches!(part, skein_fake_llm_domain::api::Part::ToolOutput { output, is_error: false, .. } if output.windows(b"Scope fixture".len()).any(|part| part == b"Scope fixture")))));
            }
        }
        let mut forge = self.forge.borrow_mut();
        while let Some(seen) = forge.domain.pop_observation() {
            if let forge::Observation::Merged { ref repository, ref base, commit, .. } = seen {
                assert!(
                    repository.as_ref() == b"acme/app" && base.as_ref() == b"main",
                    "the installed change lands on main"
                );
                assert!(
                    forge
                        .domain
                        .object(commit)
                        .expect("merge object")
                        .tree
                        .iter()
                        .any(|(path, bytes)| path.as_ref() == b"conflict.txt" && bytes.as_ref() == b"fixed\n"),
                    "the landed tree contains the delivered edit"
                );
                self.landed = true;
            }
            if let forge::Observation::Moved { repository, branch, to, .. } = seen {
                assert!(repository.as_ref() == b"acme/app", "delivery names its installed repository");
                if branch.as_ref() == b"main" {
                    continue;
                }
                self.pushed |= forge
                    .domain
                    .object(to)
                    .expect("delivered object")
                    .tree
                    .iter()
                    .any(|(path, bytes)| path.as_ref() == b"conflict.txt" && bytes.as_ref() == b"fixed\n");
            }
        }
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.stopping).then_some(PATIENCE)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.stopping && now >= PATIENCE).then(|| {
            format!(
                "M15 run: calls {}/{}, read {}, initial {}, refreshed {}, pushed {}, pages {}, landed {}",
                self.answers,
                self.calls,
                self.read,
                self.initial,
                self.refreshed,
                self.pushed,
                self.browsing,
                self.landed
            )
        })
    }
    fn passed(&self) -> bool {
        self.stopping
    }
}
/// Observe an installed token through agent reading, editing and git delivery, optionally across renewal.
#[must_use]
pub fn delivery(renew: bool) -> observer::Counts {
    observer::start();
    github::set_active(true);
    stories::set(Story::Adopt);
    stories::setup_settings(world::Phase::Story);
    let store = Rc::new(RefCell::new(Store::new()));
    world::lower_store(Some(store.clone()));
    let (forge, mut peer) =
        github::peer_with_ci(if renew { TOKEN_LIFE } else { skein_fake_forge_github::installations::TOKEN_LIFE });
    let (app, files) = observer::input(&mut peer);
    let mut disk = crate::machine::fresh();
    let input = disk.lay(&files);
    disk.close(input);
    let caller = disk.reopen_root(2);
    let native = disk.reopen_root(2);
    let mut world = skein_world::World::new_controlled(
        fixtures::SEED,
        skein_sim::Config::calm(),
        skein_world::Memory::Unchecked,
        |controls| Judge {
            controls,
            store,
            forge,
            app,
            renew,
            calls: 0,
            answers: 0,
            handed: false,
            initial: false,
            refreshed: false,
            browsing: false,
            stopping: false,
            read: false,
            pushed: false,
            landed: false,
        },
    )
    .with_machine(Machine(disk));
    world::host_programs(&mut world);
    world.spawn_restartable(world::service_roots(), world::service);
    world.spawn_root(skein_sim::Handle::new(caller.raw()), |root| Proc::Calls(Box::new(Caller::scripted(root))));
    world.spawn(|| Proc::Forge(Box::new(SignalHost::new(peer, None))));
    world.spawn(|| Proc::Peer(Box::new(SignalHost::new(provider(renew), None))));
    world.spawn_root(skein_sim::Handle::new(native.raw()), |root| {
        Proc::Client(Box::new(SignalHost::new(Person::pages(root, pages::task_steps()), None)))
    });
    let mut outcome = world.run();
    assert!(outcome.procs.iter().all(Host::is_empty), "M15 run drains");
    let counts = observer::finish(&mut outcome.machine.0);
    world::lower_store(None);
    github::set_active(false);
    counts
}
