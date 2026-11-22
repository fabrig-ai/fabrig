//! A released owner's first model query takes both queued endings once.
//! Contract: temper/docs/design/testing/checks.md, 1.
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_fake_llm_domain::api::{Finish, Line, Part, Query, Script, Turn};
use skein_lib::{ReplyTo, Token};
use temper_engine_domain as engine;
use temper_engine_domain_world::{direct::Driver, fixtures, store::Store, walking};
use temper_system_world::world;

const OWNER: &[u8] = b"@stopped-owner";
const REPORT: &[u8] = b"The implementer finished.";
const CHANGE: &[u8] = b"The change landed.";
const CHANNEL: Token = Token::new(7);

fn configured() -> (Driver, engine::Assignment) {
    // Owner and two children; leave one task slot for the root's census.
    let mut limits = walking::limits();
    limits.fleet.slots = 3;
    limits.fleet.attempts = 5;
    limits.tasks.tasks = 4;
    limits.tasks.project_tasks = 4;
    limits.tasks.tree_tasks = 4;
    limits.tasks.delegates = 3;
    limits.tasks.batch = 2;
    limits.tasks.depth = 1;
    limits.tasks.inbox_messages = 6;
    limits.tasks.inbox_bytes = 512;
    limits.people.inbox_tasks = 8;
    limits.people.inbox_recipients = 17;
    limits.authority.batch = 2;
    limits.call_records = 4;
    limits.brief.sections = 5;
    limits.journal.writes = 6200;
    limits.journal.deliveries = 100;
    limits.journal.held = 300;
    let configs = std::array::from_fn(|_| {
        let mut config = world::config(952);
        config.core.defaults.rules.maximum_run_spend = 5;
        config.core.defaults.rules.ceiling.delegation.tasks = 4;
        config.core.defaults.rules.ceiling.delegation.depth = 2;
        let policy = &mut config.core.seed_projects[0].policy.policy;
        policy.ceiling.delegation.tasks = 4;
        policy.ceiling.delegation.depth = 2;
        policy.roles[0].authority.delegation.tasks = 4;
        policy.roles[0].authority.delegation.depth = 2;
        config
    });
    let mut root = Driver::configured(Store::new(), configs, &limits);
    root.settle();
    root.send(world::hello(3));
    root.settle();
    root.sign_in();
    root.settle();
    let ask = root.chat_ask(1, OWNER);
    person(&mut root, 41, ask);
    let owner = root.last_assignment();
    (root, owner)
}

fn person(root: &mut Driver, key: u8, ask: people::Ask) {
    root.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: root.session(),
        key: [key; 16],
        ask,
    }));
    root.settle();
}

fn occurrences(query: &Query, words: &[u8]) -> usize {
    query
        .messages
        .iter()
        .flat_map(|message| &message.parts)
        .map(|part| match part {
            Part::Text { text } => text.windows(words.len()).filter(|part| *part == words).count(),
            Part::ToolCall { .. } | Part::ToolOutput { .. } | Part::Opaque { .. } => 0,
        })
        .sum()
}

#[test]
fn only_the_first_query_after_release_carries_both_endings() {
    let (mut root, owner) = configured();
    let mut change = fixtures::report_delegate(b"change", Box::new([]));
    change.contract = tasks::Contract::Change { connector: 1, kind: 2, words: 128, choices: Box::new([]) };
    fixtures::tool_call(
        &mut root,
        &owner,
        950,
        engine::Tool::Delegate { batch: Box::new([fixtures::report_delegate(b"implementer", Box::new([])), change]) },
    );
    let children: Vec<_> = world::assignments(&root).into_iter().filter(|run| run.task != owner.task).collect();
    let [implementer, change] = children.as_slice() else { panic!("two running delegates") };
    person(&mut root, 81, people::Ask::Stop { project: 1, task: owner.task });
    root.answer(CHANNEL, owner.task, owner.attempt, 0, tasks::End::Parked, engine::Parts::none());
    root.settle();
    // Their already-completed answers cross cancellation while the owner stays held.
    for (run, result) in [
        (implementer, tasks::TaskResult::Report { words: REPORT.into() }),
        (
            change,
            tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(b"h1".as_slice().into()),
                connector: 1,
                kind: 2,
                resource: 1,
                words: CHANGE.into(),
            },
        ),
    ] {
        root.answer(
            CHANNEL,
            run.task,
            run.attempt,
            0,
            tasks::End::Finished { result, cancel_delegates: false },
            engine::Parts::none(),
        );
        root.settle();
    }
    let revision = root.task(owner.task).expect("held owner").revisions.hold;
    assert_eq!(
        world::assignments(&root).iter().filter(|run| run.task == owner.task).count(),
        1,
        "no query can start while held"
    );
    person(&mut root, 82, people::Ask::Release { project: 1, task: owner.task, revision, again: None });
    let resumed = root.last_assignment();
    assert_eq!(resumed.task, owner.task);
    let call = |name: &[u8], arguments: &[u8]| Turn {
        lines: Box::new([Line::Call { name: name.into(), arguments: arguments.into() }]),
        finish: Finish::ToolCalls,
        tokens: 20,
    };
    let read = format!("{{\"messages\":{:?}}}", resumed.messages.iter().map(|word| word.number).collect::<Vec<_>>());
    let mut agent = world::scripted_agent_for(
        &resumed,
        Script {
            cue: OWNER.into(),
            turns: Box::new([
                call(b"mark_read", read.as_bytes()),
                call(b"finish", br#"{"report":"Both endings read."}"#),
            ]),
        },
    );
    world::run_assignment(&mut root, &resumed, &mut agent);
    let [first, second] = agent.prompts() else { panic!("two model queries") };
    for words in [REPORT, CHANGE] {
        assert_eq!(occurrences(first, words), 1, "the release query takes each ending once");
        // Historical conversation keeps the first query; it must not introduce a second copy.
        assert_eq!(occurrences(second, words), 1, "a read ending is not delivered again");
    }
}
