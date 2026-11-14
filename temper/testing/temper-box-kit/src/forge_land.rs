//! A person's change on the owned forge, signed in with `ChatGPT` (testing/process-tier.md and testing/live-tier.md).
pub mod conversation;
use super::{Process, sign_in};
use skein_fake_llm_domain::api::{self, Finish, Line, Script, Turn};
use skein_fake_peers::oauth;
use skein_io::kernel::Exit;
use skein_lib::Duration;
use skein_world::{Referee, real};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};

pub mod rung;
mod survivors;

/// forge-setup's first repository goes into project 1, the first made
/// (temper/docs/design/client/ctl/areas.md): work/hello is project 1's.
const FIRST_PROJECT: &str = "1";

/// The model side: processes a story adds beside the box and ctl.
#[expect(
    clippy::large_enum_variant,
    reason = "existing shared fake constructors pass peers by value; faces are moved once at startup"
)]
pub enum Face {
    /// Shipped issuer and Responses; no auxiliary model processes.
    Live,
    /// Skein's fake issuer and the existing TLS LLM peer playing the plan's scripts.
    Fake { issuer: oauth::Peer, peer: super::llm::Listening },
}

/// Initialize three fresh private directories, then patch the config for the fakes.
#[must_use]
pub fn fake_deployment(paths: &[PathBuf; 3]) -> oauth::Peer {
    initialize(paths);
    sign_in::patch_configuration(&paths[0])
}

/// Run the shipped init on a story's three fresh private directories.
pub fn initialize(paths: &[PathBuf; 3]) {
    for path in paths {
        fs::create_dir(path).expect("fresh deployment directory");
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("private deployment directory");
    }
    let initialized = std::process::Command::new(skein_scratch::paths::binary("temper-box"))
        .arg("init")
        .arg("--box")
        .args(
            ["--config", "--data", "--cache"]
                .into_iter()
                .zip(paths)
                .flat_map(|(flag, path)| [std::ffi::OsString::from(flag), path.as_os_str().to_owned()]),
        )
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .current_dir(&paths[1])
        .output()
        .expect("initialize a fresh deployment");
    assert!(initialized.status.success(), "{}", String::from_utf8_lossy(&initialized.stderr));
}

/// The person's request and the first repository's observed identity and contents.
pub struct Plan<'a> {
    /// The person's words, carrying `line`, which cue the chat.
    pub words: &'a [u8],
    /// The unique README line, without its trailing newline.
    pub line: &'a [u8],
    /// The first repository's resource, its packed tag (`temper_engine_domain_forge::tag(forge, first.id)`).
    pub repository: u64,
    /// README.md on main before the change.
    pub readme: &'a [u8],
}

/// The first box's reply, once the change has landed.
pub const LANDED: &str = "The change landed.";
/// The restarted chat's answer: the second box's reply.
pub const RESTARTED: &str = "The change is still on main.";
/// The chat's answer to a held change's escalation.
pub const HELD: &str = "The change is held: its coder could not make it.";

/// Delegate, wait, then reply as an Open chat.
#[must_use]
pub fn plan_scripts(plan: &Plan<'_>) -> Box<[Script]> {
    let line = std::str::from_utf8(plan.line).expect("README line UTF-8");
    // Past the 128 bytes a brief's task section once held: the coder's script is
    // cued by its whole spec, so it runs only when the brief carries all of it.
    let coding = format!(
        "@coding Append this README line: {line} Keep every existing line as it is, add the new line last with a \
         newline after it, and change nothing else."
    );
    assert!(coding.len() > 128, "coder words past the old task section");
    assert!(!plan.words.is_empty(), "nonempty chat cue");
    assert!(!plan.line.is_empty() && !plan.line.contains(&b'\n'), "one nonempty README line");
    assert!(plan.words.windows(plan.line.len()).any(|part| part == plan.line), "request carries the line");
    // System cues match the earliest byte position. Each task's original spec
    // supplies its cue before any conversation history. Keep the coder's spec
    // distinct from the chat cue.
    assert!(!coding.as_bytes().windows(plan.words.len()).any(|part| part == plan.words), "distinct chat cue");
    let edit = format!("{line}\n");
    let mut scripts = temper_real_loop_world::landing::change_scripts(
        temper_real_loop_world::landing::ChangeAuthority { coding_kind: 2, scope: None },
        plan.repository,
        plan.readme,
        edit.as_bytes(),
        plan.words,
        &coding,
    )
    .into_vec();
    // Keep the first repository's proof that the worker cloned the starter workflow.
    let workflow = Turn {
        lines: Box::new([Line::Call {
            name: b"read".as_slice().into(),
            arguments: br#"{"path":".forgejo/workflows/ci.yml"}"#.as_slice().into(),
        }]),
        finish: Finish::ToolCalls,
        tokens: 20,
    };
    let mut coder = std::mem::take(&mut scripts[1].turns).into_vec();
    coder.insert(1, workflow);
    scripts[1].turns = coder.into_boxed_slice();
    let text =
        |bytes: &[u8]| Turn { lines: Box::new([Line::Text { text: bytes.into() }]), finish: Finish::Stop, tokens: 20 };
    let chat = &mut scripts[0];
    chat.turns[0].lines = Box::new([Line::Call {
        name: b"change".as_slice().into(),
        arguments: serde_json::to_vec(&serde_json::json!({"words": coding, "repository": "work/hello"}))
            .expect("change tool JSON")
            .into_boxed_slice(),
    }]);
    // An Open chat replies with text and keeps listening (domain/agent/charter.md, 5).
    // The first box stops without Close; no finish request or closing turn runs here.
    chat.turns[2] = text(LANDED.as_bytes());
    scripts.into_boxed_slice()
}

/// The coder fails at once, so the change holds and escalates to the chat, which
/// answers with HELD instead of LANDED.
#[must_use]
pub fn held_scripts(plan: &Plan<'_>) -> Box<[Script]> {
    let mut scripts = plan_scripts(plan);
    scripts[1].turns = Box::new([Turn {
        lines: Box::new([Line::Call {
            name: b"finish".as_slice().into(),
            arguments: br#"{"form":"failure","reason":"cannot","fields":{}}"#.as_slice().into(),
        }]),
        finish: Finish::ToolCalls,
        tokens: 20,
    }]);
    scripts[0].turns[2] =
        Turn { lines: Box::new([Line::Text { text: HELD.as_bytes().into() }]), finish: Finish::Stop, tokens: 20 };
    scripts
}

/// The chat reports a review hold.
pub const REVIEW_HELD: &str = "The change is held: its review could not run.";

/// Every fresh reviewer fails, while the coder follows the owned-forge story.
#[must_use]
pub fn failed_review_scripts(plan: &Plan<'_>) -> Box<[Script]> {
    let mut scripts = plan_scripts(plan);
    scripts[2].turns = Box::new([Turn {
        lines: Box::new([Line::Call {
            name: b"finish".as_slice().into(),
            arguments: br#"{"form":"failure","reason":"review unavailable","fields":{}}"#.as_slice().into(),
        }]),
        finish: Finish::ToolCalls,
        tokens: 20,
    }]);
    let mut chat = std::mem::take(&mut scripts[0].turns).into_vec();
    chat.truncate(2);
    for _ in 0..8 {
        chat.push(Turn {
            lines: Box::new([Line::Text { text: REVIEW_HELD.as_bytes().into() }]),
            finish: Finish::Stop,
            tokens: 20,
        });
    }
    scripts[0].turns = chat.into_boxed_slice();
    scripts
}

/// The fake face uses the shared plan bearer and existing test TLS root.
#[must_use]
pub fn fake_face(issuer: oauth::Peer, scripts: api::Scripts) -> Face {
    Face::Fake { issuer, peer: sign_in::plan_played(scripts) }
}

/// Indexes of the face's processes, following the story's box and ctl.
pub struct FaceProcs {
    pub issuer: usize,
    pub visitor: usize,
    pub peer: usize,
}

impl Face {
    /// Add the issuer, redirect visitor and model peer to the story's world.
    pub(super) fn spawn<R: Referee<Process>>(self, world: &mut real::World<Process, R>) -> FaceProcs {
        match self {
            Self::Live => panic!("the live face runs without peers through run_live"),
            Self::Fake { issuer, peer } => FaceProcs {
                issuer: world.spawn(|| Process::Issuer(Box::new(issuer))),
                visitor: world.spawn(|| Process::Visitor(Box::default())),
                peer: peer.spawn(world),
            },
        }
    }
}

/// What the model peer saw, without credential values (skein/docs/design/testing/fakes/llm.md, 3 and 3.2).
pub struct Seen {
    pub requests: Vec<api::Query>,
    pub answers: Vec<Result<(), api::Error>>,
    /// Catalogue reads, which consume no scripted model call.
    pub lists: usize,
    pub schemes: Vec<Option<skein_llm::Scheme>>,
}

#[must_use]
pub(super) fn seen(procs: &[Process], at: &FaceProcs) -> Seen {
    peer_seen(procs, at.peer)
}

pub(super) fn peer_seen(procs: &[Process], index: usize) -> Seen {
    use skein_fake_peers::llm::Observation;
    let Process::Peer(peer) = &procs[index] else { panic!("LLM peer") };
    let mut seen = Seen { requests: Vec::new(), answers: Vec::new(), lists: 0, schemes: Vec::new() };
    for event in peer.observations() {
        match event {
            Observation::Query { query, .. } => seen.requests.push(query.clone()),
            Observation::Answered { result, .. } => seen.answers.push(*result),
            Observation::Credential { scheme, .. } => seen.schemes.push(*scheme),
            Observation::List { .. } => seen.lists += 1,
            Observation::Accepted { .. } | Observation::Head { .. } | Observation::Closed { .. } => {}
        }
    }
    seen
}

/// A face and the conversation reads its box must follow before stopping.
pub struct CtlFace {
    pub face: Face,
    pub first: bool,
    /// The chat's reply the box waits for before it stops.
    pub reply: &'static str,
}

/// The plan story's face: the first box waits for the landing, the second for the restarted answer.
#[must_use]
pub fn plan_face(issuer: oauth::Peer, scripts: Box<[Script]>, first: bool) -> CtlFace {
    let scripts = if first { super::llm::requested(scripts) } else { temper_real_loop_world::llm::cued(scripts) };
    CtlFace { face: fake_face(issuer, scripts), first, reply: if first { LANDED } else { RESTARTED } }
}

/// Follow the committed reply, paging the conversation and waiting from a read's digest.
pub(super) struct Reply {
    first: bool,
    awaited: &'static str,
    snapshot: Option<String>,
    phase: ReplyPhase,
    replied: bool,
}
enum ReplyPhase {
    Snapshot,
    Pages,
    Delegates(u64),
    Done,
}
impl Reply {
    pub fn new(first: bool, awaited: &'static str) -> Self {
        Self { first, awaited, snapshot: None, phase: ReplyPhase::Snapshot, replied: false }
    }
    pub fn advance(&mut self, calls: &mut super::ctl::Calls, expected_exits: &mut Vec<(usize, u8)>) {
        use super::ctl::{Call, Step, json};
        if matches!(self.phase, ReplyPhase::Done) || !calls.foreground_idle() {
            return;
        }
        let last = json(calls.ends().last().expect("reply read"));
        if matches!(self.phase, ReplyPhase::Snapshot) {
            self.snapshot = Some(last["digest"].as_str().expect("task read digest").to_owned());
            calls.push(Step::Run(Call::new(&["chat", "show", "1", "--json"])));
            self.phase = ReplyPhase::Pages;
            return;
        }
        if let ReplyPhase::Delegates(parent) = self.phase {
            if let Some(after) = last["ended"]["rows"]["after"]["value"].as_str() {
                calls.push(Step::Run(delegate_page(parent, Some(after))));
            } else if parent == 1 {
                calls.push(Step::Run(delegate_page(2, None)));
                self.phase = ReplyPhase::Delegates(2);
            } else {
                if !self.first {
                    // Closing adds no work: task 5 must still be unknown.
                    expected_exits.push((calls.ends().len() + 3, 2));
                    for args in [
                        &["chat", "close", "1", "--key", "close-plan", "--json"][..],
                        &["task", "show", "1", "--until", "ended", "--json"],
                        &["task", "list", "--project", FIRST_PROJECT, "--json"],
                        &["task", "show", "5", "--json"],
                    ] {
                        calls.push(Step::Run(Call::new(args)));
                    }
                }
                calls.release();
                self.phase = ReplyPhase::Done;
            }
            return;
        }
        // The first box waits for the change's reply; the second, for the
        // restarted chat's answer, before it closes the chat.
        self.replied |=
            serde_json::to_string(&last["activity"]["rows"]).expect("conversation JSON").contains(self.awaited);
        if let Some(before) = last["activity"]["before"]["value"].as_str() {
            calls.push(Step::Run(Call::new(&["chat", "show", "1", "--before", before, "--json"])));
        } else if !self.replied {
            // 'Change requested' plus wait already meets replied. A snapshot
            // taken before reading the conversation prevents missing its next reply.
            calls.push(Step::Run(Call::new(&[
                "task",
                "show",
                "1",
                "--until",
                "replied",
                "--since",
                self.snapshot.as_deref().expect("snapshot before conversation"),
                "--json",
            ])));
            calls.push(Step::Run(Call::new(&["task", "show", "1", "--json"])));
            self.phase = ReplyPhase::Snapshot;
        } else {
            calls.push(Step::Run(delegate_page(1, None)));
            self.phase = ReplyPhase::Delegates(1);
        }
    }
}

fn delegate_page(parent: u64, after: Option<&str>) -> super::ctl::Call {
    let parent = parent.to_string();
    let mut args = vec!["task", "list", "--project", FIRST_PROJECT, "--parent", &parent, "--ended", "--json"];
    if let Some(after) = after {
        args.extend(["--after", after]);
    }
    super::ctl::Call::new(&args)
}

#[must_use]
pub fn restart_script(chat_cue: &[u8]) -> Box<[Script]> {
    // A system cue's turns count every assistant message (skein/docs/design/testing/fakes/llm.md 2.4):
    // the first box's delegate, wait and reply are turns 0 to 2, played there.
    // The restart runs the chat's lost attempt again, which answers (turn 3);
    // Close then asks the parked run to finish, and its text ends it (turn 4,
    // domain/agent/charter.md, 5). An Open run offers no finish tool.
    let text =
        |bytes: &[u8]| Turn { lines: Box::new([Line::Text { text: bytes.into() }]), finish: Finish::Stop, tokens: 20 };
    let played = b"Played in the first box.";
    Box::new([Script {
        cue: chat_cue.into(),
        turns: Box::new([
            text(played),
            text(played),
            text(played),
            text(RESTARTED.as_bytes()),
            text(b"The plan chat is now closed."),
        ]),
    }])
}

/// The plan chat receives the entire landed pull line, including its address.
pub fn assert_landed_chat_pull(run: &str, evidence: &super::first_repository::Evidence) {
    let pull = format!("pull request #1 of work/hello on {0}: http://{0}/work/hello/pulls/1", evidence.address);
    assert!(
        evidence.delegation.contains(&pull),
        "{run}: the chat receives the whole pull line: {}",
        evidence.delegation
    );
}

