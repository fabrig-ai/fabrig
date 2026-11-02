mod carried;
mod fixtures;
mod limits;
mod reads;
mod requests;
mod text;
mod versions;
mod watches;

/// A party cannot create a job (jig/docs/design/client/domain/contract.md, 5.7).
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
