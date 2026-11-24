//! Family scopes and windows outside the application's opaque records (client/domain/contract/watches.md, 1.5).
use alloc::boxed::Box;
use jig_client_domain::{Family, contract as c};
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use skein_lib::{Reader, Writer, bytes};
pub(crate) fn scope(value: c::Scope) -> w::Scope {
    match value {
        c::Scope::Project(value) => w::Scope::Project(w::ScopeProject { value }),
        c::Scope::Deployment => w::Scope::Deployment,
        c::Scope::Own => w::Scope::Own,
    }
}
pub(crate) fn read_scope(value: &w::Scope) -> c::Scope {
    match value {
        w::Scope::Project(value) => c::Scope::Project(value.value),
        w::Scope::Deployment => c::Scope::Deployment,
        w::Scope::Own => c::Scope::Own,
    }
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
    let after = match window.after {
        Some(value) => Some(w::Cursor { value: bytes::copy_of(value.value.bytes()) }),
        None => None,
    };
    let window = w::Window { rows: window.rows, after };
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