#[must_use]
pub fn landed_identity(task: &str, page: &str) -> (String, String) {
    check_landed_identity(task, page).unwrap_or_else(|why| panic!("{why}"))
}
pub fn check_landed_identity(task: &str, page: &str) -> Result<(String, String), String> {
    let (landed, _) = super::ctl::check_landed_result(task)?;
    let pull = page.lines().find(|line| line.starts_with("PR #")).ok_or("pull identity on the change page")?;
    if !pull.contains(" · work/hello · ") {
        return Err(format!("pull belongs to the first repository: {pull}"));
    }
    let number = pull.strip_prefix("PR #").ok_or("pull prefix")?.split(" · ").next().ok_or("pull number")?;
    if !landed.starts_with(&format!("result: Landed · pull request #{number} · merge commit ")) {
        return Err(landed.to_owned());
    }
    Ok((landed.to_owned(), pull.to_owned()))
}

#[must_use]
pub fn plan_ended_tasks(calls: &[super::Ended]) -> (usize, [Vec<u64>; 2]) {
    let tasks_at = calls
        .iter()
        .position(|call| {
            serde_json::from_slice::<serde_json::Value>(&call.stdout)
                .is_ok_and(|value| value["ended"].is_object() && value["live"].is_array())
        })
        .expect("ended delegate pages");
    let tasks =
        [0, 1].map(|offset| ended_tasks(&calls[tasks_at..], 1 + offset, if offset == 0 { &[2] } else { &[3, 4] }));
    (tasks_at, tasks)
}

/// The chat's ended delegates and its change's, read whole, a page at a time, by `--after`
/// and the cursor each page prints (temper/docs/design/testing/live-tier.md).
pub struct EndedDelegates {
    project: u64,
    parents: [u64; 2],
    parent: usize,
    pages: usize,
    tasks: [Vec<u64>; 2],
}

impl EndedDelegates {
    /// The pages read in all before the story fails.
    pub const PAGES: usize = 32;

    /// `parents` is `[chat, change]`.
    #[must_use]
    pub fn new(project: u64, parents: [u64; 2]) -> Self {
        Self { project, parents, parent: 0, pages: 0, tasks: [Vec::new(), Vec::new()] }
    }

    /// The first read: `task list --project P --parent CHAT --ended --json`.
    #[must_use]
    pub fn first(&self) -> Vec<String> {
        self.page(self.parents[0], None)
    }

    /// Take the read of the page last asked for: the next read (the same parent's with
    /// `--after CURSOR` appended, or the change's first), or `None` once both are whole.
    /// # Errors
    /// A live delegate (`live` not empty or `live_after` set: "no live delegates"), rows
    /// missing, a row with no `chip.task`, a page `cut` with no cursor, or a read past
    /// `PAGES`.
    pub fn read(&mut self, read: &serde_json::Value) -> Result<Option<Vec<String>>, String> {
        require(self.pages < Self::PAGES, &format!("ended delegates exceed {}-page bound", Self::PAGES))?;
        require(self.parent < self.parents.len(), "both ended delegate sets already read whole")?;
        self.pages += 1;
        require(read["live"] == serde_json::json!([]) && read["live_after"].is_null(), "no live delegates")?;
        let ended = &read["ended"]["rows"];
        let rows = ended["rows"].as_array().ok_or("ended delegate rows missing")?;
        let after = ended["after"]["value"].as_str();
        require(ended["cut"] != true || after.is_some(), "ended delegate page cut without a cursor")?;
        for row in rows {
            self.tasks[self.parent].push(row["chip"]["task"].as_u64().ok_or("delegate task missing (chip.task)")?);
        }
        if let Some(after) = after {
            return Ok(Some(self.page(self.parents[self.parent], Some(after))));
        }
        self.parent += 1;
        Ok(self.parents.get(self.parent).map(|parent| self.page(*parent, None)))
    }

    /// Both parents' ended delegates, each sorted; once `read` has answered `None`.
    #[must_use]
    pub fn whole(self) -> [Vec<u64>; 2] {
        self.check_whole().unwrap_or_else(|why| panic!("{why}"))
    }
    pub fn check_whole(mut self) -> Result<[Vec<u64>; 2], String> {
        require(self.parent == self.parents.len(), "both ended delegate sets read whole")?;
        for tasks in &mut self.tasks {
            tasks.sort_unstable();
        }
        Ok(self.tasks)
    }

    fn page(&self, parent: u64, after: Option<&str>) -> Vec<String> {
        let mut words = vec![
            "task".to_owned(),
            "list".to_owned(),
            "--project".to_owned(),
            self.project.to_string(),
            "--parent".to_owned(),
            parent.to_string(),
            "--ended".to_owned(),
            "--json".to_owned(),
        ];
        if let Some(after) = after {
            words.extend(["--after".to_owned(), after.to_owned()]);
        }
        words
    }
}

#[must_use]
pub fn ended_tasks(calls: &[super::Ended], parent: u64, expected: &[u64]) -> Vec<u64> {
    let mut tasks = Vec::new();
    let mut whole = false;
    for call in calls {
        let value = super::ctl::json(call);
        let ended = &value["ended"]["rows"];
        let rows = ended["rows"].as_array().expect("ended rows");
        if !rows.iter().any(|row| row["requester"]["task"] == parent) {
            continue;
        }
        assert_no_live_tasks(&value);
        assert_eq!(ended["cut"], false, "uncut ended task page: {value}");
        whole = ended["after"].is_null();
        for row in rows {
            assert_eq!(row["requester"]["task"], parent, "one parent's delegates per query");
            tasks.push(row["chip"]["task"].as_u64().expect("task number"));
        }
    }
    assert!(whole, "every ended task page was read");
    tasks.sort_unstable();
    assert_eq!(tasks, expected, "exact ended delegates of this parent");
    tasks
}
pub fn assert_no_live_tasks(value: &serde_json::Value) {
    assert_eq!(value["live"], serde_json::json!([]), "no live tasks in this query: {value}");
    assert!(value["live_after"].is_null(), "all live tasks read");
}
/// A project's live roots read: `task list --project PROJECT` without `--ended`.
pub fn assert_no_live_roots(value: &serde_json::Value) {
    assert_eq!(value["tasks"]["rows"], serde_json::json!([]), "no live root tasks: {value}");
    assert_eq!(value["tasks"]["more"], false, "all live root tasks read: {value}");
}

pub fn assert_reply(calls: &[super::Ended], first: bool) {
    assert_eq!(replies(calls, LANDED), 1, "one committed final reply, across all conversation pages");
    assert_eq!(replies(calls, RESTARTED), usize::from(!first), "the restarted chat answers once");
}
fn replies(calls: &[super::Ended], text: &str) -> usize {
    let mut replies = std::collections::BTreeMap::new();
    for call in calls {
        let page = super::ctl::json(call);
        let Some(rows) = page["activity"]["rows"].as_array() else { continue };
        assert_eq!(page["activity"]["cut"], false);
        for row in rows {
            let Some(turn) = row.pointer("/event/owner/decoded/turn/decoded") else { continue };
            assert_eq!(turn["cut"], false, "whole turn");
            for message in turn["messages"].as_array().expect("turn messages") {
                if message["speaker"] != "assistant" {
                    continue;
                }
                for block in message["blocks"].as_array().expect("message blocks") {
                    if block["text"]["text"] == text {
                        replies.insert(row["position"].as_u64().expect("position"), turn.clone());
                    }
                }
            }
        }
    }
    replies.len()
}

pub fn assert_requests(seen: &Seen, count: usize, words: &[u8], first: bool) {
    let mut counts = [0; 3];
    for query in &seen.requests {
        let has = |cue: &[u8]| query.system.windows(cue.len()).any(|part| part == cue);
        let kind = if (first && super::llm::request_has(query, words)) || (!first && has(words)) {
            0
        } else if has(b"## Task\n\n@coding Append this README line:") {
            1
        } else if has(b"Review this change at the named head") {
            2
        } else {
            panic!("unexpected task cue")
        };
        counts[kind] += 1;
    }
    assert_eq!(counts, if first { [3, 4, 1] } else { [2, 0, 0] }, "chat, coder, review requests");
    assert_eq!(seen.requests.len(), count, "exact scripted model requests");
    assert_eq!(seen.answers, vec![Ok(()); count], "every scripted request answered");
    assert_eq!(
        seen.schemes,
        vec![Some(skein_llm::Scheme::Bearer); count + seen.lists],
        "a bearer on every model and catalogue request"
    );
}

pub fn plan_change_steps(
    steps: &mut Vec<super::ctl::Step>,
    first: bool,
    setup_calls: usize,
    words: &str,
) -> (usize, usize) {
    change_steps(steps, first, setup_calls, words, Ends::Landed)
}

