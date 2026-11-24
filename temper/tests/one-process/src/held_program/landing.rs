//! Reviewed landing from an Open program chat (temper/docs/design/testing/tiers.md, 1.5).
use super::{
    Caller, Exit, Host, Local, Proc, Story, Time, command, declare, documents, fixtures, observations, run, sign_in,
    stories, task_number, verify, world,
};
use serde_json::{Value, json};
use skein_fake_forge_domain::{Observation, api};
use skein_io::kernel::ServiceSignal;
use smith_fake_claude::{Seen, Selection};

#[path = "../../../../testing/landing_story.rs"]
mod story;
use story::{
    CODING_WORDS, ChangeAuthority, EDIT, LANDED, README, REQUESTED, REVIEW_WORDS, change_result, delegate_input,
    readme_text, verdict,
};

const WORDS: &str = "Land a small change";
const ACCOUNT_LIST: usize = 3;
const CHAT: usize = 11;
const FIRST_REPLY: usize = 13;
const TREE: usize = 14;
const LAST_REPLY: usize = 17;
const ENDED: usize = 20;

thread_local! { static LANDING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
pub(super) fn active() -> bool {
    LANDING.with(std::cell::Cell::get)
}

pub(super) fn configure(value: &mut documents::Configuration) {
    let forge = value.forge.to_string();
    let policy = &mut value["world_settings"]["policies"][0]["policy"];
    let grants = [1, 2, 3, 4, 7, 8, 10, 11, 12, 13]
        .map(|kind| json!({"connector":1,"kind":kind,"pattern":{"segments":["forge"],"last":"open","value":""}}));
    let authority = json!({"tools":1,"grants":grants,"delegation":{"kinds":[{"kind":"charter","number":1},{"kind":"charter","number":4},{"kind":"procedure","number":2}],"tasks":8,"depth":4},"budget":{"spend":100,"deadline_unix_seconds":null},"notes":0,"note_resources":[]});
    policy["ceiling"] = authority.clone();
    policy["roles"][0]["authority"] = authority;
    policy["roles"][0]["authority"]["budget"]["spend"] = 64.into();
    policy["projections"] = grants.into();
    value["world_settings"]["landing"]["projects"] = json!([{"project":1,"rules":[{
        "connector":1,"kind":4,"pattern":{"segments":["forge",forge,"acme","app","branch"],"last":"exact","value":"main"},
        "enforce":true,"ci":true,"up_to_date":true,
        "gates":[{"number":1,"blocking":true,"freshness":"exact"}],"approvals":[]
    }]}]);
    // Park the chat before running its coder, then run the reviewer and resume the chat.
    value["workers"][0]["slots"] = 1.into();
    // The task tree, the delegation and all three landing requirements fit at once.
    value.jig_over_all([
        ("hub.slots", 1),
        ("documents.rows", u64::from(temper_client_tools::limits::ROWS)),
        ("tasks.tasks", 8),
        ("tasks.project_tasks", 8),
        ("tasks.tree_tasks", 8),
        ("tasks.result_bytes", 512),
        ("tasks.authority_bytes", 512),
        ("tasks.authority_grants", 12),
        ("tasks.authority_segments", 9),
        ("tasks.executor_kinds", 4),
        ("people.amendment_bytes", 2048),
        ("people.inbox_tasks", 16),
        ("authority.grants", 16),
        ("authority.requirements", 3),
    ]);
}

pub(super) fn give(given: &mut temper_service::stock::Given) {
    let temper_service::stock::Given::Own { jig, .. } = given else { unreachable!("landing defaults") };
    jig.templates[0].flags.ended_by = jig_domain_tasks::EndedBy::Requester;
}

struct Judge {
    controls: skein_world::Controls,
    answered: usize,
    started: bool,
    stopping: bool,
    drained: bool,
    finding: Option<String>,
    deadline: Time,
}

impl Judge {
    fn advance(caller: &mut Caller, at: usize) -> Result<(), String> {
        match at {
            ACCOUNT_LIST => {
                verify(caller, ACCOUNT_LIST)?;
                command(caller, &["account", "list", "--json"]);
                command(
                    caller,
                    &[
                        "forge",
                        "connect",
                        "forgejo-admin",
                        "/setup/admin.token",
                        "--api",
                        "127.0.0.1:3000",
                        "--git",
                        "http://127.0.0.1:3000",
                        "--key",
                        "world-forge",
                    ],
                );
                command(caller, &["forge", "list", "--until", "connected:1"]);
                command(
                    caller,
                    &[
                        "repository",
                        "adopt",
                        "--project",
                        "1",
                        "--owner",
                        "acme",
                        "--name",
                        "app",
                        "--landing",
                        "main",
                        "--key",
                        "adopt-app",
                    ],
                );
                command(caller, &["chat", "start", "--project", "1", "--key", "program-land", WORDS]);
            }
            CHAT => {
                let task = task_number(&caller.answers[CHAT].output)?.to_owned();
                command(caller, &["task", "show", &task, "--until", "replied"]);
                command(caller, &["chat", "show", &task]);
                command(caller, &["task", "show", &task, "--tree", "--json"]);
            }
            TREE => {
                let (child, digest) = tree(&caller.answers[TREE].output)?;
                let task = task_number(&caller.answers[CHAT].output)?.to_owned();
                command(caller, &["task", "show", &child.to_string(), "--until", "ended"]);
                command(caller, &["task", "show", &task, "--since", &digest, "--until", "replied"]);
                command(caller, &["chat", "show", &task]);
                command(caller, &["task", "show", &task, "--until", "phase:idle"]);
                command(caller, &["task", "cancel", &task, "Change landed"]);
                command(caller, &["task", "show", &task, "--until", "ended", "--json"]);
            }
            _ => {}
        }
        Ok(())
    }

    fn act_checked(&mut self, now: Time, procs: &mut [Proc]) -> Result<(), String> {
        let [Proc::Service(service), Proc::Other(Local::World(super::HarnessProcess::Calls(caller))), hosted @ ..] =
            procs
        else {
            return Err("service and caller first".into());
        };
        if !self.started
            && let Some(link) = service.service.links().pop()
        {
            sign_in(caller, link);
            command(caller, &["account", "add-key", "1", "/setup/llm.key", "--deployment", "--key", "direct-fallback"]);
            declare(caller);
            self.started = true;
        }
        while self.answered < caller.answers.len() {
            let answer = &caller.answers[self.answered];
            if answer.exit != Exit::Code(0) {
                return Err(format!(
                    "call {}: {:?}: {} {}",
                    self.answered,
                    answer.exit,
                    String::from_utf8_lossy(&answer.output),
                    String::from_utf8_lossy(&answer.error)
                ));
            }
            Self::advance(caller, self.answered)?;
            self.answered += 1;
            self.deadline = now.saturating_add(temper_client_shell_cli::limits::DEADLINE);
        }
        if self.answered == ENDED + 1 && !self.stopping {
            caller.closed = true;
            self.controls.signal(0, ServiceSignal::Terminate);
            for process in hosted {
                match process {
                    Proc::Forge(peer) => peer.shutdown(),
                    Proc::Peer(peer) => peer.shutdown(),
                    _ => {}
                }
            }
            self.stopping = true;
        }
        Ok(())
    }
}

fn tree(output: &[u8]) -> Result<(u64, String), String> {
    let value: Value = serde_json::from_slice(output).map_err(|why| why.to_string())?;
    let tree = &value["tree"];
    let root = tree["root"]["task"].as_u64().ok_or_else(|| format!("tree root: {value}"))?;
    let mut children = tree["rows"].as_array().ok_or("tree rows")?.iter().filter(|node| node["parent"] == root);
    let child = children.next().and_then(|node| node["chip"]["task"].as_u64()).ok_or("child task number")?;
    if children.next().is_some() {
        return Err(format!("expected one child task: {value}"));
    }
    let digest = value["task"]["digest"].as_str().ok_or_else(|| format!("task digest: {value}"))?;
    Ok((child, digest.to_owned()))
}

impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        if self.finding.is_none() {
            self.finding = self.act_checked(now, procs).err();
        }
    }
    fn watch(&mut self, _: Time, procs: &[Proc]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }
        let Proc::Service(service) = &procs[0] else { return Err("service first".into()) };
        if let Some(why) = &service.observer.finding {
            return Err(why.clone());
        }
        if let Some(end) = service.service.end() {
            if !self.stopping || *end != temper_service::End::Engine(temper_service::EngineEnd::Drained) {
                return Err(format!("service ended at call {}: {end:?}", self.answered));
            }
            self.drained = true;
        }
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.drained).then_some(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.drained && now >= self.deadline).then(|| format!("landing after {} calls", self.answered))
    }
    fn passed(&self) -> bool {
        self.drained && self.finding.is_none()
    }
}

