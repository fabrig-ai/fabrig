//! The document's shape (protocol/services.md, 3).
use jig_protocol_config::{self as config, read};

/// The services selected by the document's members (protocol/cluster.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shape {
    Standalone,
    EngineAlone,
    WorkerAlone,
}
fn member(document: &skein_json::Document, at: u32, name: &[u8]) -> Option<u32> {
    read::Object::new(document, at).ok()?.member(document, name)
}
/// The nearest shape of a parsed document (protocol/cluster.md, 1).
#[must_use]
pub fn nearest_document(document: &skein_json::Document) -> Shape {
    let Some(engine) = member(document, 0, b"engine") else { return Shape::WorkerAlone };
    let Some(hosts) = member(document, engine, b"hosts") else { return Shape::Standalone };
    let Some(at) = member(document, hosts, b"listener") else { return Shape::Standalone };
    let Ok(listeners) = read::list(document, at, document.len()) else { return Shape::Standalone };
    let Some(first) = listeners.get(0) else { return Shape::Standalone };
    if member(document, *first, b"socket").is_some() { Shape::EngineAlone } else { Shape::Standalone }
}
/// The document's nearest shape (protocol/cluster.md, 1), read from its
/// members alone; `None` when the bytes are no JSON document whose root is an object.
#[must_use]
pub fn nearest(input: &[u8]) -> Option<Shape> {
    let length = u32::try_from(input.len()).ok()?;
    let document =
        config::document(input, &config::DocumentLimits { bytes: 1_048_576, tokens: length, depth: 32 }).ok()?;
    read::Object::new(&document, 0).ok()?;
    Some(nearest_document(&document))
}
