//! Fixed-size stderr tail, with the overwritten bytes counted.
use alloc::boxed::Box;
use skein_lib::{Writer, bytes};
#[derive(Debug)]
pub struct ByteRing {
    bytes: Box<[u8]>,
    next: usize,
    length: usize,
    dropped: u64,
}
impl ByteRing {
    #[must_use]
    pub fn new(capacity: u32) -> ByteRing {
        ByteRing {
            bytes: bytes::zeroed(usize::try_from(capacity).expect("u32 fits usize")),
            next: 0,
            length: 0,
            dropped: 0,
        }
    }
    pub fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            if self.bytes.is_empty() {
                self.dropped = self.dropped.saturating_add(1);
                continue;
            }
            *self.bytes.get_mut(self.next).expect("ring position within capacity") = *byte;
            self.next = self
                .next
                .checked_add(1)
                .expect("within capacity")
                .checked_rem(self.bytes.len())
                .expect("nonempty ring");
            if self.length < self.bytes.len() {
                self.length = self.length.checked_add(1).expect("within capacity");
            } else {
                self.dropped = self.dropped.saturating_add(1);
            }
        }
    }
    #[must_use]
    pub const fn dropped(&self) -> u64 {
        self.dropped
    }
    #[must_use]
    pub fn tail(&self) -> Box<[u8]> {
        let mut out = Writer::new(self.length);
        let first = if self.length == self.bytes.len() { self.next } else { 0 };
        for i in 0..self.length {
            let position =
                first.checked_add(i).expect("bounded ring").checked_rem(self.bytes.len()).expect("nonempty tail");
            out.put(&[*self.bytes.get(position).expect("within ring")]).expect("measured tail fits");
        }
        out.finish()
    }
}
