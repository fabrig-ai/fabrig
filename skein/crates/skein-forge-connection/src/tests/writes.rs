//! Write plans through the component's scripted byte peer.
use skein_forge::api::Error;
use skein_forge::forgejo::failure::Refusal;
use skein_forge::forgejo::plans::failure::{self, Asked, Classified, Drop};
use skein_lib::Duration;
fn failure_rows() -> [FailureRow; 25] {
    [
        ("redirect", Asked::Read, 301, None, None, Error::Unavailable, Drop::Path),
        ("missing", Asked::Read, 404, None, None, Error::Missing, Drop::Path),
        ("missing job", Asked::Job, 404, None, None, Error::MissingJob, Drop::Nothing),
        (
            "rate",
            Asked::Read,
            429,
            None,
            Some(Duration::from_secs(5)),
            Error::RateLimited { after: Duration::from_secs(5) },
            Drop::Nothing,
        ),
        (
            "rate without header",
            Asked::Write,
            429,
            None,
            None,
            Error::RateLimited { after: Duration::ZERO },
            Drop::Nothing,
        ),
        ("quota", Asked::Write, 413, None, None, Error::Full, Drop::Nothing),
        ("archived", Asked::Write, 423, None, None, Error::Forbidden, Drop::Row),
        ("style", Asked::Merge, 405, Some(Refusal::MergeStyle), None, Error::Refused, Drop::Row),
        ("old head", Asked::Merge, 409, Some(Refusal::HeadOutOfDate), None, Error::Stale, Drop::Nothing),
        ("delete protection", Asked::DeleteBranch, 403, None, None, Error::Protected, Drop::Nothing),
        ("unknown merge", Asked::Merge, 405, None, None, Error::Refused, Drop::Nothing),
        ("unknown read", Asked::Read, 418, None, None, Error::InvalidAnswer, Drop::Nothing),
        ("before POST", Asked::BeforeWrite, 500, None, None, Error::Unavailable, Drop::Nothing),
        ("unknown before POST", Asked::BeforeWrite, 418, None, None, Error::Unavailable, Drop::Nothing),
        ("already open", Asked::OpenPull, 409, None, None, Error::Exists, Drop::Nothing),
        ("closed", Asked::Merge, 405, Some(Refusal::MergeClosed), None, Error::Closed, Drop::Nothing),
        ("merge conflict", Asked::Merge, 405, Some(Refusal::MergeConflict), None, Error::Conflict, Drop::Nothing),
        ("merge protection", Asked::Merge, 405, Some(Refusal::MergeProtected), None, Error::Protected, Drop::Nothing),
        ("update conflict", Asked::Update, 409, Some(Refusal::UpdateConflict), None, Error::Conflict, Drop::Nothing),
        ("empty", Asked::Write, 422, Some(Refusal::Empty), None, Error::Empty, Drop::Nothing),
        ("unknown 422", Asked::OpenPull, 422, None, None, Error::Refused, Drop::Nothing),
        ("token", Asked::Write, 401, None, None, Error::Forbidden, Drop::Row),
        ("permission", Asked::Write, 403, None, None, Error::Forbidden, Drop::Row),
        ("5xx", Asked::Write, 500, None, None, Error::Timeout, Drop::Nothing),
        ("unexpected success", Asked::Write, 202, None, None, Error::Timeout, Drop::Nothing),
    ]
}
#[test]
fn failures_classify_by_status_call_and_message() {
    for (name, asked, status, refusal, retry, error, drop) in failure_rows() {
        assert_eq!(failure::classify(asked, status, refusal, retry), Classified { error, drop }, "{name}");
    }
}
type FailureRow = (&'static str, Asked, u16, Option<Refusal>, Option<Duration>, Error, Drop);
