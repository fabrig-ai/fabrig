//! Provider requests follow a started chat's picker choice on its next turn.
//! Contract: jig/docs/design/domain/tasks/changes.md; smith/docs/design/domain/session.md, 8.2.
use jig_domain_core::{CoreRecord, Record as CoreRow};
use jig_domain_tasks as tasks;
use skein_fake_llm_domain::api::{Part, Query, Script, ToolChoice};
use temper_engine_domain::Record;
use temper_system_world::composed::{World, call, text_script};

const FIRST_REPLY: &[u8] = b"The first answer on model A.";
const NEXT_WORDS: &[u8] = b"Explain the next choice, please.";
const NEXT_REPLY: &[u8] = b"The next answer is ready.";
// The first activation makes one answer; changing the model adds one summary before the next answer.
const FIRST_REQUESTS: usize = 1;
const CHANGED_REQUESTS: usize = 2;
const TWO_ASSIGNMENTS: usize = 2;

fn listening() -> World {
    let mut world = World::chat_on_two_models(960, text_script(FIRST_REPLY));
    world.until_listening();
    world
}

fn occurrences(query: &Query, words: &[u8]) -> usize {
    query
        .messages
        .iter()
        .flat_map(|message| &message.parts)
        .map(|part| match part {
            Part::Text { text } => text.windows(words.len()).filter(|bytes| *bytes == words).count(),
            Part::ToolCall { .. } | Part::ToolOutput { .. } | Part::Opaque { .. } => 0,
        })
        .sum()
}

fn healthy(world: &World) {
    assert_eq!(world.next_turn_task().tries, tasks::Tries::NONE);
    for row in world.store.rows.values() {
        if let Record::Core(CoreRow::Core(CoreRecord::Terminal(terminal))) = row {
            assert!(!matches!(terminal.end, tasks::End::Failed(_)), "{terminal:?}");
        }
    }
}

fn changed_answer(mut world: World) {
    let first_attempt = world.assignment().attempt;
    world.pick(1, Some(1), [61; 16]);
    world.say_again(NEXT_WORDS, text_script(NEXT_REPLY));
    world.run();
    assert_eq!(world.agent().prompts().last().expect("the next answer").model.as_ref(), b"fake-2");
    assert_eq!(world.observed.assignments, TWO_ASSIGNMENTS);
    assert!(world.assignment().attempt > first_attempt);
    assert_eq!(world.assignment().run.policy.model.name.as_ref(), b"fake-2");
    assert_eq!(world.assignment().run.policy.model.effort.as_deref(), Some(b"high".as_slice()));
    let prompts = world.agent().prompts();
    assert_eq!(prompts.len(), CHANGED_REQUESTS, "one summary and one answer");
    let [summary, answer] = prompts else { panic!("summary then answer") };
    assert_eq!(summary.model.as_ref(), b"fake-1");
    assert_eq!(summary.choice, ToolChoice::None);
    assert_eq!(answer.model.as_ref(), b"fake-2");
    assert_ne!(answer.choice, ToolChoice::None);
    assert_eq!(occurrences(answer, NEXT_WORDS), 1, "the next words are delivered once");
    healthy(&world);
}

#[test]
fn a_new_model_answers_the_words_said_while_the_old_run_listens() {
    changed_answer(listening());
}

#[test]
fn a_new_model_answers_the_words_said_after_the_old_run_parked() {
    let mut world = listening();
    world.past_listening();
    world.run();
    changed_answer(world);
}

#[test]
fn a_pick_alone_asks_the_provider_nothing() {
    for parked in [false, true] {
        let mut world = listening();
        if parked {
            world.past_listening();
            world.run();
        }
        world.pick(1, Some(1), [62; 16]);
        world.past_listening();
        world.run();
        assert_eq!(world.agent().prompts().len(), FIRST_REQUESTS, "parked={parked}");
        assert_eq!(world.observed.assignments, 1, "parked={parked}");
        healthy(&world);
    }
}

#[test]
fn a_new_effort_alone_answers_on_it_and_keeps_the_window() {
    let mut world = listening();
    world.pick(0, Some(1), [63; 16]);
    world.say_again(NEXT_WORDS, text_script(NEXT_REPLY));
    world.run();
    assert_eq!(world.observed.assignments, TWO_ASSIGNMENTS);
    assert_eq!(world.assignment().run.policy.model.effort.as_deref(), Some(b"high".as_slice()));
    let [answer] = world.agent().prompts() else { panic!("effort alone needs no summary") };
    assert_eq!(answer.model.as_ref(), b"fake-1");
    assert_ne!(answer.choice, ToolChoice::None);
    assert_eq!(occurrences(answer, FIRST_REPLY), 1, "the existing window is retained");
    assert_eq!(occurrences(answer, NEXT_WORDS), 1);
    healthy(&world);
}

#[test]
fn a_turn_under_way_finishes_on_the_old_choice() {
    let mut script = text_script(FIRST_REPLY);
    let mut turns = vec![call(b"message", br#"{"target":1,"form":"words","words":"Working on the first answer."}"#)];
    turns.extend(script.turns);
    script.turns = turns.into_boxed_slice();
    let mut world = World::chat_on_two_models(961, script);
    world.until_call();
    world.pick(1, Some(1), [64; 16]);
    world.until_listening();
    assert_eq!(world.observed.cancels, 0, "the picker leaves a turn under way");
    assert_eq!(world.observed.assignments, 1);
    assert_eq!(world.agent().prompts().len(), CHANGED_REQUESTS, "host call then reply");
    assert!(world.agent().prompts().iter().all(|prompt| prompt.model.as_ref() == b"fake-1"));
    world.say_again(NEXT_WORDS, text_script(NEXT_REPLY));
    world.run();
    assert_eq!(world.observed.assignments, TWO_ASSIGNMENTS);
    assert_eq!(world.agent().prompts().last().expect("next answer").model.as_ref(), b"fake-2");
    healthy(&world);
}

#[test]
fn a_chat_nobody_changes_keeps_its_run_and_its_model() {
    let mut script = text_script(FIRST_REPLY);
    let mut turns = script.turns.into_vec();
    turns.extend(text_script(NEXT_REPLY).turns);
    script.turns = turns.into_boxed_slice();
    let mut world = World::chat_on_two_models(962, script);
    world.until_listening();
    let attempt = world.assignment().attempt;
    world.say_again(NEXT_WORDS, Script { cue: Box::default(), turns: Box::default() });
    assert_eq!(world.observed.assignments, 1, "unchanged words stay in the same run");
    world.until_listening();
    assert_eq!(world.observed.assignments, 1);
    assert_eq!(world.assignment().attempt, attempt);
    let [first, answer] = world.agent().prompts() else { panic!("two answers in one attempt") };
    assert_eq!(first.model.as_ref(), b"fake-1");
    assert_eq!(answer.model.as_ref(), b"fake-1");
    assert_eq!(occurrences(answer, NEXT_WORDS), 1);
    world.run();
    healthy(&world);
}
