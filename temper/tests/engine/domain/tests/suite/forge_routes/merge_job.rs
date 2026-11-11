//! Merge jobs precede resolutions (temper/docs/design/domain/forge/change/repairs.md).
use super::light_rules::{change_row, pushed_change_in};
use super::*;
use jig_domain_link::JobKind;
use temper_engine_domain_forge_change as change;

const CHANNEL: Token = Token::new(7);
const COMMAND: &[u8] = b".temper/merge";
const RESULT: &[u8] = b"{\"settled\":[\"file\"],\"left\":[]}";
const GOAL: &[u8] = b"temper/goal";
const STEPS: u32 = 200;

#[path = "merge_job/producer.rs"]
mod producer;

fn world(up_to_date: bool, kind: bool, authority: bool) -> World {
    World::configured_limits(
        true,
        true,
        true,
        1000,
        Gate::default(),
        None,
        StoryLimits { merge_kind: kind, merge_authority: authority, ..StoryLimits::default() },
    )
    .with_landing_deployment(Box::new([engine::LandingRule {
        connector: 0,
        kind: 4,
        enforce: true,
        pattern: authority::Pattern {
            segments: Box::new([b"forge".as_slice().into()]),
            last: authority::Last::Open(Box::new([])),
        },
        ci: up_to_date,
        up_to_date,
        gates: Box::new([]),
        approvals: Box::new([]),
    }]))
}

// A fresh forge read may find a mergeable head whose merge then conflicts.
// Keep that race at the peer boundary: the write still reaches the independent fake.
fn optimistic_pull(world: &mut World) {
    for event in &mut world.events {
        if let engine::Event::Forge(event) = event
            && let forge_top::Event::Client(skein_forge_client::Event::Answered {
                result: Ok(skein_forge::api::Answer::Pull(pull)),
                ..
            }) = event.as_mut()
        {
            pull.mergeable = true;
        }
    }
}

fn next_job(world: &mut World, task: u64, optimistic: bool) -> host::JobAssignment {
    let deadline =
        world.env.now.saturating_add(world.env.limits.forge.client.settle).saturating_add(Duration::from_secs(20));
    while world.env.now <= deadline {
        if !world.jobs.is_empty() {
            return world.jobs.remove(0);
        }
        if optimistic {
            optimistic_pull(world);
        }
        world.tick();
    }
    let child = change_row(world, task).delegate.map(|(child, _)| child);
    let phase = child.and_then(|child| world.root.tasks().task(child)).map(|row| &row.phase);
    panic!(
        "merge job was not assigned: {:?}; child phase {:?}; errors {:?}",
        change_row(world, task),
        phase,
        world.call_errors
    );
}

fn check_job(world: &World, task: u64, job: &host::JobAssignment, head: u64, base: u64) {
    assert_eq!(job.kind, JobKind(forge_top::MERGE_KIND));
    assert_eq!(job.deadline, Duration::from_secs(600));
    let workspace = job.workspace.as_ref().expect("job workspace");
    assert_eq!(workspace.workstream, task, "the change owns the checkout cache");
    let items = &world.job_workspaces.get(&job.run.raw()).expect("typed job items").repositories;
    assert_eq!(items.len(), 1);
    let item = &items[0];
    let change = change_row(world, task);
    assert_eq!(item.provider, change.repository);
    assert_eq!(item.name.as_ref(), b"repo");
    assert_eq!(item.push.as_deref(), Some(change.branch.as_ref()));
    assert_eq!(item.start, engine::ForgeStart::Merge { branch: change.branch.clone(), base: translate::commit(base) });
    let row = live_task(world, job.run.raw());
    assert_eq!(row.requester, tasks::Party::Task(task));
    assert_eq!(row.executor, tasks::Executor::Job { kind: 3 });
    assert!(row.spec.words.is_empty() && row.spec.parameters.is_empty() && row.spec.inputs.is_empty());
    let mut reader = skein_lib::Reader::new(&job.spec);
    let command_length = reader.u32().expect("command length");
    assert_eq!(reader.bytes(command_length), Some(b"repo/.temper/merge".as_slice()));
    let request_length = reader.u32().expect("request length");
    let request = reader.bytes(request_length).expect("request");
    let expected = format!(
        "{{\"version\":1,\"head\":\"{}\",\"base\":\"{}\"}}",
        core::str::from_utf8(&forge_top::commit_text(&translate::commit(head))).expect("head text"),
        core::str::from_utf8(&forge_top::commit_text(&translate::commit(base))).expect("base text")
    );
    assert_eq!(request, expected.as_bytes());
    assert!(reader.is_empty());
    assert_eq!(change_row(world, task).change.resolutions, 1);
}

