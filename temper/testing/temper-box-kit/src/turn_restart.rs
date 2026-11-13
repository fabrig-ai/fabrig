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

const SETUP_CALLS: usize = super::NATIVE_SETUP_CALLS + 2;

#[derive(Debug)]
pub struct Reads {
    pub landing: Value,
    pub change: Value,
}

fn read(root: &Path, arguments: &[&str]) -> Value {
    let call = ctl::alone(root, ctl::Call::new(arguments).in_directory("private").json());
    assert_eq!(call.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&call.stderr));
    ctl::json(&call)
}

fn reads(root: &Path, landing: u64, change: u64) -> Reads {
    Reads {
        landing: read(root, &["landing", "show", &landing.to_string()]),
        change: read(root, &["change", "show", &change.to_string()]),
    }
}

/// Initialize the store without queue requirements before seeding the forge record.
#[must_use]
pub fn start(scratch: &super::Scratch) -> Throwaway {
    let forge = Throwaway::start_bare(scratch);
    let path = scratch.path().join("temper.json");
    let mut config: Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    config["application"]["landing"]["projects"] = json!([]);
    write_private(&path, &serde_json::to_vec(&config).expect("configuration JSON"));
    forge.with_repository(scratch, 20, landing_by_turn::WORKFLOW)
}

/// Select turn landing for the seeded project, whose policy names stock policy 1.
pub fn configure(root: &Path, repository: u64) {
    let path = root.join("temper.json");
    let mut config: Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    config["application"]["landing"]["projects"] = json!([{
        "project":1,"by":"turn",
        "gates":[{"forge":1,"repository":repository,
            "gates":[{"name":"gate","workflow":"gate.yaml","ignored":["docs"],"blocking":true}]}]
    }]);
    write_private(&path, &serde_json::to_vec(&config).expect("configuration JSON"));
    let path = root.join("native-landing-settings.json");
    let mut settings: Value =
        serde_json::from_slice(&fs::read(&path).expect("native landing settings")).expect("settings JSON");
    settings["items"][0]["set"]["gates"] = json!([{"number":1,"blocking":true,"freshness":"clean"}]);
    write_private(&path, &serde_json::to_vec(&settings).expect("settings JSON"));
}

/// Hold the first run's blocking review until main has moved, before readiness.
fn held_review_scripts(forge: &Throwaway) -> Box<[skein_fake_llm_domain::api::Script]> {
    use skein_fake_llm_domain::api::{Line, Piece};
    let mut scripts = landing_scripts(ANSWER, forge);
    let review = scripts
        .iter_mut()
        .find(|script| script.cue.as_ref() == b"Review this change at the named head")
        .expect("review script");
    let mut verdict = review.turns[0].clone();
    let mut lines = verdict.lines.into_vec();
    lines.insert(
        0,
        Line::Pieces {
            pieces: Box::new([Piece {
                text: b"Checked the change at the original head.".as_slice().into(),
                hold: Some(b"turn-restart-review".as_slice().into()),
            }]),
        },
    );
    verdict.lines = lines.into_boxed_slice();
    review.turns = Box::new([verdict]);
    scripts
}

/// Leave landing tasks a share of the seeded project's open period (domain/landing.md, 4.2).
fn setup(root: &Path) -> Vec<ctl::Step> {
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
    pub moved: String,
    pub run: u64,
}

fn moved_main<'a>(advanced: Option<&'a (String, String)>, previous: Option<&'a Proof>) -> Option<&'a str> {
    advanced.map(|(_, base)| base.as_str()).or_else(|| previous.map(|proof| proof.moved.as_str()))
}

/// Witness selection is checked without starting the owned forge or runner.
pub mod tests {
    /// A cold retry retains the previous base; a new observation takes precedence.
    pub fn keeps_the_moved_main_witness_on_retry() {
        let previous = super::Proof {
            reads: super::Reads { landing: serde_json::Value::Null, change: serde_json::Value::Null },
            original: "original".into(),
            moved: "previous main".into(),
            run: 1,
        };
        let advanced = ("head".into(), "current main".into());
        assert_eq!(super::moved_main(None, Some(&previous)), Some("previous main"), "retry keeps its base");
        assert_eq!(super::moved_main(Some(&advanced), None), Some("current main"));
        assert_eq!(super::moved_main(Some(&advanced), Some(&previous)), Some("current main"));
        assert_eq!(super::moved_main(None, None), None);
    }
}

