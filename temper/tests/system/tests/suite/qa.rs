//! Milestone words and report-only QA at the model boundary (testing/reference-case.md).
use jig_domain_tasks as tasks;
use skein_fake_llm_domain::api::{Part, Query, Script};
use skein_lib::{ReplyTo, Token};
use temper_engine_domain as engine;
use temper_engine_domain_world::{direct::Driver, fixtures, store::Store, walking};
use temper_system_world::{composed::call, world};

const M1: &str = "M1 ended.\nDesign followed.\nChanges landed.\nCI green.\nReviews passed.\nSTATUS updated.\nNo blocker.\nM2 may start.\nQA follows.\nA1 stays live.";

fn root() -> (Driver, engine::Assignment) {
    let mut bounds = walking::limits();
    // Director, A, A1 and M2; one message and relay, each with a provenance header.
    bounds.people.inbox_tasks = 8;
    bounds.fleet.attempts = 5;
    bounds.tasks.depth = 2;
    bounds.tasks.delegates = 2;
    bounds.tasks.tasks = 4;
    bounds.tasks.tree_tasks = 4;
    bounds.tasks.project_tasks = 4;
    bounds.tasks.message_bytes = 256;
    bounds.tasks.inbox_bytes = 1024;
    bounds.call_records = 8;
    // The engine's four-task batch fixture, including message receipts.
    bounds.journal.writes = 4170;
    bounds.journal.deliveries = 100;
    bounds.journal.held = 300;
    bounds.fleet.slots = 4;
    bounds.brief.sections = 5;
    let configs = std::array::from_fn(|_| {
        let mut config = world::config(9831);
        config.core.defaults.rules.maximum_run_spend = 10;
        config.core.defaults.rules.ceiling.delegation.depth = 3;
        config.core.defaults.rules.ceiling.delegation.tasks = 4;
        let policy = &mut config.core.seed_projects[0].policy.policy;
        policy.ceiling.delegation.depth = 3;
        policy.ceiling.delegation.tasks = 4;
        policy.roles[0].authority.delegation.depth = 3;
        policy.roles[0].authority.delegation.tasks = 4;
        config
    });
    let mut root = Driver::configured(Store::with_limits(&bounds), configs, &bounds);
    root.send(world::hello(4));
    root.settle();
    root.sign_in();
    root.settle();
    root.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(401)),
        sign_in: root.session(),
        key: [41; 16],
        ask: root.chat_ask(1, b"@director"),
    }));
    root.settle();
    let run = root.last_assignment();
    (root, run)
}

fn delegate(
    root: &mut Driver,
    parent: &engine::Assignment,
    number: u64,
    member: engine::Delegate,
) -> engine::Assignment {
    fixtures::tool_call(root, parent, number, engine::Tool::Delegate { batch: Box::new([member]) });
    let Some(engine::CallAnswer::Delegated(children)) = root.answered(number) else {
        panic!("delegation: {:?}", root.answered(number))
    };
    let child = children[0].task;
    world::assignments(root).into_iter().find(|run| run.task == child).expect("child assigned")
}

fn park(root: &mut Driver, run: &engine::Assignment) {
    root.answer(world::CHANNEL, run.task, run.attempt, 0, tasks::End::Parked, engine::Parts::none());
    root.settle();
}

fn script(cue: &[u8], turns: Vec<skein_fake_llm_domain::api::Turn>) -> Script {
    Script { cue: cue.into(), turns: turns.into_boxed_slice() }
}

fn words(query: &Query, needle: &[u8]) -> usize {
    query
        .messages
        .iter()
        .flat_map(|message| &message.parts)
        .map(|part| match part {
            Part::Text { text } => text.windows(needle.len()).filter(|part| *part == needle).count(),
            Part::ToolCall { .. } | Part::ToolOutput { .. } | Part::Opaque { .. } => 0,
        })
        .sum()
}

