//! Sized writing (programming-model.md, 6.2 and section 8): bytes built into a
//! box of a length computed first.

#![expect(clippy::disallowed_types, reason = "a writer fills a Vec allocated once, at its final length")]

use alloc::boxed::Box;
use alloc::vec::Vec;

/// Builds a `Box<[u8]>` of a length its caller computed first, from slices
/// put one after another: an encoded message, a prompt rendered from
/// fragments, a file with one span replaced.
///
/// The box is allocated once, at that length, and never grows; its bytes are
/// the caller's to count, as any payload's. A put that does not fit what is
/// left is refused whole and writes nothing. The length is the caller's own
/// (programming-model.md, section 8): a caller that computed it expects every
/// put to fit, and finishing short is a bug, which [`Writer::finish`] asserts.
#[derive(Debug)]
pub struct Writer {
    output: Output,
    len: usize,
}

#[derive(Debug)]
enum Output {
    Measure(usize),
    Write(Vec<u8>),
}

/// What a [`Writer`] or an [`Intake`](crate::Intake) refuses: bytes past the
/// length or the cap it was made for. Nothing was written.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Overflow;

impl Writer {
    /// A writer of exactly `len` bytes.
    #[must_use]
    pub fn new(len: usize) -> Writer {
        Writer { output: Output::Write(Vec::with_capacity(len)), len }
    }

    /// Count an encoding up to `limit` without allocating its output. Use
    /// [`Writer::written`] for the exact length, then repeat with [`Writer::new`].
    /// A measuring writer cannot be finished.
    #[must_use]
    pub const fn measure(limit: usize) -> Writer {
        Writer { output: Output::Measure(0), len: limit }
    }

    /// The bytes put so far.
    #[must_use]
    pub fn written(&self) -> usize {
        match &self.output {
            Output::Measure(written) => *written,
            Output::Write(bytes) => bytes.len(),
        }
    }

    /// How many more bytes fit.
    #[must_use]
    pub fn room(&self) -> usize {
        self.len.checked_sub(self.written()).expect("a writer never passes its length")
    }

    /// Appends `bytes`, or refuses them whole when they do not fit.
    pub fn put(&mut self, bytes: &[u8]) -> Result<(), Overflow> {
        if bytes.len() > self.room() {
            return Err(Overflow);
        }
        match &mut self.output {
            Output::Measure(written) => *written = written.checked_add(bytes.len()).expect("checked room"),
            Output::Write(output) => output.extend_from_slice(bytes),
        }
        Ok(())
    }

    /// The bytes written, in a box of exactly the length the writer was made
    /// for, which they must fill.
    #[must_use]
    #[expect(clippy::panic, reason = "finishing a measuring writer violates the caller's two-pass contract")]
    pub fn finish(self) -> Box<[u8]> {
        match self.output {
            Output::Measure(_) => panic!("a measuring writer cannot be finished"),
            Output::Write(bytes) => {
                assert!(bytes.len() == self.len, "a writer is finished full");
                bytes.into_boxed_slice()
            }
        }
    }
}
