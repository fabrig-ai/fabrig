//! B0's recorded question, approval, review and landing on the composed world.
use jig_domain_core::{
    documents::{InboxObject, PhaseLine, TaskLink},
    reads::PageRow,
};
use jig_domain_tasks::{self as tasks, TaskResult};
use serde_json::Value;
use skein_fake_forge_domain::api::{Protection, Setup};
use skein_fake_forge_domain::{Observation, object_bytes};
use skein_fake_llm_domain::api::{Part, Query, Role as LlmRole};
use smith_domain_session::llm;
use smith_protocol_llm_world::fixture::Dialect;
use std::fmt::Write;
use temper_engine_domain_forge::{Adoption, Role};
use temper_engine_service_vocabulary::settings;
use temper_system_world::composed::{WalkRepository, World, provider::Playback, review_rule};

const TAPE: &[u8] = include_bytes!("../../../../real-loop/tests/fixtures/b0/llm.json");
const FORGE: &[u8] = include_bytes!("../../../../real-loop/tests/fixtures/b0/forge.json");
const PERSON: &[u8] = include_bytes!("../../../../real-loop/tests/fixtures/b0/person.json");
const SEED: u64 = 17;

fn repository() -> WalkRepository {
    let document = temper_fake::forge::Document::decode(FORGE).expect("B0 forge document");
    let [repository] = document.repositories() else { panic!("one widget repository") };
    let checks = temper_fake::forge::checks(repository.ci.as_ref());
    let protection = Protection {
        branch: repository.landing.clone(),
        contexts: checks.contexts.clone(),
        approvals: 0,
        dismiss_stale: false,
        queue: false,
        linear: false,
        code_owners: false,
        bypassing_users: Box::default(),
    };
    let host = b"twin.test";
    WalkRepository {
        seed: Setup {
            id: repository.id.expect("recorded repository id"),
            name: [&*repository.owner, b"/", &*repository.name].concat().into_boxed_slice(),
            default: repository.landing.clone(),
            tree: repository.tree.clone().expect("recorded widget seed"),
            labels: Box::default(),
            checks,
            workflows: Box::default(),
            protection: Some(protection),
            hooked: false,
        },
        adoption: Adoption {
            project: 1,
            home: true,
            forge: 1,
            host: host.as_slice().into(),
            owner: repository.owner.clone(),
            name: repository.name.clone(),
            prefix: b"temper/".as_slice().into(),
            role: Role::Owned,
            landing: repository.landing.clone(),
            ci: true,
            checks: Box::default(),
        },
        binding: settings::Forge {
            log_origins: None,
            mode: temper_engine_domain_forge::Mode::Connected,
            number: 1,
            kind: settings::Kind::Forgejo,
            api: jig_protocol_config::Address::Host { name: host.as_slice().into(), port: 80 },
            transport: settings::Transport::Plaintext,
            git: [b"http://".as_slice(), host].concat().into_boxed_slice(),
            git_program: None,
            web: None,
            app_read_only: false,
            user: 19,
            identity: 4,
        },
        person: 11,
    }
}

const CHAT: u64 = 1;
const GOAL: u64 = 2;
const ADVISOR: u64 = 3;
const CHANGE: u64 = 4;
const PRODUCER: u64 = 5;
const REVIEWER: u64 = 6;
const QUESTION: u64 = 2;
const CUES: [(u64, &str); 5] = [
    (CHAT, "## Replying"),
    (GOAL, "You are a goal: the work your person asked for"),
    (
        ADVISOR,
        "You are an advisor: you read what your brief asks and report what you found; you change nothing and ask no one.",
    ),
    (PRODUCER, "Add CSV export to the widget in src/main.rs."),
    (REVIEWER, "Review this change at the named head"),
];

fn words<'a>(person: &'a Value, field: &str) -> &'a str {
    person[field].as_str().expect("B0 person's words")
}

