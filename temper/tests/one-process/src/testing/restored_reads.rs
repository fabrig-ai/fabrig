//! Read the turn-landing setup on both sides of a settled stop (temper/docs/design/testing/tiers.md, 1.5).
use crate::{
    ctl::{self, Caller, Command},
    fixtures,
    world::HarnessProcess,
};
use serde_json::{Value, json};
use skein_fake_forge_domain as forge;
use skein_fake_peers::forge::{Forge, Peer};
use skein_io::kernel::{Exit, ServiceSignal};
use skein_lib::{Secret, Time, Wall};
use skein_shell::Host;
use skein_world::{Controls, Referee};
use std::{cell::RefCell, rc::Rc};
use temper_playground::{
    Links, documents, fakes,
    machine::{self, Machine, SignalHost},
    programs::{self, Harness},
};

const LANDING_SETTINGS: &[u8] =
    include_bytes!("../../../../docs/development/qa/setups/turn-landing/files/landing.json");
const SETUP: &[&str] = &[
    "project make Hello --key qa-hello-project",
    "forge connect forgejo-admin /setup/admin.token --first work/hello --project Hello \
     --api 127.0.0.1:3000 --git http://127.0.0.1:3000 --git-program /usr/bin/git --key qa-hello",
    "forge list --until made:1 --json",
    "settings apply /setup/landing.json --against 1 --key qa-turn-landing --json",
];
const READS: &[&str] = &[
    "project show Hello --json",
    "forge list --until connected:1 --json",
    "forge repositories 1 --json",
    "landing list --project Hello --ended --json",
    "settings show project-landing 1 --revision 2 --json",
    "settings show project-gates 1 --revision 2 --json",
];

struct Restore;
impl Harness for Restore {
    type Service = ();
    type Process = HarnessProcess;
    fn agent_profile() -> smith_agent_service::profile::Profile {
        let mut profile = documents::agent_profile();
        profile.declared.tool_payload = temper_service::limits::prepare_bound().result + 2;
        profile
    }
    fn agent_settings() -> temper_service::settings::Agent {
        let mut agent = documents::agent_for(&Self::agent_profile());
        agent.bounds.message_bytes = temper_service::limits::prepare_bound().result;
        agent
    }
}
type Proc = programs::Process<Restore>;

fn fake_forge(image: Option<&[u8]>) -> (temper_fake_git_hosted::Typed, Peer) {
    let mut config = documents::hosted_git::config();
    config.limits.calls = 4;
    config.limits.page_size = 8;
    let domain = forge::Domain::new(&config, fixtures::SEED);
    let shared = Rc::new(RefCell::new(Forge::new(domain, config)));
    let mut table = fakes::table();
    table.repositories.clear();
    let transport = temper_fake_git_hosted::Typed::new(shared.clone(), &table);
    let mut peer =
        Peer::new(fixtures::FORGE, fakes::peer_limits(), table, fakes::forge_limits(), shared, fixtures::SEED)
            .expect("managed fake Forgejo");
    if let Some(bytes) = image {
        peer.restore(bytes).expect("settled Forgejo image");
    } else {
        let setup = peer.setup_mut();
        setup.account(b"temper", false, b"temper:bot");
        let admin = setup.account(b"stranger", true, b"Site admin");
        setup.token(
            admin,
            b"admin",
            &[b"write:admin", b"read:user", b"write:organization", b"write:repository"],
            Secret::new(fakes::ADMIN_TOKEN.into()),
        );
    }
    (transport, peer)
}

