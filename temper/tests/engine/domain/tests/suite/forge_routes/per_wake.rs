//! The reference goal reads its cut and then its questions part without an amendment
//! (temper/docs/design/protocol/configuration/reference.md; jig/docs/design/domain/tasks/README.md, 3, and jig/docs/design/domain/tasks/messages.md, 6).
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
    let mut defaults = temper_engine_service::stock::with_models(Box::new([catalogue]), words);
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
    let mut items = skein_lib::List::with_capacity(2); // Charter 5's overlay and template 4.
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
            settings::Edit::Set(settings::Value::Overlay(overlay)) => {
                overlay.model = Some(model.clone());
                overlay.alternatives = Some(Box::new([]));
                overlay.documents = Some(Box::new([]));
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
    world.send(engine::Event::Party(
        temper_engine_service::clients::translation::to_root(event).expect("settings route"),
    ));
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
        ended_by: match template.flags.ended_by {
            tasks::EndedBy::Result => people::EndedBy::Result,
            tasks::EndedBy::Requester => people::EndedBy::Requester,
            tasks::EndedBy::Standing => people::EndedBy::Standing,
            tasks::EndedBy::Closer(closer) => {
                people::EndedBy::Closer(people::Closer { connector: closer.connector, code: closer.code })
            }
        },
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

#[path = "stock_per_wake.rs"]
mod stock_per_wake;

fn instructions(shared: &[u8], part: &[u8]) -> Vec<u8> {
    [shared, b"\n\n", part].concat()
}

struct ReferenceGoal {
    world: World,
    first: engine::Assignment,
    template: jig_domain_core::Template,
    cut: Vec<u8>,
    questions: Vec<u8>,
    results: Vec<u8>,
}

fn reference_goal() -> ReferenceGoal {
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
            settings::Edit::Set(settings::Value::Overlay(overlay)) => {
                let stock = defaults.charters.iter().find(|charter| charter.number == CHARTER).expect("stock goal");
                Some(settings::overlaid(stock, overlay))
            }
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
    let cut = [
        charter.policy.instructions.as_ref(),
        b"\n\n",
        &charter.parts[0].instructions,
        b"\n\n",
        &charter.parts[1].instructions,
        b"\n\n",
        &charter.parts[2].instructions,
    ]
    .concat();
    let results = instructions(&charter.policy.instructions, &charter.parts[2].instructions);
    let questions = instructions(&charter.policy.instructions, &charter.parts[1].instructions);
    let mut world = World::configured_stock_limits(&defaults, &model, &limits);
    world.adopt();
    apply(&mut world, items);
    let mut event = protocol::hello_slots(CHANNEL, 3, Box::new([]));
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
    ReferenceGoal { world, first, template: *template, cut, questions, results }
}

#[test]
fn the_reference_goal_reads_its_cut_then_its_questions_part() {
    let ReferenceGoal { mut world, first, template, cut, questions, .. } = reference_goal();
    assert_eq!(first.run.policy.model.name.as_ref(), MODEL, "row 1: model");
    assert_eq!(first.run.policy.model.effort.as_deref(), Some(b"xhigh".as_slice()), "row 1: effort");
    assert!(live_task(&world, first.task).reasons.contains(tasks::Reasons::FIRST));

    let delegate = delegate(&mut world, &first);
    let child = delegate.task;
    let turns = live_task(&world, first.task).turn;
    park_after_turn(&mut world, &first, None, turns);
    world.until(Until::Ready);
    assert_eq!(live_task(&world, first.task).phase, tasks::Phase::Active(tasks::Active::Idle));
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
    assert_eq!(next.run.policy.model.name.as_ref(), MODEL, "row 3: model");
    assert_eq!(next.run.policy.model.effort.as_deref(), Some(b"high".as_slice()), "row 3: later effort");
    assert!(next.messages.contains(&question), "row 3: start carries the question");
    let stored = live_task(&world, first.task);
    assert_eq!(stored.llm.as_ref(), template.llm.as_ref(), "row 3: original choice retained");
    assert_eq!(stored.revisions.spec, 0, "row 3: no amendment");
    world.close();
}

/// The cut hears a real change's cancel, then its last live change's landing, without waiting.
#[test]
fn the_reference_goals_running_cut_hears_a_cancelled_change_and_its_last_landing() {
    let ReferenceGoal { mut world, first, cut, .. } = reference_goal();
    let row = live_task(&world, first.task);
    assert!(matches!(row.flags.ended_by, tasks::EndedBy::Closer(_)));
    assert_eq!(row.wake.results, tasks::ResultsWake::LastOrFailure);
    assert!(row.offered.contains(tasks::Reasons::RESULT));

    let answer = super::change_tool::named_read(
        &mut world,
        &first,
        100,
        b"change",
        br#"{"changes":[{"words":"Try the unavailable change","landing":"goal"},{"words":"Land the remaining change","landing":"goal"}]}"#,
    );
    let engine::CallAnswer::Delegated(made) = answer else { panic!("two live changes: {answer:?}") };
    assert_eq!(made.len(), 2);
    let (cancelled, last) = (made[0].task, made[1].task);
    assert_eq!(live_task(&world, first.task).delegates.as_ref(), &[cancelled, last]);
    let producer = super::questions::next_producer(&mut world, cancelled);
    let remaining = super::questions::next_producer(&mut world, last);
    assert_eq!(live_task(&world, producer.task).requester, tasks::Party::Task(cancelled));
    let answer = world.scripted_call(
        &first,
        101,
        engine::Tool::Cancel { target: cancelled, reason: b"Drop this change".as_slice().into() },
    );
    assert_eq!(answer, engine::CallAnswer::Controlled, "change cancelled");
    let event =
        world.protocol.answer(CHANNEL, producer.task, producer.attempt, 1, tasks::End::Refused, engine::Parts::none());
    world.send(event);
    world.until(Until::Ready);
    read_change_result(&mut world, &first, cancelled, true);
    assert_eq!(live_task(&world, first.task).delegates.as_ref(), &[last]);

    let producer = remaining;
    assert_eq!(live_task(&world, producer.task).requester, tasks::Party::Task(last));
    let branch = producer.workspace.repositories[0].push.clone().expect("change push branch");
    super::change_tool::land_change(&mut world, &producer, &branch);
    world.until(Until::Ready);
    assert_eq!(world.merges, 1, "the remaining change lands");
    read_change_result(&mut world, &first, last, false);
    let row = live_task(&world, first.task);
    assert!(row.delegates.is_empty());
    assert_eq!(row.attempt, first.attempt, "still designing in the original run");
    assert_eq!(row.reasons, tasks::Reasons::FIRST);
    assert_eq!(first.run.policy.instructions.as_ref(), cut);
    assert_eq!(world.assigned.iter().filter(|run| run.task == first.task).count(), 1);
    world.close();
}

fn read_change_result(world: &mut World, run: &engine::Assignment, change: u64, cancelled: bool) {
    for _ in 0..1000 {
        // The forge world's full landing and hold-release bound.
        if live_task(world, run.task)
            .inbox
            .iter()
            .any(|word| word.from == tasks::Party::Task(change) && matches!(word.kind, tasks::MessageKind::Result(_)))
        {
            break;
        }
        world.tick();
    }
    world.until(Until::Ready);
    let result = live_task(world, run.task)
        .inbox
        .iter()
        .find(|word| word.from == tasks::Party::Task(change) && matches!(word.kind, tasks::MessageKind::Result(_)))
        .expect("change result kept after its holds are released")
        .clone();
    if cancelled {
        assert_eq!(result.kind, tasks::MessageKind::Result(tasks::ResultKind::Cancelled));
    } else {
        assert!(matches!(result.kind, tasks::MessageKind::Result(tasks::ResultKind::Change { .. })));
        assert!(String::from_utf8_lossy(&result.words).contains("Landed:"));
    }
    assert!(
        world
            .delivered_messages
            .get(&(run.task, run.attempt))
            .is_some_and(|messages| messages.contains(&result.number)),
        "change {change}'s result is relayed to the running cut"
    );
    let turn = live_task(world, run.task).turn;
    world.scripted_read_inbox(run);
    let row = live_task(world, run.task);
    assert!(row.turn > turn, "the next reading turn takes the result");
    assert!(!row.inbox.iter().any(|word| word.number == result.number));
}

fn delegate(world: &mut World, first: &engine::Assignment) -> engine::Assignment {
    let mut delegate =
        temper_engine_domain_world::fixtures::report_delegate(b"Ask the goal about its cut", Box::new([]));
    delegate.authority = live_task(world, first.task).authority.clone();
    delegate.authority.budget.spend = 10_000; // Covers the scripted delegate's reserved completion.
    delegate.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let answer = world.scripted_call(first, DELEGATE_CALL, engine::Tool::Delegate { batch: Box::new([delegate]) });
    let engine::CallAnswer::Delegated(made) = answer else { panic!("delegate admitted: {answer:?}") };
    let child = made[0].task;
    world.until(Until::SecondAssignment);
    world.assigned.iter().find(|run| run.task == child).expect("delegate assigned").clone()
}

#[test]
fn a_tasks_words_to_a_coordinator_are_offered_to_the_run_they_wake() {
    let ReferenceGoal { mut world, first, results, .. } = reference_goal();
    let delegate = delegate(&mut world, &first);
    let turns = live_task(&world, first.task).turn;
    park_after_turn(&mut world, &first, None, turns);
    world.until(Until::Ready);
    assert_eq!(live_task(&world, first.task).phase, tasks::Phase::Active(tasks::Active::Idle));

    let answer = world.scripted_call(
        &delegate,
        QUESTION_CALL,
        engine::Tool::Message { target: first.task, form: engine::MessageForm::Words, words: QUESTION.into() },
    );
    let engine::CallAnswer::Sent { message } = answer else { panic!("delegate words sent: {answer:?}") };
    world.until(Until::Ready);
    let next = world
        .assigned
        .iter()
        .find(|run| run.task == first.task && run.attempt > first.attempt)
        .expect("task's words wake the idle goal")
        .clone();
    assert!(live_task(&world, first.task).offered.contains(tasks::Reasons::WORDS));
    let offered = next.messages.iter().find(|word| word.number == message).expect("waking words offered");
    assert_eq!(offered.kind, tasks::MessageKind::Words);
    assert_eq!(offered.from, tasks::Party::Task(delegate.task));
    assert_eq!(offered.words.as_ref(), QUESTION);
    assert_eq!(next.run.policy.instructions.as_ref(), results);
    assert!(
        String::from_utf8_lossy(&next.run.policy.instructions)
            .contains("**Words:** read them; act on what is yours, and note what is news.")
    );

    let runs = world.assigned.len();
    world.scripted_read_inbox(&next);
    let turns = live_task(&world, next.task).turn;
    park_after_turn(&mut world, &next, None, turns);
    world.until(Until::Ready);
    for _ in 0..20 {
        world.tick();
    }
    assert_eq!(world.assigned.len(), runs, "the same words cause no second wake");
    assert_eq!(live_task(&world, first.task).phase, tasks::Phase::Active(tasks::Active::Idle));
    world.close();
}
