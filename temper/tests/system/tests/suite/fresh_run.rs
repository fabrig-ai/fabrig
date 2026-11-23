//! Fresh conversations after a person's release (testing/checks.md, 1).
use jig_domain_core::{CoreRecord, Record as CoreRow};
use jig_domain_host as host;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_fake_llm_domain::api::{Part, Query, Role, Script};
use skein_lib::{ReplyTo, Token};
use smith_agent_world::World as Agent;
use temper_engine_domain::{self as engine, Delivery, Record};
use temper_engine_domain_world::{direct::Driver, fixtures, store::Store, walking};
use temper_system_world::{composed::call, world};

const CUE: &[u8] = b"@fresh-release";
const QUESTION: &[u8] = b"What is the next milestone's scope?";
const ANSWER: &[u8] = b"The next milestone owns the design.";
const OLD_GUIDE: &[u8] = b"Keep the old process.";
const GUIDE: &[u8] = b"Report the current process.";
const CHANNEL: Token = world::CHANNEL;

struct Released {
    root: Driver,
    first: Agent,
    director: Agent,
    owner: Agent,
}

fn ask(root: &mut Driver, key: u8, ask: people::Ask) {
    root.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: root.session(),
        key: [key; 16],
        ask,
    }));
    root.settle();
}

fn assigned(root: &Driver, task: u64) -> engine::Assignment {
    world::assignments(root).into_iter().rev().find(|run| run.task == task).expect("task assigned")
}

fn script(turns: Box<[skein_fake_llm_domain::api::Turn]>) -> Script {
    Script { cue: CUE.into(), turns }
}

