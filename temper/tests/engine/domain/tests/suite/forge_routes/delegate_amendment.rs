//! Amendments reach live repairs (temper/docs/design/domain/forge/change/questions.md, 1).
use super::questions::{
    answer, ask, call, make_change, next_producer, plan_world_with_limits, replies, row, wait_answer,
};
use super::*;
use temper_engine_domain_forge_change::{Delegate, Repair, Stands};
use temper_engine_domain_world::fixtures::task_amendment;

const TICKS: u32 = 200;
pub(super) const WORDS: &[u8] = b"Drop the originally requested file.";
const REASON: &[u8] = b"Drop the file";
const CONTEXT: &[u8] = b"\n\nThis change's current spec and rulings: task ";

fn repairing() -> (World, engine::Assignment, u64, engine::Assignment) {
    let (mut world, requester) = plan_world_with_limits(
        None,
        &StoryLimits { batch: Some(4), spec_bytes: Some(4096), ..StoryLimits::default() },
        true,
    );
    let change = make_change(&mut world, &requester, 93, engine::ChangeQuestions::Up);
    let producer = next_producer(&mut world, change);
    let branch = row(&world, change).branch;
    let before = world.assigned.len();
    start_failed_ci(&mut world, &producer, &branch, &[b"build"]);
    let repair = reviews::next_assignment(&mut world, before);
    assert_eq!(row(&world, change).delegate, Some((repair.task, Delegate::Repair(Repair::Ci))));
    world.until(Until::Ready);
    (world, requester, change, repair)
}

pub(super) fn amend(world: &mut World, requester: &engine::Assignment, change: u64) {
    let revision = live_task(world, change).revisions.spec;
    call(
        world,
        requester,
        111,
        engine::Tool::Amend {
            keep_parameters: true,
            target: change,
            revision,
            amendment: task_amendment(Some(WORDS), None, REASON),
        },
    );
    assert!(
        matches!(world.answers.last(), Some(engine::CallAnswer::Amended { .. })),
        "{:?}; {:?}",
        world.answers,
        world.call_errors
    );
    for _ in 0..TICKS {
        world.tick();
        if live_task(world, change).inbox.is_empty() && world.store.pending.is_empty() {
            return;
        }
    }
    panic!("the change takes its amendment: {:?}", live_task(world, change));
}

pub(super) fn assert_amended(
    world: &World,
    change: u64,
    repair: u64,
    original: &tasks::Spec,
    revision: u64,
) -> tasks::Word {
    let amended = live_task(world, repair);
    assert_eq!(amended.revisions.spec, revision + 1);
    let at = original.words.windows(CONTEXT.len()).position(|part| part == CONTEXT).expect("original context");
    assert_eq!(&amended.spec.words[..at], &original.words[..at], "opening and failure lines stay as made");
    let mut expected = original.clone();
    expected.words.clone_from(&amended.spec.words);
    assert_eq!(amended.spec, expected, "only words change");
    let new_revision = live_task(world, change).revisions.spec;
    let words = core::str::from_utf8(&amended.spec.words).expect("repair spec");
    assert!(
        words.contains(&format!(
            "This change's current spec and rulings: task {change} at spec revision {new_revision}."
        )),
        "{words}"
    );
    assert!(words.contains(&format!("Amended to spec revision {new_revision}: Drop the file")), "{words}");
    assert!(
        words.ends_with(&format!("Its current spec, at revision {new_revision}:\n{}", String::from_utf8_lossy(WORDS))),
        "{words}"
    );
    assert_eq!(world.root.tasks().task(repair).expect("core repair").spec, amended.spec);
    let amendments: Vec<_> = replies(world, repair, change)
        .into_iter()
        .filter(|word| matches!(word.kind, tasks::MessageKind::Amendment { .. }))
        .collect();
    let [message] = amendments.as_slice() else { panic!("one amendment: {amendments:?}") };
    assert_eq!(message.kind, tasks::MessageKind::Amendment { revision: revision + 1 });
    let mut reason = format!(
        "Task {change}, your change, was amended to spec revision {new_revision}. Your spec's context now holds its current spec and rulings; work to them from here on. Its reason: Drop the file"
    );
    let bound = usize::try_from(world.env.limits.tasks.message_bytes).expect("message bound");
    if reason.len() > bound {
        reason.truncate(bound - "…".len());
        reason.push('…');
    }
    assert_eq!(message.words.as_ref(), reason.as_bytes());
    (*message).clone()
}

