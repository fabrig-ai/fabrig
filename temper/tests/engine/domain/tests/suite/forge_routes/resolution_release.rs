//! A party's release retires a failed resolution and reads the goal anew
//! (temper/docs/design/domain/forge/change/repairs.md, A release).
use super::goal_branch::{branch_tip, running_goal_in};
use super::goal_landing::{ended, finish_run, land_into_goal, mine, row, status};
use super::*;
use temper_engine_domain_forge_change::{Delegate, Hold, State};

const STEPS: u32 = 1000;
const CHANNEL: Token = Token::new(7);

fn goal_world(merge_job: bool) -> (World, engine::Assignment, Box<[u8]>) {
    let mut world = World::configured_limits(
        true, true, true, 1000, Gate { agent: true, ..Gate::default() }, None,
        StoryLimits { merge_kind: merge_job, merge_authority: merge_job, ..StoryLimits::default() },
    );
    if merge_job { world = world.with_merge_job(b".temper/merge"); }
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, 2, Box::new([])));
    running_goal_in(world)
}

fn wait_held(world: &mut World, task: u64) {
    for _ in 0..STEPS {
        world.tick();
        if matches!(live_task(world, task).phase, tasks::Phase::Held { .. }) && world.store.pending.is_empty() {
            break;
        }
    }
    assert!(matches!(row(world, task).change.state, State::Held { why: Hold::Failed, .. }), "{:?}", row(world, task));
    assert!(matches!(live_task(world, task).phase, tasks::Phase::Held { .. }));
}

fn release(world: &mut World, task: u64, revision: u64) {
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(960)),
        sign_in: world.signed_in.expect("owner signed in"),
        key: [96; 16],
        ask: people::Ask::Release { project: 1, task, revision, again: None },
    }));
}

fn fail_delegate(world: &mut World, task: u64, merge_job: bool, head: u64, base: u64) -> (u64, u64) {
    let mut reviewed = None;
    for _ in 0..STEPS {
        world.tick();
        if let Some((child, Delegate::Gate { .. })) = row(world, task).delegate {
            if reviewed != Some(child) && let Some(gate) = world.assigned.iter().find(|run| run.task == child).cloned() {
                reviewed = Some(child);
                reviews::finish_review(world, &gate, tasks::TaskResult::Verdict { code: 1, fields: Box::new([]), words: b"approved".as_slice().into() });
            }
            continue;
        }
        if row(world, task).delegate.is_some_and(|(child, _)| {
            world.assigned.iter().any(|run| run.task == child) || world.jobs.iter().any(|job| job.run.raw() == child)
        }) {
            break;
        }
    }
    let (child, kind) = row(world, task).delegate.expect("conflict delegate");
    assert!(if merge_job { matches!(kind, Delegate::Merge { .. }) } else { matches!(kind, Delegate::Resolve { .. }) });
    let saved = fake::merge_commit(
        &mut world.fake, &world.fake_env.limits, head, base,
        Box::new([raw::File { path: b"plan".as_slice().into(), content: b"saved work needs a fix".as_slice().into() }]),
        b"Saved resolution",
    ).expect("saved merge contains its inputs");
    let branch = row(world, task).branch.clone();
    assert_eq!(world.external(1, raw::Op::Git(raw::Git::Push { branch, commit: saved, expected: Some(head) })), raw::Answer::Pushed(raw::Pushed::Pushed));
    status(world, saved, raw::Check::Passed);
    let reason = format!("delivery refused at saved head {}", String::from_utf8(forge_top::commit_text(&translate::commit(saved)).to_vec()).expect("head text"));
    // This world runs no delivery checks: the worker scripts a failed terminal,
    // not a successful resolution whose pushed head merely has red CI.
    if merge_job {
        let job = world.jobs.remove(0);
        let event = world.protocol.answer(CHANNEL, job.run.raw(), job.attempt.raw(), 5,
            tasks::End::Finished { result: tasks::TaskResult::Failure { reason: reason.as_bytes().into() }, cancel_delegates: false }, engine::Parts::none());
        world.send(event);
    } else {
        let resolver = world.assigned.iter().find(|run| run.task == child).expect("resolver assigned").clone();
        finish_run(world, &resolver, tasks::TaskResult::Failure { reason: reason.as_bytes().into() });
    }
    wait_held(world, task);
    let tasks::Phase::Held { source: Some(source), .. } = &live_task(world, task).phase else { panic!("failed delegate account") };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("failure words") };
    assert!(contains(words, reason.as_bytes()), "{}", String::from_utf8_lossy(words));
    (child, saved)
}

