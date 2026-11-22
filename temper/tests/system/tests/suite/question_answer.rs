//! A plan question and its answer reach the model once, in order
//! (temper/docs/design/testing/checks.md, 1).
use jig_domain_core::Record as CoreRow;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_fake_llm_domain::api::{Part, Query, Script};
use skein_lib::{ReplyTo, Token};
use temper_engine_domain::{self as engine, Delivery, Record};
use temper_engine_domain_world::{direct::Driver, fixtures, store::Store, walking};
use temper_system_world::{composed::call, world};

const QUESTION: &[u8] = b"May M1 use B's files?";
const ANSWER: &[u8] = b"Green light: M1 may use B's files.";

fn root() -> Driver {
    let config = std::array::from_fn(|_| {
        let mut config = world::config(955);
        config.core.defaults.charters[0].policy.resume = false;
        // This scenario carries new inbox words, not a prior run's tail.
        config.core.settings.tail_bytes = 0;
        config.core.defaults.rules.ceiling.delegation.depth = 2;
        let policy = &mut config.core.seed_projects[0].policy.policy;
        policy.ceiling.delegation.depth = 2;
        policy.roles[0].authority.delegation.depth = 2;
        config
    });
    let mut bounds = walking::limits();
    bounds.fleet.slots = 2;
    bounds.fleet.attempts = 4;
    // Leave room for a question beside the delegate's reserved result.
    bounds.tasks.inbox_bytes = 512;
    let mut root = Driver::configured(Store::new(), config, &bounds);
    root.send(world::hello(2));
    root.settle();
    root.sign_in();
    root.settle();
    root
}

#[test]
fn a_plan_question_and_its_answer_are_sent_and_delivered_once_in_order() {
    let mut root = root();
    root.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(401)),
        sign_in: root.session(),
        key: [40; 16],
        ask: root.chat_ask(1, b"@director"),
    }));
    root.settle();
    let director = root.last_assignment();
    fixtures::tool_call(
        &mut root,
        &director,
        950,
        engine::Tool::Delegate { batch: Box::new([fixtures::report_delegate(b"@owner", Box::new([]))]) },
    );
    let Some(engine::CallAnswer::Delegated(made)) = root.answered(950) else {
        panic!("owner delegated: {:?}", root.answered(950))
    };
    let owner_task = made[0].task;
    let owner = world::assignments(&root).into_iter().find(|run| run.task == owner_task).expect("owner assigned");
    root.answer(world::CHANNEL, director.task, director.attempt, 0, tasks::End::Parked, engine::Parts::none());
    root.settle();
    let args = serde_json::to_vec(
        &serde_json::json!({"target":director.task,"form":"question","words":String::from_utf8_lossy(QUESTION)}),
    )
    .expect("question arguments");
    let script =
        Script { cue: b"@owner".as_slice().into(), turns: Box::new([call(b"message", &args), call(b"wait", b"{}")]) };
    let mut asking = world::scripted_waiting_agent_for(&owner, None, script);
    world::run_assignment(&mut root, &owner, &mut asking);
    assert!(matches!(asking.answer(), smith_domain_run::Answer::Parked { .. }));
    let receiving = root.last_assignment();
    assert_eq!(receiving.task, director.task);
    let questions: Vec<_> = receiving.messages.iter().filter(|word| word.words.as_ref() == QUESTION).collect();
    assert_eq!(questions.len(), 1);
    let question = questions[0].number;
    assert_eq!(questions[0].from, tasks::Party::Task(owner_task));
    assert_eq!(questions[0].kind, tasks::MessageKind::Question);
    let mut hearing = world::scripted_waiting_agent_for(
        &receiving,
        None,
        Script { cue: b"@director".as_slice().into(), turns: Box::new([call(b"wait", b"{}")]) },
    );
    world::run_assignment(&mut root, &receiving, &mut hearing);
    assert!(matches!(hearing.answer(), smith_domain_run::Answer::Parked { .. }));
    assert_eq!(deliveries(hearing.prompts(), QUESTION), 1);
    assert_eq!(deliveries(hearing.prompts(), ANSWER), 0, "question arrives before the answer exists");
    let acknowledgement = format!("Message {question} sent");
    assert!(asking.prompts().iter().any(|query| query.messages.iter().flat_map(|message| &message.parts).any(|part|
        matches!(part, Part::ToolOutput { output, .. } if output.windows(acknowledgement.len()).any(|part| part == acknowledgement.as_bytes()))
    )), "the question call is acknowledged by number");

    // Answer through the person's task inbox, just as the director's person does.
    root.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(951)),
        sign_in: root.session(),
        key: [41; 16],
        ask: people::Ask::AnswerQuestion { project: 1, task: director.task, question, words: ANSWER.into() },
    }));
    root.settle();
    assert_eq!(
        root.delivered
            .iter()
            .filter(|item| matches!(item, Delivery::WebReply {
        reply: people::Reply::Outcome(people::Outcome::QuestionAnswered { question: found, .. }), ..
    } if *found == question))
            .count(),
        1
    );
    let answered =
        world::assignments(&root).into_iter().rev().find(|run| run.task == owner_task).expect("answer wakes owner");
    let answers: Vec<_> = answered.messages.iter().filter(|word| word.words.as_ref() == ANSWER).collect();
    assert_eq!(answers.len(), 1);
    assert_eq!(answers[0].from, tasks::Party::Task(director.task));
    assert_eq!(answers[0].kind, tasks::MessageKind::Answer { question });
    let mut continuing = world::scripted_waiting_agent_for(
        &answered,
        None,
        Script { cue: b"@owner".as_slice().into(), turns: Box::new([call(b"wait", b"{}")]) },
    );
    world::run_assignment(&mut root, &answered, &mut continuing);
    assert!(matches!(continuing.answer(), smith_domain_run::Answer::Parked { .. }));
    assert_eq!(deliveries(continuing.prompts(), ANSWER), 1);
    // Durable activity counts committed sends, not a repeated history prefix.
    for (task, kind, words) in [
        (director.task, tasks::MessageKind::Question, QUESTION),
        (owner_task, tasks::MessageKind::Answer { question }, ANSWER),
    ] {
        assert_eq!(root.store.rows.values().filter(|row| matches!(row,
            Record::Core(CoreRow::Tasks(tasks::Stored::Activity(row))) if row.task == task && matches!(&row.line,
                tasks::Line::Words { kind: found, words: text, .. } if *found == kind && text.as_ref() == words)
        )).count(), 1);
    }
}

// Prompts replay a growing history; count only newly delivered user input.
fn deliveries(queries: &[Query], words: &[u8]) -> usize {
    let mut previous = &[][..];
    let mut count = 0;
    for query in queries {
        let offset = if query.messages.starts_with(previous) { previous.len() } else { 0 };
        count += query.messages[offset..]
            .iter()
            .filter(|message| message.role == skein_fake_llm_domain::api::Role::User)
            .flat_map(|message| &message.parts)
            .map(|part| match part {
                Part::Text { text } => super::text::occurrences(text, words),
                Part::ToolCall { .. } | Part::ToolOutput { .. } | Part::Opaque { .. } => 0,
            })
            .sum::<usize>();
        previous = &query.messages;
    }
    count
}
