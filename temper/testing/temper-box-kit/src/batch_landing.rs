//! Dispatch-only probes and merged pull heads on the owned runner
//! (temper/docs/design/testing/process-tier.md; domain/landing.md, 5, 8–10 and 14;
//! domain/landing-search.md, 3–5 and 8).
mod narrowed;
use super::{
    ChatJudge, ChatPhase, Ending, LANDING, Process, ctl,
    forgejo::{JobBarrier, Throwaway},
    landing_scripts, person, turn_restart,
};
use serde_json::{Value, json};
use skein_fake_llm_domain::api::{Finish, Line, Piece, Rule, Rules, Script, Turn};
use skein_io::kernel::{Exit, Signal};
use skein_lib::{Duration, Time};
use skein_shell::{Clock, Host};
use skein_world::{Referee, real};
use std::{cell::RefCell, collections::BTreeMap, fs, path::Path};

// Hello is task 1, the landing chat 2. Its one delegate batch makes these
// siblings before any producer starts. The latch is a report, not a change.
const CHANGES: [u64; 3] = [3, 4, 5];
pub(super) fn call(name: &[u8], value: &Value) -> Turn {
    Turn {
        lines: Box::new([Line::Call {
            name: name.into(),
            arguments: serde_json::to_vec(value).expect("tool JSON").into_boxed_slice(),
        }]),
        finish: Finish::ToolCalls,
        tokens: 20,
    }
}

pub(super) fn hold(name: &[u8]) -> Line {
    Line::Pieces { pieces: Box::new([Piece { text: b"Waiting.".as_slice().into(), hold: Some(name.into()) }]) }
}

fn scripts(root: &Path, forge: &Throwaway) -> Box<[Script]> {
    let value: Value =
        serde_json::from_slice(&fs::read(root.join("temper.json")).expect("configuration")).expect("JSON");
    let engine = temper_real_loop_world::settings::configuration(&value);
    let stock = temper_service::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    scripted_batch(landing_scripts(super::ANSWER, forge).into_vec(), stock.rules.maximum_run_spend)
}

fn scripted_batch(mut shared: Vec<Script>, run: u64) -> Box<[Script]> {
    // Three changes and their latch fit the chat's stock 24-run budget.
    let mut owner = shared.remove(0);
    let Line::Call { arguments, .. } = &owner.turns[0].lines[0] else { panic!("delegate") };
    let delegate: Value = serde_json::from_slice(arguments).expect("delegate JSON");
    let mut batch = Vec::new();
    let mut scripts = Vec::new();
    for number in 1..=3 {
        let words = format!("@batch-code-{number} Add batch-{number}.txt");
        let mut change = delegate["batch"][0].clone();
        change["spec"]["words"] = words.clone().into();
        change["tracked"] = json!({"priority":number});
        change["authority"]["budget"]["spend"] = (4 * run).into();
        batch.push(change);
        let path = format!("batch-{number}.txt");
        let turns = vec![
            call(b"read", &json!({"path":path})),
            call(
                b"write",
                &json!({"path":path,"content":if number == 2 { "red\n".to_owned() } else { format!("Change {number}\n") }}),
            ),
            call(
                b"finish",
                &json!({"form":"change","fields":{
                "title":format!("Batch change {number}"),"body":format!("Add {path}.")}}),
            ),
        ];
        scripts.push(Script {
            cue: format!("## Task\n\n{words}").into_bytes().into_boxed_slice(),
            turns: turns.into_boxed_slice(),
        });
    }
    batch.push(json!({
        "executor":{"kind":"agent","charter":1},
        "spec":{"words":"@batch-latch Release the three ready changes","parameters":[]},
        "contract":{"kind":"report","words":32},
        "authority":{"tools":0,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},
            "budget":{"spend":run},"notes":0}
    }));
    owner.turns[0] = call(b"delegate", &json!({"batch":batch}));
    owner.turns = [
        owner.turns.as_ref(),
        &[Turn {
            lines: Box::new([Line::Text { text: b"Read the arrived results before waiting again.".as_slice().into() }]),
            finish: Finish::Stop,
            tokens: 20,
        }],
    ]
    .concat()
    .into_boxed_slice();
    // The latch and partial change results wake the owner before the batch ends.
    // Its fresh-input rules wait again until the task has no delegates left.
    scripts.push(owner);
    scripts.push(Script {
        cue: b"## Task\n\n@batch-latch".as_slice().into(),
        turns: Box::new([Turn {
            lines: Box::new([
                hold(b"batch-ready"),
                Line::Call {
                    name: b"finish".as_slice().into(),
                    arguments: br#"{"form":"report","text":"Ready together","fields":{}}"#.as_slice().into(),
                },
            ]),
            finish: Finish::ToolCalls,
            tokens: 20,
        }]),
    });
    let review = shared
        .iter_mut()
        .find(|script| script.cue.as_ref() == b"Review this change at the named head")
        .expect("shared review");
    review.turns[0].lines = [vec![hold(b"batch-reviews")], review.turns[0].lines.to_vec()].concat().into_boxed_slice();
    // The shared coder is replaced by disjoint files, so every head merges cleanly.
    shared.retain(|script| !script.cue.starts_with(b"## Task\n\n@coding"));
    scripts.extend(shared);
    narrowed::hold_repair(&mut scripts);
    scripts.into_boxed_slice()
}

