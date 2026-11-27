//! The goal answers its closer's live change (temper/docs/design/domain/forge/branches.md, 2).
use super::*;
use temper_engine_domain_forge_closer::relay::{FALLBACK, Held, Stands};
const QUESTION: &[u8] = b"Two reds remain: known reds, or change the golden test's policy?";
const ANSWER: &[u8] = b"Use known reds.";

fn repairing_landing() -> (World, engine::Assignment, Box<[u8]>, u64, u64, engine::Assignment) {
    let (mut world, goal, branch, closer, landing) = landing_with_settlement(false);
    let repair = start_repair(&mut world, &branch, landing);
    let words = core::str::from_utf8(&live_task(&world, repair.task).spec.words).expect("repair context");
    assert!(
        words.contains(&format!("This change's current spec and rulings: task {landing} at spec revision 0.")),
        "{words}"
    );
    let spec = core::str::from_utf8(&live_task(&world, landing).spec.words).expect("landing spec");
    assert!(words.ends_with(&format!("Its current spec, at revision 0:\n{spec}")), "{words}");
    (world, goal, branch, closer, landing, repair)
}

fn landing_with_settlement(settle: bool) -> (World, engine::Assignment, Box<[u8]>, u64, u64) {
    let (mut world, goal, branch) =
        goal_world_with_updates(settle, true, None, true, true, STEPS, None, false, Some(Duration::from_secs(3600)), 3);
    add_change(&mut world, &goal, &branch, 93, b"First", b"done");
    if !settle {
        pending_head(&mut world, &branch);
    }
    let closer = make_closer(&mut world, &goal, 94);
    if settle {
        let job = close_job(&mut world);
        let tasks::Party::Task(change) = live_task(&world, job.run.raw()).requester else { panic!("close change") };
        finish_close(&mut world, change, &branch, &job);
        wait_ended(&mut world, change);
        pending_head(&mut world, &branch);
    }
    let landing = wait_landing(&mut world, closer);
    (world, goal, branch, closer, landing)
}

fn pending_head(world: &mut World, branch: &[u8]) {
    let head = branch_tip(world, branch);
    status(world, head, raw::Check::Pending);
}

