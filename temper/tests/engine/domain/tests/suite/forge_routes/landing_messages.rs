//! The root delegates the producer's member messages, with their trailers only on squashes.
use super::super::goal_landing::{ended, producing_into_goal_with_message, row, status};
use super::*;
use skein_forge::api::Read;
use temper_protocol_worker_wire::v1::Record as WorkerRecord;

const TITLE: &[u8] = b"Add the squash trailers";
const WORDS: &[u8] = b"Squashes each change.\n\nMore text.";
const WORDS_WITH_TRAILER: &[u8] = b"Squashes each change.\n\nMore text.\n\nLook-request: squash-trailers";
const HEAD_MESSAGE: &[u8] = b"Keep the look request\n\nLook-request: usage-model-week\nLook-landed: usage-model-week";

const PAGE_ROWS: u32 = 8;

fn push_long_history(world: &mut World, producer: &engine::Assignment, branch: &[u8], mut head: u64) -> u64 {
    for byte in 0..u8::try_from(2 * PAGE_ROWS).expect("small history") {
        head = world.worker_commit(head, b"history", &[byte]);
    }
    let files = world
        .fake
        .object(head)
        .expect("history tip")
        .tree
        .iter()
        .map(|(path, content)| raw::File { path: path.clone(), content: content.clone() })
        .collect();
    let head = fake::commit_tree(&mut world.fake, &world.fake_env.limits, head, None, files, HEAD_MESSAGE)
        .expect("head with trailers");
    world.run_push(producer, branch, head, 1);
    status(world, head, raw::Check::Passed);
    head
}

struct Delegated {
    world: World,
    request: serde_json::Value,
    change: u64,
    goal: u64,
}

fn delegate(into_main: bool, words: &[u8], head_message: &[u8], long_history: bool) -> Delegated {
    let world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        // The two descriptions and the goal-branch upstream need more than the
        // world's 512-byte empty-description prepare carrier.
        StoryLimits {
            turn: true,
            spec_bytes: Some(1024),
            forge_rows: long_history.then_some(PAGE_ROWS),
            forge_commits: long_history.then_some(48),
            ..StoryLimits::default()
        },
    );
    let (mut world, goal, producer, change) = if into_main {
        let (mut world, goal, producer, _) = change_world_in(world, Change { tracked: true, ..Change::default() });
        let tasks::Party::Task(change) = live_task(&world, producer.task).requester else {
            panic!("producer's change")
        };
        push_change(&mut world, &producer, b"org/repo");
        (world, goal, producer, change)
    } else {
        let mut world = world;
        world.adopt();
        world.send(hello_prepare(2));
        let ask = super::super::goal_branch::goal_request(&world, true);
        let goal = assigned_chat(&mut world, 92, ask);
        let branch = format!("temper/{}/g{}", goal.task, goal.task);
        world.until(Until::Ready);
        let (change, producer) =
            producing_into_goal_with_message(&mut world, &goal, branch.as_bytes(), 93, b"feature", head_message);
        assert_eq!(row(&world, change).base.as_ref(), branch.as_bytes());
        (world, goal, producer, change)
    };
    world.send(hello_prepare(2));
    world.manual_prepare_jobs();
    let branch = producer.workspace.repositories[0].push.clone().expect("producer's push branch");
    let mut head = world.fake.branch(b"org/repo", &branch).expect("delivered head");
    if long_history {
        head = push_long_history(&mut world, &producer, &branch, head);
    }
    let base = world.fake.branch(b"org/repo", &row(&world, change).base).expect("train base");
    let head = translate::commit(head);
    let base = translate::commit(base);
    world.forge_reads = Some(Vec::new());
    let answer = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(TITLE.into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: words.into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(answer);
    for _ in 0..600 {
        world.tick();
        if !world.jobs.is_empty() {
            break;
        }
    }
    assert_eq!(world.jobs.len(), 1, "ready member delegates one prepare: {:?}", landing_rows(&world));
    let job = &world.jobs[0];
    assert_eq!(job.kind, jig_domain_link::JobKind(1));
    let spec = temper_protocol_worker_wire::v1::JobSpec::decode(
        &temper_protocol_worker_wire::v1::CEILINGS,
        &mut skein_lib::Reader::new(&job.spec),
    )
    .expect("delegated prepare spec");
    assert_eq!(spec.command.as_ref(), b"");
    let request: serde_json::Value = serde_json::from_slice(spec.request.as_ref()).expect("prepare request JSON");
    let reads = world.forge_reads.take().expect("recorded reads");
    let prefix = request["members"][0]["prefix"].as_str().expect("prepare branch");
    assert_one_containment_page(&reads, prefix.as_bytes(), base, head);
    assert_eq!(
        request["members"][0]["head"],
        String::from_utf8(engine::commit_text(&head).into_vec()).expect("head text")
    );
    Delegated { world, request, change, goal: goal.task }
}

