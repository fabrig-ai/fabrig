//! Early producers, their once rule and kept landings (temper/docs/design/domain/forge.md, 8.11).
use super::goal_branch::{branch_tip, goal_request};
use super::*;
use temper_engine_domain_forge_change as change;

const CHANNEL: Token = Token::new(7);
const STEPS: u32 = 1000; // A full change landing, including fresh reads and hold release.
const WORKER_SLOTS: u32 = 4; // The goal and three producers can run together.
const QUIET_TICKS: u32 = 40; // Four simulated seconds settle hint reads inside the producer's stall.
const TWO_CHANGES: usize = 2;
const TWO_MERGES: u32 = 2; // One merge per change in the pair.
const RESULT_BYTES: u32 = 224; // Three results, a landing subscription and 128 bytes of core news fit 1024 bytes.
const LANDINGS: usize = 2; // C2 and C3 each land once into g.


fn world() -> World {
    world_with(false)
}

fn world_with(across: bool) -> World {
    let mut world = World::configured_limits(
        true,
        true,
        true,
        STEPS,
        Gate::default(),
        None,
        StoryLimits { across, slots: Some(WORKER_SLOTS), result_bytes: Some(RESULT_BYTES), ..StoryLimits::default() },
    );
    world.env.limits.tasks.batch = 2;
    let segments = [b"forge".as_slice(), b"forge.example", b"org", b"repo", b"branch"].map(Box::from);
    world = world.with_landing_deployment(Box::new([
        engine::LandingRule {
            connector: 0,
            kind: 4,
            enforce: true,
            pattern: authority::Pattern {
                segments: segments.clone().into(),
                last: authority::Last::Exact(b"main".as_slice().into()),
            },
            ci: true,
            up_to_date: true,
            gates: Box::new([]),
            approvals: Box::new([]),
        },
        engine::LandingRule {
            connector: 0,
            kind: 4,
            enforce: true,
            pattern: authority::Pattern {
                segments: segments.into(),
                last: authority::Last::Open(b"temper".as_slice().into()),
            },
            ci: false,
            up_to_date: false,
            gates: Box::new([]),
            approvals: Box::new([]),
        },
    ]));
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, WORKER_SLOTS, Box::new([])));
    world
}

fn change_row(world: &World, task: u64) -> &forge_top::ChangeRow {
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(task))) else {
        panic!("registered change {task}")
    };
    let forge_top::Stored::Change(row) = row.as_ref() else { panic!("change row") };
    row
}

fn ended(world: &World, task: u64) -> bool {
    world.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task))))
}

