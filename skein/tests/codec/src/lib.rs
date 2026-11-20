//! Generated sample family for data/codec/README.md, sections 4 and 6.
//! The types hold bounded fields. They know no peer or transport. Tests drive
//! their constructors and, later, wire entry points.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

/// The current family schema text (skein data/codec/README.md, section 6).
pub const SAMPLE_SCHEMA: &str = include_str!("../schema/sample-v1.schema");

/// The current family schema text (skein data/codec/README.md, section 6).
pub const EVERY_SCHEMA: &str = include_str!("../schema/every-v3.schema");

/// The current family schema text (skein data/codec/README.md, section 6).
pub const SCALARS_SCHEMA: &str = include_str!("../schema/scalars-v1.schema");

/// The current family schema text (skein data/codec/README.md, section 6).
pub const LINTS_SCHEMA: &str = include_str!("../schema/lints-v1.schema");

/// The current family schema text (skein data/codec/README.md, section 6).
pub const SECRETS_SCHEMA: &str = include_str!("../schema/secrets-v1.schema");

/// The current family schema text (skein data/codec/README.md, section 6).
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
#[expect(clippy::option_option, reason = "the pin exercises all three nested-option wire states")]
pub mod every_v3;

#[rustfmt::skip]
#[path = "every/v3_lean.rs"]
pub mod every_v3_lean;

#[rustfmt::skip]
#[path = "growth/v1.rs"]
pub mod growth;

#[rustfmt::skip]
#[path = "growth/v1_lean.rs"]
pub mod growth_lean;

#[path = "../patches/mod.rs"]
pub mod patches;