/// The first box's steps when its coder fails: the change holds, and the chat,
/// woken by the held change's escalation, answers.
pub fn held_change_steps(steps: &mut Vec<super::ctl::Step>, setup_calls: usize, words: &str) -> (usize, usize) {
    change_steps(steps, true, setup_calls, words, Ends::Held)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Ends {
    Landed,
    Held,
}

fn change_steps(
    steps: &mut Vec<super::ctl::Step>,
    first: bool,
    setup_calls: usize,
    words: &str,
    ends: Ends,
) -> (usize, usize) {
    use super::ctl::{Call, Step};
    steps.push(Step::Run(Call::new(&["me", "--json"])));
    if first {
        steps.extend([
            Step::Run(Call::new(&["account", "connect", "1", "--key", "connect-plan"])),
            Step::Run(Call::new(&["account", "list", "--until", "settled", "--json"])),
        ]);
    }
    let account_at = setup_calls + if first { 3 } else { 1 };
    steps.push(Step::Run(Call::new(&["account", "list", "--json"])));
    if first {
        steps.push(Step::Run(Call::new(&[
            "chat",
            "start",
            "--project",
            FIRST_PROJECT,
            "--model",
            "gpt-6-luna",
            "--key",
            "change-on-plan",
            words,
            "--json",
        ])));
    }
    if first {
        // Chat creation answers before its delegate exists. As in the
        // first-repository story, wait for the chat's initial reply first.
        steps.push(Step::Run(Call::new(&["task", "show", "1", "--until", "replied", "--json"])));
    }
    let found_at = account_at + 1 + 2 * usize::from(first);
    steps.extend(match ends {
        Ends::Landed => super::first_repository::found_steps(),
        // The box's referee then waits for the chat's HELD reply to the
        // escalation; a chat refused before each run never gives it.
        Ends::Held => vec![
            Step::Run(Call::new(&["task", "show", "2", "--until", "held", "--json"])),
            Step::Run(Call::new(&["task", "show", "2", "--json"])),
        ],
    });
    // The referee holds ctl open, reads and pages the conversation, and
    // follows --since if the procedure's end has not yet reached the chat.
    steps.push(Step::Run(Call::new(&["task", "show", "1", "--json"])));
    (account_at, found_at)
}

pub fn assert_plan_box<T>(run: &str, first: bool, outcome: &super::boxed::CtlOutcome<T>, stderr: &str) {
    assert_eq!(outcome.exit, Some(Exit::Code(0)), "{run}: {stderr}");
    assert_eq!(outcome.left, Some(vec![]), "{run}: {stderr}");
    for (index, call) in outcome.calls.iter().enumerate() {
        assert_eq!(
            call.exit,
            Some(Exit::Code(if !first && index == outcome.calls.len() - 1 { 2 } else { 0 })),
            "{run} call {index}: stdout {}; stderr {}; box {stderr}",
            String::from_utf8_lossy(&call.stdout),
            String::from_utf8_lossy(&call.stderr)
        );
    }
}

#[must_use]
pub fn deployment_flags(paths: &[std::path::PathBuf; 3]) -> Vec<String> {
    ["--config", "--data", "--cache"]
        .into_iter()
        .zip(paths)
        .flat_map(|(flag, path)| [flag.into(), path.display().to_string()])
        .collect()
}

#[must_use]
pub fn cache_forgejo_binaries(cache: &std::path::Path) -> ([std::path::PathBuf; 2], [std::path::PathBuf; 2]) {
    use skein_forge::forgejo::pin::{RUNNER_VERSION, VERSION};
    let (forgejo, runner) = super::owned::binaries();
    let cached = [
        cache.join(format!("forgejo/{VERSION}/forgejo")),
        cache.join(format!("forgejo-runner/{RUNNER_VERSION}/forgejo-runner")),
    ];
    let sources = [forgejo, runner];
    for (source, target) in sources.iter().zip(&cached) {
        cache_binary(source, target);
    }
    (cached, sources)
}

#[must_use]
pub fn forgejo_setup_steps(run: &str) -> Vec<super::ctl::Step> {
    use super::ctl::{Call, Step};
    let mut steps = vec![Step::Mark("start"), Step::Run(Call::sign_in())];
    if run == "first" {
        steps.extend([
            Step::Run(Call::new(&["forge", "list"])),
            Step::Run(Call::new(&["forge", "list", "--json"])),
            Step::Run(Call::new(&["forge", "setup"])),
        ]);
    }
    // Real preparation, hashing two releases, runner registration and reconcile
    // take seconds; allow three minutes, plus ten seconds to stop and reap.
    steps.extend([
        Step::Run(Call::new(&["forge", "list", "--until", "settled"])),
        Step::Run(Call::new(&["forge", "list", "--json"])),
        Step::Run(Call::new(&["forge", "list"])),
    ]);
    steps
}

pub fn cache_binary(source: &std::path::Path, target: &std::path::Path) {
    use std::{
        fs,
        os::unix::fs::{DirBuilderExt, PermissionsExt},
    };
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(target.parent().expect("cache parent"))
        .expect("private binary cache");
    fs::copy(source, target).expect("copy pinned executable, never link it");
    fs::set_permissions(target, fs::Permissions::from_mode(0o700)).expect("private executable");
}

#[must_use]
pub fn assert_forgejo_setup(
    run: &str,
    calls: &[super::Ended],
    stderr: &str,
    previous: Option<u64>,
    data: &std::path::Path,
) -> u64 {
    check_forgejo_setup(run, calls, stderr, previous, data).unwrap_or_else(|why| panic!("{why}"))
}

pub fn check_forgejo_setup(
    run: &str,
    calls: &[super::Ended],
    stderr: &str,
    previous: Option<u64>,
    data: &std::path::Path,
) -> Result<u64, String> {
    let signed = calls.first().ok_or("sign-in read")?;
    if signed.exit != Some(Exit::Code(0)) {
        return Err(format!("{run}: sign-in: {}; box: {stderr}", String::from_utf8_lossy(&signed.stderr)));
    }
    check_forgejo_setup_after_sign_in(run, &calls[1..], stderr, previous, data)
}

/// The setup reads also serve a box whose signed-in person first tried the demo.
#[must_use]
pub fn assert_forgejo_setup_after_sign_in(
    run: &str,
    calls: &[super::Ended],
    stderr: &str,
    previous: Option<u64>,
    data: &std::path::Path,
) -> u64 {
    check_forgejo_setup_after_sign_in(run, calls, stderr, previous, data).unwrap_or_else(|why| panic!("{why}"))
}

pub fn check_forgejo_setup_after_sign_in(
    run: &str,
    calls: &[super::Ended],
    stderr: &str,
    previous: Option<u64>,
    data: &std::path::Path,
) -> Result<u64, String> {
    for call in calls {
        if call.exit != Some(Exit::Code(0)) {
            return Err(format!(
                "{run}: ctl stdout: {}; stderr: {}; box: {stderr}",
                String::from_utf8_lossy(&call.stdout),
                String::from_utf8_lossy(&call.stderr)
            ));
        }
    }
    if calls.len() != if run == "first" { 6 } else { 3 } {
        return Err(format!("all setup calls ran: {stderr}"));
    }
    let made = super::ctl::try_json(&calls[if run == "first" { 4 } else { 1 }])?;
    let row = super::first_repository::check_owned_row(&made)?;
    let number = row["number"].as_u64().ok_or("forge number")?;
    let host = row["host"].as_str().filter(|host| !host.is_empty()).ok_or("forge resource host")?;
    if run == "first" {
        check_first_setup(run, calls, stderr, row, number)?;
    } else if stderr.lines().any(|line| line.starts_with("fetch ") || line.contains("download")) {
        return Err(format!("the restarted group fetches nothing: {stderr}"));
    }
    let stdout = String::from_utf8_lossy(&calls.last().ok_or("forges read")?.stdout);
    let user = stdout.lines().find_map(|line| {
        line.strip_prefix(&format!("forge {number} · host: {host} · Forgejo · owned · Connected as temper ("))?
            .split(')')
            .next()?
            .parse::<u64>()
            .ok()
    });
    if user.is_none_or(|user| user == 0) {
        return Err(format!("{run}: forges row: {stdout}; box: {stderr}"));
    }
    let user = user.ok_or("connected bot's ID")?;
    if let Some(previous) = previous
        && user != previous
    {
        return Err("the second box connects as the same bot".into());
    }
    for name in ["admin_token", "api_token", "git_token", "user", "webhook_secret"] {
        if data.join(format!("forge-{number}")).join(name).exists() {
            return Err(format!("{run}: no forge-{number}/{name}"));
        }
    }
    Ok(user)
}

fn check_first_setup(
    run: &str,
    calls: &[super::Ended],
    stderr: &str,
    row: &serde_json::Value,
    number: u64,
) -> Result<(), String> {
    let offered = super::ctl::try_json(&calls[1])?;
    if offered["forges"] != serde_json::json!([]) {
        return Err("a fresh store has no forge records".into());
    }
    if String::from_utf8_lossy(&calls[0].stdout).lines().any(|line| line.starts_with("forge ")) {
        return Err(format!(
            "assertion failed: {}",
            stringify!(!String::from_utf8_lossy(&calls[0].stdout).lines().any(|line| line.starts_with("forge ")))
        ));
    }
    if number != 1 {
        return Err("the first forge record is numbered one".into());
    }
    if !calls[2].stdout.starts_with(b"setting up\n") {
        return Err(format!("assertion failed: {}", stringify!(calls[2].stdout.starts_with(b"setting up\n"))));
    }
    let adoption = String::from_utf8_lossy(&calls[2].stdout);
    if !adoption.contains(&format!("Adopted work/hello on forge {number} as owned.")) {
        return Err(format!("ctl printed the first adoption: {adoption}"));
    }
    eprintln!("{run} first repository adoption: {adoption}");
    eprintln!("{run} first repository read: {}", row["first"]);
    if row["first"]["state"] != "made" {
        return Err(format!("assertion failed: left == right; left: {:?}; right: {:?}", row["first"]["state"], "made"));
    }
    if !adoption.lines().any(|line| line == "project: 1 · hello") {
        return Err(format!("{adoption}"));
    }
    if !adoption.contains("It is the project's home.") {
        return Err(format!("{adoption}"));
    }
    if row["first"]["org"] != "work" {
        return Err(format!("assertion failed: left == right; left: {:?}; right: {:?}", row["first"]["org"], "work"));
    }
    if row["first"]["repository"] != "hello" {
        return Err(format!(
            "assertion failed: left == right; left: {:?}; right: {:?}",
            row["first"]["repository"], "hello"
        ));
    }
    if row["first"]["id"] != 1 {
        return Err("the script tags the first forge and repository".into());
    }
    if row["state"] != "connected" {
        return Err(format!("assertion failed: left == right; left: {:?}; right: {:?}", row["state"], "connected"));
    }
    // Preparation is silent on success; Forgejo's startup
    // and the runner's lines reach the box with their step prefixes.
    for step in [format!("forgejo-{number}: "), format!("runner-{number}: ")] {
        if !stderr.lines().any(|line| line.starts_with(&step)) {
            return Err(format!("group step {step}: {stderr}"));
        }
    }
    Ok(())
}

/// The SUT's cached copy still contains every byte of its pinned source.
pub fn assert_cached_binary_unchanged(cached: &std::path::Path, source: &std::path::Path) {
    use std::io::Read;
    let mut cached = std::fs::File::open(cached).expect("cached binary");
    let mut source = std::fs::File::open(source).expect("pinned source binary");
    assert_eq!(
        cached.metadata().expect("cached metadata").len(),
        source.metadata().expect("source metadata").len(),
        "cached binary unchanged"
    );
    let mut cached_bytes = vec![0; 262_144].into_boxed_slice();
    let mut source_bytes = vec![0; 262_144].into_boxed_slice();
    loop {
        let count = cached.read(&mut cached_bytes).expect("cached binary chunk");
        source.read_exact(&mut source_bytes[..count]).expect("pinned source chunk");
        assert!(cached_bytes[..count] == source_bytes[..count], "cached binary unchanged");
        if count == 0 {
            assert_eq!(source.read(&mut source_bytes).expect("pinned source EOF"), 0, "cached binary unchanged");
            break;
        }
    }
}

/// Init's shipped charters on the process story's three roots. The model
/// choice changes only ordinary provider and program configuration.
#[must_use]
pub fn live_deployment(paths: &[PathBuf; 3], model: LiveModel<'_>) -> std::net::SocketAddr {
    initialize(paths);
    super::own_listener_files(&paths[0]);
    if let LiveModel::ClaudeCode { .. } = model {
        let executable = temper_test_env::value(&temper_shell::environment::PATH)
            .into_iter()
            .flat_map(|path| std::env::split_paths(&path).collect::<Vec<_>>())
            .filter(|directory| directory.is_absolute())
            .map(|directory| directory.join("claude"))
            .find(|path| {
                fs::metadata(path)
                    .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
            })
            .expect("Claude Code must be installed on PATH");
        configure_live_model(&paths[0], model, &executable);
    }
    if let LiveModel::AnthropicKey { .. } = model {
        configure_live_model(&paths[0], model, std::path::Path::new(""));
    }
    let path = paths[0].join("temper.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    value["engine"]["limits"]["overrides"]["accounts.sign_in_deadline_ms"] = 600_000.into();
    super::write_private(&path, &serde_json::to_vec(&value).expect("live configuration"));
    value["application"]["machine"]["owned"]["listen"]["socket"]
        .as_str()
        .expect("owned listen")
        .parse()
        .expect("socket")
}

/// Select the launcher's store and admit it before starting any process; a
/// held account moves into a fresh store only.
#[must_use]
pub fn live_store(account: bool) -> (PathBuf, bool) {
    let store = std::path::PathBuf::from(
        temper_test_env::value(&temper_test_env::LIVE_STORE)
            .expect("run live through scripts/proof-live.sh forge-land"),
    );
    let kept = reserve_account_store(&store, account).unwrap_or_else(|message| {
        super::live::notice(&message);
        panic!("{message}")
    });
    (store, kept)
}

/// The owned-forge landing story's own store under `TEMPER_TEST_LIVE_STORE`, so the pair runs
/// in one invocation, each on its own deployment (testing/live-tier.md).
#[must_use]
pub fn landing_store(model: LiveModel<'_>) -> (PathBuf, bool) {
    let stores = PathBuf::from(
        temper_test_env::value(&temper_test_env::LIVE_STORE)
            .expect("run live through scripts/proof-live.sh forge-land"),
    );
    story_store(&stores, model).unwrap_or_else(|message| {
        super::live::notice(&message);
        panic!("{message}")
    })
}

/// Make the stores' directory (0700, with its parents) when absent, then reserve
/// `stores/<story>` as `reserve_live_store` does; a held `ChatGPT` record moves into a fresh
/// store only.
pub fn story_store(stores: &std::path::Path, model: LiveModel<'_>) -> Result<(PathBuf, bool), String> {
    use std::os::unix::fs::DirBuilderExt;
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(stores)
        .map_err(|error| format!("create stores directory {}: {error}", stores.display()))?;
    let store = stores.join(model.story());
    let kept = reserve_account_store(&store, matches!(model, LiveModel::ChatGpt { account: Some(_) }))?;
    Ok((store, kept))
}

fn reserve_account_store(store: &std::path::Path, account: bool) -> Result<bool, String> {
    reserve_live_store(store).and_then(|kept| {
        if kept && account { Err("a held account moves into a fresh store only".to_owned()) } else { Ok(kept) }
    })
}

/// The held record `TEMPER_TEST_LIVE_ACCOUNT` names, admitted before any store is.
#[must_use]
pub fn live_account() -> Option<PathBuf> {
    let account = PathBuf::from(temper_test_env::value(&temper_test_env::LIVE_ACCOUNT)?);
    if let Err(message) = held_account(&account) {
        super::live::notice(message);
        panic!("{message}");
    }
    Some(account)
}

/// Admit the held `ChatGPT` record a fresh store's first run moves in: a
/// regular file only its owner reads, as ctl account import's FILE must be. Its words
/// never name the path, which goes to `account import` alone.
pub fn held_account(file: &std::path::Path) -> Result<(), &'static str> {
    use std::os::unix::fs::MetadataExt;
    let metadata = fs::symlink_metadata(file).map_err(|_| "TEMPER_TEST_LIVE_ACCOUNT names no file")?;
    if !metadata.file_type().is_file() {
        return Err("TEMPER_TEST_LIVE_ACCOUNT is not a regular file");
    }
    if metadata.mode() & 0o7777 != 0o600 {
        return Err("TEMPER_TEST_LIVE_ACCOUNT is not 0600");
    }
    Ok(())
}

/// Admit the key by metadata alone; refusal words never include its path.
/// Unset, the caller skips before reserving a store or starting a process.
pub fn anthropic_key(value: Option<std::ffi::OsString>) -> Result<Option<PathBuf>, &'static str> {
    use std::os::unix::fs::MetadataExt;
    let Some(value) = value else { return Ok(None) };
    let path = PathBuf::from(value);
    let metadata = fs::symlink_metadata(&path).map_err(|_| "TEMPER_TEST_LIVE_ANTHROPIC_KEY names no file")?;
    if !metadata.is_file() {
        return Err("TEMPER_TEST_LIVE_ANTHROPIC_KEY is not a regular file or is a link");
    }
    if metadata.mode() & 0o7777 != 0o600 {
        return Err("TEMPER_TEST_LIVE_ANTHROPIC_KEY is not 0600");
    }
    if metadata.uid() != skein_shell::effective_user() {
        return Err("TEMPER_TEST_LIVE_ANTHROPIC_KEY is not owned by this user");
    }
    if metadata.len() == 0 {
        return Err("TEMPER_TEST_LIVE_ANTHROPIC_KEY is empty");
    }
    Ok(Some(path))
}

/// A live command as notices and failures print it: an imported record or added
/// key's path stays out of them, and ctl's own words are all that is said of it.
#[must_use]
pub fn printed_command(args: &[String]) -> Vec<String> {
    match args {
        [account, add_key, provider, _file, rest @ ..] if account == "account" && add_key == "add-key" => {
            [account.clone(), add_key.clone(), provider.clone(), "FILE".into()]
                .into_iter()
                .chain(rest.iter().cloned())
                .collect()
        }
        [account, import, _file, rest @ ..] if account == "account" && import == "import" => {
            [account.clone(), import.clone(), "FILE".into()].into_iter().chain(rest.iter().cloned()).collect()
        }
        _ => args.to_vec(),
    }
}

/// A unique marker for this live sitting, carried into its README and notice.
#[must_use]
pub fn live_tag() -> String {
    let minute = std::process::Command::new("/usr/bin/date")
        .args(["-u", "+%Y-%m-%dT%H:%MZ"])
        .env_clear()
        .output()
        .expect("UTC ISO minute");
    assert!(minute.status.success());
    format!("live-{}-{}", String::from_utf8(minute.stdout).expect("date UTF-8").trim(), std::process::id())
}

/// Reserve a fresh store, or validate a kept deployment using metadata only.
/// An empty directory is fresh; an incomplete deployment is never repaired.
pub fn reserve_live_store(path: &std::path::Path) -> Result<bool, String> {
    use skein_forge::forgejo::pin::{RUNNER_VERSION, VERSION};
    use std::os::unix::fs::DirBuilderExt;
    match fs::DirBuilder::new().mode(0o700).create(path) {
        Ok(()) => return Ok(false),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(format!("create kept store {}: {error}", path.display())),
    }
    let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    require(metadata.is_dir(), &format!("kept store {} is not a directory", path.display()))?;
    require(metadata.permissions().mode() & 0o777 == 0o700, "kept store must have mode 0700")?;
    if fs::read_dir(path)
        .map_err(|error| error.to_string())?
        .next()
        .transpose()
        .map_err(|error| error.to_string())?
        .is_none()
    {
        return Ok(false);
    }
    // The deployment alone: a rerun makes its own client roots, and a sitting
    // that failed before its restart never made `second`.
    for (name, directory) in [
        ("config", true),
        ("data", true),
        ("cache", true),
        ("data/deployment", true),
        ("data/forge-1/forgejo", true),
        ("config/temper.json", false),
        ("config/agent.json", false),
        ("data/box/started/1", false),
        ("data/forge-1/forgejo/app.ini", false),
        ("data/forge-1/forgejo/forgejo.db", false),
    ] {
        let present = fs::symlink_metadata(path.join(name))
            .is_ok_and(|meta| if directory { meta.is_dir() } else { meta.is_file() });
        require(present, &format!("kept store {} lacks {}", path.display(), name))?;
    }
    for name in
        [format!("cache/forgejo/{VERSION}/forgejo"), format!("cache/forgejo-runner/{RUNNER_VERSION}/forgejo-runner")]
    {
        require(
            fs::symlink_metadata(path.join(&name)).is_ok_and(|meta| meta.is_file()),
            &format!("kept store {} lacks {}", path.display(), name),
        )?;
    }
    Ok(true)
}

pub const KEPT_SIGN_IN_EXPIRED: &str = "the kept ChatGPT sign-in is expired or refused; the person signs in afresh";

/// The forge's durable identity, read from this run's private owned repository.
#[derive(Debug, PartialEq, Eq)]
pub struct ForgeIdentity {
    pub number: u64,
    pub url: String,
    pub merge: String,
    pub head: String,
    pub branches: Vec<(String, String)>,
}

pub struct LiveEvidence {
    pub task: String,
    pub page: String,
    pub completions: u32,
    pub address: std::net::SocketAddr,
    pub tasks: LiveTasks,
    pub baseline: std::collections::BTreeSet<u64>,
    pub roots: std::collections::BTreeSet<u64>,
    pub forge: ForgeIdentity,
    pub replies: std::collections::BTreeMap<u64, serde_json::Value>,
    pub delegates: [Vec<u64>; 2],
}

fn pull_number(pull: &serde_json::Value) -> Result<u64, String> {
    pull["number"].as_u64().ok_or_else(|| "pull number".into())
}
fn merged_pulls(address: std::net::SocketAddr, token: &str) -> Result<Vec<serde_json::Value>, String> {
    let mut merged = Vec::new();
    for page in 1..=100 {
        let pulls = super::owned::answer(
            address.port(),
            &format!("/api/v1/repos/work/hello/pulls?state=closed&limit=100&page={page}"),
            Some(token),
        )
        .map_err(|error| format!("owned forge pulls: {error}"))?;
        let pulls = pulls.as_array().ok_or("pull list")?;
        if pulls.is_empty() {
            return Ok(merged);
        }
        merged.extend(pulls.iter().filter(|pull| pull["merged"] == true).cloned());
    }
    Err("merged pull pagination exceeded 100 pages".into())
}