#[test]
fn a_green_light_in_the_inbox_lets_the_change_land() {
    let person: Value = serde_json::from_slice(PERSON).expect("B0 person's script");
    let document = temper_fake::llm::Document::decode(TAPE).expect("B0 LLM document").with_strict(true);
    let playback = Playback::new(&document, Dialect::OpenAi.configuration(), SEED);
    let mut world =
        World::recorded_walk(SEED, playback, repository(), review_rule(), words(&person, "request").as_bytes());
    let seed = world.branch(b"main");
    world.run();
    check_waiting(&mut world, &person);
    assert_eq!(world.branch(b"main"), seed, "main stays at the seed until approval");
    assert_eq!(world.branches(), [b"main".to_vec()]);
    assert!(
        world.forge_observations().iter().all(|seen| !matches!(
            seen,
            Observation::Moved { .. } | Observation::Opened { kind: skein_fake_forge_domain::api::Kind::Pull, .. }
        )),
        "no push or pull before the answer"
    );
    let checkpoint = world.forge_observations().len();
    world.answer_inbox_question(CHAT, QUESTION, words(&person, "answer").as_bytes());
    world.finish_goal();
    world.run();
    check_agents(&world, &person);
    check_landing(&mut world, checkpoint, &person);
    assert_eq!(world.task_overview(CHAT).chip.phase, PhaseLine::Active(tasks::Active::Idle));
    let mut operations = skein_lib::List::with_capacity(1);
    operations
        .push(jig_domain_people::Operation::Cancel {
            task: CHAT,
            reason: b"Close the chat.".as_slice().into(),
            grace: None,
        })
        .expect("one close act");
    let reply = world.notes_act(jig_domain_people::Ask::Compound { project: 1, operations }, [46; 16]);
    assert!(matches!(reply, jig_domain_people::Reply::Outcome(jig_domain_people::Outcome::Compound { .. })));
    world.run();
    assert!(world.ended_task(CHAT).is_some(), "the chat close joins its end");
    let inbox = world.inbox_needs();
    assert!(inbox.rows.is_empty() && inbox.next.is_none(), "no waiting entry remains");
}

fn check_waiting(world: &mut World, person: &Value) {
    assert_eq!(world.party.last_task, Some(CHAT));
    assert!(chat_text(world).contains("I’ve asked the work to wait for your go-ahead"));
    let inbox = world.inbox_needs();
    let [PageRow::Inbox(question)] = inbox.rows.as_slice() else { panic!("one waiting question: {inbox:?}") };
    assert!(!inbox.cut && inbox.next.is_none());
    assert!(matches!(&question.task, TaskLink::Live(chip) if chip.number == CHAT));
    assert_eq!(question.object, InboxObject::Question { message: QUESTION });
    let question = world.task_question(CHAT, QUESTION);
    assert_eq!((question.task.number, question.message, question.from), (CHAT, QUESTION, tasks::Party::Task(GOAL)));
    assert_eq!(question.words.bytes.as_ref(), words(person, "question").as_bytes());
    let tree = world.task_tree(CHAT);
    let [goal] = tree.rows.as_slice() else { panic!("only the goal before approval: {tree:?}") };
    assert_eq!((tree.root.number, goal.chip.number, goal.parent), (CHAT, GOAL, CHAT));
    assert_eq!(tree.root.phase, PhaseLine::Active(tasks::Active::Idle));
    assert!(tree.next.is_none() && tree.folded.is_empty(), "no unshown work");
    let activity = world.task_activity(CHAT);
    assert!(
        activity.rows.iter().any(|row| {
            let PageRow::Activity { line, .. } = row else { return false };
            match &line.line {
                tasks::Line::Heard(message) => {
                    message.from == tasks::Party::Task(GOAL)
                        && message.words.as_ref() == words(person, "question").as_bytes()
                }
                tasks::Line::Words { message, by, words: body, .. } => {
                    (*message, *by, body.as_ref())
                        == (QUESTION, tasks::Party::Task(GOAL), words(person, "question").as_bytes())
                }
                _ => false,
            }
        }),
        "the plan is in the chat activity"
    );
    let _ = done(world, ADVISOR);
    let advisor = query(world, ADVISOR, 7);
    assert!(calls(advisor).iter().all(|(name, _)| matches!(*name, "list" | "read" | "finish")));
    assert!(heard_tool(advisor, "const TABLE"), "the advisor read the widget source");
    assert!(heard_tool(query(world, GOAL, 4), "Message 2 sent"));
    assert!(
        !world
            .observed
            .queries
            .iter()
            .filter(|(task, _, _)| *task == GOAL)
            .any(|(_, _, query)| { calls(query).iter().any(|(name, _)| *name == "change") }),
        "the goal has not asked for a change"
    );
}

