//! A run keeps its revision, a chat moves, and the owner advances the rest (temper/docs/design/client/tools.md, 11.4).
use jig_domain_core::settings as s;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
use skein_lib::{Duration, ReplyTo, Token};
use temper_engine_domain::{Assignment, Delivery, Event, PartyInput};
use temper_engine_domain_world::{direct::Driver, store::Store, walking};
use temper_system_world::world;

const SEED: u64 = 9817;
const NEW_INSTRUCTIONS: &[u8] = b"Read the owner's newly revised instructions.";

#[test]
fn running_work_keeps_its_revision_a_chat_moves_and_an_advance_moves_the_rest() {
    let mut limits = walking::limits();
    // Two creations, one change, one advance and three waking messages.
    limits.people.requests = 7;
    // The timer reserves a delivery; each owner message carries one byte beside it.
    limits.tasks.inbox_bytes = limits.tasks.result_bytes.max(limits.tasks.message_bytes) + 1;
    let mut root = Driver::configured(Store::new(), [world::config(SEED), world::config(SEED)], &limits);
    root.settle();
    root.send(world::hello(2));
    root.settle();
    root.sign_in();
    root.settle();
    let mut parked = Vec::new();
    for (key, words, conversation) in
        [(41, b"@revision-one".as_slice(), false), (42, b"@revision-two".as_slice(), true)]
    {
        let mut ask = root.chat_ask(1, words);
        let people::Ask::Create { batch, .. } = &mut ask else { panic!("chat creation") };
        batch[0].flags.conversation = conversation;
        // Share the fixture's available spend between the two parked tasks.
        batch[0].budget = people::Budget::AsAllowed { cap: Some(250) };
        root.send(Event::Party(PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(key)),
            sign_in: root.session(),
            key: [u8::try_from(key).expect("fixture key"); 16],
            ask,
        }));
        root.settle();
        let assignment = root.last_assignment();
        assert_eq!(assignment.run.revision, 1);
        let row = root.task(assignment.task).expect("created task");
        assert_eq!(row.settings.task, 1);
        assert_eq!(row.flags.conversation, conversation);
        let mut agent =
            world::scripted_waiting_agent_for(&assignment, None, waiting(&assignment.run.policy.instructions, true));
        world::run_assignment(&mut root, &assignment, &mut agent);
        assert!(matches!(
            root.task(assignment.task).expect("parked task").phase,
            tasks::Phase::Active(tasks::Active::Idle)
        ));
        parked.push(assignment);
    }
    let mut charter = world::charter();
    charter.policy.instructions = NEW_INSTRUCTIONS.into();
    root.send(Event::Party(PartyInput::ChangeSettings {
        kinds: Box::default(),
        reply_to: ReplyTo::new(Token::new(80)),
        sign_in: root.session(),
        key: [80; 16],
        digest: [80; 32],
        change: Box::new(s::Change {
            against: 1,
            items: Box::new([s::Item {
                setting: s::Setting::Charter(1),
                edit: s::Edit::Set(s::Value::Charter(Box::new(charter))),
            }]),
            running: s::Running::Stays,
        }),
    }));
    root.settle();
    assert!(root.delivered.iter().any(|delivery| matches!(
        delivery,
        Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::SettingsChanged { revision: 2 }), .. }
    )));
    for assignment in &parked {
        assert_eq!(root.task(assignment.task).expect("held old revision").settings.task, 1);
    }
    let first = wake_and_run(&mut root, &parked[0], 90, 2, 1, &parked[0].run.policy.instructions);
    let _chat = wake_and_run(&mut root, &parked[1], 91, 3, 2, NEW_INSTRUCTIONS);
    root.send(Event::Party(PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(81)),
        sign_in: root.session(),
        key: [81; 16],
        ask: people::Ask::MoveToNewest { revision: 1 },
    }));
    root.settle();
    assert_eq!(root.task(first.task).expect("owner moved task").settings.task, 2);
    assert!(root.delivered.iter().any(|delivery| matches!(
        delivery,
        Delivery::WebReply {
            reply: people::Reply::Outcome(people::Outcome::MovedToNewest { newest: 2, moved: 1, stayed: 0 }),
            ..
        }
    )));
    let _advanced = wake_and_run(&mut root, &first, 92, 4, 2, NEW_INSTRUCTIONS);
}

fn wake_and_run(
    root: &mut Driver,
    previous: &Assignment,
    key: u8,
    at: u64,
    revision: u32,
    instructions: &[u8],
) -> Assignment {
    root.say(previous.task, key);
    root.fire_at(Duration::from_secs(at));
    root.settle();
    let next = root.last_assignment();
    assert_eq!(next.task, previous.task);
    assert!(next.attempt > previous.attempt);
    let row = root.task(next.task).expect("woken task");
    assert_eq!(next.run.revision, revision, "task row: {row:?}");
    assert_eq!(next.run.policy.instructions.as_ref(), instructions);
    assert_eq!(row.settings.task, revision);
    let mut agent = world::scripted_waiting_agent_for(&next, None, waiting(instructions, true));
    world::run_assignment(root, &next, &mut agent);
    assert!(
        agent.prompts().iter().any(|query| skein_lib::bytes::find(&query.system, instructions).is_some()),
        "smith sees revision {revision}'s instructions"
    );
    if revision == 1 {
        assert!(agent.prompts().iter().all(|query| skein_lib::bytes::find(&query.system, NEW_INSTRUCTIONS).is_none()));
    }
    assert_eq!(root.task(next.task).expect("parked task").settings.task, revision);
    next
}

fn waiting(cue: &[u8], subscribe: bool) -> Script {
    let mut turns = Vec::new();
    if subscribe {
        // Keep the result-ended task awaiting input across the revision change.
        turns.push(Turn {
            lines: Box::new([Line::Call {
                name: b"subscribe".as_slice().into(),
                arguments: br#"{"kind":"timer","at":18446744073709551615}"#.as_slice().into(),
            }]),
            finish: Finish::ToolCalls,
            tokens: 20,
        });
    }
    turns.push(Turn {
        lines: Box::new([Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() }]),
        finish: Finish::ToolCalls,
        tokens: 20,
    });
    Script { cue: cue.into(), turns: turns.into_boxed_slice() }
}