#[must_use]
pub fn last_readme_line(readme: &[u8]) -> Option<&str> {
    check_last_readme_line(readme).unwrap_or_else(|why| panic!("{why}"))
}
pub fn check_last_readme_line(readme: &[u8]) -> Result<Option<&str>, String> {
    Ok(std::str::from_utf8(readme)
        .map_err(|error| format!("README UTF-8: {error}"))?
        .lines()
        .rfind(|line| !line.is_empty()))
}

fn forge_identity(
    address: std::net::SocketAddr,
    line: &str,
    token: &str,
    baseline: &std::collections::BTreeSet<u64>,
) -> Result<ForgeIdentity, String> {
    use skein_lib::base64::{self, Alphabet};
    let get = |suffix: &str| {
        super::owned::answer(address.port(), &format!("/api/v1/repos/work/hello/{suffix}"), Some(token))
            .map_err(|why| format!("owned forge read {suffix}: {why}"))
    };
    let pulls = merged_pulls(address, token)?;
    let numbers: std::collections::BTreeSet<_> = pulls.iter().map(pull_number).collect::<Result<_, _>>()?;
    if !baseline.is_subset(&numbers) {
        return Err("previous merged pulls survive".into());
    }
    if numbers.len() != baseline.len() + 1 {
        return Err("exactly one more merged pull".into());
    }
    let pull = pulls
        .iter()
        .find(|pull| pull_number(pull).is_ok_and(|number| !baseline.contains(&number)))
        .ok_or("new merged pull")?;
    let head = pull["head"]["sha"].as_str().ok_or("pull head")?.to_owned();
    let status = get(&format!("commits/{head}/status"))?;
    if status["state"] != "success" {
        return Err("CI at the merged head".into());
    }
    if status["statuses"].as_array().ok_or("CI jobs")?.is_empty() {
        return Err("a CI job ran".into());
    }
    let readme = get(&format!("contents/README.md?ref={head}"))?;
    if readme["encoding"] != "base64" {
        return Err(format!("assertion failed: left == right; left: {:?}; right: {:?}", readme["encoding"], "base64"));
    }
    let encoded: Vec<_> = readme["content"]
        .as_str()
        .ok_or("README content")?
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();
    let readme =
        base64::decode(Alphabet::Standard, &encoded, 1 << 20).map_err(|error| format!("README base64: {error:?}"))?;
    if check_last_readme_line(&readme)? != Some(line) {
        return Err("the merged pull head ends with the tagged line".into());
    }
    let branches = get("branches?limit=100&page=1")?;
    let next_branches = get("branches?limit=100&page=2")?;
    if next_branches != serde_json::json!([]) {
        return Err(format!(
            "assertion failed: left == right; left: {:?}; right: {:?}",
            next_branches,
            serde_json::json!([])
        ));
    }
    let mut branches: Vec<_> = branches
        .as_array()
        .ok_or("branches")?
        .iter()
        .map(|branch| {
            Ok((
                branch["name"].as_str().ok_or("branch name")?.to_owned(),
                branch["commit"]["id"].as_str().ok_or("branch head")?.to_owned(),
            ))
        })
        .collect::<Result<_, String>>()?;
    branches.sort();
    let merge = pull["merge_commit_sha"].as_str().ok_or("merge hash")?.to_owned();
    if !branches.iter().any(|(name, head)| name == "main" && head == &merge) {
        return Err("main is at the pull's merge commit".into());
    }
    Ok(ForgeIdentity {
        number: pull["number"].as_u64().ok_or("pull number")?,
        url: pull["html_url"].as_str().ok_or("pull URL")?.to_owned(),
        merge,
        head,
        branches,
    })
}

/// The starter is private. Mint an independent, read-only oracle credential on
/// this disposable forge, rather than reading temper's stored credentials.
fn oracle_token(paths: &[PathBuf; 3], first: bool, forge: u64) -> Result<String, String> {
    check_scoped_token(paths, first, forge, "temper", "read:repository,read:issue")
}

/// A separate fixture credential, never one of temper's stored credentials.
#[must_use]
pub(super) fn scoped_token(paths: &[PathBuf; 3], first: bool, number: u64, login: &str, scopes: &str) -> String {
    check_scoped_token(paths, first, number, login, scopes).unwrap_or_else(|why| panic!("{why}"))
}
pub(super) fn check_scoped_token(
    paths: &[PathBuf; 3],
    first: bool,
    number: u64,
    login: &str,
    scopes: &str,
) -> Result<String, String> {
    use std::{
        io::Read,
        process::{Command, Stdio},
    };
    let forge = paths[1].join(format!("forge-{number}/forgejo"));
    let (binary, _) = super::owned::binaries();
    let mut child = Command::new(binary)
        .arg("--work-path")
        .arg(&forge)
        .arg("--config")
        .arg(forge.join("app.ini"))
        .args([
            "admin",
            "user",
            "generate-access-token",
            "--username",
            login,
            "--token-name",
            &format!(
                "live-oracle-{}-{}-{}",
                std::process::id(),
                if first { "first" } else { "restart" },
                skein_shell::Clock::new().now().now.as_nanos()
            ),
            "--scopes",
            scopes,
            "--raw",
        ])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        // The owned forge commands use this same private home.
        .env("HOME", paths[1].join(format!("forge-{number}/home")))
        .current_dir(&forge)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("oracle token command: {error}"))?;
    let status = child.wait().map_err(|error| format!("oracle token exit: {error}"))?;
    if !status.success() {
        return Err("read-only oracle token creation failed".into());
    }
    let mut bytes = Vec::new();
    child
        .stdout
        .take()
        .ok_or("oracle token output")?
        .take(1024)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("oracle token bytes: {error}"))?;
    let token = String::from_utf8(bytes).map_err(|error| format!("oracle token encoding: {error}"))?;
    let token = token.trim();
    if token.len() != 40 {
        return Err("oracle token length".into());
    }
    if !token.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("oracle token format".into());
    }
    Ok(token.to_owned())
}

/// Commands are admitted one at a time so a failure cannot start the next act.
/// A fresh store's account comes from a person's sign-in, or from a held
/// record moved in by `import` (client/ctl/areas.md), or a
/// held Claude Code directory declared and verified on the worker. A held `ChatGPT`
/// record is imported again after its first box returned it, with a distinct key.
#[must_use]
pub fn live_commands(fresh: bool, model: LiveModel<'_>) -> (Vec<Vec<String>>, usize) {
    let mut commands = vec![vec!["sign-in", "-"]];
    if fresh {
        commands.extend([vec!["forge", "list"], vec!["forge", "list", "--json"], vec!["forge", "setup"]]);
    }
    commands.extend([
        vec!["forge", "list", "--until", "settled", "--timeout", "180s"],
        vec!["forge", "list", "--json"],
        vec!["forge", "list"],
        vec!["me", "--json"],
    ]);
    if fresh {
        match model {
            LiveModel::ChatGpt { account } => {
                commands.extend(match account.map(|file| file.to_str().expect("held account path is UTF-8")) {
                    Some(file) => [
                        vec!["account", "import", file, "--key", "connect-plan"],
                        vec!["account", "list", "--until", "usable", "--timeout", "600s", "--json"],
                    ],
                    None => [
                        vec!["account", "connect", "1", "--key", "connect-plan"],
                        vec!["account", "list", "--until", "settled", "--timeout", "600s", "--json"],
                    ],
                });
            }
            LiveModel::AnthropicKey { key } => {
                commands.extend([
                    vec!["account", "add-key", "2", key.to_str().expect("key path is UTF-8"), "--key", "live-key"],
                    vec!["account", "list", "--until", "usable", "--timeout", "600s", "--json"],
                ]);
            }
            LiveModel::ClaudeCode { login } => {
                commands.push(vec![
                    "account",
                    "declare",
                    "2",
                    "claude_code",
                    source_name(login),
                    "--key",
                    "live-program",
                ]);
            }
        }
    }
    if !fresh && let LiveModel::ChatGpt { account: Some(file) } = model {
        commands.extend([
            vec!["account", "import", file.to_str().expect("held account path is UTF-8"), "--key", "restart-plan"],
            vec!["account", "list", "--until", "usable", "--timeout", "600s", "--json"],
        ]);
    }
    let account = commands.len();
    commands.push(vec!["account", "list", "--json"]);
    (commands.into_iter().map(|args| args.into_iter().map(str::to_owned).collect()).collect(), account)
}

/// The live root task numbers of a `task list --project PROJECT --json` read, whole.
pub fn live_roots(read: &serde_json::Value) -> Result<std::collections::BTreeSet<u64>, String> {
    require(read["tasks"]["more"] == false, "whole live root tasks read")?;
    read["tasks"]["rows"]
        .as_array()
        .ok_or("live root task rows missing")?
        .iter()
        .map(|row| row["chip"]["task"].as_u64().ok_or_else(|| "live root task number missing".to_owned()))
        .collect()
}

/// After the chat closes, the project's live roots are exactly those before this sitting.
pub fn require_closed_roots(read: &serde_json::Value, before: &std::collections::BTreeSet<u64>) -> Result<(), String> {
    let after = live_roots(read)?;
    let extra = after.difference(before).copied().collect::<Vec<_>>();
    let missing = before.difference(&after).copied().collect::<Vec<_>>();
    require(
        extra.is_empty() && missing.is_empty(),
        &format!("live root tasks this sitting left: {extra:?}; earlier live root tasks missing: {missing:?}"),
    )
}

/// The source selects the story's model, without copying a login or key.
#[derive(Clone, Copy)]
pub enum LiveModel<'a> {
    ChatGpt { account: Option<&'a std::path::Path> },
    ClaudeCode { login: &'a std::path::Path },
    AnthropicKey { key: &'a std::path::Path },
}

fn source_name(login: &std::path::Path) -> &str {
    let name = login.file_name().and_then(|name| name.to_str()).expect("held login source name");
    let _name = smith_host_domain::SourceName::new(name.as_bytes()).expect("valid held login source name");
    name
}

impl LiveModel<'_> {
    /// The story's launcher argument, and its store's name under `TEMPER_TEST_LIVE_STORE`.
    #[must_use]
    pub const fn story(self) -> &'static str {
        match self {
            Self::ChatGpt { .. } => "forge-land",
            Self::ClaudeCode { .. } => "forge-land-claude",
            Self::AnthropicKey { .. } => "forge-land-claude-key",
        }
    }

    /// The model every charter the story selects runs, and the chat's `--model`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ChatGpt { .. } => "gpt-6-luna",
            Self::ClaudeCode { .. } | Self::AnthropicKey { .. } => "claude-haiku-5-5",
        }
    }
    /// The stock row read after startup: the selected model's own, or the
    /// provider's dearest, whose figures a model taken by name copies
    /// (jig's `domain/engine/settings.md`, 11).
    const fn number(self) -> &'static str {
        match self {
            Self::ChatGpt { .. } => "7",
            Self::ClaudeCode { .. } | Self::AnthropicKey { .. } => "3",
        }
    }
    const fn row(self) -> &'static str {
        match self {
            Self::ChatGpt { .. } => "gpt-6-luna",
            Self::ClaudeCode { .. } | Self::AnthropicKey { .. } => "claude-fable-5-1",
        }
    }
    /// A model taken by name: its default effort; `None` for a stock row.
    const fn by_name(self) -> Option<&'static str> {
        match self {
            Self::ChatGpt { .. } => None,
            Self::ClaudeCode { .. } | Self::AnthropicKey { .. } => Some("low"),
        }
    }
    const fn unavailable(self) -> &'static str {
        match self {
            Self::ChatGpt { .. } => KEPT_SIGN_IN_EXPIRED,
            Self::AnthropicKey { .. } => {
                "the Anthropic key is refused or limited; check TEMPER_TEST_LIVE_ANTHROPIC_KEY"
            }
            Self::ClaudeCode { .. } => "the held Claude Code login is not usable; stop and return it to its holder",
        }
    }
    pub fn ready(self, value: &serde_json::Value) -> Result<(), String> {
        match self {
            Self::ChatGpt { .. } => super::live::ready_account(value),
            Self::AnthropicKey { .. } => {
                let accounts = value["accounts"]["party"].as_array().ok_or("person's accounts missing")?;
                let sources: Vec<_> = accounts
                    .iter()
                    .filter(|account| account["provider"] == 2)
                    .filter_map(|account| account["sources"].as_array())
                    .flatten()
                    .filter(|source| source["kind"] == "key")
                    .collect();
                require(!sources.is_empty(), "Anthropic provider 2 key source missing")?;
                require(
                    sources.iter().any(|source| source["usable"] == true),
                    "Anthropic provider 2 key source is not usable",
                )
            }
            Self::ClaudeCode { login } => {
                let account = declared_program(value, source_name(login))?;
                require(
                    account["sources"].as_array().ok_or("account sources missing")?.iter().any(|source| {
                        source["kind"] == "program" && source["name"] == source_name(login) && source["usable"] == true
                    }),
                    "held Claude Code source is not usable",
                )?;
                let hosts = value["accounts"]["program_hosts"].as_array().ok_or("program hosts missing")?;
                require(
                    hosts.iter().any(|host| {
                        host["name"] == "worker"
                            && host["connected"] == true
                            && host["programs"].as_array().is_some_and(|programs| {
                                programs.iter().any(|program| {
                                    program["name"] == source_name(login) && program["state"] == "usable"
                                })
                            })
                    }),
                    "held Claude Code login is not usable on the worker",
                )
            }
        }
    }
}

fn declared_program<'a>(value: &'a serde_json::Value, name: &str) -> Result<&'a serde_json::Value, String> {
    let accounts = value["accounts"]["party"].as_array().ok_or("person's accounts missing")?;
    accounts
        .iter()
        .find(|account| {
            account["provider"] == 2
                && account["sources"].as_array().is_some_and(|sources| {
                    sources.iter().any(|source| source["kind"] == "program" && source["name"] == name)
                })
        })
        .ok_or_else(|| "held Claude Code account missing".to_owned())
}

/// Verify the declared account number returned by ctl, then wait for the host.
pub fn program_verification(login: &std::path::Path, accounts: &serde_json::Value) -> Result<Vec<Vec<String>>, String> {
    let account = declared_program(accounts, source_name(login))?["number"]
        .as_u64()
        .ok_or("program account number missing")?
        .to_string();
    Ok([
        vec!["account", "verify", &account, "--source", "program", "--json"],
        vec!["account", "list", "--until", "usable", "--timeout", "600s", "--json"],
        vec!["account", "list", "--json"],
    ]
    .into_iter()
    .map(|args| args.into_iter().map(str::to_owned).collect())
    .collect())
}

/// Keep the live `OpenAI` provider and demo; the owner selects the model after startup.
pub fn retain_live_model(path: &std::path::Path, name: &str) {
    assert_eq!(name, LiveModel::ChatGpt { account: None }.name());
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(path).expect("deployment")).expect("JSON");
    for table in ["providers", "endpoints"] {
        value["engine"]["llm"][table].as_array_mut().expect("LLM table").retain(|row| row["name"] != "anthropic");
    }
    super::write_private(path, &serde_json::to_vec(&value).expect("deployment JSON"));
}

