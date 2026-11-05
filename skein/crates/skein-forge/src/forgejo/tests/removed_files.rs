//! Forgejo's removed file spelling (skein/docs/design/forge.md, 3.3 and 5.2).
use super::fixtures;
use crate::{
    api::{Op, Read},
    forgejo::{
        client,
        document::{ChangedFile, Document, FileStatus, Row},
        id::{ObjectFormat, ObjectId},
        plans::plan::{Bounds, Step},
        request::{Page, Request},
    },
};
use alloc::boxed::Box;

// A captured comparison: 13 commits, 50 file rows, 16 paths, two "removed" rows.
// Names, paths, object ids and addresses are the neutral fixtures'; text keeps its byte length.
#[expect(clippy::disallowed_macros, reason = "immutable neutralized Forgejo answer for the decoder regression")]
const COMPARE: &[u8] = include_bytes!("fixtures/compare-removed.json");
const PATHS: u32 = 16; // distinct paths in COMPARE, after repeated file rows are folded

#[test]
fn captured_compare_with_removed_files_advances_to_reverse_read() {
    let mut run = fixtures::Run::with_limits(
        fixtures::profile_limits(),
        Bounds { rows: PATHS, ..fixtures::BOUNDS },
        Op::Read(Read::Compare { before: fixtures::BASE, after: fixtures::HEAD }),
    );
    run.begin();
    assert_eq!(
        run.answer(200, COMPARE),
        Step::send(Request::Compare {
            at: fixtures::repository(),
            before: ObjectId { bytes: fixtures::HEAD, format: ObjectFormat::Sha1 },
            after: ObjectId { bytes: fixtures::BASE, format: ObjectFormat::Sha1 },
            files: false,
        })
    );
}

#[test]
fn pull_files_decode_removed_and_deleted_as_deletions() {
    let limits = fixtures::limits();
    let request =
        Request::PullFiles { at: fixtures::repository(), number: 7, page: Page { page: Some(1), limit: Some(1) } };
    for body in [
        br#"[{"filename":"notes.md","status":"removed"}]"#.as_slice(),
        br#"[{"filename":"notes.md","status":"deleted"}]"#,
    ] {
        let decoded = fixtures::decode(client::decoder(&request, &limits).expect("file decoder"), body, &limits);
        assert_eq!(decoded.document, Ok(Document::Listed { skipped: 0 }), "{body:?}");
        assert_eq!(
            decoded.rows.as_slice(),
            &[Row::File(ChangedFile {
                filename: Box::from(*b"notes.md"),
                previous_filename: None,
                status: FileStatus::Deleted,
            })],
            "{body:?}"
        );
    }
}
