//! Shared journey redaction and last scan (temper/docs/design/client/web/journey-redaction.md and temper/docs/design/client/web/journeys-recordings.md, 1.7).
mod originals;
mod redact;
pub mod scan;

pub use originals::{Kind, Original, Originals, placeholder};
pub use redact::{Framing, redact_field};
pub use scan::{Shape, What, bytes, original, shape};
