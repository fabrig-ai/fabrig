//! Two parked runs retain their settings until the owner's advance (temper/docs/design/client/tools.md, 11.4).
use jig_domain_core::settings as s;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
use skein_lib::{Duration, ReplyTo, Token};
use temper_engine_domain::{Event, PartyInput};
use temper_engine_domain_world::{direct::Driver, store::Store, walking};
use temper_system_world::world;

const SEED: u64 = 9817;
const NEW_INSTRUCTIONS: &[u8] = b"Read the owner's newly revised instructions.";

#[test]
fn two_parked_tasks_keep_their_revision_until_the_owner_advances_them() {
    let mut limits = walking::limits();
    // Two creations, one change, one advance and one message per task.
    limits.people.requests = 6;
    // The timer reserves a delivery; each owner message carries one byte beside it.
    limits.tasks.inbox_bytes = limits.tasks.result_bytes.max(limits.tasks.message_bytes) + 1;
    let mut root = Driver::configured(Store::new(), [world::config(SEED), world::config(SEED)], &limits);
    root.settle();
    root.send(world::hello(2));
    root.settle();
    root.sign_in();
    root.settle();
    let mut parked = Vec::new();
    for (key, words) in [(41, b"@revision-one".as_slice()), (42, b"@revision-two".as_slice())] {
        let ask = root.chat_ask(1, words);
        root.send(Event::Party(PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(key)),
            sign_in: root.session(),
            key: [u8::try_from(key).expect("fixture key"); 16],
            ask,
        }));
        root.settle();
        let assignment = root.last_assignment();
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
    for assignment in &parked {
        assert_eq!(root.task(assignment.task).expect("held old revision").settings.task, 1);
    }
    root.send(Event::Party(PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(81)),
        sign_in: root.session(),
        key: [81; 16],
        ask: people::Ask::MoveToNewest { revision: 1 },
    }));
    root.settle();
    for (index, assignment) in parked.iter().enumerate() {
        assert_eq!(root.task(assignment.task).expect("moved task").settings.task, 2);
        root.say(assignment.task, 90 + u8::try_from(index).expect("two tasks"));
        root.fire_at(Duration::from_secs(2 + u64::try_from(index).expect("two tasks")));
        root.settle();
        let next = root.last_assignment();
        assert_eq!(next.task, assignment.task);
        assert!(next.attempt > assignment.attempt);
        assert_eq!(next.run.policy.instructions.as_ref(), NEW_INSTRUCTIONS);
        let mut agent = world::scripted_waiting_agent_for(&next, None, waiting(&next.run.policy.instructions, false));
        world::run_assignment(&mut root, &next, &mut agent);
        assert!(
            agent.prompts().iter().any(|query| skein_lib::bytes::find(&query.system, NEW_INSTRUCTIONS).is_some()),
            "smith sees the moved revision"
        );
    }
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
