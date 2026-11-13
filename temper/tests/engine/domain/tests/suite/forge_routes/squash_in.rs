//! A goal's train preserves main's commits and starts follow-ups at its tip
//! (temper/docs/design/draft/squash-in.md, 2, 4; temper/docs/design/domain/landing.md, 5, 8, 9).
use super::catch_up::{catch_up, check_job, finish_job, next_job, push_merge};
use super::goal_branch::{branch_tip, forge_hold};
use super::light_rules::change_row;
use super::*;
use temper_engine_domain_forge_change as change;

const CHANNEL: Token = Token::new(7);
const STEPS: u32 = 1000;
const LEFT: &[u8] = b"{\"left\":[\"file\"]}";

fn goal_world() -> (World, engine::Assignment, Box<[u8]>, u64) {
    // The prepare carries both messages and the goal train's upstream beside its member.
    let (mut world, goal, branch, head) = super::catch_up::goal_world_with_limits(
        true,
        true,
        true,
        StoryLimits { turn: true, spec_bytes: Some(4096), ..StoryLimits::default() },
    );
    world.manual_prepare_jobs();
    assert_eq!(forge_hold(&world, &branch).expect("goal holds its train").task, goal.task);
    assert_eq!(branch_tip(&mut world, b"main"), head, "G starts at main's tip");
    (world, goal, branch, head)
}

fn main_moves(world: &mut World) -> u64 {
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"main", b"main work", 1)
        .expect("main gains M")
}

fn landing(world: &World, change: u64) -> Option<&forge_top::LandingRow> {
    world.store.rows.values().find_map(|record| match record {
        Record::Forge { row } => match row.as_ref() {
            forge_top::Stored::Landing(row) if row.landing.members.iter().any(|member| member.change == change) => {
                Some(row)
            }
            _ => None,
        },
        Record::Core(_) => None,
    })
}

fn ended(world: &World, task: u64) -> Option<&tasks::TaskRecord> {
    match world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)))) {
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) => Some(row),
        _ => None,
    }
}

fn prepare(world: &mut World, task: u64, branch: &[u8], base: u64, head: u64, upstream: u64) {
    for _ in 0..STEPS {
        if world.jobs.iter().any(|job| job.kind == jig_domain_link::JobKind(1)) {
            break;
        }
        world.tick();
    }
    let row = landing(world, task).expect("a landing forms on G's train");
    assert_eq!(row.landing.branch.as_ref(), branch);
    assert_eq!(row.landing.members.len(), 1);
    let member = &row.landing.members[0];
    assert_eq!(member.base, Some(translate::commit(base)));
    assert_eq!(member.head, Some(translate::commit(head)));
    assert_eq!(row.landing.round.upstream, Some(translate::commit(upstream)));
    let job = world
        .jobs
        .iter()
        .find(|job| job.kind == jig_domain_link::JobKind(1))
        .unwrap_or_else(|| panic!("prepare delegated: {row:?}; jobs={:?}", world.jobs));
    assert_eq!(live_task(world, job.run.raw()).requester, tasks::Party::Task(row.landing.task));
    let spec = temper_protocol_worker_wire::v1::JobSpec::decode(
        &temper_protocol_worker_wire::v1::CEILINGS,
        &mut skein_lib::Reader::new(&job.spec),
    )
    .expect("prepare spec");
    let request: serde_json::Value = serde_json::from_slice(spec.request()).expect("prepare request");
    assert_eq!(request["combine"], "squash");
    let text = |commit| String::from_utf8(forge_top::commit_text(&translate::commit(commit)).into_vec()).expect("hex");
    assert_eq!(request["main"], text(base));
    assert_eq!(request["members"][0]["head"], text(head));
    assert_eq!(request["train_upstream"], text(upstream));
    world.automatic_prepare_jobs();
}

fn landed(world: &mut World, task: u64, branch: &[u8], base: u64, head: u64, merged: bool) -> u64 {
    for _ in 0..STEPS {
        world.tick();
        if ended(world, task).is_some() {
            break;
        }
    }
    let row = change_row(world, task);
    assert!(matches!(row.change.state, change::State::Landed { .. }), "{row:?}");
    assert!(matches!(
        ended(world, task).expect("change ends").phase,
        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Change { .. }))
    ));
    let round = &landing(world, task).expect("landed round").landing;
    let candidate = round.members[0].candidate.expect("prepared candidate");
    assert_eq!(round.members[0].landed, Some(candidate));
    assert_eq!(
        round.round.combined,
        Some(if merged { forge_top::LandingCombine::Merge } else { forge_top::LandingCombine::Squash })
    );
    let tip = branch_tip(world, branch);
    assert_eq!(translate::commit(tip), candidate, "the turn advances G to its candidate");
    assert!(world.fake.is_ancestor(base, tip));
    assert_eq!(world.fake.is_ancestor(head, tip), merged, "only a merge retains the producer's head");
    assert!(world.writes.iter().any(|write| matches!(write,
        skein_forge::api::Write::Merge { head, how: skein_forge::api::MergeHow::FastForwardOnly, .. }
            if *head == candidate)));
    tip
}

