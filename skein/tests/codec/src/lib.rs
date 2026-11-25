//! Generated sample family for data/codec/README.md, section 4, and data/codec/golden.md, section 1.
//! Values are checked where bytes are made. They know no peer or transport.
//! Tests drive their wire entry points.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

/// The current family schema text (skein data/codec/golden.md, section 1).
pub const SAMPLE_SCHEMA: &str = include_str!("../schema/sample-v1.schema");

/// The maps fixture's record schema (data/codec/maps.md, 8).
pub const MAPPED_SCHEMA: &str = include_str!("../schema/mapped-v1.schema");

/// The maps fixture's domain mapping (data/codec/maps.md, 8).
pub const MAPPED_MAPS: &str = include_str!("../schema/mapped.maps");

/// The current family schema text (skein data/codec/golden.md, section 1).
pub const EVERY_SCHEMA: &str = include_str!("../schema/every-v3.schema");

/// The current family schema text (skein data/codec/golden.md, section 1).
pub const SCALARS_SCHEMA: &str = include_str!("../schema/scalars-v1.schema");

/// The current family schema text (skein data/codec/golden.md, section 1).
pub const LINTS_SCHEMA: &str = include_str!("../schema/lints-v1.schema");

/// The current family schema text (skein data/codec/golden.md, section 1).
pub const SECRETS_SCHEMA: &str = include_str!("../schema/secrets-v1.schema");

/// The current family schema text (skein data/codec/golden.md, section 1).
pub const GROWTH_SCHEMA: &str = include_str!("../schema/growth-v1.schema");


#[rustfmt::skip]
#[path = "generated/sample.rs"]
pub mod v1;

#[rustfmt::skip]
#[path = "scalars/v1.rs"]
pub mod scalars;

#[rustfmt::skip]
#[path = "lints/v1.rs"]
pub mod lints;

#[rustfmt::skip]
#[path = "secrets/v1.rs"]
pub mod secrets;

#[rustfmt::skip]
#[path = "every/v1.rs"]
pub mod every;

#[rustfmt::skip]
#[path = "every/v2.rs"]
pub mod every_v2;

#[rustfmt::skip]
#[path = "every/v3.rs"]
pub mod every_v3;

#[rustfmt::skip]
#[path = "growth/v1.rs"]
pub mod growth;

/// The maps fixture's owner types and by-hand pairs.
pub mod mapped_domain;

#[rustfmt::skip]
#[path = "generated/mapped.rs"]
pub mod mapped_records;

#[rustfmt::skip]
#[path = "generated/maps/mapped.rs"]
pub mod mapped_translation;