/// Run the reviewed edit through the program backend, then read its resumed chat.
pub fn reviewed_edit() {
    stories::set(Story::Adopt);
    stories::setup_settings(world::Phase::Story);
    world::lower_store(Some(std::rc::Rc::new(std::cell::RefCell::new(super::Store::new()))));
    let transport = stories::forge_with_readme(Story::Adopt, Some((README, EDIT)));
    let outcome = LANDING.with(|slot| {
        slot.set(true);
        let outcome = run(&program_story(fixtures::REPOSITORY_TAG, README), &transport, |controls| Judge {
            controls,
            answered: 0,
            started: false,
            stopping: false,
            drained: false,
            finding: None,
            deadline: Time::ZERO.saturating_add(temper_client_shell_cli::limits::DEADLINE),
        });
        slot.set(false);
        outcome
    });
    world::lower_store(None);
    assert!(outcome.procs.iter().all(Host::is_empty), "landing world drains every process");
    let Proc::Other(Local::World(super::HarnessProcess::Calls(caller))) = &outcome.procs[1] else {
        panic!("caller second")
    };
    check_chat(caller);
    check_forge(&mut transport.forge.borrow_mut());
    let peer = outcome
        .procs
        .iter()
        .find_map(|process| if let Proc::Peer(peer) = process { Some(peer) } else { None })
        .expect("direct peer hosted");
    assert!(peer.observations().is_empty(), "no direct request: {:?}", peer.observations());
    check_program(&observations(&outcome.machine, super::SOURCE));
}

