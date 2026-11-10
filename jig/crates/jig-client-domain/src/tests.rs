mod carried;
mod fixtures;
mod limits;
mod reads;
mod requests;
mod text;
mod versions;
mod watches;

/// Each compact phase selects its task query class (jig/docs/design/client/domain/testing.md, 4).
#[test]
fn every_task_phase_has_its_query_class() {
    use crate::contract::{Phase, PhaseClass};
    for (phase, class) in [
        (Phase::Waiting, PhaseClass::Waiting),
        (Phase::Idle, PhaseClass::Working),
        (Phase::Due, PhaseClass::Working),
        (Phase::Preparing, PhaseClass::Working),
        (Phase::Claimed, PhaseClass::Working),
        (Phase::Running, PhaseClass::Working),
        (Phase::BackingOff, PhaseClass::Working),
        (Phase::Active, PhaseClass::Working),
        (Phase::Closing, PhaseClass::Closing),
        (Phase::Held, PhaseClass::Held),
        (Phase::Ended, PhaseClass::Ended),
        (Phase::Done, PhaseClass::Ended),
        (Phase::Failed, PhaseClass::Ended),
        (Phase::Cancelled, PhaseClass::Ended),
    ] {
        assert_eq!(phase.class(), class, "{phase:?}");
    }
}

/// A party cannot create a job (jig/docs/design/client/domain/contract/documents.md, 7).
#[test]
fn create_naming_a_job_is_refused_as_not_offered() {
    use crate::contract::{self as c, Admission, Executor, Refusal};
    for offer in [Admission::Allowed, Admission::Refused(Refusal::Unknown)] {
        assert_eq!(
            c::admission::creation(&offer, Executor::Job { kind: 7 }, 0, None),
            Admission::Refused(Refusal::NotOffered)
        );
    }
}
