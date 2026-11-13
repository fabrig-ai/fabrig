//! A goal catches its branch up through a change (temper/docs/design/domain/forge/README.md, 6).
use super::goal_branch::{branch_tip, goal_request};
use super::light_rules::change_row;
use super::*;
use jig_domain_link::JobKind;
use temper_engine_domain_forge_change as change;

const CHANNEL: Token = Token::new(7);
const STEPS: u32 = 1000;
const COMMAND: &[u8] = b".temper/merge";
const LEFT: &[u8] = b"{\"left\":[\"file\"]}";
const MAIN_PATH: &[u8] = b"main";
const MAIN_WORK: &[u8] = b"main work";

fn goal_world(command: bool, kind: bool, control: bool) -> (World, engine::Assignment, Box<[u8]>, u64) {
    goal_world_with_limits(command, kind, control, &StoryLimits::default())
}

pub(super) fn goal_world_with_limits(
    command: bool,
    kind: bool,
    control: bool,
    limits: &StoryLimits,
) -> (World, engine::Assignment, Box<[u8]>, u64) {
    let mut world = World::configured_limits(
        true,
        true,
        true,
        STEPS,
        Gate::default(),
        None,
        StoryLimits { merge_kind: true, merge_authority: kind, ..*limits },
    )
    .with_landing_deployment(Box::new([engine::LandingRule {
        sets: Box::new([]),
        connector: 0,
        kind: 4,
        enforce: true,
        pattern: authority::Pattern {
            segments: Box::new([b"forge".as_slice().into()]),
            last: authority::Last::Open(Box::new([])),
        },
        ci: false,
        up_to_date: false,
        gates: Box::new([]),
        approvals: Box::new([]),
    }]));
    if command {
        world = world.with_merge_job(COMMAND);
    }
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, 2, Box::new([])));
    let people::Ask::Create { project, mut batch } = goal_request(&world, true) else { panic!("goal request") };
    let family = jig_domain_core::ToolFamilies::JIG.control.0;
    if control {
        batch[0].authority.tools |= family;
    } else {
        batch[0].authority.tools &= !family;
    }
    let goal = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    world.until(Until::Ready);
    let head = branch_tip(&mut world, &branch);
    (world, goal, branch, head)
}

fn call(
    world: &mut World,
    goal: &engine::Assignment,
    input: &[u8],
) -> Result<Option<engine::CallAnswer>, temper_engine_smith::Problem> {
    let input = smith_domain_run::HostInput::attested(input.into()).expect("bounded input");
    let temper_engine_smith::Decoded::Call(call) = temper_engine_smith::call(
        smith_domain_run::CallName { activation: goal.attempt, completion: 1, position: 1 },
        b"change",
        &input,
    )?
    else {
        panic!("change call")
    };
    let before = world.answers.len();
    let event = world.protocol.call(CHANNEL, goal.task, goal.attempt, 93, call);
    world.send(event);
    for _ in 0..STEPS {
        if world.answers.len() > before || !world.call_errors.is_empty() {
            return Ok(world.answers.get(before).cloned());
        }
        world.tick();
    }
    panic!("change was not answered")
}

