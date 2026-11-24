//! Cold restart evidence through the native person, ctl and the owned Forgejo
//! (temper/docs/design/domain/landing-proof.md, 2, and temper/docs/design/domain/landing.md, 15;
//! temper/docs/design/protocol/forge/engine.md, 1).
use super::{
    ANSWER, ChatJudge, ChatPhase, LANDING, Process, WITHIN, ctl, fake_llm,
    forgejo::{JobBarrier, Throwaway},
    landing_by_turn, landing_scripts, native_settings_setup, person, serve, write_private,
};
use serde_json::{Value, json};
use skein_io::kernel::{Exit, Signal};
use skein_lib::{Duration, Time};
use skein_shell::{Clock, Host};
use skein_world::{Referee, end_to_end::Streams, real};
use std::{cell::Cell, fs, path::Path};
use temper_agent_fixtures::Setting;
use temper_playground::documents::Configuration;
use temper_worker_shell::agent::Prepared;

pub(super) const SETUP_CALLS: usize = super::NATIVE_SETUP_CALLS + 2;

#[derive(Debug)]
pub struct Reads {
    pub landing: Value,
    pub change: Value,
}

#[expect(
    clippy::disallowed_methods,
    reason = "a nested ctl real loop runs on its own thread and joins before the story advances"
)]
fn read(root: &Path, arguments: &[&str]) -> Result<Value, String> {
    Ok({
        // A ctl read owns its real loop on a separate thread.
        let call = std::thread::scope(|scope| {
            scope
                .spawn(|| ctl::alone(root, ctl::Call::new(arguments).in_directory("private").json()))
                .join()
                .expect("ctl read")
        });
        if call.exit != Some(Exit::Code(0)) {
            return Err(format!("{}", String::from_utf8_lossy(&call.stderr)));
        }
        super::check_ctl_json(&call)?
    })
}

fn reads(root: &Path, landing: u64, change: u64) -> Result<Reads, String> {
    Ok({
        Reads {
            landing: read(root, &["landing", "show", &landing.to_string()])?,
            change: read(root, &["change", "show", &change.to_string()])?,
        }
    })
}

/// Initialize the store without queue requirements before seeding the forge record.
#[must_use]
pub fn start(scratch: &super::Scratch) -> Throwaway {
    let forge = Throwaway::start_bare(scratch);
    let forge = forge.with_repository(scratch, 20, landing_by_turn::WORKFLOW);
    forge.detect_manual_merges();
    forge
}

/// Select turn landing for the seeded project, whose policy names stock policy 1.
#[expect(clippy::must_use_candidate, reason = "subprocess worlds use the files; in-process checks retain the figures")]
pub fn configure(root: &Path, repository: u64) -> Configuration {
    configure_at(root, repository, Profile::Turn)
}

#[derive(Clone, Copy)]
pub enum Profile {
    Turn,
    NarrowedBatch,
}

/// The native fixture's preceding configuration and startup copies, supplied as values.
pub struct Inputs {
    pub configuration: Configuration,
    pub agent: Value,
    pub native_settings: Value,
}

