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
fn pull_file_status_spellings_decode_into_typed_statuses() {
    skein_rows::check(
        [
            ("pull_files_decode_forgejos_changed_status_as_modified", (br#"[{"filename":"notes.md","status":"changed","additions":2,"deletions":1,"changes":3,"previous_filename":"","html_url":"http://127.0.0.1:3000/fixture/specimen/src/commit/head/notes.md","contents_url":"http://127.0.0.1:3000/api/v1/repos/fixture/specimen/contents/notes.md","raw_url":"http://127.0.0.1:3000/fixture/specimen/raw/commit/head/notes.md"}]"#.as_slice(), ChangedFile { filename: Box::from(*b"notes.md"), previous_filename: Some(Box::from([])), status: FileStatus::Modified })),
            ("pull_files_decode_removed_and_deleted_as_deletions: removed", (br#"[{"filename":"notes.md","status":"removed"}]"#.as_slice(), ChangedFile { filename: Box::from(*b"notes.md"), previous_filename: None, status: FileStatus::Deleted })),
            ("pull_files_decode_removed_and_deleted_as_deletions: deleted", (br#"[{"filename":"notes.md","status":"deleted"}]"#.as_slice(), ChangedFile { filename: Box::from(*b"notes.md"), previous_filename: None, status: FileStatus::Deleted })),
        ],
        |(body, file)| { let limits = fixtures::limits(); let request = Request::PullFiles { at: fixtures::repository(), number: 7, page: Page { page: Some(1), limit: Some(1) } }; let decoded = fixtures::decode(client::decoder(&request, &limits).expect("file decoder"), body, &limits); assert_eq!(decoded.document, Ok(Document::Listed { skipped: 0 }), "{body:?}"); assert_eq!(decoded.rows.as_slice(), &[Row::File(file)], "{body:?}"); },
    );
}
