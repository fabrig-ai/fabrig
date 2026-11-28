//! B1's recorded approval, failed CI, repair and landing (temper/docs/design/testing/tiers.md, 1.1).
use jig_domain_core::{
    documents::{Ending, InboxObject, PhaseLine, ResultValue, TaskLink},
    reads::PageRow,
};
use jig_domain_tasks::{self as tasks, TaskResult};
use serde_json::Value;
use skein_fake_forge_domain::{Observation, api};
use smith_protocol_llm_world::fixture::Dialect;
use temper_system_world::composed::{World, provider::Playback, review_rule, stretches};

use super::twin::{HOST, calls, chat_text, done, hex, query, replies, repository, told, utf8};

const TAPE: &[u8] = include_bytes!("../../../../real-loop/tests/fixtures/b1/llm.json");
const FORGE: &[u8] = include_bytes!("../../../../real-loop/tests/fixtures/b1/forge.json");
const PERSON: &[u8] = include_bytes!("../../../../real-loop/tests/fixtures/b1/person.json");
const TAKES: &[u8] = include_bytes!("../../../../real-loop/tests/fixtures/b1/takes.json");
const SEED: u64 = 17;
const CHAT: u64 = 1;
const GOAL: u64 = 2;
const CHANGE: u64 = 3;
const PRODUCER: u64 = 4;
const REPAIR: u64 = 5;
const REVIEW: u64 = 6;
const QUESTION: u64 = 1;
const PULL: u64 = 2;
const CHECK: &[u8] = b"/ check (pull_request)";
const CHANGES: &[u8] = b"/ changes (pull_request)";
const REPOSITORY: &[u8] = b"playground/widget";
const NOTE: &str = "Add an optional `--csv` command-line mode that prints the fruit table with a `fruit,count` header. The existing aligned table output remains the default; unexpected or extra arguments produce a usage error.";
const REPAIR_CUE: &str = "Repair the failed CI on the change branch";

fn words<'a>(person: &'a Value, field: &str) -> &'a str {
    person[field].as_str().expect("B1 person's words")
}

