//! The root delegates the producer's member messages, with their trailers only on squashes.
use super::super::goal_landing::{producing_into_goal_with_message, row};
use super::*;
use temper_protocol_worker_wire::v1::Record as WorkerRecord;

const TITLE: &[u8] = b"Add the squash trailers";
const WORDS: &[u8] = b"Squashes each change.\n\nMore text.";
const WORDS_WITH_TRAILER: &[u8] = b"Squashes each change.\n\nMore text.\n\nLook-request: squash-trailers";
const HEAD_MESSAGE: &[u8] = b"Keep the look request\n\nLook-request: usage-model-week\nLook-landed: usage-model-week";

fn delegated_messages(into_main: bool, words: &[u8], head_message: &[u8], trailers: &str) {
    let world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        // The two descriptions and the goal-branch upstream need more than the
        // world's 512-byte empty-description prepare carrier.
        StoryLimits { turn: true, spec_bytes: Some(1024), ..StoryLimits::default() },
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
    let raw::Answer::Commit(head) = world.external(1, raw::Op::Read(raw::Read::Branch { branch })) else {
        panic!("delivered head")
    };
    let head = String::from_utf8(engine::commit_text(&translate::commit(head)).into_vec()).expect("head text");
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
    let pull = row(&world, change).pull.expect("member's pull request");
    assert_eq!(row(&world, change).goal, Some(goal.task));
    assert_eq!(row(&world, change).milestone, None, "a plain goal has no milestone");
    assert_eq!(row(&world, change).title.as_ref(), TITLE);
    let intro = "Add the squash trailers\n\nSquashes each change.\n\n";
    let temper = format!("Change: {change}\nPull-request: {pull}\nHead: {head}");
    let goal_line = format!("\nGoal: {}", goal.task);
    let merge_goal = if into_main { goal_line.as_str() } else { "" };
    let message = format!("{intro}{temper}{merge_goal}");
    let squash = format!("{intro}{trailers}\n{temper}{goal_line}");
    assert_eq!(request["combine"], if into_main { "merge" } else { "squash" });
    assert_eq!(request["members"][0]["number"], 1_u64);
    assert_eq!(request["members"][0]["head"], head);
    assert_eq!(request["members"][0]["squash"], squash);
    assert_eq!(request["members"][0]["message"], message);
}

#[test]
fn a_goal_branch_prepare_uses_the_producers_words_without_a_merge_goal_trailer() {
    delegated_messages(false, WORDS_WITH_TRAILER, b"", "Look-request: squash-trailers");
}

#[test]
fn a_main_prepare_keeps_the_members_goal_in_both_messages() {
    delegated_messages(true, WORDS_WITH_TRAILER, b"", "Look-request: squash-trailers");
}

#[test]
fn a_goal_branch_squash_keeps_the_pushed_heads_trailers_when_the_words_have_none() {
    delegated_messages(false, WORDS, HEAD_MESSAGE, "Look-request: usage-model-week\nLook-landed: usage-model-week");
}