pub fn configure_live_model(config: &std::path::Path, model: LiveModel<'_>, executable: &std::path::Path) {
    if let LiveModel::AnthropicKey { .. } = model {
        let path = config.join("temper.json");
        let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("deployment")).expect("JSON");
        let provider = value["engine"]["llm"]["providers"]
            .as_array_mut()
            .expect("providers")
            .iter_mut()
            .find(|provider| provider["name"] == "anthropic")
            .expect("init Anthropic provider")
            .as_object_mut()
            .expect("provider");
        provider.remove("keep");
        provider.remove("drop");
        super::write_private(&path, &serde_json::to_vec(&value).expect("key configuration"));
        return;
    }
    let LiveModel::ClaudeCode { login } = model else { return };
    let path = config.join("temper.json");
    let agent_path = config.join("agent.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("deployment")).expect("JSON");
    let mut agent: serde_json::Value = serde_json::from_slice(&fs::read(&agent_path).expect("agent")).expect("JSON");
    let mut endpoint = agent["endpoints"]
        .as_array()
        .expect("agent endpoints")
        .iter()
        .find(|endpoint| endpoint["name"] == "openai")
        .expect("init OpenAI endpoint")
        .clone();
    endpoint["name"] = "claude".into();
    endpoint["number"] = 2.into();
    endpoint["dialect"] = 2.into();
    endpoint["provider"] = "anthropic".into();
    endpoint["program"] = "claude_code".into();
    endpoint["address"] = "api.anthropic.com:443".into();
    endpoint["server_name"] = "api.anthropic.com".into();
    let mut route = value["engine"]["llm"]["endpoints"]
        .as_array()
        .expect("engine endpoints")
        .iter()
        .find(|endpoint| endpoint["name"] == "openai")
        .expect("init OpenAI endpoint")
        .clone();
    for key in ["name", "number", "dialect", "provider", "program", "server_name"] {
        route[key] = endpoint[key].clone();
    }
    route["address"] = serde_json::json!({"host":"api.anthropic.com", "port":443});
    let provider = serde_json::json!({"name":"anthropic", "number":2, "signed_in_plan":false,
        "manage_usage":"https://claude.ai/settings/usage", "endpoint":"claude",
        "sources":{"programs":["claude_code"]}
    });
    for (table, field, replacement) in [("providers", "name", provider), ("endpoints", "name", route)] {
        let entries = value["engine"]["llm"][table].as_array_mut().expect("init LLM table");
        entries
            .retain(|entry| entry[field] != "openai" && entry[field] != "anthropic" && entry[field] != "anthropic-api");
        entries.insert(0, replacement);
    }
    let endpoints = agent["endpoints"].as_array_mut().expect("agent endpoints");
    endpoints.retain(|endpoint| {
        endpoint["name"] != "openai" && endpoint["name"] != "anthropic" && endpoint["name"] != "anthropic-api"
    });
    endpoints.insert(0, endpoint);
    if agent["programs"].is_null() {
        agent["programs"] = serde_json::json!({});
    }
    agent["programs"]["claude_code"] = serde_json::json!({
        "path":executable, "directory":login.parent().expect("held login parent")
    });
    super::write_private(&path, &serde_json::to_vec(&value).expect("live configuration"));
    super::write_private(&agent_path, &serde_json::to_vec(&agent).expect("live agent"));
}

/// A settings change a fresh sitting applies, and its `settings apply`'s key.
#[derive(Debug, PartialEq)]
pub struct LiveChange {
    pub key: &'static str,
    pub file: serde_json::Value,
}

/// The settings changes a fresh sitting applies, in order (testing/live-tier.md):
/// a model taken by name first, alone, at the highest model number the inventory
/// leaves free, from the stock row it copies (`live-model`); then charters 1 and 2
/// naming the story's model alone (`live-settings`).
pub fn live_settings(
    model: LiveModel<'_>,
    inventory: &serde_json::Value,
    charters: [&serde_json::Value; 2],
    row: &serde_json::Value,
) -> Result<Vec<LiveChange>, String> {
    require(row["value"]["model"]["name"] == model.row(), &format!("the selected live stock model: {}", model.row()))?;
    let mut value = row["value"].clone();
    let mut changes = Vec::new();
    if let Some(effort) = model.by_name() {
        let settings = inventory["settings"].as_array().ok_or("settings inventory missing")?;
        let number = (1..=jig_domain_tasks::MODELS)
            .rev()
            .find(|number| {
                !settings.iter().any(|line| line["setting"]["kind"] == "model" && line["setting"]["number"] == *number)
            })
            .ok_or_else(|| format!("no free model number for {}", model.name()))?;
        require(
            value["model"]["efforts"].as_array().is_some_and(|efforts| efforts.iter().any(|offered| offered == effort)),
            &format!("the selected live stock model {} offers no {effort} effort for {}", model.row(), model.name()),
        )?;
        value["number"] = number.into();
        value["model"]["name"] = model.name().into();
        value["model"]["effort"] = effort.into();
        value["shown"] = serde_json::Value::Null;
        value["description"] = serde_json::Value::Null;
        value["picker"] = true.into();
        value["withdrawn"] = false.into();
        value["source"] = "person".into();
        value["estimated"] = true.into();
        changes.push(LiveChange {
            key: "live-model",
            file: serde_json::json!({"items":[{"setting":{"kind":"model","number":number},"set":value}]}),
        });
    }
    let mut items = Vec::new();
    for charter in charters {
        let mut charter_value = charter["value"].clone();
        charter_value["model"] = value["model"].clone();
        charter_value["alternatives"] = serde_json::json!([]);
        charter_value["rest"] = false.into();
        charter_value["time"] = 240_000_000_000_u64.into();
        items.push(serde_json::json!({"setting":charter["setting"],"set":charter_value}));
    }
    changes.push(LiveChange { key: "live-settings", file: serde_json::json!({"items":items}) });
    Ok(changes)
}

/// The `settings apply` of the change at `at` in `live_settings`' order, read from
/// `file`: the first against the inventory's `newest`, each later one against the
/// revision the one before it made.
#[must_use]
pub fn live_apply(change: &LiveChange, at: usize, newest: u64, file: &str) -> Vec<String> {
    let revision =
        newest.checked_add(u64::try_from(at).expect("change index fits revision")).expect("settings revision fits");
    ["settings", "apply", file, "--against", &revision.to_string(), "--key", change.key]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

pub struct LivePlan<'a> {
    pub paths: &'a [PathBuf; 3],
    pub client: &'a std::path::Path,
    pub first: bool,
    pub kept: bool,
    pub model: LiveModel<'a>,
    pub address: Option<std::net::SocketAddr>,
    pub tasks: Option<LiveTasks>,
    pub baseline: Option<&'a std::collections::BTreeSet<u64>>,
    pub roots: Option<&'a std::collections::BTreeSet<u64>>,
    pub words: &'a str,
    pub line: &'a str,
}

