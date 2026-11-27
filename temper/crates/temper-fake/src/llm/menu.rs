//! Standalone draw menus (temper/docs/design/protocol/setup/playground.md, 9).
use crate::{Refusal, read, turn};
use alloc::boxed::Box;
use jig_protocol_config::Path;
use skein_fake_llm_domain::api::{Bounds, InvalidInput, Menu, Slot};
use skein_json::Document;
use skein_lib::List;

pub(super) fn read(d: &Document, at: u32, p: &Path) -> Result<(Menu, u32), Refusal> {
    let mut o = read::object(d, at, p)?;
    let arguments_at = read::required(&mut o, d, b"arguments", p)?;
    let invalid_at = read::required(&mut o, d, b"invalid", p)?;
    let slots_at = read::required(&mut o, d, b"slots", p)?;
    let calls_at = read::required(&mut o, d, b"calls", p)?;
    read::finish(o, d, p)?;

    let calls = read::integer(d, calls_at, 16, &p.member(b"calls"))?;
    if calls == 0 {
        return Err(read::why(&p.member(b"calls"), b"must be at least 1"));
    }
    let arguments_path = p.member(b"arguments");
    let items = read::bounded_list(d, arguments_at, 1, 16, &arguments_path)?;
    let mut arguments = List::with_capacity(items.len());
    for (i, at) in items.iter().enumerate() {
        let body = turn::object_body(d, *at, crate::SCRIPT_BYTES, &read::index(&arguments_path, i))?;
        arguments.push(body).expect("sixteen bodies");
    }
    let invalid_path = p.member(b"invalid");
    let items = read::bounded_list(d, invalid_at, 0, 8, &invalid_path)?;
    let mut invalid = List::with_capacity(items.len());
    for (i, at) in items.iter().enumerate() {
        invalid.push(invalid_input(d, *at, &read::index(&invalid_path, i))?).expect("eight invalid entries");
    }
    let slots_path = p.member(b"slots");
    let items = read::bounded_list(d, slots_at, 0, 8, &slots_path)?;
    let mut slots = List::with_capacity(items.len());
    for (i, at) in items.iter().enumerate() {
        slots.push(slot(d, *at, &read::index(&slots_path, i))?).expect("eight slots");
    }
    let menu = Menu { arguments: arguments.into_boxed(), invalid: invalid.into_boxed(), slots: slots.into_boxed() };
    Ok((menu, u32::try_from(calls).expect("calls bounded to 16")))
}

fn invalid_input(d: &Document, at: u32, p: &Path) -> Result<InvalidInput, Refusal> {
    let mut o = read::object(d, at, p)?;
    let name_at = read::required(&mut o, d, b"name", p)?;
    let arguments = read::field(&mut o, d, b"arguments", crate::DOCUMENT_BYTES, p)?;
    read::finish(o, d, p)?;
    let name = if read::is_null(d, name_at) {
        None
    } else {
        Some(read::text(d, name_at, crate::DOCUMENT_BYTES, &p.member(b"name"))?)
    };
    Ok(InvalidInput { name, arguments })
}

fn slot(d: &Document, at: u32, p: &Path) -> Result<Slot, Refusal> {
    // Skein owns the slot bounds, earlier-slot references and placeholder checks.
    let mut o = read::object(d, at, p)?;
    let name = read::field(&mut o, d, b"name", crate::DOCUMENT_BYTES, p)?;
    let before = read::field(&mut o, d, b"before", crate::DOCUMENT_BYTES, p)?;
    let after = read::field(&mut o, d, b"after", crate::DOCUMENT_BYTES, p)?;
    let unissued = read::field(&mut o, d, b"unissued", crate::DOCUMENT_BYTES, p)?;
    let weight_at = read::required(&mut o, d, b"weight", p)?;
    let numeric_at = read::required(&mut o, d, b"numeric", p)?;
    let with_at = read::required(&mut o, d, b"with", p)?;
    let foreign_at = read::required(&mut o, d, b"foreign", p)?;
    read::finish(o, d, p)?;
    let weight = read::integer(d, weight_at, u64::from(u32::MAX), &p.member(b"weight"))?;
    let foreign = read::integer(d, foreign_at, u64::from(u32::MAX), &p.member(b"foreign"))?;
    let numeric = if read::is_null(d, numeric_at) { None } else { Some(bounds(d, numeric_at, &p.member(b"numeric"))?) };
    let with = if read::is_null(d, with_at) {
        Box::new([])
    } else {
        let p = p.member(b"with");
        let name = read::text(d, with_at, crate::DOCUMENT_BYTES, &p)?;
        read::nonempty(&name, &p)?;
        name
    };
    Ok(Slot {
        name,
        before,
        after,
        unissued,
        weight: u32::try_from(weight).expect("u32 bound"),
        numeric,
        with,
        foreign: u32::try_from(foreign).expect("u32 bound"),
    })
}

fn bounds(d: &Document, at: u32, p: &Path) -> Result<Bounds, Refusal> {
    let mut o = read::object(d, at, p)?;
    let low_at = read::required(&mut o, d, b"low", p)?;
    let high_at = read::required(&mut o, d, b"high", p)?;
    read::finish(o, d, p)?;
    let low = read::integer(d, low_at, u64::MAX, &p.member(b"low"))?;
    let high = read::integer(d, high_at, u64::MAX, &p.member(b"high"))?;
    Ok(Bounds { low, high })
}