fn finish(world: &mut World, producer: &engine::Assignment, title: &[u8]) {
    let event = world.protocol.answer(
        CHANNEL,
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(title.into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: b"Done: pushed.".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
}

fn push(world: &mut World, producer: &engine::Assignment, base: u64, file: &[u8]) -> u64 {
    let branch = producer.workspace.repositories[0].push.clone().expect("producer destination");
    assert_eq!(
        world.external(2, raw::Op::Git(raw::Git::Push { branch: branch.clone(), commit: base, expected: None })),
        raw::Answer::Pushed(raw::Pushed::Pushed)
    );
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, file, b"added", 2).expect("producer push")
}

fn assert_landed_tree(world: &World, main: u64, tip: u64, c2_head: u64, c3_head: u64) {
    let tree = &world.fake.object(tip).expect("goal tip").tree;
    let pushed_tree = &world.fake.object(c3_head).expect("C3 pushed head").tree;
    assert_eq!(tree.len(), pushed_tree.len());
    for (path, content) in pushed_tree {
        assert_eq!(tree.get(path), Some(content), "the goal tip holds C3's pushed tree");
    }
    let dependency_tree = &world.fake.object(c2_head).expect("C2 pushed head").tree;
    let base_tree = &world.fake.object(main).expect("main tip").tree;
    for (path, content) in dependency_tree {
        if !base_tree.contains_key(path) {
            assert_eq!(tree.get(path), Some(content), "C2's added path is kept");
        }
    }
}

fn goal_world() -> (World, engine::Assignment, Box<[u8]>, u64) {
    goal_world_in(world())
}

fn goal_world_in(mut world: World) -> (World, engine::Assignment, Box<[u8]>, u64) {

    let people::Ask::Create { project, mut batch } = goal_request(&world, true) else { panic!("goal template") };
    batch[0].authority.tools |= jig_domain_core::ToolFamilies::JIG.control.0;
    let goal = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    world.until(Until::Ready);
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    let main = branch_tip(&mut world, &branch);
    (world, goal, branch, main)
}

fn call(world: &mut World, goal: &engine::Assignment, tool: engine::Tool) -> engine::CallAnswer {
    let before = world.answers.len();
    let number = u64::try_from(before).expect("bounded calls") + 93;
    let event = world.protocol.call(
        CHANNEL,
        goal.task,
        goal.attempt,
        number,
        engine::Call { completion: 1, position: u32::try_from(number).expect("bounded calls"), tool },
    );
    world.send(event);
    for _ in 0..STEPS {
        world.tick();
        if let Some(answer) = world.answers.get(before) {
            return answer.clone();
        }
    }
    panic!("goal call {number} answered: {:?}", world.call_errors)
}

fn make(world: &mut World, goal: &engine::Assignment, words: &[u8], after: &[u64]) -> u64 {
    let answer = call(
        world,
        goal,
        engine::Tool::Change {
            changes: Box::new([engine::ChangeCall {
                landing: Some(engine::ChangeLanding::Goal),
                after: after.iter().copied().map(engine::ChangeAfter::Task).collect(),
                ..engine::ChangeCall::words(words.into(), Some(b"org/repo".as_slice().into()))
            }]),
        },
    );
    let engine::CallAnswer::Delegated(made) = answer else { panic!("change admitted: {answer:?}") };
    made[0].task
}

fn make_c2_c3(world: &mut World, goal: &engine::Assignment) -> (u64, u64) {
    let input = smith_domain_run::HostInput::attested(
        br#"{"changes":[{"name":"c2","words":"Add C2","landing":"goal"},{"words":"Add C3","landing":"goal","after":["c2"]}]}"#.as_slice().into(),
    ).expect("bounded changes");
    let temper_engine_smith::Decoded::Call(decoded) = temper_engine_smith::call(
        smith_domain_run::CallName { activation: goal.attempt, completion: 1, position: 1 },
        b"change",
        &input,
    )
    .expect("change decodes") else {
        panic!("change call")
    };
    let engine::CallAnswer::Delegated(made) = call(world, goal, decoded.tool) else { panic!("two changes") };
    assert_eq!(made.len(), TWO_CHANGES);
    (made[0].task, made[1].task)
}

fn observe_push(world: &mut World, producer: &engine::Assignment) {
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::Branch(
                producer.workspace.repositories[0].push.clone().expect("push branch"),
            ),
            key: None,
        },
    })));
}

struct EarlyStory {
    world: World,
    goal: engine::Assignment,
    branch: Box<[u8]>,
    base: u64,
    dependency: u64,
    change: u64,
    upstream: engine::Assignment,
    producer: engine::Assignment,
    head: u64,
}

fn early_story(upstream_words: &[u8], words: &[u8]) -> EarlyStory {
    let (mut world, goal, branch, base) = goal_world();
    let dependency = make(&mut world, &goal, upstream_words, &[]);
    let upstream = reviews::next_assignment(&mut world, 1);
    let head = push(&mut world, &upstream, base, b"upstream");
    observe_push(&mut world, &upstream);
    let change = make(&mut world, &goal, words, &[dependency]);
    let producer = reviews::next_assignment(&mut world, 2);
    EarlyStory { world, goal, branch, base, dependency, change, upstream, producer, head }
}

fn acknowledge_cancel(world: &mut World, task: u64) {
    assert!(
        matches!(&live_task(world, task).phase,
        tasks::Phase::Closing(closing) if matches!(closing.ending, tasks::Ending::Cancelled { .. })),
        "producer receives cancellation: {:?}",
        live_task(world, task).phase
    );
    let run = world.assigned.iter().rev().find(|run| run.task == task).expect("cancelled run").clone();
    let event = world.protocol.answer(CHANNEL, task, run.attempt, 1, tasks::End::Refused, engine::Parts::none());
    world.send(event);
    ticks(world);
}