/// Outputs of configuration, in the order the subprocess fixture lays them.
pub enum Stage<'a> {
    Landing(&'a [u8]),
    Native(&'a [u8]),
    LocalPlaces(u32),
    Figures(&'a Configuration),
}

pub struct Configured {
    pub configuration: Configuration,
    pub agent: Prepared,
    pub setting: Setting,
}

pub(super) fn configure_at(root: &Path, repository: u64, profile: Profile) -> Configuration {
    let inputs = Inputs {
        configuration: super::limit_settings::configuration(root),
        agent: temper_agent_fixtures::read(root),
        native_settings: super::cluster_turn::read(&root.join("native-landing-settings.json")),
    };
    configure_with(
        inputs,
        repository,
        profile,
        |setting| {
            setting.lay(root);
            temper_agent_fixtures::prepared(root)
        },
        |stage| match stage {
            Stage::Landing(bytes) => stage_landing_with(
                root,
                bytes,
                Path::is_dir,
                write_private,
                temper_real_loop_world::fixtures::seed_settings_file,
            ),
            Stage::Native(bytes) => super::owned::write_landing_settings(root, bytes),
            Stage::LocalPlaces(places) => super::limit_settings::local_places(root, places),
            Stage::Figures(configuration) => super::limit_settings::remember(root, configuration),
        },
    )
    .configuration
}

/// Stage the landing FILE and, when there is a store, seed those same bytes there.
pub fn stage_landing_with(
    root: &Path,
    bytes: &[u8],
    is_directory: impl FnOnce(&Path) -> bool,
    write: impl FnOnce(&Path, &[u8]),
    seed: impl FnOnce(&Path, &[u8]),
) {
    write(&root.join("landing-settings.json"), bytes);
    let state = root.join("state");
    if is_directory(&state) {
        seed(&state, bytes);
    }
}

/// Compose the turn or narrowed-batch fixture through supplied agent and output faces.
pub fn configure_with(
    inputs: Inputs,
    repository: u64,
    profile: Profile,
    mut prepare: impl FnMut(&Setting) -> Prepared,
    mut stage: impl FnMut(Stage<'_>),
) -> Configured {
    let batch = match profile {
        Profile::Turn => 1,
        Profile::NarrowedBatch => 3,
    };
    let mut landing = landing_settings(repository, batch);
    let mut configured = configure_base(inputs, batch, &landing, &mut prepare, &mut stage);
    let value = &mut configured.configuration;
    if matches!(profile, Profile::NarrowedBatch) {
        stage(Stage::LocalPlaces(8));
        // The narrowed batch reports failed ids and accepts a selected probe.
        landing["items"][1]["set"]["repositories"][0]["gates"][0]["report"] =
            json!({"artifact":"failed-tests", "format":"lines"});
        stage(Stage::Landing(&serde_json::to_vec(&landing).expect("landing settings")));
        value["workers"][0]["slots"] = 8.into();
        value.derive_places();
        // Three readiness gathers run beside repository polls and the held latch.
        value.temper_over("forge.client.pending", 16);
        value.temper_over("forge.client.calls", 8);
        value.temper_over("forge_component.connections", 8);
    }
    let agent = serde_json::from_slice(&configured.agent.bytes).expect("prepared agent JSON");
    configured.setting = super::batch_landing::compose_message_room(value, agent, &configured.setting.providers);
    configured.agent = prepare(&configured.setting);
    value.agent_bounds(&configured.agent.agent);
    staging_limits(value);
    stage(Stage::Figures(value));
    configured
}

/// Compose the bare batch bound, before the full story's message-room edit.
pub fn configure_batch_with(
    inputs: Inputs,
    repository: u64,
    batch: u16,
    mut prepare: impl FnMut(&Setting) -> Prepared,
    mut stage: impl FnMut(Stage<'_>),
) -> Configured {
    configure_base(inputs, batch, &landing_settings(repository, batch), &mut prepare, &mut stage)
}

fn landing_settings(repository: u64, batch: u16) -> Value {
    json!({"items":[
        {"setting":{"kind":"project-landing","project":1},"set":{"by":"turn","batch":batch}},
        {"setting":{"kind":"project-gates","project":1},"set":{"repositories":[{"forge":1,"repository":repository,"gates":[{"name":"gate","workflow":"gate.yml","ignored":["docs"],"blocking":true,"base":true}],"tiers":[]}]}}
    ]})
}

const STAGING_LIFETIME: skein_lib::Duration = skein_lib::Duration::from_secs(300);

/// Staging's deployment limits, shared by the owned Forgejo stories.
/// temper/docs/design/protocol/configuration/limits.md.
pub fn staging_limits(value: &mut temper_playground::documents::Configuration) {
    value.temper_over("forge.client.lifetime_ms", STAGING_LIFETIME.as_millis());
    value.temper_over("forge.client.op_bytes", 63_738);
    value.temper_over("forge_component.body_bytes", 127_497);
    value.temper_over("forge.client.answer_bytes", 1_048_576);
    for limits in ["authority.executors", "tasks.executor_kinds"] {
        value.jig_over(limits, 24);
    }
    value.jig_over("tasks.result_bytes", 63_738);
    value.jig_over("tasks.message_bytes", 63_738);
    value.jig_over("tasks.inbox_bytes", 82 * 63_738);
    value.jig_over("core.policy_bytes", 8192);
    value.jig_over("core.call_answer_bytes", 63_740);
    value.jig_over("core.call_answer_bytes", 63_740);

    value.jig_over("tasks.result_bytes", 63_738);
    value.jig_over("tasks.message_bytes", 63_738);
    value.jig_over("hub.outcome_bytes", 63_740);
    value.jig_over("hub.outcome_bytes", 63_740);
    value.jig_over("hub.event_bytes", 63_738);
    value.jig_over("hub.event_bytes", 63_738);
}

fn configure_base(
    inputs: Inputs,
    batch: u16,
    settings: &Value,
    prepare: &mut impl FnMut(&Setting) -> Prepared,
    stage: &mut impl FnMut(Stage<'_>),
) -> Configured {
    let Inputs { configuration: mut config, mut agent, mut native_settings } = inputs;
    stage(Stage::Landing(&serde_json::to_vec(settings).expect("landing settings")));
    // Each member and the train need watched branches
    // (temper/docs/design/protocol/configuration/limits.md, forge.landing_members).
    let members = u32::from(batch);
    config.temper_over("forge.landing_members", members);
    config.temper_over("forge.resources_per_task", members + 1);
    // The turn needs the prepare kind, declared only when its carriers fit
    // (temper/docs/design/domain/worker/jobs.md, kind 1).
    let bound = temper_service::limits::prepare_bound();
    config.jig_over("tasks.spec_bytes", bound.spec);
    config.jig_over("tasks.message_bytes", bound.result);
    config.jig_over("tasks.result_bytes", bound.result);
    config.jig_over("tasks.result_bytes", bound.result);
    config.temper_over("forge.client.op_bytes", bound.result);
    // A JSON string's quotes share the tool payload with its message.
    let tool_payload = bound.result + 2;
    agent["profile"]["declared"]["tool_payload"] = tool_payload.into();
    config.jig_over("hub.outcome_bytes", tool_payload);
    config.jig_over("hub.outcome_bytes", tool_payload);
    config.jig_over("hub.event_bytes", bound.result);
    // Let the call carriers follow the agent's declared payload.
    config.jig_over("core.call_answer_bytes", jig_protocol_config::stock().core.call_answer_bytes);
    super::configuration::turn_bounds::apply(&mut config, super::configuration::turn_bounds::turn_body(&agent));
    let providers = temper_agent_fixtures::providers::take(&mut config).expect("fixture providers");
    let setting = temper_agent_fixtures::compose_setting(&mut config, &agent, &providers);
    let agent = prepare(&setting);
    config.agent_bounds(&agent.agent);
    stage(Stage::Figures(&config));
    native_settings["items"][0]["set"]["gates"] = json!([{"number":1,"blocking":true,"freshness":"clean"}]);
    stage(Stage::Native(&serde_json::to_vec(&native_settings).expect("settings JSON")));
    Configured { configuration: config, agent, setting }
}

/// Leave landing tasks a share of the seeded project's open period (domain/landing.md, 4.2).
pub(super) fn setup(root: &Path) -> Vec<ctl::Step> {
    let agents = super::agents_value(&fs::read(root.join("agents.setting")).expect("agents setting copy"));
    let mut steps = native_settings_setup(root, Some(agents));
    steps.push(ctl::Step::Run(ctl::Call::new(&["project", "show", "1", "--json"]).in_directory("private")));
    steps.push(ctl::Step::Then(Box::new(|calls| {
        let project = ctl::json(calls.last().expect("project read"));
        let period = project["policy"]["period_spend"].as_u64().expect("project period");
        let spend = period / 48 * landing_by_turn::ROLE_PERIOD_IN_K;
        let amount = format!("{}.{:02}", spend / 100, spend % 100);
        ctl::Call::new(&["project", "fund", "1", &amount, "--role", "0", "--key", "turn-role-funds"])
            .in_directory("private")
    })));
    steps
}

pub struct Proof {
    pub reads: Reads,
    pub original: String,
    pub run: u64,
}

struct Judge<'a> {
    failed: Option<String>,
    chat: ChatJudge<'a>,
    root: &'a Path,
    barrier: &'a JobBarrier,
    proof: &'a Cell<Option<Proof>>,
    landed: &'a Cell<Option<Reads>>,
    previous: Option<&'a Proof>,
    retry: bool,
    ready: bool,
    released: bool,
    poll: Time,
    progress: (u32, super::patience::Patience),
}
impl Judge<'_> {
    fn sample(&mut self) -> Result<bool, String> {
        let forge = self.chat.forge.ok_or(String::from("owned forge"))?;
        if let Some(previous) = self.previous
            && !self.retry
        {
            let (landing, change) = self.members()?;
            let current = reads(self.root, landing, change)?;
            if !self.released {
                let gate = &current.landing["members"][0]["gates"][0];
                if current.landing["phase"] != "proving" || gate["proof"]["dispatched"]["run"] != previous.run {
                    return Ok(false);
                }
                let run = forge.repository_observation("acme", "widget", &format!("actions/runs/{}", previous.run));
                if run["status"] != "running" {
                    return Err("the restarted sitting finds the held run".into());
                }
                self.barrier.release(previous.run);
                self.released = true;
            }
            if current.change["step"] != "landed" || current.landing["phase"] != "landed" {
                return Ok(false);
            }
            self.landed.set(Some(current));
            return Ok(true);
        }
        let Some(run) = self.waiting_gate(forge)? else { return Ok(false) };
        let (landing, change) = self.members()?;
        let reads = reads(self.root, landing, change)?;
        let gate = &reads.landing["members"][0]["gates"][0];
        if gate["proof"]["dispatched"]["run"] != run {
            return Ok(false);
        }
        let candidate = reads.landing["members"][0]["candidate"].as_str().ok_or("candidate")?;
        let statuses = forge.repository_observation("acme", "widget", &format!("commits/{}/status", &candidate[..40]));
        if !statuses["statuses"]
            .as_array()
            .ok_or("statuses")?
            .iter()
            .any(|status| status["context"] == "temper/gate/gate" && status["status"] == "pending")
        {
            return Ok(false);
        }
        let original = reads.change["head"].as_str().ok_or("original head")?.get(..40).ok_or("head sha")?.into();
        self.proof.set(Some(Proof { reads, original, run }));
        Ok(true)
    }

    fn waiting_gate(&self, forge: &Throwaway) -> Result<Option<u64>, String> {
        Ok({
            let Some(run) = self.barrier.waiting(self.previous.map(|proof| proof.run)) else {
                return Ok(None);
            };
            let actual = forge.repository_observation("acme", "widget", &format!("actions/runs/{run}"));
            if actual["id"] != run {
                return Err(String::from("marker names the forge's actual run"));
            }
            // Only the dispatched landing gate takes the runner and reaches the barrier.
            {
                let (left, right) = (&(actual["workflow_id"]), &("gate.yml"));
                if left != right {
                    return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
                }
            }
            {
                let (left, right) = (&(actual["status"]), &("running"));
                if left != right {
                    return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
                }
            }
            Some(run)
        })
    }

    fn members(&self) -> Result<(u64, u64), String> {
        Ok({
            if let Some(previous) = self.previous {
                (
                    previous.reads.landing["task"].as_u64().ok_or(String::from("landing task"))?,
                    previous.reads.change["task"].as_u64().ok_or(String::from("change task"))?,
                )
            } else {
                let list = read(self.root, &["landing", "list", "--project", "1"])?;
                let row = list["landings"]
                    .as_array()
                    .ok_or(String::from("landing rows"))?
                    .first()
                    .ok_or(String::from("live landing"))?;
                (
                    row["task"].as_u64().ok_or(String::from("landing task"))?,
                    row["changes"][0].as_u64().ok_or(String::from("member change"))?,
                )
            }
        })
    }
}
impl Judge<'_> {
    fn stalled(&self, procs: &[Process], figures: &skein_world::Figures) -> Result<(), String> {
        let Some(Process::Person(person)) = procs.get(1) else { return Err(String::from("native person")) };
        let setup = procs.iter().find_map(|proc| match proc {
            Process::Calls(calls) => Some(calls.ends().iter().map(|call| call.exit).collect::<Vec<_>>()),
            Process::Binary(_)
            | Process::Stdout(_)
            | Process::Person(_)
            | Process::Peer(_)
            | Process::Issuer(_)
            | Process::Visitor(_) => None,
        });
        eprintln!("stalled change: {}", read(self.root, &["change", "show", "3"])?);
        eprintln!("stalled change task: {}", read(self.root, &["task", "show", "3"])?);
        eprintln!("stalled review task: {}", read(self.root, &["task", "show", "5"])?);
        let list = read(self.root, &["landing", "list", "--project", "1", "--ended"])?;
        eprintln!("stalled landings: {list}");
        for landing in list["landings"].as_array().ok_or(String::from("landing rows"))? {
            let task = landing["task"].as_u64().ok_or(String::from("landing task"))?;
            eprintln!("stalled landing: {}", read(self.root, &["landing", "show", &task.to_string()])?);
            eprintln!("stalled landing task: {}", read(self.root, &["task", "show", &task.to_string()])?);
        }
        eprintln!("stalled tasks: {}", read(self.root, &["task", "list", "--project", "1"])?);
        let forge = self.chat.forge.ok_or(String::from("forge"))?;
        let pull = forge.repository_observation("acme", "widget", "pulls/1");
        eprintln!(
            "stalled member pull #1: head={}, merged={}, merge commit={}",
            pull["head"]["sha"], pull["merged"], pull["merge_commit_sha"]
        );
        let branch = forge.repository_observation("acme", "widget", "branches/temper/2/c3");
        let head = branch["commit"]["id"].as_str().ok_or(String::from("branch head"))?;
        let commit = forge.repository_observation("acme", "widget", &format!("git/commits/{head}"));
        eprintln!(
            "stalled branch commit: {}",
            json!({"sha":head,"parents":commit["parents"],"committer":commit["committer"]})
        );
        let Some(Process::Binary(binary)) = procs.first() else { return Err(String::from("serve")) };
        for line in String::from_utf8_lossy(binary.stderr()).lines() {
            if line.contains("prepar") || line.contains("hold") || line.contains("drift") {
                eprintln!("stalled serve: {line}");
            }
        }
        let runs = forge.repository_observation("acme", "widget", "actions/runs?limit=100");
        let runs: Vec<_> = runs["workflow_runs"]
            .as_array()
            .ok_or(String::from("runs"))?
            .iter()
            .map(|run| json!({"id":run["id"],"workflow":run["workflow_id"],"status":run["status"]}))
            .collect();
        eprintln!("stalled runs: {runs:?}");
        let sitting = if self.previous.is_none() {
            "first"
        } else if self.retry {
            "retry"
        } else {
            "final"
        };
        Err(format!(
            "turn restart stalled ({sitting}): native step {}, finished={}, setup={setup:?}, forge={}; {figures}",
            person.steps_seen(),
            person.finished,
            self.chat.forge.ok_or(String::from("forge"))?.failure_state()
        ))
    }
}
impl Referee<Process> for Judge<'_> {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        if self.failed.is_some() {
            return;
        }
        let result = (|| -> Result<(), String> {
            // ChatJudge hands the settings session to the native person. Its usual
            // stop waits for a merge; this story stops at its own durable read.
            if matches!(self.chat.phase, ChatPhase::SettingUp) {
                let Some(Process::Calls(calls)) = procs.get(2) else { return Err(String::from("turn settings setup")) };
                if calls.ends().len() < SETUP_CALLS {
                    self.poll = now.saturating_add(Duration::from_secs(1));
                    return Ok(());
                }
                self.chat.act(now, procs);
                if self.chat.failed.is_some() {
                    return Ok(());
                }
            }
            let Some(Process::Person(person)) = procs.get(1) else { return Err(String::from("native person")) };
            if person.steps_seen() != self.progress.0 {
                let Some(Process::Binary(binary)) = procs.first() else { return Err(String::from("serve")) };
                self.progress = (
                    person.steps_seen(),
                    super::patience::Patience::watching("the restarted turn's progress", WITHIN, binary),
                );
            }
            if !self.chat.stopping()
                && let Some(figures) = self.progress.1.failed()
            {
                self.stalled(procs, &figures)?;
            }
            if !self.chat.stopping() && now >= self.poll {
                self.poll = now.saturating_add(Duration::from_secs(1));
                // A listener may draw another port on restart. Let the native
                // client keep that address before ctl loads its client directory.
                let settled = person.finished && person.client.as_ref().is_some_and(Host::is_empty);
                if self.previous.is_some() && !settled {
                    return Ok(());
                }
                if !self.ready {
                    self.ready = self.sample()?;
                }
                if self.ready && settled {
                    let Some(Process::Binary(binary)) = procs.first_mut() else { return Err(String::from("serve")) };
                    binary.signal(Signal::Terminate);
                    for proc in procs {
                        if let Process::Peer(peer) = proc {
                            peer.shutdown();
                        }
                    }
                    self.chat.phase = ChatPhase::Stopping;
                }
            }

            Ok(())
        })();
        if let Err(why) = result {
            self.failed = Some(why);
        }
    }
    fn watch(&mut self, now: Time, procs: &[Process]) -> Result<(), String> {
        if let Some(why) = self.failed.take() {
            return Err(why);
        }
        self.chat.watch(now, procs)
    }
    fn next_deadline(&self) -> Option<Time> {
        if self.chat.stopping() {
            self.chat.next_deadline()
        } else {
            // This referee drives its own polls; the chat's merge poll is not advanced here.
            Some(self.progress.1.until().map_or(self.poll, |until| until.min(self.poll)))
        }
    }
    fn overdue(&self, now: Time) -> Option<String> {
        self.chat.overdue(now)
    }
    fn passed(&self) -> bool {
        if self.failed.is_some() {
            return false;
        }
        self.chat.passed()
    }
}