#[test]
fn a_change_failing_ci_is_repaired_and_lands() {
    let person: Value = serde_json::from_slice(PERSON).expect("B1 person's script");
    // Only this wire twin plays the extra rounds; the real-loop recording is unchanged.
    let mut tape: Value = serde_json::from_slice(TAPE).expect("B1 LLM recording");
    let (menu, stretch) = stretches::document();
    tape["menu"] = menu;
    let turns = tape["cued"][2]["turns"].as_array_mut().expect("producer's turns");
    let before_finish = turns.len() - 1;
    assert_eq!(turns[before_finish]["lines"][0]["call"], "finish");
    turns.insert(before_finish, stretch);
    let bytes = serde_json::to_vec(&tape).expect("B1 with drawn rounds");
    let document = temper_fake::llm::Document::decode(&bytes).expect("B1 LLM document").with_strict(true);
    let end = before_finish + usize::try_from(stretches::ROUNDS).expect("stretch length");
    let playback =
        Playback::new(&document, Dialect::OpenAi.configuration(), SEED).with_stretch(PRODUCER, before_finish..end);
    let forge = temper_fake::forge::Document::decode(FORGE).expect("B1 forge document");
    let [widget] = forge.repositories() else { panic!("one widget repository") };
    let mut checks = temper_fake::forge::checks(widget.ci.as_ref());
    checks.contexts = Box::new([CHECK.into(), CHANGES.into()]);
    checks.cues = Box::new([api::ContextCue {
        context: CHANGES.into(),
        cue: api::Cue::PullFile {
            path: b"changes/{pull}.md".as_slice().into(),
            no_pull: api::Check::Skipped,
            step: b"changes".as_slice().into(),
            log: b"Missing changes/{pull}.md".as_slice().into(),
        },
    }]);
    let repository = repository(widget, checks);
    let mut world =
        World::recorded_walk(SEED, playback, repository, review_rule(), words(&person, "request").as_bytes());
    let seed = world.branch(b"main");
    assert_eq!(world.branches(), [b"main".to_vec()]);
    world.run();
    check_waiting(&mut world, &person);
    assert_eq!(world.branch(b"main"), seed, "main stays at the seed until approval");
    assert_eq!(world.branches(), [b"main".to_vec()]);
    assert!(
        world
            .forge_observations()
            .iter()
            .all(|seen| !matches!(seen, Observation::Moved { .. } | Observation::Opened { kind: api::Kind::Pull, .. })),
        "no push or pull before approval"
    );

    world.answer_inbox_question(CHAT, QUESTION, words(&person, "answer").as_bytes());
    world.finish_goal();
    world.run();
    check_agreement(&world, &person);
    let (first, repaired, merge) = check_ci(&mut world);
    check_repair(&mut world, first);
    check_landing(&mut world, &person, repaired, merge, &seed.1);
    world.observed.stretches.check().expect("wire producer played the stretch");
    assert!(
        world
            .observed
            .stretches
            .refusals()
            .contains_key(&(stretches::Refusal::ReadReference, stretches::Target::NeverIssued)),
        "wire stretch checked a read of a never-issued task"
    );
    eprintln!("wire stretches {:?}", world.observed.stretches);
    assert_eq!(world.task_overview(CHAT).chip.phase, PhaseLine::Active(tasks::Active::Idle));
    let mut operations = skein_lib::List::with_capacity(1);
    operations
        .push(jig_domain_people::Operation::Cancel { task: CHAT, reason: Box::default(), grace: None })
        .expect("one close act");
    let reply = world.notes_act(jig_domain_people::Ask::Compound { project: 1, operations }, [46; 16]);
    assert!(matches!(reply, jig_domain_people::Reply::Outcome(jig_domain_people::Outcome::Compound { .. })));
    world.run();
    assert!(world.ended_task(CHAT).is_some(), "the chat close joins its end");
    let inbox = world.inbox_needs();
    assert!(inbox.rows.is_empty() && !inbox.cut && inbox.next.is_none(), "no question left");
}

fn check_waiting(world: &mut World, person: &Value) {
    assert_eq!(world.party.last_task, Some(CHAT));
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
    assert!(tree.next.is_none() && tree.folded.is_empty(), "no hidden or truncated tree rows");
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
}

fn check_agreement(world: &World, person: &Value) {
    let notice = told(query(world, CHAT, 3));
    assert!(
        notice.find(words(person, "question")).expect("chat hears the question")
            < notice
                .find("Person answered task 2's question 1 in their inbox; task 2 has the answer and does the work")
                .expect("chat hears the numbered answer notice")
    );
    let chat = calls(query(world, CHAT, 4));
    let goals: Vec<_> = chat.iter().filter(|(name, _)| *name == "goal").collect();
    assert_eq!(goals.len(), 1, "chat delegates one goal");
    assert_eq!(goals[0].1["agree"], true, "the goal waits for agreement");
    assert_eq!(
        world.observed.queries.iter().filter(|(task, _, query)| *task == GOAL && replies(query) == 0).count(),
        1
    );
    assert!(
        chat.iter().all(|(name, args)| *name != "message" || args["form"] != "answer"),
        "chat does not relay the answer"
    );
    let approved = query(world, GOAL, 5);
    let received = told(approved);
    assert!(
        received.lines().any(|line| line == format!("1 answers question 1: {}", words(person, "answer"))),
        "the exact answer reaches the goal: {received}"
    );
    assert!(calls(approved).iter().all(|(name, _)| *name != "change"));
    assert_eq!(calls(query(world, GOAL, 6)).iter().filter(|(name, _)| *name == "change").count(), 1);
}

