use super::fixtures;
use crate::{Note as Joined, Refusal, WorkerJobEnding, WorkerLoss, worker_alone::Note};

#[test]
fn a_refused_git_start_stops_and_aborts_the_workers_service() {
    let mut service = super::worker::alone_service();
    let mut stand = fixtures::WorkerStand::new();
    stand.worker = service.worker;
    stand.below.script.invocations.front_mut().unwrap().transcript.stdout = b"git version 2.40.0\n".as_slice().into();
    for _ in 0_u16..1000 {
        if !stand.turn() {
            break;
        }
    }
    let problems = stand.worker.refused().expect("git's version is refused").clone();
    service.worker = stand.worker;
    crate::worker_alone::iterate_stage(&mut service, crate::Stage::Worker, fixtures::NOW, skein_lib::Wall::EPOCH);
    assert_eq!(service.end(), Some(&crate::worker_alone::End::Stopped(crate::worker_alone::Stop::Start(problems))));
    assert!(service.worker.aborted.is_some(), "refused startup aborts every worker child");
}

#[test]
fn a_worker_job_keeps_every_field_when_joined() {
    let ending = WorkerJobEnding::Failed {
        failure: jig_domain_host::JobFailure::Deadline,
        detail: b"deadline detail".as_slice().into(),
    };
    let note = Note::WorkerJob {
        run: 7,
        attempt: 11,
        kind: 3,
        verb: b"merge".as_slice().into(),
        ending: ending.clone(),
        why: b"deadline".as_slice().into(),
    };
    assert_eq!(
        Joined::from(note),
        Joined::WorkerJob {
            run: 7,
            attempt: 11,
            kind: 3,
            verb: b"merge".as_slice().into(),
            ending,
            why: b"deadline".as_slice().into(),
        }
    );
}
#[test]
fn a_linked_note_keeps_every_field_when_joined() {
    let address: Box<core::net::SocketAddr> = Box::new(([127, 0, 0, 1], 7100).into());
    let note = Note::Linked { worker: b"worker-a".as_slice().into(), address: address.clone() };
    assert_eq!(Joined::from(note), Joined::Linked { worker: b"worker-a".as_slice().into(), address });
}
#[test]
fn a_lost_note_keeps_every_field_when_joined() {
    let address: Box<core::net::SocketAddr> = Box::new(([127, 0, 0, 1], 7101).into());
    let why = WorkerLoss::Silence;
    let note = Note::Lost { worker: b"worker-b".as_slice().into(), address: address.clone(), why };
    assert_eq!(Joined::from(note), Joined::Lost { worker: b"worker-b".as_slice().into(), address, why });
}
#[test]
fn a_refused_note_keeps_every_field_when_joined() {
    let address: Box<core::net::SocketAddr> = Box::new(([127, 0, 0, 1], 7102).into());
    let reason = Refusal::Peer(256);
    let note = Note::Refused { worker: b"worker-c".as_slice().into(), address: address.clone(), reason };
    assert_eq!(Joined::from(note), Joined::Refused { worker: b"worker-c".as_slice().into(), address, reason });
}