pub(super) fn catch_up(world: &mut World, goal: &engine::Assignment, commit: u64) -> u64 {
    let text = forge_top::commit_text(&translate::commit(commit));
    let text = core::str::from_utf8(&text).expect("commit text");
    let input = format!(r#"{{"words":"Take main's changes","landing":"goal","merge":"{text}"}}"#);
    let Some(engine::CallAnswer::Delegated(made)) = call(world, goal, input.as_bytes()).expect("catch-up decoded")
    else {
        panic!("catch-up admitted: {:?}", world.call_errors)
    };
    let task = made[0].task;
    assert!(
        live_task(world, task)
            .spec
            .parameters
            .contains(&tasks::Parameter::Bytes { name: 16, value: forge_top::commit_text(&translate::commit(commit)) })
    );
    task
}

pub(super) fn next_job(world: &mut World, task: u64) -> host::JobAssignment {
    for _ in 0..STEPS {
        if !world.jobs.is_empty() {
            return world.jobs.remove(0);
        }
        world.tick();
    }
    panic!("merge job not assigned: {:?}; errors {:?}", change_row(world, task), world.call_errors)
}

pub(super) fn check_job(world: &World, task: u64, job: &host::JobAssignment, branch: &[u8], head: u64, base: u64) {
    assert_eq!(job.kind, JobKind(3));
    assert_eq!(
        change_row(world, task).delegate,
        Some((job.run.raw(), change::Delegate::Merge { base: translate::commit(base), from: None }))
    );
    assert_eq!(live_task(world, job.run.raw()).requester, tasks::Party::Task(task));
    assert_eq!(change_row(world, task).catch_up, Some(translate::commit(base)));
    assert_eq!(change_row(world, task).base.as_ref(), branch);
    assert_eq!(change_row(world, task).change.resolutions, 0);
    let mut reader = skein_lib::Reader::new(&job.spec);
    let command_length = reader.u32().expect("command length");
    reader.bytes(command_length).expect("merge command");
    let request_length = reader.u32().expect("request length");
    let request = reader.bytes(request_length).expect("merge request");
    let head_text = forge_top::commit_text(&translate::commit(head));
    let base_text = forge_top::commit_text(&translate::commit(base));
    let expected = format!(
        "{{\"version\":1,\"head\":\"{}\",\"base\":\"{}\"}}",
        core::str::from_utf8(&head_text).expect("head text"),
        core::str::from_utf8(&base_text).expect("base text"),
    );
    assert_eq!(request, expected.as_bytes());
    assert!(reader.is_empty());
    let workspace = job.workspace.as_ref().expect("the merge job needs its writable repository item");
    assert_eq!(workspace.workstream, task, "the change owns the checkout cache");
    let items = &world.job_workspaces.get(&job.run.raw()).expect("typed job items").repositories;
    assert_eq!(items.len(), 1);
    let item = &items[0];
    assert_eq!(item.provider, change_row(world, task).repository);
    assert_eq!(item.start, engine::ForgeStart::Merge { branch: branch.into(), base: translate::commit(base) });
    assert_eq!(item.push.as_deref(), Some(change_row(world, task).branch.as_ref()));
}

pub(super) fn push_merge(world: &mut World, task: u64, head: u64, base: u64) -> u64 {
    let merged = fake::merge_commit(
        &mut world.fake,
        &world.fake_env.limits,
        head,
        base,
        Box::new([raw::File { path: MAIN_PATH.into(), content: MAIN_WORK.into() }]),
        b"Catch up",
    )
    .expect("merge with both parents");
    assert_eq!(
        world.external(
            2,
            raw::Op::Git(raw::Git::Push {
                branch: change_row(world, task).branch.clone(),
                commit: merged,
                expected: None,
            })
        ),
        raw::Answer::Pushed(raw::Pushed::Pushed)
    );
    merged
}

pub(super) fn finish_job(world: &mut World, job: &host::JobAssignment, result: &[u8]) {
    let event = world.protocol.answer_job(
        CHANNEL,
        job.run.raw(),
        job.attempt.raw(),
        host::JobEnding::Finished { result: result.into(), work: host::Work { left: None, saved: None } },
    );
    world.send(event);
}

fn landed(world: &mut World, task: u64, branch: &[u8], base: u64) {
    for _ in 0..STEPS {
        if matches!(change_row(world, task).change.state, change::State::Landed { .. }) {
            break;
        }
        world.tick();
    }
    assert!(
        matches!(change_row(world, task).change.state, change::State::Landed { .. }),
        "{:?}",
        change_row(world, task)
    );
    assert_eq!(world.merges, 1);
    let number = change_row(world, task).pull.expect("catch-up pull request");
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number })) else { panic!("pull") };
    assert_eq!(pull.base.as_ref(), branch);
    let tip = branch_tip(world, branch);
    assert_eq!(Some(tip), pull.merged);
    let raw::Answer::Comparison { contains_base, .. } =
        world.external(1, raw::Op::Read(raw::Read::Compare { base, head: pull.commit, page: 1, limit: 10 }))
    else {
        panic!("comparison")
    };
    assert!(contains_base, "the catch-up's pushed head contains main's commit");
    world.until(Until::Ready);
    assert_eq!(
        world.external(1, raw::Op::Read(raw::Read::File { commit: tip, path: MAIN_PATH.into() })),
        raw::Answer::File(MAIN_WORK.into()),
        "the goal branch holds main's work"
    );
    assert_eq!(branch_tip(world, b"main"), base);
}

/// temper/docs/design/domain/forge/README.md, 6: the merge job produces a catch-up at the goal tip.
#[test]
fn a_catch_up_merges_mains_commit_into_a_branch_at_the_goal_branchs_tip() {
    let (mut world, goal, branch, head) = goal_world(true, true, true);
    let base = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", MAIN_PATH, MAIN_WORK, 1)
        .expect("main advanced");
    let task = catch_up(&mut world, &goal, base);
    let job = next_job(&mut world, task);
    check_job(&world, task, &job, &branch, head, base);
    push_merge(&mut world, task, head, base);
    finish_job(&mut world, &job, b"{\"left\":[]}");
    landed(&mut world, task, &branch, base);
    assert_eq!(world.assigned.len(), 1, "the goal's coordinator is the only agent");
    world.close();
}

