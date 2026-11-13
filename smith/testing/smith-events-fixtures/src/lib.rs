//! Shared fixtures (smith/docs/design/testing.md, section 7; smith/docs/design/protocol/events.md, section 1).
//! Tiny event codec entrances for deterministic tests (skein/docs/foundation/testing-strategy.md, section 1).
#![no_std]

use smith_events::Limits;

/// Metadata, captured content and list limits for focused record tests.
pub const SMALL: Limits = Limits { string: 64, content: 1024, items: 4 };