/// The turn and runner-cancellation stories use the same owned forge, policy,
/// producer and funding setup. The batch stories also hold concurrent model turns.
pub fn configure(root: &Path, repository: u64) {
    turn_restart::configure_batch(root, repository, 3);
    let path = root.join("temper.json");
    let mut value: Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    value["workers"][0]["slots"] = 8.into();
    for name in ["checkout.workspaces", "git_component.operations"] {
        value["application"]["limits"]["overrides"][name] = 8.into();
    }
    // Three readiness gathers run beside repository polls and the held latch.
    let limits = &mut value["application"]["limits"]["overrides"];
    limits["forge.client.pending"] = 16.into();
    limits["forge.client.calls"] = 8.into();
    limits["forge_component.connections"] = 8.into();
    message_room(root, &mut value);
    turn_restart::staging_limits(&mut value);
    super::write_private(&path, &serde_json::to_vec(&value).expect("configuration JSON"));
}

/// Hold the prepare job and the batch call under the standard engine profile.
pub fn message_room(root: &Path, value: &mut Value) {
    // The one model call carries three independent change authorities and a latch.
    let agent_path = root.join("agent.json");
    let mut agent: Value = serde_json::from_slice(&fs::read(&agent_path).expect("agent configuration")).expect("JSON");
    agent["profile"]["declared"]["tool_payload"] = 63_740.into();
    for endpoint in agent["endpoints"].as_array_mut().expect("agent endpoints") {
        endpoint["limits"]["output"] = 48_000.into();
    }
    // The standard store holds the prepare job; keep only the story's engine bounds.
    let prepare = temper_service::limits::prepare_bound();
    value["engine"]["limits"] = json!({"profile":"standard", "overrides":{
        "core.run_bytes":32768,
        "core.call_records":8,
        "authority.projects":3,
        "tasks.spec_bytes":prepare.spec,
        "tasks.result_bytes":prepare.result,
        "tasks.message_bytes":8190,
        "people.words":4096,
        "tasks.inbox_bytes":65536,
        "tasks.run_inbox":8,
        "core.call_answer_bytes":8192,
        "fleet.slots":value["workers"][0]["slots"],
        "hub.inbox":8,
        "hub.outcome_bytes":8192
    }});
    // Both ends declare the same assignment and job carriers at opening.
    value["workers"][0]["limits"] = value["engine"]["limits"].clone();
    let slots = value["workers"][0]["slots"].clone();
    let limits = &mut value["workers"][0]["limits"]["overrides"];
    limits["hub.accounts"] = 4.into();
    limits["hub.slots"] = slots;
    limits["hub.event_bytes"] = 8190.into();
    limits["hub.outcome_bytes"] = 8192.into();
    value["application"]["limits"]["overrides"]["checkout.message_bytes"] = 8192.into();
    value["application"]["limits"]["overrides"]["forge.client.op_bytes"] = 4096.into();
    // Turn read-back observes Forgejo's asynchronous manual-merge mark.
    // Use the shipped paces, within this story's wall bound.
    let overrides = value["application"]["limits"]["overrides"].as_object_mut().expect("application overrides");
    for name in ["forge.client.poll_ms", "forge.client.poll_max_ms", "forge.client.slow_ms"] {
        overrides.remove(name);
    }
    // A heard delivery's fallback also bounds prepare retries and merge read-back.
    overrides.insert("forge.client.fallback_ms".into(), 5000.into());
    turn_restart::staging_limits(value);
    super::configuration::turn_bounds::apply(value, super::configuration::turn_bounds::turn_body(&agent));
    // A task's retained inbox also crosses the record load edge.
    for name in ["root.loads.bytes", "root.loads.reply_bytes"] {
        let current = value["application"]["limits"]["overrides"][name].as_u64().expect("load bound");
        value["application"]["limits"]["overrides"][name] = current.max(8_388_608).into();
    }
    value["application"]["limits"]["overrides"]["root.loads.read_bytes"] = 16_777_216.into();
    value["application"]["limits"]["overrides"]["root.journal.transcript_bytes"] = 8_388_608.into();
    super::write_private(&agent_path, &serde_json::to_vec(&agent).expect("agent JSON"));
}

