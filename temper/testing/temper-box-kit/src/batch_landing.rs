//! A queue that lands green and a red narrowed on real git and the owned runner
//! (temper/docs/design/testing/process-tier.md; domain/landing.md, 5, 8–10 and 14;
//! domain/landing-search.md, 3–5 and 8).
mod narrowed;
use super::{
    ChatJudge, ChatPhase, Ending, LANDING, Process, ctl,
    forgejo::{JobBarrier, Throwaway},
    landing_scripts, person, turn_restart,
};
use serde_json::{Value, json};
use skein_fake_llm_domain::api::{Finish, Line, Piece, Script, Turn};
use skein_io::kernel::{Exit, Signal};
use skein_lib::{Duration, Time};
use skein_shell::{Clock, Host};
use skein_world::{Referee, real};
use std::{fs, path::Path};

// Hello is task 1, the landing chat 2. Its one delegate batch makes these
// siblings before any producer starts. The latch is a report, not a change.
const CHANGES: [u64; 3] = [3, 4, 5];
const FOURTH: u64 = 6;
#[derive(Clone, Copy)]
enum Story {
    Green,
    Narrowed,
}
impl Story {
    fn changes(self) -> u64 {
        match self {
            Self::Green => 4,
            Self::Narrowed => 3,
        }
    }
    fn latch(self) -> u64 {
        self.changes() + 3
    }
}

fn call(name: &[u8], value: &Value) -> Turn {
    Turn {
        lines: Box::new([Line::Call {
            name: name.into(),
            arguments: serde_json::to_vec(value).expect("tool JSON").into_boxed_slice(),
        }]),
        finish: Finish::ToolCalls,
        tokens: 20,
    }
}

fn hold(name: &[u8]) -> Line {
    Line::Pieces { pieces: Box::new([Piece { text: b"Waiting.".as_slice().into(), hold: Some(name.into()) }]) }
}

fn scripts(root: &Path, forge: &Throwaway, story: Story) -> Box<[Script]> {
    let value: Value =
        serde_json::from_slice(&fs::read(root.join("temper.json")).expect("configuration")).expect("JSON");
    let engine = temper_real_loop_world::settings::configuration(&value);
    let stock = temper_service::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    scripted_batch(landing_scripts(super::ANSWER, forge).into_vec(), stock.rules.maximum_run_spend, story)
}

