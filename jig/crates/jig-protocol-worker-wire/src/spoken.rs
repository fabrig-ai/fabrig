//! The wires each side speaks (protocol/channel/wires.md, 2 and 3).
use skein_lib::List;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spoken {
    pub name: &'static str,
    pub lowest: u16,
    pub layouts: List<[u8; 32]>,
}

impl Spoken {
    /// Fingerprints from the family's floor (protocol/channel/bridges.md, 2).
    pub fn of(name: &'static str, texts: &[&str], floor: u16) -> Result<Spoken, skein_codec::bridge::BridgeError> {
        Ok(Spoken { name, lowest: floor, layouts: skein_codec::bridge::layouts(texts, floor)? })
    }

    #[must_use]
    pub fn current(name: &'static str, version: u16, layout: [u8; 32]) -> Spoken {
        let mut layouts = List::with_capacity(1);
        layouts.push(layout).expect("one layout slot");
        Spoken { name, lowest: version, layouts }
    }

    #[must_use]
    pub fn highest(&self) -> u16 {
        let last = self.layouts.len().checked_sub(1).expect("a spoken range has at least one layout");
        self.lowest.checked_add(u16::try_from(last).expect("a spoken range fits u16")).expect("a spoken range fits u16")
    }

    #[must_use]
    pub fn layout(&self, version: u16) -> Option<[u8; 32]> {
        self.layouts.get(u32::from(version.checked_sub(self.lowest)?)).copied()
    }
}

/// A wire's agreed version, named for the owner's log.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Agreed {
    pub name: &'static str,
    pub version: u16,
}

/// Jig's wires, then its dependencies' wires, in the statement's order.
#[must_use]
pub fn spoken() -> List<Spoken> {
    let mut wires = List::with_capacity(4);
    for wire in [
        Spoken::of(crate::wire::FAMILY, crate::patches::kept::jig_worker_wire::TEXTS, crate::wire::FLOOR)
            .expect("jig's kept wire"),
        Spoken::of(
            smith_channel::pieces::wire::FAMILY,
            smith_channel::patches::kept::smith_pieces::TEXTS,
            smith_channel::pieces::wire::FLOOR,
        )
        .expect("smith's kept pieces"),
        Spoken::of(
            smith_charter::wire::FAMILY,
            smith_charter::patches::kept::smith_charter::TEXTS,
            smith_charter::wire::FLOOR,
        )
        .expect("smith's kept charter"),
        Spoken::of(
            smith_transcript::wire::FAMILY,
            smith_transcript::patches::kept::smith_transcript::TEXTS,
            smith_transcript::wire::FLOOR,
        )
        .expect("smith's kept transcript"),
    ] {
        wires.push(wire).expect("four wires have four slots");
    }
    wires
}

#[cfg(test)]
mod tests {
    use super::Spoken;

    #[test]
    fn the_opening_states_jigs_wires_then_smiths_kept_families() {
        let wires = super::spoken();
        assert_eq!(wires.len(), 4);
        for (wire, name) in wires.iter().zip(["jig-worker-wire", "smith-pieces", "smith-charter", "smith-transcript"]) {
            assert_eq!(wire.name, name);
        }
        let pieces = wires.get(1).expect("the kept pieces follow jig's wire");
        assert_eq!(pieces.lowest, smith_channel::pieces::wire::VERSION);
        assert_eq!(pieces.highest(), smith_channel::pieces::wire::VERSION);
        assert_eq!(pieces.layout(pieces.lowest), Some(smith_channel::pieces::wire::layout()));
    }

    #[test]
    fn kept_wires_state_every_layout_from_their_floor() {
        let wires = super::spoken();
        for (wire, texts, floor) in [
            (wires.get(0).expect("jig"), crate::patches::kept::jig_worker_wire::TEXTS, crate::wire::FLOOR),
            (
                wires.get(1).expect("pieces"),
                smith_channel::patches::kept::smith_pieces::TEXTS,
                smith_channel::pieces::wire::FLOOR,
            ),
            (
                wires.get(2).expect("charter"),
                smith_charter::patches::kept::smith_charter::TEXTS,
                smith_charter::wire::FLOOR,
            ),
            (
                wires.get(3).expect("transcript"),
                smith_transcript::patches::kept::smith_transcript::TEXTS,
                smith_transcript::wire::FLOOR,
            ),
        ] {
            assert_eq!(wire.lowest, floor);
            let mut count = 0;
            for text in texts {
                let schema = skein_codec::schema::parse(text.as_bytes()).expect("kept text");
                if schema.version() >= floor {
                    count += 1;
                    assert_eq!(wire.layout(schema.version()), Some(schema.fingerprints().family().digest()));
                }
            }
            assert_eq!(wire.layouts.len(), count);
        }
        let jig = wires.get(0).expect("jig");
        let rebuilt =
            Spoken::of(jig.name, crate::patches::kept::jig_worker_wire::TEXTS, crate::VERSION).expect("raised floor");
        assert_eq!(rebuilt.lowest, crate::VERSION);
        assert_eq!(rebuilt.layouts.len(), 1);
        assert_eq!(rebuilt.layout(crate::VERSION), jig.layout(crate::VERSION));
    }

    #[test]
    fn a_spoken_range_names_each_versions_layout() {
        let mut spoken = Spoken::current("example", 7, [1; 32]);
        assert_eq!(spoken.highest(), 7);
        assert_eq!(spoken.layout(7), Some([1; 32]));
        let mut layouts = skein_lib::List::with_capacity(2);
        layouts.push([1; 32]).expect("first version");
        layouts.push([2; 32]).expect("next version");
        spoken.layouts = layouts;
        assert_eq!(spoken.highest(), 8);
        assert_eq!(spoken.layout(6), None);
        assert_eq!(spoken.layout(8), Some([2; 32]));
        assert_eq!(spoken.layout(9), None);
    }
}
