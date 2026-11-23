//! A run keeps its revision, a chat moves, and the owner advances the rest (temper/docs/design/client/ctl/testing.md, 4).
use jig_domain_core::settings as s;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
use skein_lib::{Duration, ReplyTo, Token};
use smith_domain::Transcript;
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
    let parked = park_tasks(&mut root);
    revise_charter(&mut root);
    for (assignment, _) in &parked {
        assert_eq!(root.task(assignment.task).expect("held old revision").settings.task, 1);
    }
    let first = wake_and_run(&mut root, &parked[0], 90, 2, 1, &parked[0].0.run.policy.instructions);
    let _chat = wake_and_run(&mut root, &parked[1], 91, 3, 2, NEW_INSTRUCTIONS);
    root.send(Event::Party(PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(81)),
        sign_in: root.session(),
        key: [81; 16],
        ask: people::Ask::MoveToNewest { revision: 1 },
    }));
    root.settle();
    assert_eq!(root.task(first.0.task).expect("owner moved task").settings.task, 2);
    assert!(root.delivered.iter().any(|delivery| matches!(
        delivery,
        Delivery::WebReply {
            reply: people::Reply::Outcome(people::Outcome::MovedToNewest { newest: 2, moved: 1, stayed: 0 }),
            ..
        }
    )));
    let _advanced = wake_and_run(&mut root, &first, 92, 4, 2, NEW_INSTRUCTIONS);
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one amendment crosses the director's tool, the engine and the owner's query"
)]
fn a_directors_amendment_carries_the_new_direction_to_the_owners_query() {
    use temper_engine_domain as engine;
    use temper_engine_domain_world::fixtures::{report_delegate, tool_call};
    const DIRECTION: &[u8] = b"@amended-owner Take one end-to-end path first.";
    const REASON: &[u8] = b"The person wants the end-to-end story before the old plan.";
    let configs = std::array::from_fn(|_| {
        let mut config = world::config(SEED);
        config.core.defaults.rules.ceiling.delegation.depth = 2;
        config.core.seed_projects[0].policy.policy.ceiling.delegation.depth = 2;
        config.core.seed_projects[0].policy.policy.roles[0].authority.delegation.depth = 2;
        config
    });
    let mut limits = walking::limits();
    limits.tasks.depth = 2;
    limits.fleet.slots = 2;
    // Delegate, park on a timer, then amend at the observed revision.
    limits.call_records = 3;
    let mut root = Driver::configured(Store::new(), configs, &limits);
    root.settle();
    root.send(world::hello(2));
    root.settle();
    root.sign_in();
    root.settle();
    root.send(Event::Party(PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(41)),
        sign_in: root.session(),
        key: [41; 16],
        ask: root.chat_ask(1, b"Direct the owner"),
    }));
    root.settle();
    let director = root.last_assignment();
    tool_call(
        &mut root,
        &director,
        90,
        engine::Tool::Delegate {
            batch: Box::new([report_delegate(b"@amended-owner Follow the old plan", Box::new([]))]),
        },
    );
    let owner = root.last_assignment();
    assert_ne!(owner.task, director.task, "delegate: {:?}", root.answered(90));
    tool_call(
        &mut root,
        &owner,
        92,
        engine::Tool::Subscribe {
            kind: tasks::SubscriptionKind::Timer { at: skein_lib::Wall::from_nanos(u64::MAX), period: None },
        },
    );
    root.answer(world::CHANNEL, owner.task, owner.attempt, 0, tasks::End::Parked, engine::Parts::none());
    root.settle();
    let revision = root.task(owner.task).expect("owner").revisions.spec;
    let arguments = serde_json::to_vec(&serde_json::json!({
        "target": owner.task, "revision": revision,
        "amendment": {"instructions": String::from_utf8_lossy(DIRECTION), "reason": String::from_utf8_lossy(REASON)}
    }))
    .expect("amend arguments");
    let script = Script {
        cue: b"Direct the owner".as_slice().into(),
        turns: Box::new([temper_system_world::composed::call(b"amend", &arguments)]),
    };
    let mut director_agent = world::scripted_waiting_agent_for(&director, None, script);
    director_agent.enable_parent_host_calls();
    assert!(!director_agent.drive(1000), "director yields its amendment");
    let calls = director_agent.pending_host_calls();
    let submission = calls.first().expect("amend call");
    let temper_engine_smith::Decoded::Call(body) =
        temper_engine_smith::call(submission.name, &submission.tool, &submission.input).expect("declared amend tool")
    else {
        panic!("amend reaches the engine")
    };
    let engine::Tool::Amend { amendment, .. } = &body.tool else { panic!("director amends") };
    assert_eq!(amendment.reason.as_ref(), REASON, "the director supplies the reason");
    tool_call(&mut root, &director, 91, body.tool);
    assert!(matches!(root.answered(91), Some(engine::CallAnswer::Amended { .. })), "{:?}", root.answered(91));
    root.settle();
    let amended = root.last_assignment();
    assert_eq!(amended.task, owner.task);
    assert!(amended.attempt > owner.attempt);
    let mut agent = world::scripted_agent_for(
        &amended,
        Script {
            cue: b"@amended-owner".as_slice().into(),
            turns: Box::new([temper_system_world::composed::call(
                b"finish",
                br#"{"report":"The new direction is heard."}"#,
            )]),
        },
    );
    world::run_assignment(&mut root, &amended, &mut agent);
    let query = agent.prompts().first().expect("owner query");
    assert_eq!(
        query
            .messages
            .iter()
            .flat_map(|message| &message.parts)
            .filter(|part| {
                matches!(part, skein_fake_llm_domain::api::Part::Text { text }
            if skein_lib::bytes::find(text, DIRECTION).is_some())
            })
            .count(),
        1,
        "the amendment instructions are told once in the query"
    );
}