fn scripted_batch(mut shared: Vec<Script>, run: u64, story: Story) -> Box<[Script]> {
    // Four runs per change and one for the latch fit the chat's stock 24-run budget.
    let mut owner = shared.remove(0);
    let Line::Call { arguments, .. } = &owner.turns[0].lines[0] else { panic!("delegate") };
    let delegate: Value = serde_json::from_slice(arguments).expect("delegate JSON");
    let mut batch = Vec::new();
    let mut scripts = Vec::new();
    for number in 1..=story.changes() {
        let words = format!("@batch-code-{number} Add batch-{number}.txt");
        let mut change = delegate["batch"][0].clone();
        change["spec"]["words"] = words.clone().into();
        change["tracked"] = json!({"priority":number});
        change["authority"]["budget"]["spend"] = (4 * run).into();
        batch.push(change);
        let path = format!("batch-{number}.txt");
        let mut turns = vec![
            call(b"read", &json!({"path":path})),
            call(
                b"write",
                &json!({"path":path,"content":match (story, number) {
                    (Story::Narrowed, 2) => "red\n".to_owned(),
                    _ => format!("Change {number}\n"),
                }}),
            ),
            call(
                b"finish",
                &json!({"form":"change","fields":{
                "title":format!("Batch change {number}"),"body":format!("Add {path}.")}}),
            ),
        ];
        if number == 4 {
            turns[0].lines = Box::new([hold(b"batch-fourth"), turns[0].lines[0].clone()]);
        }
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
    // A latch's result wakes the owner before the changes end. Keep its finish
    // from cancelling those children; only the green story releases its finish.
    let finish = &mut owner.turns[2];
    finish.lines = [vec![hold(b"batch-owner")], finish.lines.to_vec()].concat().into_boxed_slice();
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
    if let Story::Narrowed = story {
        narrowed::hold_repair(&mut scripts);
    }
    scripts.into_boxed_slice()
}

/// The turn and runner-cancellation stories use the same owned forge, policy,
/// producer and funding setup. The batch stories also hold concurrent model turns.
pub fn configure(root: &Path, repository: u64) {
    turn_restart::configure_batch(root, repository, 3);
    let path = root.join("temper.json");
    let mut value: Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    for name in ["tasks.tasks", "tasks.project_tasks", "tasks.tree_tasks", "tasks.exemptions"] {
        value["engine"]["limits"]["overrides"][name] = 64.into();
    }
    for name in ["tasks.batch", "authority.batch", "tasks.delegates", "people.pending"] {
        value["engine"]["limits"]["overrides"][name] = 8.into();
    }
    value["engine"]["limits"]["overrides"]["people.operations"] = 9.into();
    value["engine"]["limits"]["overrides"]["people.inbox_tasks"] = 128.into();
    value["engine"]["limits"]["overrides"]["fleet.attempts"] = 8.into();
    value["engine"]["limits"]["overrides"]["brief.briefs"] = 8.into();
    value["workers"][0]["slots"] = 8.into();
    value["workers"][0]["limits"]["overrides"]["hub.slots"] = 8.into();
    for name in ["checkout.workspaces", "git_component.operations"] {
        value["application"]["limits"]["overrides"][name] = 8.into();
    }
    // The one model call carries four independent change authorities and a latch.
    let agent_path = root.join("agent.json");
    let mut agent: Value = serde_json::from_slice(&fs::read(&agent_path).expect("agent configuration")).expect("JSON");
    agent["profile"]["declared"]["tool_payload"] = 8192.into();
    value["engine"]["limits"]["overrides"]["tasks.message_bytes"] = 8192.into();
    value["engine"]["limits"]["overrides"]["tasks.inbox_bytes"] = 8192.into();
    value["engine"]["limits"]["overrides"]["core.call_answer_bytes"] = 8192.into();
    let limits = &mut value["workers"][0]["limits"]["overrides"];
    limits["tasks.message_bytes"] = 8192.into();
    limits["tasks.inbox_bytes"] = 8192.into();
    limits["core.call_answer_bytes"] = 8192.into();
    limits["hub.event_bytes"] = 8190.into();
    value["application"]["limits"]["overrides"]["checkout.message_bytes"] = 8192.into();
    super::configuration::turn_bounds::apply(&mut value, super::configuration::turn_bounds::turn_body(&agent));
    super::write_private(&agent_path, &serde_json::to_vec(&agent).expect("agent JSON"));
    super::write_private(&path, &serde_json::to_vec(&value).expect("configuration JSON"));
}

fn read(root: &Path, arguments: &[&str]) -> Value {
    let ended = ctl::alone(root, ctl::Call::new(arguments).in_directory("private").json());
    assert_eq!(ended.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&ended.stderr));
    ctl::json(&ended)
}
fn change(root: &Path, task: u64) -> Value {
    read(root, &["change", "show", &task.to_string()])
}
fn landings(root: &Path) -> Value {
    read(root, &["landing", "list", "--project", "1", "--ended"])
}
fn landing(root: &Path, task: u64) -> Value {
    read(root, &["landing", "show", &task.to_string()])
}
fn get(forge: &Throwaway, route: &str) -> Value {
    forge.repository_observation("acme", "widget", route)
}
fn sha(value: &Value) -> &str {
    &value.as_str().expect("commit")[..40]
}
fn main(forge: &Throwaway) -> String {
    get(forge, "branches/main")["commit"]["id"].as_str().expect("main tip").into()
}
fn members(panel: &Value) -> &[Value] {
    panel["members"].as_array().expect("members")
}
fn candidate(panel: &Value) -> &str {
    sha(&members(panel).last().expect("top")["candidate"])
}
fn member_branch(task: u64, number: usize) -> String {
    if number == 1 { format!("temper/{task}/l{task}") } else { format!("temper/{task}/l{task}-{number}") }
}
fn gate_runs(forge: &Throwaway) -> Vec<Value> {
    let runs = get(forge, "actions/runs?limit=100");
    assert_eq!(
        runs["total_count"].as_u64(),
        Some(u64::try_from(runs["workflow_runs"].as_array().expect("runs").len()).expect("run count")),
        "one bounded page holds every run"
    );
    runs["workflow_runs"].as_array().expect("runs").clone()
}

