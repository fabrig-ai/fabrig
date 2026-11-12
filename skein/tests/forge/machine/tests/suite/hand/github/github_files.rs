//! GitHub contents operations through the component and the in-process fake.
use skein_forge::api::{Answer, Error, File, Op, Read};
use skein_forge_connection::seam::Access;
use skein_forge_machine_world::github::{Files as Fake, object};
use skein_forge_machine_world::hand::{self as f, CallHeap};
use skein_lib::Duration;
use std::boxed::Box;

fn harness(fake: &Fake) -> f::Harness {
    harness_with(fake, f::shared())
}
fn harness_with(fake: &Fake, shared: skein_forge_connection::Shared) -> f::Harness {
    let mut settings = f::github_settings();
    settings.forges.get_mut(0).expect("forge").access = None;
    let limits = skein_forge_connection::Limits {
        kept_bytes: u64::from(shared.calls) * u64::from(shared.answer_bytes),
        ..f::limits()
    };
    let mut h = f::Harness::with(limits, shared, settings);
    h.component.test_calls_ready(f::app_access(skein_fake_forge_github::keys::KEYS.first().expect("test key"), 7, 9));
    f::github_account(&mut h, 1, 22, b"acme", fake.token.expose());
    f::github_repository(&mut h, f::REPO, 22);
    h
}
fn run(h: &mut f::Harness, fake: &mut Fake, read: Read) -> skein_forge_connection::Answered {
    h.call(1, f::REPO, Op::Read(read));
    h.run(Duration::ZERO);
    h.connected(f::FORGE);
    for _ in 0_u32..64 {
        if h.request_bytes(f::FORGE).is_empty() {
            h.connected(f::FORGE);
        }
        let response = fake.answer(&h.request_bytes(f::FORGE));
        h.answer(f::FORGE, &response);
        h.run(Duration::ZERO);
        if let Some(answer) = h.answered(1) {
            return answer.clone();
        }
    }
    unreachable!("contents plan settles within sixty-four exchanges")
}
#[test]
fn each_contents_operation_answers_through_the_github_fake() {
    let mut fake = Fake::new(b"contents", false);
    let first = object(fake.first);
    let head = object(fake.head);
    let file =
        File { path: Box::from(b"file".as_slice()), before: None, after: Some(Box::from(b"contents".as_slice())) };
    for (read, cost, expected) in [
        (
            Read::File { head, path: Box::from(b"file".as_slice()), max_bytes: 8 },
            1,
            Answer::File {
                head,
                path: Box::from(b"file".as_slice()),
                bytes: Box::from(b"contents".as_slice()),
                truncated: false,
            },
        ),
        (
            Read::Compare { before: first, after: head },
            1,
            Answer::Compare {
                before: first,
                after: head,
                files: Box::new([file.path.clone()]),
                ahead: 1,
                behind: 0,
                changed: 1,
            },
        ),
        (
            Read::PullFiles { number: fake.number, head, page: 1, skip: 0, sides: true },
            4,
            Answer::PullFiles { head, files: Box::new([file]), more: false, cut: false },
        ),
    ] {
        let mut h = harness(&fake);
        let answer = run(&mut h, &mut fake, read);
        assert_eq!((answer.cost, answer.result), (cost, Ok(expected)));
    }
}
#[test]
fn file_retains_its_bound_and_marks_one_byte_past_it() {
    // The requested head is smaller than both the answer and HTTP chunk caps.
    for (content, max, kept, truncated) in [
        (b"four".as_slice(), 4, b"four".as_slice(), false),
        (b"fives".as_slice(), 4, b"five".as_slice(), true),
        (b"x".as_slice(), 0, b"".as_slice(), true),
    ] {
        let mut fake = Fake::new(content, false);
        let mut h = harness(&fake);
        let head = object(fake.head);
        let answer = run(&mut h, &mut fake, Read::File { head, path: Box::from(b"file".as_slice()), max_bytes: max });
        assert_eq!(
            (answer.cost, answer.result),
            (1, Ok(Answer::File { head, path: Box::from(b"file".as_slice()), bytes: Box::from(kept), truncated }))
        );
    }
}
#[test]
fn pull_files_reads_before_sides_at_the_merge_base() {
    let mut fake = Fake::new(b"feature only", true);
    assert_ne!(fake.base, fake.first);
    let mut h = harness(&fake);
    let head = object(fake.head);
    let number = fake.number;
    let answer = run(&mut h, &mut fake, Read::PullFiles { number, head, page: 1, skip: 0, sides: true });
    // main has this path, but the merge base has an empty tree.
    assert_eq!(
        (answer.cost, answer.result),
        (
            4,
            Ok(Answer::PullFiles {
                head,
                files: Box::new([File {
                    path: Box::from(b"file".as_slice()),
                    before: None,
                    after: Some(Box::from(b"feature only".as_slice()))
                }]),
                more: false,
                cut: false
            })
        )
    );
    assert!(skein_lib::bytes::find(&h.request_bytes(f::FORGE), b"compare/").is_some());
}

