//! The reference goal reads its cut and then its questions part without an amendment
//! (temper/docs/design/protocol/configuration.md, 13.9; jig/docs/design/domain/tasks.md, 3 and 8.6).
use super::*;
use jig_domain_core::settings;
use jig_domain_link::EndpointBound;

const CHARTER: u32 = 5;
const TEMPLATE: u32 = 4;
const SETTINGS_REQUEST: u8 = 90;
const GOAL_REQUEST: u8 = 92;
const DELEGATE_CALL: u64 = 93;
const QUESTION_CALL: u64 = 94;
const CHANNEL: Token = Token::new(7);
const BUDGET: u64 = 2_500_000; // The reference goal-branch fixture's allowance.
const MODEL: &[u8] = b"claude-opus-5-5";
const QUESTION: &[u8] = b"Should the cut keep the documented behavior? I recommend keeping it.";

fn authority(authority: &mut authority::Authority) {
    authority.budget.spend = BUDGET;
    authority.delegation.kinds =
        [authority.delegation.kinds.as_ref(), &[authority::Executor::Charter(CHARTER)]].concat().into();
}

fn defaults(model: &engine::Model, words: u32) -> settings::Defaults {
    let catalogue = settings::CatalogueModel {
        number: 1,
        model: model.clone(),
        shown: None,
        description: None,
        picker: true,
        withdrawn: false,
        source: settings::ModelSource::Stock,
        estimated: false,
        first_read: None,
    };
    let mut defaults = temper_service::stock::with_models(Box::new([catalogue]), words);
    // The initial rules name charter 5 before the owner applies its reference value.
    let mut coordinator = defaults.charters[0].clone();
    coordinator.number = CHARTER;
    defaults.charters = [defaults.charters.as_ref(), &[coordinator]].concat().into();
    defaults.rules.period_spend = BUDGET;
    authority(&mut defaults.rules.ceiling);
    let policy = defaults.policies.get_mut(&1).expect("stock project policy");
    policy.policy.period_spend = BUDGET;
    authority(&mut policy.policy.ceiling);
    for role in &mut policy.policy.roles {
        authority(&mut role.authority);
        role.period_spend = BUDGET;
    }
    defaults
}

fn reference_items(model: &engine::Model, words: u32) -> Box<[settings::Item]> {
    use jig_client::contract as c;
    let limits = temper_client_tools::limits::tools();
    let decoded = temper_client_tools::jig::settings_file::decode(
        include_bytes!("../../../../../../docs/examples/reference/settings.json"),
        &limits,
    )
    .expect("shipped reference settings");
    let mut items = skein_lib::List::with_capacity(2); // Charter 5 and template 4 travel whole.
    for item in decoded.into_boxed() {
        if item.setting == c::Setting::Charter(c::SettingNumber { number: CHARTER })
            || item.setting == c::Setting::Template(c::SettingNumber { number: TEMPLATE })
        {
            items.push(item).expect("two reference settings");
        }
    }
    let ask = c::Ask::ChangeSettings(c::ChangeSettings { against: 1, items, running: c::Running::Stays });
    let jig_client_protocol_wire::Ask::ChangeSettings(change) =
        jig_client_protocol_link::translate::asks::ask(&ask, &limits.settings).expect("ctl settings wire")
    else {
        panic!("settings change")
    };
    let catalogue = jig_protocol_engine_clients::Model {
        new_until: None,
        number: 1,
        source: settings::ModelSource::Stock,
        estimated: false,
        name: model.name.clone(),
        provider: Some(model.provider),
        window: model.window,
        output: model.output,
        efforts: model.efforts.clone(),
        effort: model.effort.clone(),
        prices: jig_protocol_engine_clients::Prices {
            input: model.input_price,
            cached: model.cached_price,
            cache_write: model.cache_write_price,
            output: model.output_price,
            unit: model.price_unit,
        },
        description: None,
        shown: None,
        picker: true,
    };
    let mut items =
        jig_protocol_engine_clients::translate::settings::items(change.items().into(), &[catalogue], &limits.settings)
            .expect("reference settings reach the core with catalogue models");
    for item in &mut items {
        match &mut item.edit {
            settings::Edit::Set(settings::Value::Charter(charter)) => {
                charter.policy.model = model.clone();
                charter.policy.alternatives = Box::new([]);
                charter.documents = Box::new([]);
            }
            settings::Edit::Set(settings::Value::Template(template)) => {
                template.contract = tasks::Contract::Report { words };
            }
            _ => panic!("only the reference charter and template"),
        }
    }
    items
}

