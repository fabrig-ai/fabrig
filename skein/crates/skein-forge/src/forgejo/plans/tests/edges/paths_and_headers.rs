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
    [Answer::PullFiles { head: HEAD, files: Box::new([file.clone()]), more: false, cut: false }]
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
fn actions_leaf_404_keeps_the_path() {
    let job = Read::Job { part: api::LogPart::End, attempt: ATTEMPT, max_bytes: RAW_BYTES };
    for (name, read, answers, error) in [
        ("run read", Read::Run { run: 7 }, &[][..], Error::Missing),
        ("run", job.clone(), &[][..], Error::MissingJob),
        ("job", job.clone(), &[(200, RUN)][..], Error::MissingJob),
        ("log", job, &[(200, RUN), (200, JOB)][..], Error::MissingJob),
    ] {
        let mut run = Run::known(Op::Read(read));
        replies(&mut run, answers);
        assert_eq!(run.answer(404, MISSING), Step::Done(Err(error)), "{name}");
        assert_eq!(run.tables().repository(REPO), Some(&fixtures::specimen()), "{name}");
        let mut next = crate::forgejo::plans::plan::Plan::new(REPO, Op::Read(Read::Settings));
        assert_eq!(
            next.begin(fixtures::BOUNDS, fixtures::WORD, run.tables()),
            Ok(Step::Done(Ok(Answer::Settings(api::Settings {
                default_branch: Box::from(*b"main"),
                merge: true,
                squash: false,
                rebase: false,
            })))),
            "{name}"
        );
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
            Read::PullFiles { number: 1, head: HEAD, page: 1, skip: 0, sides: true },
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
#[expect(clippy::wildcard_enum_match_arm, reason = "the table asks only for the three real-row listings")]
fn items_keep_whole_rows_until_the_answer_bytes_and_name_the_cut() {
    // Summaries, reviews and remarks have distinct fixed widths and whole bodies.
    for (kind, row_bytes) in [(0, 130_u32), (1, 129), (2, 66)] {
        for (count, room, skip, kept, cut) in [
            (8, 5_u32, 0, 5, true),
            (6, 5, 0, 5, true),
            (8, 8, 0, 8, false),
            (8, 7, 0, 7, true),
            (8, 5, 5, 3, false),
            (8, 1, 5, 1, true),
        ] {
            if kind == 0 && skip != 0 {
                continue;
            }
            let body = sized_page(kind, count);
            let mut limits = fixtures::limits();
            limits.json.length = u32::try_from(body.len()).unwrap();
            let answer_bytes = room.saturating_mul(row_bytes);
            let bounds = crate::forgejo::plans::plan::Bounds { page_size: 8, answer_bytes, ..fixtures::BOUNDS };
            let mut run = Run::with_limits(limits, bounds, Op::Read(sized_read(kind, skip)));
            run.begin();
            let answer = run.answer(200, &body);
            let Step::Done(Ok(answer)) = answer else { panic!("whole prefix: {answer:?}") };
            assert_eq!(crate::measure::answer(&answer), Some(u64::from(kept * row_bytes)));
            let mut ids = skein_lib::List::with_capacity(8);
            let (more, actual_cut) = match answer {
                Answer::Items { items, more, cut, .. } => {
                    for row in items {
                        ids.push(row.number).unwrap();
                    }
                    (more, cut)
                }
                Answer::Reviews { reviews, more, cut } => {
                    for row in reviews {
                        ids.push(row.id).unwrap();
                    }
                    (more, cut)
                }
                Answer::Remarks { remarks, more, cut } => {
                    for row in remarks {
                        ids.push(row.id).unwrap();
                    }
                    (more, cut)
                }
                value => panic!("listing: {value:?}"),
            };
            assert_eq!(ids.len(), kept);
            for (index, id) in ids.iter().enumerate() {
                assert_eq!(*id, u64::from(skip).saturating_add(u64::try_from(index).unwrap()).saturating_add(1));
            }
            assert_eq!(more, cut || (kind != 2 && count == 8));
            assert_eq!(actual_cut, cut);
        }
        let body = sized_page(kind, 8);
        let mut limits = fixtures::limits();
        limits.json.length = u32::try_from(body.len()).unwrap();
        let mut run = Run::with_limits(
            limits,
            crate::forgejo::plans::plan::Bounds {
                page_size: 8,
                answer_bytes: row_bytes.saturating_sub(1),
                ..fixtures::BOUNDS
            },
            Op::Read(sized_read(kind, 0)),
        );
        run.begin();
        assert_eq!(
            run.answer(200, &body),
            Step::Done(Err(Error::TooLarge(api::Excess::at_least(
                api::Bound::AnswerBytes,
                u64::from(row_bytes.saturating_sub(1)),
                u64::from(row_bytes)
            ))))
        );
    }
}
fn sized_read(kind: u8, skip: u32) -> Read {
    match kind {
        0 => Read::Items { since: Time::ZERO, page: 1, kind: None, single: false },
        1 => Read::Reviews { number: 1, page: 1, skip },
        2 => Read::Remarks { number: 1, review: 7, page: 1, skip },
        _ => unreachable!(),
    }
}
fn sized_page(kind: u8, count: u8) -> Box<[u8]> {
    if kind == 0 {
        return items_page(count);
    }
    let tail = if kind == 1 {
        br#","user":{"id":2,"login":"writer"},"state":"APPROVED","commit_id":"0100000000000000000000000000000000000000","body":"b","submitted_at":"1970-01-01T00:00:00Z","official":false,"dismissed":false}"#.as_slice()
    } else {
        br#","user":{"id":2,"login":"writer"},"path":"x","position":1,"body":"b"}"#.as_slice()
    };
    let mut body =
        skein_lib::Writer::new(usize::from(count).saturating_mul(tail.len().saturating_add(8)).saturating_add(1));
    body.put(b"[").unwrap();
    for number in 1_u8..=count {
        if number > 1 {
            body.put(b",").unwrap();
        }
        body.put(b"{\"id\":").unwrap();
        body.put(&[b'0'.saturating_add(number)]).unwrap();
        body.put(tail).unwrap();
    }
    body.put(b"]").unwrap();
    body.finish()
}

#[test]
fn a_single_item_page_requests_one_and_keeps_fullness_at_one() {
    let mut run = Run::known(Op::Read(Read::Items { since: Time::ZERO, page: 4, kind: None, single: true }));
    run.begin();
    run.assert_listing("issues?state=all&since=1970-01-01T00%3A00%3A00Z&sort=leastupdate&page=4&limit=", b"1");
    let Step::Done(Ok(Answer::Items { items, more, cut, .. })) = run.answer(200, &items_page(1)) else {
        panic!("single page")
    };
    assert_eq!(items.len(), 1);
    assert!(more);
    assert!(!cut);
}

fn items_page(count: u8) -> Box<[u8]> {
    let row = br#","state":"open","user":{"id":2,"login":"writer"},"labels":[],"updated_at":"1970-01-01T00:00:00Z","title":"t","body":"b"}"#;
    let row_bytes = b"{\"number\":".len().checked_add(1).unwrap().checked_add(row.len()).unwrap();
    let length = usize::from(count).checked_mul(row_bytes.checked_add(1).unwrap()).unwrap().checked_add(1).unwrap();
    let mut body = skein_lib::Writer::new(length);
    body.put(b"[").unwrap();
    for number in 1_u8..=count {
        if number > 1 {
            body.put(b",").unwrap();
        }
        body.put(b"{\"number\":").unwrap();
        body.put(&[b'0'.checked_add(number).unwrap()]).unwrap();
        body.put(row).unwrap();
    }
    body.put(b"]").unwrap();
    body.finish()
}

// The full page contains PAGE_SIZE rows; the short page contains one fewer.
const FULL_FILES: &[u8] = br#"[{"filename":"x","status":"added"},{"filename":"y","status":"added"}]"#;
const SHORT_FILES: &[u8] = br#"[{"filename":"x","status":"added"}]"#;
const FULL_PATHS: [&[u8]; 2] = [b"x", b"y"];
fn added_file(path: &[u8]) -> api::File {
    api::File { path: Box::from(path), before: None, after: Some(Box::new([])) }
}
fn pull_files_page(body: &[u8], paths: &[&[u8]], has_more: Option<bool>, sides: bool) -> Step {
    let mut run = Run::known(Op::Read(Read::PullFiles { number: 1, head: HEAD, page: 1, skip: 0, sides }));
    run.begin();
    run.answer(200, PULL);
    let mut step = run.answer_with(200, body, crate::forgejo::plans::Fields { has_more, ..fixtures::fields() });
    if sides {
        for _path in paths {
            step = run.answer(200, b"");
        }
    } else {
        run.assert_sent(&[
            "GET /api/v1/repos/fixture/specimen/pulls/1",
            "GET /api/v1/repos/fixture/specimen/pulls/1/files?page=1&limit=2",
        ]);
    }
    step
}

struct PullFilesPage {
    body: &'static [u8],
    paths: &'static [&'static [u8]],
    has_more: Option<bool>,
    sides: bool,
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
                    sides: true,
                    expected: Step::Done(Ok(Answer::PullFiles {
                        head: HEAD,
                        files: Box::new([added_file(b"x")]),
                        more: false,
                        cut: false,
                    })),
                }
            }),
            ("full_pull_files_without_has_more_has_more", {
                PullFilesPage {
                    body: FULL_FILES,
                    paths: &FULL_PATHS[..usize::try_from(PAGE_SIZE).unwrap()],
                    has_more: None,
                    sides: true,
                    expected: Step::Done(Ok(Answer::PullFiles {
                        head: HEAD,
                        files: Box::new([added_file(b"x"), added_file(b"y")]),
                        more: true,
                        cut: false,
                    })),
                }
            }),
            ("explicit_last_pull_files_page_has_no_more", {
                PullFilesPage {
                    body: FULL_FILES,
                    paths: &FULL_PATHS[..usize::try_from(PAGE_SIZE).unwrap()],
                    has_more: Some(false),
                    sides: true,
                    expected: Step::Done(Ok(Answer::PullFiles {
                        head: HEAD,
                        files: Box::new([added_file(b"x"), added_file(b"y")]),
                        more: false,
                        cut: false,
                    })),
                }
            }),
            (
                "paths_only_last_page",
                PullFilesPage {
                    body: FULL_FILES,
                    paths: &FULL_PATHS,
                    has_more: Some(false),
                    sides: false,
                    expected: Step::Done(Ok(Answer::PullFiles {
                        head: HEAD,
                        files: Box::new([
                            api::File { path: Box::from(*b"x"), before: None, after: None },
                            api::File { path: Box::from(*b"y"), before: None, after: None },
                        ]),
                        more: false,
                        cut: false,
                    })),
                },
            ),
        ],
        |case| assert_eq!(pull_files_page(case.body, case.paths, case.has_more, case.sides), case.expected),
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
    let mut run = Run::known(Op::Read(Read::Item { number: 1, after: 0, since: Time::ZERO }));
    run.begin();
    assert_eq!(run.answer(404, MISSING), Step::Done(Err(Error::Missing)));
    assert_eq!(run.tables().repository(REPO), None);
    for read in [
        Read::Settings,
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
    let mut run = Run::known(Op::Read(Read::PullFiles { number: 1, head: HEAD, page: 3, skip: 0, sides: true }));
    run.begin();
    run.answer(200, PULL);
    assert_eq!(
        run.answer(200, b"null"),
        Step::Done(Ok(Answer::PullFiles { head: HEAD, files: Box::new([]), more: false, cut: false }))
    );
    run.assert_sent(&[
        "GET /api/v1/repos/fixture/specimen/pulls/1",
        "GET /api/v1/repos/fixture/specimen/pulls/1/files?page=3&limit=2",
    ]);
}
