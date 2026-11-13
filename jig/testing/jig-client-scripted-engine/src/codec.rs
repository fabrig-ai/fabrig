//! In-memory wire records admitted by the client's link (client/domain/testing.md, 4).
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v21 as w;
use skein_lib::{List, Writer, bytes};

mod facts;
mod front;
mod turns;
mod values;
pub(crate) use front::{answer, ask, delivery, none, read, selector, subject};
use turns::turn_bytes;
pub use values::{
    wire_malformed_settings, wire_proposal_action_settings, wire_retained_action_settings, wire_revision_answer,
    wire_revisions_page, wire_setting_document, wire_settings_document,
};

#[cfg(test)]
mod tests;

fn problem() -> w::Problem {
    w::Problem { path: w::Path::OwnedRecord, reason: skein_codec::Reason::Bound }
}

fn encoded<T>(
    value: &T,
    measure: fn(&T) -> u32,
    encode: fn(&T, &mut Writer) -> Result<(), skein_lib::Overflow>,
) -> Result<Box<[u8]>, w::Problem> {
    let mut writer = Writer::new(usize::try_from(measure(value)).map_err(|_| problem())?);
    encode(value, &mut writer).map_err(|_| problem())?;
    Ok(writer.finish())
}

fn pieces<T>(
    rows: impl Iterator<Item = Result<T, w::Problem>>,
    measure: fn(&T) -> u32,
    encode: fn(&T, &mut Writer) -> Result<(), skein_lib::Overflow>,
) -> Result<Box<[u8]>, w::Problem> {
    let mut bytes = Vec::new();
    for row in rows {
        let row = row?;
        let record = encoded(&row, measure, encode)?;
        bytes.extend_from_slice(&measure(&row).to_be_bytes());
        bytes.extend_from_slice(&record);
    }
    Ok(bytes.into_boxed_slice())
}

fn list<T>(rows: impl Iterator<Item = Result<T, w::Problem>>) -> Result<List<T>, w::Problem> {
    let rows: Vec<_> = rows.collect::<Result<_, _>>()?;
    Ok(List::from_boxed(rows.into_boxed_slice()))
}

fn empty_pieces<T>(rows: &List<T>) -> Result<Box<[u8]>, w::Problem> {
    if rows.is_empty() { Ok(Box::new([])) } else { Err(problem()) }
}