/// Fixture admission is checked without starting the owned forge or a process story.
pub mod tests {
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
            super::Story::Green,
        );
        let owner = scripts.iter().find(|script| script.cue.as_ref() == b"@forge_chat").expect("owner");
        let super::Line::Call { arguments, .. } = &owner.turns[0].lines[0] else { panic!("delegate") };
        let delegate: serde_json::Value = serde_json::from_slice(arguments).expect("delegate JSON");
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
#[derive(Debug)]
enum Phase {
    Reviews,
    Keeping,
    Gates,
    Batch,
    Fourth,
    Next,
    Landed,
    Ending,
    Search(narrowed::Step),
}
struct Judge<'a> {
    chat: ChatJudge<'a>,
    root: &'a Path,
    forge: &'a Throwaway,
    barrier: &'a JobBarrier,
    phase: Phase,
    poll: Time,
    base: String,
    first: Option<Proof>,
    second: Option<Proof>,
    story: Story,
    probes: Vec<u64>,
}

fn release(procs: &mut [Process], name: &[u8]) {
    for proc in procs {
        if let Process::Peer(peer) = proc {
            peer.release(name).expect("release scripted turn");
        }
    }
}
fn calls(procs: &mut [Process]) -> &mut ctl::Calls {
    let Process::Calls(calls) = &mut procs[2] else { panic!("person's ctl") };
    calls
}

