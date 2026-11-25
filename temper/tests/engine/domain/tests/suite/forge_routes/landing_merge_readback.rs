//! A merge raced by a goal's design delivery settles its own round, not an effect hold
//! (temper/docs/design/domain/landing.md, 4.2, and temper/docs/design/domain/landing-turn.md, 1–2, and temper/docs/design/domain/landing.md, 16).
use super::super::{goal_branch::branch_tip, goal_delivery, goal_landing};
use super::*;

const STEPS: u32 = 1600; // Two rounds plus the client's bounded uncertain-write recovery.

fn landing(world: &World, task: u64) -> &forge_top::LandingRow {
    landing_rows(world).into_iter().find(|row| row.landing.task == task).expect("landing row")
}

#[test]
fn a_design_delivery_racing_a_turns_merge_ends_the_round_and_frees_the_goal_train() {
    let (mut world, goal, branch) = goal_delivery::running_goal_with(true);
    world.forgejo_merge_divergence = true;
    world.held_merge_calls = Some(Vec::new());
    let (first, producer) = goal_landing::producing_into_goal(&mut world, &goal, &branch, 101, b"first");
    finish_producer(&mut world, &producer, forge_world::REPO);
    for _ in 0..STEPS {
        world.tick();
        if world.held_merge_calls.as_ref().is_some_and(|calls| !calls.is_empty()) {
            break;
        }
    }
    assert_eq!(world.held_merge_calls.as_ref().expect("paused merges").len(), 1);
    let row = landing_rows(&world).into_iter().next().expect("first landing");
    let task = row.landing.task;
    let pull = row.landing.round.pull.expect("round pull");
    assert!(matches!(row.landing.phase, forge_top::LandingPhase::Turn { .. }));
    let base = branch_tip(&mut world, &branch);
    let moved = goal_delivery::deliver(&mut world, &goal, base, b"design.md");
    world.release_merge_calls();

    let mut close_entry = None;
    let mut close_before_hold = false;
    let mut held = None;
    let mut held_at = None;
    let mut close_sent_at = None;
    for tick in 0..STEPS {
        world.tick();
        let row = landing(&world, task);
        if let Some(entry) = row.entries.iter().find(|entry| entry.operation == forge_top::Operation::Close) {
            close_entry = Some(entry.entry);
            if let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Entry(entry.entry)))
                && let forge_top::Stored::Entry { entry, .. } = row.as_ref()
            {
                assert_eq!(entry.task, task, "the landing owns its cleanup's close");
                assert_eq!(entry.effect.write, api::Write::Close { number: pull });
            }
            if !matches!(task_row(&world, task).expect("landing task").phase, tasks::Phase::Held { .. }) {
                close_before_hold = true;
            }
        }
        let phase = &task_row(&world, task).expect("landing task").phase;
        if matches!(phase, tasks::Phase::Held { .. }) {
            held = Some(phase.clone());
            held_at.get_or_insert(tick);
        }
        if world.writes.contains(&api::Write::Close { number: pull }) {
            close_sent_at.get_or_insert(tick);
        }
        if ended(&world, first) {
            break;
        }
    }
    let raw::Answer::Pull(old) = world.external(1, raw::Op::Read(raw::Read::Pull { number: pull })) else {
        panic!("old round pull")
    };
    let (second, producer) = goal_landing::producing_into_goal(&mut world, &goal, &branch, 102, b"second");
    finish_producer(&mut world, &producer, forge_world::REPO);
    for _ in 0..STEPS {
        world.tick();
        if ended(&world, second) {
            break;
        }
    }
    if let Some(tasks::Phase::Held { source: Some(source), .. }) = &held
        && let tasks::HoldSource::Account { words, .. } = source.as_ref()
    {
        eprintln!(
            "round #{pull}: landing {task} queued Close entry {close_entry:?} before hold={close_before_hold}; held tick {held_at:?}, Close sent tick {close_sent_at:?}; round ending {:?}; old PR {:?}, merged {:?}; second landing {:?}; {}",
            landing(&world, task).landing.round.ending,
            old.state,
            old.merged,
            world.root.forge().landing_of(second),
            String::from_utf8_lossy(words)
        );
    }
    assert!(held.is_none(), "a merge outcome is the landing's, never an effect hold: {held:?}");
    assert_eq!(old.merged, None);
    assert_eq!(old.state, raw::State::Closed);
    assert!(close_entry.is_some());
    assert!(close_before_hold);
    let row = landing(&world, task);
    assert!(row.landing.preparation > 1, "a fresh round on the moved goal tip");
    assert_eq!(row.landing.members[0].base, Some(translate::commit(moved)));
    assert!(ended(&world, first), "next round lands the first change");
    assert!(world.writes.iter().filter(|write| matches!(write, api::Write::OpenPull { .. })).count() >= 3);
    assert!(ended(&world, second), "the next landing can form on the freed goal train");
}

