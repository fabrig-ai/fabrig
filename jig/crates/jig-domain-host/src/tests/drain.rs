//! Turn-end shutdown (jig/docs/design/domain/hosts/hub.md, 7).
use super::*;
use crate::{FromAgent, ToAgent};

#[test]
fn a_drain_asks_active_and_waiting_runs_once_and_announces_before_their_outputs() {
    let mut h = Harness::new();
    let (_, _, active) = h.live(1);
    let (owner, _, waiting) = h.live(2);
    assert!(h.step(Event::Agent(FromAgent::Yielded { owner })).is_empty());
    assert_eq!(h.step(Event::Drain).as_ref(), [
        Request::Up(LinkUp::Draining),
        Request::Agent(ToAgent::Drain { agent: active }),
        Request::Agent(ToAgent::Drain { agent: waiting }),
    ]);
    assert!(h.step(Event::Drain).is_empty());
    for _ in 0..2 {
        let requests = h.step(jobs::opened());
        let [Request::Hello { .. }, Request::Up(LinkUp::Draining)] = requests.as_ref() else {
            panic!("hello while draining: {requests:?}");
        };
    }
}

#[test]
fn a_drain_refuses_new_runs_and_jobs_busy_even_with_free_slots() {
    let mut h = Harness::new();
    h.step(jobs::opened());
    assert_eq!(h.step(Event::Drain).as_ref(), [Request::Up(LinkUp::Draining)]);
    let requests = h.step(fixtures::assign(assignment(1)));
    let [Request::Up(LinkUp::Answer { answer: crate::RunAnswer { ending: crate::Ending::Refused(Refusal::Busy), .. }, .. })] = requests.as_ref() else {
        panic!("drain refuses run: {requests:?}");
    };
    let requests = h.step(Event::Down(LinkDown::AssignJob { job: crate::JobAssignment {
        run: Token::new(2), attempt: Token::new(20), kind: crate::JobKind(7),
        spec: Box::new([]), deadline: skein_lib::Duration::ZERO, workspace: None, grants: Box::new([]),
    }}));
    let [Request::Up(LinkUp::JobAnswer { answer: crate::JobEnding::Refused(crate::JobRefusal::Busy), .. })] = requests.as_ref() else {
        panic!("drain refuses job: {requests:?}");
    };
    assert_eq!(h.domain.hosted(), 0);
}

#[test]
fn a_drain_cancels_preparing_and_starting_runs_without_a_turn_to_end() {
    let mut h = Harness::new();
    let (preparing, prepared_space) = h.assign(1);
    let (starting, started_space) = h.assign(2);
    h.step(Event::Prepared { owner: starting, workspace: started_space });
    assert_eq!(h.step(Event::Drain).as_ref(), [Request::Up(LinkUp::Draining)]);
    crate::resume(&mut h.domain, &h.env, &mut h.out);
    assert_eq!(h.out.pop(), Some(Request::Abort { owner: preparing }));
    assert!(h.out.is_empty());
    crate::resume(&mut h.domain, &h.env, &mut h.out);
    assert!(h.out.is_empty());
    let requests = h.step(Event::Prepared { owner: preparing, workspace: prepared_space });
    cancelled_without_save(&requests, prepared_space);
    let agent = Token::new(302);
    assert_eq!(h.step(Event::Agent(FromAgent::Started { owner: starting, agent })).as_ref(), [Request::Agent(ToAgent::Cancel { agent })]);
    let requests = h.step(Event::Agent(FromAgent::Gone { owner: starting, detail: Box::new([]) }));
    cancelled_without_save(&requests, started_space);
}

fn cancelled_without_save(requests: &[Request], space: Token) {
    let [Request::Release { workspace }, Request::Up(LinkUp::Answer { answer, .. })] = requests else {
        panic!("cancel tail: {requests:?}");
    };
    assert_eq!(*workspace, space);
    assert_eq!(answer.ending, crate::Ending::Failed {
        failure: Failure::Cancelled(Reason::Shutdown), detail: Box::new([]), work: Work { left: None, saved: None },
    });
}

#[test]
fn a_drained_runs_answer_saves_releases_and_keeps_what_it_said() {
    let work = Work { left: None, saved: Some(Token::new(77)) };
    for (finish, ending) in [
        (Finish::Failed { failure: crate::RunFailure::Cancelled }, crate::Ending::Failed {
            failure: Failure::Cancelled(Reason::Shutdown), detail: Box::new([]), work: work.clone(),
        }),
        (Finish::Parked, crate::Ending::Parked { work: work.clone() }),
        (Finish::Ended { outcome: b"accepted".as_slice().into() }, crate::Ending::Ended { outcome: b"accepted".as_slice().into(), work: work.clone() }),
        (Finish::Failed { failure: crate::RunFailure::Model }, crate::Ending::Failed {
            failure: Failure::Run(crate::RunFailure::Model), detail: Box::new([]), work: work.clone(),
        }),
    ] {
        let mut h = Harness::new();
        let (owner, workspace, agent) = h.live(1);
        assert_eq!(h.step(Event::Drain).as_ref(), [Request::Up(LinkUp::Draining), Request::Agent(ToAgent::Drain { agent })]);
        assert_eq!(h.step(fixtures::finished(owner, finish)).as_ref(), [Request::Agent(ToAgent::Cancel { agent })]);
        assert_eq!(h.step(Event::Agent(FromAgent::Gone { owner, detail: Box::new([]) })).as_ref(), [Request::Save { owner, workspace }]);
        let requests = h.step(Event::Saved { owner, at: work.saved });
        let [Request::Release { workspace: released }, Request::Up(LinkUp::Answer { answer, .. })] = requests.as_ref() else {
            panic!("saved drain tail: {requests:?}");
        };
        assert_eq!(*released, workspace);
        assert_eq!(answer.ending, ending);
    }
}

#[test]
fn cancel_after_a_drain_stops_live_runs_but_not_one_already_stopping() {
    let mut h = Harness::new();
    let (stopping, _, first) = h.live(1);
    let (_, _, second) = h.live(2);
    h.step(Event::Drain);
    assert_eq!(h.step(fixtures::finished(stopping, Finish::Parked)).as_ref(), [Request::Agent(ToAgent::Cancel { agent: first })]);
    assert!(h.step(Event::CancelAll { reason: Reason::Shutdown }).is_empty());
    crate::resume(&mut h.domain, &h.env, &mut h.out);
    assert!(h.out.is_empty());
    crate::resume(&mut h.domain, &h.env, &mut h.out);
    assert_eq!(h.out.pop(), Some(Request::Agent(ToAgent::Cancel { agent: second })));
    assert!(h.out.is_empty());
    assert!(h.step(Event::CancelAll { reason: Reason::Shutdown }).is_empty());
    for _ in 0..2 {
        crate::resume(&mut h.domain, &h.env, &mut h.out);
        assert!(h.out.is_empty());
    }
}
