//! Cold restart evidence through the native person, ctl and the owned Forgejo
//! (temper/docs/design/domain/landing.md, 7 and 15;
//! temper/docs/design/protocol/forge/engine.md, 1).
use super::{
    ANSWER, ChatJudge, ChatPhase, Ending, LANDING, Process, WITHIN, ctl, fake_llm,
    forgejo::{JobBarrier, Throwaway},
    landing_by_turn, landing_scripts, native_settings_setup, person, serve, write_private,
};
use serde_json::{Value, json};
use skein_io::kernel::{Exit, Signal};
use skein_lib::{Duration, Time};
use skein_shell::{Clock, Host};
use skein_world::{Referee, end_to_end::Streams, real};
use std::{cell::Cell, fs, path::Path};

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
    let path = scratch.path().join("temper.json");
    let mut config: Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    config["application"]["landing"]["projects"] = json!([]);
    write_private(&path, &serde_json::to_vec(&config).expect("configuration JSON"));
    let forge = forge.with_repository(scratch, 20, landing_by_turn::WORKFLOW);
    forge.detect_manual_merges();
    forge
}

/// Select turn landing for the seeded project, whose policy names stock policy 1.
pub fn configure(root: &Path, repository: u64) {
    configure_batch(root, repository, 1);
    // The prepare job needs the standard engine profile and room for its result.
    let path = root.join("temper.json");
    let mut value: Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    super::batch_landing::message_room(root, &mut value);
    staging_limits(&mut value);
    write_private(&path, &serde_json::to_vec(&value).expect("configuration JSON"));
}

/// The narrowed batch reports failed ids and accepts a selected probe.
pub(super) fn selected_gate(value: &mut Value) {
    let gate = &mut value["application"]["landing"]["projects"][0]["gates"][0]["gates"][0];
    gate["report"] = json!({"artifact":"failed-tests", "format":"lines"});
}

const STAGING_LIFETIME: skein_lib::Duration = skein_lib::Duration::from_secs(300);

/// Staging's deployment limits, shared by the owned Forgejo stories.
/// temper/docs/design/protocol/configuration/limits.md.
pub fn staging_limits(value: &mut Value) {
    value["engine"]["limits"]["profile"] = "standard".into();
    value["application"]["limits"]["profile"] = "standard".into();
    value["application"]["limits"]["overrides"]["checkout.message_bytes"] = 63_740.into();
    value["application"]["limits"]["overrides"]["forge.client.lifetime_ms"] = STAGING_LIFETIME.as_millis().into();
    value["application"]["limits"]["overrides"]["forge.client.op_bytes"] = 63_738.into();
    value["application"]["limits"]["overrides"]["forge_component.body_bytes"] = 127_497.into();
    value["application"]["limits"]["overrides"]["forge.client.answer_bytes"] = 1_048_576.into();
    value["application"]["limits"]["overrides"].as_object_mut().expect("limits").remove("forge.key_entries");
    for limits in ["authority.executors", "tasks.executor_kinds"] {
        value["engine"]["limits"]["overrides"][limits] = 24.into();
    }
    value["engine"]["limits"]["overrides"]["tasks.result_bytes"] = 63_738.into();
    value["engine"]["limits"]["overrides"]["tasks.message_bytes"] = 63_738.into();
    value["engine"]["limits"]["overrides"]["tasks.inbox_bytes"] = (82 * 63_738).into();
    value["engine"]["limits"]["overrides"]["core.policy_bytes"] = 8192.into();
    value["engine"]["limits"]["overrides"]["core.call_answer_bytes"] = 63_740.into();
    value["workers"][0]["limits"]["overrides"]["core.call_answer_bytes"] = 63_740.into();
    value["workers"][0]["limits"]["profile"] = "standard".into();
    value["workers"][0]["limits"]["overrides"]["tasks.result_bytes"] = 63_738.into();
    value["workers"][0]["limits"]["overrides"]["tasks.message_bytes"] = 63_738.into();
    value["engine"]["limits"]["overrides"]["hub.outcome_bytes"] = 63_740.into();
    value["workers"][0]["limits"]["overrides"]["hub.outcome_bytes"] = 63_740.into();
    value["engine"]["limits"]["overrides"]["hub.event_bytes"] = 63_738.into();
    value["workers"][0]["limits"]["overrides"]["hub.event_bytes"] = 63_738.into();
}