fn check_agents(world: &World, person: &Value) {
    // Each wire query carries the cue for its own task, not a later cue inside a quotation.
    for (task, turns) in [(CHAT, 6), (GOAL, 8), (ADVISOR, 8), (PRODUCER, 8), (REVIEWER, 4)] {
        let selected = world
            .observed
            .queries
            .iter()
            .filter(|(owner, _, _)| *owner == task)
            .map(|(_, _, query)| replies(query))
            .collect::<Vec<_>>();
        assert!(selected.iter().copied().eq(0..turns), "task {task}'s recorded turns: {selected:?}");
    }
    for (task, _, query) in &world.observed.queries {
        let system = utf8(&query.system);
        let own = CUES.iter().filter_map(|(task, cue)| system.find(cue).map(|at| (at, task))).min();
        assert_eq!(own.map(|(_, found)| found), Some(task));
    }
    let chat = query(world, CHAT, 5);
    let chat_calls = calls(chat);
    let goals: Vec<_> = chat_calls.iter().filter(|(name, _)| *name == "goal").collect();
    assert_eq!(goals.len(), 1, "one agreed goal call");
    assert_eq!(goals[0].1["agree"], true);
    assert_eq!(
        world.observed.queries.iter().filter(|(task, _, query)| *task == GOAL && replies(query) == 0).count(),
        1
    );
    assert!(chat_calls.iter().all(|(name, args)| *name != "message" || args["form"] != "answer"));
    let question = chat_calls
        .iter()
        .find(|(name, args)| *name == "message" && args["form"] == "question")
        .expect("chat's question");
    assert_eq!(question.1["target"], GOAL, "the take bound the goal's number as a JSON number");
    let chat_words = told(query(world, CHAT, 4));
    let plan = chat_words.find(words(person, "question")).expect("chat hears the plan");
    let notice = chat_words
        .find("Person answered task 2's question 2 in their inbox; task 2 has the answer and does the work")
        .expect("chat hears the numbered answer notice");
    assert!(plan < notice);
    assert!(!told(chat).contains("Refused:"));

    let asked = calls(query(world, GOAL, 4));
    let question = asked
        .iter()
        .find(|(name, args)| *name == "message" && args["form"] == "question")
        .expect("goal's plan question");
    assert_eq!(question.1["target"], CHAT, "the requester take bound the chat's number");
    assert_eq!(question.1["words"], words(person, "question"));
    let approved = query(world, GOAL, 5);
    assert!(
        told(approved).lines().any(|line| line == format!("1 answers question 2: {}", words(person, "answer"))),
        "the goal hears the exact approval as the chat's answer: {}",
        told(approved)
    );
    assert!(calls(approved).iter().all(|(name, _)| *name != "change"));
    assert!(heard_tool(query(world, GOAL, 6), "Delegated tasks: 4"));
    assert_eq!(calls(query(world, GOAL, 7)).iter().filter(|(name, _)| *name == "change").count(), 1);
}

