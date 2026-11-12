//! The last scan (temper/docs/design/client/web/journeys.md, 3.7–3.8).
use crate::Originals;
use skein_lib::bytes::find;
mod shapes;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Key,
    Jwt,
    Bearer,
    Code,
    RefreshToken,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum What {
    Original,
    Shape(Shape),
}

/// Seek originals, their digests, and the shapes of client/web/journeys.md, 3.7.
#[must_use]
pub fn bytes(bytes: &[u8], originals: &Originals) -> Option<What> {
    if original(bytes, originals) { Some(What::Original) } else { shape(bytes).map(What::Shape) }
}
/// Seek every form and digest of a sought original.
#[must_use]
pub fn original(bytes: &[u8], originals: &Originals) -> bool {
    for original in originals.sought() {
        if original.forms().iter().any(|form| find(bytes, form).is_some())
            || original.digests().iter().any(|(digest, _)| find(bytes, digest).is_some())
        {
            return true;
        }
    }
    false
}
/// Seek the shapes of client/web/journeys.md, 3.7.
#[must_use]
pub fn shape(bytes: &[u8]) -> Option<Shape> {
    shapes::scan(bytes)
}

#[cfg(test)]
mod tests;
