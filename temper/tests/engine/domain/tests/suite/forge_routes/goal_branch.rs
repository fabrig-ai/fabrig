//! A goal holds its own branch through the root, core and independent forge
//! (temper/docs/design/domain/forge/README.md, 2, and temper/docs/design/domain/forge/branches.md, 1;
//! jig/docs/design/domain/tasks/holds.md, 1–3).
use super::*;
use jig_domain_link::EndpointBound;

const FORGE_CONNECTOR: u16 = 0;
const OWN_GOAL_BRANCH: u16 = 1; // temper/docs/design/domain/forge/README.md, 2, Symbols.
const GOAL_REQUEST: u8 = 92;
const CHAT_REQUEST: u8 = 93;
const CHANGE_CALL: u64 = 94;
const STEPS: u32 = 1000; // The change world's bound for a full landing.
const FIRST_RUN: usize = 0;
const PRODUCER_RUN: usize = 1;
const ONE_MERGE: u32 = 1;
const GOAL_BUDGET: u64 = 300; // Leaves 200 of the owner's 500 units for an outside chat.
// The stock coordinator's 11K allowance at the fixture's Sol prices (K = 8,720).
const STOCK_GOAL_BUDGET: u64 = 95_920;

pub(super) fn stock_defaults() -> jig_domain_core::settings::Defaults {
    let catalogue = jig_domain_core::settings::CatalogueModel {
        first_read: None,
        number: 1,
        model: stock::box_model(),
        shown: None,
        description: None,
        picker: true,
        withdrawn: false,
        source: jig_domain_core::settings::ModelSource::Stock,
        estimated: false,
    };
    temper_service::stock::with_models(
        Box::new([catalogue]),
        temper_engine_domain_world::forge_routes::box_limits().tasks.result_bytes,
    )
}

