//! Answering 404s retain cached paths, and listing headers govern their own page.
use super::{MISSING, replies, side_cases};
use crate::api::{self, Answer, Error, Op, Read};
use crate::forgejo::plans::plan::Step;
use crate::forgejo::tests::fixtures::{self, ATTEMPT, HEAD, JOB, PAGE_SIZE, PULL, RAW_BYTES, REPO, RUN, Run};
use alloc::boxed::Box;
use skein_lib::Time;

const FAILING: &[u8] = br#"{"state":"failure","total_count":1,"statuses":[{"creator":null,"updated_at":"1970-01-01T00:00:00Z","context":"build","status":"failure","description":"failed","target_url":"/fixture/specimen/actions/runs/5/jobs/1"}]}"#;
const RUNS: &[u8] = br#"{"total_count":1,"workflow_runs":[{"id":7,"index_in_repo":5,"commit_sha":"0100000000000000000000000000000000000000"}]}"#;

fn side_answers(file: api::File) -> [Answer; 1] {
    [Answer::PullFiles { head: HEAD, files: Box::new([file.clone()]), more: false }]
}
fn failed_check() -> api::Status {
    api::Status {
        at: Time::ZERO,
        context: Box::from(&b"build"[..]),
        check: api::Check::Failed,
        description: Box::from(&b"failed"[..]),
        url: Box::from(&b"http://127.0.0.1:3000/fixture/specimen/actions/runs/5/jobs/1"[..]),
        job: None,
    }
}

#[test]
fn answering_404_keeps_the_path() {
    for (before, after, statuses) in
        [(None, Some(Box::from(&b""[..])), [404, 200]), (Some(Box::from(&b""[..])), None, [200, 404])]
    {
        let file = api::File { path: Box::from(super::FILE_PATH), before, after };
        for ((read, answers, _sends), expected) in side_cases().into_iter().zip(side_answers(file)) {
            let mut run = Run::known(Op::Read(read));
            replies(&mut run, answers);
            run.answer(statuses[0], b"");
            assert_eq!(run.answer(statuses[1], b""), Step::Done(Ok(expected)));
            assert_eq!(run.tables().repository(REPO), Some(&fixtures::specimen()));
        }
    }
    for (name, answers) in [("run", &[(200, FAILING)][..]), ("job", &[(200, FAILING), (200, RUNS)][..])] {
        let mut run = Run::known(Op::Read(Read::Checks { commit: HEAD }));
        replies(&mut run, answers);
        assert_eq!(run.answer(404, MISSING), Step::Done(Ok(Answer::Checks(Box::new([failed_check()])))), "{name}");
        assert_eq!(run.tables().repository(REPO), Some(&fixtures::specimen()), "{name}");
    }
}

#[test]
fn terminal_404_drops_the_path() {
    let job = Read::Job { part: api::LogPart::End, attempt: ATTEMPT, max_bytes: RAW_BYTES };
    for (name, read, answers, error) in [
        ("run", job.clone(), &[][..], Error::MissingJob),
        ("job", job.clone(), &[(200, RUN)][..], Error::MissingJob),
        ("log", job, &[(200, RUN), (200, JOB)][..], Error::MissingJob),
    ] {
        let mut run = Run::known(Op::Read(read));
        replies(&mut run, answers);
        assert_eq!(run.answer(404, MISSING), Step::Done(Err(error)), "{name}");
        assert_eq!(run.tables().repository(REPO), None, "{name}");
    }
}

#[test]
fn redirect_drops_the_path() {
    for (name, read, answers) in [
        ("file", Read::File { head: HEAD, path: Box::from(super::FILE_PATH), max_bytes: RAW_BYTES }, &[][..]),
        ("job", Read::Job { part: api::LogPart::End, attempt: ATTEMPT, max_bytes: RAW_BYTES }, &[(200, RUN)][..]),
        ("checks", Read::Checks { commit: HEAD }, &[(200, FAILING)][..]),
        (
            "pull side",
            Read::PullFiles { number: 1, head: HEAD, page: 1 },
            &[(200, PULL), (200, super::CHANGED_FILES)][..],
        ),
    ] {
        let mut run = Run::known(Op::Read(read));
        replies(&mut run, answers);
        assert_eq!(run.answer(302, b""), Step::Done(Err(Error::Unavailable)), "{name}");
        assert_eq!(run.tables().repository(REPO), None, "{name}");
    }
}

