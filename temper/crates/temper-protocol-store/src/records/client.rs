use super::{Bounds, Undecodable, w};
use alloc::boxed::Box;

pub(super) fn encode(value: &skein_forge_client::Stored) -> w::ClientRow {
    let value = match value {
        skein_forge_client::Stored::Writer(_) => unreachable!("the connector keeps the client writer separately"),
        skein_forge_client::Stored::Live(value) => w::ClientValue::Live(Box::new(
            crate::generated::store::live_record_to_record(value, &super::Context::UNBOUNDED).expect(super::FITS),
        )),
        skein_forge_client::Stored::Repository(value) => w::ClientValue::Repository(
            crate::generated::store::repository_record_to_record(value, &super::Context::UNBOUNDED).expect(super::FITS),
        ),
    };
    w::ClientRow { value }
}
pub(super) fn decode(value: w::ClientRow, bounds: &Bounds) -> Result<skein_forge_client::Stored, Undecodable> {
    let context = super::Context::bounded(bounds);
    Ok(match value.value {
        w::ClientValue::Live(value) => skein_forge_client::Stored::Live(
            crate::generated::store::live_record_to_domain(*value, &context).or(Err(Undecodable::Malformed))?,
        ),
        w::ClientValue::Repository(value) => skein_forge_client::Stored::Repository(
            crate::generated::store::repository_record_to_domain(value, &context).or(Err(Undecodable::Malformed))?,
        ),
    })
}