#[test]
fn whole_file_bodies_fit_the_answer_bound_and_refuse_one_byte_past_it() {
    // PullFiles pays 64 bytes for File and four bytes for its path.
    const ANSWER: u32 = 4096;
    for extra in [0, 1] {
        let content = f::filled(4028 + extra, b'x');
        let mut fake = Fake::new(&content, false);
        let mut h = harness_with(&fake, skein_forge_connection::Shared { answer_bytes: ANSWER, ..f::shared() });
        let head = object(fake.head);
        let number = fake.number;
        let answer = run(&mut h, &mut fake, Read::PullFiles { number, head, page: 1, skip: 0, sides: true });
        if extra == 1 {
            let Err(Error::TooLarge(_)) = answer.result else {
                panic!("expected a typed size refusal");
            };
        } else {
            assert_eq!(
                answer.result,
                Ok(Answer::PullFiles {
                    head,
                    files: Box::new([File { path: Box::from(b"file".as_slice()), before: None, after: Some(content) }]),
                    more: false,
                    cut: false
                })
            );
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum End {
    Loss,
    Close,
}
#[test]
fn active_file_handoffs_drop_on_connection_loss_and_component_close() {
    for end in [End::Loss, End::Close] {
        let fake = Fake::new(b"contents", false);
        let mut h = harness(&fake);
        h.call(
            1,
            f::REPO,
            Op::Read(Read::File { head: object(fake.head), path: Box::from(b"file".as_slice()), max_bytes: 8 }),
        );
        h.run(Duration::ZERO);
        h.connected(f::FORGE);
        assert_eq!(h.component.test_github_handoffs(), 1);
        match end {
            End::Loss => h.reset(f::FORGE),
            End::Close => h.send(skein_forge_connection::Request::Close),
        }
        h.run(Duration::ZERO);
        assert_eq!(h.component.test_github_handoffs(), 0, "{end:?}");
    }
}

#[test]
fn largest_github_compare_page_stays_under_the_ceiling() {
    // A full Compare page after warming the connection.
    const CEILING: u64 = 18_000;
    measure_files_page(true, CEILING);
}

#[test]
fn largest_github_pull_files_page_stays_under_the_ceiling() {
    // A full PullFiles page after warming the connection.
    const CEILING: u64 = 19_000;
    measure_files_page(false, CEILING);
}

#[expect(clippy::print_stdout, reason = "report measured call peaks for the heap ledger")]
fn measure_files_page(comparison: bool, ceiling: u64) {
    let shared = f::shared();
    let content = f::filled(256, b'x');
    let mut fake = Fake::with_files(&content, false, shared.rows);
    let mut h = harness(&fake);
    let head = object(fake.head);
    // Allocate the connection's stack and socket before counting call state.
    drop(run(&mut h, &mut fake, Read::File { head, path: Box::from(b"file0".as_slice()), max_bytes: 0 }));
    drop(h.answers.remove(&1));
    let read = if comparison {
        Read::Compare { before: object(fake.first), after: head }
    } else {
        Read::PullFiles { number: fake.number, head, page: 1, skip: 0, sides: true }
    };
    let mut call_heap = CallHeap { peak: 0, held: 0 };
    let span = skein_heap::Span::start();
    h.call(1, f::REPO, Op::Read(read));
    h.run(Duration::ZERO);
    h.connected(f::FORGE);
    call_heap.count(span.end());
    for _ in 0_u32..64 {
        let span = skein_heap::Span::start();
        if h.request_bytes(f::FORGE).is_empty() {
            h.connected(f::FORGE);
        }
        call_heap.count(span.end());
        // The fake's request processing and response allocations are its own.
        let response = fake.answer(&h.request_bytes(f::FORGE));
        let span = skein_heap::Span::start();
        h.answer(f::FORGE, &response);
        h.run(Duration::ZERO);
        call_heap.count(span.end());
        if let Some(answer) = h.answered(1) {
            let Ok(answer) = &answer.result else { unreachable!("full files page succeeds") };
            if comparison {
                let Answer::Compare { files, ahead, changed, .. } = answer else { unreachable!("Compare answer") };
                assert_eq!(*ahead, 1);
                assert_eq!(*changed, u64::from(shared.rows));
                assert_eq!(u32::try_from(files.len()).expect("row count"), shared.rows);
            } else {
                let Answer::PullFiles { files, more, .. } = answer else { unreachable!("PullFiles answer") };
                assert!(!more);
                assert_eq!(u32::try_from(files.len()).expect("row count"), shared.rows);
                for file in files {
                    assert_eq!(file.after.as_deref(), Some(content.as_ref()));
                }
            }
            assert!(
                u64::try_from(call_heap.peak).expect("positive peak") <= ceiling,
                "comparison={comparison}: {} > {ceiling}",
                call_heap.peak
            );
            println!("comparison={comparison}: peak {} bytes <= ceiling {ceiling}", call_heap.peak);
            return;
        }
    }
    unreachable!("full page settles within sixty-four exchanges")
}