#[test]
#[expect(clippy::too_many_lines, reason = "the bounded hold, independent train and release share one root story")]
fn an_exhausted_turn_holds_only_its_goal_train_with_its_fresh_reads_then_releases() {
    use temper_engine_domain_forge_landing::{Hold, Phase};
    let (mut world, goal, branch) = goal_delivery::running_goal_with(true);
    let attempts = world.env.limits.forge.client.write_attempts;
    assert_eq!(attempts, 2);
    world.lose_merges = u32::MAX;
    let (first, producer) = goal_landing::producing_into_goal(&mut world, &goal, &branch, 101, b"first");
    finish_producer(&mut world, &producer, forge_world::REPO);
    for _ in 0..STEPS {
        world.tick();
        if landing_rows(&world).iter().any(|row| matches!(row.landing.phase, Phase::Held { .. })) {
            break;
        }
    }
    let row = landing_rows(&world).into_iter().next().expect("held landing");
    let task = row.landing.task;
    let pull = row.landing.round.pull.expect("round pull");
    assert_eq!(row.landing.round.merges, 2, "two merge effects, each with its own client retry bound");
    assert!(matches!(row.landing.phase, Phase::Held { why: Hold::Turn, .. }), "{row:?}");
    let tip = branch_tip(&mut world, &branch);
    let expected = format!(
        "turn: merge of #{pull}: forge timed out; read #{pull} open, {} at {}",
        String::from_utf8_lossy(&branch),
        String::from_utf8_lossy(&forge_top::commit_text(&translate::commit(tip)))
    );
    for _ in 0..STEPS {
        world.tick();
        if matches!(live_task(&world, task).phase, tasks::Phase::Held { .. }) {
            break;
        }
    }
    let tasks::Phase::Held { why: tasks::Hold::Procedure, source: Some(source), .. } = &live_task(&world, task).phase
    else {
        panic!("policy holds, not EffectFailed: {:?}", live_task(&world, task).phase)
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("hold account") };
    assert_eq!(words.as_ref(), expected.as_bytes());
    assert_eq!(world.merges, attempts * attempts, "two client attempts per merge effect");
    world.lose_merges = 0;
    let raw::Answer::Pull(read) = world.external(1, raw::Op::Read(raw::Read::Pull { number: pull })) else {
        panic!("pull")
    };
    assert_eq!((read.state, read.merged), (raw::State::Open, None));

    let (waiting, producer) = goal_landing::producing_into_goal(&mut world, &goal, &branch, 102, b"waiting");
    finish_producer(&mut world, &producer, forge_world::REPO);
    let people::Ask::Create { project, mut batch } = super::super::goal_branch::goal_request(&world, true) else {
        panic!("goal")
    };
    batch[0].budget = people::Budget::AsAllowed { cap: Some(100) };
    super::super::tiers::sign_in(&mut world);
    let other = assigned_chat(&mut world, 93, people::Ask::Create { project, batch });
    let other_branch = format!("temper/{}/g{}", other.task, other.task).into_bytes();
    world.until(Until::Ready);
    let (other_change, producer) = goal_landing::producing_into_goal(&mut world, &other, &other_branch, 103, b"other");
    finish_producer(&mut world, &producer, forge_world::REPO);
    for _ in 0..STEPS {
        world.tick();
        if ended(&world, other_change) {
            break;
        }
    }
    assert!(
        ended(&world, other_change),
        "other train lands: {:?}; jobs {:?}; tasks {:?}",
        landing_rows(&world),
        world.jobs,
        world
            .store
            .rows
            .values()
            .filter_map(|r| if let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) = r {
                Some((row.number, &row.phase))
            } else {
                None
            })
            .collect::<Vec<_>>()
    );
    assert!(matches!(landing(&world, task).landing.phase, Phase::Held { why: Hold::Turn, .. }));
    assert_eq!(world.root.forge().landing_of(waiting), None, "the held train alone cannot form");
    assert!(
        landing_rows(&world)
            .iter()
            .any(|row| row.landing.branch.as_ref() == other_branch && row.landing.phase == Phase::Landed)
    );
    assert_eq!(branch_tip(&mut world, &branch), tip);

    super::super::tiers::sign_in(&mut world);
    let revision = live_task(&world, task).revisions.hold;
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(960)),
        sign_in: world.signed_in.expect("owner"),
        key: [96; 16],
        ask: people::Ask::Release { project: 1, task, revision, again: None },
    }));
    for _ in 0..STEPS {
        world.tick();
        assert!(!matches!(
            task_row(&world, task).expect("landing").phase,
            tasks::Phase::Held { why: tasks::Hold::EffectFailed, .. }
        ));
        if ended(&world, first) && ended(&world, waiting) {
            break;
        }
    }
    assert!(ended(&world, first) && ended(&world, waiting), "release frees both changes");
    assert_eq!(landing(&world, task).landing.preparation, 2);
    assert_eq!(landing(&world, task).landing.phase, Phase::Landed);
    assert_eq!(landing(&world, task).landing.round.merges, 1, "release reset the round's count");
}

