//! Channel-local pause asks and observations (domain/engine/workers.md, 5–6).
use super::*;
use jig_domain_link::{JobBound, JobKind, PauseMode, PauseState};

const WORKER: u32 = 37; // C1's configured position in Harness.
const JOB: JobBound = JobBound { kind: JobKind(7), spec: 4, result: 8, deadline: Duration::from_secs(3) };

fn worker(h: &mut Harness, channel: Token, position: u32) {
    h.step(Event::Hello {
        worker: position,
        channel,
        hello: Hello {
            reads_usage: true,
            slots: 2,
            stop_bound: Duration::ZERO,
            bounds: fixtures::bounds(),
            jobs: Box::new([JOB]),
            workstreams: Box::new([]),
            hosting: Box::new([]),
        },
    });
}

#[test]
fn pause_asks_keep_runs_and_jobs_off_freed_slots() {
    for mode in [PauseMode::Pause, PauseMode::Drain] {
        let mut h = Harness::new(LIMITS);
        assert_eq!(h.domain.worker_pause(WORKER), PauseState::Taking);
        worker(&mut h, C1, WORKER);
        assert_eq!(h.place(R1, A1, 1), C1);
        for _ in 0_u32..2 {
            assert_eq!(
                h.step(Event::PauseWorker { worker: WORKER, mode }).as_ref(),
                &[Request::Pause { channel: C1, mode }]
            );
        }
        assert_eq!(h.domain.worker_pause(WORKER), PauseState::Paused);
        assert!(!h.domain.host_views().get(0).expect("host").draining, "pause is not shutdown or busy");
        h.start(R2, A2, 2);
        h.step(Event::StartJob { reply_to: to(A3), run: R3, attempt: A3, workstream: 3, job: JOB, adopt: false });
        assert_eq!(h.answer(C1, R1, A1, Answer::Ended, 1).as_ref(), &[answered(A1, R1, Answer::Ended, 1)]);
        h.acknowledge(R1, A1);
        assert!(h.settle().is_empty(), "pause survives a hosted answer and its freed slot");
        worker(&mut h, C2, 38);
        assert_eq!(
            h.settle().as_ref(),
            &[
                assign(C2, R2, A2),
                placed(R2, A2),
                Request::AssignJob { channel: C2, run: R3, attempt: A3 },
                placed(R3, A3),
            ]
        );
        for state in [PauseState::Taking, PauseState::Draining, PauseState::Drained, PauseState::Paused] {
            h.step(Event::Paused { channel: C1, state });
            assert_eq!(
                h.domain.worker_pause(WORKER),
                if state == PauseState::Taking { PauseState::Paused } else { state }
            );
        }
    }
}

#[test]
fn worker_words_keep_placement_out_until_taking_even_after_resume() {
    for state in [PauseState::Paused, PauseState::Draining, PauseState::Drained] {
        let mut h = Harness::new(LIMITS);
        worker(&mut h, C1, WORKER);
        h.step(Event::Paused { channel: C1, state });
        assert_eq!(h.domain.worker_pause(WORKER), state);
        h.start(R1, A1, 1);
        assert!(h.settle().is_empty(), "the worker's word alone excludes placement");
        assert_eq!(
            h.step(Event::PauseWorker { worker: WORKER, mode: PauseMode::Resume }).as_ref(),
            &[Request::Pause { channel: C1, mode: PauseMode::Resume },]
        );
        assert!(h.settle().is_empty(), "a resume does not replace the worker's word");
        assert_eq!(h.domain.worker_pause(WORKER), state);
        h.step(Event::Paused { channel: C1, state: PauseState::Taking });
        assert_eq!(h.domain.worker_pause(WORKER), PauseState::Taking);
        assert_eq!(h.settle().as_ref(), &[assign(C1, R1, A1), placed(R1, A1)]);
    }
}

#[test]
fn pausing_requeues_a_chosen_attempt_and_loss_discards_the_pause() {
    for mode in [PauseMode::Pause, PauseMode::Drain] {
        let mut h = Harness::new(LIMITS);
        worker(&mut h, C1, WORKER);
        worker(&mut h, C2, 38);
        h.step(Event::Start {
            placement: Some(crate::PlacementConstraint { channel: C1, kind: HostKind::Worker }),
            version: crate::Version { charter: 1, revision: 1 },
            grants: Box::new([]),
            reply_to: to(A1),
            run: R1,
            attempt: A1,
            workstream: 1,
            kinds: Kinds::Workers,
            assignment: references(),
        });
        h.step(Event::PauseWorker { worker: WORKER, mode });
        assert_eq!(h.settle().as_ref(), &[assign(C2, R1, A1), placed(R1, A1)]);
        h.step(Event::Paused { channel: C1, state: PauseState::Drained });
        h.step(Event::Lost { channel: C1 });
        assert_eq!(h.domain.worker_pause(WORKER), PauseState::Taking);
        assert_eq!(h.domain.worker_channel(WORKER), None);
        worker(&mut h, C3, WORKER);
        h.step(Event::Paused { channel: C1, state: PauseState::Paused });
        assert_eq!(h.domain.worker_pause(WORKER), PauseState::Taking, "a stale channel cannot pause its replacement");
        assert_eq!(h.place(R2, A2, 2), C3, "the returning worker's new channel takes work");
    }
}
