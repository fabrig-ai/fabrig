use super::fixtures;
use crate::{Note as Joined, Refusal, WorkerJobEnding, WorkerLoss, worker_alone::Note};



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
