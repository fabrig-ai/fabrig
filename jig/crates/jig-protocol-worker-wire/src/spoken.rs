//! The current texts each side speaks (jig/docs/design/protocol/channel/layouts.md, 7).
use skein_lib::List;

/// One wire at its frozen version and current layout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spoken {
    pub name: &'static str,
    pub version: u16,
    pub layout: [u8; 32],
    pub text: &'static str,
}

/// Jig's wires, then its dependencies' wires, in the statement's order.
#[must_use]
pub fn spoken() -> List<Spoken> {
    use smith_channel::pieces;
    let mut wires = List::with_capacity(4);
    for wire in [
        Spoken {
            name: crate::wire::FAMILY,
            version: crate::VERSION,
            layout: crate::wire::layout(),
            text: crate::wire::TEXT,
        },
        Spoken {
            name: pieces::wire::FAMILY,
            version: pieces::wire::VERSION,
            layout: pieces::wire::layout(),
            text: pieces::wire::TEXT,
        },
        Spoken {
            name: smith_charter::wire::FAMILY,
            version: smith_charter::wire::VERSION,
            layout: smith_charter::wire::layout(),
            text: smith_charter::wire::TEXT,
        },
        Spoken {
            name: smith_transcript::wire::FAMILY,
            version: smith_transcript::wire::VERSION,
            layout: smith_transcript::wire::layout(),
            text: smith_transcript::wire::TEXT,
        },
    ] {
        wires.push(wire).expect("one slot per wire");
    }
    wires
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_opening_speaks_jigs_four_current_texts_in_order() {
        let wires = super::spoken();
        let expected = [crate::SCHEMA, smith_channel::PIECES_SCHEMA, smith_charter::SCHEMA, smith_transcript::SCHEMA];
        assert_eq!(wires.len(), 4);
        for (wire, text) in wires.iter().zip(expected) {
            let schema = skein_codec::schema::parse(text.as_bytes()).expect("current text");
            assert_eq!(wire.text, text);
            assert_eq!(wire.name.as_bytes(), schema.spelled(schema.family()));
            assert_eq!(wire.version, schema.version());
            assert_eq!(wire.layout, schema.fingerprints().family().digest());
        }
    }
}
