//! A complete buffer served as the side below of a stream (lib.md, 7).

use alloc::boxed::Box;

use crate::bytes;
use crate::length::usize_of;

use super::{Read, Up};

/// A held buffer, read as a stream: the side below of the machine that
/// reads it. All of it is here, so every demand is answered at once, in
/// the same step, and what a demand cannot be met by is the stream's end.
///
/// It keeps the contract of a stream as the side below (lib.md, 7): a
/// demand is answered once, by exactly what its read asks for, or by `End`
/// when what is left can never meet it; `End` comes once, and nothing
/// after it. It asks nothing of room: the data is read, not written.
#[derive(Debug)]
pub struct Held {
    bytes: Box<[u8]>,
    /// Where what is not yet read begins.
    at: usize,
    ended: bool,
}

impl Held {
    /// The buffer `bytes`, none of it read.
    #[must_use]
    pub fn new(bytes: Box<[u8]>) -> Held {
        Held { bytes, at: 0, ended: false }
    }

    /// The answer to a demand that reads `read`: the bytes it asks for, as
    /// lib's intake would meet it, or `End` when what is left cannot. None
    /// for a withdrawal, `Read::Nothing`, and none once `End` was told.
    pub fn answer(&mut self, read: Read) -> Option<Up> {
        if self.ended {
            return None;
        }
        let rest = self.bytes.get(self.at..).expect("within the data");
        let met = match read {
            Read::Nothing => return None,
            Read::Fill(n) => {
                let n = usize_of(n);
                if n <= rest.len() { Some(n) } else { None }
            }
            Read::Arrived { max } => {
                assert!(max > 0, "a read of what has arrived has a positive maximum");
                if rest.is_empty() { None } else { Some(rest.len().min(usize_of(max))) }
            }
            Read::Scan { until, max } => {
                let max = usize_of(max);
                let window = rest.get(..max.min(rest.len())).expect("within the rest");
                match bytes::find(window, until.as_bytes()) {
                    Some(at) => Some(at.saturating_add(until.as_bytes().len())),
                    None if rest.len() >= max => Some(max),
                    None => None,
                }
            }
            Read::Line { max } => {
                let max = usize_of(max);
                let window = rest.get(..max.min(rest.len())).expect("within the rest");
                match bytes::line_end(window) {
                    Some(at) => Some(at.saturating_add(1)),
                    None if rest.len() >= max => Some(max),
                    None => None,
                }
            }
        };
        let Some(n) = met else {
            self.ended = true;
            return Some(Up::End);
        };
        let taken = bytes::copy_of(rest.get(..n).expect("what the read met"));
        self.at = self.at.saturating_add(n);
        Some(Up::Bytes(taken))
    }
}
