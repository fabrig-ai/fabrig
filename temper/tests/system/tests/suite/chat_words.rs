//! A person's words during a turn (temper/docs/design/domain/agent/README.md, 7).
use super::text::occurrences;
use jig_domain_core::{CoreRecord, Record as CoreRow};
use jig_domain_tasks as tasks;
use skein_fake_llm_domain::api::{Part, Query};
use smith_agent_world::CompletionTerminal;
use temper_engine_domain::{Key, Record};
use temper_system_world::composed::{World, text_script};

const FIRST: &[u8] = b"The first QA reply is complete.";
const WORDS: &[u8] = b"These are the queued QA words.";
const SECOND: &[u8] = b"The queued QA words were answered.";
// One call and committed turn per reply in this Open run.
const REPLIES: usize = 2;

#[test]
fn words_during_a_turn_are_answered_once() {
    let mut script = text_script(FIRST);
    let mut turns = script.turns.to_vec();
    turns.extend(text_script(SECOND).turns);
    script.turns = turns.into_boxed_slice();
    let mut world = World::absence_chat(947, script, b"Hello", 1);
    world.until_model_held();
    world.act(
        jig_fake_parties::Ask::Say { task: jig_fake_parties::Task::Last, words: WORDS.into(), release: None },
        [42; 16],
    );
    for _ in 0..10000 {
        world.tick();
        if !world.observed.messages.is_empty() {
            break;
        }
    }
    assert!(!world.observed.messages.is_empty(), "the person's message reaches the held run");
    assert_eq!(world.agent().prompts().len(), 1);
    assert!(world.agent().turns().is_empty(), "the words arrive before the first response");
    world.release_model();
    world.run();

    let [first, second] = world.agent().prompts() else { panic!("two calls, with no nudge or retry") };
    assert_eq!(message_occurrences(first, WORDS), 0, "the in-flight request cannot take later words");
    assert_eq!(message_occurrences(second, WORDS), 1, "the next request takes the words exactly once");
    for query in [first, second] {
        assert_eq!(message_occurrences(query, b"You stopped without calling `finish`"), 0, "no finite-run nudge");
    }
    assert_eq!(world.agent().completions().len(), REPLIES, "each model call answered once");
    for call in world.agent().completions() {
        assert!(matches!(call.terminal, Some((_, CompletionTerminal::Completed(_)))), "both calls answered ok");
    }
    assert_eq!(world.observed.assignments, 1, "the same run answers the queued words");
    assert_eq!(world.observed.answers, 1);
    assert_eq!(world.observed.turns, REPLIES);
    assert_eq!(world.observed.turn_acks, REPLIES);

    let task = world.assignment().task;
    for reply in [FIRST, SECOND] {
        let committed: usize = world
            .store
            .rows
            .values()
            .map(|row| match row {
                Record::Core(CoreRow::Core(CoreRecord::Turn(turn))) if turn.task == task => {
                    occurrences(&turn.transcript, reply)
                }
                _ => 0,
            })
            .sum();
        assert_eq!(committed, 1, "each reply committed once");
    }
    let Some(Record::Core(CoreRow::Tasks(tasks::Stored::Live(row)))) =
        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(task))))
    else {
        panic!("the Open chat stays live");
    };
    assert_eq!(row.phase, tasks::Phase::Active(tasks::Active::Idle), "the chat parks, not held or ended");
    assert!(row.placed.is_none(), "no placed hold");
    assert_eq!(row.hold, 0, "no hold created");
    assert_eq!(row.result_position, 0, "text alone supplies no result");
    assert_eq!(row.tries, tasks::Tries::NONE);
    assert_ne!(world.absence_tries(), []);
    assert!(world.absence_tries().iter().all(|tries| *tries == tasks::Tries::NONE), "no try spent before parking");
    assert!(!world.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)))));
}

fn message_occurrences(query: &Query, words: &[u8]) -> usize {
    query
        .messages
        .iter()
        .flat_map(|message| &message.parts)
        .map(|part| match part {
            Part::Text { text } => occurrences(text, words),
            Part::ToolCall { .. } | Part::ToolOutput { .. } | Part::Opaque { .. } => 0,
        })
        .sum()
}
