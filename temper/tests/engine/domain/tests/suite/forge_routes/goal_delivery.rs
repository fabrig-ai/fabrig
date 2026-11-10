//! A goal's delivery lands by a change while its run keeps writing
//! (temper/docs/design/domain/forge/change/README.md, 1 and 3, and temper/docs/design/domain/forge/change/questions.md, 1, and temper/docs/design/domain/forge/branches.md, 1;
//! temper/docs/design/domain/agent/tools.md;
//! temper/docs/design/protocol/configuration/reference.md).
use super::goal_branch::{branch_tip, forge_hold, goal_request, writer};
use super::goal_landing::{assert_goal_holds, ended, finish_run, land_into_goal, row, status};
use super::*;
use std::collections::BTreeMap;
use temper_engine_domain_forge_change::State;

const CHANNEL: Token = Token::new(7);
const STEPS: u32 = 1000; // The forge world's bound for a full landing and hold release.
const WORKER_SLOTS: u32 = 3; // Room for the goal, a sibling producer and an early producer.
const RESULT_BYTES: u32 = 224; // Three results, a landing subscription and 128 bytes of core news fit 1024 bytes.
const GOAL_REQUEST: u8 = 92;
const SIBLING_CALL: u64 = 93;
const DELIVERY_CALL: u64 = 94;
const DEPENDENT_CALL: u64 = 95;
const SIBLING_MERGES: u32 = 1;
const DELIVERY_MERGES: u32 = 2; // The sibling and C1 each land into g.
const DELIVERY_FILE: &[u8] = b"design";
const DELIVERY_CONTENT: &[u8] = b"the design";
const WAIT_TICKS: u32 = 20; // Two simulated seconds give the delegate time to claim the busy slot.

fn running_goal() -> (World, engine::Assignment, Box<[u8]>) {
    goal_world(false)
}

fn goal_world(merge: bool) -> (World, engine::Assignment, Box<[u8]>) {
    let mut world = World::configured_limits(
        true,
        true,
        true,
        STEPS,
        Gate::default(),
        None,
        StoryLimits {
            slots: Some(WORKER_SLOTS),
            result_bytes: Some(RESULT_BYTES),
            merge_kind: merge,
            merge_authority: merge,
            ..StoryLimits::default()
        },
    )
    .with_landing_deployment(Box::new([engine::LandingRule {
        connector: 0,
        kind: 4,
        enforce: true,
        pattern: authority::Pattern {
            segments: [b"forge".as_slice(), b"forge.example", b"org", b"repo", b"branch"].map(Box::from).into(),
            last: authority::Last::Open(b"temper".as_slice().into()),
        },
        ci: true,
        // The sibling moves g; this story lands the delivery without asking for an update of r.
        up_to_date: false,
        gates: Box::new([]),
        approvals: Box::new([]),
    }]));
    if merge {
        world = world.with_merge_job(b".temper/merge");
    }
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, WORKER_SLOTS, Box::new([])));
    let people::Ask::Create { project, mut batch } = goal_request(&world, true) else { panic!("goal template") };
    batch[0].authority.tools |= jig_domain_core::ToolFamilies::JIG.control.0;
    let goal = assigned_chat(&mut world, GOAL_REQUEST, people::Ask::Create { project, batch });
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    world.until(Until::Ready);
    // The first assignment can precede g's confirmation. Consume that news so
    // the story starts with the confirmed goal's delivery workspace.
    super::park_chat(&mut world, &goal);
    let goal = world
        .assigned
        .iter()
        .rev()
        .find(|run| run.task == goal.task && run.attempt > goal.attempt)
        .expect("confirmation wakes the delivery run")
        .clone();
    (world, goal, branch)
}

fn await_end(world: &mut World, task: u64) {
    for _ in 0..STEPS {
        world.tick();
        if ended(world, task).is_some() {
            return;
        }
    }
    panic!("task {task} ended")
}

fn change_keys(world: &World) -> Vec<Key> {
    world.store.rows.keys().filter(|key| matches!(key, Key::Forge(forge_top::Key::Change(_)))).cloned().collect()
}

fn assert_design_first_recipe() {
    let settings: serde_json::Value =
        serde_json::from_str(include_str!("../../../../../../docs/examples/reference/settings.json"))
            .expect("reference settings");
    let coordinator = &settings["items"][0]["set"];
    let shared = coordinator["instructions"].as_str().expect("shared instructions");
    let results = coordinator["parts"][2]["instructions"].as_str().expect("result instructions");
    for words in [shared, results] {
        assert!(words.contains("Use after only for live prerequisites"), "the recipe must limit after to live tasks");
        assert!(
            words.contains(
                "once design has landed into g, cite its path and section in words and omit its ended task from after"
            ),
            "the recipe must cite landed design without an ended dependency"
        );
    }
    let ask = include_str!("../../../../../../crates/temper-service/src/stock/ask.txt");
    assert!(ask.contains("Use after only for live prerequisites"), "Ask temper must propose live dependencies");
    assert!(ask.contains("cite design's repository path and section in words; omit its ended task from after"));
}

