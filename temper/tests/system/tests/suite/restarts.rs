//! Restart the engine while Smith holds a named model response.
use jig_domain_core::{CoreRecord, Record as CoreRow};
use jig_domain_tasks as tasks;
use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
use skein_lib::Duration;
use skein_world::faults::{FaultLedger, Struck};
use temper_engine_domain::{Key, Record};
use temper_faults::restarts::{ENGINE_CRASH_MID_TURN, ENGINE_RESTART_MID_TURN};
use temper_system_world::composed::{
    World,
    restarts::{Point, Restart},
    text_script,
};

pub(super) const WORDS: &[u8] = b"Explain how this run survives the engine restart.";
const REPLY: &[u8] = b"The same worker finishes after the engine opens.";
const BEFORE: &[u8] = b"Before the restart";
const MEANWHILE: &[u8] = b"A turn while the engine is away";

/// Two engine documentation reads, followed by the chat's reply.
pub(super) fn script() -> Script {
    let mut script = text_script(REPLY);
    let read_docs = |text: &[u8]| Turn {
        lines: Box::new([
            Line::Text { text: text.into() },
            Line::Call { name: b"read_docs".as_slice().into(), arguments: br#"{"topic":"chat"}"#.as_slice().into() },
        ]),
        finish: Finish::ToolCalls,
        tokens: 1,
    };
    script.turns = vec![read_docs(BEFORE), read_docs(MEANWHILE), script.turns[0].clone()].into_boxed_slice();
    script
}

#[test]
fn settled_engine_restart_at_third_query_keeps_the_attempt_past_grace() {
    let mut world = World::absence_chat_with_grace(944, script(), WORDS, 3, Duration::from_secs(300));
    world.until_model_held();
    assert_eq!(world.agent().prompts().len(), 3);
    assert_eq!(committed_occurrences(&world, BEFORE), 1);
    let attempt = world.assignment().attempt;
    let mut ledger = FaultLedger::new();
    let injection = world.inject_mid_turn(Restart::Settled, &mut ledger);
    let stopped = world.restart_now();
    assert_eq!(world.restart_word(), Some(Duration::from_secs(1200)));
    assert_eq!(ledger.get(injection).until, None);
    world.elapse_absence(Duration::from_secs(301));
    assert_eq!(world.observed.cancels, 0);
    let turns = world.observed.turns;
    world.release_model();
    for _ in 0..1000 {
        world.tick();
        if world.observed.turns > turns {
            break;
        }
    }
    assert!(world.observed.turns > turns, "the held turn goes on while the engine is away");
    world.elapse_absence(Duration::from_secs(299));
    assert_eq!(world.observed.cancels, 0);
    world.open_after_mid_turn(&mut ledger, injection);
    let opened = world.restart_now();
    let [entry] = ledger.entries() else { panic!("one restart") };
    assert_eq!(entry.kind, ENGINE_RESTART_MID_TURN);
    assert_eq!(entry.from, stopped);
    assert_eq!(entry.until, Some(opened));
    assert!(opened > stopped);
    assert_eq!(entry.struck, Struck::default());
    world.join_link();
    assert_adopted(&world, attempt, jig_domain_link::Phase::Answered);
    world.run();
    world.close_absence_chat();
    assert_finished(&world, attempt);
}

#[test]
fn crashed_engine_at_second_query_reopens_with_the_same_attempt() {
    let mut world = World::absence_chat(941, script(), WORDS, 2);
    world.until_model_held();
    assert_eq!(world.agent().prompts().len(), 2);
    assert_eq!(committed_occurrences(&world, BEFORE), 1);
    let attempt = world.assignment().attempt;
    let mut ledger = FaultLedger::new();
    let stopped = world.restart_now();
    let injection = world.inject_mid_turn(Restart::Crash, &mut ledger);
    assert_eq!(world.restart_word(), None, "a crash says no restart word");
    assert_eq!(ledger.get(injection).until, None);
    world.elapse_absence(Duration::from_millis(100));
    world.open_after_mid_turn(&mut ledger, injection);
    let opened = world.restart_now();
    let [entry] = ledger.entries() else { panic!("one crash") };
    assert_eq!(entry.kind, ENGINE_CRASH_MID_TURN);
    assert_eq!(entry.from, stopped);
    assert_eq!(entry.until, Some(opened));
    assert!(opened > stopped);
    assert_eq!(entry.struck, Struck::default());
    world.join_link();
    assert_adopted(&world, attempt, jig_domain_link::Phase::Active);
    finish(&mut world, attempt, Point::Model(2));
}

pub(super) fn assert_adopted(world: &World, attempt: u64, phase: jig_domain_link::Phase) {
    let [first, returned] = world.absence_hellos() else { panic!("initial and returning hello") };
    assert_eq!(first.as_slice(), []);
    let [hosted] = returned.as_slice() else { panic!("one surviving run") };
    assert_eq!((hosted.run.raw(), hosted.attempt.raw()), (world.assignment().task, attempt));
    assert_eq!(hosted.phase, phase);
}

pub(super) fn finish(world: &mut World, attempt: u64, point: Point) {
    world.model_after_mid_turn(point, 3);
    world.watch_absence_reply();
    world.release_model();
    world.until_reply(REPLY);
    let watch = world.run_watch.as_ref().expect("live reply");
    assert_eq!(occurrences(&watch.text, REPLY), 1);
    assert_eq!(watch.turn.expect("reply turn").0, attempt);
    world.release_reply();
    world.run();
    world.close_absence_chat();
    assert_finished(world, attempt);
}

fn assert_finished(world: &World, attempt: u64) {
    assert_eq!(world.absence_assignments().len(), 1);
    assert_eq!(world.assignment().attempt, attempt);
    let attempts = world
        .store
        .rows
        .values()
        .filter(|row| matches!(row, Record::Core(CoreRow::Tasks(tasks::Stored::Attempt(_)))))
        .count();
    assert_eq!(attempts, 1);
    assert_eq!(world.observed.spawns, 1);
    assert_eq!(world.observed.gone, 1);
    assert_eq!(world.observed.cancels, 0);
    assert_eq!(world.observed.killed, 0);
    assert_eq!(world.observed.relays, 2, "both documentation calls reach the host");
    assert_eq!(world.observed.replies, 2);
    assert_eq!(world.agent().prompts().len(), 3);
    assert_eq!(world.agent().turns().len(), 3);
    assert_eq!(world.observed.turn_acks, 3);
    assert!(world.observed.turns >= world.observed.turn_acks, "a lost turn is resent after the hello");
    assert_ne!(world.absence_tries(), []);
    assert!(world.absence_tries().iter().all(|tries| *tries == tasks::Tries::NONE));
    let Some(Record::Core(CoreRow::Tasks(tasks::Stored::Ended(row)))) =
        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(world.assignment().task))))
    else {
        panic!("chat ended durably")
    };
    assert_eq!(row.tries, tasks::Tries::NONE);
    for text in [BEFORE, MEANWHILE, REPLY] {
        assert_eq!(committed_occurrences(world, text), 1, "each turn is durable once");
    }
}

fn committed_occurrences(world: &World, text: &[u8]) -> usize {
    world
        .store
        .rows
        .values()
        .map(|row| match row {
            Record::Core(CoreRow::Core(CoreRecord::Turn(turn))) => occurrences(&turn.transcript, text),
            _ => 0,
        })
        .sum()
}

fn occurrences(text: &[u8], words: &[u8]) -> usize {
    text.windows(words.len()).filter(|part| *part == words).count()
}