fn check_ci(world: &mut World) -> (u64, u64, u64) {
    world.assert_review_ci();
    let records = world.forge_observations();
    let moves: Vec<_> = records
        .iter()
        .enumerate()
        .filter_map(|(at, record)| {
            let Observation::Moved { repository, branch, from, to, .. } = record else { return None };
            (repository.as_ref() == REPOSITORY && branch.as_ref() != b"main").then_some((at, branch, *from, *to))
        })
        .collect();
    let &[(pushed, branch, None, first), (pushed_repair, repaired_branch, Some(previous), repaired)] = moves.as_slice()
    else {
        panic!("producer and one repair push: {moves:?}")
    };
    assert_eq!(branch, repaired_branch);
    assert_eq!(previous, first);
    assert_ne!(first, repaired);
    let pulls: Vec<_> = records
        .iter()
        .enumerate()
        .filter_map(|(at, record)| {
            let Observation::Opened { repository, number, branches: Some(branches), .. } = record else { return None };
            Some((at, repository.as_ref(), *number, branches))
        })
        .collect();
    let &[(opened, REPOSITORY, PULL, pull)] = pulls.as_slice() else {
        panic!("one pull through red, repair and green: {pulls:?}")
    };
    assert_eq!((&pull.head, pull.base.as_ref(), pull.commit), (branch, b"main".as_slice(), first));
    let merges: Vec<_> = records
        .iter()
        .enumerate()
        .filter_map(|(at, record)| {
            let Observation::Merged { repository, number, base, head, commit, .. } = record else { return None };
            Some((at, repository.as_ref(), *number, base.as_ref(), *head, *commit))
        })
        .collect();
    let &[(landed_at, REPOSITORY, PULL, b"main", head, merge)] = merges.as_slice() else {
        panic!("one merge to main: {merges:?}")
    };
    assert_eq!(head, repaired);
    assert!(pushed < opened && opened < pushed_repair && pushed_repair < landed_at);
    for (head, context, expected, before) in [
        (first, CHECK, api::Check::Passed, pushed_repair),
        (first, CHANGES, api::Check::Failed, pushed_repair),
        (repaired, CHECK, api::Check::Passed, landed_at),
        (repaired, CHANGES, api::Check::Passed, landed_at),
    ] {
        let sequence: Vec<_> = records
            .iter()
            .enumerate()
            .skip(opened)
            .filter_map(|(at, record)| {
                let Observation::Reported { repository, commit, context: name, state, .. } = record else {
                    return None;
                };
                (repository.as_ref() == REPOSITORY && *commit == head && name.as_ref() == context)
                    .then_some((at, *state))
            })
            .collect();
        let &[(pending, api::Check::Pending), (terminal, state)] = sequence.as_slice() else {
            panic!("head {} context {}: {sequence:?}", hex(head), utf8(context))
        };
        assert_eq!(state, expected);
        assert!(pending < terminal && terminal < before, "CI must be terminal before the repair push or merge");
    }
    (first, repaired, merge)
}

fn check_repair(world: &mut World, first: u64) {
    assert_eq!(world.reviewed_change().result.repair_count, 1);
    let repair = world.task_overview(REPAIR);
    assert_eq!(repair.requester, tasks::Party::Task(CHANGE));
    let change = world.task_overview(CHANGE);
    assert_eq!(change.spec_revision, 0);
    let request = utf8(&change.spec.words.bytes);
    let opening = format!(
        "Repair CI at {}: / changes (pull_request)\n{REPAIR_CUE}. Preserve the requested change, as its current spec and rulings state it, and unrelated base content, \
         including both independent appended lines and existing blank lines, and do not remove content to make a check pass, \
         unless its current spec or a ruling says otherwise.\n\n\
         This change's current spec and rulings: task {CHANGE} at spec revision 0. They come first: its original request, \
         its pull request's title and body, and CI's and reviews' remarks are supporting evidence, and where they differ from these, follow these. \
         To preserve the requested change is to preserve it as its current spec and rulings state it.\nIts current spec, at revision 0:\n{request}",
        &hex(first)[..12],
    );
    assert!(!repair.spec.words.cut);
    assert_eq!(repair.spec.words.bytes.as_ref(), opening.as_bytes());
    let system = utf8(&query(world, REPAIR, 0).system);
    let ci = system.split_once("## CI\n\n").expect("CI brief").1.split_once("\n## ").expect("next section").0;
    assert!(ci.contains(&format!("Repair the failed check at change head {}", &hex(first)[..16])));
    // The typed forge returns the job path; the Forgejo machine joins it to the origin.
    let link = ci
        .split_once("Failed check: / changes (pull_request)\nDescription: CI failed\nLink: ")
        .expect("failed check description and link")
        .1
        .lines()
        .next()
        .expect("job link");
    let run = link
        .strip_prefix("/playground/widget/actions/runs/")
        .and_then(|path| path.strip_suffix("/jobs/1"))
        .expect("changes job on widget");
    assert!(run.parse::<u64>().expect("numbered workflow run") > 0);
    assert!(ci.contains("Failed step: 1 changes\nEnd of its log:\n"), "{ci}");
    let log = ci
        .split_once("End of its log:\n")
        .expect("workflow log")
        .1
        .split_once("\n\nRepair the failed check.")
        .expect("repair instruction follows the log")
        .0;
    assert_eq!(
        log.trim_end(),
        "failing check / changes (pull_request) 0\n\
         failing check / changes (pull_request) 1\n\
         failing check / changes (pull_request) 2\n\
         failing check / changes (pull_request) 3\n\
         Missing changes/2.md",
        "the whole failed step log"
    );
    assert!(!ci.contains("cut"), "whole failure log");
}

