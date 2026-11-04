//! A goal holds its own branch through the root, core and independent forge
//! (temper/docs/design/domain/forge.md, 2 and 11.1;
//! jig/docs/design/domain/tasks.md, 6.1–6.3).
use super::*;

const FORGE_CONNECTOR: u16 = 0;
const OWN_GOAL_BRANCH: u16 = 1; // forge.md, 2, Symbols.
const GOAL_REQUEST: u8 = 92;
const CHAT_REQUEST: u8 = 93;
const CHANGE_CALL: u64 = 94;
const STEPS: u32 = 1000; // The change world's bound for a full landing.
const FIRST_RUN: usize = 0;
const PRODUCER_RUN: usize = 1;
const ONE_MERGE: u32 = 1;
const GOAL_BUDGET: u64 = 300; // Leaves 200 of the owner's 500 units for an outside chat.
const REFERENCE_BUDGET: u64 = 2_500_000;

fn reference_template() -> jig_domain_core::Template {
    use jig_client::contract as c;
    let limits = temper_client_tools::limits::tools();
    let decoded = temper_client_tools::jig::settings_file::decode(
        include_bytes!("../../../../../../docs/examples/reference/settings.json"),
        &limits,
    )
    .expect("shipped reference settings");
    let mut item = decoded
        .into_boxed()
        .into_vec()
        .into_iter()
        .find(|item| item.setting == c::Setting::Template(c::SettingNumber { number: 4 }))
        .expect("reference template 4");
    let c::SettingEdit::Set(c::SettingSet { value: c::SettingValue::Template(template) }) = &mut item.edit else {
        panic!("reference template value")
    };
    template.llm = None; // The scripted worker uses the fixture's model.
    let mut items = skein_lib::List::with_capacity(1);
    items.push(item).expect("one template");
    let ask = c::Ask::ChangeSettings(c::ChangeSettings { against: 1, items, running: c::Running::Stays });
    let jig_client_protocol_wire::Ask::ChangeSettings(change) =
        jig_client_protocol_link::translate::asks::ask(&ask, &limits.settings).expect("settings wire")
    else {
        panic!("settings change")
    };
    let items = jig_protocol_engine_clients::translate::settings::items(change.items().into(), &[], &limits.settings)
        .expect("reference template reaches the core");
    let jig_domain_core::settings::Edit::Set(jig_domain_core::settings::Value::Template(template)) =
        items.into_vec().pop().expect("one template").edit
    else {
        panic!("core template value")
    };
    *template
}

fn reference_defaults() -> jig_domain_core::settings::Defaults {
    let llm = jig_protocol_config::Llm {
        endpoints: Box::new([]),
        providers: Box::new([]),
        descriptions: Box::new([None]),
        models: Box::new([stock::box_model()]),
    };
    let mut defaults = temper_service::stock::defaults(&llm);
    // Admission needs charter 5; the scripted world takes the stock run policy,
    // whose model and document requirements fit its fake worker and repository.
    let mut coordinator = defaults.charters[0].clone();
    coordinator.number = 5;
    coordinator.documents = Box::new([]);
    defaults.charters = [defaults.charters.as_ref(), &[coordinator]].concat().into();
    defaults.templates = [defaults.templates.as_ref(), &[reference_template()]].concat().into();
    defaults.rules.period_spend = REFERENCE_BUDGET;
    reference_authority(&mut defaults.rules.ceiling);
    let policy = defaults.policies.get_mut(&1).expect("stock project policy");
    policy.policy.period_spend = REFERENCE_BUDGET;
    reference_authority(&mut policy.policy.ceiling);
    for role in &mut policy.policy.roles {
        reference_authority(&mut role.authority);
        role.period_spend = REFERENCE_BUDGET;
    }
    defaults
}

fn reference_authority(authority: &mut authority::Authority) {
    authority.budget.spend = REFERENCE_BUDGET;
    authority.delegation.kinds =
        [authority.delegation.kinds.as_ref(), &[authority::Executor::Charter(5)]].concat().into();
}