fn get(forge: &Throwaway, route: &str) -> Value {
    forge.repository_observation("acme", "widget", route)
}
fn sha(value: &Value) -> Result<&str, String> {
    Ok({ value.as_str().ok_or(String::from("commit"))?.get(..40).ok_or(String::from("commit"))? })
}
fn members(panel: &Value) -> Result<&[Value], String> {
    Ok({ panel["members"].as_array().ok_or(String::from("members"))? })
}
fn candidate(panel: &Value) -> Result<&str, String> {
    Ok({ sha(&members(panel)?.last().ok_or(String::from("top"))?["candidate"])? })
}
fn member_branch(task: u64, number: usize) -> String {
    if number == 1 { format!("temper/{task}/l{task}") } else { format!("temper/{task}/l{task}-{number}") }
}
fn gate_runs(forge: &Throwaway) -> Result<Vec<Value>, String> {
    Ok({
        let runs = get(forge, "actions/runs?limit=100");
        if runs["total_count"].as_u64()
            != Some(
                u64::try_from(runs["workflow_runs"].as_array().ok_or(String::from("runs"))?.len())
                    .map_err(|error| format!("run count: {error:?}"))?,
            )
        {
            return Err(String::from("one bounded page holds every run"));
        }
        runs["workflow_runs"].as_array().ok_or(String::from("runs"))?.clone()
    })
}

/// Fixture admission is checked without starting the owned forge or a process story.
pub mod tests {
    pub fn gate_receipt_waits_for_an_unpublished_status_context() {
        use serde_json::json;
        assert_eq!(super::narrowed::gate_status(&json!({"statuses":null}), "failure"), Ok(false));
        assert_eq!(
            super::narrowed::gate_status(
                &json!({"statuses":[{"context":"temper/gate/gate","status":"pending"}]}),
                "failure"
            ),
            Ok(false)
        );
        assert_eq!(
            super::narrowed::gate_status(
                &json!({"statuses":[{"context":"temper/gate/gate","status":"failure"}]}),
                "failure"
            ),
            Ok(true)
        );
        assert!(super::narrowed::gate_status(&json!({"statuses":"malformed"}), "failure").is_err());
    }