fn stock_request(world: &World, template: &jig_domain_core::Template) -> people::Ask {
    let people::Ask::Create { project, mut batch } = world.chat(1, b"TEXT") else { panic!("goal batch") };
    let people::TemplateExecutor::Charter(charter) = template.executor else { panic!("coordinator charter") };
    let tasks::Contract::Report { words } = template.contract else { panic!("goal report") };
    assert_eq!(template.budget, people::TemplateBudget::Given);
    assert_eq!(template.deadline, people::TemplateDeadline::Given);
    let member = &mut batch[0];
    member.executor = people::TaskExecutor::Agent { charter };
    member.contract = people::Contract::Report { words };
    member.budget = people::Budget::Named;
    member.authority.spend = STOCK_GOAL_BUDGET;
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

fn stock_goal_world() -> (World, people::Ask) {
    let defaults = stock_defaults();
    let mut limits = temper_engine_domain_world::forge_routes::box_limits();
    limits.documents.lines = 8;
    limits.forge.change_policy.questions = 1;
    let mut world = World::configured_stock_limits(&defaults, &stock::box_model(), &limits)
        .with_settings_owner(&forge_top::user_key(1, 7));
    world.adopt();
    let mut event = protocol::hello_slots(Token::new(7), 3, Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut event
    else {
        panic!("worker hello")
    };
    hello.bounds = jig_test_profile::fixtures::bounds();
    hello.bounds.turns = 800;
    hello.bounds.time = Duration::from_secs(28_800);
    let model = stock::box_model();
    hello.bounds.endpoints =
        Box::new([EndpointBound { endpoint: model.endpoint, window: model.window, output: model.output }]);
    world.send(event);
    let template = defaults.templates.iter().find(|template| template.number == 4).expect("template 4");
    let request = stock_request(&world, template);
    (world, request)
}

/// Start the stock draft-goal template with its held branch.
pub(super) fn running_stock_goal() -> (World, engine::Assignment, Box<[u8]>) {
    let (mut world, request) = stock_goal_world();
    let outcome = ask(&mut world, GOAL_REQUEST, request);
    let people::Outcome::Created { first: goal, last, .. } = outcome else {
        panic!("stock goal must be admitted: {outcome:?}")
    };
    assert_eq!(goal, last);
    world.until(Until::Ready);
    let assigned = world.assigned.iter().find(|run| run.task == goal).expect("stock coordinator starts").clone();
    let branch = format!("temper/{goal}/g{goal}").into_bytes().into_boxed_slice();
    (world, assigned, branch)
}

#[test]
fn the_stock_template_four_starts_on_the_configured_forge() {
    let (mut world, assigned, branch) = running_stock_goal();
    let goal = assigned.task;
    assert_eq!(assigned.charter, 5);
    let mut held = resource(&branch);
    held.connector = 1;
    assert!(live_task(&world, goal).holdings.contains(&tasks::Holding::Write { resource: held, kind: 1 }));
    assert_eq!(forge_hold(&world, &branch).expect("exclusive forge hold").task, goal);
    assert_eq!(branch_tip(&mut world, &branch), branch_tip(&mut world, b"main"));
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
fn a_modifying_goals_first_run_waits_for_making_and_writes_design() {
    let (mut world, request) = stock_goal_world();
    world.hold_write_answers = Some(HoldWrite::All);
    let people::Outcome::Created { first: task, last, .. } = ask(&mut world, GOAL_REQUEST, request) else {
        panic!("goal admitted")
    };
    assert_eq!(task, last);
    let branch = format!("temper/{task}/g{task}").into_bytes().into_boxed_slice();
    for _ in 0..STEPS {
        world.tick();
        if world.writes.iter().any(|write| {
            matches!(write,
            skein_forge::api::Write::CreateBranch { branch: made, .. } if *made == branch)
        }) {
            break;
        }
    }
    assert!(world.writes.iter().any(|write| matches!(write,
        skein_forge::api::Write::CreateBranch { branch: made, .. } if *made == branch)));
    for _ in 0..20 {
        world.tick();
    }
    assert!(world.assigned.is_empty(), "the first run must not start on the base while g is making");
    let hold = forge_hold(&world, &branch).expect("making goal hold");
    assert!(matches!(hold.writer, Some(forge_top::Writer::Entry(_))));
    assert!(hold.design_writers.is_empty());
    world.release_write_answers();
    let goal = reviews::next_assignment(&mut world, 0);
    assert_eq!(goal.task, task);
    let item = &goal.workspace.repositories[0];
    assert_eq!(item.start, engine::ForgeStart::Base(branch.clone()));
    assert_eq!(item.push.as_deref(), Some(branch.as_ref()));
    assert_eq!(item.landing.as_deref(), Some(branch.as_ref()));
    assert!(item.delivers);
    let mut goal_resource = resource(&branch);
    goal_resource.connector = 1;
    assert!(
        !world.store.rows.values().any(|row| matches!(row,
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Writer(slot))) if slot.resource == goal_resource)),
        "design takes no jig slot"
    );
    assert_eq!(forge_hold(&world, &branch).expect("confirmed goal hold").writer, None);
    assert_eq!(
        forge_hold(&world, &branch).expect("confirmed goal hold").design_writers.as_ref(),
        &[forge_top::DesignWriter { task, attempt: goal.attempt, rereading: false }]
    );
    let charter = temper_engine_smith::charter(&goal);
    assert!(charter.grants.deliver.is_some(), "the first run can deliver its first-wave design");
    let home = goal
        .sections
        .iter()
        .find(|section| section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Home))
        .expect("design delivery line");
    assert_eq!(home.body, engine::BriefBody::Text(b"deliver pushes the checkout's Markdown straight to the goal branch, merging its moved tip first; code goes through change.".as_slice().into()));
    early_start::stop(&mut world, &goal);
}