impl LivePlan<'_> {
    /// The sitting's client directory, `client` beneath the run's root: every ctl call's
    /// DIRECTORY, the held record's import and give-back included (testing/live-tier.md).
    #[must_use]
    pub fn client_directory(&self) -> PathBuf {
        self.client.join("client")
    }

    /// Install the held record's give-back in the story's store; `binary` is the temper
    /// the hook runs.
    pub fn install_return_hook(&self, binary: &std::path::Path) -> Result<(), String> {
        if let LiveModel::ChatGpt { account: Some(account) } = self.model {
            super::held_login::return_hook(
                binary,
                self.paths[0].parent().expect("store"),
                &self.client_directory(),
                account,
            )?;
        }
        Ok(())
    }

    /// The held record's import as the sitting runs it, behind the give-back's lock.
    #[must_use]
    pub fn held_import(&self, binary: &std::path::Path, words: &[&str]) -> super::ctl::Call {
        super::held_login::import_guard(binary, self.paths[0].parent().expect("store"), &self.client_directory(), words)
    }

    /// Return before this box stops; only the first box permits a later restart import.
    #[must_use]
    pub fn return_call(&self) -> super::ctl::Call {
        let hook = self.paths[0].parent().expect("store").join("give-back");
        super::ctl::Call::command(&hook, if self.first { &["--box-ended"] } else { &[] })
    }

    fn starting_phase(&self) -> LivePhase {
        if self.first && !self.kept && matches!(self.model, LiveModel::ClaudeCode { .. }) {
            LivePhase::ProgramDeclared
        } else {
            LivePhase::Setup
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LiveTasks {
    pub project: u64,
    pub chat: u64,
    pub change: u64,
}

/// Only committed assistant text at or after the child's end counts as its reply.
pub fn replies_after(
    activity: &serde_json::Value,
    ended_at: u64,
) -> Result<std::collections::BTreeMap<u64, serde_json::Value>, String> {
    let mut replies = std::collections::BTreeMap::new();
    for row in activity["rows"].as_array().ok_or("chat activity missing")? {
        if row["at"].as_u64().ok_or("activity timestamp missing")? < ended_at {
            continue;
        }
        let Some(turn) = row.pointer("/event/owner/decoded/turn/decoded") else { continue };
        require(turn["cut"] == false, "whole reply turn")?;
        for message in turn["messages"].as_array().ok_or("turn messages missing")? {
            if message["speaker"] == "assistant"
                && message["blocks"].as_array().ok_or("message blocks missing")?.iter().any(|block| {
                    block
                        .pointer("/text/text")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|text| !text.trim().is_empty())
                })
            {
                replies.insert(row["position"].as_u64().ok_or("reply position missing")?, turn.clone());
            }
        }
    }
    Ok(replies)
}

/// testing/live-tier.md: the chat's one delegate is its change; an agent's
/// task, as a goal is, fails the story at once, naming it.
pub fn change_delegate(row: &serde_json::Value) -> Result<u64, String> {
    let task = row["chip"]["task"].as_u64().ok_or("delegate task missing (chip.task)")?;
    let (kind, executor) = row["chip"]["executor"]
        .as_object()
        .and_then(|executor| executor.iter().next())
        .ok_or("delegate executor missing (chip.executor)")?;
    if kind == "procedure" {
        return Ok(task);
    }
    let description = if kind == "agent" {
        let charter = executor["charter"].as_u64().ok_or("delegate charter missing (chip.executor.agent.charter)")?;
        format!("an agent's on charter {charter}")
    } else {
        format!("a {kind}'s")
    };
    Err(format!(
        "the chat's delegate task {task} is {description}, not a change: the shipped chat chooses change (testing/live-tier.md)"
    ))
}

/// testing/live-tier.md: the landing check's words name the chat's
/// committed calls, those that succeeded and those that failed.
pub fn require_chat_change(evidence: &conversation::Evidence) -> Result<(), String> {
    let succeeded = evidence.tools.iter().cloned().collect::<Vec<_>>().join(", ");
    let failed = evidence.failed.iter().cloned().collect::<Vec<_>>().join(", ");
    require(
        evidence.tools.contains("change"),
        &format!(
            "the chat's committed change call succeeded: its committed calls succeeded [{succeeded}] and failed [{failed}]"
        ),
    )
}

/// Names from actual committed call blocks, never from the model's prose.
#[must_use]
pub fn successful_tools(activity: &serde_json::Value) -> std::collections::BTreeSet<String> {
    let Some(rows) = activity["rows"].as_array() else { return std::collections::BTreeSet::new() };
    committed_tools(rows)
        .filter(|call| call["failed"] == false && !call["outcome"].is_null())
        .filter_map(|call| call["tool"].as_str().map(str::to_owned))
        .collect()
}

/// Names of the chat's failed committed calls from detailed activity.
#[must_use]
pub fn failed_tools(activity: &serde_json::Value) -> std::collections::BTreeSet<String> {
    let Some(rows) = activity["rows"].as_array() else { return std::collections::BTreeSet::new() };
    committed_tools(rows)
        .filter(|call| call["failed"] == true)
        .filter_map(|call| call["tool"].as_str().map(str::to_owned))
        .collect()
}

fn committed_tools(rows: &[serde_json::Value]) -> impl Iterator<Item = &serde_json::Value> {
    rows.iter()
        .filter_map(|row| {
            row.pointer("/event/owner/decoded/turn/decoded/messages").and_then(serde_json::Value::as_array)
        })
        .flatten()
        .filter(|message| message["speaker"] == "assistant")
        .flat_map(|message| message["blocks"].as_array().into_iter().flatten())
        .filter_map(|block| block.get("call"))
}

pub fn require_chat_progress(task: &serde_json::Value) -> Result<(), String> {
    let bad = "chat progress facts missing";
    let owner = task.pointer("/chip/details/owner").ok_or(bad)?;
    let phase = owner.get("phase").ok_or(bad)?;
    if phase.get("active").and_then(serde_json::Value::as_str) != Some("idle") {
        return Ok(());
    }
    let details = &owner["details"];
    let attempt = details.get("attempt").ok_or(bad)?;
    require(attempt.is_null() || attempt.is_u64(), bad)?;
    let children = details["live_delegates"].as_u64().ok_or(bad)?;
    let waits = task.get("waits_on").ok_or(bad)?;
    let run_wait = task.get("run_wait").ok_or(bad)?;
    let entries = task["subscriptions"]["entries"].as_array().ok_or(bad)?;
    let cut = task["cut"].as_bool().ok_or(bad)?;
    let wake = task.get("wake").ok_or(bad)?;
    let shipped = serde_json::json!({
        "words":"immediate", "notices":"immediate", "news":"immediate",
        "results":"last_or_failure", "questions":true, "answers":true, "timers":true
    });
    let empty_park = !cut
        && !attempt.is_null()
        && children == 0
        && waits == "news"
        && run_wait.is_null()
        && entries.is_empty()
        && wake == &shipped;
    require(!empty_park, "chat parked without delegating a change")
}

fn require(condition: bool, message: &str) -> Result<(), String> {
    if condition { Ok(()) } else { Err(message.to_owned()) }
}
fn live_json(call: &super::Ended) -> Result<serde_json::Value, String> {
    serde_json::from_slice(&call.stdout).map_err(|error| format!("live ctl JSON: {error}"))
}

enum LivePhase {
    ProgramDeclared,
    Setup,
    Settings,
    SettingsChanged,
    Roots,
    Created,
    Delegates,
    Waiting,
    Progress,
    Snapshot,
    Pages,
    Ended,
    Closing,
    Done,
    Returning,
    Stopping,
    Settled,
}
#[expect(clippy::struct_excessive_bools, reason = "the judge tracks independent observations beside its story phase")]
struct LiveJudge<'a> {
    plan: LivePlan<'a>,
    visitor: Option<usize>,
    account_at: usize,
    tasks: LiveTasks,
    baseline: std::collections::BTreeSet<u64>,
    roots: std::collections::BTreeSet<u64>,
    address: Option<std::net::SocketAddr>,
    token: Option<String>,
    found_at: usize,
    checked: usize,
    pending: std::collections::VecDeque<Vec<String>>,
    arguments: Vec<Vec<String>>,
    active_until: Option<super::patience::Patience>,
    wait_started: Option<skein_lib::Time>,
    wait_until: Option<super::patience::Patience>,
    wait_call: Option<usize>,
    failure: &'a std::cell::Cell<Option<String>>,
    signing_until: Option<super::patience::Patience>,
    account_checked: bool,
    linked: bool,
    cleanup: bool,
    at_stop: Vec<survivors::Row>,
    survivors_noted: bool,
    ending: Option<super::held_login::Ending>,
    return_at: Option<usize>,
    phase: LivePhase,
    snapshot: Option<String>,
    conversation: conversation::Evidence,
    ended: Option<EndedDelegates>,
    evidence: &'a std::cell::Cell<Option<LiveEvidence>>,
    completions: &'a std::cell::Cell<Option<Result<u32, String>>>,
    deadline: super::patience::Patience,
    poll: skein_lib::Time,
}
impl LiveJudge<'_> {
    fn advance(&mut self, now: skein_lib::Time, calls: &mut super::ctl::Calls) -> Result<(), String> {
        if !calls.foreground_idle() || !self.pending.is_empty() {
            return Ok(());
        }
        if matches!(self.phase, LivePhase::SettingsChanged) {
            self.queue(&["task", "list", "--project", &self.tasks.project.to_string(), "--json"]);
            self.phase = LivePhase::Roots;
            return Ok(());
        }
        let last = live_json(calls.ends().last().ok_or("live read missing")?)?;
        let chat = self.tasks.chat.to_string();
        let change = self.tasks.change.to_string();
        let project = self.tasks.project.to_string();
        match self.phase {
            LivePhase::ProgramDeclared => {
                let LiveModel::ClaudeCode { login } = self.plan.model else { return Err("program model".into()) };
                let commands = program_verification(login, &last)?;
                self.account_at = self.arguments.len() + commands.len() - 1;
                self.pending.extend(commands);
                self.phase = LivePhase::Setup;
            }
            LivePhase::Setup => self.setup(now, calls)?,
            LivePhase::Settings => self.settings(calls)?,
            LivePhase::SettingsChanged => return Err("settings change handled before the JSON reads".into()),
            LivePhase::Roots => {
                self.roots = live_roots(&last)?;
                let key = format!("live-chat-{}", now.as_nanos());
                self.queue(&[
                    "chat",
                    "start",
                    "--project",
                    &project,
                    "--model",
                    self.plan.model.name(),
                    "--key",
                    &key,
                    self.plan.words,
                    "--json",
                ]);
                self.phase = LivePhase::Created;
            }
            LivePhase::Created => {
                self.tasks.chat = last["outcome"]["done"]["document"]["created"]["first"]
                    .as_u64()
                    .ok_or("created chat number missing")?;
                self.wait_started = Some(now);
                self.wait_until =
                    Some(super::patience::Patience::arm("the live change's watch", Duration::from_secs(1800)));
                self.queue_delegates();
            }
            LivePhase::Delegates => self.find_delegates(now, calls, &last)?,
            LivePhase::Waiting => {
                self.wait_call = Some(calls.ends().len() - 1);
                if last["met"] == false {
                    self.queue(&["task", "show", &change, "--json"]);
                    self.queue(&["task", "list", "--project", &project, "--parent", &change, "--json"]);
                    self.queue(&["account", "list", "--json"]);
                    self.phase = LivePhase::Progress;
                } else {
                    // The person's signal that the change ended; the landing's checks follow.
                    super::live::notice(&format!(
                        "CHANGE ENDED {} after {}s: task{}={}",
                        if self.plan.first { "first box" } else { "after restart" },
                        self.wait_started.map_or(0, |started| now.saturating_since(started).as_nanos() / 1_000_000_000),
                        self.tasks.change,
                        last["task"]["chip"]["phase"].as_str().unwrap_or("?")
                    ));
                    self.found_at = calls.ends().len() - 1;
                    self.queue(&["task", "show", &change]);
                    self.queue(&["task", "show", &chat, "--json"]);
                    self.phase = LivePhase::Snapshot;
                }
            }
            LivePhase::Progress => self.progress(now, calls, &last)?,
            LivePhase::Snapshot => {
                let ended = live_json(&calls.ends()[self.found_at])?;
                require(ended["met"] == true, "change ended within its watch")?;
                let ended_at = ended["task"]["chip"]["details"]["owner"]["details"]["ended_at"]
                    .as_u64()
                    .ok_or("change ended timestamp missing")?;
                self.snapshot = Some(last["digest"].as_str().ok_or("chat digest missing")?.to_owned());

                self.conversation = conversation::Evidence::new(self.tasks.chat, ended_at);
                self.queue(&["chat", "show", &chat, "--json"]);
                self.phase = LivePhase::Pages;
            }
            LivePhase::Pages => self.read_page(calls, &last)?,
            LivePhase::Ended => {
                if let Some(words) = self.ended.as_mut().ok_or("ended delegate reader missing")?.read(&last)? {
                    self.pending.push_back(words);
                } else {
                    self.queue_closing();
                }
            }
            LivePhase::Closing => {
                self.check_landing(calls)?;
                self.phase = LivePhase::Done;
            }
            LivePhase::Done | LivePhase::Returning | LivePhase::Stopping | LivePhase::Settled => {}
        }
        Ok(())
    }
    fn setup(&mut self, now: skein_lib::Time, calls: &super::ctl::Calls) -> Result<(), String> {
        let address = if let Some(address) = self.address {
            address
        } else {
            // first_repository::live's configuration socket read, without
            // its started-record or transcript reads on a kept deployment.
            let document: serde_json::Value = serde_json::from_slice(
                &fs::read(self.plan.paths[0].join("temper.json")).map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
            document["application"]["machine"]["owned"]["listen"]["socket"]
                .as_str()
                .ok_or("forge address missing")?
                .parse()
                .map_err(|_| "forge socket invalid")?
        };
        self.address = Some(address);
        let owned = super::first_repository::check_owned_forge(self.plan.client)?;
        let number = owned["number"].as_u64().ok_or("forge number missing")?;
        let token = oracle_token(self.plan.paths, self.plan.first, number)?;
        self.setup_project(calls, address, &token, u16::try_from(number).map_err(|_| "forge number invalid")?)?;
        self.token = Some(token);
        if self.plan.first {
            // All fixed setup indices were consumed before these extra reads.
            if self.plan.kept {
                self.queue(&["task", "list", "--project", &self.tasks.project.to_string(), "--json"]);
                self.phase = LivePhase::Roots;
            } else {
                self.queue(&["settings", "show", "--json"]);
                self.queue(&["settings", "show", "charter", "1", "--json"]);
                self.queue(&["settings", "show", "charter", "2", "--json"]);
                self.queue(&["settings", "show", "model", self.plan.model.number(), "--json"]);
                self.phase = LivePhase::Settings;
            }
        } else {
            self.wait_started = Some(now);
            self.wait_until =
                Some(super::patience::Patience::arm("the live change's watch", Duration::from_secs(1800)));
            self.wait_round(now)?;
        }

        Ok(())
    }
    fn setup_project(
        &mut self,
        calls: &super::ctl::Calls,
        address: std::net::SocketAddr,
        token: &str,
        forge: u16,
    ) -> Result<(), String> {
        if self.plan.first {
            let pulls = merged_pulls(address, token)?;
            self.baseline = pulls.iter().map(pull_number).collect::<Result<_, _>>()?;
            if self.plan.kept {
                let me = live_json(&calls.ends()[4])?;
                self.tasks.project = read_kept_project(self.plan.client, &me, address, token, forge)?;
            } else {
                let forges = live_json(&calls.ends()[5])?;
                let forge = super::first_repository::check_owned_row(&forges)?;
                require(
                    forge["first"]["org"] == "work" && forge["first"]["repository"] == "hello",
                    "first repository is work/hello",
                )?;
                let repository = forge["first"]["repository"].as_str().ok_or("first repository missing")?;
                self.tasks.project =
                    super::first_repository::project_number(&live_json(&calls.ends()[7])?, repository)?;
            }
        }
        Ok(())
    }
    fn settings(&mut self, calls: &super::ctl::Calls) -> Result<(), String> {
        let reads = &calls.ends()[calls.ends().len() - 4..];
        let inventory = live_json(&reads[0])?;
        let revision = inventory["newest"].as_u64().ok_or("settings revision missing")?;
        let catalogue = live_json(&reads[3])?;
        let charters = [live_json(&reads[1])?, live_json(&reads[2])?];
        let changes = live_settings(self.plan.model, &inventory, [&charters[0], &charters[1]], &catalogue)?;
        for (at, change) in changes.iter().enumerate() {
            let file = self.plan.client.join(format!("{}.json", change.key));
            fs::write(&file, serde_json::to_vec(&change.file).map_err(|error| error.to_string())?)
                .map_err(|error| error.to_string())?;
            self.pending.push_back(live_apply(
                change,
                at,
                revision,
                file.to_str().ok_or("settings FILE path is not UTF-8")?,
            ));
        }
        self.phase = LivePhase::SettingsChanged;
        Ok(())
    }
    fn find_delegates(
        &mut self,
        now: skein_lib::Time,
        calls: &super::ctl::Calls,
        last: &serde_json::Value,
    ) -> Result<(), String> {
        let mut delegates = Vec::new();
        for row in last["live"].as_array().ok_or("live delegates missing")? {
            delegates.push(row);
        }
        let ended = &last["ended"]["rows"];
        require(
            last["live_after"].is_null() && ended["after"].is_null() && ended["cut"] == false,
            "whole chat delegation set",
        )?;
        for row in ended["rows"].as_array().ok_or("ended delegates missing")? {
            delegates.push(row);
        }
        if delegates.is_empty() {
            let chat = live_json(calls.ends().iter().rev().nth(1).ok_or("chat snapshot missing")?)?;
            require_chat_progress(&chat)?;
            if let Some(figures) = self.wait_until.as_mut().ok_or("delegate deadline missing")?.failed() {
                return Err(format!("chat did not delegate within 1800s: {figures}"));
            }
            self.queue(&[
                "task",
                "show",
                &self.tasks.chat.to_string(),
                "--until",
                "replied",
                "--timeout",
                "5s",
                "--json",
            ]);
            self.queue(&["account", "list", "--json"]);
            self.queue_delegates();
        } else {
            require(delegates.len() == 1, "one change delegated by the new chat")?;
            self.tasks.change = change_delegate(delegates[0])?;
            super::live::notice(&format!(
                "CHANGE task={} chat={} project={}",
                self.tasks.change, self.tasks.chat, self.tasks.project
            ));
            self.wait_round(now)?;
        }

        Ok(())
    }
    fn read_page(&mut self, _calls: &super::ctl::Calls, last: &serde_json::Value) -> Result<(), String> {
        let chat = self.tasks.chat.to_string();
        if let Some(words) = self.conversation.read(last)? {
            self.pending.push_back(words);
        } else if self.conversation.replies.is_empty() {
            self.queue(&[
                "task",
                "show",
                &chat,
                "--until",
                "replied",
                "--since",
                &self.snapshot.clone().ok_or("snapshot before conversation missing")?,
                "--timeout",
                "5s",
                "--json",
            ]);
            self.queue(&["account", "list", "--json"]);
            self.queue(&["task", "show", &chat, "--json"]);
            self.phase = LivePhase::Snapshot;
        } else {
            let ended = EndedDelegates::new(self.tasks.project, [self.tasks.chat, self.tasks.change]);
            self.pending.push_back(ended.first());
            self.ended = Some(ended);
            self.phase = LivePhase::Ended;
        }

        Ok(())
    }
    fn progress(
        &mut self,
        now: skein_lib::Time,
        calls: &super::ctl::Calls,
        _last: &serde_json::Value,
    ) -> Result<(), String> {
        let task = live_json(&calls.ends()[calls.ends().len() - 3])?;
        let last = live_json(&calls.ends()[calls.ends().len() - 2])?;
        let children = last["tasks"]["rows"]
            .as_array()
            .ok_or("children rows missing")?
            .iter()
            .map(|row| format!("{}:{}", row["chip"]["task"], row["chip"]["phase"].as_str().unwrap_or("?")))
            .collect::<Vec<_>>()
            .join(",");
        super::live::notice(&format!(
            "PROGRESS {} task{}={} children={} waits_on={} details={}",
            now.saturating_since(self.wait_started.ok_or("watch start missing")?).as_nanos() / 1_000_000_000,
            self.tasks.change,
            task["chip"]["phase"].as_str().unwrap_or("?"),
            children,
            task["waits_on"],
            task["chip"]["details"]
        ));
        if let Some(figures) = self.wait_until.as_mut().ok_or("watch deadline missing")?.failed() {
            self.fail(&format!("change watch exceeded 1800s: {figures}"), calls, self.wait_call);
        } else {
            self.wait_round(now)?;
        }
        Ok(())
    }
    fn stopping(&self) -> bool {
        matches!(self.phase, LivePhase::Stopping | LivePhase::Settled)
    }

    fn check_landing(&mut self, calls: &super::ctl::Calls) -> Result<(), String> {
        let last = live_json(calls.ends().last().ok_or("closing reads missing")?)?;
        if !self.plan.first {
            require(live_json(&calls.ends()[calls.ends().len() - 2])?["met"] == true, "chat tree closed")?;
            require_closed_roots(&last, &self.roots)?;
        }
        let page = live_change_page(client_address(calls)?, self.plan.client, self.tasks.change)?;
        require_chat_change(&self.conversation)?;
        let delegates = self.ended.take().ok_or("ended delegate reader missing")?.check_whole()?;
        require(delegates[0].contains(&self.tasks.change), "change task belongs to the chat that called change")?;
        require(!self.conversation.tools.contains("delegate"), "no raw delegate started the task")?;
        let mut tasks = vec![self.tasks.chat, self.tasks.change];
        tasks.extend(&delegates[1]);
        tasks.sort_unstable();
        tasks.dedup();
        let completions = super::live::ctl_completions(self.plan.client, &tasks)?;
        self.completions.set(Some(Ok(completions)));
        require((1..=25).contains(&completions), "live completion budget: 1–25")?;
        super::live::notice(&format!(
            "COMPLETIONS chat={} change={} count={completions}",
            self.tasks.chat, self.tasks.change
        ));
        let forge = forge_identity(
            self.address.ok_or("forge address missing")?,
            self.plan.line,
            self.token.as_deref().ok_or("oracle credential missing")?,
            &self.baseline,
        )?;
        self.evidence.set(Some(LiveEvidence {
            task: String::from_utf8_lossy(&calls.ends()[self.found_at + 1].stdout).into_owned(),
            page,
            completions,
            address: self.address.ok_or("forge address missing")?,
            tasks: self.tasks.clone(),
            baseline: self.baseline.clone(),
            roots: self.roots.clone(),
            forge,
            replies: std::mem::take(&mut self.conversation.replies),
            delegates,
        }));
        Ok(())
    }
}
impl LiveJudge<'_> {
    fn queue_closing(&mut self) {
        if !self.plan.first {
            let chat = self.tasks.chat.to_string();
            let project = self.tasks.project.to_string();
            let key = format!("close-live-chat-{chat}");
            for args in [
                &["chat", "close", &chat, "--key", &key, "--json"][..],
                &["task", "tree", &chat, "--until", "ended", "--timeout", "300s", "--json"],
                &["task", "list", "--project", &project, "--json"],
            ] {
                self.queue(args);
            }
        }
        self.phase = LivePhase::Closing;
    }
    fn queue_delegates(&mut self) {
        self.queue(&["task", "show", &self.tasks.chat.to_string(), "--json"]);
        self.queue(&[
            "task",
            "list",
            "--project",
            &self.tasks.project.to_string(),
            "--parent",
            &self.tasks.chat.to_string(),
            "--ended",
            "--json",
        ]);
        self.phase = LivePhase::Delegates;
    }
    fn queue(&mut self, args: &[&str]) {
        self.pending.push_back(args.iter().map(|arg| (*arg).to_owned()).collect());
    }
    fn wait_round(&mut self, now: skein_lib::Time) -> Result<(), String> {
        let change = self.tasks.change.to_string();
        let left = self
            .wait_until
            .as_ref()
            .ok_or("watch deadline missing")?
            .until()
            .map_or(60, |until| until.saturating_since(now).as_nanos() / 1_000_000_000);
        self.queue(&[
            "task",
            "show",
            &change,
            "--until",
            "ended",
            "--timeout",
            &format!("{}s", left.clamp(1, 60)),
            "--json",
        ]);
        self.phase = LivePhase::Waiting;
        Ok(())
    }
    fn dump(&self, index: usize, call: &super::Ended) {
        super::live::notice(&format!(
            "EVIDENCE cwd={} argv=temper ctl client {:?} exit={:?}\nstdout: {}\nstderr: {}",
            self.plan.client.display(),
            self.arguments.get(index),
            call.exit,
            String::from_utf8_lossy(&call.stdout),
            String::from_utf8_lossy(&call.stderr)
        ));
    }
    fn fail(&mut self, message: &str, calls: &super::ctl::Calls, failing: Option<usize>) {
        let chat = self.tasks.chat.to_string();
        let change = self.tasks.change.to_string();
        let project = self.tasks.project.to_string();
        let previous = self.failure.take();
        if previous.is_some() {
            self.failure.set(previous);
            return;
        }
        let account = if self.plan.kept || self.account_checked {
            self.diagnostic(&["account", "list", "--json"]).and_then(|read| live_json(&read).ok())
        } else {
            None
        };
        let message = if account.as_ref().is_some_and(|value| self.plan.model.ready(value).is_err()) {
            self.plan.model.unavailable()
        } else {
            message
        };
        let message = if let Some(index) = failing {
            format!("{message}: argv={:?} exit={:?}", self.arguments.get(index), calls.ends()[index].exit)
        } else {
            message.to_owned()
        };
        self.failure.set(Some(super::live::redact(&message)));
        self.completions.set(Some(self.count_completions()));
        super::live::notice(&format!("FAILURE {message}"));
        if let Some(index) = failing {
            self.dump(index, &calls.ends()[index]);
        } else if let Some(call) = calls.pending() {
            self.dump(calls.ends().len(), call);
        }
        // Each independent read uses ctl's existing 60s bound. Even an unknown
        // task or a broken read is evidence, never a reason to omit later reads.
        for args in [
            &["task", "show", &chat, "--json"][..],
            &["task", "show", &change, "--json"],
            &["task", "list", "--project", &project, "--json"],
        ] {
            self.diagnostic(args);
        }
        let mut before = None::<String>;
        for _ in 0..32 {
            let mut args = vec!["chat", "show", &chat, "--json"];
            if let Some(cursor) = &before {
                args.extend(["--before", cursor]);
            }
            let Some(read) = self.diagnostic(&args) else { break };
            let Ok(page) = serde_json::from_slice::<serde_json::Value>(&read.stdout) else { break };
            if let Some(rows) = page["activity"]["rows"].as_array() {
                for call in committed_tools(rows) {
                    super::live::notice(&format!(
                        "TOOL name={} failed={} outcome={}",
                        call["tool"], call["failed"], call["outcome"]
                    ));
                }
            }
            before = page["activity"]["before"]["value"].as_str().map(str::to_owned);
            if before.is_none() {
                break;
            }
        }
        self.pending.clear();
        self.phase = LivePhase::Done;
    }
    fn count_completions(&self) -> Result<u32, String> {
        let mut tasks = vec![];
        for parent in [self.tasks.chat, self.tasks.change].into_iter().filter(|task| *task != 0) {
            tasks.push(parent);
            let read = super::ctl::try_alone(
                self.plan.client,
                super::ctl::Call::new(&[
                    "task",
                    "list",
                    "--project",
                    &self.tasks.project.to_string(),
                    "--parent",
                    &parent.to_string(),
                    "--ended",
                    "--json",
                ]),
            )?;
            let rows = live_json(&read)?;
            let ended = &rows["ended"]["rows"];
            require(
                rows["live_after"].is_null() && ended["after"].is_null() && ended["cut"] == false,
                "completion task list incomplete",
            )?;
            for row in rows["live"]
                .as_array()
                .ok_or("live delegates missing")?
                .iter()
                .chain(ended["rows"].as_array().ok_or("ended delegates missing")?)
            {
                tasks.push(row["chip"]["task"].as_u64().ok_or("delegate task missing (chip.task)")?);
            }
        }
        tasks.sort_unstable();
        tasks.dedup();
        super::live::ctl_completions(self.plan.client, &tasks)
    }
    fn diagnostic(&self, args: &[&str]) -> Option<super::Ended> {
        if let Ok(read) = super::ctl::try_alone(self.plan.client, super::ctl::Call::new(args)) {
            // Do not copy conversation prose or tool arguments into notices.
            let stdout = if args.first() == Some(&"chat") {
                "(committed tools below)".into()
            } else {
                String::from_utf8_lossy(&read.stdout)
            };
            super::live::notice(&format!(
                "EVIDENCE cwd={} argv=temper ctl client {args:?} exit={:?}\nstdout: {stdout}\nstderr: {}",
                self.plan.client.display(),
                read.exit,
                String::from_utf8_lossy(&read.stderr)
            ));
            Some(read)
        } else {
            super::live::notice(&format!("EVIDENCE {args:?}: bounded read unavailable"));
            None
        }
    }
    fn check_calls(&mut self, _now: skein_lib::Time, calls: &super::ctl::Calls) {
        for index in self.checked..calls.ends().len() {
            let call = &calls.ends()[index];
            if (self.plan.kept || self.account_checked)
                && self.arguments.get(index).is_some_and(|args| args == &["account", "list", "--json"])
                && live_json(call).and_then(|value| self.plan.model.ready(&value)).is_err()
            {
                self.fail(self.plan.model.unavailable(), calls, Some(index));
                break;
            }
            if (self.plan.kept || self.account_checked)
                && let Ok(value) = live_json(call)
            {
                let task = value.get("task").unwrap_or(&value);
                if matches!(
                    task.pointer("/run_wait/account/why").and_then(serde_json::Value::as_str),
                    Some("needs_sign_in" | "refused")
                ) {
                    self.fail(self.plan.model.unavailable(), calls, Some(index));
                    break;
                }
            }
            let not_yet = self
                .arguments
                .get(index)
                .is_some_and(|args| args.iter().any(|arg| matches!(arg.as_str(), "--until" | "--since")))
                && call.exit == Some(Exit::Code(3));
            if call.exit != Some(Exit::Code(0)) && !not_yet {
                self.fail("live ctl failed", calls, Some(index));
                break;
            }
            if self.plan.first
                && !self.plan.kept
                && matches!(self.plan.model, LiveModel::ChatGpt { .. })
                && index == self.account_at - 2
            {
                super::live::notice(&String::from_utf8_lossy(&call.stdout));
                if matches!(self.plan.model, LiveModel::ChatGpt { account: None }) {
                    self.signing_until = Some(super::patience::Patience::arm(
                        "the live forge's ChatGPT sign-in",
                        Duration::from_secs(600),
                    ));
                }
            }
        }
        if calls.ends().len() > self.checked {
            self.active_until = None;
        }
        self.checked = calls.ends().len();
        if !matches!(self.phase, LivePhase::Done | LivePhase::Returning | LivePhase::Stopping | LivePhase::Settled)
            && !matches!(self.phase, LivePhase::ProgramDeclared)
            && !self.account_checked
            && calls.ends().len() > self.account_at
        {
            match live_json(&calls.ends()[self.account_at]).and_then(|value| self.plan.model.ready(&value)) {
                Ok(()) => self.account_checked = true,
                Err(message) => self.fail(
                    if self.plan.kept { self.plan.model.unavailable() } else { &message },
                    calls,
                    Some(self.account_at),
                ),
            }
        }
    }
    fn watch_waits(&mut self, binary: &skein_world::end_to_end::Binary) {
        self.deadline.watch(binary);
        for patience in [&mut self.active_until, &mut self.wait_until, &mut self.signing_until].into_iter().flatten() {
            patience.watch(binary);
        }
    }
    fn return_failed(&mut self, message: &str) {
        let message = format!("LOGIN NOT BACK: {message}; store retained for recovery");
        super::live::notice(&message);
        self.failure.set(Some(message));
    }
    fn finish_story(
        &mut self,
        now: skein_lib::Time,
        binary: &mut skein_world::end_to_end::Binary,
        calls: &mut super::ctl::Calls,
    ) -> Result<(), String> {
        if matches!(self.phase, LivePhase::Done) {
            let failed = self.failure.take();
            self.ending = Some(super::held_login::Ending::new(
                matches!(self.plan.model, LiveModel::ChatGpt { account: Some(_) }),
                self.plan.first,
                failed.is_some(),
                now,
            ));
            self.failure.set(failed);
            self.phase = LivePhase::Returning;
        }
        let ended = self.return_at.and_then(|index| calls.ends().get(index));
        match self.ending.as_mut().ok_or("return stage")?.next(now, calls.foreground_idle(), ended.map(|end| end.exit))
        {
            super::held_login::End::Return => {
                self.return_at = Some(calls.ends().len());
                calls.push(super::ctl::Step::Run(self.plan.return_call()));
            }
            super::held_login::End::Wait => {}
            super::held_login::End::Stop(failure) => {
                if let Some(ended) = ended {
                    super::live::notice(&String::from_utf8_lossy(&ended.stdout));
                    super::live::notice(&String::from_utf8_lossy(&ended.stderr));
                }
                if let Some(message) = failure {
                    self.return_failed(message);
                }
                self.stop(binary, calls);
            }
        }
        Ok(())
    }
    fn stop(&mut self, binary: &mut skein_world::end_to_end::Binary, calls: &mut super::ctl::Calls) {
        calls.release();
        calls.cancel();
        self.at_stop = survivors::rows(&binary.tree_status().live_pids);
        binary.signal(skein_io::kernel::Signal::Terminate);
        self.phase = LivePhase::Stopping;
    }
    fn drive(
        &mut self,
        now: skein_lib::Time,
        binary: &mut skein_world::end_to_end::Binary,
        calls: &mut super::ctl::Calls,
    ) -> Result<(), String> {
        use skein_shell::Host;
        if self.stopping()
            && !self.survivors_noted
            && let Some(left) = &binary.tree_status().live_at_leader_exit
        {
            if !left.is_empty() {
                super::live::notice(&survivors::lines(left, &survivors::rows(left), &self.at_stop));
            }
            self.survivors_noted = true;
        }
        calls.observe_box_stderr(binary.stderr(), now);
        if !self.linked && super::printed_link(calls.stdout()).is_some() {
            super::live::client_notice(&String::from_utf8_lossy(calls.link()));
            self.linked = true;
        }
        if matches!(self.phase, LivePhase::Done | LivePhase::Returning) {
            self.finish_story(now, binary, calls)?;
        }
        if !self.stopping() && !matches!(self.phase, LivePhase::Returning) {
            self.check_calls(now, calls);
        }
        if !self.stopping()
            && !matches!(self.phase, LivePhase::Done | LivePhase::Returning)
            && binary.exit_status().is_none()
        {
            self.advance(now, calls)?;
        }
        self.watch_waits(binary);
        if !self.stopping() && !matches!(self.phase, LivePhase::Done | LivePhase::Returning) {
            if binary.exit_status().is_some() {
                super::live::notice(&format!(
                    "BOX exit={:?} stderr: {}",
                    binary.exit_status(),
                    String::from_utf8_lossy(binary.stderr())
                ));
                self.fail("live box stopped before its reads", calls, None);
            } else if let Some(figures) = self
                .signing_until
                .as_mut()
                .filter(|_| !self.account_checked)
                .and_then(super::patience::Patience::failed)
            {
                self.fail(&format!("ChatGPT sign-in exceeded 10 minutes: {figures}"), calls, None);
            } else if let Some(figures) = self.deadline.failed() {
                self.fail(&format!("live story exceeded its budget: {figures}"), calls, None);
            } else if let Some(figures) = self.active_until.as_mut().and_then(super::patience::Patience::failed) {
                self.fail(&format!("live ctl exceeded its per-call bound: {figures}"), calls, None);
            } else if now >= self.poll {
                self.poll = now.saturating_add(Duration::from_secs(1));
            }
        }
        if !self.stopping()
            && !matches!(self.phase, LivePhase::Done | LivePhase::Returning)
            && calls.foreground_idle()
            && let Some(args) = self.pending.pop_front()
        {
            let seconds = args
                .windows(2)
                .find(|pair| pair[0] == "--timeout")
                .and_then(|pair| pair[1].strip_suffix('s'))
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            self.active_until = Some(super::patience::Patience::watching(
                "the live forge's ctl call",
                Duration::from_secs(seconds + 60),
                binary,
            ));
            let words = args.iter().map(String::as_str).collect::<Vec<_>>();
            calls.push(super::ctl::Step::Run(if words == ["sign-in", "-"] {
                super::ctl::Call::sign_in()
            } else if words.starts_with(&["account", "import"])
                && matches!(self.plan.model, LiveModel::ChatGpt { account: Some(_) })
            {
                self.plan.held_import(&skein_scratch::paths::binary("temper-box"), &words)
            } else {
                super::ctl::Call::new(&words)
            }));
            self.arguments.push(printed_command(&args));
        }
        if matches!(self.phase, LivePhase::Done | LivePhase::Returning) {
            self.finish_story(now, binary, calls)?;
        }
        self.watch_waits(binary);
        if self.stopping()
            && !self.cleanup
            && !binary.is_empty()
            && let Some(figures) = self.deadline.failed()
        {
            super::live::notice(&format!("live story cleanup: {figures}"));
            binary.signal_tree(skein_io::kernel::Signal::Kill);
            self.cleanup = true;
        }
        Ok(())
    }
}
impl Referee<Process> for LiveJudge<'_> {
    fn act(&mut self, now: skein_lib::Time, procs: &mut [Process]) {
        if let Some(index) = self.visitor {
            let Process::Calls(calls) = &procs[1] else { return };
            if let Some(address) = calls.ends().iter().find_map(sign_in::authorization_address) {
                let address = address.to_vec();
                let Process::Visitor(visitor) = &mut procs[index] else { return };
                visitor.hand(&address);
                self.visitor = None;
            }
        }
        let (drivers, peers) = procs.split_at_mut(2);
        let [Process::Binary(binary), Process::Calls(calls)] = drivers else { return };
        if let Err(message) = self.drive(now, binary, calls) {
            self.fail(&message, calls, calls.ends().len().checked_sub(1));
        }
        if self.stopping() {
            for peer in peers {
                match peer {
                    Process::Peer(peer) => peer.shutdown(),
                    Process::Issuer(issuer) => issuer.shutdown(),
                    Process::Calls(_)
                    | Process::Binary(_)
                    | Process::Stdout(_)
                    | Process::Person(_)
                    | Process::Visitor(_) => {}
                }
            }
        }
    }
    fn watch(&mut self, _now: skein_lib::Time, procs: &[Process]) -> Result<(), String> {
        use skein_shell::Host;
        if let Some(why) = self.failure.take() {
            self.failure.set(Some(why.clone()));
            return Err(why);
        }
        if self.stopping() && procs.iter().all(Host::is_empty) {
            self.phase = LivePhase::Settled;
        }
        Ok(())
    }
    fn next_deadline(&self) -> Option<skein_lib::Time> {
        if matches!(self.phase, LivePhase::Settled) || self.cleanup {
            None
        } else if matches!(self.phase, LivePhase::Returning) {
            self.ending.as_ref().map(super::held_login::Ending::deadline)
        } else if self.stopping() {
            self.deadline.until()
        } else {
            Some(self.deadline.until().map_or(self.poll, |until| self.poll.min(until)))
        }
    }
    fn passed(&self) -> bool {
        let finding = self.failure.take();
        let passed = finding.is_none() && matches!(self.phase, LivePhase::Settled);
        self.failure.set(finding);
        passed
    }
}

