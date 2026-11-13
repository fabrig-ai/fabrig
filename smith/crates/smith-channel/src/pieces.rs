//! The channel pieces a host keeps across builds, laid out as the channel's
//! declarations (smith/docs/design/protocol/README.md, 8). No separate codec or
//! runtime state is kept: readers and writers use the channel's record types.

/// The kept pieces' family, version, floor and layout.
pub mod wire {
    /// The family named by the current schema's header.
    pub const FAMILY: &str = "smith-pieces";

    /// The version named by the current schema's header.
    pub const VERSION: u16 = 1;

    /// The lowest version this build speaks.
    pub const FLOOR: u16 = crate::patches::kept::smith_pieces::FLOOR;

    #[expect(clippy::disallowed_macros, reason = "the wire identity embeds its current schema")]
    const TEXT: &str = include_str!("../schema/pieces-v1.schema");

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
        use skein_codec::schema::{DeclIx, parse};

        #[test]
        fn identity_tracks_schema() {
            let schema = parse(TEXT.as_bytes()).expect("pieces schema");
            assert_eq!(FAMILY, "smith-pieces");
            assert_eq!(FAMILY.as_bytes(), schema.spelled(schema.family()));
            assert_eq!(VERSION, schema.version());
            assert_eq!(VERSION, 1);
            assert_eq!(FLOOR, 1);
            assert_eq!(FLOOR, crate::patches::kept::smith_pieces::FLOOR);
            assert_eq!(layout(), schema.fingerprints().family().digest());
        }

        #[test]
        fn every_piece_has_the_channels_layout() {
            let pieces = parse(TEXT.as_bytes()).expect("pieces schema");
            let channel = parse(crate::wire::TEXT.as_bytes()).expect("channel schema");
            let pieces_layouts = pieces.fingerprints();
            let channel_layouts = channel.fingerprints();
            for index in 0..pieces.declarations() {
                let piece = DeclIx::new(index);
                let name = pieces.spelled(pieces.declaration(piece).name);
                let original = channel.find(name).expect("each piece is a channel declaration");
                assert_eq!(pieces_layouts.declaration(piece), channel_layouts.declaration(original), "{name:?}");
            }
        }
    }
}