/// Each call starts a new shipped serve process and drains it and every host.
#[must_use]
pub fn run(
    root: &Path,
    forge: &Throwaway,
    barrier: &JobBarrier,
    previous: Option<&Proof>,
) -> (Option<Proof>, Option<Reads>) {
    run_to(root, forge, barrier, previous, false)
}

/// A cancelled run is redispatched by the policy; stop inside that retry's job.
#[must_use]
pub fn retry(root: &Path, forge: &Throwaway, barrier: &JobBarrier, previous: &Proof) -> Proof {
    run_to(root, forge, barrier, Some(previous), true).0.expect("stopped inside the retry")
}

fn run_to(
    root: &Path,
    forge: &Throwaway,
    barrier: &JobBarrier,
    previous: Option<&Proof>,
    retry: bool,
) -> (Option<Proof>, Option<Reads>) {
    let first = previous.is_none();
    let peer = fake_llm(landing_scripts(ANSWER, forge));
    peer.configure(root);
    let script = if first { person::adopt_and_start_change(forge) } else { person::found() };
    let clock = Clock::new();
    let mut binary = serve(root);
    let streams = binary.take_streams().expect("serve pipes");
    let mut fds = streams.descriptors();
    let Streams::Pipes { input, output } = streams else { panic!("pipes") };
    let person_root = skein_shell::open_root(root).expect("native root");
    let proof = Cell::new(None);
    let landed = Cell::new(None);
    let progress = super::patience::Patience::watching("the restarted turn's progress", WITHIN, &binary);
    let mut world = real::World::new(Judge {
        failed: None,
        chat: ChatJudge {
            failed: None,
            phase: if first { ChatPhase::SettingUp } else { ChatPhase::Running },
            deadline: super::patience::Patience::watching("the restarted turn's landing", LANDING, &binary),
            forge: Some(forge),
            poll: clock.now().now,
            merged: false,
        },
        root,
        barrier,
        proof: &proof,
        landed: &landed,
        previous,
        retry,
        ready: false,
        released: false,
        poll: clock.now().now,
        progress: (0, progress),
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    if first {
        world.spawn_with_fds(vec![person_root], || {
            Process::Person(Box::new(person::Person::waiting_for_ctl_with_key(person_root, script)))
        });
        let ctl_root = skein_shell::open_root(root).expect("ctl root");
        fds.push(ctl_root);
        world.spawn_with_fds(fds, || {
            Process::Calls(Box::new(ctl::Calls::new(ctl_root, setup(root), Some(&streams), None)))
        });
    } else {
        fds.push(person_root);
        world
            .spawn_with_fds(fds, || Process::Person(Box::new(person::Person::new(person_root, input, output, script))));
    }
    peer.spawn(&mut world);
    let outcome = world.run(&clock, LANDING.saturating_add(Duration::from_secs(10)));
    assert!(outcome.procs.iter().all(Host::is_empty), "all real io settles before restarting");
    let Process::Binary(binary) = &outcome.procs[0] else { panic!("serve") };
    assert_eq!(binary.exit_status(), Some(Exit::Code(0)), "{}", String::from_utf8_lossy(binary.stderr()));
    (proof.take(), landed.take())
}
