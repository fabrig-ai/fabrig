//! Changed files without line changes (skein/docs/design/forge/forgejo.md, 1.3).
use crate::api::{Answer, File, Op, Read};
use crate::forgejo::{
    plans::plan::{Bounds, Step},
    tests::fixtures::{BOUNDS, HEAD, PULL, Run},
};

// An added text file, a binary wire golden without line changes, and a deletion.
const FILES: &[u8] = br#"[{"filename":"new.txt","status":"added","additions":1,"deletions":0},{"filename":"wire.bin","status":"unchanged","additions":0,"deletions":0},{"filename":"old.txt","status":"deleted","additions":0,"deletions":1}]"#;
const PAGE_SIZE: u32 = 3; // All three FILES rows in one full page.
const BEFORE: &[u8] = b"\0\xff\x01\0";
const AFTER: &[u8] = b"\0\xff\x02\0";
const PULL_GET: &str = "GET /api/v1/repos/fixture/specimen/pulls/1";
const FILES_GET: &str = "GET /api/v1/repos/fixture/specimen/pulls/1/files?page=1&limit=3";

fn listed(sides: bool) -> (Run, Step) {
    let mut run = Run::with(
        Bounds { page_size: PAGE_SIZE, ..BOUNDS },
        Op::Read(Read::PullFiles { number: 1, head: HEAD, page: 1, skip: 0, sides }),
    );
    run.begin();
    run.answer(200, PULL);
    let step = run.answer(200, FILES);
    (run, step)
}

#[test]
fn unchanged_binary_reads_both_sides_without_altering_bytes() {
    let (mut run, mut step) = listed(true);
    for body in [b"new\n".as_slice(), BEFORE, AFTER, b"old\n"] {
        if let Step::Done(_) = step {
            break;
        }
        step = run.answer(200, body);
    }
    assert_eq!(
        step,
        Step::Done(Ok(Answer::PullFiles {
            head: HEAD,
            files: Box::new([
                File { path: Box::from(*b"new.txt"), before: None, after: Some(Box::from(*b"new\n")) },
                File { path: Box::from(*b"wire.bin"), before: Some(Box::from(BEFORE)), after: Some(Box::from(AFTER)) },
                File { path: Box::from(*b"old.txt"), before: Some(Box::from(*b"old\n")), after: None },
            ]),
            more: true,
            cut: false,
        }))
    );
    run.assert_sent(&[
        PULL_GET,
        FILES_GET,
        "GET /api/v1/repos/fixture/specimen/raw/new.txt?ref=0100000000000000000000000000000000000000",
        "GET /api/v1/repos/fixture/specimen/raw/wire.bin?ref=0300000000000000000000000000000000000000",
        "GET /api/v1/repos/fixture/specimen/raw/wire.bin?ref=0100000000000000000000000000000000000000",
        "GET /api/v1/repos/fixture/specimen/raw/old.txt?ref=0300000000000000000000000000000000000000",
    ]);
}

#[test]
fn unchanged_binary_lists_its_path_without_reading_sides() {
    let (run, step) = listed(false);
    assert_eq!(
        step,
        Step::Done(Ok(Answer::PullFiles {
            head: HEAD,
            files: Box::new([
                File { path: Box::from(*b"new.txt"), before: None, after: None },
                File { path: Box::from(*b"wire.bin"), before: None, after: None },
                File { path: Box::from(*b"old.txt"), before: None, after: None },
            ]),
            more: true,
            cut: false,
        }))
    );
    run.assert_sent(&[PULL_GET, FILES_GET]);
}