#[test]
fn releasing_a_goals_failed_making_reads_and_makes_its_branch_again() {
    for (failed_write, present) in [(false, false), (false, true), (true, false), (true, true)] {
        let mut world = adopted_world();
        let request = goal_request(&world, true);
        let people::Outcome::Created { first: goal, .. } = ask(&mut world, GOAL_REQUEST, request) else {
            panic!("goal created")
        };
        let mut refused = false;
        for _ in 0..STEPS {
            for event in &mut world.events {
                if let engine::Event::Forge(event) = event
                    && let forge_top::Event::Client(skein_forge_client::Event::Answered { result, .. }) = event.as_mut()
                    && ((!failed_write && *result == Err(skein_forge::api::Error::Missing))
                        || (failed_write && matches!(result, Ok(skein_forge::api::Answer::Branch(_)))))
                {
                    // A terminal refusal at the read or creation boundary.
                    *result = Err(skein_forge::api::Error::Forbidden);
                    refused = true;
                }
            }
            world.tick();
            if matches!(live_task(&world, goal).phase, tasks::Phase::Held { why: tasks::Hold::EffectFailed, .. }) {
                break;
            }
        }
        assert!(refused, "the goal branch read or creation was refused");
        assert!(matches!(live_task(&world, goal).phase, tasks::Phase::Held { why: tasks::Hold::EffectFailed, .. }));
        world.until(Until::Ready);
        let branch = format!("temper/{goal}/g{goal}").into_bytes().into_boxed_slice();
        if failed_write {
            world.external(2, raw::Op::Write(raw::Write::DeleteBranch { branch: branch.clone() }));
        }
        let tip = if present {
            world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
            fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"outside", b"present", 2)
                .expect("existing branch")
        } else {
            branch_tip(&mut world, b"main")
        };
        let revision = live_task(&world, goal).revisions.hold;
        ask(&mut world, CHAT_REQUEST, people::Ask::Release { project: 1, task: goal, revision, again: None });
        world.until(Until::Ready);
        assert_eq!(branch_tip(&mut world, &branch), tip, "a present branch is never reset");
        assert!(!matches!(live_task(&world, goal).phase, tasks::Phase::Held { .. }));
        assert_eq!(forge_hold(&world, &branch).expect("goal's hold").writer, None);
        world.close();
    }
}

#[test]
fn a_goal_without_push_authority_inspects_g_without_a_delivery_or_writer() {
    let mut world = adopted_world();
    let people::Ask::Create { project, mut batch } = goal_request(&world, true) else { panic!("goal batch") };
    batch[0].authority.grants = batch[0].authority.grants.iter().filter(|grant| grant.kind != 2).cloned().collect();
    let goal = assigned_chat(&mut world, GOAL_REQUEST, people::Ask::Create { project, batch });
    world.until(Until::Ready);
    let branch = format!("temper/{}/g{}", goal.task, goal.task);
    let run_branch = format!("temper/{}/r{}", goal.task, goal.task);
    assert!(goal.run.policy.modify);
    assert!(!goal.run.authority.grants.iter().any(|grant| grant.kind == 2));
    let repository = &goal.workspace.repositories[0];
    assert_eq!(repository.start, engine::ForgeStart::Branch(branch.as_bytes().into()));
    assert_eq!(repository.push, None);
    assert_eq!(repository.landing, None);
    assert!(
        !goal.sections.iter().any(|section| section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Home)),
        "a read-only run has no design delivery line"
    );
    assert!(!repository.delivers);
    assert_eq!(writer(&world, branch.as_bytes()), None);
    assert_eq!(writer(&world, run_branch.as_bytes()), None);
    let charter = temper_engine_smith::charter(&goal);
    assert!(charter.grants.deliver.is_none());
    assert!(!charter.grants.tools.modify);
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
    assert_eq!(world.assigned.len(), 0);
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

fn seeded_goal() -> (World, engine::Assignment, Box<[u8]>) {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits {
            slots: Some(3),
            result_bytes: Some(tasks::HOLD_ACCOUNT_BYTES),
            stall: Some(Duration::from_secs(300)),
            ..StoryLimits::default()
        },
    )
    .with_goal_question_capacity(12, 4096);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 3, Box::new([])));
    let request = goal_request(&world, true);
    let goal = assigned_chat(&mut world, GOAL_REQUEST, request);
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    world.until(Until::Ready);
    let engine::CallAnswer::Delegated(warmup) = change_tool::change_landing(
        &mut world,
        &goal,
        CHANGE_CALL,
        None,
        b"Seed fix",
        Some(engine::ChangeLanding::Goal),
    ) else {
        panic!("seed change admitted")
    };
    let seed = super::reviews::next_assignment(&mut world, 1);
    let seed_branch = super::goal_landing::row(&world, warmup[0].task).branch.clone();
    change_tool::land_change(&mut world, &seed, &seed_branch);
    world.until(Until::Ready);
    world.scripted_read_inbox(&goal);
    (world, goal, branch)
}

