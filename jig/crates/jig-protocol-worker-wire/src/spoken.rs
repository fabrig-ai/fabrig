//! The wires each side speaks (protocol/channel/wires.md, 2 and 3).
use skein_lib::List;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spoken {
    pub name: &'static str,
    pub lowest: u16,
    pub layouts: List<[u8; 32]>,
}

impl Spoken {
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
        Spoken::current(crate::wire::FAMILY, crate::wire::VERSION, crate::wire::layout()),
        Spoken::current(smith_channel::wire::FAMILY, smith_channel::wire::VERSION, smith_channel::wire::layout()),
        Spoken::current(smith_charter::wire::FAMILY, smith_charter::wire::VERSION, smith_charter::wire::layout()),
        Spoken::current(smith_transcript::wire::FAMILY, smith_transcript::wire::VERSION, smith_transcript::wire::layout()),
    ] {
        wires.push(wire).expect("four wires have four slots");
    }
    wires
}

#[cfg(test)]
mod tests {
    use super::Spoken;

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
