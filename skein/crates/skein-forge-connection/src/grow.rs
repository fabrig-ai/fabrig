//! Forge connection grow (forge/connection.md, sections 3 and 8).
use alloc::boxed::Box;
use skein_lib::List;

/// A write past the cap, refused whole: nothing written.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Full;

/// Bytes of unknown length, at most `cap`.
#[derive(Debug)]
pub struct Grow {
    bytes: List<u8>,
    cap: u32,
}

impl Grow {
    pub(crate) fn len(&self) -> u32 {
        self.bytes.len()
    }
    pub(crate) fn room(&self) -> u32 {
        self.cap.saturating_sub(self.bytes.len())
    }
    /// Starts at `first` bytes (at most `cap`).
    #[must_use]
    pub fn new(first: u32, cap: u32) -> Grow {
        Grow { bytes: List::with_capacity(first.min(cap)), cap }
    }

    /// Appends `piece` whole, growing to twice the capacity or what it needs, never past `cap`.
    pub fn write(&mut self, piece: &[u8]) -> Result<(), Full> {
        let Ok(length) = u32::try_from(piece.len()) else { return Err(Full) };
        let Some(need) = self.bytes.len().checked_add(length) else { return Err(Full) };
        if need > self.cap {
            return Err(Full);
        }
        if need > self.bytes.capacity() {
            let mut grown = List::with_capacity(need.max(self.bytes.capacity().saturating_mul(2)).min(self.cap));
            for byte in self.bytes.as_slice() {
                grown.push(*byte).expect("grown past the length");
            }
            self.bytes = grown;
        }
        for byte in piece {
            self.bytes.push(*byte).expect("room checked above");
        }
        Ok(())
    }

    /// The bytes, in a box of exactly their length.
    #[must_use]
    pub fn finish(self) -> Box<[u8]> {
        self.bytes.into_boxed()
    }
}
impl Grow {
    pub(crate) fn capacity(&self) -> u32 {
        self.bytes.capacity()
    }
}
/// A body's last bytes, optionally ending at its first failed process (forge/connection.md, 4.5).
#[derive(Debug)]
pub struct LogTail {
    tail: skein_lib::Tail,
    dropped: bool,
    failure: Option<FailureEnd>,
}
/// Recognises the runner's process-failure line across body chunks.
#[derive(Debug)]
struct FailureEnd {
    matched: u8,
    ended: bool,
}
impl FailureEnd {
    fn byte(&mut self, byte: u8) {
        const MARKER: &[u8] = b"##[error]Process completed with exit code ";
        if usize::from(self.matched) == MARKER.len() {
            self.ended = byte == b'\n';
        } else if MARKER.get(usize::from(self.matched)) == Some(&byte) {
            self.matched = self.matched.saturating_add(1);
        } else {
            self.matched = u8::from(byte == b'#');
        }
    }
}
impl LogTail {
    #[must_use]
    pub fn new(max: u32) -> Self {
        Self { tail: skein_lib::Tail::with_capacity(max), dropped: false, failure: None }
    }
    /// Retain through the first runner process-failure line, draining cleanup.
    #[must_use]
    pub fn failed_step(max: u32) -> Self {
        Self { failure: Some(FailureEnd { matched: 0, ended: false }), ..Self::new(max) }
    }
    pub fn write(&mut self, piece: &[u8]) {
        for byte in piece {
            if let Some(failure) = &mut self.failure {
                if failure.ended {
                    self.dropped = true;
                    continue;
                }
                failure.byte(*byte);
            }
            self.dropped |= self.tail.len() == self.tail.capacity();
            self.tail.push(*byte);
        }
    }
    /// Whether any prefix or cleanup byte was omitted.
    #[must_use]
    pub fn dropped(&self) -> bool {
        self.dropped
    }
    #[must_use]
    pub fn finish(self) -> Box<[u8]> {
        self.tail.into_boxed()
    }
}