fn check_chat(caller: &Caller) {
    assert_eq!(caller.answers.len(), ENDED + 1);
    assert!(caller.answers.iter().all(|answer| answer.exit == Exit::Code(0)), "including final chat end join");
    let first = std::str::from_utf8(&caller.answers[FIRST_REPLY].output).expect("first chat read");
    assert_eq!(first.matches(REQUESTED).count(), 1, "{first}");
    let text = std::str::from_utf8(&caller.answers[LAST_REPLY].output).expect("final chat read");
    assert!(text.find(REQUESTED).expect("first reply") < text.find(LANDED).expect("last reply"), "{text}");
    for reply in [REQUESTED, LANDED] {
        assert_eq!(text.matches(reply).count(), 1, "{text}");
    }
}

fn check_forge(forge: &mut skein_fake_peers::forge::Forge) {
    let mut merged = None;
    while let Some(observation) = forge.domain.pop_observation() {
        if let Observation::Merged { repository, head, .. } = observation {
            assert_eq!(repository.as_ref(), b"acme/app");
            merged = Some(head);
        }
    }
    let head = merged.expect("forge recorded a merge");
    let inspect = |read| forge.domain.inspect(&forge.config, b"acme/app", &read).expect("forge read");
    let api::Answer::Checks(checks) = inspect(api::Read::Checks { commit: head }) else { panic!("merge checks") };
    assert!(!checks.is_empty() && checks.iter().all(|check| check.status.state == api::Check::Passed));
    let api::Answer::Commit(main) = inspect(api::Read::Branch { branch: b"main".as_slice().into() }) else {
        panic!("main commit")
    };
    let object = forge.domain.object(main).expect("main object");
    assert!(object.tree.get(b"README.md".as_slice()).expect("main README").ends_with(EDIT));
}

fn check_program(bytes: &[u8]) {
    let observations = smith_fake_claude::observations(bytes);
    let prints: Vec<_> =
        observations.iter().filter(|seen| seen.arguments.first().is_some_and(|arg| arg == "-p")).collect();
    assert_eq!(prints.len(), 4, "{observations:?}");
    let argument = |index: usize, flag: &str| {
        prints[index].arguments.windows(2).find(|pair| pair[0] == flag).expect("program flag")[1].as_str()
    };
    assert_eq!(argument(3, "--resume"), argument(0, "--session-id"));
    assert!(prints[..3].iter().all(|print| !print.arguments.iter().any(|arg| arg == "--resume")));
    assert_eq!(
        smith_fake_claude::selections(bytes),
        [
            Seen::Selected { script: Selection::Cued(0), turn: 0 },
            Seen::Selected { script: Selection::Cued(1), turn: 0 },
            Seen::Selected { script: Selection::Cued(2), turn: 0 },
            Seen::Selected { script: Selection::Cued(0), turn: 1 },
        ]
    );
    let acts: Vec<_> =
        smith_fake_claude::seen(bytes).into_iter().filter(|seen| matches!(seen, Seen::Acted { .. })).collect();
    assert_eq!(
        acts,
        [
            Seen::Acted { tool: "Read".into(), path: "README.md".into(), error: false },
            Seen::Acted { tool: "Edit".into(), path: "README.md".into(), error: false },
        ]
    );
}

/// The program twin uses built-in file acts and an Open chat's text replies.
fn program_story(repository: u64, readme: &[u8]) -> smith_fake_claude::Story {
    use smith_fake_claude::{Script, Step, Story, Turn};
    let call = |tool: &str, input| Step::Call { tool: tool.into(), input };
    let (original, changed) = readme_text(readme, EDIT);
    Story {
        // The Open system cue keeps the session cursor when a refreshed brief repeats the request.
        cued: vec![
            Script {
                cue: "## Replying\n\nEnd your turn with text to reply".into(),
                turns: vec![
                    Turn {
                        steps: vec![
                            call(
                                "delegate",
                                delegate_input(
                                    ChangeAuthority { coding_kind: 1, scope: None },
                                    repository,
                                    CODING_WORDS,
                                ),
                            ),
                            Step::Text(REQUESTED.into()),
                        ],
                        ..Turn::default()
                    },
                    Turn { steps: vec![Step::Text(LANDED.into())], ..Turn::default() },
                ],
            },
            Script {
                cue: format!("## Task\n\n{CODING_WORDS}"),
                turns: vec![Turn {
                    steps: vec![
                        Step::Act { tool: "Read".into(), input: json!({"file_path":"README.md"}) },
                        Step::Act {
                            tool: "Edit".into(),
                            input: json!({"file_path":"README.md","old_string":original,"new_string":changed}),
                        },
                        Step::Outcome { input: json!({"outcome":change_result()}) },
                    ],
                    ..Turn::default()
                }],
            },
            Script {
                cue: REVIEW_WORDS.into(),
                turns: vec![Turn {
                    steps: vec![Step::Outcome { input: json!({"outcome":verdict()}) }],
                    ..Turn::default()
                }],
            },
        ],
        ..Story::default()
    }
}
