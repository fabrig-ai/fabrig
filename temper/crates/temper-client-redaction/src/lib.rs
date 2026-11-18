//! Shared journey redaction and last scan (temper/docs/design/client/web/journey-redaction.md and temper/docs/design/client/web/journeys.md, 3.7).
mod originals;
mod redact;
pub mod scan;
mod varying;

pub use originals::{Kind, Original, Originals, placeholder};
pub use redact::{Framing, redact_field};
pub use scan::{Shape, What, bytes, original, shape};
pub use varying::{Normalised, Varied, Varying};