impl Judge<'_> {
    fn reviews_pending(&self) -> bool {
        CHANGES.into_iter().all(|task| {
            let change = change(self.root, task);
            change["step"] == "gating" && change["gates"][0]["verdict"] == "pending"
        })
    }
    fn keep_together(&mut self, procs: &mut [Process]) {
        assert_eq!(landings(self.root)["landings"].as_array().expect("landings").len(), 0);
        // The same dependency is lifted in one durable completion. This makes
        // three independent units eligible at quiescence, not a joined unit or
        // a race between three separately released reviews (landing.md, 9).
        for task in CHANGES {
            calls(procs).push(ctl::Step::Run(
                ctl::Call::new(&[
                    "task",
                    "amend",
                    &task.to_string(),
                    "--rev",
                    "0",
                    "--dependency",
                    &self.story.latch().to_string(),
                    "Become ready with the other two changes",
                    "--key",
                    &format!("batch-keep-{task}"),
                ])
                .in_directory("private"),
            ));
        }
        self.phase = Phase::Keeping;
    }
    fn proof(&self, previous: Option<u64>, expected: &[u64]) -> Option<Proof> {
        let run = self.barrier.waiting(previous)?;
        let list = landings(self.root);
        let rows = list["landings"].as_array().expect("landing list");
        assert_eq!(rows.len(), if previous.is_some() { 2 } else { 1 }, "one landing per batch");
        let row = rows.iter().find(|row| row["phase"] == "proving")?;
        let task = row["task"].as_u64().expect("landing task");
        assert_eq!(row["changes"], json!(expected), "sealed in queue order");
        let panel = landing(self.root, task);
        if members(&panel).iter().any(|member| member["candidate"].is_null()) {
            return None;
        }
        assert_eq!(panel["phase"], "proving");
        assert_eq!(panel["round"]["number"], 1);
        assert_eq!(panel["train"]["branch"], "main");
        assert_eq!(
            members(&panel).iter().map(|member| member["change"].as_u64().expect("change")).collect::<Vec<_>>(),
            expected
        );
        self.dispatched(run, candidate(&panel));
        let runs = gate_runs(self.forge);
        assert_eq!(runs.len(), if previous.is_some() { 2 } else { 1 }, "no push, member or merge runs");
        self.stack(&panel);
        Some(Proof { panel, run })
    }
    fn dispatched(&self, run: u64, prefix: &str) {
        let actual = get(self.forge, &format!("actions/runs/{run}"));
        assert_eq!(actual["id"], run);
        assert_eq!(actual["workflow_id"], "gate.yaml");
        assert_eq!(actual["trigger_event"], "workflow_dispatch");
        assert_eq!(actual["trigger_user"]["login"], "temper");
        assert_eq!(actual["status"], "running");
        assert_eq!(actual["commit_sha"], prefix, "proof at the chosen prefix");
    }
    fn stack(&self, panel: &Value) {
        let task = panel["task"].as_u64().expect("landing task");
        let mut below = sha(&members(panel)[0]["base"]).to_owned();
        for (index, member) in members(panel).iter().enumerate() {
            let number = index + 1;
            let branch = member_branch(task, number);
            let prefix = sha(&member["candidate"]);
            let actual = get(self.forge, &format!("branches/{branch}"));
            assert_eq!(actual["commit"]["id"], prefix, "a branch for each member's prefix");
            let commit = get(self.forge, &format!("git/commits/{prefix}"));
            assert_eq!(commit["parents"].as_array().expect("parents").len(), 2, "no-fast-forward merge");
            assert_eq!(commit["parents"][0]["sha"], below, "main's first-parent line preserves queue order");
            assert_eq!(commit["parents"][1]["sha"], sha(&member["head"]));
            let change = change(self.root, member["change"].as_u64().expect("change"));
            assert_eq!(sha(&change["head"]), sha(&member["head"]), "no member head rewritten");
            below = prefix.into();
        }
        assert_eq!(panel["round"]["branch"], member_branch(task, members(panel).len()));
    }
}