fn released() -> Released {
    let configs = std::array::from_fn(|_| {
        let mut config = world::config(951);
        config.core.defaults.charters[0].policy.resume = false;
        config.core.defaults.rules.ceiling.delegation.depth = 2;
        let policy = &mut config.core.seed_projects[0].policy.policy;
        policy.ceiling.delegation.depth = 2;
        policy.roles[0].authority.delegation.depth = 2;
        config
    });
    let mut limits = walking::limits();
    limits.tasks.depth = 2;
    // Reserve the delegate's report beside its question and the release notice.
    limits.tasks.inbox_bytes = 512;
    limits.fleet.slots = 2;
    limits.fleet.workstreams = 2;
    limits.fleet.attempts = 4;
    // One placed hold writes both members' snapshots in the same decision.
    limits.journal.writes = 2 * temper_engine_domain_world::journal_limits::WALKING_WRITES;
    limits.call_records = 4;
    let mut root = Driver::configured(Store::new(), configs, &limits);
    root.send(world::hello(2));
    root.settle();
    root.sign_in();
    root.settle();
    let create = root.chat_ask(1, CUE);
    ask(&mut root, 40, create);
    let director = root.last_assignment();
    fixtures::tool_call(
        &mut root,
        &director,
        900,
        engine::Tool::Delegate { batch: Box::new([fixtures::report_delegate(CUE, Box::new([]))]) },
    );
    root.settle();
    let Some(engine::CallAnswer::Delegated(made)) = root.answered(900) else {
        panic!("delegated: {:?}", root.answered(900))
    };
    let owner = assigned(&root, made[0].task);
    let mut first =
        world::scripted_waiting_agent_with_guide(&director, None, script(Box::new([call(b"wait", b"{}")])), OLD_GUIDE);
    world::run_assignment(&mut root, &director, &mut first);
    ask(&mut root, 41, people::Ask::Stop { project: 1, task: director.task });
    root.answer(CHANNEL, owner.task, owner.attempt, 0, tasks::End::Parked, engine::Parts::none());
    root.settle();
    let revision = root.task(director.task).expect("held director").revisions.hold;
    ask(&mut root, 42, people::Ask::Release { project: 1, task: director.task, revision, again: None });
    let next = assigned(&root, director.task);
    let owner_run = assigned(&root, owner.task);
    fixtures::tool_call(
        &mut root,
        &owner_run,
        901,
        engine::Tool::Message { target: director.task, form: engine::MessageForm::Question, words: QUESTION.into() },
    );
    root.settle();
    let Some(engine::CallAnswer::Sent { message }) = root.answered(901) else {
        panic!("owner's question: {:?}, {:?}", root.answered(901), root.delivered.last())
    };
    let question = *message;
    let history = (!next.transcript.is_empty()).then(|| world::transcript(&first));
    let mut director = world::scripted_waiting_agent_with_guide(
        &next,
        history,
        script(Box::new([call(b"subscribe", br#"{"kind":"timer","at":18446744073709551615}"#), call(b"wait", b"{}")])),
        GUIDE,
    );
    deliver_messages(&root, &next, &mut director);
    world::run_assignment(&mut root, &next, &mut director);
    ask(&mut root, 43, people::Ask::AnswerQuestion { project: 1, task: next.task, question, words: ANSWER.into() });
    let mut owner = world::scripted_waiting_agent_with_guide(
        &owner_run,
        None,
        script(Box::new([call(b"finish", br#"{"report":"Scope reported."}"#)])),
        GUIDE,
    );
    deliver_messages(&root, &owner_run, &mut owner);
    world::run_assignment(&mut root, &owner_run, &mut owner);
    Released { root, first, director, owner }
}

fn deliver_messages(root: &Driver, assignment: &engine::Assignment, agent: &mut Agent) {
    // The parent sends only after Smith accepts the assigned run.
    for _ in 0..1000 {
        if agent
            .messages_seen()
            .iter()
            .any(|(_, seen)| matches!(seen, smith_agent_world::messages_referee::Seen::Admitted))
        {
            break;
        }
        agent.drive(1);
    }
    // Preserve the engine's message names, labels and rendered text.
    for delivery in &root.delivered {
        if let Delivery::Link { down: host::LinkDown::Message { run, attempt, name, sender, words, .. }, .. } = delivery
            && run.raw() == assignment.task
            && attempt.raw() == assignment.attempt
        {
            agent.message_at(agent.now(), *name, sender.clone(), words.clone());
        }
    }
}

fn occurrences(query: &Query, words: &[u8]) -> usize {
    query
        .messages
        .iter()
        .flat_map(|message| &message.parts)
        .map(|part| {
            let bytes = match part {
                Part::Text { text } | Part::ToolOutput { output: text, .. } => text,
                Part::ToolCall { arguments, .. } => arguments,
                Part::Opaque { .. } => return 0,
            };
            super::text::occurrences(bytes, words)
        })
        .sum()
}

#[test]
fn a_released_fresh_run_has_no_earlier_assistant_turns() {
    let released = released();
    assert!(!released.first.turns().is_empty(), "the opening turn was committed");
    let first = released.director.prompts().first().expect("fresh query");
    assert!(first.messages.iter().all(|message| message.role != Role::Assistant));
}

#[test]
fn a_numbered_question_is_delivered_once_in_one_growing_history() {
    let released = released();
    let [first, second] = released.director.prompts() else { panic!("answer and wait queries") };
    // Task 2 asks question 2: the release notice took message 1.
    assert_eq!(occurrences(first, b"2 asks (question 2"), 1);
    assert_eq!(occurrences(first, QUESTION), 1);
    assert_eq!(occurrences(second, QUESTION), 1);
    assert_eq!(first.system, second.system);
    assert!(second.messages.starts_with(&first.messages));
}

#[test]
fn repository_rules_are_not_messages_or_reread_notices() {
    let released = released();
    for (agent, guide) in [(&released.first, OLD_GUIDE), (&released.director, GUIDE), (&released.owner, GUIDE)] {
        assert_ne!(agent.prompts(), []);
        for query in agent.prompts() {
            assert!(skein_lib::bytes::find(&query.system, guide).is_some());
            for words in [OLD_GUIDE, GUIDE, b"re-read", b"reread"] {
                assert_eq!(occurrences(query, words), 0);
            }
        }
    }
}

#[test]
fn released_runs_do_not_hide_invalid_starts_behind_retries() {
    let released = released();
    let terminals: Vec<_> = released
        .root
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            Record::Core(CoreRow::Core(CoreRecord::Terminal(row))) => Some(&row.end),
            _ => None,
        })
        .collect();
    // The opening director and stopped owner, then each released run.
    assert_eq!(terminals.len(), 4);
    assert!(terminals.iter().all(|end| !matches!(end, tasks::End::Failed(tasks::Class::Invalid))));
}

#[test]
fn the_owners_answer_reaches_its_query_once() {
    let released = released();
    let [query] = released.owner.prompts() else { panic!("one reporting query") };
    assert_eq!(occurrences(query, ANSWER), 1);
}