/// temper/docs/design/domain/forge/README.md, 6: the job's unresolved merge starts again at the goal tip.
#[test]
fn a_catch_up_the_merge_job_cannot_settle_goes_to_a_resolution() {
    let (mut world, goal, branch, head) = goal_world(true, true, true);
    let base = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", MAIN_PATH, MAIN_WORK, 1)
        .expect("main advanced");
    let task = catch_up(&mut world, &goal, base);
    let job = next_job(&mut world, task);
    check_job(&world, task, &job, &branch, head, base);
    finish_job(&mut world, &job, LEFT);
    let resolution = super::reviews::next_assignment(&mut world, 1);
    let item = &resolution.workspace.repositories[0];
    assert_eq!(item.start, engine::ForgeStart::Merge { branch: branch.clone(), base: translate::commit(base) });
    assert_eq!(item.push.as_deref(), Some(change_row(&world, task).branch.as_ref()));
    assert!(live_task(&world, resolution.task).spec.words.windows(LEFT.len()).any(|words| words == LEFT));
    assert_eq!(change_row(&world, task).change.resolutions, 1);
    push_merge(&mut world, task, head, base);
    let event = world.protocol.answer(
        CHANNEL,
        resolution.task,
        resolution.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(b"Catch up".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: b"resolved".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    landed(&mut world, task, &branch, base);
    world.close();
}

/// temper/docs/design/domain/forge/README.md, 6: a contained commit ends without a producer or pull.
#[test]
fn a_catch_up_already_contained_ends_with_no_change() {
    let (mut world, goal, _, _) = goal_world(true, true, true);
    let base = branch_tip(&mut world, b"main");
    let task = catch_up(&mut world, &goal, base);
    let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)));
    for _ in 0..STEPS {
        if world.store.rows.contains_key(&key) {
            break;
        }
        world.tick();
    }
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) = world.store.rows.get(&key)
    else {
        panic!("catch-up ended")
    };
    let tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::NoChange { words })) = &row.phase else {
        panic!("no change: {:?}", row.phase)
    };
    let commit = forge_top::commit_text(&translate::commit(base));
    assert_eq!(
        words.as_ref(),
        format!("The landing branch already contains commit {}.", core::str::from_utf8(&commit).expect("commit text"))
            .as_bytes()
    );
    assert_eq!(world.jobs.len(), 0);
    assert!(change_row(&world, task).delegate.is_none());
    assert!(change_row(&world, task).pull.is_none());
    assert_eq!(world.assigned.len(), 1);
    assert_eq!(world.merges, 0);
    world.close();
}

fn refused(command: bool, kind: bool, control: bool, words: &[u8]) {
    let (mut world, goal, _, _) = goal_world(command, kind, control);
    let before = world.store.rows.clone();
    let text = forge_top::commit_text(&translate::commit(1));
    // Leave out landing so merge alone must trigger the control-family rule.
    let input = format!(r#"{{"words":"Catch up","merge":"{}"}}"#, core::str::from_utf8(&text).expect("commit text"));
    assert_eq!(call(&mut world, &goal, input.as_bytes()), Ok(None));
    assert_eq!(world.call_errors[0].text.as_ref(), words);
    assert_eq!(world.store.rows, before, "refusal makes no change or delegate");
    world.close();
}

/// temper/docs/design/domain/forge/README.md, 6: a catch-up requires the project's `merge_job`.
#[test]
fn a_catch_up_is_refused_without_a_merge_job() {
    refused(false, true, true, b"change's merge: the project names no merge_job.");
}

/// temper/docs/design/domain/forge/README.md, 6: a catch-up requires delegation to Job(3).
#[test]
fn a_catch_up_is_refused_without_the_merge_kind() {
    refused(true, false, true, b"change's merge: its authority does not delegate to the merge job.");
}

/// temper/docs/design/domain/forge/README.md, 6: merge is a plan member requiring control.
#[test]
fn a_catch_up_is_refused_without_control() {
    refused(
        true,
        true,
        false,
        b"change's merge is a plan's: this task's authority lacks the control family (amend, cancel, release).",
    );
}

/// temper/docs/design/domain/forge/README.md, 6: a malformed commit refuses merge before creating a change.
#[test]
fn a_catch_up_is_refused_for_a_malformed_commit() {
    let (mut world, goal, _, _) = goal_world(true, true, true);
    let before = world.store.rows.clone();
    let problem = call(&mut world, &goal, br#"{"words":"Catch up","landing":"goal","merge":"not a commit"}"#)
        .expect_err("malformed merge");
    assert_eq!(problem, temper_engine_smith::Problem::Range(b"merge"));
    assert_eq!(world.store.rows, before);
    world.close();
}