fn finish_job(world: &mut World, job: &host::JobAssignment, result: &[u8]) {
    let event = world.protocol.answer_job(
        CHANNEL,
        job.run.raw(),
        job.attempt.raw(),
        host::JobEnding::Finished { result: result.into(), work: host::Work { left: None, saved: None } },
    );
    world.send(event);
}

fn push_merge(world: &mut World, task: u64, head: u64, base: u64) -> u64 {
    let branch = change_row(world, task).branch.clone();
    let merged = fake::merge_commit(
        &mut world.fake,
        &world.fake_env.limits,
        head,
        base,
        Box::new([raw::File { path: b"file".as_slice().into(), content: b"settled".as_slice().into() }]),
        b"Resolve merge",
    )
    .expect("resolved merge with both parents");
    assert_eq!(
        world.external(2, raw::Op::Git(raw::Git::Push { branch, commit: merged, expected: Some(head) })),
        raw::Answer::Pushed(raw::Pushed::Pushed)
    );
    merged
}

fn passed_ci(world: &mut World, head: u64) {
    assert_eq!(
        world.external(
            1,
            raw::Op::Write(raw::Write::Status {
                commit: head,
                context: b"build".as_slice().into(),
                state: raw::Check::Passed,
                description: Box::new([]),
                link: Box::new([]),
            })
        ),
        raw::Answer::Done
    );
}

fn landed(world: &mut World, task: u64, head: u64) {
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
    assert!(world.writes.iter().any(|write| matches!(write, skein_forge::api::Write::Merge { head: merged, .. } if *merged == translate::commit(head))));
}

/// temper/docs/design/domain/forge/change/repairs.md: a goal branch's conflict can be settled by its merge job alone.
#[test]
fn a_conflict_into_a_goal_branch_is_settled_by_the_merge_job_and_merges() {
    let (mut world, task, head, base) = pushed_change_in(world(false, true, true).with_merge_job(COMMAND), true, GOAL);
    let job = next_job(&mut world, task, true);
    check_job(&world, task, &job, head, base);
    assert!(
        world.writes.iter().any(|write| matches!(write, skein_forge::api::Write::Merge { .. })),
        "the forge refused the optimistic merge for its conflict"
    );
    assert_eq!(world.merges, 1, "the conflicting merge was attempted once");
    assert!(live_task(&world, task).authority.delegation.kinds.contains(&tasks::AuthorityExecutor::Job(3)));
    let merged = push_merge(&mut world, task, head, base);
    finish_job(&mut world, &job, RESULT);
    landed(&mut world, task, merged);
    assert_eq!(world.assigned.len(), 2, "only the chat and producer ran as agents");
    assert_eq!(change_row(&world, task).change.resolutions, 1);
    world.close();
}