#[test]
fn items_without_date_is_invalid_answer() {
    let mut run = Run::new(Op::Read(Read::Items { single: false, since: Time::ZERO, page: 1, kind: None }));
    run.begin();
    run.answer(200, fixtures::REPOSITORY);
    assert_eq!(run.answer_with(200, b"[]", fixtures::fields()), Step::Done(Err(Error::InvalidAnswer)));
    run.assert_sent(&[
        "GET /api/v1/repositories/1",
        "GET /api/v1/repos/fixture/specimen/issues?state=all&since=1970-01-01T00%3A00%3A00Z&sort=leastupdate&page=1&limit=2",
    ]);
}

#[test]
fn items_keep_whole_rows_until_the_answer_bytes_and_name_the_cut() {
    // Each retained summary has 128 fixed bytes, a one-byte title and one-byte body.
    const ROW_BYTES: u32 = 130;
    let body = (1..=8).map(|number| format!(r#"{{"number":{number},"state":"open","user":{{"id":2,"login":"writer"}},"labels":[],"updated_at":"1970-01-01T00:00:00Z","title":"t","body":"b"}}"#)).collect::<Vec<_>>().join(",");
    let body = format!("[{body}]");
    let mut limits = fixtures::limits();
    limits.json.length = u32::try_from(body.len()).unwrap();
    for (answer_bytes, kept, cut) in [(5 * ROW_BYTES, 5, true), (8 * ROW_BYTES, 8, false), (8 * ROW_BYTES - 1, 7, true)] {
        let bounds = crate::forgejo::plans::plan::Bounds { page_size: 8, answer_bytes, ..fixtures::BOUNDS };
        let mut run = Run::with_limits(limits, bounds, Op::Read(Read::Items { since: Time::ZERO, page: 1, kind: None, single: false }));
        run.begin();
        let answer = run.answer(200, body.as_bytes());
        let Step::Done(Ok(Answer::Items { items, more, cut: actual_cut, .. })) = answer else { panic!("whole prefix of items: {answer:?}") };
        assert_eq!(items.len(), kept);
        assert_eq!(items.iter().map(|item| item.number).collect::<Vec<_>>(), (1..=u64::try_from(kept).unwrap()).collect::<Vec<_>>());
        assert!(more, "a cut or full page has more");
        assert_eq!(actual_cut, cut);
        assert_eq!(crate::measure::answer(&Answer::Items { items, more, cut, now: Time::ZERO }), Some(u64::try_from(kept).unwrap() * u64::from(ROW_BYTES)));
    }
    let mut run = Run::with_limits(limits, crate::forgejo::plans::plan::Bounds { page_size: 8, answer_bytes: ROW_BYTES - 1, ..fixtures::BOUNDS }, Op::Read(Read::Items { since: Time::ZERO, page: 1, kind: None, single: false }));
    run.begin();
    assert_eq!(run.answer(200, body.as_bytes()), Step::Done(Err(Error::TooLarge(api::Excess::at_least(api::Bound::AnswerBytes, u64::from(ROW_BYTES - 1), u64::from(ROW_BYTES))))));
}

// The full page contains PAGE_SIZE rows; the short page contains one fewer.
const FULL_FILES: &[u8] = br#"[{"filename":"x","status":"added"},{"filename":"y","status":"added"}]"#;
const SHORT_FILES: &[u8] = br#"[{"filename":"x","status":"added"}]"#;
const FULL_PATHS: [&[u8]; 2] = [b"x", b"y"];
fn added_file(path: &[u8]) -> api::File {
    api::File { path: Box::from(path), before: None, after: Some(Box::new([])) }
}
fn pull_files_page(body: &[u8], paths: &[&[u8]], has_more: Option<bool>) -> Step {
    let mut run = Run::known(Op::Read(Read::PullFiles { number: 1, head: HEAD, page: 1 }));
    run.begin();
    run.answer(200, PULL);
    let mut step = run.answer_with(200, body, crate::forgejo::plans::Fields { has_more, ..fixtures::fields() });
    for _path in paths {
        step = run.answer(200, b"");
    }
    step
}

struct PullFilesPage {
    body: &'static [u8],
    paths: &'static [&'static [u8]],
    has_more: Option<bool>,
    expected: Step,
}
#[test]
fn pull_file_pagination_uses_headers_or_page_fullness() {
    skein_rows::check(
        [
            ("short_pull_files_without_has_more_is_last", {
                let short_rows = usize::try_from(PAGE_SIZE.saturating_sub(1)).unwrap();
                PullFilesPage {
                    body: SHORT_FILES,
                    paths: &FULL_PATHS[..short_rows],
                    has_more: None,
                    expected: Step::Done(Ok(Answer::PullFiles {
                        head: HEAD,
                        files: Box::new([added_file(b"x")]),
                        more: false,
                    })),
                }
            }),
            ("full_pull_files_without_has_more_has_more", {
                PullFilesPage {
                    body: FULL_FILES,
                    paths: &FULL_PATHS[..usize::try_from(PAGE_SIZE).unwrap()],
                    has_more: None,
                    expected: Step::Done(Ok(Answer::PullFiles {
                        head: HEAD,
                        files: Box::new([added_file(b"x"), added_file(b"y")]),
                        more: true,
                    })),
                }
            }),
            ("explicit_last_pull_files_page_has_no_more", {
                PullFilesPage {
                    body: FULL_FILES,
                    paths: &FULL_PATHS[..usize::try_from(PAGE_SIZE).unwrap()],
                    has_more: Some(false),
                    expected: Step::Done(Ok(Answer::PullFiles {
                        head: HEAD,
                        files: Box::new([added_file(b"x"), added_file(b"y")]),
                        more: false,
                    })),
                }
            }),
        ],
        |case| assert_eq!(pull_files_page(case.body, case.paths, case.has_more), case.expected),
    );
}

#[test]
fn branch_and_contents_404_keep_the_row() {
    for (read, misses) in [
        (Read::Branch { branch: Box::from(&b"topic"[..]) }, 1_u32),
        (Read::File { head: HEAD, path: Box::from(super::FILE_PATH), max_bytes: RAW_BYTES }, 2_u32),
    ] {
        let mut run = Run::known(Op::Read(read));
        let mut answer = run.begin();
        for _ in 0..misses {
            answer = run.answer(404, MISSING);
        }
        assert_eq!(answer, Step::Done(Err(Error::Missing)));
        assert_eq!(run.tables().repository(REPO), Some(&fixtures::specimen()));
    }
}

#[test]
fn repository_404_drops_the_row() {
    for read in [Read::Settings, Read::Item { number: 1, after: 0, since: Time::ZERO }] {
        let mut run = Run::known(Op::Read(read));
        run.begin();
        assert_eq!(run.answer(404, MISSING), Step::Done(Err(Error::Missing)));
        assert_eq!(run.tables().repository(REPO), None);
    }
    for read in [
        Read::Branch { branch: Box::from(&b"topic"[..]) },
        Read::File { head: HEAD, path: Box::from(super::FILE_PATH), max_bytes: RAW_BYTES },
    ] {
        let mut run = Run::new(Op::Read(read));
        run.begin();
        run.tables().keep_repository(REPO, fixtures::specimen());
        assert_eq!(run.answer(404, MISSING), Step::Done(Err(Error::Missing)));
        assert_eq!(run.tables().repository(REPO), None, "resolution is not a leaf");
    }
}

#[test]
fn null_pull_files_page_past_last_finishes_at_the_observed_head() {
    let mut run = Run::known(Op::Read(Read::PullFiles { number: 1, head: HEAD, page: 3 }));
    run.begin();
    run.answer(200, PULL);
    assert_eq!(
        run.answer(200, b"null"),
        Step::Done(Ok(Answer::PullFiles { head: HEAD, files: Box::new([]), more: false }))
    );
    run.assert_sent(&[
        "GET /api/v1/repos/fixture/specimen/pulls/1",
        "GET /api/v1/repos/fixture/specimen/pulls/1/files?page=3&limit=2",
    ]);
}
