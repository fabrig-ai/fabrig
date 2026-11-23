//! The worker's absences across both roots (temper/docs/design/domain/worker/README.md, 2 and 9).
use super::text::occurrences;
use jig_domain_core::{CoreRecord, Record as CoreRow};
use jig_domain_host as host;
use jig_domain_tasks as tasks;
use skein_fake_llm_domain::api::{Finish, Line, Part, Query, Script, Turn};
use skein_lib::Duration;
use temper_engine_domain::{Key, Record};
use temper_system_world::composed::{World, text_script};

const BEFORE: &[u8] = b"Before the engine restart";
const WORDS: &[u8] = b"Explain how the worker survives a restart.";
const REPLY: &[u8] = b"The worker keeps the conversation.";

fn interrupted_script() -> Script {
    let mut script = text_script(REPLY);
    let reply = script.turns[0].clone();
    script.turns = Box::new([
        Turn {
            lines: Box::new([
                Line::Text { text: BEFORE.into() },
                Line::Call {
                    name: b"read_docs".as_slice().into(),
                    arguments: br#"{"topic":"chat"}"#.as_slice().into(),
                },
            ]),
            finish: Finish::ToolCalls,
            tokens: 1,
        },
        reply,
    ]);
    script
}

#[test]
fn an_engine_restarted_within_the_bound_keeps_the_same_attempt() {
    same_attempt_after_starts(0);
}