impl Judge<'_> {
    fn stop(&mut self, procs: &mut [Process]) {
        calls(procs).release();
        let Process::Binary(binary) = &mut procs[0] else { panic!("serve") };
        binary.signal(Signal::Terminate);
        for proc in procs {
            if let Process::Peer(peer) = proc {
                peer.shutdown();
            }
        }
        self.chat.phase = ChatPhase::Stopping;
    }
    fn landed(&self, proof: &Proof) -> bool {
        let panel = landing(self.root, proof.panel["task"].as_u64().expect("landing task"));
        if panel["phase"] != "landed" {
            return false;
        }
        assert_eq!(members(&panel).len(), members(&proof.panel).len());
        for (member, prepared) in members(&panel).iter().zip(members(&proof.panel)) {
            if !self.member_landed(member, prepared) {
                return false;
            }
        }
        let pull =
            get(self.forge, &format!("pulls/{}", panel["round"]["pull"]["number"].as_u64().expect("landing pull")));
        assert_eq!(pull["merged"], true);
        assert_eq!(pull["head"]["sha"], candidate(&proof.panel));
        assert_eq!(pull["merge_commit_sha"], candidate(&proof.panel), "one fast-forward, no extra merge commit");
        let run = get(self.forge, &format!("actions/runs/{}", proof.run));
        assert_eq!(run["status"], "success");
        true
    }
    fn member_landed(&self, member: &Value, prepared: &Value) -> bool {
        assert_eq!(member["change"], prepared["change"]);
        if member["landed"].is_null() {
            return false;
        }
        assert_eq!(member["landed"], prepared["candidate"], "each member lands at its own prefix");
        let change = change(self.root, member["change"].as_u64().expect("change"));
        if change["merge"].is_null() {
            return false;
        }
        assert_eq!(sha(&change["merge"]), sha(&member["landed"]));
        let number = member["pull"]["number"].as_u64().expect("member pull");
        let pull = get(self.forge, &format!("pulls/{number}"));
        if pull["merged"] != true {
            return false;
        }
        assert_eq!(pull["merge_commit_sha"], sha(&member["landed"]));
        true
    }
    fn sample(&mut self, procs: &mut [Process]) {
        match self.phase {
            Phase::Reviews => {
                if self.reviews_pending() {
                    self.keep_together(procs);
                }
            }
            Phase::Keeping => {
                let calls = calls(procs);
                if calls.foreground_idle() {
                    assert_eq!(calls.ends().len(), turn_restart::SETUP_CALLS + 3);
                    for call in calls.ends() {
                        assert_eq!(call.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&call.stderr));
                    }
                    self.phase = Phase::Gates;
                }
            }
            Phase::Gates => {
                release(procs, b"batch-reviews");
                if CHANGES.into_iter().all(|task| change(self.root, task)["gates"][0]["verdict"] == "passed") {
                    assert_eq!(landings(self.root)["landings"].as_array().expect("landings").len(), 0);
                    release(procs, b"batch-ready");
                    self.phase = Phase::Batch;
                }
            }
            Phase::Batch => {
                if let Some(proof) = self.proof(None, &CHANGES) {
                    assert_eq!(main(self.forge), self.base, "main stays at the base throughout proof");
                    self.first = Some(proof);
                    match self.story {
                        Story::Green => {
                            release(procs, b"batch-fourth");
                            self.phase = Phase::Fourth;
                        }
                        Story::Narrowed => {
                            self.barrier.release(self.first.as_ref().expect("top proof").run);
                            self.phase = Phase::Search(narrowed::Step::One);
                        }
                    }
                }
            }
            Phase::Fourth => {
                release(procs, b"batch-reviews");
                let fourth = change(self.root, FOURTH);
                if fourth["step"] == "ready" {
                    let first = self.first.as_ref().expect("first proof");
                    let list = landings(self.root);
                    assert_eq!(list["landings"].as_array().expect("landings").len(), 1);
                    let panel = landing(self.root, first.panel["task"].as_u64().expect("landing"));
                    assert_eq!(panel["phase"], "proving", "fourth ready while the batch proves");
                    assert_eq!(members(&panel).len(), CHANGES.len(), "a formed batch is sealed");
                    for (member, prepared) in members(&panel).iter().zip(members(&first.panel)) {
                        assert_eq!(member["change"], prepared["change"]);
                        assert_eq!(member["candidate"], prepared["candidate"]);
                    }
                    assert_eq!(get(self.forge, &format!("actions/runs/{}", first.run))["status"], "running");
                    assert_eq!(main(self.forge), self.base);
                    self.barrier.release(first.run);
                    self.phase = Phase::Next;
                }
            }
            Phase::Next => {
                let first = self.first.as_ref().expect("first proof");
                if self.landed(first)
                    && let Some(proof) = self.proof(Some(first.run), &[FOURTH])
                {
                    assert_eq!(main(self.forge), candidate(&first.panel));
                    assert_eq!(sha(&members(&proof.panel)[0]["base"]), candidate(&first.panel));
                    self.barrier.release(proof.run);
                    self.second = Some(proof);
                    self.phase = Phase::Landed;
                }
            }
            Phase::Landed => {
                let second = self.second.as_ref().expect("second proof");
                if self.landed(second) {
                    assert_eq!(main(self.forge), candidate(&second.panel));
                    let runs = gate_runs(self.forge);
                    assert_eq!(runs.len(), 2, "one dispatched proof per landing, none after merging");
                    for proof in [self.first.as_ref().expect("first"), second] {
                        let runs: Vec<_> =
                            runs.iter().filter(|run| run["commit_sha"] == candidate(&proof.panel)).collect();
                        assert_eq!(runs.len(), 1);
                        assert_eq!(runs[0]["id"], proof.run);
                        assert_eq!(runs[0]["status"], "success");
                    }
                    release(procs, b"batch-owner");
                    self.phase = Phase::Ending;
                }
            }
            Phase::Search(step) => self.search(step, procs),
            Phase::Ending => {
                release(procs, b"batch-owner");
                let Process::Person(person) = &procs[1] else { panic!("person") };
                if person.finished && person.client.as_ref().is_some_and(Host::is_empty) {
                    self.stop(procs);
                }
            }
        }
    }
}
impl Referee<Process> for Judge<'_> {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        if matches!(self.chat.phase, ChatPhase::SettingUp) {
            if calls(procs).ends().len() < turn_restart::SETUP_CALLS {
                return;
            }
            self.chat.act(now, procs);
        }
        if !self.chat.stopping() && now >= self.poll {
            self.poll = now.saturating_add(Duration::from_millis(250));
            let Process::Person(person) = &procs[1] else { panic!("person") };
            if person.finished {
                self.sample(procs);
            }
        }
    }
    fn observe(&mut self, now: Time, procs: &[Process]) {
        self.chat.observe(now, procs);
    }
    fn next_deadline(&self) -> Option<Time> {
        if self.chat.stopping() {
            self.chat.next_deadline()
        } else {
            Some(self.chat.deadline.until().map_or(self.poll, |until| until.min(self.poll)))
        }
    }
    fn overdue(&self, now: Time) -> Option<String> {
        self.chat.overdue(now).map(|why| format!("batch {:?}: {why}", self.phase))
    }
    fn passed(&self) -> bool {
        self.chat.passed()
    }
}

