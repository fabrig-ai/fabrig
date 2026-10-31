//! Tiny event codec entrances for deterministic tests (skein/docs/foundation/testing-strategy.md, section 1).
use crate::Limits;

/// Metadata, captured content and list limits for focused record tests.
pub const SMALL: Limits = Limits { string: 64, content: 1024, items: 4 };
