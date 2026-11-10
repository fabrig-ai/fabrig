//! A producer asks the requester named in its brief (temper/docs/design/domain/forge/change/questions.md, 1).
use super::early_start::goal_world;
use super::questions::{call, make_change, next_producer, row};
use super::*;

const TICKS: u32 = 200; // The question relay settles within the change story's step bound.
const OWN_TASK_CALL: u64 = 94;
const REQUESTER_CALL: u64 = 95;
const QUESTION: &[u8] = b"Should I keep the documented behavior? I recommend keeping it.";

#[test]
fn a_producer_asking_its_own_task_is_told_where_its_change_is() {
    let (mut world, goal, _, _) = goal_world();
    let change = make_change(&mut world, &goal, OWN_TASK_CALL - 1, engine::ChangeQuestions::Up);
    let producer = next_producer(&mut world, change);
    world.until(Until::Ready);
    let brief = producer
        .sections
        .iter()
        .map(temper_engine_smith::section)
        .find(|section| section.title.as_ref() == b"Task")
        .expect("task brief");
    let brief = std::str::from_utf8(&brief.text).expect("task brief text");
    assert!(brief.contains(&format!("[This task: {},", producer.task)));
    assert!(brief.contains(&format!("[Requested by task {change}]")));

    call(
        &mut world,
        &producer,
        OWN_TASK_CALL,
        engine::Tool::Message { target: producer.task, form: engine::MessageForm::Question, words: QUESTION.into() },
    );
    let refused = world.answers.last().expect("message answered");
    let rendered = temper_engine_smith::answer_for_tool(
        refused,
        b"message",
        smith_domain_run::HostAnswer::CAPACITY.try_into().expect("host answer bound"),
        temper_engine_smith::ForgeReadBounds::STOCK,
    );
    assert!(rendered.error());
    assert_eq!(rendered.text(), b"Authority refused: task reference. Your brief's [Requested by ...] line names your requester; [This task: N] is you. If it says [Requested by person N], they have no task to message: ask with delegate, a person task addressed to person N.");
    assert!(live_task(&world, producer.task).questions.is_empty());
    assert!(live_task(&world, change).questions.is_empty());
    assert!(row(&world, change).questions.open.is_empty());
    assert!(!matches!(live_task(&world, producer.task).phase, tasks::Phase::Held { .. }));
    assert!(!matches!(live_task(&world, change).phase, tasks::Phase::Held { .. }));

    call(
        &mut world,
        &producer,
        REQUESTER_CALL,
        engine::Tool::Message { target: change, form: engine::MessageForm::Question, words: QUESTION.into() },
    );
    for _ in 0..TICKS {
        if !row(&world, change).questions.open.is_empty() && world.store.pending.is_empty() {
            break;
        }
        world.tick();
    }
    assert!(world.call_errors.is_empty(), "{:?}", world.call_errors);
    let question = live_task(&world, producer.task).questions.first().expect("producer asked its change");
    assert_eq!(question.asked, tasks::Party::Task(change));
    assert_eq!(row(&world, change).questions.open.first().expect("change took its question").words.as_ref(), QUESTION);
}
