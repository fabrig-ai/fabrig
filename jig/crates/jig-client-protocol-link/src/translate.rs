//! Exhaustive owner translations (protocol/client/wire.md, 1, and protocol/client/README.md, 3).
pub mod answers;
pub mod asks;
pub mod documents;
mod facts;
pub mod live;
mod llm;
mod model_sets;
mod operations;
pub(crate) mod programs;
mod task_wait;
use crate::Limits;
use alloc::boxed::Box;
use jig_client_protocol_wire::v21 as w;
use skein_lib::Reader;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Undecodable {
    Malformed,
    TooLarge,
    Version,
}
pub(crate) fn too_large() -> w::Problem {
    w::Problem { path: w::Path::OwnedRecord, reason: skein_codec::Reason::Bound }
}
pub(crate) fn text_problem() -> w::Problem {
    too_large()
}
pub(crate) fn piece_problem(reason: skein_codec::Reason) -> w::Problem {
    w::Problem { path: w::Path::OwnedRecord, reason }
}
/// Splits framing while retaining its codec reason (skein/docs/design/data/codec/README.md, 3).
pub(crate) fn piece(reader: &mut Reader<'_>) -> Result<Option<Box<[u8]>>, w::Problem> {
    match skein_codec::piece::split(reader) {
        Ok(value) => Ok(value),
        Err(reason) => Err(piece_problem(reason)),
    }
}
/// Splits and charges one row before its owner decodes it (skein/docs/design/data/codec/README.md, 3).
pub(crate) fn next_piece(
    reader: &mut Reader<'_>,
    sum: &mut u64,
    limits: &Limits,
) -> Result<Option<Box<[u8]>>, w::Problem> {
    match skein_codec::piece::split_within(reader, sum, limits.client().document_bytes) {
        Ok(value) => Ok(value),
        Err(reason) => Err(piece_problem(reason)),
    }
}
pub mod objects;
pub mod pages;

pub mod reads;
pub(crate) mod turn;

pub mod watches;

pub(crate) mod envelope;
pub mod store;

mod compound;

mod queries;

mod account_hosts;

pub mod settings;
pub mod settings_proposals;

pub(crate) fn classify(problem: w::Problem) -> Undecodable {
    match problem.reason {
        skein_codec::Reason::Bound => Undecodable::TooLarge,
        skein_codec::Reason::Malformed
        | skein_codec::Reason::Tag
        | skein_codec::Reason::Utf8
        | skein_codec::Reason::Short
        | skein_codec::Reason::Bool
        | skein_codec::Reason::Version
        | skein_codec::Reason::Trailing => Undecodable::Malformed,
    }
}