/// temper/docs/design/domain/forge/change/repairs.md: a job leaving conflicts supplies the following resolution's account.
#[test]
fn a_conflict_the_merge_job_leaves_goes_to_a_resolution() {
    const LEFT: &[u8] = b"{\"left\":[\"file\"]}";
    let (mut world, task, head, base) = pushed_change_in(world(false, true, true).with_merge_job(COMMAND), true, GOAL);
    let job = next_job(&mut world, task, false);
    check_job(&world, task, &job, head, base);
    finish_job(&mut world, &job, LEFT);
    let resolution = super::reviews::next_assignment(&mut world, 2);
    let words = &live_task(&world, resolution.task).spec.words;
    assert_preserving_resolution(words);
    assert!(
        words.ends_with(
            format!(
                "\n\nThe merge job settled what the project's merge command knows and left the rest:\n{}",
                core::str::from_utf8(LEFT).expect("result text")
            )
            .as_bytes()
        )
    );
    assert_eq!(change_row(&world, task).change.resolutions, 1);
    assert!(change_row(&world, task).merge_result.is_none(), "making the resolution consumes the account");
    let merged = push_merge(&mut world, task, head, base);
    finish_resolution(&mut world, &resolution);
    landed(&mut world, task, merged);
    world.close();
}

fn assert_preserving_resolution(words: &[u8]) {
    let text = core::str::from_utf8(words).expect("resolution text");
    assert!(text.starts_with("Resolve the merge conflict."));
    for expected in [
        "Preserve the requested change",
        "unrelated content already landed on the base",
        "both independent appended lines and existing blank lines",
        "Do not remove content to make a check pass",
        "The worker commits and pushes the resolved merge",
    ] {
        assert!(text.contains(expected), "{text}");
    }
}