fn apply(world: &mut World, items: Box<[settings::Item]>) {
    let event = jig_domain_core::Event::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(u64::from(SETTINGS_REQUEST))),
        sign_in: world.signed_in.expect("owner session"),
        key: [SETTINGS_REQUEST; 16],
        digest: [SETTINGS_REQUEST; 32],
        change: Box::new(settings::Change { against: 1, items, running: settings::Running::Stays }),
    };
    world.send(engine::Event::Party(temper_service::clients::translation::to_root(event).expect("settings route")));
    world.until(Until::Ready);
    assert!(
        world
            .web_replies
            .iter()
            .any(|reply| matches!(reply, people::Reply::Outcome(people::Outcome::SettingsChanged { .. }))),
        "owner's settings applied: {:?}",
        world.web_replies
    );
}

fn request(world: &World, template: &jig_domain_core::Template) -> people::Ask {
    let people::Ask::Create { project, mut batch } = world.chat(1, b"Cut the reference goal") else {
        panic!("goal batch")
    };
    let people::TemplateExecutor::Charter(charter) = template.executor else { panic!("goal charter") };
    let tasks::Contract::Report { words } = template.contract else { panic!("goal report") };
    let member = &mut batch[0];
    member.executor = people::TaskExecutor::Agent { charter };
    member.contract = people::Contract::Report { words };
    member.budget = people::Budget::Named;
    member.authority.spend = BUDGET;
    // The reference goal-branch fixture's 2026-11-04T13:00Z deadline.
    member.authority.deadline = Some(skein_lib::Wall::from_nanos(1_793_797_200_000_000_000));
    member.deadline = people::Deadline::Named;
    member.tracked = template.tracked;
    member.label = template.label;
    member.flags = people::Flags {
        conversation: template.flags.conversation,
        listed: template.flags.listed,
        retention: people::Retention::Kept,
        ended_by: people::EndedBy::Result,
    };
    member.llm = template.llm.as_ref().map(|choice| people::LlmChoice {
        account: None,
        main: people::ModelEffort { model: choice.main.model, effort: choice.main.effort },
        subagents: choice
            .subagents
            .iter()
            .map(|model| people::ModelEffort { model: model.model, effort: model.effort })
            .collect(),
        later: choice.later.as_ref().map(|later| people::LlmLater {
            main: people::ModelEffort { model: later.main.model, effort: later.main.effort },
            subagents: later
                .subagents
                .iter()
                .map(|model| people::ModelEffort { model: model.model, effort: model.effort })
                .collect(),
        }),
    });
    member.symbols = template
        .resources
        .iter()
        .map(|symbol| people::Symbol { connector: symbol.connector, symbol: symbol.symbol })
        .collect();
    people::Ask::Create { project, batch }
}

fn instructions(shared: &[u8], part: &[u8]) -> Vec<u8> {
    [shared, b"\n\n", part].concat()
}

