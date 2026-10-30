//! Status job resolution (skein/docs/design/forge.md, 5.2 and 9).
use crate::api::{self, Answer, Error, Op, Read};
use crate::forgejo::plans::plan::Step;
use crate::forgejo::tests::fixtures::{self, HEAD, REPO, Run};
use alloc::{boxed::Box, format};
use skein_lib::Time;

// The link selects run number 5 and job index 0; their API ids are 7 and 1.
const LINK: &str = "/fixture/specimen/actions/runs/5/jobs/0";
const RUNS: &[u8] = br#"{"total_count":1,"workflow_runs":[{"id":7,"index_in_repo":5,"commit_sha":"0100000000000000000000000000000000000000"}]}"#;
const JOBS: &[u8] = br#"[{"id":1,"run_id":7,"attempt":3}]"#;
const ATTEMPT: api::JobAttempt = api::JobAttempt { head: HEAD, run: 7, job: 1, attempt: 3 };
const LOOKUPS: [&str; 2] = [
    "GET /api/v1/repos/fixture/specimen/actions/runs?head_sha=0100000000000000000000000000000000000000&run_number=5&page=1&limit=1",
    "GET /api/v1/repos/fixture/specimen/actions/runs/7/jobs",
];

#[expect(clippy::disallowed_macros, reason = "wire fixtures vary the status state, link and page count")]
fn body(state: &str, link: &str, total: u64) -> Box<[u8]> {
    format!(r#"{{"state":"{state}","total_count":{total},"statuses":[{{"creator":null,"updated_at":"1970-01-01T00:00:00Z","context":"build","status":"{state}","description":"CI","target_url":"{link}"}}]}}"#).into_bytes().into_boxed_slice()
}
#[expect(clippy::disallowed_macros, reason = "the expectation makes each fixture link absolute")]
fn status(check: api::Check, link: &str, job: Option<api::JobAttempt>) -> api::Status {
    api::Status {
        at: Time::ZERO,
        context: Box::from(&b"build"[..]),
        check,
        description: Box::from(&b"CI"[..]),
        url: format!("http://127.0.0.1:3000{link}").into_bytes().into_boxed_slice(),
        job,
    }
}
fn resolved_check(state: &str, check: api::Check) {
    let mut run = Run::known(Op::Read(Read::Checks { commit: HEAD }));
    run.begin();
    run.answer(200, &body(state, LINK, 1));
    run.answer(200, RUNS);
    assert_eq!(run.answer(200, JOBS), Step::Done(Ok(Answer::Checks(Box::new([status(check, LINK, Some(ATTEMPT))])))));
    let [runs, jobs] = LOOKUPS;
    run.assert_sent(&[
        "GET /api/v1/repos/fixture/specimen/commits/0100000000000000000000000000000000000000/status?page=1&limit=2",
        runs,
        jobs,
    ]);
}
#[test]
fn a_passed_check_resolves_to_its_jobs_id_not_its_links_index() {
    resolved_check("success", api::Check::Passed);
}
#[test]
fn pending_and_failed_checks_resolve_too() {
    for (state, check) in [("pending", api::Check::Pending), ("failure", api::Check::Failed)] {
        resolved_check(state, check);
    }
}
#[test]
fn statuses_resolve_their_page_and_keep_more() {
    for (state, check, ci) in [
        ("pending", api::Check::Pending, api::Ci::Pending),
        ("success", api::Check::Passed, api::Ci::Passed),
        ("failure", api::Check::Failed, api::Ci::Failed),
    ] {
        for (total, more) in [(3, false), (4, true)] {
            let mut run = Run::known(Op::Read(Read::Statuses { commit: HEAD, page: 2 }));
            run.begin();
            run.answer(200, &body(state, LINK, total));
            run.answer(200, RUNS);
            assert_eq!(
                run.answer(200, JOBS),
                Step::Done(Ok(Answer::Statuses { ci, statuses: Box::new([status(check, LINK, Some(ATTEMPT))]), more }))
            );
            let [runs, jobs] = LOOKUPS;
            run.assert_sent(&[
                "GET /api/v1/repos/fixture/specimen/commits/0100000000000000000000000000000000000000/status?page=2&limit=2",
                runs,
                jobs,
            ]);
        }
    }
}
#[test]
fn an_unresolvable_link_gives_no_job() {
    for (read, checks) in [(Read::Checks { commit: HEAD }, true), (Read::Statuses { commit: HEAD, page: 1 }, false)] {
        for (link, replies) in [
            ("/external-ci", &[][..]),
            ("/another/repository/actions/runs/5/jobs/0", &[][..]),
            (LINK, &[(200, br#"{"total_count":0,"workflow_runs":[]}"#.as_slice())][..]),
            (LINK, &[(404, b"{}".as_slice())][..]),
            (LINK, &[(200, RUNS), (404, b"{}".as_slice())][..]),
            (LINK, &[(200, RUNS), (200, b"[]".as_slice())][..]),
            (LINK, &[(200, RUNS), (200, br#"[{"id":1,"run_id":8,"attempt":3}]"#.as_slice())][..]),
            (LINK, &[(200, br#"{"total_count":1,"workflow_runs":[{"id":7,"index_in_repo":6,"commit_sha":"0100000000000000000000000000000000000000"}]}"#.as_slice())][..]),
            (LINK, &[(200, br#"{"total_count":1,"workflow_runs":[{"id":7,"index_in_repo":5,"commit_sha":"0300000000000000000000000000000000000000"}]}"#.as_slice())][..]),
        ] {
            let mut run = Run::known(Op::Read(read.clone()));
            run.begin();
            let mut step = run.answer(200, &body("success", link, 1));
            for (code, bytes) in replies {
                step = run.answer(*code, bytes);
            }
            let statuses = Box::new([status(api::Check::Passed, link, None)]);
            let expected = if checks {
                Answer::Checks(statuses)
            } else {
                Answer::Statuses { ci: api::Ci::Passed, statuses, more: false }
            };
            assert_eq!(step, Step::Done(Ok(expected)), "{read:?}: {link} {replies:?}");
            assert_eq!(run.tables().repository(REPO), Some(&fixtures::specimen()));
        }
    }
}
#[test]
fn resolving_statuses_keeps_the_answer_byte_bound() {
    let expected = Answer::Statuses {
        ci: api::Ci::Passed,
        statuses: Box::new([status(api::Check::Passed, LINK, Some(ATTEMPT))]),
        more: false,
    };
    let size =
        u32::try_from(crate::forgejo::plans::reads::size::answer(&expected, fixtures::BOUNDS.rows).unwrap()).unwrap();
    for (answer_bytes, fits) in [(size, true), (size.saturating_sub(1), false)] {
        let mut run = Run::with(
            crate::forgejo::plans::plan::Bounds { answer_bytes, ..fixtures::BOUNDS },
            Op::Read(Read::Statuses { commit: HEAD, page: 1 }),
        );
        run.begin();
        let mut step = run.answer(200, &body("success", LINK, 1));
        if fits {
            run.answer(200, RUNS);
            step = run.answer(200, JOBS);
        }
        assert_eq!(step, Step::Done(if fits { Ok(expected.clone()) } else { Err(Error::TooLarge) }));
    }
}
#[test]
fn a_pull_folds_ci_without_resolving_jobs() {
    let mut run = Run::known(Op::Read(Read::Pull { number: 1 }));
    run.begin();
    run.answer(200, fixtures::PULL);
    let Step::Done(Ok(Answer::Pull(pull))) = run.answer(200, &body("success", LINK, 1)) else {
        panic!("pull ends after its status page");
    };
    assert_eq!(pull.ci, api::Ci::Passed);
    run.assert_sent(&[
        "GET /api/v1/repos/fixture/specimen/pulls/1",
        "GET /api/v1/repos/fixture/specimen/commits/0100000000000000000000000000000000000000/status?page=1&limit=2",
    ]);
}