fn assert_ended_design_refused(world: &mut World, goal: &engine::Assignment, design: u64, words: &str) {
    let assigned = world.assigned.len();
    let made = live_task(world, goal.task).made;
    let changes = change_keys(world);
    let input = serde_json::json!({"words": words, "landing": "goal", "after": [design]}).to_string();
    let answer = mine(world, goal, DEPENDENT_CALL, input.as_bytes());
    let engine::CallAnswer::DelegationRefused(problem) = answer else { panic!("ended dependency refused: {answer:?}") };
    assert_eq!(problem.why, tasks::Refusal::Dependencies);
    assert_eq!(live_task(world, goal.task).made, made, "refusal creates no task");
    assert_eq!(change_keys(world), changes, "refusal creates no change");
    assert_eq!(world.assigned.len(), assigned, "refusal starts no producer");
}

fn pending_design_delivery(world: &mut World, goal: &engine::Assignment, branch: &[u8]) -> (u64, u64, u64) {
    let base = branch_tip(world, branch);
    let head = deliver(world, goal, base, b"docs/design/input.md");
    let design = make_delivery(world, goal, DELIVERY_CALL);
    (base, head, design)
}

#[test]
fn a_design_first_recipe_cites_landed_design_without_an_ended_dependency() {
    let (mut world, goal, branch) = running_goal();
    let (_, design_head, design) = pending_design_delivery(&mut world, &goal, &branch);
    land_delivery(&mut world, design, design_head);
    assert!(matches!(ended(&world, design).expect("design ended").phase, tasks::Phase::Ended(tasks::Ending::Done(_))));
    assert_file(&mut world, &branch, b"docs/design/input.md");
    read_news(&mut world, &goal);

    coding_from_landed_design(world, &goal, &branch, design);
    assert_design_first_recipe();
}

fn coding_from_landed_design(mut world: World, goal: &engine::Assignment, branch: &[u8], design: u64) {
    let words = "Implement docs/design/input.md, section 1, landed into g";
    assert_ended_design_refused(&mut world, goal, design, words);
    let assigned = world.assigned.len();
    let input = serde_json::json!({"words": words, "landing": "goal"}).to_string();
    let answer = mine(&mut world, goal, DEPENDENT_CALL + 1, input.as_bytes());
    let engine::CallAnswer::Delegated(made) = answer else { panic!("citing change admitted: {answer:?}") };
    let coding = made[0].task;
    assert!(live_task(&world, coding).dependencies.is_empty());
    let producer = reviews::next_assignment(&mut world, assigned);
    assert_eq!(live_task(&world, producer.task).requester, tasks::Party::Task(coding));
    assert_eq!(producer.workspace.repositories[0].start, engine::ForgeStart::Base(branch.into()));
    let start = branch_tip(&mut world, branch);
    assert_file(&mut world, branch, b"docs/design/input.md");
    let head = early_start::push(&mut world, &producer, start, b"implementation");
    world.tick();
    status(&mut world, head, raw::Check::Passed);
    finish_run(
        &mut world,
        &producer,
        tasks::TaskResult::Change {
            title: tasks::ChangeTitle::new(b"Implement the landed design".as_slice().into()),
            connector: 0,
            kind: 2,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
            words: b"pushed".as_slice().into(),
        },
    );
    land_delivery(&mut world, coding, head);
    assert_file(&mut world, branch, b"docs/design/input.md");
    assert_file(&mut world, branch, b"implementation");
    early_start::stop(&mut world, goal);
}

fn assert_confirmed_design_recipe() {
    let settings: serde_json::Value =
        serde_json::from_str(include_str!("../../../../../../docs/examples/reference/settings.json"))
            .expect("reference settings");
    let coordinator = &settings["items"][0]["set"];
    for (place, words) in [
        ("shared", coordinator["instructions"].as_str().expect("shared instructions")),
        ("first-run batch", coordinator["parts"][0]["instructions"].as_str().expect("first instructions")),
        ("designer result", coordinator["parts"][2]["instructions"].as_str().expect("result instructions")),
    ] {
        assert!(
            words.contains("Do not issue dependent coding until all required design is confirmed landed into g"),
            "{place}: coding dispatch must wait for confirmed design landing"
        );
        assert!(
            words.contains("A push, delivery or designer report is insufficient"),
            "{place}: production evidence must not stand in for landing confirmation"
        );
    }
    let ask = include_str!("../../../../../../crates/temper-service/src/stock/ask.txt");
    assert!(ask.contains("confirm all required design landed into g before dispatching coding"));
    assert!(ask.contains("A push, delivery or designer report is insufficient"));
    let first = coordinator["parts"][0]["instructions"].as_str().expect("first instructions");
    assert!(first.contains("Keep coding that uses pending design out of the first batch"));
    assert!(!first.contains("and the changes the design already specifies, after S where they use its shapes"));
}