struct Judge<'a> {
    chat: ChatJudge<'a>,
    root: &'a Path,
    barrier: &'a JobBarrier,
    proof: &'a Cell<Option<Proof>>,
    landed: &'a Cell<Option<Reads>>,
    previous: Option<&'a Proof>,
    retry: bool,
    ready: bool,
    advanced: Option<(String, String)>,
    poll: Time,
    progress: (u32, super::patience::Patience),
}
impl Judge<'_> {
    fn sample(&mut self) -> bool {
        let forge = self.chat.forge.expect("owned forge");
        if let Some(previous) = self.previous
            && !self.retry
        {
            let landing = previous.reads.landing["task"].as_u64().expect("landing task");
            let change = previous.reads.change["task"].as_u64().expect("change task");
            // Let the outer world drive its peers and binary pipes between reads.
            let change = read(self.root, &["change", "show", &change.to_string()]);
            if change["step"] != "landed" {
                return false;
            }
            let landing = read(self.root, &["landing", "show", &landing.to_string()]);
            if landing["phase"] != "landed" {
                return false;
            }
            self.landed.set(Some(Reads { landing, change }));
            return true;
        }
        if self.previous.is_none() && self.advanced.is_none() {
            let pulls = forge.repository_observation("acme", "widget", "pulls?state=open&limit=2");
            let Some(pull) = pulls.as_array().expect("pull list").first() else { return false };
            let original = pull["head"]["sha"].as_str().expect("original head").to_owned();
            let change = read(self.root, &["change", "show", "3"]);
            let gate = &change["gates"][0];
            if gate["verdict"] != "pending" {
                return false;
            }
            assert_eq!(change["step"], "gating");
            assert_eq!(gate["number"], 1);
            assert_eq!(gate["kind"], "review");
            assert_eq!(gate["blocks"], true);
            assert_eq!(change["head"], original);
            let list = read(self.root, &["landing", "list", "--project", "1"]);
            assert!(list["landings"].as_array().expect("landing rows").is_empty(), "review holds readiness");
            let before = forge.repository_observation("acme", "widget", "branches/main");
            let before = before["commit"]["id"].as_str().expect("main before move");
            forge.add_file("notes.txt", landing_by_turn::NOTES);
            let main = forge.repository_observation("acme", "widget", "branches/main");
            let moved = main["commit"]["id"].as_str().expect("moved main");
            assert_ne!(before, moved, "notes moved main");
            let compare = forge.repository_observation("acme", "widget", &format!("compare/{original}...{moved}"));
            assert!(
                !compare["commits"].as_array().expect("comparison commits").is_empty(),
                "original lacks moved main"
            );
            eprintln!(
                "turn restart: main moved from {before} to {moved} while the review held readiness at {original}"
            );
            self.advanced = Some((original, moved.to_owned()));
        }
        let Some(run) = self.waiting_gate(forge) else { return false };
        let (landing, change) = self.members();
        let reads = reads(self.root, landing, change);
        let gate = &reads.landing["members"][0]["gates"][0];
        if self.retry && (gate["faults"] != 1 || gate["proof"]["dispatched"]["attempt"] != 2) {
            return false;
        }
        let candidate = reads.landing["members"][0]["candidate"].as_str().expect("candidate");
        let statuses = forge.repository_observation("acme", "widget", &format!("commits/{}/status", &candidate[..40]));
        if !statuses["statuses"]
            .as_array()
            .expect("statuses")
            .iter()
            .any(|status| status["context"] == "temper/gate/gate" && status["status"] == "pending")
        {
            return false;
        }
        let (original, moved) = self.previous.map_or_else(
            || self.advanced.clone().expect("main advanced"),
            |previous| (previous.original.clone(), previous.moved.clone()),
        );
        self.proof.set(Some(Proof { reads, original, moved, run }));
        true
    }

    fn waiting_gate(&self, forge: &Throwaway) -> Option<u64> {
        let run = self.barrier.waiting(self.previous.map(|proof| proof.run))?;
        let actual = forge.repository_observation("acme", "widget", &format!("actions/runs/{run}"));
        assert_eq!(actual["id"], run, "marker names the forge's actual run");
        // Only the dispatched landing gate takes the runner and reaches the barrier.
        assert_eq!(actual["workflow_id"], "gate.yaml");
        assert_eq!(actual["status"], "running");
        Some(run)
    }

    fn members(&self) -> (u64, u64) {
        if let Some(previous) = self.previous {
            (
                previous.reads.landing["task"].as_u64().expect("landing task"),
                previous.reads.change["task"].as_u64().expect("change task"),
            )
        } else {
            let list = read(self.root, &["landing", "list", "--project", "1"]);
            let row = list["landings"].as_array().expect("landing rows").first().expect("live landing");
            (row["task"].as_u64().expect("landing task"), row["changes"][0].as_u64().expect("member change"))
        }
    }
}
impl Judge<'_> {
    fn stalled(&self, procs: &[Process], figures: &skein_world::Figures) -> ! {
        let Process::Person(person) = &procs[1] else { panic!("native person") };
        let setup = procs.iter().find_map(|proc| match proc {
            Process::Calls(calls) => Some(calls.ends().iter().map(|call| call.exit).collect::<Vec<_>>()),
            Process::Binary(_)
            | Process::Stdout(_)
            | Process::Person(_)
            | Process::Peer(_)
            | Process::Issuer(_)
            | Process::Visitor(_) => None,
        });
        self.chat.forge.expect("forge").pull_update_failure(moved_main(self.advanced.as_ref(), self.previous));
        eprintln!("stalled change: {}", read(self.root, &["change", "show", "3"]));
        eprintln!("stalled change task: {}", read(self.root, &["task", "show", "3"]));
        eprintln!("stalled review task: {}", read(self.root, &["task", "show", "5"]));
        let list = read(self.root, &["landing", "list", "--project", "1", "--ended"]);
        eprintln!("stalled landings: {list}");
        for landing in list["landings"].as_array().expect("landing rows") {
            let task = landing["task"].as_u64().expect("landing task");
            eprintln!("stalled landing: {}", read(self.root, &["landing", "show", &task.to_string()]));
            eprintln!("stalled landing task: {}", read(self.root, &["task", "show", &task.to_string()]));
        }
        eprintln!("stalled tasks: {}", read(self.root, &["task", "list", "--project", "1"]));
        let forge = self.chat.forge.expect("forge");
        let pull = forge.repository_observation("acme", "widget", "pulls/1");
        eprintln!("stalled pull head: {}", pull["head"]["sha"]);
        let branch = forge.repository_observation("acme", "widget", "branches/temper/2/c3");
        let head = branch["commit"]["id"].as_str().expect("branch head");
        let commit = forge.repository_observation("acme", "widget", &format!("git/commits/{head}"));
        eprintln!(
            "stalled branch commit: {}",
            json!({"sha":head,"parents":commit["parents"],"committer":commit["committer"]})
        );
        let witness = self
            .advanced
            .as_ref()
            .map(|(original, moved)| (original, moved))
            .or_else(|| self.previous.map(|proof| (&proof.original, &proof.moved)));
        if let Some((original, moved)) = witness {
            for base in [moved, original] {
                eprintln!(
                    "stalled comparison {base}...{head}: {}",
                    forge.repository_observation("acme", "widget", &format!("compare/{base}...{head}"))
                );
            }
        }
        let Process::Binary(binary) = &procs[0] else { panic!("serve") };
        for line in String::from_utf8_lossy(binary.stderr()).lines() {
            if line.contains("prepar") || line.contains("hold") || line.contains("drift") {
                eprintln!("stalled serve: {line}");
            }
        }
        let runs = forge.repository_observation("acme", "widget", "actions/runs?limit=100");
        let runs: Vec<_> = runs["workflow_runs"]
            .as_array()
            .expect("runs")
            .iter()
            .map(|run| json!({"id":run["id"],"workflow":run["workflow_id"],"status":run["status"]}))
            .collect();
        eprintln!("stalled runs: {runs:?}");
        panic!(
            "turn restart stalled: native step {}, finished={}, setup={setup:?}, forge={}; {figures}",
            person.steps_seen(),
            person.finished,
            self.chat.forge.expect("forge").failure_state()
        );
    }
}
impl Referee<Process> for Judge<'_> {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        // ChatJudge hands the settings session to the native person. Its usual
        // stop waits for a merge; this story stops at its own durable read.
        if matches!(self.chat.phase, ChatPhase::SettingUp) {
            let Process::Calls(calls) = &procs[2] else { panic!("turn settings setup") };
            if calls.ends().len() < SETUP_CALLS {
                self.poll = now.saturating_add(Duration::from_millis(250));
                return;
            }
            self.chat.act(now, procs);
        }
        let Process::Person(person) = &procs[1] else { panic!("native person") };
        if person.steps_seen() != self.progress.0 {
            let Process::Binary(binary) = &procs[0] else { panic!("serve") };
            self.progress = (
                person.steps_seen(),
                super::patience::Patience::watching("the restarted turn's progress", WITHIN, binary),
            );
        }
        if !self.chat.stopping()
            && let Some(figures) = self.progress.1.failed()
        {
            self.stalled(procs, &figures);
        }
        if !self.chat.stopping() && now >= self.poll {
            self.poll = now.saturating_add(Duration::from_millis(250));
            // A listener may draw another port on restart. Let the native
            // client keep that address before ctl loads its client directory.
            let settled = person.finished && person.client.as_ref().is_some_and(Host::is_empty);
            if self.previous.is_some() && !settled {
                return;
            }
            if !self.ready {
                self.ready = self.sample();
                if self.previous.is_none() && self.advanced.is_some() {
                    // A release before the stream reaches its hold is not remembered.
                    for proc in procs.iter_mut() {
                        if let Process::Peer(peer) = proc {
                            peer.release(b"turn-restart-review").expect("release blocking review");
                        }
                    }
                }
            }
            if self.ready && settled {
                let Process::Binary(binary) = &mut procs[0] else { panic!("serve") };
                binary.signal(Signal::Terminate);
                for proc in procs {
                    if let Process::Peer(peer) = proc {
                        peer.shutdown();
                    }
                }
                self.chat.phase = ChatPhase::Stopping;
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
            // This referee drives its own polls; the chat's merge poll is not advanced here.
            Some(self.progress.1.until().map_or(self.poll, |until| until.min(self.poll)))
        }
    }
    fn overdue(&self, now: Time) -> Option<String> {
        self.chat.overdue(now)
    }
    fn passed(&self) -> bool {
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
    let peer = fake_llm(if first { held_review_scripts(forge) } else { landing_scripts(ANSWER, forge) });
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
        chat: ChatJudge {
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
        advanced: None,
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