fn merging_siblings() -> (World, engine::Assignment, Box<[u8]>, u64) {
    let (mut world, goal, branch) = seeded_goal();
    let first_answer = change_tool::change_landing(
        &mut world,
        &goal,
        CHANGE_CALL + 1,
        None,
        b"First fix",
        Some(engine::ChangeLanding::Goal),
    );
    let engine::CallAnswer::Delegated(first) = first_answer else { panic!("first change admitted: {first_answer:?}") };
    let producer = super::reviews::next_assignment(&mut world, 2);
    let engine::CallAnswer::Delegated(second) = change_tool::change_landing(
        &mut world,
        &goal,
        CHANGE_CALL + 2,
        None,
        b"Sibling fix",
        Some(engine::ChangeLanding::Goal),
    ) else {
        panic!("sibling admitted")
    };
    let sibling = super::reviews::next_assignment(&mut world, 3);
    assert_ne!(producer.task, sibling.task);
    let first_branch = super::goal_landing::row(&world, first[0].task).branch.clone();
    world.merges = 0;
    world.hold_write_answers = Some(HoldWrite::Merge);
    change_tool::land_change(&mut world, &producer, &first_branch);
    for _ in 0..STEPS {
        if !world.held_write_answers.is_empty() {
            break;
        }
        world.tick();
    }
    assert_eq!(world.held_write_answers.len(), 1, "fake forge applied the merge");
    (world, goal, branch, second[0].task)
}

fn hint_goal_branch(world: &mut World, branch: &[u8]) {
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::Branch(branch.into()),
            key: None,
        },
    })));
}

pub(super) fn observe_goal_branch(world: &mut World, branch: &[u8]) {
    hint_goal_branch(world, branch);
    for _ in 0..200 {
        world.tick();
    }
}

/// Delay a keep observation without reserving a procedure's fresh-read chain.
pub(super) fn start_slow_keep_read(world: &mut World, branch: &[u8]) -> Token {
    for _ in 0..STEPS {
        world.tick();
        let deciding = world.store.rows.values().any(|record| {
            matches!(record,
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))
                if matches!(row.executor, tasks::Executor::Procedure { .. })
                && matches!(row.phase, tasks::Phase::Active(tasks::Active::Due)))
        });
        if !deciding
            && world.root.quiescent()
            && world.events.is_empty()
            && world.store.pending.is_empty()
            && world.pending.is_empty()
        {
            break;
        }
    }
    assert!(world.pending.is_empty(), "prior forge reads settle before delaying keep");
    world.slow_branch = Some(branch.into());
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Client(skein_forge_client::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::Branch(branch.into()),
            key: None,
        },
    }))));
    for _ in 0..STEPS {
        if let Some(call) = world.pending.iter().find_map(|(call, op)| match op {
            skein_forge::api::Op::Read(skein_forge::api::Read::Branch { branch: read }) if read.as_ref() == branch => {
                Some(*call)
            }
            _ => None,
        }) {
            world.slow_branch = None;
            return call;
        }
        world.tick();
    }
    panic!("the keep read must start before the merge")
}

#[test]
fn a_merge_into_the_goal_branch_observed_before_its_answer_is_not_drift() {
    let (mut world, goal, branch, sibling) = merging_siblings();
    let main = branch_tip(&mut world, b"main");
    assert_ne!(branch_tip(&mut world, &branch), main);
    observe_goal_branch(&mut world, &branch);
    assert!(
        !matches!(live_task(&world, goal.task).phase, tasks::Phase::Held { .. }),
        "own merge must not hold the goal: {:?}",
        live_task(&world, goal.task).phase
    );
    assert!(
        !matches!(live_task(&world, sibling).phase, tasks::Phase::Held { .. }),
        "sibling: {:?}",
        live_task(&world, sibling).phase
    );
    assert!(forge_hold(&world, &branch).expect("goal branch hold").drift.is_none());
    world.release_write_answers();
    world.until(Until::Ready);
    observe_goal_branch(&mut world, &branch);
    assert_eq!(writer(&world, &branch), None, "confirmed merge frees its writer");
    assert_eq!(branch_tip(&mut world, b"main"), main);
    finish_siblings(&mut world, &goal);
}