fn no_hold_or_escalation(world: &World, task: u64) {
    assert!(matches!(live_task(world, task).phase, tasks::Phase::Active(_)));
    assert!(matches!(live_task(world, task).escalation, tasks::Escalation::Unheld { .. }));
}

fn merged_start(world: &mut World, base: u64, head: u64) -> u64 {
    let mut tree: std::collections::BTreeMap<_, _> = world
        .fake
        .object(base)
        .expect("base head")
        .tree
        .iter()
        .map(|(path, content)| (path.clone(), content.clone()))
        .collect();
    tree.extend(
        world
            .fake
            .object(head)
            .expect("dependency head")
            .tree
            .iter()
            .map(|(path, content)| (path.clone(), content.clone())),
    );
    let files = tree.into_iter().map(|(path, content)| raw::File { path, content }).collect();
    fake::merge_commit(&mut world.fake, &world.fake_env.limits, base, head, files, b"Merge dependency")
        .expect("prepared start")
}

fn landed(world: &mut World, task: u64) {
    for _ in 0..STEPS {
        world.tick();
        if ended(world, task) {
            break;
        }
    }
    assert!(
        matches!(change_row(world, task).change.state, change::State::Landed { .. }),
        "change {task} landed: {:?}",
        change_row(world, task)
    );
    assert!(ended(world, task));
}

fn stop(world: &mut World, goal: &engine::Assignment) {
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(99)),
        sign_in: world.signed_in.expect("owner session"),
        key: [99; 16],
        ask: people::Ask::Cancel {
            project: 1,
            task: goal.task,
            reason: b"Story observed".as_slice().into(),
            grace: None,
        },
    }));
    ticks(world);
    for run in world.assigned.clone() {
        if !ended(world, run.task) {
            let event =
                world.protocol.answer(CHANNEL, run.task, run.attempt, 5, tasks::End::Parked, engine::Parts::none());
            world.send(event);
        }
    }
    for _ in 0..STEPS {
        world.tick();
        if ended(world, goal.task) {
            break;
        }
    }
    assert!(ended(world, goal.task), "story's goal cancelled");
    world.close();
}

fn ticks(world: &mut World) {
    for _ in 0..QUIET_TICKS {
        world.tick();
    }
}

fn active_without_producer(world: &World, task: u64) {
    assert!(matches!(live_task(world, task).phase, tasks::Phase::Active(_)));
    assert_eq!(change_row(world, task).delegate, None, "{:?}", change_row(world, task));
}

#[test]
fn a_dependent_waits_for_its_dependencys_first_push() {
    let (mut world, goal, branch, main) = goal_world();
    let c2 = make(&mut world, &goal, b"Add C2", &[]);
    let producer_c2 = reviews::next_assignment(&mut world, 1);
    let c3 = make(&mut world, &goal, b"Add C3", &[c2]);
    let stall = world.env.limits.forge.change_policy.stall;
    let past_stall = world.env.now.saturating_add(stall).saturating_add(Duration::from_secs(1));
    while world.env.now < past_stall {
        world.tick();
    }
    active_without_producer(&world, c3);
    assert_eq!(live_task(&world, c3).waiting_on.as_ref(), &[c2]);
    let head = push(&mut world, &producer_c2, main, b"C2");
    observe_push(&mut world, &producer_c2);
    let producer_c3 = reviews::next_assignment(&mut world, 2);
    assert_eq!(live_task(&world, producer_c3.task).requester, tasks::Party::Task(c3));
    assert_eq!(
        producer_c3.workspace.repositories[0].start,
        engine::ForgeStart::Merged { branch: branch.clone(), heads: Box::new([translate::commit(head)]) }
    );
    assert_eq!(live_task(&world, c3).started_from.as_ref(), &[c2]);
    stop(&mut world, &goal);
}