fn start_repair(world: &mut World, branch: &[u8], landing: u64) -> engine::Assignment {
    let head = branch_tip(world, branch);
    status(world, head, raw::Check::Failed);
    let repair = producer_for(world, landing);
    assert_eq!(live_task(world, repair.task).requester, tasks::Party::Task(landing));
    assert_eq!(row(world, landing).delegate, Some((repair.task, change::Delegate::Repair(change::Repair::Ci))));
    repair
}
fn passed_up(world: &mut World, goal: u64, closer: u64, landing: u64) -> u64 {
    for _ in 0..STEPS {
        if let Some(word) = live_task(world, goal)
            .inbox
            .iter()
            .find(|word| word.from == tasks::Party::Task(closer) && word.kind == tasks::MessageKind::Question)
        {
            let expected =
                format!("Change {landing}, the goal's landing, asks:\n\n{}", String::from_utf8_lossy(QUESTION));
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
    assert_eq!(
        closer_row(world, closer).expect("closer record").relay.open.as_ref().expect("open closer question").stands,
        Stands::Asked { question: up, answer: None, held: None }
    );
    (number, up)
}
fn resumed(world: &mut World, agent: &engine::Assignment, number: u64) -> engine::Assignment {
    for _ in 0..STEPS {
        if let Some(run) = world.assigned.iter().find(|run| run.task == agent.task && run.attempt > agent.attempt) {
            assert!(
                run.messages.iter().any(|word| word.kind == tasks::MessageKind::Answer { question: number }
                    && word.words.as_ref() == ANSWER)
            );
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
    assert!(!live_task(&world, goal.task).inbox.iter().any(|word| {
        word.from == tasks::Party::Task(closer) && word.words.starts_with(b"Your answer to question ")
    }));
    finish_repair(&mut world, &branch, &repair, raw::Check::Passed);
    wait_ended(&mut world, landing);
    assert!(matches!(row(&world, landing).change.state, change::State::Landed { .. }));
    wait_ended(&mut world, closer);
    world.close();
}
#[test]
fn a_closer_tells_its_goal_once_when_it_holds_an_answer_across_a_restart() {
    let (mut world, goal, _, closer, landing, repair) = repairing_landing();
    let (_, up) = ask(&mut world, goal.task, closer, landing, &repair);
    let batch = world.env.limits.tasks.batch;
    world.drain_cut();
    world.env.limits.tasks.batch = 1;
    restore_answer_with_earlier_words(&mut world, goal.task, closer, up, landing);
    world.restart_same();
    for _ in 0..STEPS {
        world.tick();
        if held_answer(&world, closer).is_some() {
            break;
        }
    }
    let held = held_answer(&world, closer).expect("the first step retains the answer");
    assert!(held.notified);
    let utc = held.since.utc_second();
    let time = core::str::from_utf8(&utc).expect("UTC second");
    let held_words =
        format!("Your answer to question {up} is held until this closer has message room; held since {time}.");
    let delivered_words = format!("Your answer to question {up}, held since {time}, is delivered to task {landing}.");
    let notices = answer_notices(&world, goal.task, closer);
    let [notice] = notices.as_slice() else { panic!("one held notice: {notices:?}") };
    assert_eq!(notice.words.as_ref(), held_words.as_bytes());
    assert_eq!((notice.at, &notice.kind), (held.since, &tasks::MessageKind::Words));
    assert!(live_task(&world, goal.task).notice_credits.is_empty(), "the held notice spends its kept room");

    world.env.limits.tasks.batch = batch;
    world.crash_restart(1);
    for _ in 0..STEPS {
        world.tick();
        if answer_notices(&world, goal.task, closer).len() >= 2 {
            break;
        }
    }
    let notices = answer_notices(&world, goal.task, closer);
    let [first, last] = notices.as_slice() else { panic!("held and delivered once each: {notices:?}") };
    assert_eq!(first.words.as_ref(), held_words.as_bytes());
    assert_eq!(last.words.as_ref(), delivered_words.as_bytes());
    assert_eq!(last.kind, tasks::MessageKind::Words);
    let answer = live_task(&world, landing)
        .inbox
        .iter()
        .find(|word| word.from == tasks::Party::Task(closer))
        .expect("the answer and delivery words commit together");
    assert_eq!(answer.words.as_ref(), ANSWER);
    assert!(matches!(answer.kind, tasks::MessageKind::Answer { .. }));
    assert_eq!(answer.at, last.at);
    assert!(closer_row(&world, closer).expect("closer record").relay.open.is_none());
    world.close();
}

fn held_answer(world: &World, closer: u64) -> Option<Held> {
    let Stands::Asked { held, .. } = closer_row(world, closer)?.relay.open.as_ref()?.stands else { return None };
    held
}

fn answer_notices(world: &World, goal: u64, closer: u64) -> Vec<&tasks::Word> {
    super::super::questions::replies(world, goal, closer)
        .into_iter()
        .filter(|word| word.words.starts_with(b"Your answer to question "))
        .collect()
}

fn restore_answer_with_earlier_words(world: &mut World, goal: u64, closer: u64, question: u64, landing: u64) {
    // The closer takes this whole prefix in one step. The earlier words spend
    // the batch's only slot; jig has spent the answer credit but kept the notice room.
    let mut task = Box::new(live_task(world, closer).clone());
    let mut header = world.store.header();
    let mut inbox = task.inbox.into_vec();
    for (from, kind, words) in [
        (landing, tasks::MessageKind::Words, b"Earlier words from the landing.".as_slice()),
        (goal, tasks::MessageKind::Answer { question }, ANSWER),
    ] {
        header.messages += 1;
        inbox.push(tasks::Word {
            number: header.messages,
            from: tasks::Party::Task(from),
            kind,
            words: words.into(),
            at: world.env.wall,
            hits: 1,
            eligible: true,
        });
    }
    task.inbox = inbox.into_boxed_slice();
    task.last_message = header.messages;
    task.questions = Box::new([]);
    task.phase = tasks::Phase::Active(tasks::Active::Due);
    world.store.seed(
        Box::new([
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task))),
            Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(header))),
        ]),
        Box::new([]),
    );
}