fn finish_resolution(world: &mut World, resolution: &engine::Assignment) {
    let event = world.protocol.answer(
        CHANNEL,
        resolution.task,
        resolution.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
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
}

/// temper/docs/design/domain/forge/change/repairs.md: an update's conflict tries the merge job before asking an agent.
#[test]
fn an_update_conflict_goes_to_the_merge_job_before_a_resolution() {
    let (mut world, task, head, base) =
        pushed_change_in(world(true, true, true).with_merge_job(COMMAND), true, b"main");
    passed_ci(&mut world, head);
    passed_ci(&mut world, base);
    let job = next_job(&mut world, task, false);
    check_job(&world, task, &job, head, base);
    assert!(world.writes.iter().any(|write| matches!(write, skein_forge::api::Write::Update { .. })));
    let merged = push_merge(&mut world, task, head, base);
    passed_ci(&mut world, merged);
    finish_job(&mut world, &job, RESULT);
    landed(&mut world, task, merged);
    assert_eq!(change_row(&world, task).result.head, Some(translate::commit(merged)), "the new head was checked again");
    assert_eq!(world.assigned.len(), 2);
    world.close();
}

/// temper/docs/design/domain/forge/change/repairs.md: a conflict without a project command goes straight to a resolution.
#[test]
fn without_a_merge_job_a_conflict_goes_to_a_resolution() {
    let (mut world, task, head, base) = pushed_change_in(world(false, true, true), true, GOAL);
    let resolution = next_resolution(&mut world, task);
    assert!(world.jobs.is_empty());
    assert_preserving_resolution(&live_task(&world, resolution.task).spec.words);
    let merged = push_merge(&mut world, task, head, base);
    finish_resolution(&mut world, &resolution);
    landed(&mut world, task, merged);
    world.close();
}

fn next_resolution(world: &mut World, task: u64) -> engine::Assignment {
    let deadline =
        world.env.now.saturating_add(world.env.limits.forge.client.settle).saturating_add(Duration::from_secs(20));
    while world.env.now <= deadline {
        if let Some(resolution) = world.assigned.get(2) {
            return resolution.clone();
        }
        world.tick();
    }
    panic!("resolution was not assigned: {:?}", change_row(world, task));
}

/// temper/docs/design/domain/forge/change/repairs.md: command, declared kind, spec room and authority all gate merge jobs.
#[test]
fn a_merge_job_needs_its_declared_kind_authority_and_spec_room() {
    for (kind, authority, command) in [(false, true, COMMAND), (true, false, COMMAND), (true, true, &[b'x'; 4096][..])]
    {
        let (mut world, task, _, _) =
            pushed_change_in(world(false, kind, authority).with_merge_job(command), true, GOAL);
        let _ = next_resolution(&mut world, task);
        assert!(world.jobs.is_empty());
        assert_eq!(
            live_task(&world, task).authority.delegation.kinds.contains(&tasks::AuthorityExecutor::Job(3)),
            authority
        );
        world.close();
    }
}

/// jig/docs/design/domain/engine/executors.md, 5: transport failures exhaust retries and hold the job.
#[test]
fn a_failed_merge_job_is_held_after_its_retries() {
    let (mut world, task, head, base) = pushed_change_in(world(false, true, true).with_merge_job(COMMAND), true, GOAL);
    // A permanent failure is terminal only after the configured retry allowance.
    let mut last_job = None;
    for _ in 0..=world.env.limits.tasks.retries.permanent.retries {
        let job = next_job(&mut world, task, false);
        let event = world.protocol.answer_job(
            CHANNEL,
            job.run.raw(),
            job.attempt.raw(),
            host::JobEnding::Failed {
                failure: host::JobFailure::Undelivered(host::DeliveryOutcome::Refused),
                detail: b"merge delivery refused".as_slice().into(),
                work: host::Work { left: None, saved: None },
            },
        );
        world.send(event);
        last_job = Some(job);
    }
    let job = last_job.expect("at least the first attempt");
    for _ in 0..STEPS {
        if matches!(
            live_task(&world, job.run.raw()).phase,
            tasks::Phase::Held { why: tasks::Hold::Failures(tasks::Class::Permanent), .. }
        ) {
            break;
        }
        world.tick();
    }
    assert!(matches!(
        live_task(&world, job.run.raw()).phase,
        tasks::Phase::Held { why: tasks::Hold::Failures(tasks::Class::Permanent), .. }
    ));
    let tasks::Phase::Held { source: Some(source), .. } = &live_task(&world, job.run.raw()).phase else {
        panic!("the job's hold carries its cause");
    };
    assert_eq!(**source, tasks::HoldSource::Preparation { words: b"merge delivery refused".as_slice().into() });
    assert_eq!(change_row(&world, task).change.resolutions, 1, "no additional conflict delegate is made");
    assert_eq!(world.merges, 0);
    assert_eq!(world.assigned.len(), 2);
    let revision = live_task(&world, job.run.raw()).revisions.hold;
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(94)),
        sign_in: world.signed_in.expect("owner signed in"),
        key: [94; 16],
        ask: people::Ask::Release { project: 1, task: job.run.raw(), revision, again: None },
    }));
    let retry = next_job(&mut world, task, false);
    let merged = push_merge(&mut world, task, head, base);
    finish_job(&mut world, &retry, RESULT);
    landed(&mut world, task, merged);
    world.close();
}

/// temper/docs/design/domain/worker/jobs.md: a retry prepares its workspace afresh after a transient failure.
#[test]
fn a_failed_merge_jobs_retry_keeps_the_changes_workspace() {
    let (mut world, task, head, base) = pushed_change_in(world(false, true, true).with_merge_job(COMMAND), true, GOAL);
    let first = next_job(&mut world, task, false);
    check_job(&world, task, &first, head, base);
    let event = world.protocol.answer_job(
        CHANNEL,
        first.run.raw(),
        first.attempt.raw(),
        host::JobEnding::Failed {
            failure: host::JobFailure::Unprepared(host::Preparation::Transient),
            detail: b"checkout temporarily unavailable".as_slice().into(),
            work: host::Work { left: None, saved: None },
        },
    );
    world.send(event);
    let retry = next_job(&mut world, task, false);
    assert_eq!(retry.run, first.run);
    assert_ne!(retry.attempt, first.attempt);
    check_job(&world, task, &retry, head, base);
    let merged = push_merge(&mut world, task, head, base);
    finish_job(&mut world, &retry, RESULT);
    landed(&mut world, task, merged);
    world.close();
}