#[test]
fn a_dependency_kept_by_its_own_dependency_holds_the_early_start() {
    let (mut world, goal, branch, main) = goal_world();
    let c1 = make(&mut world, &goal, b"Add C1", &[]);
    let producer_c1 = reviews::next_assignment(&mut world, 1);
    let c1_head = push(&mut world, &producer_c1, main, b"C1");
    observe_push(&mut world, &producer_c1);
    let c2 = make(&mut world, &goal, b"Add C2", &[c1]);
    let producer_c2 = reviews::next_assignment(&mut world, 2);
    assert_eq!(
        producer_c2.workspace.repositories[0].start,
        engine::ForgeStart::Merged { branch: branch.clone(), heads: Box::new([translate::commit(c1_head)]) }
    );
    let start = merged_start(&mut world, main, c1_head);
    let c2_head = push(&mut world, &producer_c2, start, b"C2");
    observe_push(&mut world, &producer_c2);
    let c3 = make(&mut world, &goal, b"Add C3", &[c2]);
    ticks(&mut world);
    active_without_producer(&world, c3);
    assert_eq!(live_task(&world, c2).waiting_on.as_ref(), &[c1]);
    assert_eq!(live_task(&world, c3).waiting_on.as_ref(), &[c2]);
    finish(&mut world, &producer_c1, b"Add C1");
    landed(&mut world, c1);
    let producer_c3 = reviews::next_assignment(&mut world, 3);
    assert_eq!(live_task(&world, producer_c3.task).requester, tasks::Party::Task(c3));
    assert_eq!(
        producer_c3.workspace.repositories[0].start,
        engine::ForgeStart::Merged { branch, heads: Box::new([translate::commit(c2_head)]) }
    );
    assert!(!ended(&world, c2));
    stop(&mut world, &goal);
}

#[test]
fn an_early_start_that_conflicts_waits_for_its_dependency_to_land() {
    let EarlyStory { mut world, goal, branch, dependency: c7, change: c8, upstream, producer, head, .. } =
        early_story(b"Add C7", b"Add C8");
    world.until(Until::Ready);
    world.send(protocol::failed_preparation(
        CHANNEL,
        producer.task,
        producer.attempt,
        engine::Preparation::Conflicted { tag: producer.workspace.repositories[0].tag, head: translate::commit(head) },
    ));
    ticks(&mut world);
    let facts: Vec<_> = std::iter::from_fn(|| world.root.pop_fact()).collect();
    assert!(
        facts.contains(&engine::Fact::PreparationFailed {
            task: producer.task,
            attempt: producer.attempt,
            preparation: engine::Preparation::Conflicted {
                tag: producer.workspace.repositories[0].tag,
                head: translate::commit(head)
            },
        }),
        "the refused preparation names C7's head: {facts:?}"
    );
    assert!(matches!(change_row(&world, c8).change.early, change::Early::Spent(_)), "conflict spent: {facts:?}");
    acknowledge_cancel(&mut world, producer.task);
    active_without_producer(&world, c8);
    no_hold_or_escalation(&world, c7);
    no_hold_or_escalation(&world, c8);
    no_hold_or_escalation(&world, goal.task);
    assert_eq!(
        change_row(&world, c8).change.early,
        change::Early::Spent(Box::new([change::StartedFrom { task: c7, head: translate::commit(head) }]))
    );
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(cancelled)))) =
        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(producer.task))))
    else {
        panic!("conflicting producer cancelled")
    };
    assert!(matches!(cancelled.phase, tasks::Phase::Ended(tasks::Ending::Cancelled { .. })));
    assert_eq!(live_task(&world, c8).started_from.as_ref(), &[c7]);
    let next_index = world.assigned.len();
    finish(&mut world, &upstream, b"Add C7");
    landed(&mut world, c7);
    let next = reviews::next_assignment(&mut world, next_index);
    assert_eq!(live_task(&world, next.task).requester, tasks::Party::Task(c8));
    assert_eq!(next.workspace.repositories[0].start, engine::ForgeStart::Base(branch.clone()));
    let tip = branch_tip(&mut world, &branch);
    push(&mut world, &next, tip, b"C8");
    finish(&mut world, &next, b"Add C8");
    landed(&mut world, c8);
    stop(&mut world, &goal);
}

