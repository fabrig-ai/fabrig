//! Sized encoding and bounded decoding of the layer's private image.
use super::Refusal;
use alloc::boxed::Box;
use skein_lib::{List, Reader, Writer, bytes, len_u64, usize_of};

#[derive(Debug)]
pub(crate) struct Encoder {
    length: usize,
    out: Option<Writer>,
}
impl Encoder {
    pub(crate) const fn measure() -> Self {
        Self { length: 0, out: None }
    }
    pub(crate) fn writer(&self) -> Self {
        Self { length: 0, out: Some(Writer::new(self.length)) }
    }
    fn put(&mut self, value: &[u8]) {
        self.length = self.length.checked_add(value.len()).expect("image length fits memory");
        if let Some(out) = &mut self.out {
            out.put(value).expect("measured image");
        }
    }
    pub(crate) fn num(&mut self, value: u64) {
        self.put(&value.to_be_bytes());
    }
    pub(crate) fn flag(&mut self, value: bool) {
        self.put(&[u8::from(value)]);
    }
    pub(crate) fn bytes(&mut self, value: &[u8]) {
        self.num(len_u64(value.len()));
        self.put(value);
    }
    pub(crate) fn strings(&mut self, values: &[Box<[u8]>]) {
        self.num(len_u64(values.len()));
        for value in values {
            self.bytes(value);
        }
    }
    pub(crate) fn finish(self) -> Box<[u8]> {
        self.out.expect("writing pass").finish()
    }
}
pub(crate) fn num(input: &mut Reader<'_>, what: &'static str) -> Result<u64, Refusal> {
    input.u64().ok_or(Refusal::Malformed(what))
}
pub(crate) fn small(input: &mut Reader<'_>, what: &'static str) -> Result<u32, Refusal> {
    u32::try_from(num(input, what)?).or(Err(Refusal::Limits(what)))
}
pub(crate) fn count(input: &mut Reader<'_>, limit: u32, what: &'static str) -> Result<u32, Refusal> {
    let value = small(input, what)?;
    if value > limit {
        return Err(Refusal::Limits(what));
    }
    Ok(value)
}
pub(crate) fn flag(input: &mut Reader<'_>, what: &'static str) -> Result<bool, Refusal> {
    match input.u8() {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        Some(2..=255) | None => Err(Refusal::Malformed(what)),
    }
}
pub(crate) fn data(input: &mut Reader<'_>, limit: usize, what: &'static str) -> Result<Box<[u8]>, Refusal> {
    let length = num(input, what)?;
    if length > len_u64(limit) {
        return Err(Refusal::Limits(what));
    }
    let length = u32::try_from(length).or(Err(Refusal::Limits(what)))?;
    Ok(bytes::copy_of(input.bytes(length).ok_or(Refusal::Malformed(what))?))
}
pub(crate) fn strings(
    input: &mut Reader<'_>,
    cap: u32,
    limit: usize,
    what: &'static str,
) -> Result<Box<[Box<[u8]>]>, Refusal> {
    let length = count(input, cap, what)?;
    let mut values = List::with_capacity(length);
    for _ in 0..length {
        values.push(data(input, limit, what)?).expect("admitted string count");
    }
    Ok(values.into_boxed())
}
pub(crate) fn reader(bytes: &[u8]) -> Result<Reader<'_>, Refusal> {
    if bytes.len() > usize_of(u32::MAX) {
        return Err(Refusal::Limits("image bytes"));
    }
    Ok(Reader::new(bytes))
}
