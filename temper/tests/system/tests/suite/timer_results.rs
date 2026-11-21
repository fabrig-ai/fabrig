//! Result and timer words in model queries (temper/docs/design/testing/checks.md, 1).
use jig_domain_tasks as tasks;
use skein_fake_llm_domain::api::{Message, Part, Role, Script};
use temper_engine_domain as engine;
use temper_engine_domain_world::{
    direct::Driver,
    fixtures::{report_delegate, tool_call},
};
use temper_system_world::{composed::call, world};

const MERGE: &[u8] = b"0123456789abcdef0123456789abcdef";
const REPORT: &[u8] = b"A1 reports a4 landed after the deployment repair.";

fn script(turns: Vec<skein_fake_llm_domain::api::Turn>) -> Script {
    Script { cue: Box::new([]), turns: turns.into() }
}

fn finish() -> skein_fake_llm_domain::api::Turn {
    call(b"finish", br#"{"report":"The landing is complete."}"#)
}

fn owner(words: &[u8], tracked: bool) -> (Driver, engine::Assignment) {
    let configs = std::array::from_fn(|_| {
        let mut config = world::config(9820);
        // Reserve a run, not the owner's whole allotment: a4 needs its own run.
        config.core.defaults.rules.maximum_run_spend = 10;
        config.core.defaults.rules.ceiling.delegation.depth = 2;
        config.core.seed_projects[0].policy.policy.ceiling.delegation.depth = 2;
        config.core.seed_projects[0].policy.policy.roles[0].authority.delegation.depth = 2;
        config
    });
    let mut limits = temper_engine_domain_world::walking::limits();
    limits.fleet.slots = 2;
    // The delegate's result and a timer wake each reserve an inbox entry.
    limits.tasks.inbox_bytes = 256;
    let mut root = Driver::configured(temper_engine_domain_world::store::Store::new(), configs, &limits);
    root.settle();
    root.send(world::hello(2));
    root.settle();
    root.sign_in();
    root.settle();
    let mut ask = root.chat_ask(1, words);
    let jig_domain_people::Ask::Create { batch, .. } = &mut ask else { panic!("owner creation") };
    batch[0].tracked = tracked.then_some(1);
    root.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(401)),
        sign_in: root.session(),
        key: [41; 16],
        ask,
    }));
    root.settle();
    let assignment = root.last_assignment();
    (root, assignment)
}

fn child(root: &mut Driver, owner: &engine::Assignment, contract: tasks::Contract) -> engine::Assignment {
    let delegate = engine::Delegate { contract, ..report_delegate(b"a4", Box::new([])) };
    tool_call(root, owner, 101, engine::Tool::Delegate { batch: Box::new([delegate]) });
    let Some(engine::CallAnswer::Delegated(numbers)) = root.answered(101) else {
        panic!("one delegate: {:?}", root.answered(101))
    };
    let task = numbers[0].task;
    world::assignments(root)
        .into_iter()
        .find(|assignment| assignment.task == task)
        .unwrap_or_else(|| panic!("child assigned: {:?}", root.task(task)))
}

fn complete(root: &mut Driver, child: &engine::Assignment, result: tasks::TaskResult) {
    root.answer(
        world::CHANNEL,
        child.task,
        child.attempt,
        0,
        tasks::End::Finished { result, cancel_delegates: false },
        engine::Parts::none(),
    );
    root.settle();
}

fn occurrences(messages: &[Message], words: &[u8]) -> usize {
    messages
        .iter()
        .filter(|message| message.role == Role::User)
        .flat_map(|message| &message.parts)
        .map(|part| match part {
            Part::Text { text } | Part::ToolOutput { output: text, .. } => {
                text.windows(words.len()).filter(|part| *part == words).count()
            }
            Part::ToolCall { .. } | Part::Opaque { .. } => 0,
        })
        .sum()
}

#[test]
fn a1_hears_a4s_merged_ending_once() {
    let (mut root, owner) = owner(b"A1 owns a4", true);
    assert_eq!(root.task(owner.task).expect("A1").tracked, Some(1));
    let child =
        child(&mut root, &owner, tasks::Contract::Change { connector: 1, kind: 2, words: 128, choices: Box::new([]) });
    root.answer(world::CHANNEL, owner.task, owner.attempt, 0, tasks::End::Parked, engine::Parts::none());
    root.settle();
    complete(
        &mut root,
        &child,
        tasks::TaskResult::Change {
            title: tasks::ChangeTitle::new(b"a4".as_slice().into()),
            connector: 1,
            kind: 2,
            resource: 17,
            words: MERGE.into(),
        },
    );
    let resumed = root.last_assignment();
    assert_eq!(resumed.task, owner.task);
    assert!(resumed.attempt > owner.attempt);
    let mut agent = world::scripted_agent_for(&resumed, script(vec![finish()]));
    world::run_assignment(&mut root, &resumed, &mut agent);
    let [query] = agent.prompts() else { panic!("one result-bearing query") };
    assert_eq!(occurrences(&query.messages, format!("Result from task {}:", child.task).as_bytes()), 1);
    assert_eq!(occurrences(&query.messages, MERGE), 1, "the merged identity, not the producer's words");
}

#[test]
fn the_directors_timer_is_acknowledged_before_a1s_report_and_its_removal_is_told() {
    let (mut root, owner) = owner(b"The director waits for A1", false);
    let child = child(&mut root, &owner, tasks::Contract::Report { words: 128 });
    let mut director = world::scripted_waiting_agent_for(
        &owner,
        None,
        script(vec![call(b"subscribe", br#"{"kind":"timer","at":18446744073709551615}"#), call(b"wait", b"{}")]),
    );
    world::run_assignment(&mut root, &owner, &mut director);
    let subscriptions = &root.task(owner.task).expect("parked director").subscriptions;
    let [subscription] = subscriptions.as_ref() else { panic!("one armed timer") };
    let number = subscription.number;
    let added = format!("Subscription {number} added");
    assert!(director.prompts().iter().any(|query| occurrences(&query.messages, added.as_bytes()) == 1));
    assert!(director.prompts().iter().all(|query| occurrences(&query.messages, REPORT) == 0));
    complete(&mut root, &child, tasks::TaskResult::Report { words: REPORT.into() });
    let resumed = root.last_assignment();
    assert_eq!(resumed.task, owner.task);
    let mut director = world::scripted_agent_for(
        &resumed,
        script(vec![call(b"unsubscribe", format!("{{\"subscription\":{number}}}").as_bytes()), finish()]),
    );
    world::run_assignment(&mut root, &resumed, &mut director);
    let [reported, removed] = director.prompts() else { panic!("report, then removal acknowledgement") };
    assert_eq!(occurrences(&reported.messages, REPORT), 1);
    assert_eq!(occurrences(&reported.messages, b"Subscription removed"), 0);
    assert_eq!(occurrences(&removed.messages, b"Subscription removed"), 1);
}
