//! An owned output buffer grows geometrically, within its configured cap.
use alloc::boxed::Box;
use skein_lib::List;
#[derive(Debug)]
pub struct Capped {
    bytes: List<u8>,
    cap: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Full;
impl Capped {
    #[must_use]
    pub fn new(cap: u32) -> Capped {
        Capped { bytes: List::with_capacity(cap.min(64)), cap }
    }
    pub fn write(&mut self, bytes: &[u8]) -> Result<(), Full> {
        let count = u32::try_from(bytes.len()).or(Err(Full))?;
        let needed = self.bytes.len().checked_add(count).ok_or(Full)?;
        if needed > self.cap {
            return Err(Full);
        }
        if needed > self.bytes.capacity() {
            let capacity = self.bytes.capacity().saturating_mul(2).max(needed).min(self.cap);
            let mut next = List::with_capacity(capacity);
            for byte in &self.bytes {
                next.push(*byte).expect("grown capacity holds old bytes");
            }
            self.bytes = next;
        }
        for byte in bytes {
            self.bytes.push(*byte).expect("admitted write fits");
        }
        Ok(())
    }
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }
    #[must_use]
    pub fn finish(self) -> Box<[u8]> {
        self.bytes.into_boxed()
    }
    #[must_use]
    pub fn worst_case(cap: u32) -> Option<u64> {
        List::<u8>::price(cap)
    }
}