fn peer(root: &Path, forge: &Throwaway, story: Story) -> super::llm::Listening {
    let mut recipe = temper_real_loop_world::llm::scripted(Box::new([]));
    // Three reviews, the fourth coder, the latch and the owner may be held
    // together. The transport and script domain admit the same concurrent calls.
    recipe.limits.connections = 8;
    recipe.limits.io.sockets = 16;
    recipe.domain_limits.calls = 8;
    recipe.domain = skein_fake_llm_domain::Domain::played(
        &recipe.domain_limits,
        17,
        temper_real_loop_world::llm::cued(scripts(root, forge, story)),
        skein_fake_llm_domain::api::Menu { arguments: Box::new([]), invalid: Box::new([]) },
    )
    .expect("concurrent batch scripts");
    super::llm_recipe(recipe)
}

/// One world drives the native person, ctl Steps, fake model and shipped binary.
/// The referee's polls and the whole story are bounded by the landing deadline.
pub fn run(root: &Path, forge: &Throwaway, barrier: &JobBarrier) {
    run_story(root, forge, barrier, Story::Green);
}

/// The same owned-turn world, with a red second member and held repair delegate.
pub fn run_narrowed(root: &Path, forge: &Throwaway, barrier: &JobBarrier) {
    run_story(root, forge, barrier, Story::Narrowed);
}

fn run_story(root: &Path, forge: &Throwaway, barrier: &JobBarrier, story: Story) {
    let peer = peer(root, forge, story);
    peer.configure(root);
    let clock = Clock::new();
    let mut binary = super::serve(root);
    let streams = binary.take_streams().expect("serve pipes");
    let mut fds = streams.descriptors();
    let person_root = skein_shell::open_root(root).expect("native root");
    let progress = super::patience::Patience::watching("the batch landing", LANDING, &binary);
    let mut world = real::World::new(Judge {
        chat: ChatJudge {
            phase: ChatPhase::SettingUp,
            ending: Ending::Stop,
            deadline: progress,
            forge: Some(forge),
            poll: clock.now().now,
            merged: false,
        },
        root,
        forge,
        barrier,
        phase: Phase::Reviews,
        poll: clock.now().now,
        base: main(forge),
        first: None,
        second: None,
        story,
        probes: Vec::new(),
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