fn check_landing(world: &mut World, checkpoint: usize, person: &Value) {
    world.assert_review_ci();
    let observations = &world.forge_observations()[checkpoint..];
    let opened: Vec<_> = observations
        .iter()
        .enumerate()
        .filter_map(|(at, seen)| {
            let Observation::Opened {
                kind: skein_fake_forge_domain::api::Kind::Pull,
                number,
                title,
                body,
                branches,
                ..
            } = seen
            else {
                return None;
            };
            Some((at, *number, utf8(title), utf8(body), branches.as_ref().expect("pull branches")))
        })
        .collect();
    let &[(opened_at, pull, title, body, branches)] = opened.as_slice() else { panic!("one pull opened: {opened:?}") };
    let merges: Vec<_> = observations
        .iter()
        .enumerate()
        .filter_map(|(at, seen)| {
            let Observation::Merged { number, head, commit, base, .. } = seen else { return None };
            Some((at, *number, *head, *commit, base.as_ref()))
        })
        .collect();
    let &[(merged_at, merged_pull, merged_head, merge, b"main")] = merges.as_slice() else {
        panic!("one merge to main: {merges:?}")
    };
    assert_eq!((merged_pull, merged_head), (pull, branches.commit), "CI and review designate the merged head");
    let pushed_at = observations
        .iter()
        .position(|seen| {
            matches!(seen,
                Observation::Moved { branch, to, .. } if branch == &branches.head && *to == merged_head
            )
        })
        .expect("the worker pushed the pull's head");
    let green_at = observations
        .iter()
        .position(|seen| {
            matches!(seen,
                Observation::Reported { commit, context, state: skein_fake_forge_domain::api::Check::Passed, .. }
                    if *commit == merged_head && context.as_ref() == temper_fake::forge::CONTEXT
            )
        })
        .expect("the check was green on the merged head");
    assert!(pushed_at < opened_at && opened_at < merged_at, "push, open, land");
    assert!(pushed_at < green_at && green_at < merged_at, "CI sees the push before landing");
    let merge_id = hex(merge);
    check_review(query(world, REVIEWER, 3), pull, title, body, branches);

    let delivered = told(query(world, GOAL, 7));
    let delivered_id = delivered
        .split_once(&format!("Result from task {CHANGE}:"))
        .and_then(|(_, result)| result.split_whitespace().next())
        .expect("goal hears the change's merge id");
    assert_eq!(delivered_id, merge_id);
    let pull_name = format!("Pull request #{pull}");
    let TaskResult::Report { words: report } = done(world, GOAL) else { panic!("goal reports") };
    let report = utf8(report);
    for name in [merge_id.as_str(), pull_name.as_str(), "merged into main"] {
        assert!(report.contains(name), "goal's report must name {name}: {report}");
    }
    let chat = chat_text(world);
    let last = chat.lines().next_back().expect("chat's last reply");
    for name in [merge_id.as_str(), pull_name.as_str(), words(person, "last_word")] {
        assert!(last.contains(name), "chat's last reply must name {name}: {last}");
    }
    let (head, tree) = world.branch(b"main");
    assert_eq!(head, merge);
    let source = tree.get(b"src/main.rs".as_slice()).expect("widget source landed");
    assert!(utf8(source).contains(r#"writeln!(writer, "fruit,count")?;"#), "the CSV header landed");
    assert_eq!(source.as_slice(), recorded_write().as_bytes(), "the walk's whole program landed");
    let _ = done(world, PRODUCER);
    let TaskResult::Verdict { code: 1, words: verdict, .. } = done(world, REVIEWER) else {
        panic!("reviewer approved")
    };
    assert!(utf8(verdict).starts_with("Approve."));
}

fn check_review(query: &Query, pull: u64, title: &str, body: &str, branches: &skein_fake_forge_domain::Branches) {
    let brief = utf8(&query.system);
    assert!(brief.contains(include_str!("../../../../../crates/temper-engine-service-stock/src/stock/review.txt")));
    let pull_at = brief.find("## Pull request").expect("review's pull section");
    let branch = utf8(&branches.head);
    let base = utf8(&branches.base);
    let head = hex(branches.commit);
    let subject = format!("Pull request: #{pull}\nBranch: {branch}\nBase: {base}\nHead: {head}\n");
    assert!(brief.find(&subject).expect("named review subject") < pull_at);
    for words in [
        format!("Title: {title}"),
        format!("Body: {body}"),
        "Files and diff:".into(),
        "File: src/main.rs".into(),
        "After:\n".into(),
        "fruit,count".into(),
    ] {
        assert!(brief[pull_at..].contains(&words), "review includes {words}: {brief}");
    }
    let calls = calls(query);
    assert_eq!(calls[0].0, "read_issues");
    assert_eq!(calls[0].1["number"], pull);
    assert!(heard_tool(query, &format!("Pull {pull} head {branch} commit {head}")), "numbered pull read succeeds");
    assert_eq!(calls[1].0, "read_issues");
    assert_eq!(calls[1].1["number"], pull);
    assert_eq!(calls[1].1["commit"], head);
    assert_eq!(calls[2].0, "read_ci");
    assert_eq!(calls[2].1["commit"], head);
    assert!(heard_tool(query, "fn write_csv"), "reviewer read the produced source");
    assert!(!told(query).contains("Error:"), "successful source review: {}", told(query));
}

fn query(world: &World, task: u64, said: usize) -> &Query {
    let mut queries =
        world.observed.queries.iter().filter(|(owner, _, query)| *owner == task && replies(query) == said);
    let (_, _, query) = queries.next().unwrap_or_else(|| panic!("task {task} with {said} prior replies"));
    assert!(queries.next().is_none(), "one request for task {task}'s turn {said}");
    query
}

fn replies(query: &Query) -> usize {
    query.messages.iter().filter(|message| message.role == LlmRole::Assistant).count()
}

fn calls(query: &Query) -> Vec<(&str, Value)> {
    query
        .messages
        .iter()
        .filter(|message| message.role == LlmRole::Assistant)
        .flat_map(|message| &message.parts)
        .filter_map(|part| {
            let Part::ToolCall { name, arguments, .. } = part else { return None };
            Some((utf8(name), serde_json::from_slice(arguments).expect("observed tool JSON")))
        })
        .collect()
}

fn told(query: &Query) -> String {
    query
        .messages
        .iter()
        .filter(|message| message.role == LlmRole::User)
        .flat_map(|message| &message.parts)
        .filter_map(|part| match part {
            Part::Text { text } | Part::ToolOutput { output: text, .. } => Some(utf8(text)),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn heard_tool(query: &Query, words: &str) -> bool {
    query
        .messages
        .iter()
        .flat_map(|message| &message.parts)
        .any(|part| matches!(part, Part::ToolOutput { output, is_error: false, .. } if utf8(output).contains(words)))
}

fn chat_text(world: &World) -> String {
    world
        .committed_turns(CHAT)
        .iter()
        .flat_map(|turn| &turn.messages)
        .filter(|message| message.role == llm::Role::Assistant)
        .flat_map(|message| &message.content)
        .filter_map(|block| block.text().map(utf8))
        .collect::<Vec<_>>()
        .join("\n")
}

fn done(world: &World, task: u64) -> &TaskResult {
    let row = world.ended_task(task).unwrap_or_else(|| panic!("task {task} ended"));
    let tasks::Phase::Ended(tasks::Ending::Done(result)) = &row.phase else {
        panic!("task {task} succeeded: {:?}", row.phase)
    };
    result
}

fn utf8(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("the peer's text is UTF-8")
}

fn hex(commit: u64) -> String {
    object_bytes(commit).iter().fold(String::new(), |mut hex, byte| {
        write!(hex, "{byte:02x}").expect("commit hex");
        hex
    })
}

fn recorded_write() -> String {
    let tape: Value = serde_json::from_slice(TAPE).expect("recorded tape");
    tape["cued"][3]["turns"]
        .as_array()
        .expect("producer turns")
        .iter()
        .flat_map(|turn| turn["lines"].as_array().expect("turn lines"))
        .find(|line| line["call"] == "write" && line["arguments"]["path"] == "src/main.rs")
        .expect("recorded src/main.rs write")["arguments"]["content"]
        .as_str()
        .expect("recorded source")
        .into()
}