#[test]
#[ignore = "forge world's Store::crash reopens the scripted catalogue, losing template 4's model and efforts"]
fn the_reference_goal_reads_its_cut_then_its_questions_part() {
    let mut model = stock::box_model();
    model.name = MODEL.into();
    // The scripted catalogue uses the reference's token prices, within charter 5's spend.
    model.input_price = 1;
    model.cached_price = 1;
    model.cache_write_price = None;
    model.output_price = 1;
    model.price_unit = 1000;
    model.efforts = Box::new([Box::from(b"xhigh".as_slice()), Box::from(b"high".as_slice())]);
    model.effort = Some(b"xhigh".as_slice().into());
    let mut limits = temper_engine_domain_world::forge_routes::box_limits();
    limits.documents.lines = 8; // The reference goal-branch fixture's document room.
    let defaults = defaults(&model, limits.tasks.result_bytes);
    let items = reference_items(&model, limits.tasks.result_bytes);
    let charter = items
        .iter()
        .find_map(|item| match &item.edit {
            settings::Edit::Set(settings::Value::Charter(charter)) => Some(charter.clone()),
            _ => None,
        })
        .expect("decoded charter 5");
    let template = items
        .iter()
        .find_map(|item| match &item.edit {
            settings::Edit::Set(settings::Value::Template(template)) => Some(template.clone()),
            _ => None,
        })
        .expect("decoded template 4");
    let cut = instructions(&charter.policy.instructions, &charter.parts[0].instructions);
    let questions = instructions(&charter.policy.instructions, &charter.parts[1].instructions);
    let mut world = World::configured_stock_limits(&defaults, &model, &limits);
    world.adopt();
    apply(&mut world, items);
    let mut event = protocol::hello_slots(CHANNEL, 2, Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut event
    else {
        panic!("worker hello")
    };
    hello.bounds = jig_test_profile::fixtures::bounds();
    hello.bounds.time = Duration::from_secs(28_800); // Charter 5's reference run bound.
    hello.bounds.endpoints =
        Box::new([EndpointBound { endpoint: model.endpoint, window: model.window, output: model.output }]);
    world.send(event);
    let ask = request(&world, &template);
    let first = assigned_chat(&mut world, GOAL_REQUEST, ask);
    assert_eq!(first.run.policy.model.name.as_ref(), MODEL, "row 1: model");
    assert_eq!(first.run.policy.model.effort.as_deref(), Some(b"xhigh".as_slice()), "row 1: effort");
    assert!(live_task(&world, first.task).reasons.contains(tasks::Reasons::FIRST));

    let mut delegate =
        temper_engine_domain_world::fixtures::report_delegate(b"Ask the goal about its cut", Box::new([]));
    delegate.authority = live_task(&world, first.task).authority.clone();
    delegate.authority.budget.spend = 10_000; // Covers the scripted delegate's reserved completion.
    delegate.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let answer = world.scripted_call(&first, DELEGATE_CALL, engine::Tool::Delegate { batch: Box::new([delegate]) });
    let engine::CallAnswer::Delegated(made) = answer else { panic!("delegate admitted: {answer:?}") };
    let child = made[0].task;
    world.until(Until::SecondAssignment);
    let delegate = world.assigned.iter().find(|run| run.task == child).expect("delegate assigned").clone();
    let answer = world.scripted_call(
        &delegate,
        QUESTION_CALL,
        engine::Tool::Message { target: first.task, form: engine::MessageForm::Question, words: QUESTION.into() },
    );
    let engine::CallAnswer::Sent { message } = answer else { panic!("delegate's question sent: {answer:?}") };
    world.until(Until::Ready);
    let question = live_task(&world, first.task)
        .inbox
        .iter()
        .find(|word| word.number == message)
        .expect("row 2: question keeps its inbox place")
        .clone();
    assert_eq!(question.from, tasks::Party::Task(child));
    assert_eq!(question.words.as_ref(), QUESTION);
    assert_eq!(question.kind, tasks::MessageKind::Question);
    let withheld =
        !world.delivered_messages.get(&(first.task, first.attempt)).is_some_and(|messages| messages.contains(&message));

    let turns = live_task(&world, first.task).turn;
    park_after_turn(&mut world, &first, None, turns);
    world.until(Until::Ready);
    let next = world
        .assigned
        .iter()
        .find(|run| run.task == first.task && run.attempt > first.attempt)
        .expect("row 3: question wakes the next run");
    let cut_read = first.run.policy.instructions.as_ref() == cut;
    let questions_read = next.run.policy.instructions.as_ref() == questions;
    assert!(
        cut_read && questions_read,
        "row 1: shared instructions and cut: {cut_read}; row 3: shared instructions and questions: {questions_read}"
    );
    assert!(withheld, "row 2: question withheld from the cut");
    assert_eq!(next.run.policy.model.name.as_ref(), MODEL, "row 3: model");
    assert_eq!(next.run.policy.model.effort.as_deref(), Some(b"high".as_slice()), "row 3: later effort");
    assert!(next.messages.contains(&question), "row 3: start carries the question");
    let stored = live_task(&world, first.task);
    assert_eq!(stored.llm.as_ref(), template.llm.as_ref(), "row 3: original choice retained");
    assert_eq!(stored.revisions.spec, 0, "row 3: no amendment");
    world.close();
}