fn live_design_allows_early_coding() {
    let (mut world, goal, branch) = running_goal();
    let (base, head, design) = pending_design_delivery(&mut world, &goal, &branch);
    let assigned = world.assigned.len();
    let input = serde_json::json!({"words": "Build on pending design", "landing": "goal", "after": [design]});
    let answer = mine(&mut world, &goal, DEPENDENT_CALL, input.to_string().as_bytes());
    let engine::CallAnswer::Delegated(made) = answer else { panic!("early coding admitted: {answer:?}") };
    let coding = made[0].task;
    let producer = reviews::next_assignment(&mut world, assigned);
    assert_eq!(live_task(&world, producer.task).requester, tasks::Party::Task(coding));
    assert_eq!(
        producer.workspace.repositories[0].start,
        engine::ForgeStart::Merged { branch: branch.clone(), heads: Box::new([translate::commit(head)]) }
    );
    assert_eq!(live_task(&world, coding).started_from.as_ref(), &[design]);
    assert!(ended(&world, design).is_none(), "dispatch and producer start precede design landing");
    assert_eq!(branch_tip(&mut world, &branch), base);
    assert_eq!(world.merges, 0);
    // This reproduces dispatch and run-start order; it makes no coding edit before landing.
    land_delivery(&mut world, design, head);
    early_start::stop(&mut world, &goal);
}

#[test]
fn design_first_waits_for_confirmed_landing_before_coding() {
    live_design_allows_early_coding();
    let (mut world, goal, branch) = running_goal();
    let (base, head, design) = pending_design_delivery(&mut world, &goal, &branch);
    let changes = change_keys(&world);
    let assigned = world.assigned.len();
    for _ in 0..WAIT_TICKS {
        world.tick();
    }
    assert!(ended(&world, design).is_none(), "a pushed delivery is not a confirmed landing");
    assert_eq!(branch_tip(&mut world, &branch), base);
    assert_eq!(change_keys(&world), changes, "the scripted lead waits without dispatching coding");
    assert_eq!(world.assigned.len(), assigned, "waiting starts no coding producer");
    land_delivery(&mut world, design, head);
    assert_file(&mut world, &branch, b"docs/design/input.md");
    read_news(&mut world, &goal);
    coding_from_landed_design(world, &goal, &branch, design);
    // The world exercises the recipe; these checks pin the model-facing rule, not model compliance.
    assert_confirmed_design_recipe();
}