/// Share the owned turn setup while selecting the story's batch bound.
pub(super) fn configure_batch(root: &Path, repository: u64, batch: u16) {
    let path = root.join("temper.json");
    let mut config: Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    config["application"]["landing"]["projects"] = json!([{
        "project":1,"by":"turn","batch":batch,
        "gates":[{"forge":1,"repository":repository,
            "gates":[{"name":"gate","workflow":"gate.yml","ignored":["docs"],"blocking":true,"base":true}]}]
    }]);
    // Each member and the train need watched branches
    // (temper/docs/design/protocol/configuration/limits.md, forge.landing_members).
    let members = u32::from(batch);
    config["application"]["limits"]["overrides"]["forge.landing_members"] = members.into();
    config["application"]["limits"]["overrides"]["forge.resources_per_task"] = (members + 1).into();
    // The turn needs the prepare kind, declared only when its carriers fit
    // (temper/docs/design/domain/worker/jobs.md, kind 1).
    let prepare = temper_service::limits::prepare_bound();
    config["engine"]["limits"]["overrides"]["tasks.spec_bytes"] = prepare.spec.into();
    config["engine"]["limits"]["overrides"]["tasks.message_bytes"] = prepare.result.into();
    config["engine"]["limits"]["overrides"]["tasks.result_bytes"] = prepare.result.into();
    config["workers"][0]["limits"]["overrides"]["tasks.result_bytes"] = prepare.result.into();
    config["application"]["limits"]["overrides"]["forge.client.op_bytes"] = prepare.result.into();
    let agent_path = root.join("agent.json");
    let mut agent: Value =
        serde_json::from_slice(&fs::read(&agent_path).expect("agent configuration")).expect("agent JSON");
    // A JSON string's quotes share the tool payload with its message.
    let tool_payload = prepare.result + 2;
    agent["profile"]["declared"]["tool_payload"] = tool_payload.into();
    config["engine"]["limits"]["overrides"]["hub.outcome_bytes"] = tool_payload.into();
    config["workers"][0]["limits"]["overrides"]["hub.outcome_bytes"] = tool_payload.into();
    config["workers"][0]["limits"]["overrides"]["hub.event_bytes"] = prepare.result.into();
    config["application"]["limits"]["overrides"]["checkout.message_bytes"] = tool_payload.into();
    // Let the call carriers follow the agent's declared payload.
    config["engine"]["limits"]["overrides"].as_object_mut().expect("engine overrides").remove("core.call_answer_bytes");
    config["workers"][0]["limits"]["overrides"]
        .as_object_mut()
        .expect("worker overrides")
        .remove("core.call_answer_bytes");
    super::configuration::turn_bounds::apply(&mut config, super::configuration::turn_bounds::turn_body(&agent));
    write_private(&agent_path, &serde_json::to_vec(&agent).expect("agent JSON"));
    write_private(&path, &serde_json::to_vec(&config).expect("configuration JSON"));
    let path = root.join("native-landing-settings.json");
    let mut settings: Value =
        serde_json::from_slice(&fs::read(&path).expect("native landing settings")).expect("settings JSON");
    settings["items"][0]["set"]["gates"] = json!([{"number":1,"blocking":true,"freshness":"clean"}]);
    write_private(&path, &serde_json::to_vec(&settings).expect("settings JSON"));
}

/// Leave landing tasks a share of the seeded project's open period (domain/landing.md, 4.2).
pub(super) fn setup(root: &Path) -> Vec<ctl::Step> {
    let mut steps = native_settings_setup(root);
    steps.push(ctl::Step::Run(ctl::Call::new(&["project", "show", "1", "--json"]).in_directory("private")));
    steps.push(ctl::Step::Then(Box::new(|calls| {
        let project = ctl::json(calls.last().expect("project read"));
        let period = project["policy"]["period_spend"].as_u64().expect("project period");
        let amount = (period / 48 * landing_by_turn::ROLE_PERIOD_IN_K).to_string();
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

/// Configuration is checked without starting the owned forge or runner.
pub mod tests {
    /// A turn's task carriers must admit the prepare kind and its worker's result.
    pub fn turn_configuration_declares_the_prepare_job() {
        let scratch = super::super::Scratch::new("turn_prepare_carriers");
        let api = "127.0.0.1:3000".parse().expect("fixture forge");
        let forge = super::super::Forge::Throwaway {
            forgejo: std::path::Path::new("/fixture/forgejo"),
            api,
            git: format!("http://{api}/"),
        };
        super::super::configuration(scratch.path(), &forge);
        super::configure_batch(scratch.path(), 1, 3);
        let config: serde_json::Value =
            serde_json::from_slice(&std::fs::read(scratch.path().join("temper.json")).expect("configuration"))
                .expect("configuration JSON");
        let engine = temper_real_loop_world::settings::configuration(&config);
        let prepare = temper_service::limits::prepare_bound();
        assert!(
            temper_service::limits::prepare_jobs(&engine.limits).contains(&prepare),
            "the turn declares the prepare job"
        );
        assert!(
            config["workers"][0]["limits"]["overrides"]["tasks.result_bytes"].as_u64().expect("worker result carrier")
                >= u64::from(prepare.result),
            "the worker carries the prepare result"
        );
        scratch.remove();
    }
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
            ending: Ending::Stop,
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