fn reference_request(world: &World, template: &jig_domain_core::Template) -> people::Ask {
    let people::Ask::Create { project, mut batch } = world.chat(1, b"TEXT") else { panic!("goal batch") };
    let people::TemplateExecutor::Charter(charter) = template.executor else { panic!("coordinator charter") };
    let tasks::Contract::Report { words } = template.contract else { panic!("goal report") };
    assert_eq!(template.budget, people::TemplateBudget::Given);
    assert_eq!(template.deadline, people::TemplateDeadline::Given);
    let member = &mut batch[0];
    member.executor = people::TaskExecutor::Agent { charter };
    member.contract = people::Contract::Report { words };
    member.budget = people::Budget::Named;
    member.authority.spend = REFERENCE_BUDGET;
    // 2026-11-04T13:00Z, the reference walk's deadline.
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
    assert!(template.llm.is_none());
    member.llm = None;
    member.symbols = template
        .resources
        .iter()
        .map(|symbol| people::Symbol { connector: symbol.connector, symbol: symbol.symbol })
        .collect();
    people::Ask::Create { project, batch }
}

#[test]
fn reference_template_four_starts_on_the_configured_forge() {
    let defaults = reference_defaults();
    let mut limits = temper_engine_domain_world::forge_routes::box_limits();
    limits.documents.lines = 8;
    let mut world = World::configured_stock_limits(&defaults, &stock::box_model(), &limits);
    world.adopt();
    let mut event = protocol::hello(Token::new(7), Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut event
    else {
        panic!("worker hello")
    };
    hello.bounds = jig_test_profile::fixtures::bounds();
    let model = stock::box_model();
    hello.bounds.models = Box::new([host::ModelBound { name: model.name, window: model.window, output: model.output }]);
    world.send(event);
    let template = defaults.templates.iter().find(|template| template.number == 4).expect("template 4");
    let request = reference_request(&world, template);
    let outcome = ask(&mut world, GOAL_REQUEST, request);
    let people::Outcome::Created { first: goal, last, .. } = outcome else {
        panic!("reference goal must be admitted: {outcome:?}")
    };
    assert_eq!(goal, last);
    world.until(Until::Ready);
    let branch = format!("temper/{goal}/g{goal}");
    let mut held = resource(branch.as_bytes());
    held.connector = 1;
    assert!(live_task(&world, goal).holdings.contains(&tasks::Holding::Write { resource: held, kind: 1 }));
    assert_eq!(forge_hold(&world, branch.as_bytes()).expect("exclusive forge hold").task, goal);
    assert_eq!(branch_tip(&mut world, branch.as_bytes()), branch_tip(&mut world, b"main"));
    world.close();
}

pub(super) fn adopted_world() -> World {
    let mut world = World::configured(true, true);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    world
}

pub(super) fn goal_request(world: &World, tracked: bool) -> people::Ask {
    let people::Ask::Create { project, mut batch } = world.chat(1, b"Make a small fix on my branch") else {
        unreachable!("chat template creates a batch")
    };
    let member = &mut batch[0];
    member.tracked = tracked.then_some(0);
    member.label = 2;
    member.flags = people::Flags { listed: true, ..people::Flags::PLAIN };
    member.contract = people::Contract::Report { words: 128 };
    member.budget = people::Budget::AsAllowed { cap: Some(GOAL_BUDGET) };
    member.symbols = Box::new([people::Symbol { connector: FORGE_CONNECTOR, symbol: OWN_GOAL_BRANCH }]);
    people::Ask::Create { project, batch }
}

pub(super) fn running_goal() -> (World, engine::Assignment, Box<[u8]>) {
    let mut world = adopted_world();
    let ask = goal_request(&world, true);
    let goal = assigned_chat(&mut world, GOAL_REQUEST, ask);
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    world.until(Until::Ready);
    (world, goal, branch)
}

pub(super) fn resource(branch: &[u8]) -> tasks::Name {
    tasks::Name {
        connector: FORGE_CONNECTOR,
        path: Box::new([
            forge_world::REPO.forge.to_be_bytes().into(),
            forge_world::REPO.repository.to_be_bytes().into(),
            branch.into(),
        ]),
    }
}

pub(super) fn branch_tip(world: &mut World, branch: &[u8]) -> u64 {
    let answer = world.external(1, raw::Op::Read(raw::Read::Branch { branch: branch.into() }));
    let raw::Answer::Commit(tip) = answer else { panic!("branch {branch:?}: {answer:?}") };
    tip
}

fn ask(world: &mut World, key: u8, ask: people::Ask) -> people::Outcome {
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: world.signed_in.expect("owner session"),
        key: [key; 16],
        ask,
    }));
    for _ in 0..STEPS {
        world.tick();
        if let Some(outcome) =
            world.store.rows.values().find_map(|row| match row {
                Record::Core(jig_domain_core::Record::People(people::Stored::Answer {
                    key: request, outcome, ..
                })) if request.key == [key; 16] => Some(outcome.clone()),
                _ => None,
            })
        {
            return outcome;
        }
    }
    panic!("person request {key} did not answer")
}

