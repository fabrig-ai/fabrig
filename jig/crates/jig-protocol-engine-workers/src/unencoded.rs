//! A record's encoding refusal (jig/docs/design/protocol/channel/records.md, 2).
use alloc::boxed::Box;
use skein_codec::Reason;
use skein_lib::{Decimal, Writer};

/// Why a record cannot encode: what didn't fit, and the figures known.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Unencoded {
    /// A codec path's name, or the encoder's own lower-case dotted field.
    pub field: &'static str,
    /// The codec's reason, where a codec refused it.
    pub reason: Option<Reason>,
    /// The field's bytes or count, where the encoder measured it.
    pub figure: Option<u64>,
    /// The configured limit it was held to.
    pub bound: Option<u64>,
    /// The family's ceiling for it.
    pub ceiling: Option<u64>,
}

impl Unencoded {
    pub(crate) const fn field(field: &'static str) -> Self {
        Self { field, reason: None, figure: None, bound: None, ceiling: None }
    }

    /// `WHAT cannot encode: FIELD`, then what is known, with thousands' commas.
    #[must_use]
    pub fn words(&self, what: &[u8]) -> Box<[u8]> {
        let reason: &[u8] = match self.reason {
            None => b"",
            Some(Reason::Short) => b" too short",
            Some(Reason::Bound) => b" too large",
            Some(Reason::Tag) => b" bad tag",
            Some(Reason::Bool) => b" bad boolean",
            Some(Reason::Utf8) => b" not UTF-8",
            Some(Reason::Version) => b" unknown version",
            Some(Reason::Trailing) => b" trailing bytes",
            Some(Reason::Malformed) => b" malformed",
        };
        let pieces = [what, b" cannot encode: ", self.field.as_bytes(), reason];
        let figures: [(&[u8], Option<u64>); 3] =
            [(b" ", self.figure), (b", bound ", self.bound), (b", wire ceiling ", self.ceiling)];
        let mut length = 0_usize;
        for piece in pieces {
            length = length.checked_add(piece.len()).expect("diagnostic length fits");
        }
        for (label, value) in figures {
            if let Some(value) = value {
                let digits = Decimal::of(value);
                let commas = digits.as_bytes().len().saturating_sub(1).checked_div(3).expect("three is nonzero");
                length = length.checked_add(label.len()).expect("figure label length fits");
                length = length.checked_add(digits.as_bytes().len()).expect("figure digits length fits");
                length = length.checked_add(commas).expect("figure commas length fits");
            }
        }
        let mut writer = Writer::new(length);
        for piece in pieces {
            writer.put(piece).expect("measured diagnostic");
        }
        for (label, value) in figures {
            if let Some(value) = value {
                writer.put(label).expect("measured figure label");
                let digits = Decimal::of(value);
                let bytes = digits.as_bytes();
                for (index, digit) in bytes.iter().enumerate() {
                    let remaining = bytes.len().checked_sub(index).expect("digit within number");
                    if index != 0 && remaining.checked_rem(3) == Some(0) {
                        writer.put(b",").expect("measured thousands' comma");
                    }
                    writer.put(&[*digit]).expect("measured digit");
                }
            }
        }
        writer.finish()
    }
}
