//! Link pauses keep the hub up (jig/docs/design/domain/hosts/hub.md, 7).
use super::*;
use crate::{FromAgent, ToAgent};
use jig_domain_link::{PauseMode, PauseState};

fn ask(mode: PauseMode) -> Event {
    Event::Down(LinkDown::Pause { mode })
}

fn changed(state: PauseState) -> [Request; 2] {
    [Request::Up(LinkUp::Paused { state }), Request::Paused { state }]
}

fn job() -> Event {
    Event::Down(LinkDown::AssignJob {
        job: crate::JobAssignment {
            run: Token::new(7),
            attempt: Token::new(70),
            kind: crate::JobKind(7),
            spec: Box::new([]),
            deadline: skein_lib::Duration::ZERO,
            workspace: None,
            grants: Box::new([]),
        },
    })
}

fn sources_open(h: &mut Harness) {
    let name = crate::SourceName::new(b"login").expect("bounded source name");
    let requests = h.step(Event::Down(LinkDown::Sources { sources: Box::new([name]) }));
    let [Request::Source { name: checked, operation: crate::sources::Operation::Check, .. }] = &*requests else {
        panic!("link pauses keep sources open: {requests:?}")
    };
    assert_eq!(*checked, name);
}

fn turn(owner: Token) -> Event {
    Event::Agent(FromAgent::Turn {
        owner,
        turn: crate::Turn {
            opens_window: false,
            refused: Box::new([]),
            turn: 1,
            spent: 0,
            read: None,
            body: b"turn".as_slice().into(),
        },
    })
}

#[test]
fn a_link_pause_refuses_runs_and_jobs_but_keeps_the_live_runs_turn_and_answer() {
    let mut h = Harness::new();
    h.step(jobs::opened());
    let (owner, workspace, agent) = h.live(1);
    assert_eq!(&*h.step(ask(PauseMode::Pause)), changed(PauseState::Paused));
    assert_eq!(h.domain.hosted(), 1, "a slot is free");
    let requests = h.step(fixtures::assign(assignment(2)));
    let [
        Request::Up(LinkUp::Answer {
            answer: crate::RunAnswer { ending: crate::Ending::Refused(Refusal::Busy), .. },
            ..
        }),
    ] = &*requests
    else {
        panic!("paused run refusal: {requests:?}")
    };
    let requests = h.step(job());
    let [Request::Up(LinkUp::JobAnswer { answer: crate::JobEnding::Refused(crate::JobRefusal::Busy), .. })] =
        &*requests
    else {
        panic!("paused job refusal: {requests:?}")
    };
    let requests = h.step(turn(owner));
    let [Request::Up(LinkUp::Turn { .. }), Request::Agent(ToAgent::Acknowledge { .. })] = &*requests else {
        panic!("paused turn: {requests:?}")
    };
    assert_eq!(
        &*h.step(Event::Agent(FromAgent::Finished {
            owner,
            turns: 1,
            spent: 0,
            read: None,
            finish: Finish::Ended { outcome: b"done".as_slice().into() }
        })),
        [Request::Agent(ToAgent::Cancel { agent })]
    );
    assert_eq!(
        &*h.step(Event::Agent(FromAgent::Gone { owner, detail: Box::new([]) })),
        [Request::Save { owner, workspace }]
    );
    let requests = h.step(Event::Saved { owner, at: None });
    let [
        Request::Release { .. },
        Request::Up(LinkUp::Answer { answer: crate::RunAnswer { ending: crate::Ending::Ended { .. }, .. }, .. }),
    ] = &*requests
    else {
        panic!("paused answer: {requests:?}")
    };
    assert!(!h.domain.shut);
    sources_open(&mut h);
}

#[test]
fn a_link_drain_asks_active_and_waiting_runs_once_without_shutdown() {
    let mut h = Harness::new();
    let (_, _, active) = h.live(1);
    let (owner, _, waiting) = h.live(2);
    h.step(Event::Agent(FromAgent::Yielded { owner }));
    let expected = [
        Request::Up(LinkUp::Paused { state: PauseState::Draining }),
        Request::Paused { state: PauseState::Draining },
        Request::Agent(ToAgent::Drain { agent: active }),
        Request::Agent(ToAgent::Drain { agent: waiting }),
    ];
    assert_eq!(&*h.step(ask(PauseMode::Drain)), expected);
    sources_open(&mut h);
    assert!(!h.domain.shut);
    assert_eq!(&*h.step(ask(PauseMode::Drain)), [Request::Up(LinkUp::Paused { state: PauseState::Draining })]);
    assert_eq!(&*h.step(ask(PauseMode::Pause)), [Request::Up(LinkUp::Paused { state: PauseState::Draining })]);
    assert_eq!(&*h.step(ask(PauseMode::Resume)), changed(PauseState::Taking));
}