pub(super) fn change(
    world: &mut World,
    run: &engine::Assignment,
    landing: engine::ChangeLanding,
) -> engine::CallAnswer {
    let before = world.answers.len();
    let event = world.protocol.call(
        Token::new(7),
        run.task,
        run.attempt,
        CHANGE_CALL,
        engine::Call {
            completion: 1,
            position: 1,
            tool: engine::Tool::Change {
                changes: Box::new([engine::ChangeCall {
                    landing: Some(landing),
                    ..engine::ChangeCall::words(b"Small fix".as_slice().into(), None)
                }]),
            },
        },
    );
    world.send(event);
    for _ in 0..STEPS {
        world.tick();
        if let Some(answer) = world.answers.get(before) {
            return answer.clone();
        }
    }
    panic!("change did not answer: {:?}", world.call_errors)
}

pub(super) fn forge_hold<'a>(world: &'a World, branch: &[u8]) -> Option<&'a forge_top::Hold> {
    let what = forge_top::What::Branch(branch.split(|byte| *byte == b'/').map(Box::from).collect());
    world.store.rows.values().find_map(|row| match row {
        Record::Forge { row } => match row.as_ref() {
            forge_top::Stored::Hold(hold)
                if hold.name.forge == forge_world::REPO.forge
                    && hold.name.repository == forge_world::REPO.repository
                    && hold.name.what == what =>
            {
                Some(hold)
            }
            _ => None,
        },
        Record::Core(_) => None,
    })
}

pub(super) fn writer(world: &World, branch: &[u8]) -> Option<tasks::Writer> {
    let resource = resource(branch);
    world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Writer(slot))) if slot.resource == resource => {
                Some(slot.writer)
            }
            _ => None,
        })
        .or_else(|| match forge_hold(world, branch)?.writer? {
            forge_top::Writer::Entry(entry) => Some(tasks::Writer::Effect { entry }),
            forge_top::Writer::Run { task, attempt } | forge_top::Writer::Lost { task, attempt } => {
                Some(tasks::Writer::Run { task, attempt })
            }
        })
}

#[test]
fn a_tracked_goal_naming_its_own_goal_branch_holds_it_made_at_mains_tip() {
    let (mut world, goal, branch) = running_goal();
    let row = live_task(&world, goal.task);
    assert_eq!(row.root, goal.task);
    assert_eq!(row.tracked, Some(0));
    assert!(row.holdings.contains(&tasks::Holding::Write { resource: resource(&branch), kind: 1 }));
    assert!(matches!(row.phase, tasks::Phase::Active(tasks::Active::Claimed { .. } | tasks::Active::Running { .. })));
    assert_eq!(world.assigned[FIRST_RUN].task, goal.task);
    let main = branch_tip(&mut world, b"main");
    assert_eq!(branch_tip(&mut world, &branch), main);
    assert_eq!(writer(&world, &branch), None, "the goal's inspecting run takes no writer slot");
    world.close();
}

#[test]
fn an_untracked_task_naming_its_own_goal_branch_is_refused() {
    let mut world = adopted_world();
    let before = world.external(1, raw::Op::Git(raw::Git::Clone));
    let request = goal_request(&world, false);
    assert_eq!(
        ask(&mut world, GOAL_REQUEST, request),
        people::Outcome::Refused(people::Refusal::Limit { which: people::LimitKind::HoldKind })
    );
    world.until(Until::Ready);
    assert_eq!(world.external(1, raw::Op::Git(raw::Git::Clone)), before, "no branch is made");
    assert!(world.assigned.is_empty());
    world.close();
}

