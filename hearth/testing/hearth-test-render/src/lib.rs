//! The render goldens' readers, generated repositories and scripted parent (hearth/docs/design/testing/README.md, 2.9).
pub mod golden;
pub mod json;
pub mod parent;
pub mod properties;
pub mod repositories;

pub use golden::repository_of;
use hearth_domain_render::{
    revision::{Entry, Oid},
    vocabulary::Decoded,
};

#[derive(Clone, Debug)]
pub struct Repository {
    pub entries: Vec<Entry>,
    pub blobs: Vec<(Oid, Box<[u8]>)>,
    pub documents: Vec<(Oid, Decoded)>,
}
#[must_use]
pub fn oid(byte: u8) -> Oid {
    Oid(vec![byte; 40].into_boxed_slice())
}

/// The JSON stream has one outstanding read and one terminal result.
pub const JSON_QUEUE: u32 = 1;
/// Each byte can advance both decoder directions and its bounded containers.
pub const JSON_TURNS_PER_BYTE: usize = 8;
/// Opening, end-of-stream and terminal delivery fit within thirty-two turns.
pub const JSON_TURN_GRACE: usize = 32;