fn check_landing(world: &mut World, person: &Value, repaired: u64, merge: u64, seed: &temper_fake_checkout::git::Tree) {
    let scripts = recorded_scripts();
    let expected =
        recorded_source(utf8(seed.get(b"src/main.rs".as_slice()).expect("seed source")), &scripts["cued"][2]["turns"]);
    let review = utf8(&query(world, REVIEW, 0).system);
    assert!(review.contains(&format!("Head: {}\n", hex(repaired))), "review names H2: {review}");
    let diff = review.split_once("Files and diff:").expect("review's repaired diff").1;
    for words in ["File: src/main.rs", "File: changes/2.md", "After:\n", "fruit,count", &expected, NOTE] {
        assert!(diff.contains(words), "H2 review includes {words}: {review}");
    }
    let TaskResult::Verdict { code: 1, words: verdict, .. } = done(world, REVIEW) else { panic!("reviewer approved") };
    assert!(utf8(verdict).starts_with("Approve."));
    assert_eq!(
        utf8(verdict),
        scripts["cued"][4]["turns"][6]["lines"][0]["arguments"]["text"].as_str().expect("recorded verdict")
    );
    let _ = done(world, PRODUCER);
    let _ = done(world, REPAIR);
    let (head, tree) = world.branch(b"main");
    assert_eq!(head, merge);
    let source = utf8(tree.get(b"src/main.rs".as_slice()).expect("widget source landed"));
    assert!(source.contains("println!(\"fruit,count\")"));
    assert_eq!(source, expected, "the walk's whole produced source landed");
    assert_eq!(tree.get(b"changes/2.md".as_slice()).expect("repair's note").as_slice(), NOTE.as_bytes());

    check_result(world, repaired, merge, &scripts);
    let TaskResult::Report { words: report } = done(world, GOAL) else { panic!("goal reports") };
    let report = utf8(report);
    let chat = chat_text(world, CHAT);
    let last = chat.lines().next_back().expect("chat's last reply");
    assert!(last.contains(words(person, "last_word")));
    for citation in [&format!("PR #{PULL}"), &hex(merge)[..12]] {
        assert!(report.contains(citation), "goal cites {citation}: {report}");
        assert!(last.contains(citation), "chat cites {citation}: {last}");
    }
    assert!(told(query(world, CHAT, 4)).contains(report), "chat hears the goal's landing report");
    assert!(!last.contains("tests passed"), "no unseen test claimed");
}

