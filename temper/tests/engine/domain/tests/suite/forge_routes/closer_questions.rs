//! The goal answers its closer's live change (temper/docs/design/domain/forge/branches.md, 2).
use super::*;
use temper_engine_domain_forge_closer::relay::{FALLBACK, Stands};
const QUESTION: &[u8] = b"Two reds remain: known reds, or change the golden test's policy?";
const ANSWER: &[u8] = b"Use known reds.";

fn repairing_landing() -> (World, engine::Assignment, Box<[u8]>, u64, u64, engine::Assignment) {
    let (mut world, goal, branch) = goal_world_with_updates(false, true, None, true, true, STEPS, None, false, Some(Duration::from_secs(3600)), 3);
    add_change(&mut world, &goal, &branch, 93, b"First", b"done");
    let head = branch_tip(&mut world, &branch);
    status(&mut world, head, raw::Check::Pending);
    let closer = make_closer(&mut world, &goal, 94);
    let landing = wait_landing(&mut world, closer);
    let before = world.assigned.len();
    status(&mut world, head, raw::Check::Failed);
    let repair = reviews::next_assignment(&mut world, before);
    assert_eq!(live_task(&world, repair.task).requester, tasks::Party::Task(landing));
    assert_eq!(row(&world, landing).delegate, Some((repair.task, change::Delegate::Repair(change::Repair::Ci))));
    (world, goal, branch, closer, landing, repair)
}
fn passed_up(world: &mut World, goal: u64, closer: u64, landing: u64) -> u64 {
    for _ in 0..STEPS {
        if let Some(word) = live_task(world, goal).inbox.iter().find(|word| word.from == tasks::Party::Task(closer) && word.kind == tasks::MessageKind::Question) {
            let expected = format!("Change {landing}, the goal's landing, asks:\n\n{}", String::from_utf8_lossy(QUESTION));
            assert_eq!(word.words.as_ref(), expected.as_bytes());
            return word.number;
        }
        world.tick();
    }
    panic!("closer passes the question up: {:?}; errors {:?}", closer_row(world, closer), world.call_errors);
}
fn ask(world: &mut World, goal: u64, closer: u64, landing: u64, agent: &engine::Assignment) -> (u64, u64) {
    let number = world.producer_question(agent, 110, QUESTION);
    let turns = live_task(world, agent.task).turn;
    park_after_turn(world, agent, None, turns);
    let up = passed_up(world, goal, closer, landing);
    assert_eq!(closer_row(world, closer).expect("closer record").relay.open.as_ref().expect("open closer question").stands, Stands::Asked { question: up, answer: None });
    (number, up)
}
fn resumed(world: &mut World, agent: &engine::Assignment, number: u64) -> engine::Assignment {
    for _ in 0..STEPS {
        if let Some(run) = world.assigned.iter().find(|run| run.task == agent.task && run.attempt > agent.attempt) {
            assert!(run.messages.iter().any(|word| word.kind == tasks::MessageKind::Answer { question: number } && word.words.as_ref() == ANSWER));
            return run.clone();
        }
        world.tick();
    }
    panic!("answer resumes the same agent: errors {:?}", world.call_errors);
}
fn repair_asks(restart: bool) {
    let (mut world, goal, branch, closer, landing, repair) = repairing_landing();
    let (number, up) = ask(&mut world, goal.task, closer, landing, &repair);
    if restart {
        world.drain_cut();
        let kept = closer_row(&world, closer).expect("closer record").relay.clone();
        world.restart_same();
        world.until(Until::Ready);
        assert_eq!(closer_row(&world, closer).expect("closer record").relay, kept);
    }
    world.goal_answer(&goal, 111, closer, up, ANSWER);
    let repair = resumed(&mut world, &repair, number);
    assert!(closer_row(&world, closer).expect("closer record").relay.open.is_none());
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"repair", b"fixed", 1).expect("repair pushes");
    status(&mut world, head, raw::Check::Passed);
    finish_run(&mut world, &repair, tasks::TaskResult::Change {
        title: tasks::ChangeTitle::new(b"Repair".as_slice().into()), connector: 0, kind: 2,
        resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository), words: b"repaired".as_slice().into(),
    });
    wait_ended(&mut world, landing);
    assert!(matches!(row(&world, landing).change.state, change::State::Landed { .. }));
    wait_ended(&mut world, closer);
    world.close();
}
#[test]
fn a_landings_ci_repair_asks_the_goal_through_its_closer_and_goes_on() { repair_asks(false); }
#[test]
fn a_restart_keeps_the_closers_question_and_relays_its_answer() { repair_asks(true); }

