use crate::{Note as Joined, WorkerJobEnding, worker_alone::Note};

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
        Joined::Worker(Note::WorkerJob {
            run: 7,
            attempt: 11,
            kind: 3,
            verb: b"merge".as_slice().into(),
            ending,
            why: b"deadline".as_slice().into(),
        })
    );
}