#[test]
fn a_change_into_a_goal_branch_from_outside_its_goal_is_refused_naming_it() {
    let (mut world, goal, branch) = running_goal();
    let request = adoption_chat(&world);
    let chat = assigned_chat(&mut world, CHAT_REQUEST, request);
    let answer = change(&mut world, &chat, engine::ChangeLanding::Branch(branch.clone()));
    let engine::CallAnswer::DelegationRefused(problem) = answer else { panic!("refused change: {answer:?}") };
    assert_eq!(problem.why, tasks::Refusal::HoldTaken);
    assert_eq!(problem.blocked_by.as_deref(), Some([goal.task].as_slice()));
    assert!(
        live_task(&world, goal.task).holdings.contains(&tasks::Holding::Write { resource: resource(&branch), kind: 1 })
    );
    assert!(!world.store.rows.keys().any(|key| matches!(key, Key::Forge(forge_top::Key::Change(_)))));
    world.close();
}

#[test]
fn a_goals_end_keeps_its_own_branch() {
    let (mut world, goal, branch) = running_goal();
    let tip = branch_tip(&mut world, &branch);
    let event = world.protocol.answer(
        Token::new(7),
        goal.task,
        goal.attempt,
        1,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"Done".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    let ended = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(goal.task)));
    for _ in 0..STEPS {
        world.tick();
        if world.store.rows.contains_key(&ended) {
            break;
        }
    }
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) = world.store.rows.get(&ended)
    else {
        panic!("goal ended")
    };
    assert!(matches!(row.phase, tasks::Phase::Ended(tasks::Ending::Done(_))));
    world.until(Until::Ready);
    assert!(!world.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(goal.task)))));
    assert!(forge_hold(&world, &branch).is_none(), "ending releases the forge hold");
    assert_eq!(writer(&world, &branch), None);
    assert_eq!(branch_tip(&mut world, &branch), tip, "the branch survives at its original tip");
    world.close();
}

#[test]
fn a_change_by_the_goal_into_its_own_branch_lands_there_taking_its_writer_slot() {
    let (mut world, goal, branch) = running_goal();
    let answer = change(&mut world, &goal, engine::ChangeLanding::Goal);
    let engine::CallAnswer::Delegated(made) = answer else { panic!("goal change admitted: {answer:?}") };
    let change = made[FIRST_RUN].task;
    world.until(Until::SecondAssignment);
    let producer = world.assigned[PRODUCER_RUN].clone();
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(change))) else {
        panic!("registered change")
    };
    let forge_top::Stored::Change(row) = row.as_ref() else { panic!("change row") };
    assert_eq!(row.base, branch);
    let change_branch = row.branch.clone();
    let main = branch_tip(&mut world, b"main");
    world.hold_write_answers = Some(HoldWrite::Merge);
    change_tool::land_change(&mut world, &producer, &change_branch);
    for _ in 0..STEPS {
        if !world.held_write_answers.is_empty() {
            break;
        }
        world.tick();
    }
    let entry = world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            Record::Forge { row } => match row.as_ref() {
                forge_top::Stored::Entry { entry, .. }
                    if entry.task == change && matches!(entry.effect.write, skein_forge::api::Write::Merge { .. }) =>
                {
                    Some(entry.number)
                }
                _ => None,
            },
            Record::Core(_) => None,
        })
        .expect("in-flight merge entry");
    assert_eq!(writer(&world, &branch), Some(tasks::Writer::Effect { entry }));
    world.release_write_answers();
    world.until(Until::Ready);
    assert_eq!(world.merges, ONE_MERGE);
    assert_eq!(writer(&world, &branch), None, "the settled merge frees its slot");
    assert_ne!(branch_tip(&mut world, &branch), main, "the change advances the goal's branch");
    assert_eq!(branch_tip(&mut world, b"main"), main);
    world.close();
}