    pub fn the_scripted_batch_fits_its_budget_and_call() {
        let scratch = super::super::Scratch::new("batch-admission");
        let api = "127.0.0.1:3000".parse().expect("fixture forge");
        let forge = super::super::Forge::Throwaway {
            forgejo: std::path::Path::new("/fixture/forgejo"),
            api,
            git: format!("http://{api}/"),
        };
        super::super::configuration(scratch.path(), &forge);
        super::configure(scratch.path(), 1);
        let value = serde_json::from_slice(&std::fs::read(scratch.path().join("temper.json")).expect("configuration"))
            .expect("JSON");
        let engine = temper_real_loop_world::settings::configuration(&value);
        let stock = temper_service::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
        let scripts = super::scripted_batch(
            super::super::forgejo::native_scripts(b"hello", api, 1, b"README").into_vec(),
            stock.rules.maximum_run_spend,
        );
        let owner = scripts.iter().find(|script| script.cue.as_ref() == b"@forge_chat").expect("owner");
        let super::Line::Call { arguments, .. } = &owner.turns[0].lines[0] else { panic!("delegate") };
        let delegate: serde_json::Value = serde_json::from_slice(arguments).expect("delegate JSON");
        let tracked =
            delegate["batch"].as_array().expect("batch").iter().filter(|task| !task["tracked"].is_null()).count();
        assert!(
            u32::try_from(tracked).expect("tracked changes") <= engine.limits.core.tasks.project_goals,
            "the independent changes fit the project's goal limit"
        );
        let result_credits = u32::try_from(delegate["batch"].as_array().expect("batch").len())
            .expect("batch members")
            .checked_mul(engine.limits.core.tasks.result_bytes)
            .expect("batch result credits");
        assert!(result_credits <= engine.limits.core.tasks.inbox_bytes, "the batch's result credits fit its inbox");
        let spend: u64 = delegate["batch"]
            .as_array()
            .expect("batch")
            .iter()
            .map(|change| change["authority"]["budget"]["spend"].as_u64().expect("spend"))
            .sum();
        let policy = &stock.policies.get(&1).expect("stock policy").policy;
        let owner = policy.roles.iter().find(|role| role.number == 0).expect("owner role");
        assert!(
            spend + stock.rules.maximum_run_spend <= owner.authority.budget.spend,
            "the chat funds every delegate and retains one run for itself"
        );
        let agent: serde_json::Value =
            serde_json::from_slice(&std::fs::read(scratch.path().join("agent.json")).expect("agent configuration"))
                .expect("JSON");
        assert!(
            u64::try_from(arguments.len()).expect("call length")
                <= agent["profile"]["declared"]["tool_payload"].as_u64().expect("payload"),
            "one admitted model call carries the whole batch"
        );
        scratch.remove();
    }
}

struct Proof {
    panel: Value,
    run: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Reviews,
    Keeping,
    Gates,
    Batch,
    Search(narrowed::Step),
}
// One sample reads each panel through the world's Calls actor. A missing panel
// yields the sample until its response arrives, keeping the fake peer runnable.
const READ_PENDING: &str = "batch observation pending";
#[derive(Default)]
struct Reads {
    cycle: BTreeMap<Vec<String>, Result<Value, String>>,
    last: BTreeMap<Vec<String>, Result<Value, String>>,
    pending: Option<Read>,
}
struct Read {
    arguments: Vec<String>,
    index: Option<usize>,
}
#[derive(Clone, Copy)]
enum PeerStop {
    Waiting,
    Due(Time),
    Stopped,
}
struct Judge<'a> {
    failed: Option<String>,
    chat: ChatJudge<'a>,
    forge: &'a Throwaway,
    barrier: &'a JobBarrier,
    phase: Phase,
    poll: Time,
    first: Option<Proof>,
    second: Option<Proof>,
    probes: Vec<u64>,
    reads: RefCell<Reads>,
    amends: usize,
    observations: std::cell::RefCell<std::collections::BTreeMap<u64, Value>>,
    peer_stop: PeerStop,
}

fn release(procs: &mut [Process], name: &[u8]) -> Result<(), String> {
    for proc in procs {
        if let Process::Peer(peer) = proc {
            peer.release(name).map_err(|error| format!("release scripted turn: {error:?}"))?;
        }
    }

    Ok(())
}
fn calls(procs: &mut [Process]) -> Result<&mut ctl::Calls, String> {
    Ok({
        let Some(Process::Calls(calls)) = procs.get_mut(2) else { return Err(String::from("person's ctl")) };
        calls
    })
}