#[test]
fn a_link_drain_cancels_runs_without_a_turn_to_end_for_shutdown() {
    let mut h = Harness::new();
    let (preparing, space) = h.assign(1);
    let (starting, started_space) = h.assign(2);
    h.step(Event::Prepared { owner: starting, workspace: started_space });
    assert_eq!(&*h.step(ask(PauseMode::Drain)), changed(PauseState::Draining));
    crate::resume(&mut h.domain, &h.env, &mut h.out);
    assert_eq!(h.out.pop(), Some(Request::Abort { owner: preparing }));
    crate::resume(&mut h.domain, &h.env, &mut h.out);
    assert!(h.out.is_empty());
    let requests = h.step(Event::Prepared { owner: preparing, workspace: space });
    let [
        Request::Release { .. },
        Request::Up(LinkUp::Answer {
            answer:
                crate::RunAnswer {
                    ending: crate::Ending::Failed { failure: Failure::Cancelled(Reason::Shutdown), .. }, ..
                },
            ..
        }),
    ] = &*requests
    else {
        panic!("preparing cancellation: {requests:?}")
    };
    let agent = Token::new(302);
    assert_eq!(&*h.started(starting, agent), [Request::Agent(ToAgent::Cancel { agent })]);
    assert!(!h.domain.shut);
}

#[test]
fn drained_waits_for_the_job_and_every_turn_and_answer_acknowledgement() {
    let mut h = Harness::new();
    h.step(jobs::opened());
    let (owner, space, _) = h.live(1);
    let requests = h.step(job());
    let [Request::Job(crate::ToJob::Run { owner: job, .. })] = &*requests else { panic!("{requests:?}") };
    let job = *job;
    h.step(Event::Job(crate::FromJob::Started { owner: job }));
    h.step(turn(owner));
    let requests = h.step(ask(PauseMode::Drain));
    let [
        Request::Up(LinkUp::Paused { state: PauseState::Draining }),
        Request::Paused { .. },
        Request::Agent(ToAgent::Drain { .. }),
    ] = &*requests
    else {
        panic!("job runs on: {requests:?}")
    };
    h.step(Event::Agent(FromAgent::Finished {
        owner,
        turns: 1,
        spent: 0,
        read: None,
        finish: Finish::Failed { failure: crate::RunFailure::Cancelled, model: None },
    }));
    h.step(Event::Agent(FromAgent::Gone { owner, detail: Box::new([]) }));
    let requests = h.step(Event::Saved { owner, at: None });
    let [
        Request::Release { workspace },
        Request::Up(LinkUp::Answer {
            answer:
                crate::RunAnswer {
                    ending: crate::Ending::Failed { failure: Failure::Cancelled(Reason::Shutdown), .. }, ..
                },
            ..
        }),
    ] = &*requests
    else {
        panic!("drained cancellation: {requests:?}")
    };
    assert_eq!(*workspace, space);
    h.step(Event::Down(LinkDown::Acknowledge { run: Token::new(1), attempt: Token::new(1001) }));
    h.domain.reclaim();
    assert_eq!(h.domain.pause, PauseState::Draining, "job is still live");
    h.step(Event::Job(crate::FromJob::Ended {
        owner: job,
        end: crate::JobEnd::Finished { result: Box::new([]), delivery: None },
    }));
    h.step(Event::Job(crate::FromJob::Gone { owner: job, detail: Box::new([]) }));
    assert_eq!(h.domain.pause, PauseState::Draining, "job answer is still kept");
    h.step(Event::Down(LinkDown::Acknowledge { run: Token::new(7), attempt: Token::new(70) }));
    h.domain.reclaim();
    assert_eq!(h.domain.hosted(), 0);
    assert!(!h.domain.is_ready(), "an unacknowledged turn prevents drained even after reclaim");
    crate::resume(&mut h.domain, &h.env, &mut h.out);
    assert!(h.out.is_empty(), "no drained notification before the turn acknowledgement");
    assert_eq!(h.domain.pause, PauseState::Draining, "turn is still kept");
    assert_eq!(
        &*h.step(Event::Down(LinkDown::TurnKept { run: Token::new(1), attempt: Token::new(1001), turn: 1 })),
        changed(PauseState::Drained)
    );
    for mode in [PauseMode::Pause, PauseMode::Drain] {
        assert_eq!(&*h.step(ask(mode)), [Request::Up(LinkUp::Paused { state: PauseState::Drained })]);
    }
    assert_eq!(&*h.step(ask(PauseMode::Resume)), changed(PauseState::Taking));
    h.assign(2);
}