struct Judge {
    controls: Controls,
    opening: Opening,
    queued: bool,
    checked: usize,
    stopping: bool,
    deadline: Time,
    finding: Option<String>,
}
impl Judge {
    fn step(&mut self, now: Time, procs: &mut [Proc]) -> Result<(), String> {
        let [Proc::Service(service), Proc::Other(HarnessProcess::Calls(caller)), Proc::Forge(peer), ..] = procs else {
            return Err("expected service, ctl caller and Forgejo peer".into());
        };
        service.service.notes().clear();
        if self.stopping {
            return Ok(());
        }
        let setup = self.opening == Opening::Make;
        let calls = if setup { SETUP } else { READS };
        let sign_in = usize::from(setup);
        if !self.queued
            && let Some(link) = service.service.links().pop()
        {
            if setup {
                let mut command = Command::new(&[b"sign-in", b"-"]);
                command.input = link;
                caller.commands.push_back(command);
            }
            for call in calls {
                let words = call.split_ascii_whitespace().map(str::as_bytes).collect::<Vec<_>>();
                caller.commands.push_back(Command::new(&words));
            }
            self.queued = true;
        }
        for answer in &caller.answers[self.checked..] {
            if answer.exit != Exit::Code(0) {
                let call = self.checked.checked_sub(sign_in).and_then(|index| calls.get(index)).copied();
                let call = call.unwrap_or("sign-in");
                return Err(format!(
                    "{:?}, ctl {call}: {:?}: {} {}",
                    self.opening,
                    answer.exit,
                    String::from_utf8_lossy(&answer.output),
                    String::from_utf8_lossy(&answer.error),
                ));
            }
            self.checked += 1;
            self.deadline = now.saturating_add(temper_client_shell_cli::limits::DEADLINE);
        }
        if self.checked == calls.len() + sign_in {
            caller.closed = true;
            self.controls.signal(0, ServiceSignal::Terminate);
            peer.shutdown();
            self.stopping = true;
        }
        Ok(())
    }
}
impl Referee<Proc> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        if self.finding.is_none() {
            self.finding = self.step(now, procs).err();
        }
    }
    fn watch(&mut self, _: Time, procs: &[Proc]) -> Result<(), String> {
        if let Some(finding) = &self.finding {
            return Err(finding.clone());
        }
        if let Proc::Service(service) = &procs[0]
            && let Some(end) = service.service.end()
            && (!self.stopping || *end != temper_service::End::Engine(temper_service::EngineEnd::Drained))
        {
            return Err(format!("service ended at call {}: {end:?}", self.checked));
        }
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        Some(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now >= self.deadline).then(|| format!("deployment read or stop did not settle after {} calls", self.checked))
    }
    fn passed(&self) -> bool {
        self.stopping && self.finding.is_none()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Opening {
    Make,
    Baseline,
    Reopen,
}

struct Place {
    disk: skein_fake_machine::Machine,
    forge: Option<Box<[u8]>>,
    clock: skein_sim::Config,
}
impl Place {
    fn start(&mut self, deployment: Rc<documents::Configuration>, opening: Opening) -> Vec<Value> {
        let setup = opening == Opening::Make;
        let root = self.disk.reopen_root(2);
        let machine = Machine::new(std::mem::replace(&mut self.disk, machine::fresh()));
        let (transport, peer) = fake_forge(self.forge.as_deref());
        let parameters = programs::Parameters {
            deployment,
            seed: fixtures::SEED,
            links: Links::Keep,
            forge: transport,
            route: Some(fakes::forge_limits().git()),
            disk: machine.0.clone(),
        };
        let outcome = programs::with(parameters, || {
            let mut world = skein_world::World::new_controlled(
                fixtures::SEED,
                self.clock,
                skein_world::Memory::Unchecked,
                |controls| Judge {
                    controls,
                    opening,
                    queued: false,
                    checked: 0,
                    stopping: false,
                    deadline: Time::ZERO.saturating_add(temper_client_shell_cli::limits::DEADLINE),
                    finding: None,
                },
            )
            .with_machine(machine);
            for (program, roots) in programs::programs::<Restore>() {
                world.host_roots(program, roots);
            }
            // A new ctl process draws a fresh envelope key for the repositories read.
            // Keep the person's directory, not the earlier client's random stream.
            let caller = if opening == Opening::Reopen {
                ctl::program(|spawn, inherited| {
                    Proc::Other(HarnessProcess::Ctl(Box::new(ctl::host_with_seed(
                        spawn,
                        inherited,
                        fixtures::SEED + 1,
                    ))))
                })
            } else {
                ctl::program(|spawn, inherited| Proc::Other(HarnessProcess::Ctl(Box::new(ctl::host(spawn, inherited)))))
            };
            world.host_roots(caller, ctl::roots);
            world.spawn_restartable(programs::service_roots(), programs::service::<Restore>);
            world.spawn_root(skein_sim::Handle::new(root.raw()), |root| {
                Proc::Other(HarnessProcess::Calls(Box::new(Caller::scripted(root))))
            });
            world.spawn(|| Proc::Forge(Box::new(SignalHost::new(peer, None))));
            world.run()
        });
        assert!(outcome.procs.iter().all(Host::is_empty), "deployment and its peers drained");
        let [Proc::Service(service), Proc::Other(HarnessProcess::Calls(caller)), Proc::Forge(peer), ..] =
            outcome.procs.as_slice()
        else {
            panic!("settled deployment")
        };
        assert_eq!(service.service.end(), Some(&temper_service::End::Engine(temper_service::EngineEnd::Drained)));
        self.forge = Some(peer.image().expect("settled forge"));
        self.clock.wall = Wall::from_nanos(self.clock.wall.as_nanos() + outcome.end.as_nanos());
        std::mem::swap(&mut *outcome.machine.0.borrow_mut(), &mut self.disk);
        let answers = if setup { &caller.answers[3..] } else { caller.answers.as_slice() };
        answers.iter().map(|answer| serde_json::from_slice(&answer.output).expect("ctl JSON answer")).collect()
    }
}

fn reads(mut answers: Vec<Value>) -> Vec<Value> {
    let [project, connected, repositories, landings, landing, gates] = answers.as_mut_slice() else {
        panic!("six ctl reads: {answers:?}");
    };
    assert_eq!(project["number"], 1);
    assert_eq!(project["cut"], false);
    let [home] = project["resources"].as_array().expect("project resources").as_slice() else {
        panic!("one home repository: {project}");
    };
    assert_eq!(home["role"], "owned");
    assert_eq!(home["home"], true);
    assert_eq!(home["part"]["key"]["path"], json!(["0001", "00000001"]));
    // The digest of a read is not stored project state.
    project.as_object_mut().expect("project object").remove("digest");
    assert_eq!(connected["met"], true);
    let [row] = connected["forges"]["forges"].as_array().expect("forges").as_slice() else {
        panic!("one forge: {connected}");
    };
    assert_eq!(row["number"], 1);
    assert_eq!(row["state"], "connected");
    assert_eq!(row["mode"], "managed");
    assert_eq!(repositories["more"], false);
    assert!(
        repositories["rows"]
            .as_array()
            .expect("repository rows")
            .iter()
            .any(|row| { row["id"] == 1 && row["owner"]["login"] == "work" && row["name"] == "hello" }),
        "work/hello: {repositories}"
    );
    // Compare landing state, not the clock sampled for the read.
    let now = landings.as_object_mut().expect("landing list object").remove("now");
    assert!(now.is_some_and(|value| value.is_u64()), "landing list clock");
    assert_eq!(*landings, json!({"project":1,"turn":null,"turn_since":null,"landings":[]}));
    assert_eq!(landing["setting"], json!({"kind":"project-landing","project":1}));
    assert_eq!(landing["revision"], 2);
    assert_eq!(landing["value"]["by"], "turn");
    assert_eq!(landing["value"]["batch"], 2, "the revised batch size");
    assert_eq!(gates["setting"], json!({"kind":"project-gates","project":1}));
    assert_eq!(gates["revision"], 2);
    let [repository] = gates["value"]["repositories"].as_array().expect("gate repositories").as_slice() else {
        panic!("one repository's gates: {gates}");
    };
    assert_eq!(repository["forge"], 1);
    assert_eq!(repository["repository"], 1);
    let [gate] = repository["gates"].as_array().expect("repository gates").as_slice() else {
        panic!("one blocking gate: {repository}");
    };
    assert_eq!(gate["name"], "gate");
    assert_eq!(gate["workflow"], "gate.yml");
    assert_eq!(gate["blocking"], true);
    answers
}

pub fn restored_deployment_reads() {
    let mut deployment = documents::world_document();
    deployment["world_settings"]["empty_projects"] = true.into();
    // The project selects turn landing, which requires the prepare job's request and result bounds.
    let prepare = temper_service::limits::prepare_bound();
    deployment.jig_over_all([
        ("tasks.spec_bytes", u64::from(prepare.spec)),
        ("tasks.batch_bytes", 32_768),
        ("tasks.result_bytes", u64::from(prepare.result)),
        ("tasks.message_bytes", u64::from(prepare.result)),
        ("tasks.inbox_bytes", 32_768),
        ("hub.outcome_bytes", u64::from(prepare.result + 2)),
    ]);
    deployment.temper_over("forge.client.op_bytes", prepare.result);
    deployment.temper_over("checkout.message_bytes", prepare.result + 2);
    deployment.agent_bounds(&Restore::agent_settings());
    let deployment = Rc::new(deployment);
    let mut disk = machine::fresh();
    machine::setup(&mut disk);
    machine::lay_handover(&mut disk, b"/setup/landing.json", LANDING_SETTINGS);
    let mut place = Place { disk, forge: None, clock: skein_sim::Config::calm() };
    let setup = place.start(deployment.clone(), Opening::Make);
    assert_eq!(setup[0]["met"], true, "first repository adopted: {}", setup[0]);
    assert_eq!(setup[1], json!({"revision": 2}));
    // Kinds 4 and 5 are taken at the next start. Read after that start, stop
    // settled, then read again without signing in or doing any setup again.
    let before = reads(place.start(deployment.clone(), Opening::Baseline));
    let after = reads(place.start(deployment, Opening::Reopen));
    for ((call, before), after) in READS.iter().zip(before).zip(after) {
        assert_eq!(after, before, "ctl {call}: the same document on the next start");
    }
}