impl Judge<'_> {
    fn observe(&self, number: u64, value: Value) {
        let mut observations = self.observations.borrow_mut();
        if observations.get(&number) != Some(&value) {
            eprintln!("batch observation {number}: {value}");
            observations.insert(number, value);
        }
    }
    fn report(&self, procs: &[Process]) {
        if let Some(Process::Binary(binary)) = procs.first() {
            eprintln!("batch service exit: {:?}", binary.exit_status());
            eprintln!("batch service: {}", super::without_secrets(binary.stderr()));
        }
        eprintln!(
            "batch phase: {:?}; empty hosts: {:?}",
            self.phase,
            procs.iter().map(Host::is_empty).collect::<Vec<_>>()
        );
        for (arguments, value) in &self.reads.borrow().last {
            eprintln!("batch ctl {arguments:?}: {value:?}");
        }
        for proc in procs {
            if let Process::Peer(peer) = proc {
                for event in peer.observations() {
                    if let skein_fake_peers::llm::Observation::Query { query, .. } = event {
                        let tools: Vec<_> = query
                            .messages
                            .iter()
                            .flat_map(|message| message.parts.iter())
                            .filter_map(|part| {
                                if let skein_fake_llm_domain::api::Part::ToolCall { name, .. } = part {
                                    Some(String::from_utf8_lossy(name))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        eprintln!("batch model query: tools {tools:?}");
                    }
                }
            }
        }
        for proof in [self.first.as_ref(), self.second.as_ref()].into_iter().flatten() {
            eprintln!("batch retained landing: {}", proof.panel);
        }
    }
    fn read(&self, arguments: &[&str]) -> Result<Value, String> {
        let key: Vec<String> = arguments.iter().map(|word| (*word).to_owned()).collect();
        let mut reads = self.reads.borrow_mut();
        if let Some(value) = reads.cycle.get(&key) {
            return value.clone();
        }
        if reads.pending.is_none() {
            reads.pending = Some(Read { arguments: key, index: None });
        }
        Err(READ_PENDING.into())
    }
    fn change(&self, task: u64) -> Result<Value, String> {
        self.read(&["change", "show", &task.to_string()])
    }
    fn landing(&self, task: u64) -> Result<Value, String> {
        self.read(&["landing", "show", &task.to_string()])
    }
    fn landings(&self) -> Result<Value, String> {
        self.read(&["landing", "list", "--project", "1", "--ended"])
    }
    fn drive_reads(&self, procs: &mut [Process]) -> Result<bool, String> {
        let mut reads = self.reads.borrow_mut();
        let Some(pending) = &mut reads.pending else { return Ok(true) };
        let calls = calls(procs)?;
        if let Some(index) = pending.index {
            let Some(ended) = calls.ends().get(index) else { return Ok(false) };
            let value = if ended.exit == Some(Exit::Code(0)) {
                super::check_ctl_json(ended)
            } else {
                Err(format!("ctl {:?}: {}", pending.arguments, String::from_utf8_lossy(&ended.stderr)))
            };
            let key = pending.arguments.clone();
            reads.last.insert(key.clone(), value.clone());
            reads.cycle.insert(key, value);
            reads.pending = None;
            return Ok(true);
        }
        if calls.foreground_idle() {
            let arguments: Vec<&str> = pending.arguments.iter().map(String::as_str).collect();
            pending.index = Some(calls.ends().len());
            calls.push(ctl::Step::Run(ctl::Call::new(&arguments).in_directory("private").json()));
        }
        Ok(false)
    }
    fn reviews_pending(&self) -> Result<bool, String> {
        let owner = self.read(&["task", "show", "2"])?;
        if !owner["hold"].is_null() {
            return Err(format!("batch owner held: {}", owner["hold"]));
        }
        Ok({
            CHANGES.as_slice().iter().copied().try_fold(true, |all, task| {
                if !all {
                    return Ok(false);
                }
                // A producer may not have made its change yet.
                let change = match self.change(task) {
                    Ok(change) => change,
                    Err(why) if why.contains("\"error\":\"unknown\"") => return Ok(false),
                    Err(why) => return Err(why),
                };
                Ok::<_, String>(change["step"] == "gating" && change["gates"][0]["verdict"] == "pending")
            })?
        })
    }
    fn keep_together(&mut self, procs: &mut [Process]) -> Result<(), String> {
        // The same dependency is lifted in one durable completion. This makes
        // three independent units eligible at quiescence, not a joined unit or
        // a race between three separately released reviews (landing.md, 9).
        self.amends = calls(procs)?.ends().len();
        for task in CHANGES.as_slice() {
            calls(procs)?.push(ctl::Step::Run(
                ctl::Call::new(&[
                    "task",
                    "amend",
                    &task.to_string(),
                    "--rev",
                    "0",
                    "--dependency",
                    &6.to_string(),
                    "Become ready with the other two changes",
                    "--key",
                    &format!("batch-keep-{task}"),
                ])
                .in_directory("private"),
            ));
        }
        self.phase = Phase::Keeping;

        Ok(())
    }
    fn proof(&self) -> Result<Option<Proof>, String> {
        let Some(run) = self.barrier.waiting(None) else { return Ok(None) };
        let list = self.landings()?;
        let Some(row) = list["landings"].as_array().ok_or("landing list")?.iter().find(|row| row["phase"] == "proving")
        else {
            return Ok(None);
        };
        let panel = self.landing(row["task"].as_u64().ok_or("landing task")?)?;
        if members(&panel)?.iter().any(|member| member["candidate"].is_null()) {
            return Ok(None);
        }
        self.dispatched(run, candidate(&panel)?)?;
        if gate_runs(self.forge)?.len() != 1 {
            return Err("only the top's dispatch, no push runs".into());
        }
        Ok(Some(Proof { panel, run }))
    }
    fn dispatched(&self, run: u64, prefix: &str) -> Result<(), String> {
        let actual = get(self.forge, &format!("actions/runs/{run}"));
        {
            let (left, right) = (&(actual["id"]), &(run));
            if left != right {
                return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
            }
        }
        {
            let (left, right) = (&(actual["workflow_id"]), &("gate.yml"));
            if left != right {
                return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
            }
        }
        {
            let (left, right) = (&(actual["trigger_event"]), &("workflow_dispatch"));
            if left != right {
                return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
            }
        }
        {
            let (left, right) = (&(actual["trigger_user"]["login"]), &("temper"));
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
        if actual["commit_sha"] != prefix {
            return Err(String::from("proof at the chosen prefix"));
        }

        Ok(())
    }
}

impl Judge<'_> {
    fn stop(&mut self, procs: &mut [Process]) -> Result<(), String> {
        calls(procs)?.release();
        let Some(Process::Binary(binary)) = procs.first_mut() else { return Err(String::from("serve")) };
        binary.signal(Signal::Terminate);
        self.chat.phase = ChatPhase::Stopping;
        eprintln!("batch stopping at {:?}", Clock::new().now().now);

        Ok(())
    }
    fn member_landed(&self, member: &Value, prepared: &Value) -> Result<bool, String> {
        let number = member["pull"]["number"].as_u64().ok_or("member pull")?;
        let pull = get(self.forge, &format!("pulls/{number}"));
        if pull["merged"] != true {
            return Ok(false);
        }
        if pull["merge_commit_sha"] != sha(&prepared["candidate"])? {
            return Err("the member pull is merged at its prefix".into());
        }
        Ok(true)
    }
    fn sample(&mut self, procs: &mut [Process]) -> Result<(), String> {
        match self.phase {
            Phase::Reviews => {
                if self.reviews_pending()? {
                    self.keep_together(procs)?;
                }
            }
            Phase::Keeping => {
                let calls = calls(procs)?;
                if calls.foreground_idle() {
                    {
                        let (left, right) = (&(calls.ends().len()), &(self.amends + CHANGES.as_slice().len()));
                        if left != right {
                            return Err(format!(
                                "assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"
                            ));
                        }
                    }
                    for call in &calls.ends()[self.amends..] {
                        if call.exit != Some(Exit::Code(0)) {
                            return Err(format!("{}", String::from_utf8_lossy(&call.stderr)));
                        }
                    }
                    self.phase = Phase::Gates;
                }
            }
            Phase::Gates => {
                release(procs, b"batch-reviews")?;
                if CHANGES.as_slice().iter().copied().try_fold(true, |all, task| {
                    if !all {
                        return Ok(false);
                    }
                    let row = self.read(&["task", "show", &task.to_string()])?;
                    Ok::<_, String>(
                        self.change(task)?["gates"][0]["verdict"] == "approved"
                            && row["chip"]["phase"] == "idle"
                            && row["delegates"].as_array().is_some_and(Vec::is_empty),
                    )
                })? {
                    release(procs, b"batch-ready")?;
                    self.phase = Phase::Batch;
                }
            }
            Phase::Batch => {
                release(procs, b"batch-ready")?;
                if let Some(proof) = self.proof()? {
                    self.barrier.release(proof.run);
                    self.first = Some(proof);
                    self.phase = Phase::Search(narrowed::Step::One);
                }
            }
            Phase::Search(step) => self.search(step, procs)?,
        }

        Ok(())
    }
}
impl Referee<Process> for Judge<'_> {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        if self.failed.is_some() {
            return;
        }
        let result = (|| -> Result<(), String> {
            if let PeerStop::Due(_) = self.peer_stop {
                // Only after the binary and its clients end can held replies
                // finish without delivering a repair result to a live delegate.
                for name in [b"batch-owner".as_slice(), b"batch-repair".as_slice(), b"goal-finish".as_slice()] {
                    release(procs, name)?;
                }
                for proc in procs.iter_mut() {
                    if let Process::Peer(peer) = proc {
                        peer.shutdown();
                    }
                }
                self.peer_stop = PeerStop::Stopped;
            }
            if matches!(self.chat.phase, ChatPhase::SettingUp) {
                if calls(procs)?.ends().len() < turn_restart::SETUP_CALLS {
                    return Ok(());
                }
                self.chat.act(now, procs);
                if self.chat.failed.is_some() {
                    return Ok(());
                }
            }
            if !self.chat.stopping() && now >= self.poll {
                self.poll = now.saturating_add(Duration::from_secs(1));
                let Some(Process::Person(person)) = procs.get(1) else { return Err(String::from("person")) };
                if person.finished {
                    if !self.drive_reads(procs)? {
                        self.poll = now.saturating_add(Duration::from_millis(10));
                        return Ok(());
                    }
                    match self.sample(procs) {
                        Ok(()) => self.reads.borrow_mut().cycle.clear(),
                        Err(why) if why == READ_PENDING => {
                            self.drive_reads(procs)?;
                            self.poll = now.saturating_add(Duration::from_millis(10));
                        }
                        Err(why) => return Err(why),
                    }
                }
            }

            Ok(())
        })();
        if let Err(why) = result {
            self.failed = Some(if let Some(Process::Binary(binary)) = procs.first() {
                format!("{why}; serve stderr: {}", String::from_utf8_lossy(binary.stderr()))
            } else {
                why
            });
        }
    }
    fn watch(&mut self, now: Time, procs: &[Process]) -> Result<(), String> {
        let result = match self.failed.take() {
            Some(why) => Err(why),
            None => self.chat.watch(now, procs),
        };
        if result.is_err() {
            self.report(procs);
        }
        if self.chat.stopping()
            && matches!(self.peer_stop, PeerStop::Waiting)
            && let Some(Process::Binary(binary)) = procs.first()
            && binary.is_empty()
        {
            self.peer_stop = PeerStop::Due(now);
        }
        result
    }
    fn next_deadline(&self) -> Option<Time> {
        if self.chat.stopping() {
            match self.peer_stop {
                PeerStop::Due(at) => skein_lib::earliest(Some(at), self.chat.next_deadline()),
                PeerStop::Waiting | PeerStop::Stopped => self.chat.next_deadline(),
            }
        } else {
            Some(self.chat.deadline.until().map_or(self.poll, |until| until.min(self.poll)))
        }
    }
    fn overdue(&self, now: Time) -> Option<String> {
        self.chat.overdue(now).map(|why| format!("batch {:?}: {why}", self.phase))
    }
    fn passed(&self) -> bool {
        if self.failed.is_some() {
            return false;
        }
        self.chat.passed()
    }
}