#[test]
fn a_released_goal_landing_uses_fresh_facts_once_including_after_restart() {
    for (merge_job, restart, conflict) in [(false, false, false), (false, true, false), (false, true, true), (true, true, false)] {
        let (mut world, goal, branch) = goal_world(merge_job);
        let head = land_into_goal(&mut world, &goal, &branch, 93, b"plan");
        let base = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"plan", b"main differs", 1).expect("main conflicts");
        status(&mut world, base, raw::Check::Passed);
        let answer = mine(&mut world, &goal, 94, br#"{"words":"Land the goal whole","branch":"mine"}"#);
        let Some(engine::CallAnswer::Delegated(made)) = answer else { panic!("goal landing admitted: {answer:?}") };
        let task = made[0].task;
        let (old, saved) = fail_delegate(&mut world, task, merge_job, head, base);
        let fresh = land_into_goal(&mut world, &goal, &branch, 95, b"fix");
        assert_ne!(fresh, saved);
        assert_eq!(world.external(1, raw::Op::Read(raw::Read::Commits { before: fresh, after: saved })), raw::Answer::Commits { commits: Box::new([]) }, "H2 contains H1");
        status(&mut world, fresh, raw::Check::Passed);
        let base = if conflict {
            let tip = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"fix", b"main conflicts again", 1).expect("base advanced");
            status(&mut world, tip, raw::Check::Passed);
            tip
        } else { base };
        let revision = live_task(&world, task).revisions.hold;
        let made = live_task(&world, task).made;
        let resolutions = row(&world, task).resolution_tasks.len();
        world.forge_reads = Some(Vec::new());
        release(&mut world, task, revision);
        for _ in 0..STEPS {
            world.tick();
            if row(&world, task).task_release.as_ref().is_some_and(|r| r.revision == revision) && world.store.pending.is_empty() {
                break;
            }
        }
        assert_eq!(row(&world, task).task_release.as_ref().map(|r| r.revision), Some(revision));
        assert_ne!(row(&world, task).change.last_head, Some(translate::commit(fresh)), "cut precedes the fresh decision");
        if restart { world.restart_same(); }
        for _ in 0..STEPS {
            world.tick();
            assert!(!matches!(row(&world, task).change.state, State::Held { .. }), "no old refusal: {:?}", row(&world, task));
            if row(&world, task).change.last_head == Some(translate::commit(fresh)) && world.store.pending.is_empty() {
                break;
            }
        }
        assert_eq!(row(&world, task).change.last_head, Some(translate::commit(fresh)));
        assert!(!matches!(live_task(&world, task).phase, tasks::Phase::Held { .. }));
        // Deliver the same release to the connector again, as after replay.
        let before = row(&world, task).clone();
        world.send(engine::Event::Forge(Box::new(forge_top::Event::ReleaseChange {
            task, release: before.task_release.clone(),
        })));
        assert_eq!(row(&world, task), &before);
        if conflict {
            for _ in 0..STEPS {
                if row(&world, task).delegate.is_some() { break; }
                world.tick();
            }
            let (child, delegate) = row(&world, task).delegate.expect("fresh resolution");
            assert_ne!(child, old);
            assert_eq!(delegate, Delegate::Resolve { base: translate::commit(base) });
            for _ in 0..STEPS {
                world.tick();
                if world.assigned.iter().any(|run| run.task == child) && world.store.pending.is_empty() { break; }
            }
            let resolver = world.assigned.iter().find(|run| run.task == child).expect("fresh resolution assigned");
            assert!(matches!(resolver.workspace.repositories[0].start, engine::ForgeStart::Merge { base: pinned, .. } if pinned == translate::commit(base)));
            let hex = forge_top::commit_text(&translate::commit(fresh));
            let pin = format!("Resolve the conflict from change head {}", String::from_utf8(hex.to_vec()).expect("head text"));
            assert!(resolver.sections.iter().any(|section| matches!(&section.body, engine::BriefBody::Text(words) if contains(words, pin.as_bytes()))));
            world.restart_same();
            for _ in 0..100 { world.tick(); }
            assert_eq!(row(&world, task).delegate, Some((child, delegate)));
            assert_eq!(row(&world, task).resolution_tasks.len(), resolutions + 1);
        } else {
            for _ in 0..100 {
                world.tick();
                assert!(!matches!(row(&world, task).change.state, State::Held { .. }));
            }
            assert_eq!(row(&world, task).change.last_head, Some(translate::commit(fresh)));
            assert_eq!(row(&world, task).resolution_tasks.len(), resolutions);
            assert_eq!(live_task(&world, task).made, made + 1, "only the fresh gate is asked");
            assert!(matches!(row(&world, task).delegate, Some((_, Delegate::Gate { head, .. })) if head == translate::commit(fresh)));
            let pull = row(&world, task).pull.expect("landing pull");
            assert!(world.forge_reads.as_ref().expect("recorded reads").iter().any(|(_, read)| {
                matches!(read, skein_forge::api::Read::Checks { commit } if *commit == translate::commit(fresh))
                    || matches!(read, skein_forge::api::Read::Pull { number, ci: true } if *number == pull)
            }), "H2 is checked before its fresh gate");
        }
        assert!(world.jobs.is_empty(), "no new merge job");
        assert!(ended(&world, old).is_some(), "failed delegate is durable");
        assert_eq!(branch_tip(&mut world, &branch), fresh);
        world.close();
    }
}