#[test]
fn a_closers_landing_amends_its_live_ci_repair_without_a_producer() {
    use super::super::delegate_amendment;
    let (mut world, goal, branch, closer, landing, repair) = repairing_landing();
    assert!(
        !world.assigned.iter().any(|run| {
            run.task != repair.task
                && world.root.tasks().task(run.task).is_some_and(|task| task.requester == tasks::Party::Task(landing))
        }),
        "the closer's landing has no producer"
    );
    let original = live_task(&world, repair.task).spec.clone();
    let revision = live_task(&world, repair.task).revisions.spec;
    delegate_amendment::amend(&mut world, &goal, landing);
    let message = delegate_amendment::assert_amended(&world, landing, repair.task, &original, revision);
    let turns = live_task(&world, repair.task).turn;
    park_after_turn(&mut world, &repair, None, turns);
    // Preparing the next brief can wait for the existing forge rate window.
    let window = world.env.limits.forge.client.window;
    super::super::stall_hold::jump(&mut world, window);
    let resumed = delegate_amendment::next_attempt(&mut world, &repair);
    delegate_amendment::assert_brief(&world, &resumed, &message);
    finish_repair(&mut world, &branch, &resumed, raw::Check::Passed);
    wait_ended(&mut world, landing);
    wait_ended(&mut world, closer);
    world.close();
}

#[test]
fn a_landings_ci_repair_asks_the_goal_through_its_closer_and_goes_on() {
    repair_asks(false);
}
#[test]
fn a_landing_repair_asks_while_the_closer_settles_status() {
    use super::super::stall_hold;
    const NEW_WORK: u64 = 95;
    const WAIT_FOR_WORK: u64 = 96;
    const ANSWER_CALL: u64 = 111;
    let (mut world, goal, branch, closer, landing) = landing_with_settlement(true);
    let first_repair = start_repair(&mut world, &branch, landing);
    let window = world.env.limits.forge.client.window;
    stall_hold::jump(&mut world, window);
    let (work, producer) = new_work(&mut world, &goal, NEW_WORK);
    closer_waits_for(&mut world, &goal, closer, work, WAIT_FOR_WORK);
    // The first repair releases the writer slot for the merge. New work moves
    // the head, so the repair that asks is the one at the new head.
    finish_repair(&mut world, &branch, &first_repair, raw::Check::Pending);
    wait_ended(&mut world, first_repair.task);
    produce(&mut world, work, &branch, &producer, b"New work", b"done");
    stall_hold::jump(&mut world, window);
    let job = close_job(&mut world);
    let tasks::Party::Task(settlement) = live_task(&world, job.run.raw()).requester else { panic!("close change") };
    let repair = start_repair(&mut world, &branch, landing);
    // The close's hooks read shares the forge rate window with the landing's
    // CI read. Let that observation settle before the restart oracle's cut.
    stall_hold::jump(&mut world, window);
    let (number, up) = ask(&mut world, goal.task, closer, landing, &repair);
    world.goal_answer(&goal, ANSWER_CALL, closer, up, ANSWER);
    let repair = resumed(&mut world, &repair, number);
    assert!(matches!(row(&world, work).change.state, change::State::Landed { .. }));
    assert_eq!(
        closer_row(&world, closer).expect("closer record").closer.running,
        Some(closer::Running { step: closer::Step::Close, change: settlement })
    );
    assert!(ended(&world, settlement).is_none(), "status settlement is still live");
    let fallback_reached = world.store.rows.values().any(|record| {
        let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Activity(line))) = record else {
            return false;
        };
        (line.task == landing || line.task == repair.task)
            && matches!(&line.line, tasks::Line::Words { kind: tasks::MessageKind::Answer { .. }, words, .. }
                if words.as_ref() == FALLBACK)
    });
    assert!(!fallback_reached, "neither the landing nor the repair got the canned answer");
    world.close();
}