fn peer(root: &Path, forge: &Throwaway) -> super::llm::Listening {
    let mut recipe = temper_real_loop_world::llm::scripted(Box::new([]));
    // Three reviews, the latch and the owner may be held
    // together. The transport and script domain admit the same concurrent calls.
    recipe.limits.connections = 8;
    recipe.limits.io.sockets = 16;
    recipe.domain_limits.calls = 8;
    // The single-agent observation room scales with the independent producers.
    let producers = 3;
    recipe.limits.observations = recipe.limits.observations.checked_mul(producers).expect("batch observations");
    recipe.limits.observation_bytes =
        recipe.limits.observation_bytes.checked_mul(producers).expect("batch observation bytes");
    let scripts = temper_real_loop_world::llm::cued(scripts(root, forge));
    let rules = Rules {
        cued: scripts
            .cued
            .iter()
            .map(|script| -> Box<[Rule]> {
                if script.cue.as_ref() == b"@forge_chat" && script.turns.len() > 3 {
                    Box::new([
                        Rule { cue: b"Messages arrived. Read them before continuing.".as_slice().into(), turn: 3 },
                        Rule { cue: b"Nothing more will come:".as_slice().into(), turn: 2 },
                        Rule { cue: b"Begin the work your brief describes.".as_slice().into(), turn: 0 },
                        Rule { cue: Box::new([]), turn: 1 },
                    ])
                } else {
                    Box::new([])
                }
            })
            .collect(),
        asked: Box::new([]),
    };
    recipe.domain = skein_fake_llm_domain::Domain::played_by_input(
        &recipe.domain_limits,
        17,
        scripts,
        skein_fake_llm_domain::api::Menu { arguments: Box::new([]), invalid: Box::new([]) },
        rules,
    )
    .expect("concurrent batch scripts");
    super::llm_recipe(recipe).without_records()
}