pub(super) fn next_attempt(world: &mut World, repair: &engine::Assignment) -> engine::Assignment {
    for _ in 0..TICKS {
        world.tick();
        if let Some(run) = world.assigned.iter().find(|run| run.task == repair.task && run.attempt > repair.attempt) {
            return run.clone();
        }
    }
    panic!("the amended repair resumes");
}

pub(super) fn assert_brief(world: &World, run: &engine::Assignment, message: &tasks::Word) {
    assert!(run.messages.contains(message), "the next attempt hears its amendment");
    assert!(
        task_brief(run).starts_with(&live_task(world, run.task).spec.words),
        "the next brief reads the amended spec"
    );
}

#[test]
fn a_live_ci_repair_hears_the_amendment_and_its_next_brief_uses_it() {
    let (mut world, requester, change, repair) = repairing();
    let original = live_task(&world, repair.task).spec.clone();
    assert!(original.words.starts_with(b"Repair CI at "));
    assert!(contains(&original.words, b": build\nRepair the failed CI on the change branch."));
    let revision = live_task(&world, repair.task).revisions.spec;
    amend(&mut world, &requester, change);
    let message = assert_amended(&world, change, repair.task, &original, revision);
    for _ in 0..TICKS {
        world.tick();
        if world
            .delivered_messages
            .get(&(repair.task, repair.attempt))
            .is_some_and(|numbers| numbers.contains(&message.number))
        {
            break;
        }
    }
    assert!(
        world
            .delivered_messages
            .get(&(repair.task, repair.attempt))
            .is_some_and(|numbers| numbers.contains(&message.number))
    );
    let turns = live_task(&world, repair.task).turn;
    park_after_turn(&mut world, &repair, None, turns);
    let resumed = next_attempt(&mut world, &repair);
    assert_brief(&world, &resumed, &message);
    world.close();
}

#[test]
fn an_amendment_keeps_the_repairs_question_open_until_its_answer() {
    let (mut world, requester, change, repair) = repairing();
    let question = ask(&mut world, &repair, change);
    let turns = live_task(&world, repair.task).turn;
    park_after_turn(&mut world, &repair, None, turns);
    world.until(Until::Ready);
    assert_eq!(live_task(&world, repair.task).phase, tasks::Phase::Active(tasks::Active::Idle));
    let original = live_task(&world, repair.task).spec.clone();
    let revision = live_task(&world, repair.task).revisions.spec;
    let open = live_task(&world, repair.task).questions.clone();
    amend(&mut world, &requester, change);
    let message = assert_amended(&world, change, repair.task, &original, revision);
    assert_eq!(row(&world, change).questions.open.len(), 1);
    assert_eq!(row(&world, change).questions.open[0].stands, Stands::Asked);
    assert_eq!(live_task(&world, repair.task).questions, open);
    for _ in 0..TICKS {
        world.tick();
    }
    assert!(
        !world.assigned.iter().any(|run| run.task == repair.task && run.attempt > repair.attempt),
        "an amendment never answers"
    );
    answer(&mut world, &requester, change, 112);
    wait_answer(&mut world, repair.task, change);
    let resumed = next_attempt(&mut world, &repair);
    assert_brief(&world, &resumed, &message);
    assert!(resumed.messages.iter().any(|message| message.kind == tasks::MessageKind::Answer { question }));
    assert!(live_task(&world, repair.task).questions.is_empty());
    world.close();
}
