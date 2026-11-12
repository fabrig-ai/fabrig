//! A goal writes design straight to g while changes land beside it
//! (temper/docs/design/domain/forge/change/README.md, 1 and 3, and temper/docs/design/domain/forge/change/questions.md, 1, and temper/docs/design/domain/forge/branches.md, 1;
//! temper/docs/design/domain/agent/tools.md;
//! temper/docs/design/protocol/configuration/reference.md).
use super::goal_branch::{branch_tip, forge_hold, goal_request, writer};
use super::goal_landing::{assert_goal_holds, ended, finish_run, land_into_goal, row, status};
use super::*;
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
const WAIT_TICKS: u32 = 20; // Two simulated seconds give the delegate time to claim the busy slot.

fn running_goal() -> (World, engine::Assignment, Box<[u8]>) {
    let mut world = World::configured_limits(
        true,
        true,
        true,
        STEPS,
        Gate::default(),
        None,
        StoryLimits { slots: Some(WORKER_SLOTS), result_bytes: Some(RESULT_BYTES), ..StoryLimits::default() },
    )
    .with_landing_deployment(Box::new([engine::LandingRule {
        sets: Box::new([]),
        connector: 0,
        kind: 4,
        enforce: true,
        pattern: authority::Pattern {
            segments: [b"forge".as_slice(), b"forge.example", b"org", b"repo", b"branch"].map(Box::from).into(),
            last: authority::Last::Open(b"temper".as_slice().into()),
        },
        ci: true,
        // Child changes merge beside the goal's design writer.
        up_to_date: false,
        gates: Box::new([]),
        approvals: Box::new([]),
    }]));
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, WORKER_SLOTS, Box::new([])));
    let people::Ask::Create { project, mut batch } = goal_request(&world, true) else { panic!("goal template") };
    batch[0].authority.tools |= jig_domain_core::ToolFamilies::JIG.control.0;
    let goal = assigned_chat(&mut world, GOAL_REQUEST, people::Ask::Create { project, batch });
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    world.until(Until::Ready);
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
    let ask = include_str!("../../../../../../crates/temper-engine-service/src/stock/ask.txt");
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
    let assigned = world.assigned.len();
    let answer = mine(world, goal, DELIVERY_CALL, br#"{"words":"Write the design","landing":"goal"}"#);
    let engine::CallAnswer::Delegated(made) = answer else { panic!("design change: {answer:?}") };
    let design = made[0].task;
    let producer = reviews::next_assignment(world, assigned);
    let head = early_start::push(world, &producer, base, b"docs/design/input.md");
    status(world, head, raw::Check::Pending);
    finish_run(
        world,
        &producer,
        tasks::TaskResult::Change {
            title: tasks::ChangeTitle::new(b"Write design".as_slice().into()),
            connector: 0,
            kind: 2,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
            words: b"pushed".as_slice().into(),
        },
    );
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
    let ask = include_str!("../../../../../../crates/temper-engine-service/src/stock/ask.txt");
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
fn a_goals_design_delivery_and_a_sibling_merge_reach_g_without_a_slot() {
    let (mut world, goal, branch) = running_goal();
    let item = &goal.workspace.repositories[0];
    assert_eq!(item.start, engine::ForgeStart::Base(branch.clone()));
    assert_eq!(item.push.as_deref(), Some(branch.as_ref()));
    assert_eq!(item.landing.as_deref(), Some(branch.as_ref()));
    assert_eq!(writer(&world, &branch), None);
    let designers = &forge_hold(&world, &branch).expect("goal hold").design_writers;
    assert_eq!(
        designers.as_ref(),
        &[forge_top::DesignWriter { task: goal.task, attempt: goal.attempt, rereading: false }]
    );
    let home = goal
        .sections
        .iter()
        .find(|section| section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Home))
        .expect("design delivery section");
    assert_eq!(home.body, engine::BriefBody::Text(b"deliver pushes the checkout's Markdown straight to the goal branch, merging its moved tip first; code goes through change.".as_slice().into()));
    let base = branch_tip(&mut world, &branch);
    deliver(&mut world, &goal, base, b"design.md");
    let merged = land_into_goal(&mut world, &goal, &branch, SIBLING_CALL, b"sibling");
    assert_eq!(world.merges, SIBLING_MERGES);
    assert_eq!(writer(&world, &branch), None);
    assert_goal_holds(&world, goal.task, &branch);
    let head = deliver(&mut world, &goal, merged, b"after-merge.md");
    super::goal_branch::observe_goal_branch(&mut world, &branch);
    assert_eq!(
        forge_hold(&world, &branch).expect("live goal hold").drift,
        None,
        "a design push after the merge is still owned"
    );
    assert_file(&mut world, &branch, b"design.md");
    assert_file(&mut world, &branch, b"sibling");
    assert_file(&mut world, &branch, b"after-merge.md");
    park(&mut world, &goal);
    for _ in 0..STEPS {
        world.tick();
        if forge_hold(&world, &branch).expect("retained goal hold").design_writers.is_empty() {
            break;
        }
    }
    let hold = forge_hold(&world, &branch).expect("retained goal hold");
    assert!(hold.design_writers.is_empty(), "{hold:?}");
    assert!(hold.drift.is_none());
    let name = hold.name.clone();
    assert!(world.store.rows.values().any(|row| matches!(row,
        Record::Forge { row } if matches!(row.as_ref(), forge_top::Stored::BranchHead(record) if record.name == name && record.commit == translate::commit(head) && record.owned))));
    super::goal_branch::observe_goal_branch(&mut world, &branch);
    assert!(forge_hold(&world, &branch).expect("goal hold after poll").drift.is_none());
    let second = wake(&mut world, &goal);
    assert_eq!(second.workspace.repositories[0].start, engine::ForgeStart::Base(branch.clone()));
    assert_eq!(second.workspace.repositories[0].push.as_deref(), Some(branch.as_ref()));
    early_start::stop(&mut world, &second);
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

fn push_branch(run: &engine::Assignment) -> Box<[u8]> {
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
    let answer = mine(world, run, number, br#"{"words":"Land goal","branch":"mine"}"#);
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
    world.signed_in = None;
    world.sign_in();
    assigned_chat(
        world,
        GOAL_REQUEST + 1,
        people::Ask::Say { project: 1, task: run.task, words: b"Continue".as_slice().into() },
    )
}

fn assert_file(world: &mut World, branch: &[u8], file: &[u8]) {
    let tip = branch_tip(world, branch);
    assert_eq!(
        world.fake.object(tip).expect("branch tip").tree.get(file).map(AsRef::as_ref),
        Some(b"added".as_slice())
    );
}

#[test]
fn a_goals_second_run_starts_from_its_direct_design_delivery() {
    let (mut world, goal, branch) = running_goal();
    let base = branch_tip(&mut world, &branch);
    let head = deliver(&mut world, &goal, base, b"first.md");
    park(&mut world, &goal);
    let second = wake(&mut world, &goal);
    let item = &second.workspace.repositories[0];
    assert_eq!(item.start, engine::ForgeStart::Base(branch.clone()));
    assert_eq!(item.push.as_deref(), Some(branch.as_ref()));
    let head = deliver(&mut world, &second, head, b"second.md");
    assert_eq!(branch_tip(&mut world, &branch), head);
    assert_eq!(world.merges, 0);
    assert_file(&mut world, &branch, b"first.md");
    assert_file(&mut world, &branch, b"second.md");
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
    let r = push_branch(&goal);
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
                || matches!(op, skein_forge::api::Op::Read(skein_forge::api::Read::Pull { number, ci: true })
                    if row(&world, c1).pull == Some(*number))
        });
        assert_eq!(world.merges, 0, "the superseded head never lands");
        assert_eq!(writer(&world, &r), None);
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

fn deleted(world: &World, branch: &[u8]) -> bool {
    world.writes.iter().any(|write| {
        matches!(write,
        skein_forge::api::Write::DeleteBranch { branch: gone } if gone.as_ref() == branch)
    })
}

#[test]
fn a_goals_run_branch_is_not_watched_after_claim_or_restart() {
    use super::resource_admission::{branch_reads, kept_branch};
    let (mut world, goal, branch) = running_goal();
    let r = format!("temper/{}/r{}", goal.task, goal.task).into_bytes();
    world.forge_reads = Some(Vec::new());
    // A claim merges existing names with its writes. Older saved names can include r.
    let resources = [&r[..], branch.as_ref()].map(|branch| forge_top::Name {
        forge: 1,
        repository: 2,
        what: forge_top::What::Branch(branch.split(|byte| *byte == b'/').map(Box::from).collect()),
    });
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Names {
        task: goal.task,
        resources: Box::new(resources),
    })));
    world.until(Until::Ready);
    for restart in [false, true] {
        if restart {
            world.restart_same();
            world.until(Until::Ready);
        }
        assert!(forge_hold(&world, &r).is_some(), "the unused run branch remains held");
        assert!(!kept_branch(&world, &r), "a workspace claim must not keep r");
        assert!(kept_branch(&world, &branch), "g remains watched");
        for _ in 0..STEPS {
            world.tick();
        }
        assert_eq!(branch_reads(&world, &r), 0, "no run-branch reads, restart={restart}");
        assert!(branch_reads(&world, &branch) > 0);
    }
    world.close();
}

#[test]
fn a_goals_close_deletes_no_unused_run_branch() {
    let (mut world, goal, branch) = running_goal();
    let r = format!("temper/{}/r{}", goal.task, goal.task).into_bytes();
    let base = branch_tip(&mut world, &branch);
    deliver(&mut world, &goal, base, b"design.md");
    let report = tasks::TaskResult::Report { words: b"Done".as_slice().into() };
    finish_run(&mut world, &goal, report.clone());
    await_end(&mut world, goal.task);
    world.until(Until::Ready);
    assert_eq!(ended(&world, goal.task).expect("goal ended").phase, tasks::Phase::Ended(tasks::Ending::Done(report)));
    assert!(!deleted(&world, &r));
    assert!(forge_hold(&world, &r).is_none());
    world.close();
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
    land_delivery(&mut world, change, head);
    finish_run(&mut world, &goal, tasks::TaskResult::Report { words: b"Done".as_slice().into() });
    await_end(&mut world, goal.task);
    world.until(Until::Ready);
    world.close();
}