/// Three changes on the owned runner, with a red second member and held repair.
pub fn run_narrowed(root: &Path, forge: &Throwaway, barrier: &JobBarrier) {
    forge.detect_manual_merges();
    let peer = peer(root, forge);
    peer.configure(root);
    let clock = Clock::new();
    let mut binary = super::serve(root);
    let streams = binary.take_streams().expect("serve pipes");
    let mut fds = streams.descriptors();
    let person_root = skein_shell::open_root(root).expect("native root");
    let progress = super::patience::Patience::watching("the batch landing", LANDING, &binary);
    let mut world = real::World::new(Judge {
        failed: None,
        chat: ChatJudge {
            failed: None,
            phase: ChatPhase::SettingUp,
            ending: Ending::Stop,
            deadline: progress,
            forge: Some(forge),
            poll: clock.now().now,
            merged: false,
        },
        forge,
        barrier,
        phase: Phase::Reviews,
        poll: clock.now().now,
        first: None,
        second: None,
        probes: Vec::new(),
        reads: RefCell::new(Reads::default()),
        amends: 0,
        observations: std::cell::RefCell::new(std::collections::BTreeMap::new()),
        peer_stop: PeerStop::Waiting,
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(vec![person_root], || {
        Process::Person(Box::new(person::Person::waiting_for_ctl_with_key(
            person_root,
            person::adopt_and_start_change(forge),
        )))
    });
    let ctl_root = skein_shell::open_root(root).expect("ctl root");
    fds.push(ctl_root);
    world.spawn_with_fds(fds, || {
        let mut calls = ctl::Calls::new(ctl_root, turn_restart::setup(root), Some(&streams), None);
        calls.hold();
        Process::Calls(Box::new(calls))
    });
    peer.spawn(&mut world);
    let outcome = world.run(&clock, LANDING.saturating_add(Duration::from_secs(10)));
    assert!(outcome.procs.iter().all(Host::is_empty), "nothing of the tree remains after Terminate");
    let Process::Binary(binary) = &outcome.procs[0] else { panic!("serve") };
    assert_eq!(binary.exit_status(), Some(Exit::Code(0)), "{}", String::from_utf8_lossy(binary.stderr()));
}