#[test]
fn a_goals_delivery_lands_into_its_goal_branch_by_a_change() {
    let (mut world, goal, branch) = running_goal();

    // 1. Delivering leaves the goal's report contract intact.
    let charter = temper_engine_smith::charter(&goal);
    assert!(charter.grants.deliver.is_some(), "the goal's charter grants deliver");
    assert!(charter.grants.tools.modify);
    let deliver = charter.grants.deliver.as_ref().expect("delivery grant");
    assert!(deliver.checks_must_pass);
    assert_eq!(
        deliver.fields.iter().map(|field| field.name.as_ref()).collect::<Vec<_>>(),
        [b"title".as_slice(), b"body"]
    );
    assert!(charter.outcome.change.is_none());
    assert!(charter.outcome.report.is_some());

    // 2. The run writes r and leaves g's slot free for merges.
    let run_branch = format!("temper/{}/r{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    assert_eq!(goal.workspace.repositories[0].start, engine::ForgeStart::Base(branch.clone()));
    assert_eq!(goal.workspace.repositories[0].push, Some(run_branch.clone()));
    let goal_writer = Some(tasks::Writer::Run { task: goal.task, attempt: goal.attempt });
    assert_eq!(writer(&world, &run_branch), goal_writer);
    assert_eq!(writer(&world, &branch), None);

    // 3. The scripted worker pushes H1 and hints its run branch.
    let base = branch_tip(&mut world, &branch);
    assert_eq!(
        world.external(1, raw::Op::Git(raw::Git::Create { branch: run_branch.clone(), commit: base })),
        raw::Answer::Branch(raw::Created::Created)
    );
    let head =
        fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &run_branch, DELIVERY_FILE, DELIVERY_CONTENT, 1)
            .expect("the goal pushes its delivery");
    assert_ne!(head, base);
    assert_eq!(branch_tip(&mut world, &run_branch), head);
    status(&mut world, head, raw::Check::Pending);
    early_start::observe_push(&mut world, &goal);

    // 4. A sibling lands into g while the goal keeps its run slot.
    let sibling_head = land_into_goal(&mut world, &goal, &branch, SIBLING_CALL, b"sibling");
    assert_ne!(sibling_head, base);
    assert_eq!(world.merges, SIBLING_MERGES);
    assert_eq!(writer(&world, &run_branch), goal_writer);
    assert_goal_holds(&world, goal.task, &branch);

    read_news(&mut world, &goal);

    // 5. C1 lands the run branch without making a producer.
    let assigned = world.assigned.len();
    let answer =
        mine(&mut world, &goal, DELIVERY_CALL, br#"{"words":"Land the design","branch":"run","landing":"goal"}"#);
    let engine::CallAnswer::Delegated(made) = answer else { panic!("C1 delegated: {answer:?}") };
    let c1 = made[0].task;
    assert_eq!(row(&world, c1).branch, run_branch);
    assert_eq!(row(&world, c1).base, branch);
    assert_eq!(row(&world, c1).delegate, None);
    assert_eq!(world.assigned.len(), assigned, "C1 has no producer");

    // 6. C2 starts early from H1 while C1 waits for its checks.
    let input = format!(r#"{{"words":"Build on the design","landing":"goal","after":[{c1}]}}"#);
    let answer = mine(&mut world, &goal, DEPENDENT_CALL, input.as_bytes());
    let engine::CallAnswer::Delegated(made) = answer else { panic!("C2 delegated: {answer:?}") };
    let c2 = made[0].task;
    let producer = reviews::next_assignment(&mut world, assigned);
    assert_eq!(live_task(&world, producer.task).requester, tasks::Party::Task(c2));
    assert_eq!(
        producer.workspace.repositories[0].start,
        engine::ForgeStart::Merged { branch: branch.clone(), heads: Box::new([translate::commit(head)]) }
    );
    assert_eq!(live_task(&world, c2).started_from.as_ref(), &[c1]);
    assert!(ended(&world, c1).is_none(), "C2 starts before C1 lands");

    // 7. C1 lands H1 while the goal's run still writes r.
    status(&mut world, head, raw::Check::Passed);
    for _ in 0..STEPS {
        world.tick();
        assert_eq!(writer(&world, &run_branch), goal_writer);
        if ended(&world, c1).is_some() {
            break;
        }
    }
    assert!(ended(&world, c1).is_some(), "C1 closes while the goal runs");
    assert!(matches!(row(&world, c1).change.state, State::Landed { .. }), "{:?}", row(&world, c1));
    assert_eq!(row(&world, c1).change.last_head, Some(translate::commit(head)));
    assert_eq!(world.merges, DELIVERY_MERGES);
    let tip = branch_tip(&mut world, &branch);
    let tree = &world.fake.object(tip).expect("goal tip").tree;
    assert_eq!(tree.get(DELIVERY_FILE).map(AsRef::as_ref), Some(DELIVERY_CONTENT));
    assert_eq!(tree.get(b"sibling".as_slice()).map(AsRef::as_ref), Some(b"added".as_slice()));

    // 8. C1's close keeps the run branch and the goal's holds.
    world.until(Until::Ready);
    assert_eq!(branch_tip(&mut world, &run_branch), head);
    assert!(!world.writes.iter().any(|write| matches!(write,
        skein_forge::api::Write::DeleteBranch { branch: deleted } if *deleted == run_branch)));
    assert_eq!(forge_hold(&world, &run_branch).expect("run branch hold").task, goal.task);
    assert_goal_holds(&world, goal.task, &branch);

    // 9. The early producer completes, then the goal finishes with a report.
    let base = branch_tip(&mut world, &branch);
    let implementation_head = early_start::push(&mut world, &producer, base, b"implementation");
    world.tick();
    status(&mut world, implementation_head, raw::Check::Passed);
    finish_run(
        &mut world,
        &producer,
        tasks::TaskResult::Change {
            title: tasks::ChangeTitle::new(b"Build on the design".as_slice().into()),
            connector: 0,
            kind: 2,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
            words: b"pushed".as_slice().into(),
        },
    );
    await_end(&mut world, c2);
    let report = tasks::TaskResult::Report { words: b"Done".as_slice().into() };
    finish_run(&mut world, &goal, report.clone());
    await_end(&mut world, goal.task);
    assert_eq!(ended(&world, goal.task).expect("goal ended").phase, tasks::Phase::Ended(tasks::Ending::Done(report)));
    world.close();
}

fn read_news(world: &mut World, run: &engine::Assignment) {
    world.scripted_read_inbox(run);
}

fn mine(world: &mut World, run: &engine::Assignment, number: u64, input: &[u8]) -> engine::CallAnswer {
    let input = smith_domain_run::HostInput::attested(input.into()).expect("bounded change input");
    let temper_engine_smith::Decoded::Call(call) = temper_engine_smith::call(
        smith_domain_run::CallName { activation: run.attempt, completion: 1, position: 1 },
        b"change",
        &input,
    )
    .expect("change decodes") else {
        panic!("change call")
    };
    world.scripted_call(run, number, call.tool)
}

fn run_branch(run: &engine::Assignment) -> Box<[u8]> {
    run.workspace.repositories[0].push.clone().expect("goal delivery destination")
}

fn deliver(world: &mut World, run: &engine::Assignment, base: u64, file: &[u8]) -> u64 {
    let head = early_start::push(world, run, base, file);
    world.tick();
    status(world, head, raw::Check::Pending);
    early_start::observe_push(world, run);
    head
}

fn make_delivery(world: &mut World, run: &engine::Assignment, number: u64) -> u64 {
    let answer = mine(world, run, number, br#"{"words":"Land delivery","branch":"run","landing":"goal"}"#);
    let engine::CallAnswer::Delegated(made) = answer else { panic!("delivery admitted: {answer:?}") };
    made[0].task
}

fn land_delivery(world: &mut World, change: u64, head: u64) {
    status(world, head, raw::Check::Passed);
    await_end(world, change);
    assert!(matches!(row(world, change).change.state, State::Landed { .. }), "{:?}", row(world, change));
    assert_eq!(row(world, change).change.last_head, Some(translate::commit(head)));
    world.until(Until::Ready);
}

fn park(world: &mut World, run: &engine::Assignment) {
    // With no pending delegate, a timer beyond the story's horizon keeps the
    // goal waiting; topic subscriptions alone do not.
    if live_task(world, run.task).delegates.is_empty() {
        if let Some(subscription) = live_task(world, run.task).subscriptions.first().map(|item| item.number) {
            world.scripted_call(run, DEPENDENT_CALL, engine::Tool::Unsubscribe { subscription });
        }
        let answer = world.scripted_call(
            run,
            DEPENDENT_CALL + 1,
            engine::Tool::Subscribe {
                kind: tasks::SubscriptionKind::Timer { at: skein_lib::Wall::from_nanos(u64::MAX), period: None },
            },
        );
        assert!(matches!(answer, engine::CallAnswer::Subscribed { .. }), "{answer:?}");
    }
    read_news(world, run);
    let task = live_task(world, run.task);
    super::park_after_turn(world, run, None, task.turn);
    world.until(Until::Ready);
    assert_eq!(live_task(world, run.task).phase, tasks::Phase::Active(tasks::Active::Idle));
}

fn wake(world: &mut World, run: &engine::Assignment) -> engine::Assignment {
    assigned_chat(
        world,
        GOAL_REQUEST + 1,
        people::Ask::Say { project: 1, task: run.task, words: b"Continue".as_slice().into() },
    )
}

fn assert_next_start(run: &engine::Assignment, branch: &[u8], head: u64, destination: &[u8]) {
    let item = &run.workspace.repositories[0];
    assert_eq!(item.start, engine::ForgeStart::Merge { branch: branch.into(), base: translate::commit(head) });
    assert_eq!(item.landing.as_deref(), Some(branch));
    assert_eq!(item.push.as_deref(), Some(destination));
    assert!(item.delivers);
}

fn merge_tree(world: &World, tip: u64, head: u64) -> BTreeMap<Box<[u8]>, Box<[u8]>> {
    let mut tree = world
        .fake
        .object(tip)
        .expect("goal tip")
        .tree
        .iter()
        .map(|(path, content)| (path.clone(), content.clone()))
        .collect::<BTreeMap<_, _>>();
    tree.extend(
        world.fake.object(head).expect("run tip").tree.iter().map(|(path, content)| (path.clone(), content.clone())),
    );
    tree
}

fn merge_start(world: &mut World, branch: &[u8], head: u64) -> u64 {
    let tip = branch_tip(world, branch);
    let files = merge_tree(world, tip, head).into_iter().map(|(path, content)| raw::File { path, content }).collect();
    fake::merge_commit(&mut world.fake, &world.fake_env.limits, tip, head, files, b"Prepare next goal run")
        .expect("scripted checkout merges both tips")
}

fn assert_file(world: &mut World, branch: &[u8], file: &[u8]) {
    let tip = branch_tip(world, branch);
    assert_eq!(
        world.fake.object(tip).expect("branch tip").tree.get(file).map(AsRef::as_ref),
        Some(b"added".as_slice())
    );
}

#[test]
fn a_goals_second_run_delivers_from_g_with_its_run_branch_merged_in() {
    let (mut world, goal, branch) = running_goal();
    let r = run_branch(&goal);
    let base = branch_tip(&mut world, &branch);
    let h1 = deliver(&mut world, &goal, base, b"first");
    let c1 = make_delivery(&mut world, &goal, DELIVERY_CALL);
    land_delivery(&mut world, c1, h1);
    park(&mut world, &goal);
    let second = wake(&mut world, &goal);
    assert_next_start(&second, &branch, h1, &r);
    let start = merge_start(&mut world, &branch, h1);
    let h2 = deliver(&mut world, &second, start, b"second");
    assert_eq!(branch_tip(&mut world, &r), h2);
    let c2 = make_delivery(&mut world, &second, DELIVERY_CALL + 10);
    assert_ne!(c2, c1, "an ended change no longer excludes its successor");
    land_delivery(&mut world, c2, h2);
    assert_eq!(world.merges, DELIVERY_MERGES);
    assert_file(&mut world, &branch, b"first");
    assert_file(&mut world, &branch, b"second");
    early_start::stop(&mut world, &second);
}

#[test]
fn a_run_branch_left_holding_what_g_lacks_reaches_the_next_run() {
    let (mut world, goal, branch) = running_goal();
    let r = run_branch(&goal);
    let base = branch_tip(&mut world, &branch);
    let h1 = deliver(&mut world, &goal, base, b"undelivered");
    park(&mut world, &goal);
    assert_eq!(world.merges, 0, "the first run made no change");
    assert_eq!(branch_tip(&mut world, &branch), base);
    let second = wake(&mut world, &goal);
    assert_next_start(&second, &branch, h1, &r);
    let start = merge_start(&mut world, &branch, h1);
    let h2 = deliver(&mut world, &second, start, b"next");
    let change = make_delivery(&mut world, &second, DELIVERY_CALL);
    land_delivery(&mut world, change, h2);
    assert_eq!(world.merges, SIBLING_MERGES);
    assert_file(&mut world, &branch, b"undelivered");
    assert_file(&mut world, &branch, b"next");
    early_start::stop(&mut world, &second);
}

fn checking(world: &mut World, change: u64, head: u64) {
    for _ in 0..STEPS {
        world.tick();
        if row(world, change).change.state == (State::Checking { head: translate::commit(head) }) {
            return;
        }
    }
    panic!("change checks head {head}: {:?}", row(world, change));
}

#[test]
fn a_delivery_under_its_open_change_is_that_changes_new_head() {
    let (mut world, goal, branch) = running_goal();
    let r = run_branch(&goal);
    let base = branch_tip(&mut world, &branch);
    let h1 = deliver(&mut world, &goal, base, b"first");
    let c1 = make_delivery(&mut world, &goal, DELIVERY_CALL);
    checking(&mut world, c1, h1);
    let h2 = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &r, b"second", b"added", 1)
        .expect("the ongoing goal delivers again");
    status(&mut world, h2, raw::Check::Pending);
    early_start::observe_push(&mut world, &goal);
    status(&mut world, h1, raw::Check::Passed);
    let mut read_ci = false;
    for _ in 0..STEPS {
        world.tick();
        read_ci |= world.pending.values().any(|op| {
            matches!(op,
            skein_forge::api::Op::Read(skein_forge::api::Read::Checks { commit })
                if *commit == translate::commit(h2))
        });
        assert_eq!(world.merges, 0, "the superseded head never lands");
        assert_eq!(writer(&world, &r), Some(tasks::Writer::Run { task: goal.task, attempt: goal.attempt }));
        if read_ci && row(&world, c1).change.state == (State::Checking { head: translate::commit(h2) }) {
            break;
        }
    }
    assert!(read_ci, "CI is read again at the new delivery");
    assert_eq!(row(&world, c1).change.state, State::Checking { head: translate::commit(h2) });
    land_delivery(&mut world, c1, h2);
    let pull = row(&world, c1).pull.expect("delivery pull");
    assert!(!world.writes.iter().any(|write| matches!(write,
        skein_forge::api::Write::Merge { number, head, .. } if *number == pull && *head == translate::commit(h1))));
    assert_file(&mut world, &branch, b"first");
    assert_file(&mut world, &branch, b"second");
    early_start::stop(&mut world, &goal);
}

#[test]
fn a_conflict_on_a_run_branch_goes_to_the_merge_job_and_a_resolution() {
    let (mut world, goal, branch) = goal_world(true);
    let r = run_branch(&goal);
    let base = branch_tip(&mut world, &branch);
    deliver(&mut world, &goal, base, b"conflict");
    let h1 = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &r, b"conflict", b"run work", 1)
        .expect("run and goal will disagree on the file");
    world.tick();
    status(&mut world, h1, raw::Check::Pending);
    early_start::observe_push(&mut world, &goal);
    let moved = land_into_goal(&mut world, &goal, &branch, SIBLING_CALL, b"conflict");
    read_news(&mut world, &goal);
    let c1 = make_delivery(&mut world, &goal, DELIVERY_CALL);
    status(&mut world, h1, raw::Check::Passed);
    for _ in 0..STEPS {
        world.tick();
        if row(&world, c1).delegate.is_some() {
            break;
        }
    }
    let (delegate, kind) = row(&world, c1).delegate.expect("conflict delegates its merge");
    assert!(matches!(kind, temper_engine_domain_forge_change::Delegate::Merge { .. }), "{kind:?}");
    for _ in 0..WAIT_TICKS {
        world.tick();
    }
    assert!(world.jobs.is_empty(), "the merge job waits while the goal writes r");
    assert_eq!(writer(&world, &r), Some(tasks::Writer::Run { task: goal.task, attempt: goal.attempt }));
    let job = freed_merge_job(&mut world, &goal, delegate);
    assert_eq!(job.run.raw(), delegate);
    assert_eq!(job.kind, jig_domain_link::JobKind(3));
    let item = &world.job_workspaces[&job.run.raw()].repositories[0];
    assert_eq!(item.start, engine::ForgeStart::Merge { branch: r.clone(), base: translate::commit(moved) });
    assert_eq!(item.push.as_deref(), Some(r.as_ref()));
    assert_eq!(forge_hold(&world, &r).expect("goal's run hold").task, goal.task);
    assert_eq!(writer(&world, &r), Some(tasks::Writer::Run { task: delegate, attempt: job.attempt.raw() }));
    assert!(live_task(&world, delegate).holdings.is_empty(), "the job borrows its ancestor's hold");
    let assigned = world.assigned.len();
    let answer = world.protocol.answer_job(
        CHANNEL,
        job.run.raw(),
        job.attempt.raw(),
        host::JobEnding::Finished {
            result: br#"{"left":["conflict"]}"#.as_slice().into(),
            work: host::Work { left: None, saved: None },
        },
    );
    world.send(answer);
    let resolution = reviews::next_assignment(&mut world, assigned);
    assert_eq!(live_task(&world, resolution.task).requester, tasks::Party::Task(c1));
    assert_eq!(
        resolution.workspace.repositories[0].start,
        engine::ForgeStart::Merge { branch: r.clone(), base: translate::commit(moved) }
    );
    assert_eq!(run_branch(&resolution), r);
    assert_eq!(row(&world, c1).change.resolutions, 1);
    let mut tree = merge_tree(&world, h1, moved);
    tree.insert(b"conflict".as_slice().into(), b"resolved".as_slice().into());
    let files = tree.into_iter().map(|(path, content)| raw::File { path, content }).collect();
    let resolved = fake::merge_commit(&mut world.fake, &world.fake_env.limits, h1, moved, files, b"Resolve conflict")
        .expect("resolution keeps both parents");
    assert_eq!(
        world.external(1, raw::Op::Git(raw::Git::Push { branch: r.clone(), commit: resolved, expected: Some(h1) })),
        raw::Answer::Pushed(raw::Pushed::Pushed)
    );
    status(&mut world, resolved, raw::Check::Passed);
    finish_run(
        &mut world,
        &resolution,
        tasks::TaskResult::Change {
            title: tasks::ChangeTitle::new(b"Resolve delivery".as_slice().into()),
            connector: 0,
            kind: 2,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
            words: b"resolved".as_slice().into(),
        },
    );
    land_delivery(&mut world, c1, resolved);
    let next = world
        .assigned
        .iter()
        .rev()
        .find(|run| run.task == goal.task && run.attempt > goal.attempt)
        .cloned()
        .unwrap_or_else(|| wake(&mut world, &goal));
    assert_next_start(&next, &branch, resolved, &r);
    let tip = branch_tip(&mut world, &branch);
    assert_eq!(
        world.fake.object(tip).expect("landed resolution").tree.get(b"conflict".as_slice()).map(AsRef::as_ref),
        Some(b"resolved".as_slice())
    );
    early_start::stop(&mut world, &next);
}

fn deleted(world: &World, branch: &[u8]) -> bool {
    world.writes.iter().any(|write| {
        matches!(write,
        skein_forge::api::Write::DeleteBranch { branch: gone } if gone.as_ref() == branch)
    })
}

#[test]
fn a_goals_close_takes_its_run_branch() {
    let (mut world, goal, branch) = running_goal();
    let r = run_branch(&goal);
    let base = branch_tip(&mut world, &branch);
    let head = deliver(&mut world, &goal, base, b"delivered");
    let c1 = make_delivery(&mut world, &goal, DELIVERY_CALL);
    land_delivery(&mut world, c1, head);
    assert!(!deleted(&world, &r), "the change closes without deleting its requester's branch");
    assert_eq!(branch_tip(&mut world, &r), head);
    assert_eq!(forge_hold(&world, &r).expect("goal retains r").task, goal.task);
    let report = tasks::TaskResult::Report { words: b"Done".as_slice().into() };
    finish_run(&mut world, &goal, report.clone());
    await_end(&mut world, goal.task);
    world.until(Until::Ready);
    assert_eq!(ended(&world, goal.task).expect("goal ended").phase, tasks::Phase::Ended(tasks::Ending::Done(report)));
    assert!(deleted(&world, &r), "the goal's close deletes its run branch");
    assert!(forge_hold(&world, &r).is_none());
    assert_eq!(world.fake.branch(b"org/repo", &r), None);
    world.close();
}

fn freed_merge_job(world: &mut World, goal: &engine::Assignment, delegate: u64) -> host::JobAssignment {
    let wait_began = world.env.now;
    park(world, goal);
    if matches!(live_task(world, delegate).phase, tasks::Phase::Held { .. }) {
        world.signed_in = None;
        world.sign_in();
        let revision = live_task(world, delegate).revisions.hold;
        assert!(matches!(live_task(world, delegate).phase, tasks::Phase::Held { why: tasks::Hold::HoldsWaited, .. }));
        eprintln!(
            "slot wait held: refused={}; attempt={}; now={:?}",
            live_task(world, delegate).refusals,
            live_task(world, delegate).attempt,
            world.env.now
        );
        world.send(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(96)),
            sign_in: world.signed_in.expect("owner session"),
            key: [96; 16],
            ask: people::Ask::Release { project: 1, task: delegate, revision, again: None },
        }));
        world.until(Until::Ready);
    }
    for _ in 0..STEPS {
        world.tick();
        if !world.jobs.is_empty() {
            break;
        }
    }
    assert!(!world.jobs.is_empty(), "the parked run frees r for the merge job: {:?}", live_task(world, delegate).phase);
    let job = world.jobs.remove(0);
    eprintln!(
        "merge job {delegate} writes r under goal {}; wait={:?}; stall={:?}",
        goal.task,
        world.env.now.saturating_since(wait_began),
        world.env.limits.forge.change_policy.stall
    );
    job
}