#[test]
fn a_failed_early_producer_waits_and_the_next_merges_the_landing_tip() {
    let EarlyStory { mut world, goal, branch, base, dependency: c4, change: c6, upstream, producer, head } =
        early_story(b"Add C4", b"Add C6");
    let start = merged_start(&mut world, base, head);
    let c6_head = push(&mut world, &producer, start, b"C6");
    let event = world.protocol.answer(
        CHANNEL,
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Failure { reason: b"C4 still has stubs".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts {
            saved: None,
            pushed: Box::new([forge_top::Pushed {
                tag: producer.workspace.repositories[0].tag,
                commit: translate::commit(c6_head),
            }]),
            preparation: None,
        },
    );
    world.send(event);
    ticks(&mut world);
    active_without_producer(&world, c6);
    no_hold_or_escalation(&world, c6);
    no_hold_or_escalation(&world, goal.task);
    assert!(matches!(change_row(&world, c6).change.early, change::Early::Spent(_)));
    assert_eq!(change_row(&world, c6).pull, None);
    let next_index = world.assigned.len();
    finish(&mut world, &upstream, b"Add C4");
    landed(&mut world, c4);
    let next = reviews::next_assignment(&mut world, next_index);
    let tip = branch_tip(&mut world, &branch);
    let c6_branch = producer.workspace.repositories[0].push.clone().expect("C6 branch");
    assert_ne!(next.task, producer.task, "the once rule makes a new producer");
    assert_eq!(live_task(&world, next.task).requester, tasks::Party::Task(c6));
    assert_eq!(
        next.workspace.repositories[0].start,
        engine::ForgeStart::Merge { branch: c6_branch.clone(), base: translate::commit(tip) }
    );
    let merged = merged_start(&mut world, c6_head, tip);
    assert_eq!(
        world.external(2, raw::Op::Git(raw::Git::Push { branch: c6_branch, commit: merged, expected: Some(c6_head) })),
        raw::Answer::Pushed(raw::Pushed::Pushed)
    );
    finish(&mut world, &next, b"Add C6");
    landed(&mut world, c6);
    stop(&mut world, &goal);
}

#[test]
fn an_early_producer_done_while_kept_waits_then_merges() {
    let EarlyStory { mut world, goal, base, dependency: c2, change: c3, upstream, producer, head, .. } =
        early_story(b"Add C2", b"Add C3");
    let start = merged_start(&mut world, base, head);
    let c3_head = push(&mut world, &producer, start, b"C3");
    finish(&mut world, &producer, b"Add C3");
    for _ in 0..STEPS {
        world.tick();
        if change_row(&world, c3).pull.is_some() {
            break;
        }
    }
    assert!(change_row(&world, c3).pull.is_some(), "C3 opens while C2 produces");
    assert!(matches!(live_task(&world, upstream.task).phase, tasks::Phase::Active(tasks::Active::Running { .. })));
    assert_eq!(live_task(&world, c3).waiting_on.as_ref(), &[c2]);
    assert!(change_row(&world, c3).change.ready_since.is_none());
    assert_eq!(world.merges, 0);
    let runs = world.assigned.len();
    finish(&mut world, &upstream, b"Add C2");
    landed(&mut world, c3);
    assert!(ended(&world, c2));
    assert_eq!(world.assigned.len(), runs, "C3 needs no further run");
    assert_eq!(change_row(&world, c3).change.last_head, Some(translate::commit(c3_head)));
    assert_eq!(world.merges, TWO_MERGES);
    stop(&mut world, &goal);
}

#[test]
fn an_amendment_dropping_a_dependency_started_from_is_refused_naming_it() {
    let (mut world, goal, _, base) = goal_world();
    let (c2, c3) = make_c2_c3(&mut world, &goal);
    let upstream = reviews::next_assignment(&mut world, 1);
    push(&mut world, &upstream, base, b"C2");
    observe_push(&mut world, &upstream);
    let producer = reviews::next_assignment(&mut world, 2);
    let revision = spec_revision(&world, c3);
    let mut amendment = temper_engine_domain_world::fixtures::task_amendment(None, None, b"Drop C2");
    amendment.dependencies = Some(Box::new([]));
    let answer = call(&mut world, &goal, engine::Tool::Amend { target: c3, revision, amendment });
    let engine::CallAnswer::ControlRefused(problem) = answer else { panic!("amendment refused: {answer:?}") };
    assert_eq!(problem.why, tasks::Refusal::Dependencies);
    assert_eq!(problem.blocked_by.as_deref(), Some([c2].as_slice()));
    assert_eq!(live_task(&world, c3).dependencies.as_ref(), &[c2]);
    assert_eq!(spec_revision(&world, c3), revision);
    assert_eq!(
        call(&mut world, &goal, engine::Tool::Cancel { target: c3, reason: b"Make C3 without C2".as_slice().into() }),
        engine::CallAnswer::Controlled
    );
    ticks(&mut world);
    acknowledge_cancel(&mut world, producer.task);
    assert!(ended(&world, c3));
    let next_index = world.assigned.len();
    let replacement = make(&mut world, &goal, b"Add C3", &[]);
    assert_ne!(replacement, c3);
    assert!(live_task(&world, replacement).dependencies.is_empty());
    assert!(live_task(&world, replacement).started_from.is_empty());
    let next = reviews::next_assignment(&mut world, next_index);
    assert_eq!(live_task(&world, next.task).requester, tasks::Party::Task(replacement));
    stop(&mut world, &goal);
}

#[test]
fn a_dependency_into_another_repository_does_not_wait_for_a_push() {
    let mut world = world_with(true);
    world.adopted.clear();
    world.send(engine::Event::Party(engine::PartyInput::Application {
        scope: people::ApplicationScope::Project(1),
        reply_to: ReplyTo::new(Token::new(94)),
        sign_in: world.signed_in.expect("owner session"),
        key: [94; 16],
        digest: [6; 32],
        carried: None,
        request: engine::Application::Forge(forge_top::Ask::Adopt(forge_top::Adoption {
            project: 1,
            home: false,
            forge: forge_world::REPO.forge,
            host: b"forge.example".as_slice().into(),
            owner: b"org".as_slice().into(),
            name: b"lib".as_slice().into(),
            prefix: b"temper/".as_slice().into(),
            role: forge_top::Role::Owned,
            landing: b"main".as_slice().into(),
            ci: true,
            checks: Box::new([]),
        })),
    }));
    world.until(Until::Adopted);
    assert!(
        matches!(world.adopted.as_slice(), [forge_top::ApplicationOutcome::Adopted(_)]),
        "library adopted: {:?}",
        world.adopted
    );
    let (mut world, goal, _, _) = goal_world_in(world);
    let upstream = make(&mut world, &goal, b"Change repo", &[]);
    reviews::next_assignment(&mut world, 1);
    let answer = call(
        &mut world,
        &goal,
        engine::Tool::Change {
            changes: Box::new([engine::ChangeCall {
                after: Box::new([engine::ChangeAfter::Task(upstream)]),
                ..engine::ChangeCall::words(b"Change lib".as_slice().into(), Some(b"org/lib".as_slice().into()))
            }]),
        },
    );
    let engine::CallAnswer::Delegated(made) = answer else { panic!("dependent admitted: {answer:?}") };
    let dependent = made[0].task;
    let producer = reviews::next_assignment(&mut world, 2);
    assert_eq!(live_task(&world, producer.task).requester, tasks::Party::Task(dependent));
    assert_eq!(producer.workspace.repositories[0].provider, skein_forge::api::Repository { forge: 1, repository: 4 });
    assert_eq!(producer.workspace.repositories[0].start, engine::ForgeStart::Base(b"main".as_slice().into()));
    assert_eq!(change_row(&world, upstream).change.last_head, None);
    assert_eq!(world.fake.branch(b"org/repo", &change_row(&world, upstream).branch), None);
    assert_eq!(live_task(&world, dependent).waiting_on.as_ref(), &[upstream]);
    assert!(live_task(&world, dependent).started_from.is_empty());
    assert_eq!(change_row(&world, dependent).change.early, change::Early::Started(Box::new([])));
    no_hold_or_escalation(&world, upstream);
    no_hold_or_escalation(&world, dependent);
    stop(&mut world, &goal);
}

#[test]
fn a_dependent_change_starts_early_on_its_dependencys_pushed_head() {
    let (mut world, goal, branch, main) = goal_world();
    let (c2, c3) = make_c2_c3(&mut world, &goal);
    assert!(matches!(live_task(&world, c3).phase, tasks::Phase::Active(_)));
    assert_eq!(live_task(&world, c3).waiting_on.as_ref(), &[c2]);
    assert_eq!(change_row(&world, c3).delegate, None);
    let producer_c2 = reviews::next_assignment(&mut world, 1);
    let c2_head = push(&mut world, &producer_c2, main, b"C2");
    // Scripted workers deliver with their terminal answer. Mergeability keeps C2 pending
    // after that answer, while C3 produces and opens its own pull.
    world.fake_env.limits.settle_min = Duration::from_secs(20);
    world.fake_env.limits.settle_max = Duration::from_secs(20);
    finish(&mut world, &producer_c2, b"Add C2");
    let producer_c3 = reviews::next_assignment(&mut world, 2);
    assert!(!ended(&world, c2));
    assert_eq!(live_task(&world, producer_c3.task).requester, tasks::Party::Task(c3));
    assert_eq!(
        producer_c3.workspace.repositories[0].start,
        engine::ForgeStart::Merged { branch: branch.clone(), heads: Box::new([translate::commit(c2_head)]) }
    );
    assert_eq!(live_task(&world, c3).started_from.as_ref(), &[c2]);
    let files = world
        .fake
        .object(c2_head)
        .expect("dependency head")
        .tree
        .iter()
        .map(|(path, content)| raw::File { path: path.clone(), content: content.clone() })
        .collect();
    let start = fake::merge_commit(&mut world.fake, &world.fake_env.limits, main, c2_head, files, b"Merge C2")
        .expect("prepared start");
    let c3_head = push(&mut world, &producer_c3, start, b"C3");
    world.fake_env.limits.settle_min = Duration::ZERO;
    world.fake_env.limits.settle_max = Duration::ZERO;
    finish(&mut world, &producer_c3, b"Add C3");
    for _ in 0..STEPS {
        world.tick();
        if change_row(&world, c3).pull.is_some() {
            break;
        }
    }
    assert!(change_row(&world, c3).pull.is_some(), "C3 opens while C2 is pending");
    assert!(!ended(&world, c2));
    assert_eq!(live_task(&world, c3).waiting_on.as_ref(), &[c2]);
    assert!(change_row(&world, c3).change.ready_since.is_none(), "a kept change is not ready");
    assert_eq!(world.merges, 0);
    let pull = change_row(&world, c2).pull.expect("C2 opened");
    fake::check(&mut world.fake, &world.fake_env, b"org/repo", pull, Duration::ZERO);
    for _ in 0..STEPS {
        world.tick();
        if ended(&world, c3) {
            break;
        }
    }
    assert!(ended(&world, c2), "C2 lands");
    assert!(ended(&world, c3), "C3 lands after C2");
    assert_eq!(world.merges, TWO_MERGES);
    let change::State::Landed { merge, .. } = change_row(&world, c3).change.state else { panic!("C3 landed") };
    let merged_tip = branch_tip(&mut world, &branch);
    assert_eq!(translate::commit(merged_tip), merge);
    let raw::Answer::Commits { commits } =
        world.external(1, raw::Op::Read(raw::Read::Commits { before: main, after: merged_tip }))
    else {
        panic!("goal history")
    };
    let change::State::Landed { merge: c2_merge, .. } = change_row(&world, c2).change.state else {
        panic!("C2 landed")
    };
    assert_eq!(commits.len(), LANDINGS, "each change contributes one squash landing");
    let c2_landing = commits.iter().find(|commit| translate::commit(commit.id) == c2_merge).expect("C2 landing");
    let c3_landing = commits.iter().find(|commit| commit.id == merged_tip).expect("C3 landing");
    assert_eq!(c2_landing.parents.as_ref(), &[main]);
    assert_eq!(c3_landing.parents.as_ref(), &[c2_landing.id]);
    assert_landed_tree(&world, main, merged_tip, c2_head, c3_head);
    assert_eq!(change_row(&world, c3).change.last_head, Some(translate::commit(c3_head)));
    world.close();
}