fn live_deadline(budget: Duration, binary: &skein_world::end_to_end::Binary) -> super::patience::Patience {
    let within = Duration::from_nanos(budget.as_nanos().saturating_sub(Duration::from_secs(10).as_nanos()));
    super::patience::Patience::watching("the live forge's story", within, binary)
}

/// The live face has only the supervised box and ctl, no issuer, visitor or peer.
pub fn run_live(
    plan: LivePlan<'_>,
    budget: Duration,
    face: Face,
) -> Result<super::boxed::CtlOutcome<LiveEvidence>, String> {
    run_live_counted(plan, budget, face, &mut Ok(0))
}

/// The sitting's completion count survives a returned finding, for the live summary.
#[expect(
    clippy::too_many_lines,
    reason = "The sitting admits its hosts and retains their findings and completion count together."
)]
pub fn run_live_counted(
    plan: LivePlan<'_>,
    budget: Duration,
    face: Face,
    completions: &mut Result<u32, String>,
) -> Result<super::boxed::CtlOutcome<LiveEvidence>, String> {
    use skein_shell::{Clock, Host};
    use skein_world::end_to_end::{Binary, Command, Expectation, Mode};
    if let Face::Fake { peer, .. } = &face {
        peer.configure(&plan.paths[0]);
    }
    plan.install_return_hook(&skein_scratch::paths::binary("temper-box"))?;
    let clock = Clock::new();
    let mut binary = Binary::start_with_tree(
        Command {
            program: skein_scratch::paths::binary("temper-box"),
            arguments: deployment_flags(plan.paths)
                .into_iter()
                .map(Into::into)
                .chain(["--echo".into(), "steps".into()])
                .collect(),
            directory: plan.paths[0].parent().ok_or("scratch root")?.into(),
            environment: vec![("PATH".into(), "/usr/bin:/bin".into())],
        },
        Mode::Pipes,
        262_144,
        Expectation::Supervise,
        false,
    )
    .map_err(|error| format!("live box starts: {error:?}"))?;
    let streams = binary.take_streams().ok_or("box pipes")?;
    let mut fds = streams.descriptors();
    let root = skein_shell::open_root(plan.client).map_err(|error| format!("fresh client root: {error}"))?;
    fds.push(root);
    let (commands, account_at) = live_commands(plan.first && !plan.kept, plan.model);
    let failure = std::cell::Cell::new(None);
    let evidence = std::cell::Cell::new(None);
    let count = std::cell::Cell::new(None);
    let mut world = real::World::new(LiveJudge {
        tasks: plan.tasks.clone().unwrap_or_default(),
        baseline: plan.baseline.cloned().unwrap_or_default(),
        roots: plan.roots.cloned().unwrap_or_default(),
        address: plan.address,
        token: None,
        visitor: matches!(face, Face::Fake { .. }).then_some(3),
        account_at,
        found_at: 0,
        checked: 0,
        pending: commands.into(),
        arguments: Vec::new(),
        active_until: None,
        wait_started: None,
        wait_until: None,
        wait_call: None,
        failure: &failure,
        signing_until: None,
        account_checked: false,
        linked: false,
        cleanup: false,
        at_stop: Vec::new(),
        survivors_noted: false,
        ending: None,
        return_at: None,
        phase: plan.starting_phase(),
        plan,
        snapshot: None,
        conversation: conversation::Evidence::new(0, 0),
        ended: None,
        evidence: &evidence,
        completions: &count,
        deadline: live_deadline(budget, &binary),
        poll: clock.now().now,
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds, || {
        let mut calls = super::ctl::Calls::new(root, vec![], Some(&streams), None).with_box_stderr();
        calls.hold();
        Process::Calls(Box::new(calls))
    });
    if matches!(face, Face::Fake { .. }) {
        face.spawn(&mut world);
    }
    // Failure diagnostics: at most 36 ctl reads (60s each), a 20s native
    // change-page read, a 60s return, and ten seconds to cancel, drain and reap afterwards.
    let drain_budget = Duration::from_secs(36 * 60 + 90);
    let ended = world.try_run(&clock, Duration::from_nanos(budget.as_nanos().saturating_add(drain_budget.as_nanos())));
    *completions = count.take().unwrap_or_else(|| Err("completion count unavailable".into()));
    let ended = ended?;
    if !ended.procs.iter().all(Host::is_empty) {
        return Err("live box, descendants and ctl did not drain".into());
    }
    let [Process::Binary(binary), Process::Calls(calls), ..] = ended.procs.as_slice() else {
        return Err("live box and ctl".into());
    };
    if let Some(message) = failure.take() {
        super::live::notice(&format!("BOX stderr: {}", String::from_utf8_lossy(binary.stderr())));
        return Err(message);
    }
    Ok(super::boxed::CtlOutcome {
        model: None,
        exit: binary.exit_status(),
        stdout: calls.stdout().to_vec(),
        stderr: binary.stderr().to_vec(),
        left: binary.tree_status().live_at_leader_exit.clone(),
        calls: calls
            .ends()
            .iter()
            .map(|call| super::Ended { exit: call.exit, stdout: call.stdout.clone(), stderr: call.stderr.clone() })
            .collect(),
        checked: evidence.take(),
        visits: Vec::new(),
    })
}

fn live_change_page(engine: std::net::SocketAddr, client: &std::path::Path, task: u64) -> Result<String, String> {
    use skein_shell::{Clock, Host};
    use skein_world::{Referee, real};
    use temper_client_shell_terminal::client::{NativeClient, Settings, Start};
    struct Judge<'a> {
        task: u64,
        opened: bool,
        closing: bool,
        page: &'a std::cell::Cell<String>,
        until: super::patience::Patience,
    }
    impl Referee<NativeClient> for Judge<'_> {
        fn act(&mut self, _now: skein_lib::Time, clients: &mut [NativeClient]) {
            let client = &mut clients[0];
            if client.started() && !self.opened {
                client.go(temper_client_domain::Address::Task { task: self.task });
                self.opened = true;
            }
            if self.closing {
                return;
            }
            let mut page = String::new();
            for node in client.tree().nodes().iter().filter(|node| !node.text.is_empty()) {
                for run in &node.text {
                    page.push_str(&String::from_utf8_lossy(&run.text));
                }
                page.push('\n');
            }
            if (page.contains("work/hello") && page.contains("CI at its head"))
                || self.until.failed().is_some_and(|figures| {
                    eprintln!("native change page incomplete: {figures}");
                    true
                })
            {
                self.page.set(page);
                client.close();
                self.closing = true;
            }
        }
        fn watch(&mut self, _now: skein_lib::Time, _clients: &[NativeClient]) -> Result<(), String> {
            Ok(())
        }
        fn next_deadline(&self) -> Option<skein_lib::Time> {
            if self.closing { None } else { self.until.until() }
        }
        fn passed(&self) -> bool {
            self.closing
        }
    }
    let root = skein_shell::open_root(client).map_err(|error| format!("client's parent root: {error}"))?;
    let clock = Clock::new();
    let page = std::cell::Cell::new(String::new());
    let mut world = real::World::new(Judge {
        task,
        opened: false,
        closing: false,
        page: &page,
        until: super::patience::Patience::arm(
            "the live forge's native change page",
            skein_lib::Duration::from_secs(10),
        ),
    });
    let reader = NativeClient::new(
        Settings {
            engine: skein_io::Addr::Ip(engine),
            directory: b"client".as_slice().into(),
            limits: temper_client_shell_terminal::limits::native(),
        },
        root,
        skein_shell::effective_user(),
        91,
        Start::Token,
    )
    .map_err(|error| format!("native reader: {error:?}"))?;
    world.spawn_with_fds(vec![root], || reader);
    let ended = world.try_run(&clock, skein_lib::Duration::from_secs(20))?;
    if !ended.procs.iter().all(Host::is_empty) {
        return Err("native reader drained".into());
    }
    Ok(page.take())
}

