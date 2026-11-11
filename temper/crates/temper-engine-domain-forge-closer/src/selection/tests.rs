use super::*;
use skein_forge::api::{Check, Status};
use skein_lib::Time;

const HEAD: Commit = [1; 32];
const NEXT: Commit = [2; 32];
fn attempt(job: u64) -> JobAttempt {
    JobAttempt { head: HEAD, run: 3, job, attempt: 1 }
}
#[expect(clippy::disallowed_methods, reason = "test fixtures map optional job identities")]
fn status(job: Option<u64>) -> Status {
    Status {
        at: Time::from_nanos(0),
        context: Box::new([]),
        check: Check::Passed,
        description: Box::new([]),
        url: Box::new([]),
        job: job.map(attempt),
    }
}
fn closer() -> Closer {
    let mut closer = Closer::new();
    closer.selections = Box::new([PullSelection::new(4, Some(HEAD)), PullSelection::new(9, Some(NEXT))]);
    closer
}
fn job(closer: &mut Closer, id: u64, log: &[u8]) {
    assert_eq!(selection_read(closer), Some(Read::Job { attempt: attempt(id), max_bytes: 4096, part: LogPart::Start }));
    selection_answer(closer, Ok(Answer::Job { attempt: attempt(id), step: None, log: log.into(), truncated: true }));
}
#[test]
fn selections_read_checks_then_jobs_in_forge_order_and_stop_at_the_first_line() {
    let mut closer = closer();
    assert_eq!(selection_read(&closer), Some(Read::Checks { commit: HEAD }));
    selection_answer(
        &mut closer,
        Ok(Answer::Checks(Box::new([status(None), status(Some(7)), status(Some(2)), status(Some(8))]))),
    );
    job(&mut closer, 7, b"cargo ...\nselection: ignored");
    job(&mut closer, 2, b"selection: full\n...");
    assert_eq!(
        closer.selections[0].progress,
        SelectionProgress::Done { line: Some(b"selection: full".as_slice().into()) }
    );
    assert_eq!(selection_read(&closer), Some(Read::Checks { commit: NEXT }));
    selection_answer(&mut closer, Ok(Answer::Checks(Box::new([status(None)]))));
    assert_eq!(closer.selections[1].progress, SelectionProgress::Done { line: None });
    assert_eq!(selection_read(&closer), None);
}
#[test]
#[expect(clippy::disallowed_methods, reason = "the test constructs nine statuses to cross the eight-job bound")]
fn selection_reads_at_most_eight_jobs_then_settles_unknown() {
    assert_eq!(SELECTION_JOBS, 8);
    let mut closer = closer();
    selection_answer(&mut closer, Ok(Answer::Checks((0..9).map(|id| status(Some(id))).collect())));
    for id in 0..8 {
        job(&mut closer, id, b"cargo ...");
    }
    assert_eq!(closer.selections[0].progress, SelectionProgress::Done { line: None });
    assert_eq!(selection_read(&closer), Some(Read::Checks { commit: NEXT }));
}
#[test]
fn failed_checks_or_log_reads_settle_unknown_and_continue_with_the_next_pull() {
    for checks_fail in [true, false] {
        let mut closer = closer();
        if !checks_fail {
            selection_answer(&mut closer, Ok(Answer::Checks(Box::new([status(Some(1)), status(Some(2))]))));
        }
        selection_answer(&mut closer, Err(Error::Refused));
        assert_eq!(closer.selections[0].progress, SelectionProgress::Done { line: None });
        assert_eq!(selection_read(&closer), Some(Read::Checks { commit: NEXT }));
    }
}
#[test]
fn a_pull_without_a_landed_head_reads_its_head_before_its_checks() {
    let mut closer = Closer::new();
    closer.selections = Box::new([PullSelection::new(4, None)]);
    assert_eq!(selection_read(&closer), Some(Read::Pull { number: 4, ci: false }));
    selection_answer(&mut closer, Err(Error::Refused));
    assert_eq!(selection_read(&closer), None);
}
