use super::{Bounds, Undecodable, w};
use crate::generated::store;
use temper_engine_domain_forge_vocabulary::{What, writes::Writes};

pub(super) fn encode(row: &Writes) -> w::Writes {
    store::writes_to_record(row).expect(super::FITS)
}

pub(super) fn decode(row: w::Writes, bounds: &Bounds) -> Result<Writes, Undecodable> {
    let row = store::writes_to_domain(row, &super::Context::bounded(bounds)).or(Err(Undecodable::Malformed))?;
    let What::Branch(_) = row.name.what else { return Err(Undecodable::Malformed) };
    let mut previous = 0;
    for entry in &row.entries {
        if entry.order <= previous {
            return Err(Undecodable::Malformed);
        }
        previous = entry.order;
    }
    if row.next <= previous {
        return Err(Undecodable::Malformed);
    }
    Ok(row)
}