fn client_address(calls: &super::ctl::Calls) -> Result<std::net::SocketAddr, String> {
    let link = std::str::from_utf8(calls.link()).map_err(|_| "client link encoding")?;
    link.strip_prefix("http://")
        .ok_or("local client link")?
        .split('/')
        .next()
        .ok_or("client listener")?
        .parse()
        .map_err(|_| "client listener address".into())
}

/// Read durable adoptions even when the kept sitting has never landed a change.
fn read_kept_project(
    client: &std::path::Path,
    me: &serde_json::Value,
    address: std::net::SocketAddr,
    token: &str,
    forge: u16,
) -> Result<u64, String> {
    let repository = super::owned::answer(address.port(), "/api/v1/repos/work/hello", Some(token))
        .map_err(|error| format!("kept work/hello read failed: {error}"))?;
    let repository = repository["id"]
        .as_u64()
        .and_then(|number| u32::try_from(number).ok())
        .filter(|number| *number != 0)
        .ok_or("kept work/hello repository number invalid")?;
    let mut projects = Vec::new();
    for project in visible_projects(me)? {
        let read =
            super::ctl::try_alone(client, super::ctl::Call::new(&["project", "show", &project.to_string(), "--json"]))?;
        require(read.exit == Some(Exit::Code(0)), "kept project read failed")?;
        let document = live_json(&read)?;
        require(document["number"] == project, "kept project read names another project")?;
        projects.push(document);
    }
    kept_project(me, &projects, forge, repository)
}

fn visible_projects(me: &serde_json::Value) -> Result<Vec<u64>, String> {
    require(me["cut"] == false, "person's projects read is incomplete")?;
    let mut numbers = Vec::new();
    for row in me["projects"].as_array().ok_or("person's projects missing")? {
        let number = row["project"].as_u64().filter(|number| *number != 0).ok_or("person's project number invalid")?;
        require(!numbers.contains(&number), "person's project number repeated")?;
        numbers.push(number);
    }
    Ok(numbers)
}

/// Init's forge connector is 1; adoption paths encode forge and repository numbers
/// as big-endian bytes (`temper-engine-domain/src/route/forge_route.rs`, `report_adoption`).
fn kept_project(
    me: &serde_json::Value,
    projects: &[serde_json::Value],
    forge: u16,
    repository: u32,
) -> Result<u64, String> {
    let visible = visible_projects(me)?;
    let part = serde_json::json!({"connector":1,"kind":"resource",
        "key":{"path":[format!("{forge:04x}"),format!("{repository:08x}")]}});
    let mut found = None;
    for project in projects {
        let number = project["number"].as_u64().ok_or("kept project number missing")?;
        require(visible.contains(&number), "kept project is not visible to the person")?;
        require(project["cut"] == false, "kept project's adoptions read is incomplete")?;
        for resource in project["resources"].as_array().ok_or("kept project's adoptions missing")? {
            if resource["role"] == "owned" && resource["part"] == part {
                require(found.is_none(), "kept work/hello adoption is ambiguous")?;
                found = Some(number);
            }
        }
    }
    found.ok_or_else(|| "kept deployment lacks an owned work/hello adoption".to_owned())
}

pub mod tests {
    use serde_json::{Value, json};

    pub fn survivor_lines_name_each_process_its_parent_and_group() {
        use super::survivors::{Row, lines, parse};

        let row = |pid, parent, group, command: &str| Row { pid, parent, group, session: 1, command: command.into() };
        let now = [row(30, 20, 10, "child --flag"), row(40, 20, 20, "other")];
        let stopped = [row(10, 1, 10, "leader"), row(20, 10, 10, "parent"), row(99, 1, 99, "unrelated")];
        for (left, at_stop, expected) in [
            (vec![30], &[][..], "SURVIVOR pid=30 parent=20 group=10 session=1 command=child --flag\n"),
            (vec![31], &stopped[..], "SURVIVOR pid=31 gone\n"),
            (
                vec![30, 31, 40],
                &stopped[..],
                concat!(
                    "SURVIVOR pid=30 parent=20 group=10 session=1 command=child --flag\n",
                    "AT STOP pid=10 parent=1 group=10 session=1 command=leader\n",
                    "AT STOP pid=20 parent=10 group=10 session=1 command=parent\n",
                    "SURVIVOR pid=31 gone\n",
                    "SURVIVOR pid=40 parent=20 group=20 session=1 command=other\n",
                    "AT STOP pid=20 parent=10 group=10 session=1 command=parent\n",
                ),
            ),
        ] {
            assert_eq!(lines(&left, &now, at_stop), expected);
        }
        for (stat, expected) in [
            ("30 (a name ) ( with spaces) S 20 10 1 0 999", Some((20, 10, 1))),
            ("30 (name) S 20 10", None),
            ("30 (name) S bad 10 1", None),
            ("no closing delimiter", None),
        ] {
            let parsed = parse(30, stat, b"child\0--flag\0");
            assert_eq!(parsed.as_ref().map(|row| (row.parent, row.group, row.session)), expected, "{stat}");
            if let Some(parsed) = parsed {
                assert_eq!(lines(&[30], &[parsed], &[]), lines(&[30], &now, &[]));
            }
        }
    }

    fn person() -> Value {
        json!({"projects":[{"project":2,"name":{"bytes":"hello"}},
            {"project":7,"name":{"bytes":"renamed"}}],"cut":false})
    }

    fn adopted(project: u64, forge: u16, repository: u32, role: &str) -> Value {
        json!({"number":project,"resources":[{"role":role,"home":true,
            "part":{"connector":1,"kind":"resource","key":{"path":[
                format!("{forge:04x}"),format!("{repository:08x}")]}}}],"cut":false})
    }

    pub fn a_kept_deployment_with_no_merged_pulls_can_find_its_project() {
        let projects = [adopted(2, 1, 9, "owned"), adopted(7, 3, 42, "owned")];
        assert_eq!(super::kept_project(&person(), &projects, 3, 42), Ok(7));
    }

    pub fn a_kept_project_matches_the_owned_adoptions_identity() {
        for (forge, repository, role) in [(1, 42, "owned"), (3, 9, "owned"), (3, 42, "context")] {
            let projects = [adopted(7, forge, repository, role)];
            assert!(super::kept_project(&person(), &projects, 3, 42).is_err());
        }
        let mut project = adopted(7, 3, 42, "owned");
        project["resources"][0]["part"]["connector"] = json!(2);
        assert!(super::kept_project(&person(), &[project], 3, 42).is_err());
    }

    pub fn a_kept_project_requires_complete_visible_unambiguous_adoptions() {
        let project = adopted(7, 3, 42, "owned");
        for me in [
            json!({}),
            json!({"projects":[],"cut":false}),
            json!({"projects":[{"project":7}],"cut":true}),
            json!({"projects":[{"project":0}],"cut":false}),
        ] {
            assert!(super::kept_project(&me, std::slice::from_ref(&project), 3, 42).is_err());
        }
        let projects = [adopted(2, 3, 42, "owned"), project.clone()];
        assert!(super::kept_project(&person(), &projects, 3, 42).is_err());
        for key in ["cut", "resources", "number"] {
            let mut incomplete = project.clone();
            incomplete.as_object_mut().expect("project").remove(key);
            assert!(super::kept_project(&person(), &[incomplete], 3, 42).is_err());
        }
    }
}
