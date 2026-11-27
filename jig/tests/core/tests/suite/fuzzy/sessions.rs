//! Wake times and cold restores do not extend standing sessions (standing.md, 5).
use crate::sessions::{claimed_at, consume_and_park, ends, gap, opened, restart, say};
use jig_core_world::{fixtures, fixtures::live_row, world::World};
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{Duration, Rng, length::len_u64};
use skein_world::{
    sweep::{cases, gate},
    this_test,
};

#[test]
fn random_wakes_and_restarts_resume_only_warm_sessions_and_record_each_fresh_start() {
    let mut fresh_seen = false;
    let mut resumed_seen = false;
    let mut restarted = false;
    cases(this_test!("random_wakes_and_restarts_resume_only_warm_sessions_and_record_each_fresh_start"), 31..35, |seed| {
        let mut rng = Rng::new(seed);
        let (mut world, first, child) = opened();
        let mut run = first;
        for step in 0..12_u8 {
            consume_and_park(&mut world, run);
            let session = live_row(&world, run.0).session.expect("open work has a session");
            let previous_ends = ends(&world, run.0).len();
            let seconds = rng.below(541);
            gap(&mut world, seconds);
            if rng.below(2) == 0 {
                restart(&mut world, child);
                restarted = true;
                assert_eq!(live_row(&world, run.0).session, Some(session));
            }
            say(&mut world, run.0, 100 + step);
            let assigned = world.assigned.as_ref().expect("standing task's next run");
            assert_eq!(assigned.task, run.0, "seed {seed}, step {step}");
            let resumed = !assigned.transcript.is_empty();
            let next = (assigned.task, assigned.attempt);
            let elapsed = claimed_at(&world, next.0, next.1).as_nanos().saturating_sub(session.last.as_nanos());
            assert_eq!(
                resumed,
                elapsed <= Duration::from_secs(270).as_nanos(),
                "seed {seed}, step {step}, gap {seconds}"
            );
            let ended = ends(&world, run.0);
            if resumed {
                resumed_seen = true;
                assert_eq!(ended.len(), previous_ends);
                assert_eq!(live_row(&world, run.0).session, Some(session));
            } else {
                fresh_seen = true;
                assert_eq!(ended.len(), previous_ends + 1);
                assert_eq!(ended.last(), Some(&(session.first, tasks::SessionEnd::Quiet)));
                assert_eq!(live_row(&world, run.0).session.expect("fresh record").first, next.1);
                let began = world
                    .store
                    .rows
                    .values()
                    .find_map(|row| match row {
                        jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Activity(
                            line,
                        ))) if line.task == run.0
                            && matches!(line.line, tasks::Line::RunBegan { attempt, .. } if attempt == next.1) =>
                        {
                            Some(line.position)
                        }
                        _ => None,
                    })
                    .expect("fresh run began");
                assert!(world.store.rows.values().any(|row| matches!(row,
                    jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Activity(line)))
                        if line.task == run.0 && line.position + 1 == began
                            && line.line == tasks::Line::SessionEnded { attempt: session.first, end: tasks::SessionEnd::Quiet }
                )), "each fresh run immediately follows its session's end");
            }
            run = next;
        }
        world.close();
        Ok::<(), String>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
    if gate() {
        assert!(fresh_seen && resumed_seen && restarted);
    }
}

fn room_for_wake(world: &World, leaves_first: bool) -> bool {
    let kept = world
        .store
        .rows
        .keys()
        .filter(|key| matches!(key, root::Key::Core(core::Key::People(people::Key::Answer(_) | people::Key::Acted(_)))))
        .count();
    len_u64(kept) + 1 + u64::from(leaves_first) <= u64::from(world.env().limits.core.people.requests)
}

#[test]
fn random_talks_words_leaves_and_wakes_pair_talks_and_resume_only_with_a_live_talk() {
    cases(
        this_test!("random_talks_words_leaves_and_wakes_pair_talks_and_resume_only_with_a_live_talk"),
        51..55,
        |seed| {
            let mut rng = Rng::new(seed);
            let (mut world, mut run, _) = opened();
            let mut suppressed_quiet = false;
            let mut pairs = crate::sessions::TalkPairs::new(run.0);
            for step in 0..18_u8 {
                consume_and_park(&mut world, run);
                let session = live_row(&world, run.0).session;
                let seconds = match rng.below(3) {
                    0 => 1,
                    1 => 600,
                    _ => 1_801,
                };
                gap(&mut world, seconds);
                let action = rng.below(3);
                let leaves_first = action == 2 && live_row(&world, run.0).talk.is_some();
                if !room_for_wake(&world, leaves_first) {
                    // These longer gaps outlast retained requests in this world.
                    // Redraw the wait before either request, not after a refusal.
                    gap(&mut world, if rng.below(2) == 0 { 600 } else { 1_801 });
                }
                let row = live_row(&world, run.0);
                let state = tasks::talk_state(row.talk, row.session, world.wall_time());
                if action == 0 || row.talk.is_none() && action == 2 {
                    crate::sessions::talk(&mut world, run.0, 100 + step);
                } else {
                    if action == 2 {
                        fixtures::ask(&mut world, 100 + step, people::Ask::Leave { project: 1, task: run.0 });
                    }
                    say(&mut world, run.0, 150 + step);
                }
                let assigned = world.assigned.as_ref().expect("talk wake assigned");
                let next = (assigned.task, assigned.attempt);
                assert_eq!(next.0, run.0);
                assert_ne!(next.1, run.1);
                if let Some(session) = session {
                    let elapsed = claimed_at(&world, next.0, next.1).as_nanos().saturating_sub(session.last.as_nanos());
                    if !assigned.transcript.is_empty() && elapsed > Duration::from_secs(270).as_nanos() {
                        assert_eq!(state, tasks::TalkState::Open, "seed {seed}, step {step}");
                        assert_ne!(action, 2);
                        suppressed_quiet = true;
                    }
                }
                pairs.check(&world);
                run = next;
            }
            if gate() {
                assert!(suppressed_quiet && pairs.saw_lapse, "seed {seed} exercises suppression and lapse");
            }
            world.close();
            Ok::<(), String>(())
        },
    )
    .unwrap_or_else(|case| panic!("{case}"));
}