fn finish_current_task(world: &mut World, task: u64, result: &tasks::TaskResult) {
    let ended = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)));
    let mut answered = None;
    for _ in 0..STEPS {
        world.tick();
        if world.store.rows.contains_key(&ended) {
            return;
        }
        if let Some(run) = world.assigned.iter().rev().find(|run| run.task == task)
            && answered != Some(run.attempt)
        {
            let attempt = run.attempt;
            let event = world.protocol.answer(
                Token::new(7),
                task,
                attempt,
                9,
                tasks::End::Finished { result: result.clone(), cancel_delegates: false },
                engine::Parts::none(),
            );
            world.send(event);
            answered = Some(attempt);
        }
    }
    panic!("task {task} did not finish: {:?}", live_task(world, task).phase)
}

fn finish_siblings(world: &mut World, goal: &engine::Assignment) {
    let sibling = world.assigned.iter().rev().find(|run| run.task != goal.task).expect("sibling producer").task;
    finish_current_task(world, sibling, &tasks::TaskResult::NoChange { words: b"No further change".as_slice().into() });
    finish_current_task(world, goal.task, &tasks::TaskResult::Report { words: b"Done".as_slice().into() });
    world.until(Until::Ready);
    world.close();
}

#[test]
fn a_merge_answer_before_the_goal_branch_observation_is_not_drift() {
    let (mut world, goal, branch, sibling) = merging_siblings();
    let main = branch_tip(&mut world, b"main");
    world.release_write_answers();
    world.until(Until::Ready);
    observe_goal_branch(&mut world, &branch);
    assert!(!matches!(live_task(&world, goal.task).phase, tasks::Phase::Held { .. }));
    assert!(!matches!(live_task(&world, sibling).phase, tasks::Phase::Held { .. }));
    assert!(forge_hold(&world, &branch).expect("goal branch hold").drift.is_none());
    assert_eq!(writer(&world, &branch), None);
    assert_eq!(branch_tip(&mut world, b"main"), main);
    finish_siblings(&mut world, &goal);
}

#[derive(Clone, Copy)]
enum BranchMove {
    Outside,
    Backwards,
    Deleted,
    Unowned,
}

fn goal_drift(movement: BranchMove) {
    let (mut world, goal, branch, _) = merging_siblings();
    if matches!(movement, BranchMove::Unowned) {
        world.release_write_answers();
        world.until(Until::Ready);
        super::park_chat(&mut world, &goal);
        for _ in 0..STEPS {
            world.tick();
            if forge_hold(&world, &branch).expect("goal hold").design_writers.is_empty() {
                break;
            }
        }
        assert!(
            forge_hold(&world, &branch).expect("goal hold").design_writers.is_empty(),
            "no design writer remains for this move"
        );
    }
    observe_goal_branch(&mut world, &branch);
    let expected = branch_tip(&mut world, &branch);
    let observed = match movement {
        BranchMove::Outside | BranchMove::Unowned => Some(
            fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"outside", b"moved", 2)
                .expect("outside push"),
        ),
        BranchMove::Backwards | BranchMove::Deleted => {
            world.external(1, raw::Op::Write(raw::Write::DeleteBranch { branch: branch.clone() }));
            if matches!(movement, BranchMove::Backwards) {
                world.external(1, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
                Some(1)
            } else {
                None
            }
        }
    };
    if matches!(movement, BranchMove::Outside) {
        world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
            hint: skein_forge::api::Hint {
                repository: forge_world::REPO,
                key: None,
                change: skein_forge::api::Change::BranchMoved {
                    branch: branch.clone(),
                    head: translate::commit(observed.expect("outside push has a head")),
                    actor: 2,
                },
            },
        })));
    }
    observe_goal_branch(&mut world, &branch);
    assert_drift_account(&world, goal.task, &branch, expected, observed);
    settle_drift(&mut world, &goal, movement);
}