#[test]
fn a_landings_resolution_asks_the_goal_through_its_closer_and_goes_on() {
    let (mut world, goal, branch) = goal_world_with_updates(false, true, None, true, true, STEPS, None, true, Some(Duration::from_secs(3600)), 3);
    add_change(&mut world, &goal, &branch, 93, b"file", b"done");
    let base = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"file", b"base moved", 1).expect("main conflicts with g");
    status(&mut world, base, raw::Check::Passed);
    let closer = make_closer(&mut world, &goal, 94);
    let landing = wait_landing(&mut world, closer);
    let resolution = producer_for(&mut world, landing);
    assert_eq!(row(&world, landing).delegate, Some((resolution.task, change::Delegate::Resolve { base: translate::commit(base) })));
    let (number, up) = ask(&mut world, goal.task, closer, landing, &resolution);
    world.goal_answer(&goal, 111, closer, up, ANSWER);
    let resolution = resumed(&mut world, &resolution, number);
    let previous = world.fake.branch(b"org/repo", &branch).expect("g head");
    let resolved = fake::merge_commit(&mut world.fake, &world.fake_env.limits, previous, base,
        Box::new([raw::File { path: b"file".as_slice().into(), content: b"resolved".as_slice().into() }]), b"Resolve conflict").expect("worker resolves");
    assert_eq!(world.external(2, raw::Op::Git(raw::Git::Push { branch: branch.clone(), commit: resolved, expected: Some(previous) })), raw::Answer::Pushed(raw::Pushed::Pushed));
    status(&mut world, resolved, raw::Check::Passed);
    finish_run(&mut world, &resolution, tasks::TaskResult::Change {
        title: tasks::ChangeTitle::new(b"Resolve".as_slice().into()), connector: 0, kind: 2,
        resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository), words: b"resolved".as_slice().into(),
    });
    wait_ended(&mut world, landing);
    assert!(matches!(row(&world, landing).change.state, change::State::Landed { .. }));
    wait_ended(&mut world, closer);
    world.close();
}
#[test]
fn the_status_settlements_change_has_questions_up() {
    let (mut world, goal, _) = goal_world(true, true);
    let closer = make_closer(&mut world, &goal, 94);
    let job = close_job(&mut world);
    let tasks::Party::Task(change) = live_task(&world, job.run.raw()).requester else { panic!("close change") };
    assert!(closer_row(&world, closer).expect("closer record").close_changes.contains(&change));
    assert_eq!(row(&world, change).questions.route, change::Route::Up);
    world.close();
}
#[test]
fn an_ended_landing_drops_the_late_answer_with_one_notice() {
    let (mut world, goal, _, closer, landing, repair) = repairing_landing();
    let (_, up) = ask(&mut world, goal.task, closer, landing, &repair);
    cancel_landing(&mut world, &goal, landing);
    world.goal_answer(&goal, 111, closer, up, ANSWER);
    let expected = format!("Change {landing} ended; your answer to question {up} goes no further: {}", String::from_utf8_lossy(QUESTION));
    for _ in 0..STEPS {
        if closer_row(&world, closer).expect("closer record").relay.open.is_none() && live_task(&world, goal.task).inbox.iter().any(|word| word.words.as_ref() == expected.as_bytes()) { break; }
        world.tick();
    }
    assert!(closer_row(&world, closer).expect("closer record").relay.open.is_none());
    for _ in 0..20 { world.tick(); }
    let notices: Vec<_> = live_task(&world, goal.task).inbox.iter().filter(|word| word.from == tasks::Party::Task(closer) && word.words.as_ref() == expected.as_bytes()).collect();
    assert_eq!(notices.len(), 1);
    assert_eq!(notices[0].kind, tasks::MessageKind::Words);
    world.close();
}
#[test]
fn the_closer_answers_other_questions_and_passes_its_changes_notices() {
    let (mut world, goal, _, closer, landing, repair) = repairing_landing();
    let engine::CallAnswer::Sent { message: fallback } = world.scripted_call(&goal, 109, engine::Tool::Message {
        target: closer, form: engine::MessageForm::Question, words: b"Will you decide?".as_slice().into(),
    }) else { panic!("goal asks closer") };
    for _ in 0..STEPS {
        if live_task(&world, goal.task).inbox.iter().any(|word| word.kind == tasks::MessageKind::Answer { question: fallback }) { break; }
        world.tick();
    }
    assert!(live_task(&world, goal.task).inbox.iter().any(|word| word.from == tasks::Party::Task(closer) && word.kind == tasks::MessageKind::Answer { question: fallback } && word.words.as_ref() == FALLBACK));
    let (number, up) = ask(&mut world, goal.task, closer, landing, &repair);
    let answer = world.scripted_call(&goal, 112, engine::Tool::Cancel { target: repair.task, reason: b"stop the asker".as_slice().into() });
    assert!(matches!(answer, engine::CallAnswer::Controlled));
    wait_ended(&mut world, repair.task);
    wait_held(&mut world, landing, goal.task);
    world.goal_answer(&goal, 113, closer, up, ANSWER);
    let revision = live_task(&world, landing).revisions.hold;
    let answer = world.scripted_call(&goal, 114, engine::Tool::Release { target: landing, revision, again: None });
    assert!(matches!(answer, engine::CallAnswer::Revised { .. }), "{answer:?}");
    let mut expected = format!("Change {landing}: Task {} ended; your answer to its question {number} goes no further: {}", repair.task, String::from_utf8_lossy(QUESTION));
    let bound = usize::try_from(world.env.limits.tasks.message_bytes).expect("message bound");
    assert!(expected.len() > bound, "the prefixed notice exercises cutting");
    expected.truncate(bound - "…".len());
    expected.push('…');
    for _ in 0..STEPS {
        if live_task(&world, goal.task).inbox.iter().any(|word| word.words.as_ref() == expected.as_bytes()) { break; }
        world.tick();
    }
    assert!(live_task(&world, goal.task).inbox.iter().any(|word| word.from == tasks::Party::Task(closer) && word.kind == tasks::MessageKind::Words && word.words.as_ref() == expected.as_bytes()));
    world.close();
}
