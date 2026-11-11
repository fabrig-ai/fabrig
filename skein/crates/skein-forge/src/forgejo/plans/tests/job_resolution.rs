//! Status job resolution (skein/docs/design/forge/forgejo.md, 1.2 and 2).
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
fn checks_resolve_linked_jobs_independently_of_check_state() {
    skein_rows::check(
        [
            ("a_passed_check_resolves_to_its_jobs_id_not_its_links_index", ("success", api::Check::Passed)),
            ("pending_and_failed_checks_resolve_too: pending", ("pending", api::Check::Pending)),
            ("pending_and_failed_checks_resolve_too: failed", ("failure", api::Check::Failed)),
        ],
        |(state, check)| resolved_check(state, check),
    );
}
#[expect(clippy::disallowed_macros, reason = "wire fixtures vary status states and job links")]
fn page(rows: &[(&str, &str)], total: u64) -> Box<[u8]> {
    let head = format!(r#"{{"state":"pending","total_count":{total},"statuses":["#);
    let mut encoded = skein_lib::List::with_capacity(u32::try_from(rows.len()).unwrap());
    let mut size = head.len().saturating_add(2);
    for (index, (state, link)) in rows.iter().enumerate() {
        let row = format!(r#"{{"creator":null,"updated_at":"1970-01-01T00:00:00Z","context":"build","status":"{state}","description":"CI","target_url":"{link}"}}"#).into_bytes().into_boxed_slice();
        size = size.saturating_add(row.len()).saturating_add(usize::from(index > 0));
        encoded.push(row).unwrap();
    }
    let mut out = skein_lib::Writer::new(size);
    out.put(head.as_bytes()).unwrap();
    for (index, row) in encoded.iter().enumerate() {
        if index > 0 {
            out.put(b",").unwrap();
        }
        out.put(row).unwrap();
    }
    out.put(b"]}").unwrap();
    out.finish()
}

#[test]
fn statuses_of_one_run_share_its_lookup_and_each_take_their_own_job() {
    let mut run =
        Run::with_limits(fixtures::profile_limits(), fixtures::BOUNDS, Op::Read(Read::Checks { commit: HEAD }));
    run.begin();
    let second_link = "/fixture/specimen/actions/runs/5/jobs/1";
    run.answer(200, &page(&[("pending", LINK), ("success", second_link)], 2));
    run.answer(200, &page(&[], 2));
    run.answer(200, RUNS);
    let jobs = br#"[{"id":1,"run_id":7,"attempt":3},{"id":92,"run_id":7,"attempt":4}]"#;
    assert_eq!(
        run.answer(200, jobs),
        Step::Done(Ok(Answer::Checks(Box::new([
            status(api::Check::Pending, LINK, Some(ATTEMPT)),
            status(api::Check::Passed, second_link, Some(api::JobAttempt { job: 92, attempt: 4, ..ATTEMPT })),
        ]))))
    );
    let [runs, jobs] = LOOKUPS;
    run.assert_sent(&[
        "GET /api/v1/repos/fixture/specimen/commits/0100000000000000000000000000000000000000/status?page=1&limit=2",
        "GET /api/v1/repos/fixture/specimen/commits/0100000000000000000000000000000000000000/status?page=2&limit=2",
        runs,
        jobs,
    ]);
}

#[test]
fn linked_runs_are_read_once_each_in_first_status_order() {
    let mut run =
        Run::with_limits(fixtures::profile_limits(), fixtures::BOUNDS, Op::Read(Read::Checks { commit: HEAD }));
    run.begin();
    let other = "/fixture/specimen/actions/runs/8/jobs/0";
    run.answer(200, &page(&[("success", other), ("pending", LINK)], 3));
    run.answer(200, &body("failure", other, 3));
    run.answer(200, br#"{"total_count":1,"workflow_runs":[{"id":99,"index_in_repo":8,"commit_sha":"0100000000000000000000000000000000000000"}]}"#);
    run.answer(200, br#"[{"id":82,"run_id":99,"attempt":2}]"#);
    run.answer(200, RUNS);
    assert_eq!(
        run.answer(200, JOBS),
        Step::Done(Ok(Answer::Checks(Box::new([
            status(api::Check::Passed, other, Some(api::JobAttempt { run: 99, job: 82, attempt: 2, ..ATTEMPT })),
            status(api::Check::Pending, LINK, Some(ATTEMPT)),
            status(api::Check::Failed, other, Some(api::JobAttempt { run: 99, job: 82, attempt: 2, ..ATTEMPT })),
        ]))))
    );
    let [runs, jobs] = LOOKUPS;
    run.assert_sent(&[
        "GET /api/v1/repos/fixture/specimen/commits/0100000000000000000000000000000000000000/status?page=1&limit=2",
        "GET /api/v1/repos/fixture/specimen/commits/0100000000000000000000000000000000000000/status?page=2&limit=2",
        "GET /api/v1/repos/fixture/specimen/actions/runs?head_sha=0100000000000000000000000000000000000000&run_number=8&page=1&limit=1",
        "GET /api/v1/repos/fixture/specimen/actions/runs/99/jobs", runs, jobs,
    ]);
}

#[test]
fn statuses_resolve_their_page_and_keep_more() {
    skein_rows::check(
        [
            ("short despite a large total", ("pending", api::Check::Pending, api::Ci::Pending, 9, false)),
            ("full despite a page-local total", ("success", api::Check::Passed, api::Ci::Passed, 2, true)),
            ("short failed", ("failure", api::Check::Failed, api::Ci::Failed, 1, false)),
        ],
        |(state, check, ci, total, more)| {
            let mut run = Run::with_limits(
                fixtures::profile_limits(),
                fixtures::BOUNDS,
                Op::Read(Read::Statuses { commit: HEAD, page: 2 }),
            );
            run.begin();
            let statuses = if more {
                let link = "/fixture/specimen/actions/runs/5/jobs/1";
                let step = run.answer(200, &page(&[(state, LINK), (state, link)], total));
                let Step::Send(_) = step else { panic!("{step:?}") };
                Box::from([status(check, LINK, Some(ATTEMPT)), status(check, link, None)].as_slice())
            } else {
                let step = run.answer(200, &body(state, LINK, total));
                let Step::Send(_) = step else { panic!("{step:?}") };
                Box::from([status(check, LINK, Some(ATTEMPT))].as_slice())
            };
            // page()'s envelope is pending; use the same verdict for both full rows.
            let ci = if more { api::Ci::Pending } else { ci };
            run.answer(200, RUNS);
            assert_eq!(run.answer(200, JOBS), Step::Done(Ok(Answer::Statuses { ci, statuses, more })));
            let [runs, jobs] = LOOKUPS;
            run.assert_sent(&[
                "GET /api/v1/repos/fixture/specimen/commits/0100000000000000000000000000000000000000/status?page=2&limit=2",
                runs,
                jobs,
            ]);
        },
    );
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
        assert_eq!(
            step,
            Step::Done(if fits {
                Ok(expected.clone())
            } else {
                Err(Error::TooLarge(api::Excess::at_least(
                    api::Bound::AnswerBytes,
                    u64::from(answer_bytes),
                    u64::from(size),
                )))
            })
        );
    }
}
#[test]
fn a_pull_folds_ci_without_resolving_jobs() {
    let mut run = Run::known(Op::Read(Read::Pull { number: 1, ci: true }));
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

#[test]
#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "vary the wire envelope independently of its row"
)]
fn pull_ci_uses_a_short_first_envelope_except_warning() {
    skein_rows::check(
        [
            ("latest context envelope wins", ("success", "pending", api::Ci::Passed)),
            ("warning cannot hide pending", ("warning", "pending", api::Ci::Pending)),
            ("a failed envelope stays failed", ("failure", "success", api::Ci::Failed)),
        ],
        |(envelope, row, expected)| {
            let mut run = Run::known(Op::Read(Read::Pull { number: 1, ci: true }));
            run.begin();
            run.answer(200, fixtures::PULL);
            let body = body(row, LINK, 1);
            let body = core::str::from_utf8(&body).unwrap().replacen(
                &format!("\"state\":\"{row}\""),
                &format!("\"state\":\"{envelope}\""),
                1,
            );
            let Step::Done(Ok(Answer::Pull(pull))) = run.answer(200, body.as_bytes()) else {
                panic!("short page ends pull")
            };
            assert_eq!(pull.ci, expected);
        },
    );
}
