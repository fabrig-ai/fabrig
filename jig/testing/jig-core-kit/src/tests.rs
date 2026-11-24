//! Focused workload rules (`jig/docs/design/domain/testing-workloads.md`, 8).
use crate::{
    bounds, parks, people,
    workload::{Bound, Expect, Party, Workload},
};
use skein_lib::{Duration, Time};
use skein_world::streams::{Stream, Streams};
use std::collections::BTreeSet;

#[test]
fn workload_kinds_have_unique_names_and_their_families() {
    for (family, kinds) in [
        (
            people::FAMILY,
            vec![
                people::CANCELS_A_TASK,
                people::HOLDS_A_TASK,
                people::RELEASES_A_TASK,
                people::ANSWERS_LATE,
                people::NEVER_ANSWERS,
                people::ANSWERS_TWICE,
                people::ANSWERS_AN_ENDED_RUN,
                people::WORDS_AFTER_ITS_RUN_ENDED,
            ],
        ),
        (
            parks::FAMILY,
            vec![
                parks::ANSWERED_WHILE_PARKED,
                parks::ANSWERED_AFTER_ITS_END,
                parks::ANSWERED_TO_ITS_SUCCESSOR,
                parks::COORDINATOR_ENDS_WHILE_A_CHILD_WAITS,
            ],
        ),
        (bounds::FAMILY, vec![bounds::SUBSCRIPTIONS, bounds::CREATION_REPORTS, bounds::KEPT_ANSWERS]),
    ] {
        assert!(kinds.iter().all(|kind| kind.family == family));
        assert_eq!(kinds.iter().map(|kind| kind.name).collect::<BTreeSet<_>>().len(), kinds.len());
    }
    assert_eq!(people::FAMILY, "people and parties");
    assert_eq!(parks::FAMILY, "runs that park, resume and move");
    assert_eq!(bounds::FAMILY, "the engine's own bounds");
    // These names are also the adapter's table names in jig-inspect/src/tables.rs.
    for (name, kind) in [
        ("subscriptions", bounds::SUBSCRIPTIONS),
        ("creation reports", bounds::CREATION_REPORTS),
        ("kept answers", bounds::KEPT_ANSWERS),
    ] {
        assert_eq!(bounds::table(name), kind);
    }
}

#[test]
fn workload_and_party_draws_do_not_consume_each_others_streams() {
    let streams = Streams::new(1);
    let workload = Workload::draw(streams);
    assert_eq!(workload.tasks[0].budget, 60, "Workload/tasks stream");
    assert_eq!(workload.tasks.len(), 6);
    let configured = [people::HOLDS_A_TASK, people::RELEASES_A_TASK, people::ANSWERS_LATE];
    let owner = workload.acts(streams, Party::Owner, &configured);
    let _ = streams.rng(Stream::Parties, "member").next_u64();
    let _ = workload.acts(streams, Party::Member, &[people::CANCELS_A_TASK]);
    assert_eq!(owner, workload.acts(streams, Party::Owner, &configured));
    assert_eq!(workload, Workload::draw(streams));
    let member = workload.acts(streams, Party::Member, &configured);
    let _ = workload.acts(streams, Party::Owner, &[]);
    assert_eq!(member, workload.acts(streams, Party::Member, &configured));
    assert_eq!(workload, Workload::draw(Streams::new(1)));
    assert_ne!(workload, Workload::draw(Streams::new(2)));
}

#[test]
fn workload_draws_only_eligible_targets_and_releases_its_own_hold() {
    let mut releases = 0;
    let mut questions = 0;
    for seed in 0..64 {
        let streams = Streams::new(seed);
        let workload = Workload::draw(streams);
        let configured = [
            people::CANCELS_A_TASK,
            people::HOLDS_A_TASK,
            people::RELEASES_A_TASK,
            people::ANSWERS_LATE,
            people::NEVER_ANSWERS,
            people::ANSWERS_TWICE,
            people::ANSWERS_AN_ENDED_RUN,
            people::WORDS_AFTER_ITS_RUN_ENDED,
            crate::open::AMENDS_A_PASSED_DEADLINE,
            crate::open::WIDENS_GOAL_AUTHORITY,
        ];
        for party in [Party::Owner, Party::Member] {
            let acts = workload.acts(streams, party, &configured);
            for (index, act) in acts.iter().enumerate() {
                if act.kind == crate::open::AMENDS_A_PASSED_DEADLINE {
                    assert!(party == Party::Owner || workload.tasks[act.target].party == party);
                } else {
                    assert_eq!(workload.tasks[act.target].party, party);
                }
                if act.kind == people::ANSWERS_LATE {
                    questions += 1;
                    assert_eq!(workload.tasks[act.target].kind, crate::workload::TaskKind::Question);
                }
                if act.kind == people::RELEASES_A_TASK {
                    releases += 1;
                    assert!(
                        acts[..index].iter().any(|hold| hold.kind == people::HOLDS_A_TASK && hold.target == act.target)
                    );
                }
            }
        }
        for (index, task) in workload.tasks.iter().enumerate() {
            if task.parent.is_none() {
                let count = workload.tasks[index..]
                    .iter()
                    .take_while(|next| next.name == task.name || next.name.starts_with(&(task.name.clone() + ".")))
                    .count();
                assert!(count <= 5);
            }
            assert!(task.name.split('.').count() <= 3);
        }
    }
    assert!(releases > 0 && questions > 0);
}

#[test]
fn workload_expectation_finds_a_held_task_past_its_bound() {
    let last = (Time::ZERO, 0);
    let moment = (Time::from_nanos(2_000_000_000), 2);
    let bound = Bound { time: Duration::from_secs(1), iterations: 1 };
    assert!(Expect::Ends.check(("t0.0", 42), 19, moment, last, bound, true).is_ok());
    let finding = Expect::Ends.check(("t0.0", 42), 19, moment, last, bound, false).expect_err("held beyond bound");
    for word in ["t0.0", "42", "19", "iteration 2"] {
        assert!(finding.contains(word), "{finding}");
    }
    assert!(Expect::Waits.check(("t0.0", 42), 19, moment, last, bound, false).is_ok());
}
