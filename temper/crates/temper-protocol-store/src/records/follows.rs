//! Follow records (temper/docs/design/protocol/store.md, 2, family 42).
use super::{Bounds, FITS, Undecodable, names, w};
use alloc::boxed::Box;
use skein_codec::Record;
use skein_lib::{List, Reader};
use temper_engine_domain_forge_vocabulary as d;
pub(super) fn encode(row: &d::FollowRow) -> w::FollowRow {
    crate::generated::store::follow_row_to_record(row, &super::Context::UNBOUNDED).expect(FITS)
}
pub(super) fn decode(row: w::FollowRow, bounds: &Bounds) -> Result<d::FollowRow, Undecodable> {
    let context = super::Context {
        follow_files: bounds.forge().follow_files,
        answer_name_bytes: bounds.limits().answer_name_name,
        ..super::Context::UNBOUNDED
    };
    crate::generated::store::follow_row_to_domain(row, &context).or(Err(Undecodable::Malformed))
}
#[expect(clippy::borrowed_box, reason = "maps pairs keep the prescribed borrowed, fallible signature")]
#[expect(clippy::unnecessary_wraps, reason = "maps pairs keep the prescribed borrowed, fallible signature")]
pub(crate) fn follow_files_to_record(
    value: &Box<[Box<[u8]>]>,
    _context: &super::Context,
) -> Result<w::FollowFiles, w::Problem> {
    Ok(w::FollowFiles { files: names::encode_piece(value) })
}

pub(crate) fn follow_files_to_domain(
    value: w::FollowFiles,
    context: &super::Context,
) -> Result<Box<[Box<[u8]>]>, w::Problem> {
    let mut limits = w::CEILINGS;
    limits.answer_name_name = context.answer_name_bytes;
    let mut reader = Reader::new(&value.files);
    let mut files = List::with_capacity(context.follow_files);
    for _ in 0..=context.follow_files {
        let Some(item) = skein_codec::piece::split(&mut reader)
            .or(Err(w::paths::FOLLOW_FILES_FILES.problem(skein_codec::Reason::Malformed)))?
        else {
            return Ok(files.into_boxed());
        };
        let file = w::AnswerName::decode(&limits, &mut Reader::new(&item))?;
        files.push(file.name).or(Err(w::paths::FOLLOW_FILES_FILES.problem(skein_codec::Reason::Bound)))?;
    }
    Err(w::paths::FOLLOW_FILES_FILES.problem(skein_codec::Reason::Bound))
}
