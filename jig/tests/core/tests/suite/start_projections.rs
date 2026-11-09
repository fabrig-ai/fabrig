//! Direct started-start projection rows become ordinary serving feeds.
use jig_core_world::fixtures::{link, phase};
use jig_domain_core::{
    self as core,
    start::{Action, Phase, SweepInput},
};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, Time, Wall};

struct Harness {
    core: core::Core,
    env: Env<core::Limits>,
    out: Queue<core::Request>,
}
impl Harness {
    fn new() -> Self {
        let limits = jig_test_profile::limits().core;
        let mut h = Self {
            core: core::Core::new(jig_test_profile::config(411).core, &limits).expect("configured core"),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            out: Queue::uncapped(),
        };
        phase(&h.core.start_begin(false, Wall::EPOCH), Phase::LoadCore);
        let mut header = h.core.counters.deployment();
        header.commits = 1;
        assert_eq!(
            h.core.restore(&h.env, core::Record::Core(core::CoreRecord::Deployment(header))),
            core::Restore::Deployment { commits: 1 }
        );
        h
    }
    fn row(&mut self, row: core::ProjectionRecord) {
        assert_eq!(
            self.core.restore(&self.env, core::Record::Core(core::CoreRecord::Projection(row))),
            core::Restore::Kept
        );
    }
}
fn closing() -> tasks::Phase {
    tasks::Phase::Closing(tasks::Closing {
        stage: tasks::Stage::Effects,
        ending: tasks::Ending::Failed { reason: Box::new([]) },
    })
}
fn goal(pending: Box<[u16]>) -> core::ProjectionRecord {
    core::ProjectionRecord::Goal {
        goal: core::ProjectionGoal {
            number: 17,
            project: people::DEPLOYMENT,
            priority: 3,
            words: b"goal".as_slice().into(),
            phase: closing(),
        },
        pending,
        changed: true,
        fed_at: None,
    }
}
fn piece() -> core::ProjectionRecord {
    core::ProjectionRecord::Task {
        goal: 17,
        task: core::ProjectionTask {
            number: 18,
            requester: tasks::Party::Task(17),
            words: b"child".as_slice().into(),
            phase: closing(),
        },
    }
}

#[test]
fn early_pieces_and_pending_closing_header_become_a_serving_feed() {
    let mut h = Harness::new();
    let task = piece();
    let milestone = core::ProjectionMilestone {
        identity: core::MilestoneId::Lifecycle { task: 18, position: 0 },
        phase: None,
        change: None,
        words: b"stored milestone".as_slice().into(),
    };
    h.row(task.clone());
    h.row(core::ProjectionRecord::Milestone { goal: 17, milestone: milestone.clone() });
    let pending = h.core.connectors.clone();
    let header = goal(pending.clone());
    h.row(header.clone());
    assert_eq!(h.core.start_failure(), None);
    phase(&link(&mut h.core, &h.env, &mut h.out), Phase::Sweep);
    assert!(h.out.is_empty());
    phase(&h.core.start_sweep(&h.env, SweepInput::Finished, &mut h.out), Phase::Switch);
    phase(&h.core.start_done(&h.env, Phase::Switch, &mut h.out), Phase::Wires);
    let Action::Commit(token) = h.core.start_done(&h.env, Phase::Wires, &mut h.out) else {
        panic!("Wires waits for release")
    };
    phase(&h.core.start_resume(&h.env, token, &mut h.out), Phase::AdoptRuns);
    let mut output = core::finish_decision(&mut h.core, &h.env);
    let mut feeds = 0;
    while let Some(request) = output.pop() {
        if let core::Request::Ask { connector, ask: core::Ask::ProjectGoal { feed } } = request {
            assert!(pending.contains(&connector));
            let core::ProjectionRecord::Goal { goal, .. } = &header else { unreachable!() };
            let core::ProjectionRecord::Task { task, .. } = &task else { unreachable!() };
            assert_eq!(feed.goal, *goal);
            assert_eq!(feed.plan.as_ref(), std::slice::from_ref(task));
            assert_eq!(feed.milestones.as_ref(), std::slice::from_ref(&milestone));
            feeds += 1;
        }
    }
    assert_eq!(feeds, pending.len());
}

#[test]
fn an_orphan_stays_loaded_until_the_link_step() {
    let mut h = Harness::new();
    h.row(piece());
    assert_eq!(h.core.start_failure(), None);
    let expected = core::start::Failure::Link {
        record: core::Key::Core(core::CoreKey::Projection(core::ProjectionKey::Task { goal: 17, task: 18 })),
        linked: core::Linked::Record(core::Key::Core(core::CoreKey::Projection(core::ProjectionKey::Goal(17)))),
    };
    assert!(matches!(link(&mut h.core, &h.env, &mut h.out), Action::Refused(failure) if failure == expected));
    assert_eq!(h.core.start_failure(), Some(expected));
    assert_eq!(h.core.start_phase(), Some(Phase::Link));
    assert!(h.out.is_empty());
}

#[test]
fn a_fit_refusal_names_the_stored_projection_key() {
    let mut h = Harness::new();
    let mut row = goal(Box::new([]));
    let core::ProjectionRecord::Goal { goal, .. } = &mut row else { unreachable!() };
    let limit = h.env.limits.tasks.spec_bytes;
    goal.words = vec![b'x'; skein_lib::length::usize_of(limit + 1)].into_boxed_slice();
    let key = core::Key::Core(core::CoreKey::Projection(row.key()));
    assert_eq!(h.core.restore(&h.env, core::Record::Core(core::CoreRecord::Projection(row))), core::Restore::Refused);
    let expected = core::start::Failure::LiveSet {
        set: core::start::LiveSet::Projections,
        key,
        held: u64::from(limit) + 1,
        limit: u64::from(limit),
    };
    assert_eq!(h.core.start_failure(), Some(expected.clone()));
    assert!(
        matches!(h.core.start_done(&h.env, Phase::LoadCore, &mut h.out), Action::Refused(found) if found == expected)
    );
}