fn settle_drift(world: &mut World, goal: &engine::Assignment, movement: BranchMove) {
    world.release_write_answers();
    world.until(Until::Ready);
    world.signed_in = None;
    world.sign_in();
    let revision = live_task(world, goal.task).revisions.hold;
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(101)),
        sign_in: world.signed_in.expect("owner"),
        key: [101; 16],
        ask: if matches!(movement, BranchMove::Deleted) {
            people::Ask::Cancel {
                project: 1,
                task: goal.task,
                reason: b"Keep the deletion".as_slice().into(),
                grace: None,
            }
        } else {
            people::Ask::Release { project: 1, task: goal.task, revision, again: None }
        },
    }));
    world.until(Until::Ready);
    assert!(
        !matches!(live_task(world, goal.task).phase, tasks::Phase::Held { .. }),
        "release: {:?}",
        world.web_replies
    );
    finish_siblings(world, goal);
}

fn assert_drift_account(world: &World, goal: u64, branch: &[u8], expected: u64, observed: Option<u64>) {
    let drift = forge_hold(world, branch).expect("goal branch hold").drift.as_ref().expect("outside move retained");
    assert_eq!(
        drift.change,
        forge_top::DriftChange::Branch {
            expected: translate::commit(expected),
            observed: observed.map(translate::commit),
        }
    );
    let tasks::Phase::Held { why: tasks::Hold::Drift, source: Some(source), .. } = &live_task(world, goal).phase else {
        panic!("goal retains drift account")
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("forge account") };
    let words = std::str::from_utf8(words).expect("hold text");
    assert!(words.contains(std::str::from_utf8(branch).expect("branch text")) && words.contains("org/repo"), "{words}");
    assert!(words.contains(&format!("at {} ns since Unix epoch", drift.at.as_nanos())), "{words}");
    assert!(
        words.contains(std::str::from_utf8(&engine::commit_text(&translate::commit(expected))).expect("head text")),
        "{words}"
    );
    let moved = observed.map_or_else(
        || "deletion".into(),
        |head| String::from_utf8(engine::commit_text(&translate::commit(head)).into()).expect("head text"),
    );
    assert!(words.contains(&format!("observed {moved}")), "{words}");
}

#[test]
fn an_outside_push_during_a_goal_merge_names_the_resource_and_move() {
    goal_drift(BranchMove::Outside);
}

#[test]
fn a_backward_move_during_a_goal_merge_names_the_resource_and_move() {
    goal_drift(BranchMove::Backwards);
}

#[test]
fn a_deletion_during_a_goal_merge_names_the_resource_and_move() {
    goal_drift(BranchMove::Deleted);
}

#[test]
fn a_forward_move_without_a_goal_writer_names_the_resource_and_move() {
    goal_drift(BranchMove::Unowned);
}

fn delayed_goal_ancestry(world: &mut World, branch: &[u8]) -> engine::Event {
    let head = translate::commit(branch_tip(world, branch));
    world.forge_reads = Some(Vec::new());
    hint_goal_branch(world, branch);
    for _ in 0..STEPS {
        world.tick();
        if let Some(index) = world.events.iter().position(|event| {
            matches!(event,
            engine::Event::Forge(event) if matches!(event.as_ref(), forge_top::Event::Client(
                skein_forge_client::Event::Answered { call, result: Ok(skein_forge::api::Answer::Commits { .. }), .. }
            ) if world.forge_reads.as_ref().expect("recorded reads").iter().any(|(asked, read)|
                asked == call && matches!(read, skein_forge::api::Read::Commits { before, page: 1, .. } if *before == head))))
        }) {
            return world.events.remove(index).expect("selected ancestry read");
        }
    }
    panic!("the keep reads the observed merge's ancestry before its confirmation")
}

#[test]
fn a_goal_merge_confirmed_while_its_observation_is_being_compared_is_not_drift() {
    let (mut world, goal, branch, sibling) = merging_siblings();
    let comparison = delayed_goal_ancestry(&mut world, &branch);
    world.release_write_answers();
    for _ in 0..STEPS {
        world.tick();
        if writer(&world, &branch).is_none() {
            break;
        }
    }
    assert_eq!(writer(&world, &branch), None);
    world.send(comparison);
    observe_goal_branch(&mut world, &branch);
    assert!(!matches!(live_task(&world, goal.task).phase, tasks::Phase::Held { .. }));
    assert!(!matches!(live_task(&world, sibling).phase, tasks::Phase::Held { .. }));
    assert!(forge_hold(&world, &branch).expect("goal branch hold").drift.is_none());
    finish_siblings(&mut world, &goal);
}
