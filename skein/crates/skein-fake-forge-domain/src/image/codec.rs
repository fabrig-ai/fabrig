//! Sized writing and bounded reading for the domain's length-prefixed image.
use super::Refusal;
use alloc::boxed::Box;
use skein_lib::{Duration, Reader, Time, Writer};

pub(super) enum Sink {
    Measure(usize),
    Write(Writer),
}
impl Sink {
    pub(super) fn put(&mut self, bytes: &[u8]) {
        match self {
            Self::Measure(size) => *size = size.checked_add(bytes.len()).expect("held image fits address space"),
            Self::Write(writer) => writer.put(bytes).expect("image measured before writing"),
        }
    }
    pub(super) fn u8(&mut self, value: u8) {
        self.put(&[value]);
    }
    pub(super) fn flag(&mut self, value: bool) {
        self.u8(u8::from(value));
    }
    pub(super) fn u32(&mut self, value: u32) {
        self.put(&value.to_be_bytes());
    }
    pub(super) fn u64(&mut self, value: u64) {
        self.put(&value.to_be_bytes());
    }
    pub(super) fn len(&mut self, value: usize) {
        self.u32(u32::try_from(value).expect("domain counts fit u32"));
    }
    pub(super) fn bytes(&mut self, value: &[u8]) {
        self.len(value.len());
        self.put(value);
    }
    pub(super) fn optional_u64(&mut self, value: Option<u64>) {
        self.flag(value.is_some());
        if let Some(value) = value {
            self.u64(value);
        }
    }
}
pub(super) fn u8(input: &mut Reader<'_>) -> Result<u8, Refusal> {
    input.u8().ok_or(Refusal::Malformed("byte"))
}
pub(super) fn u32(input: &mut Reader<'_>) -> Result<u32, Refusal> {
    input.u32().ok_or(Refusal::Malformed("count"))
}
pub(super) fn u64(input: &mut Reader<'_>) -> Result<u64, Refusal> {
    input.u64().ok_or(Refusal::Malformed("number"))
}
pub(super) fn flag(input: &mut Reader<'_>) -> Result<bool, Refusal> {
    match u8(input)? {
        0 => Ok(false),
        1 => Ok(true),
        _tag => Err(Refusal::Malformed("flag")),
    }
}
pub(super) fn time(input: &mut Reader<'_>) -> Result<Time, Refusal> {
    Ok(Time::from_nanos(u64(input)?))
}
pub(super) fn duration(input: &mut Reader<'_>) -> Result<Duration, Refusal> {
    Ok(Duration::from_nanos(u64(input)?))
}
pub(super) fn optional_u64(input: &mut Reader<'_>) -> Result<Option<u64>, Refusal> {
    if flag(input)? { Ok(Some(u64(input)?)) } else { Ok(None) }
}
pub(super) fn optional_time(input: &mut Reader<'_>) -> Result<Option<Time>, Refusal> {
    if flag(input)? { Ok(Some(time(input)?)) } else { Ok(None) }
}
pub(super) fn count(input: &mut Reader<'_>, cap: u32, what: &'static str) -> Result<u32, Refusal> {
    let n = u32(input)?;
    if n > input.remaining() {
        return Err(Refusal::Malformed(what));
    }
    if n > cap {
        return Err(Refusal::Limits(what));
    }
    Ok(n)
}
pub(super) fn bytes(input: &mut Reader<'_>, cap: u32, what: &'static str) -> Result<Box<[u8]>, Refusal> {
    let n = count(input, cap, what)?;
    Ok(Box::from(input.bytes(n).ok_or(Refusal::Malformed(what))?))
}
pub(super) fn require(ok: bool, what: &'static str) -> Result<(), Refusal> {
    if ok { Ok(()) } else { Err(Refusal::Malformed(what)) }
}
