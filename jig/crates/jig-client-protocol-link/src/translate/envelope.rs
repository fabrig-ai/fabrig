//! Family scopes and windows outside the application's opaque records (client/domain/contract/watches.md, 1.5).
use crate::generated::client;
use alloc::boxed::Box;
use jig_client_domain::{Family, contract as c};
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use skein_lib::{Reader, Writer, bytes};
pub(crate) fn scope(value: c::Scope) -> w::Scope {
    client::scope_to_record(&value)
}
pub(crate) fn read_scope(value: &w::Scope) -> c::Scope {
    client::scope_to_domain(value.clone())
}
pub(crate) fn request(scope_value: c::Scope, record: &Family, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    let value = w::FamilyRequest { scope: scope(scope_value), record: bytes::copy_of(&record.bytes) };
    let mut writer = Writer::new(skein_lib::length::usize_of(value.measure(limits)?));
    value.write(limits, &mut writer).expect("measured family request");
    Ok(writer.finish())
}
pub(crate) fn record(
    scope_value: c::Scope,
    window: c::Window,
    record: Family,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let window = client::window_to_record(&window);
    let value = w::FamilyRecord { scope: scope(scope_value), window, record: record.bytes };
    let mut writer = Writer::new(skein_lib::length::usize_of(value.measure(limits)?));
    value.write(limits, &mut writer).expect("measured family record");
    Ok(writer.finish())
}
pub(crate) fn restore(bytes: &[u8], limits: &w::Limits) -> Result<(c::Scope, Box<[u8]>), w::Problem> {
    let value = w::FamilyRequest::decode(limits, &mut Reader::new(bytes))?;
    Ok((read_scope(&value.scope), value.record))
}
pub(crate) fn versions(value: &w::NotServed) -> Option<c::VersionRange> {
    for range in &value.served {
        if range.owner == value.owner {
            return Some(c::VersionRange { first: range.first, last: range.last });
        }
    }
    None
}
pub(crate) fn family(owner: &w::Owner) -> Option<u16> {
    match owner {
        w::Owner::Jig => None,
        w::Owner::Family(owner) => Some(owner.number),
    }
}
