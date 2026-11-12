//! A moved pin ends an idle resolution (temper/docs/design/domain/forge/change/repairs.md).
use super::light_rules::{change_row, pushed_change_in};
use super::*;
use temper_engine_domain_forge_change::{Delegate, Hold, State};

const STEPS: u32 = 300;

fn ended(world: &World, task: u64) -> Option<&tasks::TaskRecord> {
    match world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)))) {
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) => Some(row),
        _ => None,
    }
}

fn idle_resolution(world: &mut World, change: u64) -> u64 {
    let deadline =
        world.env.now.saturating_add(world.env.limits.forge.client.settle).saturating_add(Duration::from_secs(20));
    while world.env.now <= deadline {
        if let Some((child, Delegate::Resolve { .. })) = change_row(world, change).delegate {
            assert!(!world.assigned.iter().any(|run| run.task == child), "the resolution has no run yet");
            return child;
        }
        world.tick();
    }
    panic!("resolution not made: {:?}", change_row(world, change));
}

fn observe(world: &mut World, facts: &mut Vec<engine::Fact>) {
    while let Some(fact) = world.root.pop_fact() {
        facts.push(fact);
    }
}

fn moved_head(conflict: bool) {
    // This producer-created branch is not a requester's held branch: a push
    // without a delegate in its writer slot is drift, even by temper's identity.
    // Leave a fresh read/store conversation between tries in this world's
    // 100 ms ticks, rather than the fixture's sub-tick 10 ms retry cadence.
    let retry = tasks::Retry { retries: 1, base: Duration::from_secs(5), max: Duration::from_secs(5) };
    let configured = super::merge_job::world(false, true, true).with_transient_retry(retry);
    let (mut world, change, head, base) = pushed_change_in(configured, true, b"temper/goal");
    let resolution = idle_resolution(&mut world, change);
    let branch = change_row(&world, change).branch.clone();
    let moved = if conflict {
        fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"other", b"another push", 2)
            .expect("branch advanced while resolution prepares")
    } else {
        super::merge_job::push_merge(&mut world, change, head, base)
    };
    let moved_commit = translate::commit(moved);
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::BranchMoved { branch, head: moved_commit, actor: 2 },
            key: None,
        },
    })));
    let mut facts = Vec::new();
    for _ in 0..STEPS {
        world.tick();
        observe(&mut world, &mut facts);
        if ended(&world, resolution).is_some()
            && matches!(change_row(&world, change).change.state, State::Held { why: Hold::BranchMoved, .. })
            && matches!(live_task(&world, change).phase, tasks::Phase::Held { .. })
        {
            break;
        }
    }
    let old = ended(&world, resolution).expect("the moved head cancels the idle resolution");
    let tasks::Phase::Ended(tasks::Ending::Cancelled { reason, .. }) = &old.phase else {
        panic!("cancelled resolution: {:?}", old.phase);
    };
    let hex = forge_top::commit_text(&moved_commit);
    let hex = core::str::from_utf8(&hex).expect("head text");
    assert_eq!(reason.as_ref(), format!("the change's head moved to {hex}").as_bytes());
    let failures: Vec<_> = facts
        .iter()
        .filter_map(|fact| match fact {
            engine::Fact::BriefFailed { task, counts, reason } if *task == resolution => Some((*counts, reason)),
            _ => None,
        })
        .collect();
    assert!(failures.len() <= 1, "the old pin is tried at most once: {failures:?}");
    for (counts, reason) in failures {
        assert_eq!(counts, "transient");
        assert_eq!(reason.as_ref(), b"pull: the change's head moved");
    }
    assert_eq!(
        change_row(&world, change).change.state,
        State::Held { was: Box::new(State::WaitingProducer { base: translate::commit(base) }), why: Hold::BranchMoved }
    );
    assert_eq!(change_row(&world, change).change.last_head, Some(translate::commit(head)));
    let shown = temper_engine_domain_world::tier_task_show::show(&mut world, change);
    assert_eq!(shown.status, 0);
    assert!(contains(&shown.stdout, b"its branch moved:"));
    assert!(!contains(&shown.stdout, format!("its resolution, task {resolution}, was cancelled").as_bytes()));
    let revision = live_task(&world, change).revisions.hold;
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(980)),
        sign_in: world.signed_in.expect("owner"),
        key: [98; 16],
        ask: people::Ask::Release { project: 1, task: change, revision, again: None },
    }));
    if conflict {
        let fresh = fresh_resolution(&mut world, change, resolution, moved_commit);
        let resolved = super::merge_job::push_merge(&mut world, change, moved, base);
        super::merge_job::finish_resolution(&mut world, &fresh);
        super::merge_job::landed(&mut world, change, resolved);
    } else {
        super::merge_job::landed(&mut world, change, moved);
        assert_eq!(world.assigned.len(), 2, "no fresh resolution when the conflict is gone");
        assert_eq!(change_row(&world, change).change.last_head, Some(moved_commit));
    }
    observe(&mut world, &mut facts);
    assert!(
        !facts.iter().any(|fact| matches!(fact, engine::Fact::BriefFailed { task, .. } if *task != resolution)),
        "the fresh brief gathered once: {facts:?}"
    );
    world.close();
}

fn fresh_resolution(world: &mut World, change: u64, prior: u64, head: [u8; 32]) -> engine::Assignment {
    let deadline =
        world.env.now.saturating_add(world.env.limits.forge.client.settle).saturating_add(Duration::from_secs(20));
    while world.env.now <= deadline && world.assigned.len() < 3 {
        world.tick();
    }
    let fresh = world
        .assigned
        .get(2)
        .unwrap_or_else(|| {
            panic!("fresh resolution: {:?}; phase: {:?}", change_row(world, change), live_task(world, change).phase)
        })
        .clone();
    assert_ne!(fresh.task, prior);
    assert_eq!(live_task(world, fresh.task).tries, tasks::Tries::NONE, "the fresh pin gathers on its first try");
    let pull = fresh
        .sections
        .iter()
        .find_map(|section| match &section.body {
            engine::BriefBody::Text(words)
                if section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Pull) =>
            {
                Some(words)
            }
            _ => None,
        })
        .expect("resolution pull section");
    let hex = forge_top::commit_text(&head);
    let hex = core::str::from_utf8(&hex).expect("head text");
    assert!(
        contains(pull, format!("\nResolve the conflict from change head {hex}\n").as_bytes()),
        "{}",
        String::from_utf8_lossy(pull)
    );
    assert_eq!(change_row(world, change).change.resolutions, 1, "release resets the resolution allowance");
    assert_eq!(change_row(world, change).change.last_head, Some(head));
    fresh
}

#[test]
fn an_idle_resolution_ends_at_a_moved_head_and_resolves_afresh_after_release() {
    moved_head(true);
}

#[test]
fn an_idle_resolution_ends_at_a_head_without_conflict_and_goes_on_after_release() {
    moved_head(false);
}
