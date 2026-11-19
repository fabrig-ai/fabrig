//! The two limit settings' shared file form (protocol/configuration/limit-settings.md, 2).
use crate::file::{Decoder, Error, error, field};
use crate::{Text, view as c};
use alloc::boxed::Box;
use skein_json::writer::Encoder;
use skein_lib::{List, Reader, Writer};
use temper_client_protocol_wire::v1 as w;

pub(crate) fn read(d: &Decoder<'_>, at: u32) -> Result<List<c::Override>, Error> {
    let positions = d.array(at, d.kinds.temper_limits_overrides, b"overrides")?;
    let mut values = List::with_capacity(positions.len());
    for &at in &positions {
        let fields = d.object(at, &[b"name", b"value", b"reason"])?;
        values
            .push(c::Override {
                name: d.words(field(&fields, b"name")?, d.kinds.limit_override_name, b"name")?.into_bytes(),
                value: d.number(field(&fields, b"value")?)?,
                reason: d.words(field(&fields, b"reason")?, d.kinds.limit_override_reason, b"reason")?.into_bytes(),
            })
            .expect("counted overrides");
    }
    Ok(values)
}
pub(super) fn value(d: &Decoder<'_>, at: u32) -> Result<Box<[u8]>, Error> {
    let values = read(d, at)?;
    let mut overrides = List::with_capacity(values.len());
    for value in values.into_boxed() {
        overrides
            .push(
                w::LimitOverride::new(
                    &d.kinds,
                    w::LimitOverrideParts { name: value.name, value: value.value, reason: value.reason },
                )
                .or(Err(error(b"overrides", b"wire value bound")))?,
            )
            .expect("counted overrides");
    }
    let record = w::TemperLimits::new(&d.kinds, w::TemperLimitsParts { overrides })
        .or(Err(error(b"overrides", b"wire value bound")))?;
    let mut out = Writer::new(skein_lib::usize_of(record.measure()));
    record.encode(&mut out).expect("measured record");
    Ok(out.finish())
}
pub(crate) fn decoded(bytes: &[u8]) -> Option<Box<[c::Override]>> {
    let mut reader = Reader::new(bytes);
    let record = w::TemperLimits::decode(&w::CEILINGS, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    let mut values = List::with_capacity(record.overrides().len());
    for value in record.into_parts().overrides.into_boxed() {
        let w::LimitOverrideParts { name, value, reason } = value.into_parts();
        values.push(c::Override { name, value, reason }).expect("decoded overrides");
    }
    Some(values.into_boxed())
}
pub(super) fn render(bytes: &[u8], out: &mut Encoder) -> bool {
    let Some(values) = decoded(bytes) else { return false };
    encode(&values, out);
    true
}
pub(crate) fn encode(values: &[c::Override], out: &mut Encoder) {
    out.array_start();
    for value in values {
        out.object_start();
        out.key(b"name");
        out.string(&value.name);
        out.key(b"value");
        out.unsigned(value.value);
        out.key(b"reason");
        out.string(&value.reason);
        out.object_end();
    }
    out.array_end();
}
pub(crate) fn line(value: &c::Override, out: &mut Text) {
    out.put(&value.name);
    out.put(b" = ");
    out.number(value.value);
    out.put(b"  ");
    out.put(&value.reason);
}