#[test]
fn a_milestone_message_starts_m2_and_reaches_the_director_once() {
    let (mut root, director) = root();
    let mut member = fixtures::report_delegate(b"@owner", Box::new([]));
    member.authority.budget.spend = 100;
    member.authority.delegation =
        tasks::Delegation { kinds: Box::new([tasks::AuthorityExecutor::Charter(1)]), tasks: 2, depth: 1 };
    let owner = delegate(&mut root, &director, 70, member);
    let subowner = delegate(&mut root, &owner, 71, fixtures::report_delegate(b"@subowner", Box::new([])));
    park(&mut root, &director);
    park(&mut root, &owner);
    let input =
        serde_json::to_vec(&serde_json::json!({"target":owner.task,"form":"words","words":M1})).expect("message JSON");
    let mut sender = world::scripted_waiting_agent_for(
        &subowner,
        None,
        script(b"@subowner", vec![call(b"message", &input), call(b"wait", b"{}")]),
    );
    world::run_assignment(&mut root, &subowner, &mut sender);
    let receiving =
        world::assignments(&root).into_iter().rev().find(|run| run.task == owner.task).expect("message wakes A");
    let relay = format!("A1's M1 report:\n{M1}");
    let input = serde_json::to_vec(&serde_json::json!({"target":director.task,"form":"words","words":relay}))
        .expect("relay JSON");
    let mut owner_agent = world::scripted_waiting_agent_for(&receiving, None,
        script(b"@owner", vec![
            call(b"delegate", br#"{"batch":[{"executor":{"kind":"agent","charter":1},"spec":{"words":"M2 sub-owner"},"contract":{"kind":"report","words":128},"authority":{"tools":0,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},"budget":{"spend":10},"notes":0}}]}"#),
            call(b"message", &input), call(b"wait", b"{}") ]));
    world::drive_assignment(&mut root, &receiving, &mut owner_agent);
    let receiving = world::assignments(&root)
        .into_iter()
        .rev()
        .find(|run| run.task == director.task)
        .expect("relay wakes director");
    let mut director_agent =
        world::scripted_waiting_agent_for(&receiving, None, script(b"@director", vec![call(b"wait", b"{}")]));
    world::drive_assignment(&mut root, &receiving, &mut director_agent);
    assert_eq!(M1.lines().count(), 10);
    assert_eq!(owner_agent.prompts().len(), 3, "delegate, relay and wait only");
    assert!(
        root.root
            .tasks()
            .view_tasks()
            .iter()
            .any(|row| row.number != director.task && row.number != owner.task && row.number != subowner.task),
        "the model delegated M2 on receiving M1: {:?}",
        root.protocol.answers
    );
    for query in owner_agent.prompts() {
        for part in query.messages.iter().flat_map(|message| &message.parts) {
            if let Part::ToolOutput { output, is_error, .. } = part {
                assert!(!is_error, "{}", String::from_utf8_lossy(output));
            }
        }
        assert_eq!(words(query, M1.as_bytes()), 1, "M1 is offered once, not duplicated in the next query");
    }
    let [query] = director_agent.prompts() else { panic!("one director query") };
    assert_eq!(words(query, relay.as_bytes()), 1, "the exact relayed report reaches the director once");
}

#[test]
fn an_undeclared_tester_status_returns_unknown_tool_to_the_model() {
    let (mut root, parent) = root();
    let mut member = fixtures::report_delegate(b"@tester", Box::new([]));
    member.authority.tools = tasks::Tools(0);
    let tester = delegate(&mut root, &parent, 70, member);
    let mut settings = smith_protocol_llm_world::fixture::settings(9831);
    let bounds = smith_protocol_llm_world::fixture::limits();
    let receiving = smith_protocol_llm_fixtures::receiving(&bounds.client, settings.limits.decoded_call_bytes)
        .expect("receiving bounds");
    settings.limits.session.completion_bytes = receiving.max_completion_bytes;
    settings.limits.session.completion_blocks = receiving.max_completion_blocks;
    settings.limits.session.failure_bytes = receiving.max_failure_bytes;
    // The protocol peer supplies account zero and its fixed model window; the
    // task's result contract, brief and authority-derived declarations stay intact.
    let mut charter = temper_engine_smith::charter(&tester);
    charter.llm.account = Some(0);
    charter.llm.window = smith_protocol_llm_world::MODEL_WINDOW;
    let mut agent = smith_protocol_llm_world::World::with_workspace_protocol_charter(
        settings, None, None, skein_fake_checkout::Checkout::new(),
        (smith_protocol_llm_world::fixture::Dialect::OpenAi.configuration(), bounds),
        Box::new([script(b"@tester", vec![
        call(b"effect_forge", br#"{"repository":{"forge":1,"repository":1},"resource":{"kind":"repository"},"write":{"kind":"status","commit":"0000000000000000000000000000000000000001","context":"tester-without-authority","check":"passed"}}"#),
        call(b"finish", br#"{"form":"report","text":"tests/qa.rs\n#[test] fn regression() {}","fields":{}}"#),
    ])]), charter);
    agent.run(100_000);
    let [first, answer] = agent.prompts() else { panic!("status then report: {:?}", agent.answer()) };
    assert!(!first.tools.iter().any(|tool| tool.name.as_ref() == b"effect_forge" || tool.name.as_ref() == b"delegate"));
    let errors: Vec<_> = answer
        .messages
        .iter()
        .flat_map(|message| &message.parts)
        .filter_map(|part| match part {
            Part::ToolOutput { output, is_error, .. } => Some((output.as_ref(), *is_error)),
            Part::Text { .. } | Part::ToolCall { .. } | Part::Opaque { .. } => None,
        })
        .collect();
    assert_eq!(errors, [(b"Error: unknown tool".as_slice(), false)]);
    assert!(agent.host_submissions().is_empty(), "the undeclared call never reaches the engine");
}
