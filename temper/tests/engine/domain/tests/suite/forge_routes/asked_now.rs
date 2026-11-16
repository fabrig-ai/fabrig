//! Rulings reach later work (temper/docs/design/domain/forge/change/questions.md, 1,
//! What later work is told; gates.md, 1; repairs.md, A repair).
use super::questions::{call, make_change, next_producer, plan_world_with_gate, row};
use super::reviews::{finish_review, next_assignment, produce};
use super::*;
use temper_engine_domain_forge_change::{Stands, State};

const CHANNEL: Token = Token::new(7);
const TICKS: u32 = 200;
// The landing's target review and merge pass through the queue's window.
const LAND_TICKS: u32 = 1000;
const QUESTION: &[u8] = b"Question: keep both lines?\nKeep one or both? I recommend both.";
const ANSWER: &[u8] = b"keep both lines";
const AMENDED: &[u8] = b"Drop the file named in the original request.";

fn world() -> (World, engine::Assignment, u64, engine::Assignment, Box<[u8]>) {
    let (mut world, requester) = plan_world_with_gate(
        None,
        Gate { agent: true, ..Gate::default() },
        &StoryLimits {
            turn: true,
            batch: Some(4),
            landing_members: Some(4),
            spec_bytes: Some(4096),
            ..StoryLimits::default()
        },
        false,
    );
    let change = make_change(&mut world, &requester, 93, engine::ChangeQuestions::Up);
    let producer = next_producer(&mut world, change);
    let branch = row(&world, change).branch;
    (world, requester, change, producer, branch)
}

fn review(world: &mut World, reviewer: &engine::Assignment, code: u32) {
    finish_review(
        world,
        reviewer,
        tasks::TaskResult::Verdict {
            code,
            fields: Box::new([]),
            words: if code == 1 { b"approved".as_slice().into() } else { b"restore the other line".as_slice().into() },
        },
    );
}

fn repair_answer() -> (World, u64, engine::Assignment, u64) {
    let (mut world, requester, change, producer, branch) = world();
    produce(&mut world, &producer, &branch);
    let reviewer = next_assignment(&mut world, 3);
    review(&mut world, &reviewer, 2);
    let repair = next_assignment(&mut world, 4);
    world.producer_question(&repair, 101, QUESTION);
    for _ in 0..TICKS {
        world.tick();
        if row(&world, change).questions.open.first().is_some_and(|q| q.stands == Stands::Asked)
            && world.store.pending.is_empty()
        {
            break;
        }
    }
    assert_eq!(row(&world, change).questions.open[0].line.as_ref(), b"Question: keep both lines?");
    let parked =
        world.protocol.answer(CHANNEL, repair.task, repair.attempt, 0, tasks::End::Parked, engine::Parts::none());
    world.send(parked);
    let question = live_task(&world, change).questions[0].number;
    call(
        &mut world,
        &requester,
        102,
        engine::Tool::Message { target: change, form: engine::MessageForm::Answer { question }, words: ANSWER.into() },
    );
    let resumed = next_assignment(&mut world, 5);
    assert_eq!(resumed.task, repair.task);
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"both lines", 1)
        .expect("repair pushes");
    super::delegates::fixtures::answer_producer(&mut world, &resumed);
    let next_review = next_assignment(&mut world, 6);
    (world, change, next_review, repair.task)
}

fn assert_answer(world: &World, run: &engine::Assignment, repair: u64) {
    let words = core::str::from_utf8(&live_task(world, run.task).spec.words).expect("task words");
    let ruling = format!("Task {repair} asked: Question: keep both lines?\nAnswer: keep both lines");
    assert!(words.contains(&ruling), "{words}");
    assert!(words.ends_with("Its request, at spec revision 0:\nMake the agreed fix"), "{words}");
}

#[test]
fn a_repair_answer_reaches_the_next_review_and_approval_merges() {
    let (mut world, change, reviewer, repair) = repair_answer();
    assert_answer(&world, &reviewer, repair);
    // The scripted reviewer approves only after reading the ruling in its spec.
    review(&mut world, &reviewer, 1);
    for _ in 0..LAND_TICKS {
        world.tick();
        if world.assigned.len() > 7 {
            break;
        }
    }
    let landing_review = world
        .assigned
        .get(7)
        .unwrap_or_else(|| {
            let delegate = row(&world, change).delegate.expect("landing reviewer").0;
            panic!(
                "landing reviewer {} {:?}; assignments {}",
                delegate,
                live_task(&world, delegate),
                world.assigned.len()
            );
        })
        .clone();
    assert!(matches!(row(&world, change).change.state, State::Judging { .. }));
    assert_answer(&world, &landing_review, repair);
    review(&mut world, &landing_review, 1);
    for _ in 0..LAND_TICKS {
        world.tick();
        if matches!(row(&world, change).change.state, State::Landed { .. }) {
            break;
        }
    }
    assert!(matches!(row(&world, change).change.state, State::Landed { .. }), "{:?}", row(&world, change));
    assert_eq!(world.merges, 1);
}

#[test]
fn a_repair_answer_reaches_the_next_repairs_preservation_words() {
    let (mut world, _, reviewer, first_repair) = repair_answer();
    review(&mut world, &reviewer, 2);
    let repair = next_assignment(&mut world, 7);
    assert_answer(&world, &repair, first_repair);
    let words = core::str::from_utf8(&live_task(&world, repair.task).spec.words).expect("repair words");
    assert!(words.contains("unless the change's request as revised, or an answer given on it, says otherwise."));
}

#[test]
fn an_amended_request_reaches_the_next_review_and_repair() {
    let (mut world, requester, change, producer, branch) = world();
    // Amend after the first review was made; it is not changed in place.
    produce(&mut world, &producer, &branch);
    let first_review = next_assignment(&mut world, 3);
    let amendment = temper_engine_domain_world::fixtures::task_amendment(Some(AMENDED), None, b"Drop the file");
    call(
        &mut world,
        &requester,
        103,
        engine::Tool::Amend { keep_parameters: true, target: change, revision: 0, amendment },
    );
    assert_eq!(live_task(&world, change).revisions.spec, 1);
    review(&mut world, &first_review, 2);
    let repair = next_assignment(&mut world, 4);
    assert_amended(&world, &repair);
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"amended change", 1)
        .expect("amended repair pushes");
    super::delegates::fixtures::answer_producer(&mut world, &repair);
    let next_review = next_assignment(&mut world, 5);
    assert_amended(&world, &next_review);
}

fn assert_amended(world: &World, run: &engine::Assignment) {
    let words = core::str::from_utf8(&live_task(world, run.task).spec.words).expect("spec words");
    let expected = format!("Its request, at spec revision 1:\n{}", core::str::from_utf8(AMENDED).expect("amendment"));
    assert!(words.ends_with(&expected), "{words}");
}
