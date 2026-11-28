//! Moves keyed by FIELDS' shapes and members (temper/docs/design/protocol/configuration/file-formats.md, 3).
use super::{Shape, form};
use crate::Text;

#[derive(Clone, Copy)]
pub(crate) struct Formats {
    pub newest: u64,
    /// In increasing order of the format that made the move.
    pub moves: &'static [Move],
}

pub(crate) const FORMATS: Formats = Formats { newest: 1, moves: &[] };

pub(crate) struct Move {
    pub format: u64,
    pub shape: Shape,
    pub member: &'static [u8],
    pub to: Destination,
}

#[cfg_attr(not(test), expect(dead_code, reason = "settings format 1 has no production moves"))]
pub(crate) enum Destination {
    Renamed { member: &'static [u8] },
    Converted { member: &'static [u8], conversion: Conversion, words: &'static [u8] },
    MovedOut { words: &'static [u8] },
    Gone { release: &'static [u8] },
}

#[derive(Clone, Copy)]
pub(crate) enum Conversion {
    #[cfg(test)]
    LegacySwitch,
}

impl Formats {
    pub fn write(self, out: &mut Text) {
        out.put(b"\n## Formats\n\nNewest format `temper ctl` reads: ");
        out.number(self.newest);
        out.put(b"; no `format`: 1.\n");
        for movement in self.moves {
            out.put(b"format ");
            out.number(movement.format);
            out.put(b": ");
            out.put(form(movement.shape).name);
            out.put(b".");
            out.put(movement.member);
            out.put(b": ");
            match movement.to {
                Destination::Renamed { member } => {
                    out.put(b"renamed ");
                    out.put(member);
                }
                Destination::Converted { member, words, .. } => {
                    out.put(b"converted to ");
                    out.put(member);
                    out.put(b": ");
                    out.put(words);
                }
                Destination::MovedOut { words } => {
                    out.put(b"moved out: ");
                    out.put(words);
                }
                Destination::Gone { release } => {
                    out.put(b"gone in ");
                    out.put(release);
                }
            }
            out.put(b"\n");
        }
    }
}