fn park_tasks(root: &mut Driver) -> Vec<(Assignment, Transcript)> {
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
            world::scripted_waiting_agent_for(&assignment, None, waiting(&assignment.run.policy.instructions, 0));
        world::run_assignment(root, &assignment, &mut agent);
        assert!(matches!(
            root.task(assignment.task).expect("parked task").phase,
            tasks::Phase::Active(tasks::Active::Idle)
        ));
        parked.push((assignment, world::transcript(&agent)));
    }
    parked
}

fn revise_charter(root: &mut Driver) {
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
}

fn wake_and_run(
    root: &mut Driver,
    previous: &(Assignment, Transcript),
    key: u8,
    at: u64,
    revision: u32,
    instructions: &[u8],
) -> (Assignment, Transcript) {
    let (previous, transcript) = previous;
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
    let mut agent = world::scripted_waiting_agent_for(
        &next,
        Some(transcript.clone()),
        waiting(instructions, transcript.turns.len()),
    );
    world::run_assignment(root, &next, &mut agent);
    assert!(agent.turns().first().expect("restored turn").sequence > 1, "history is resumed");
    assert!(matches!(agent.answer(), smith_domain_run::Answer::Parked { .. }), "{:?}", agent.answer());
    assert!(
        agent.prompts().iter().any(|query| skein_lib::bytes::find(&query.system, instructions).is_some()),
        "smith sees revision {revision}'s instructions"
    );
    if revision == 1 {
        assert!(agent.prompts().iter().all(|query| skein_lib::bytes::find(&query.system, NEW_INSTRUCTIONS).is_none()));
    }
    assert_eq!(root.task(next.task).expect("parked task").settings.task, revision);
    let mut saved = world::transcript(&agent);
    let mut turns = transcript.turns.to_vec();
    turns.extend_from_slice(&saved.turns);
    saved.turns = turns.into();
    (next, saved)
}

fn waiting(cue: &[u8], preceding: usize) -> Script {
    let mut turns = Vec::new();
    // Keep the result-ended task awaiting input across the revision change.
    turns.push(Turn {
        lines: Box::new([Line::Call {
            name: b"subscribe".as_slice().into(),
            arguments: br#"{"kind":"timer","at":18446744073709551615}"#.as_slice().into(),
        }]),
        finish: Finish::ToolCalls,
        tokens: 20,
    });
    // The provider counts saved assistant turns before selecting the next scripted reply.
    for _ in 0..=preceding {
        turns.push(Turn {
            lines: Box::new([Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() }]),
            finish: Finish::ToolCalls,
            tokens: 20,
        });
    }
    Script { cue: cue.into(), turns: turns.into_boxed_slice() }
}