#[test]
fn a_clean_catch_up_lands_merged_on_the_goals_turn() {
    let (mut world, goal, branch, head) = goal_world();
    let main = main_moves(&mut world);
    let task = catch_up(&mut world, &goal, main);
    let job = next_job(&mut world, task);
    check_job(&world, task, &job, &branch, head, main);
    let merged = push_merge(&mut world, task, head, main);
    finish_job(&mut world, &job, b"{\"left\":[]}");
    prepare(&mut world, task, &branch, head, merged, main);
    let tip = landed(&mut world, task, &branch, head, merged, true);
    assert!(world.fake.is_ancestor(main, tip), "M stays main's commit");
    assert_eq!(branch_tip(&mut world, b"main"), main);
    assert_eq!(world.assigned.len(), 1, "only the goal coordinator is an agent");
    assert_eq!(world.merges, 1);
    world.close();
}

fn finish_producer(world: &mut World, producer: &engine::Assignment) {
    let event = world.protocol.answer(
        CHANNEL,
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(b"Change".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: b"pushed".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
}

#[test]
fn a_conflicting_catch_up_resolves_then_lands_merged_on_the_goals_turn() {
    let (mut world, goal, branch, head) = goal_world();
    let main = main_moves(&mut world);
    let task = catch_up(&mut world, &goal, main);
    let job = next_job(&mut world, task);
    check_job(&world, task, &job, &branch, head, main);
    finish_job(&mut world, &job, LEFT);
    let resolution = super::reviews::next_assignment(&mut world, 1);
    assert_eq!(live_task(&world, resolution.task).requester, tasks::Party::Task(task));
    assert_eq!(resolution.charter, change_row(&world, task).producing_charter);
    assert!(live_task(&world, resolution.task).spec.words.windows(LEFT.len()).any(|words| words == LEFT));
    assert_eq!(change_row(&world, task).change.resolutions, 1);
    let item = &resolution.workspace.repositories[0];
    assert_eq!(item.start, engine::ForgeStart::Merge { branch: branch.clone(), base: translate::commit(main) });
    assert_eq!(item.push.as_deref(), Some(change_row(&world, task).branch.as_ref()));
    let merged = push_merge(&mut world, task, head, main);
    finish_producer(&mut world, &resolution);
    prepare(&mut world, task, &branch, head, merged, main);
    let tip = landed(&mut world, task, &branch, head, merged, true);
    assert!(world.fake.is_ancestor(main, tip));
    assert_eq!(world.assigned.len(), 2, "goal and resolution are the only agents");
    assert_eq!(world.merges, 1);
    world.close();
}

#[test]
fn a_contained_catch_up_ends_without_a_landing_under_turn() {
    let (mut world, goal, branch, head) = goal_world();
    let task = catch_up(&mut world, &goal, head);
    for _ in 0..STEPS {
        if ended(&world, task).is_some() {
            break;
        }
        world.tick();
    }
    assert!(matches!(
        ended(&world, task).expect("contained catch-up ends").phase,
        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::NoChange { .. }))
    ));
    assert!(landing(&world, task).is_none());
    assert!(change_row(&world, task).delegate.is_none());
    assert!(change_row(&world, task).pull.is_none());
    assert!(world.jobs.is_empty());
    assert_eq!(world.assigned.len(), 1);
    assert_eq!(world.merges, 0);
    assert_eq!(branch_tip(&mut world, &branch), head);
    world.close();
}

fn new_change(world: &mut World, goal: &engine::Assignment, call: u64) -> (u64, engine::Assignment) {
    let index = world.assigned.len();
    let engine::CallAnswer::Delegated(made) =
        super::change_tool::change_landing(world, goal, call, None, b"Follow-up", Some(engine::ChangeLanding::Goal))
    else {
        panic!("change delegated")
    };
    let task = made[0].task;
    let producer = super::reviews::next_assignment(world, index);
    assert_eq!(live_task(world, producer.task).requester, tasks::Party::Task(task));
    (task, producer)
}

#[test]
fn a_change_after_a_squashed_turn_starts_from_the_goal_tip_on_its_own_branch() {
    let (mut world, goal, branch, base) = goal_world();
    let (a, producer_a) = new_change(&mut world, &goal, 94);
    let a_branch = producer_a.workspace.repositories[0].push.clone().expect("A branch");
    assert_eq!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: a_branch.clone(), commit: base })),
        raw::Answer::Branch(raw::Created::Created)
    );
    let a_head =
        fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &a_branch, b"A", b"added", 2).expect("A pushed");
    finish_producer(&mut world, &producer_a);
    prepare(&mut world, a, &branch, base, a_head, base);
    let squash = landed(&mut world, a, &branch, base, a_head, false);
    let object = world.fake.object(squash).expect("A squash");
    assert_eq!(object.parent, Some(base));
    assert_eq!(object.merge_parent, None);
    assert_ne!(squash, a_head);
    world.manual_prepare_jobs();
    let (b, producer_b) = new_change(&mut world, &goal, 95);
    let item = &producer_b.workspace.repositories[0];
    assert_eq!(item.start, engine::ForgeStart::Base(branch.clone()));
    assert_eq!(item.landing.as_deref(), Some(branch.as_ref()));
    let own_branch = format!("temper/{}/c{b}", live_task(&world, b).root).into_bytes();
    assert_eq!(item.push.as_deref(), Some(own_branch.as_slice()));
    assert_ne!(item.push.as_deref(), Some(a_branch.as_ref()));
    assert_eq!(branch_tip(&mut world, &branch), squash);
    assert!(
        world
            .fake
            .object(squash)
            .expect("G tip")
            .tree
            .iter()
            .any(|(path, bytes)| path.as_ref() == b"A" && bytes.as_ref() == b"added")
    );
    world.close();
}