#[test]
fn the_link_drain_bound_cancels_runs_and_jobs_without_shutting_the_hub() {
    let mut h = Harness::new();
    h.step(jobs::opened());
    let (owner, _, agent) = h.live(1);
    let requests = h.step(job());
    let [Request::Job(crate::ToJob::Run { owner: job, .. })] = &*requests else { panic!("{requests:?}") };
    let job = *job;
    h.step(Event::Job(crate::FromJob::Started { owner: job }));
    h.step(ask(PauseMode::Drain));
    assert!(h.step(Event::DrainPassed).is_empty());
    sources_open(&mut h);
    for _ in 0_u32..2 {
        crate::resume(&mut h.domain, &h.env, &mut h.out);
    }
    assert_eq!(h.out.pop(), Some(Request::Agent(ToAgent::Cancel { agent })));
    assert_eq!(h.out.pop(), Some(Request::Job(crate::ToJob::Stop { owner: job })));
    assert!(h.out.is_empty());
    assert!(!h.domain.shut);
    assert_eq!(h.domain.ready.len(), 0);
    assert!(h.domain.hosting(owner).is_some());
    assert_eq!(&*h.step(ask(PauseMode::Resume)), changed(PauseState::Taking));
    // Free a slot through the cancellation tail; resume admits new work.
    let requests = h.step(Event::Job(crate::FromJob::Gone { owner: job, detail: Box::new([]) }));
    let [
        Request::Up(LinkUp::JobAnswer {
            answer: crate::JobEnding::Failed { failure: crate::JobFailure::Cancelled(Reason::Shutdown), .. },
            ..
        }),
    ] = &*requests
    else {
        panic!("bounded job cancellation: {requests:?}")
    };
    h.step(Event::Down(LinkDown::Acknowledge { run: Token::new(7), attempt: Token::new(70) }));
    h.domain.reclaim();
    h.assign(2);
}

#[test]
fn a_drain_bound_outside_draining_changes_nothing_and_resume_admits() {
    for mode in [PauseMode::Resume, PauseMode::Pause, PauseMode::Drain] {
        let mut h = Harness::new();
        if mode != PauseMode::Drain {
            h.live(1);
        }
        h.step(ask(mode));
        let before = h.domain.pause;
        assert!(h.step(Event::DrainPassed).is_empty());
        assert_eq!(h.domain.pause, before);
        assert!(h.domain.ready.is_empty(), "a drain bound outside draining cancels no live run");
        assert!(!h.domain.shut);
        h.step(ask(PauseMode::Resume));
        h.assign(2);
    }
}

#[test]
fn hello_repeats_a_link_pause_but_shutdown_wins_and_never_says_paused_again() {
    for mode in [PauseMode::Pause, PauseMode::Drain] {
        for shutdown in [Event::Drain, Event::CancelAll { reason: Reason::Shutdown }] {
            let mut h = Harness::new();
            h.live(1);
            h.step(ask(mode));
            for _ in 0_u32..2 {
                let requests = h.step(jobs::opened());
                let [
                    Request::Agent(ToAgent::Renewals { .. }),
                    Request::Hello { .. },
                    Request::Up(LinkUp::Paused { state }),
                ] = &*requests
                else {
                    panic!("paused hello: {requests:?}")
                };
                assert_eq!(*state, h.domain.pause);
            }
            let requests = h.step(shutdown);
            assert_eq!(requests.first(), Some(&Request::Up(LinkUp::Draining)));
            for mode in [PauseMode::Pause, PauseMode::Drain, PauseMode::Resume] {
                assert!(h.step(ask(mode)).is_empty());
            }
            assert!(h.step(Event::DrainPassed).is_empty());
            let requests = h.step(jobs::opened());
            let [Request::Agent(ToAgent::Renewals { .. }), Request::Hello { .. }, Request::Up(LinkUp::Draining)] =
                &*requests
            else {
                panic!("shutdown hello: {requests:?}")
            };
        }
    }
}