fn unsent_merge(world: &World) -> Option<(u64, u64, u64)> {
    world.store.rows.values().find_map(|record| match record {
        Record::Forge { row } => match row.as_ref() {
            forge_top::Stored::Entry { entry, held: false } if entry.attempt.is_none() => match entry.effect.write {
                api::Write::Merge { number, how: api::MergeHow::FastForwardOnly, .. } => {
                    Some((entry.task, entry.number, number))
                }
                _ => None,
            },
            _ => None,
        },
        Record::Core(_) => None,
    })
}

#[test]
fn a_withdrawn_unsent_turn_merge_is_read_open_at_the_base_and_asked_again() {
    let (mut world, goal, branch) = goal_delivery::running_goal_with(true);
    let (change, producer) = goal_landing::producing_into_goal(&mut world, &goal, &branch, 101, b"first");
    finish_producer(&mut world, &producer, forge_world::REPO);
    let mut withdrawn = None;
    for _ in 0..STEPS {
        world.tick();
        if let Some((task, entry, number)) = unsent_merge(&world) {
            assert_eq!(world.merges, 0, "withdraw before the client sends the merge");
            world.send(engine::Event::Forge(Box::new(forge_top::Event::Withdraw { entry })));
            withdrawn = Some((task, entry, number));
            break;
        }
    }
    let (task, entry, pull) = withdrawn.expect("unsent merge");
    let base = branch_tip(&mut world, &branch);
    let raw::Answer::Pull(read) = world.external(1, raw::Op::Read(raw::Read::Pull { number: pull })) else {
        panic!("pull")
    };
    assert_eq!((read.state, read.merged), (raw::State::Open, None));
    assert_eq!(read.base_commit, Some(base));
    let mut saw_words = false;
    for _ in 0..STEPS {
        world.tick();
        let row = landing(&world, task);
        saw_words |= row
            .landing
            .round
            .failure
            .windows(b"withdrawn before completion".len())
            .any(|w| w == b"withdrawn before completion");
        let phase = &task_row(&world, task).expect("landing task").phase;
        assert!(!matches!(phase, tasks::Phase::Held { .. }), "merge entry {entry}, pull #{pull}: {phase:?}");
        if ended(&world, change) {
            break;
        }
    }
    assert!(saw_words, "the terminal outcome's words survive until the next merge");
    assert!(ended(&world, change));
    let row = landing(&world, task);
    assert_eq!(row.landing.round.pull, Some(pull), "the still-open pull is reused");
    assert_eq!(row.landing.preparation, 1);
    assert_eq!(row.landing.round.merges, 2);
    assert_eq!(row.landing.stale_proofs, 0);
    assert_eq!(
        world
            .writes
            .iter()
            .filter(|write| matches!(write, api::Write::Merge { how: api::MergeHow::FastForwardOnly, .. }))
            .count(),
        1,
        "only the second turn merge reaches the fake; squash also records its member"
    );
}

#[test]
fn two_withdrawn_merges_hold_the_turn_with_the_last_outcome_and_reads() {
    use temper_engine_domain_forge_landing::{Hold, Phase};
    let (mut world, goal, branch) = goal_delivery::running_goal_with(true);
    assert_eq!(world.env.limits.forge.client.write_attempts, 2);
    let (change, producer) = goal_landing::producing_into_goal(&mut world, &goal, &branch, 101, b"first");
    finish_producer(&mut world, &producer, forge_world::REPO);
    let mut entries = Vec::new();
    let mut held = None;
    for _ in 0..STEPS {
        world.tick();
        if let Some((task, entry, pull)) = unsent_merge(&world)
            && !entries.contains(&entry)
        {
            entries.push(entry);
            world.send(engine::Event::Forge(Box::new(forge_top::Event::Withdraw { entry })));
            held = Some((task, pull));
        }
        if let Some((task, _)) = held
            && matches!(task_row(&world, task).expect("landing").phase, tasks::Phase::Held { .. })
        {
            break;
        }
    }
    let (task, pull) = held.expect("turn");
    assert_eq!(entries.len(), 2, "exactly two merge effects, not unbounded withdraw/remake");
    assert_eq!(landing(&world, task).landing.round.merges, 2);
    assert!(matches!(landing(&world, task).landing.phase, Phase::Held { why: Hold::Turn, .. }));
    let tip = branch_tip(&mut world, &branch);
    let expected = format!(
        "turn: merge of #{pull}: withdrawn before completion; read #{pull} open, {} at {}",
        String::from_utf8_lossy(&branch),
        String::from_utf8_lossy(&forge_top::commit_text(&translate::commit(tip)))
    );
    let tasks::Phase::Held { why: tasks::Hold::Procedure, source: Some(source), .. } = &live_task(&world, task).phase
    else {
        panic!("policy owns the hold: {:?}", live_task(&world, task).phase)
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("hold account") };
    assert_eq!(words.as_ref(), expected.as_bytes());
    assert_eq!(world.merges, 0, "neither unsent merge reached the forge");
    assert!(!ended(&world, change));
}
