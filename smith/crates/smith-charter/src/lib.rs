//! Bounded charter and result records; this crate retains no runtime state and
//! knows no domain, endpoint configuration or host policy. Records,
//! encoders and decoders are generated from `smith/docs/design/protocol/charter.md`, sections 2-6.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

/// The current family schema text (skein/docs/design/data/codec/README.md, section 6).
#[expect(clippy::disallowed_macros, reason = "the owner embeds its current schema text once")]
pub const SCHEMA: &str = include_str!("../schema/charter-v5.schema");

use alloc::boxed::Box;

#[rustfmt::skip]
mod generated {
    pub mod v5 {
        include!("generated/charter.rs");
    }
}

pub use generated::v5;

pub use v5::{
    Budget, CEILINGS, ChangeRule, Charter, Contract, Conventions, Effect, Families, Field, FieldRule, Form, HostTool,
    Item, ItemKind, Llm, Mode, Prices, Record, Reserve, RunResult, Section, TextRule, Tools, VerdictRule,
};

impl Charter {
    /// The wire format version this codec writes and reads.
    pub const FORMAT_VERSION: u64 = {
        let [high, low] = wire::VERSION.to_be_bytes();
        u64::from_be_bytes([0, 0, 0, 0, 0, 0, high, low])
    };
}

/// Provider effort name, bounded to 32 UTF-8 bytes by the charter codec.
pub type Effort = Box<[u8]>;

/// This crate's spoken family and layout (temper/docs/design/protocol/wires.md, 3.1).
pub mod wire {
    /// The family named by the current schema's header.
    pub const FAMILY: &str = "smith-charter";

    /// The version named by the current schema's header.
    pub const VERSION: u16 = crate::v5::VERSION;

    /// The lowest version this build speaks.
    pub const FLOOR: u16 = crate::patches::kept::smith_charter::FLOOR;

    const TEXT: &str = crate::SCHEMA;

    /// The current schema's family fingerprint.
    #[must_use]
    pub fn layout() -> [u8; 32] {
        skein_codec::schema::parse(TEXT.as_bytes())
            .expect("the embedded current schema is valid")
            .fingerprints()
            .family()
            .digest()
    }

    #[cfg(test)]
    mod tests {
        use super::{FAMILY, FLOOR, TEXT, VERSION, layout};

        #[test]
        fn host_tool_input_path_names_the_schema_field() {
            let name = crate::v5::name(crate::v5::paths::HOST_TOOL_INPUT).expect("host tool input path");
            assert_eq!(name, "HostTool.input");
        }

        #[test]
        #[expect(clippy::disallowed_macros, reason = "the patch test embeds the kept version-four schema")]
        fn a_version_four_charter_takes_up_contract_one() {
            use crate::Record;
            use skein_codec::patch::Capacity;
            use skein_lib::Reader;

            let capacity = Capacity { versions: 2, patches: 2, text: 8192, bytes: 8192, nodes: 256 };
            let mut bridge = crate::patches::kept::smith_charter::bridge(capacity).expect("charter bridge");
            let schema =
                skein_codegen::parse(include_str!("../schema/kept/smith-charter-v4.schema")).expect("kept v4 schema");
            let mut old = None;
            for value in skein_codegen::goldens(&schema) {
                if value.name == "record_charter_smallest.bin" {
                    old = Some(value.bytes);
                }
            }
            let old = old.expect("v4 charter");
            let current = bridge.translate(4, VERSION, b"Charter", old.as_slice().into()).expect("v4 taken up");
            let charter = crate::Charter::decode(&crate::CEILINGS, &mut Reader::new(&current)).expect("v5 charter");
            assert_eq!(charter.tools.contract, 1);
        }

        #[test]
        fn identity_tracks_schema() {
            let schema = skein_codec::schema::parse(TEXT.as_bytes()).expect("current schema");
            assert_eq!(FAMILY.as_bytes(), schema.spelled(schema.family()));
            assert_eq!(VERSION, schema.version());
            assert_eq!(FLOOR, crate::patches::kept::smith_charter::FLOOR);
            assert_eq!(FLOOR, 4);
            assert_eq!(layout(), schema.fingerprints().family().digest());

            let changed = TEXT.replacen("max: u32", "max: u64", 1);
            assert_ne!(changed, TEXT, "the mutation changes a field's type");
            let changed = skein_codec::schema::parse(changed.as_bytes()).expect("changed schema");
            assert_ne!(layout(), changed.fingerprints().family().digest());
        }
    }
}

#[path = "../patches/mod.rs"]
pub mod patches;