fn assert_one_containment_page(reads: &[(Token, Read)], prefix: &[u8], base: api::Commit, head: api::Commit) {
    // The containment read follows the round's check for an existing prefix.
    let page = |read: &Read| match read {
        Read::Commits { before, after, page } if *before == base && *after == head => Some(*page),
        _ => None,
    };
    let mut gathers = 0;
    for (index, (_, read)) in reads.iter().enumerate() {
        if !matches!(read, Read::Branch { branch } if branch.as_ref() == prefix) {
            continue;
        }
        let pages = reads[index + 1..].iter().map_while(|(_, read)| page(read)).collect::<Vec<_>>();
        if !pages.is_empty() {
            assert_eq!(pages, [1], "one containment read in each gather");
            gathers += 1;
        }
    }
    assert!(gathers > 0, "the round gathered containment: {reads:?}");
    assert!(!reads.iter().any(|(_, read)| page(read).is_some_and(|page| page > 1)));
}

fn assert_messages(delegated: &Delegated, into_main: bool, trailers: &str, head_trailers: bool) {
    let Delegated { world, request, change, goal } = delegated;
    let member = &request["members"][0];
    let head = member["head"].as_str().expect("member head");
    let pull = row(world, *change).pull.expect("member's pull request");
    assert_eq!(row(world, *change).goal, Some(*goal));
    assert_eq!(row(world, *change).milestone, None, "a plain goal has no milestone");
    assert_eq!(row(world, *change).title.as_ref(), TITLE);
    let intro = "Add the squash trailers\n\nSquashes each change.\n\n";
    let temper = format!("Change: {change}\nPull-request: {pull}\nHead: {head}");
    let goal_line = format!("\nGoal: {goal}");
    let merge_goal = if into_main { goal_line.as_str() } else { "" };
    let message = format!("{intro}{temper}{merge_goal}");
    let squash = if trailers.is_empty() {
        format!("{intro}{temper}{goal_line}")
    } else {
        format!("{intro}{trailers}\n{temper}{goal_line}")
    };
    assert_eq!(request["combine"], if into_main { "merge" } else { "squash" });
    assert_eq!(member["number"], 1_u64);
    assert_eq!(member["squash"], squash);
    assert_eq!(member["message"], message);
    if head_trailers {
        assert_eq!(member["head_trailers"], true);
    } else {
        assert_eq!(member.get("head_trailers"), None);
    }
}

#[test]
fn a_goal_branch_prepare_uses_the_producers_words_without_a_merge_goal_trailer() {
    let delegated = delegate(false, WORDS_WITH_TRAILER, HEAD_MESSAGE, false);
    assert_messages(&delegated, false, "Look-request: squash-trailers", false);
}

#[test]
fn a_main_prepare_keeps_the_members_goal_in_both_messages() {
    let delegated = delegate(true, WORDS_WITH_TRAILER, b"", false);
    assert_messages(&delegated, true, "Look-request: squash-trailers", false);
}

#[test]
fn a_goal_branch_squash_keeps_the_pushed_heads_trailers_when_the_words_have_none() {
    let delegated = delegate(false, WORDS, HEAD_MESSAGE, false);
    assert_messages(&delegated, false, "Look-request: usage-model-week\nLook-landed: usage-model-week", true);
}

#[test]
fn a_goal_branch_squash_requests_the_head_trailers_past_the_first_page() {
    let delegated = delegate(false, WORDS, b"", true);
    assert_messages(&delegated, false, "", true);
}

#[test]
fn a_main_merge_with_a_long_history_never_requests_head_trailers() {
    let delegated = delegate(true, WORDS, b"", true);
    assert_messages(&delegated, true, "", false);
}

#[test]
fn an_earlier_workers_squash_of_a_flagged_member_still_lands() {
    const COMBINED: &[u8] = br#"{"combined":[{"member":1,"by":"squash"}]}"#;
    let Delegated { mut world, request, change, .. } = delegate(false, WORDS, HEAD_MESSAGE, false);
    let member = &request["members"][0];
    assert_eq!(member["head_trailers"], true);
    let prefix = member["prefix"].as_str().expect("prepare branch").as_bytes();
    let base = row(&world, change).base.clone();
    let base_tip = world.fake.branch(b"org/repo", &base).expect("train base");

    // The scripted worker uses the request's squash verbatim and ignores the flag.
    let result = world.push_prepare();
    let tip = world.fake.branch(b"org/repo", prefix).expect("the earlier worker pushed a prefix");
    let squash = world.fake.object(tip).expect("pushed squash");
    assert_eq!(squash.parent, Some(base_tip));
    assert_eq!(squash.merge_parent, None);
    assert_eq!(squash.message.as_ref(), member["squash"].as_str().expect("squash message").as_bytes());
    let job = world.jobs.remove(0);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&result).expect("prepare result"),
        serde_json::from_slice::<serde_json::Value>(COMBINED).expect("combined member")
    );
    super::super::catch_up::finish_job(&mut world, &job, COMBINED);
    for _ in 0..600 {
        if ended(&world, change).is_some() {
            break;
        }
        world.tick();
    }
    assert!(matches!(
        ended(&world, change).expect("change ends").phase,
        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Change { .. }))
    ));
    let landing =
        landing_rows(&world).into_iter().find(|row| row.landing.members[0].change == change).expect("landing");
    assert_eq!(landing.landing.members[0].landed, Some(translate::commit(tip)));
    assert_eq!(landing.landing.round.combined, Some(forge_top::LandingCombine::Squash));
    assert_eq!(world.fake.branch(b"org/repo", &base), Some(tip));
    world.close();
}