// temper/docs/design/domain/forge/queues.md: a queued change follows its goal's priority-only amendment.
#[test]
fn a_goals_priority_amendment_moves_its_queued_change() {
    let (mut world, goal, branch) = running_goal();
    let base = branch_tip(&mut world, &branch);
    let head = deliver(&mut world, &goal, base, b"priority");
    let change = make_delivery(&mut world, &goal, DELIVERY_CALL);
    status(&mut world, head, raw::Check::Passed);
    for _ in 0..STEPS {
        world.tick();
        if matches!(row(&world, change).change.state, State::Queued { .. }) {
            break;
        }
    }
    assert!(matches!(row(&world, change).change.state, State::Queued { .. }), "setup: the change is queued");
    let old_priority = live_task(&world, goal.task).tracked.expect("tracked goal");
    assert_eq!(row(&world, change).priority, i32::try_from(old_priority).unwrap());
    let priority = old_priority + 1;
    let amendment = people::Amendment {
        tracked: Some(people::Tracking::Tracked { priority }),
        addressed: None,
        instructions: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        llm: None,
        reason: b"Change queue priority".as_slice().into(),
    };
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(96)),
        sign_in: world.signed_in.expect("owner session"),
        key: [96; 16],
        ask: people::Ask::Amend { project: 1, task: goal.task, revision: 0, amendment, release: None },
    }));
    world.until(Until::Ready);
    assert_eq!(live_task(&world, goal.task).tracked, Some(priority), "the priority-only amendment was committed");
    assert_eq!(
        row(&world, change).priority,
        i32::try_from(priority).unwrap(),
        "the goal feed must save its change's new priority"
    );
    world.close();
}