fn finish_repair(world: &mut World, branch: &[u8], repair: &engine::Assignment, check: raw::Check) {
    let head = world.run_push_edit(repair, branch, b"repair", b"fixed", 1);
    status(world, head, check);
    finish_run(
        world,
        repair,
        tasks::TaskResult::Change {
            title: tasks::ChangeTitle::new(b"Repair".as_slice().into()),
            connector: 0,
            kind: 2,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
            words: b"repaired".as_slice().into(),
        },
    );
}

#[test]
fn the_closer_refuses_answers_that_do_not_name_its_question() {
    use super::super::questions::call;
    let (mut world, goal, _, closer, landing, repair) = repairing_landing();
    call(
        &mut world,
        &goal,
        109,
        engine::Tool::Message {
            target: closer,
            form: engine::MessageForm::Answer { question: 1 },
            words: ANSWER.into(),
        },
    );
    let expected =
        format!("Task {closer} has no question open to you; read_task on task {closer} shows its questions.");
    assert_eq!(world.call_errors.last().map(|error| error.text.as_ref()), Some(expected.as_bytes()));
    let (number, up) = ask(&mut world, goal.task, closer, landing, &repair);
    let sent = world.answers.len();
    let mut expected = format!(
        "Question {number} is not open at task {closer}; its question to you is {up}: {}. Answer it: message target={closer}, form=answer, question={up}.",
        core::str::from_utf8(QUESTION).unwrap()
    );
    let bound = usize::try_from(world.env.limits.tasks.message_bytes).unwrap();
    if expected.len() > bound {
        expected.replace_range(bound - "…".len().., "…");
    }
    call(
        &mut world,
        &goal,
        111,
        engine::Tool::Message {
            target: closer,
            form: engine::MessageForm::Answer { question: number },
            words: ANSWER.into(),
        },
    );
    assert_eq!(world.call_errors.last().map(|error| error.text.as_ref()), Some(expected.as_bytes()));
    assert_eq!(world.answers.len(), sent);
    assert_eq!(
        closer_row(&world, closer).unwrap().relay.open.as_ref().unwrap().stands,
        Stands::Asked { question: up, answer: None, held: None }
    );
    world.goal_answer(&goal, 112, closer, up, ANSWER);
    resumed(&mut world, &repair, number);
    world.close();
}

#[test]
fn a_restart_keeps_the_closers_question_and_relays_its_answer() {
    repair_asks(true);
}