/// The oracle reads the walk's recorded values, not the replay's takes.
fn recorded_scripts() -> Value {
    let takes: Value = serde_json::from_slice(TAKES).expect("walk takes");
    let mut bytes = utf8(TAPE).to_owned();
    for take in takes.as_array().expect("takes list") {
        let name = take["name"].as_str().expect("take name");
        let value = take["value"].as_str().expect("recorded value");
        let quoted = serde_json::to_string(value).expect("quoted value");
        let escaped = quoted.strip_prefix('"').and_then(|s| s.strip_suffix('"')).expect("string content");
        bytes = bytes.replace(&format!("\"{{{{#{name}}}}}\""), value).replace(&format!("{{{{{name}}}}}"), escaped);
    }
    let mut scripts: Value = serde_json::from_str(&bytes).expect("recorded scripts");
    for script in scripts["cued"].as_array_mut().expect("cued scripts") {
        for turn in script["turns"].as_array_mut().expect("turns") {
            turn["lines"].as_array_mut().expect("lines").retain(|line| line.get("take").is_none());
        }
    }
    scripts
}

fn check_result(world: &mut World, repaired: u64, merge: u64, scripts: &Value) {
    let change = world.task_overview(CHANGE);
    assert!(matches!(change.chip.phase, PhaseLine::Ended));
    let Some(Ending::Done(ResultValue::Change { kind: 1, part, words })) = &change.result else {
        panic!("landed result: {:?}", change.result)
    };
    let TaskResult::Change { kind: 1, resource: PULL, .. } = done(world, CHANGE) else {
        panic!("landed pull resource")
    };
    assert_eq!(part.key, tasks::PartKey::Number(PULL));
    assert!(!words.cut, "the task document retains the whole bounded result");
    let words = utf8(&words.bytes);
    // The 512-byte result contract reserves 59 bytes for its change envelope.
    assert_eq!(words.len(), 453, "the ASCII evidence fills the words allowance");
    let lines: Vec<_> = words.lines().collect();
    assert_eq!(lines[0], hex(merge));
    assert_eq!(
        lines[1],
        format!("pull request #{PULL} of playground/widget on {HOST}: http://{HOST}/playground/widget/pulls/{PULL}")
    );
    assert_eq!(lines[2], format!("Landed: pull request #{PULL} merged into main as {}.", &hex(merge)[..12]));
    let producer =
        scripts["cued"][2]["turns"][3]["lines"][0]["arguments"]["fields"]["body"].as_str().expect("producer's report");
    let repair =
        scripts["cued"][3]["turns"][2]["lines"][0]["arguments"]["fields"]["body"].as_str().expect("repair's report");
    let head = hex(repaired);
    let line = format!("successful repair task {REPAIR}: {repair}, then CI passed at {head}");
    assert_eq!(lines[3], "repairs: 1: CI failed: / changes (pull_request)");
    assert!(lines[4].len() > line.len() - head.len() && line.starts_with(lines[4]), "repair line: {}", lines[4]);
    let whole = format!(
        "repairs: 1: CI failed: / changes (pull_request)\n\
         {line}\n\
         Producer report (historical; before repairs):\n{producer}\n\
         checks at {head}: / check (pull_request) passed; / changes (pull_request) passed\n\
         reviews: approved at {head}",
    );
    let evidence = words.splitn(4, '\n').nth(3).expect("evidence after the landing identity");
    let (kept, marker) = evidence.rsplit_once("\n[").expect("marked evidence truncation");
    assert!(whole.starts_with(kept), "retained evidence prefix: {words}");
    let omitted: usize = marker.strip_suffix(" bytes cut]\n").expect("terminal cut marker").parse().expect("cut count");
    assert_eq!(omitted, whole.len() - kept.len(), "exact omitted evidence bytes: {words}");
    assert!(told(query(world, GOAL, 7)).contains(words), "change's requester heard the whole result");
}

fn recorded_source(seed: &str, producer: &Value) -> String {
    let mut expected = seed.to_owned();
    for edit in producer[2]["lines"][0]["arguments"]["edits"].as_array().expect("walk edits") {
        let old = edit["old"].as_str().expect("old source");
        let new = edit["new"].as_str().expect("new source");
        assert_eq!(expected.matches(old).count(), 1);
        expected = expected.replacen(old, new, 1);
    }
    expected
}
