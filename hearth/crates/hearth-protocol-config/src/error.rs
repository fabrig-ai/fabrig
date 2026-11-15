//! A document's first fault, where the faulty thing starts (hearth/docs/design/domain/render/errors.md, 1).
use crate::Position;
use alloc::boxed::Box;
pub use hearth_domain_render_vocabulary::errors::{Declared, Decode, Expected, Field, Object, Required};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Error {
    pub at: Position,
    pub fault: Fault,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Fault {
    Json(skein_json::tokenizer::Error),
    Stream(skein_lib::stream::Fault),
    Duplicate { key: Box<[u8]>, first: Position },
    Decode(Decode),
}