#[test]
fn a_landings_resolution_asks_the_goal_through_its_closer_and_goes_on() {
    let (mut world, goal, branch) =
        goal_world_with_updates(false, true, None, true, true, STEPS, None, true, Some(Duration::from_secs(3600)), 3);
    add_change(&mut world, &goal, &branch, 93, b"file", b"done");
    let base = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"file", b"base moved", 1)
        .expect("main conflicts with g");
    status(&mut world, base, raw::Check::Passed);
    let closer = make_closer(&mut world, &goal, 94);
    let landing = wait_landing(&mut world, closer);
    let resolution = producer_for(&mut world, landing);
    assert_eq!(
        row(&world, landing).delegate,
        Some((resolution.task, change::Delegate::Resolve { base: translate::commit(base) }))
    );
    let (number, up) = ask(&mut world, goal.task, closer, landing, &resolution);
    world.goal_answer(&goal, 111, closer, up, ANSWER);
    let resolution = resumed(&mut world, &resolution, number);
    let previous = world.fake.branch(b"org/repo", &branch).expect("g head");
    let resolved = fake::merge_commit(
        &mut world.fake,
        &world.fake_env.limits,
        previous,
        base,
        Box::new([raw::File { path: b"file".as_slice().into(), content: b"resolved".as_slice().into() }]),
        b"Resolve conflict",
    )
    .expect("worker resolves");
    world.run_push(&resolution, &branch, resolved, 2);
    status(&mut world, resolved, raw::Check::Passed);
    finish_run(
        &mut world,
        &resolution,
        tasks::TaskResult::Change {
            title: tasks::ChangeTitle::new(b"Resolve".as_slice().into()),
            connector: 0,
            kind: 2,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
            words: b"resolved".as_slice().into(),
        },
    );
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
    let expected = format!(
        "Change {landing} ended; your answer to question {up} goes no further: {}",
        String::from_utf8_lossy(QUESTION)
    );
    for _ in 0..STEPS {
        if closer_row(&world, closer).expect("closer record").relay.open.is_none()
            && live_task(&world, goal.task).inbox.iter().any(|word| word.words.as_ref() == expected.as_bytes())
        {
            break;
        }
        world.tick();
    }
    assert!(closer_row(&world, closer).expect("closer record").relay.open.is_none());
    world.advance_ticks(20);
    let notices: Vec<_> = live_task(&world, goal.task)
        .inbox
        .iter()
        .filter(|word| word.from == tasks::Party::Task(closer) && word.words.as_ref() == expected.as_bytes())
        .collect();
    assert_eq!(notices.len(), 1);
    assert_eq!(notices[0].kind, tasks::MessageKind::Words);
    world.close();
}
#[test]
fn the_closer_answers_other_questions_and_passes_its_changes_notices() {
    let (mut world, goal, _, closer, landing, repair) = repairing_landing();
    let engine::CallAnswer::Sent { message: fallback } = world.scripted_call(
        &goal,
        109,
        engine::Tool::Message {
            target: closer,
            form: engine::MessageForm::Question,
            words: b"Will you decide?".as_slice().into(),
        },
    ) else {
        panic!("goal asks closer")
    };
    for _ in 0..STEPS {
        if live_task(&world, goal.task)
            .inbox
            .iter()
            .any(|word| word.kind == tasks::MessageKind::Answer { question: fallback })
        {
            break;
        }
        world.tick();
    }
    assert!(live_task(&world, goal.task).inbox.iter().any(|word| word.from == tasks::Party::Task(closer)
        && word.kind == tasks::MessageKind::Answer { question: fallback }
        && word.words.as_ref() == FALLBACK));
    let (number, up) = ask(&mut world, goal.task, closer, landing, &repair);
    let answer = world.scripted_call(
        &goal,
        112,
        engine::Tool::Cancel { target: repair.task, reason: b"stop the asker".as_slice().into() },
    );
    assert!(matches!(answer, engine::CallAnswer::Controlled));
    wait_ended(&mut world, repair.task);
    wait_held(&mut world, landing, goal.task);
    world.goal_answer(&goal, 113, closer, up, ANSWER);
    let revision = live_task(&world, landing).revisions.hold;
    let answer = world.scripted_call(&goal, 114, engine::Tool::Release { target: landing, revision, again: None });
    assert!(matches!(answer, engine::CallAnswer::Revised { .. }), "{answer:?}");
    let mut expected = format!(
        "Change {landing}: Task {} ended; your answer to its question {number} goes no further: {}",
        repair.task,
        String::from_utf8_lossy(QUESTION)
    );
    let bound = usize::try_from(world.env.limits.tasks.message_bytes).expect("message bound");
    assert!(expected.len() > bound, "the prefixed notice exercises cutting");
    expected.truncate(bound - "…".len());
    expected.push('…');
    for _ in 0..STEPS {
        if live_task(&world, goal.task).inbox.iter().any(|word| word.words.as_ref() == expected.as_bytes()) {
            break;
        }
        world.tick();
    }
    assert!(live_task(&world, goal.task).inbox.iter().any(|word| word.from == tasks::Party::Task(closer)
        && word.kind == tasks::MessageKind::Words
        && word.words.as_ref() == expected.as_bytes()));
    world.close();
}
