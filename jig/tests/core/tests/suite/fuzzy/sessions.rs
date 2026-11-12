//! Wake times and cold restores do not extend standing sessions (standing.md, 5).
use crate::sessions::{claimed_at, consume_and_park, ends, gap, opened, restart, say};
use jig_core_world::fixtures::live_row;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, Rng};

#[test]
fn random_wakes_and_restarts_resume_only_warm_sessions_and_record_each_fresh_start() {
    let mut fresh_seen = false;
    let mut resumed_seen = false;
    let mut restarted = false;
    for seed in 31..35 {
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
    }
    assert!(fresh_seen && resumed_seen && restarted);
}

#[test]
fn random_talks_words_leaves_and_wakes_pair_talks_and_resume_only_with_a_live_talk() {
    use jig_core_world::fixtures;
    use jig_domain_core as core;
    use jig_domain_people as people;
    use jig_test_domain as root;
    for seed in 51..55 {
        let mut rng = Rng::new(seed);
        let (mut world, mut run, _) = opened();
        let mut suppressed_quiet = false;
        let mut saw_lapse = false;
        for step in 0..18_u8 {
            consume_and_park(&mut world, run);
            let session = live_row(&world, run.0).session;
            let seconds = match rng.below(3) {
                0 => 1,
                1 => 600,
                _ => 1_801,
            };
            gap(&mut world, seconds);
            let row = live_row(&world, run.0);
            let state = tasks::talk_state(row.talk, row.session, world.wall_time());
            let action = rng.below(3);
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
            let mut open = None;
            for record in world.store.rows.values() {
                if let root::Record::Core(core::Record::Tasks(tasks::Stored::Activity(line))) = record
                    && line.task == run.0
                {
                    match line.line {
                        tasks::Line::TalkOpened { party } => assert_eq!(open.replace(party), None),
                        tasks::Line::TalkEnded { party, lapsed } => {
                            assert_eq!(open.take(), Some(party), "each opened talk ends exactly once");
                            saw_lapse |= lapsed;
                        }
                        _ => {}
                    }
                }
            }
            assert_eq!(open, live_row(&world, run.0).talk.map(|talk| talk.party));
            run = next;
        }
        assert!(suppressed_quiet && saw_lapse, "seed {seed} exercises suppression and lapse");
        world.close();
    }
}