#[test]
fn a_said_restart_keeps_the_attempt_past_grace_while_its_turns_go_on() {
    let mut script = interrupted_script();
    let mut meanwhile = script.turns[0].clone();
    meanwhile.lines = Box::new([
        Line::Text { text: b"A turn while the engine is away".as_slice().into() },
        Line::Call { name: b"list".as_slice().into(), arguments: br#"{"path":"."}"#.as_slice().into() },
    ]);
    script.turns = Box::new([script.turns[0].clone(), meanwhile, script.turns[1].clone()]);
    let mut world = World::absence_chat_with_grace(944, script, WORDS, 2, Duration::from_secs(300));
    world.until_model_held();
    let attempt = world.assignment().attempt;
    world.settle_engine();
    assert_eq!(world.restart_word(), Some(Duration::from_secs(1200)));
    world.cut_link();
    world.stop_engine();
    let turns = world.observed.turns;
    world.elapse_absence(Duration::from_secs(301));
    assert_eq!(world.observed.cancels, 0, "the grace cancels nothing after the word");
    world.continue_until_model(3);
    for _ in 0..1000 {
        world.tick();
        if world.observed.turns > turns {
            break;
        }
    }
    assert!(
        world.observed.turns > turns,
        "the worker sends the next turn while its engine is away: {:?}",
        world.observed
    );
    world.elapse_absence(Duration::from_secs(299));
    assert_eq!(world.observed.cancels, 0, "600 seconds is within the restart wait");
    world.reopen_engine();
    world.join_link();
    let [_, hello] = world.absence_hellos() else { panic!("the initial and returning hello") };
    let [hosted] = hello.as_slice() else { panic!("one surviving attempt") };
    assert_eq!((hosted.run.raw(), hosted.attempt.raw()), (world.assignment().task, attempt));
    assert_eq!(hosted.phase, jig_domain_link::Phase::Active);
    world.until_model_held();
    world.watch_absence_reply();
    finish_reply(&mut world);
    assert_eq!(committed_occurrences(&world, b"A turn while the engine is away"), 1);
    assert_eq!(committed_occurrences(&world, REPLY), 1);
    assert_eq!(world.absence_assignments().len(), 1);
    assert_eq!(attempt_rows(&world).len(), 1);
    assert_eq!(world.observed.spawns, 1);
    assert_eq!(world.observed.cancels, 0);
    assert_no_tries(&world);
}

#[test]
fn two_refused_engine_starts_within_the_bound_keep_the_same_attempt() {
    same_attempt_after_starts(2);
}

fn same_attempt_after_starts(failures: usize) {
    let mut world = World::absence_chat(941, interrupted_script(), WORDS, 2);
    world.until_model_held();
    let attempt = world.assignment().attempt;
    assert_eq!(committed_occurrences(&world, BEFORE), 1, "the first turn is committed before the absence");
    world.watch_absence_reply();
    world.cut_link();
    world.stop_engine();
    world.advance(Duration::from_millis(100));
    for _ in 0..failures {
        world.refuse_engine_start();
        world.advance(Duration::from_millis(100));
    }
    world.reopen_engine();
    world.join_link();
    assert_eq!(world.refused_engine_starts(), failures);
    assert_eq!(world.absence_admissions(), 2, "only the admitted engine stops the absence");
    let [first, adopted] = world.absence_hellos() else { panic!("one initial hello and one reconnect") };
    assert_eq!(first.as_slice(), []);
    let [adopted] = adopted.as_slice() else { panic!("one hosted attempt") };
    assert_eq!((adopted.run.raw(), adopted.attempt.raw()), (world.assignment().task, attempt));
    assert_eq!(adopted.phase, jig_domain_link::Phase::Active);
    assert_eq!(world.observed.cancels, 0, "the live run survives the engine restart");
    finish_reply(&mut world);
    let [assignment] = world.absence_assignments() else { panic!("one assignment") };
    assert_eq!(assignment.attempt, attempt);
    assert_eq!(world.observed.spawns, 1, "one run, not a replacement");
    assert_eq!(world.absence_prompts().len(), 2, "the model is asked once per turn");
    assert_no_tries(&world);
    assert_eq!(attempt_rows(&world).len(), 1);
    assert_eq!(world.observed.killed, 0);
}

#[test]
fn past_the_engine_away_bound_the_attempt_is_interrupted_and_resumes_without_a_try() {
    let mut world = World::absence_chat(942, interrupted_script(), WORDS, 2);
    world.until_model_held();
    assert_eq!(committed_occurrences(&world, BEFORE), 1);
    world.watch_absence_reply();
    world.cut_link();
    world.stop_engine();
    world.advance(world.engine_away_grace());
    world.until_worker_keeps_answer();
    assert_eq!(world.observed.cancels, 1);
    world.reopen_engine();
    world.join_link();
    world.until_model_held();
    let [first, second] = world.absence_assignments() else { panic!("two assignments") };
    assert_eq!(first.task, second.task);
    assert_ne!(first.attempt, second.attempt);
    assert_eq!(first.conversation, second.conversation, "the replacement keeps the conversation");
    assert!(first.transcript.is_empty());
    assert_eq!(second.transcript.len(), 1, "only the committed first turn is restored");
    assert_eq!(second.transcript.iter().map(|turn| occurrences(turn, BEFORE)).sum::<usize>(), 1);
    let interrupted = first.attempt;
    assert!(
        world.absence_answers().iter().any(|(_, attempt, ending)| *attempt == interrupted
            && matches!(ending, host::Ending::Failed { failure: host::Failure::Cancelled(host::Reason::Contact), .. })),
        "the worker answers Cancelled(Contact)"
    );
    assert_interrupted(&world, interrupted);
    let prompts = world.absence_prompts();
    let [_, before, resumed] = prompts.as_slice() else {
        panic!("first, held second, resumed second query: {}", prompts.len())
    };
    assert!(resumed.messages.len() >= before.messages.len());
    for (before, resumed) in before.messages.iter().zip(&resumed.messages) {
        assert_eq!(before.role, resumed.role);
        assert!(resumed.parts.starts_with(&before.parts), "each resumed message retains its interrupted prefix");
    }
    assert_eq!(world.absence_admissions(), 2);
    finish_reply(&mut world);
    assert_eq!(world.absence_prompts().len(), 3);
    assert_eq!(world.absence_assignments().len(), 2);
    assert_eq!(attempt_rows(&world).len(), 2);
    assert_eq!(world.observed.spawns, 2);
    assert_no_tries(&world);
    assert_eq!(world.observed.killed, 0);
}

#[test]
fn stopping_a_worker_before_the_first_turn_keeps_the_first_message_once() {
    let mut world = World::absence_chat(943, text_script(REPLY), WORDS, 1);
    world.until_model_held();
    assert_eq!(committed_occurrences(&world, REPLY), 0, "the first turn has not committed");
    world.watch_absence_reply();
    world.stop_worker();
    world.start_worker();
    world.until_model_held();
    let [first, second] = world.absence_assignments() else { panic!("two assignments") };
    assert_eq!(first.task, second.task);
    assert_ne!(first.attempt, second.attempt);
    assert!(second.transcript.is_empty(), "no first turn committed");
    assert!(
        world.absence_answers().iter().any(|(_, attempt, ending)| *attempt == first.attempt
            && matches!(
                ending,
                host::Ending::Failed { failure: host::Failure::Cancelled(host::Reason::Shutdown), .. }
            )),
        "the root stops with its shutdown cancellation"
    );
    let prompts = world.absence_prompts();
    let [first, second] = prompts.as_slice() else { panic!("two model queries") };
    for query in [first, second] {
        assert_eq!(message_occurrences(query, WORDS), 1, "the first message reaches each attempt once");
    }
    finish_reply(&mut world);
    assert_eq!(world.absence_prompts().len(), 2);
    assert_eq!(world.absence_assignments().len(), 2);
    assert_eq!(world.observed.spawns, 2);
    assert_eq!(attempt_rows(&world).len(), 2);
    assert_no_tries(&world);
    assert_eq!(world.observed.killed, 0);
}

fn finish_reply(world: &mut World) {
    world.release_model();
    world.until_reply(REPLY);
    let watch = world.run_watch.as_ref().expect("the run watch");
    assert_eq!(occurrences(&watch.text, REPLY), 1, "the reply is delivered once");
    assert_eq!(watch.turn.expect("reply turn").0, world.assignment().attempt);
    world.release_reply();
    world.run();
    assert_eq!(committed_occurrences(world, REPLY), 1, "the reply is committed once");
    assert_eq!(world.observed.gone, world.observed.spawns, "every agent is reaped");
    world.close_absence_chat();
}

fn assert_no_tries(world: &World) {
    assert_ne!(world.absence_tries(), []);
    assert!(
        world.absence_tries().iter().all(|tries| *tries == tasks::Tries::NONE),
        "no try spent, even before parking"
    );
    let task = world.assignment().task;
    let Some(Record::Core(CoreRow::Tasks(tasks::Stored::Ended(row)))) =
        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task))))
    else {
        panic!("the closed chat is ended")
    };
    assert_eq!(row.tries, tasks::Tries::NONE);
}

fn assert_interrupted(world: &World, attempt: u64) {
    assert!(world.store.rows.values().any(|row| matches!(row,
        Record::Core(CoreRow::Core(CoreRecord::Terminal(row))) if row.attempt == attempt && row.end == tasks::End::Recheck
    )), "the engine ends the cancelled attempt interrupted, without a try");
    assert!(attempt_rows(world).iter().any(|row| row.attempt == attempt && row.ended.is_some()));
}

fn attempt_rows(world: &World) -> Vec<&tasks::AttemptRow> {
    world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            Record::Core(CoreRow::Tasks(tasks::Stored::Attempt(row))) => Some(row),
            _ => None,
        })
        .collect()
}

fn committed_occurrences(world: &World, words: &[u8]) -> usize {
    world
        .store
        .rows
        .values()
        .map(|row| match row {
            Record::Core(CoreRow::Core(CoreRecord::Turn(turn))) => occurrences(&turn.transcript, words),
            _ => 0,
        })
        .sum()
}

fn message_occurrences(query: &Query, words: &[u8]) -> usize {
    query
        .messages
        .iter()
        .flat_map(|message| &message.parts)
        .map(|part| match part {
            Part::Text { text } => occurrences(text, words),
            _ => 0,
        })
        .sum()
}
