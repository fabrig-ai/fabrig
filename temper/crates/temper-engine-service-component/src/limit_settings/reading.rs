//! The service's paged, unified limits read (configuration/limit-settings.md, 2.4 and 7).
use super::LimitSettings;
use alloc::boxed::Box;
use jig_domain_core::settings::Override;
use jig_protocol_config::names as jig;
use skein_lib::{List, Map};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_engine_service_vocabulary::limits as own;
use w::Record;

pub(crate) fn answer(state: &LimitSettings, after: Option<&[u8]>, held: u64, bounds: &w::Limits) -> Option<Box<[u8]>> {
    let stock = super::resolve_limits(&state.inputs, &[], &[]).ok()?;
    let desired = super::resolve_limits(&state.inputs, &state.jig, &state.temper).ok();
    let mut rows: Map<Box<[u8]>, w::LimitRow> = Map::uncapped();
    for name in &jig::NAMES {
        let setting = setting(&state.jig, name.name.as_bytes(), w::LimitKind::Jig, bounds).ok()?;
        let in_force = (name.value)(&state.in_force.jig);
        let wanted = match &setting {
            Some(setting) => setting.value,
            None => match &desired {
                Some(desired) => (name.value)(&desired.jig),
                None => (name.value)(&stock.jig),
            },
        };
        let row = w::LimitRow {
            name: name.name.as_bytes().into(),
            line: name.line.as_bytes().into(),
            unit: match name.unit {
                jig::Unit::Count => w::LimitUnit::Count,
                jig::Unit::Bytes => w::LimitUnit::Bytes,
                jig::Unit::Milliseconds => w::LimitUnit::Milliseconds,
            },
            stock: Some((name.value)(&stock.jig)),
            setting,
            in_force: Some(in_force),
            applies: applies(name.applies),
            waiting: (state.held && has(&state.jig, name.name.as_bytes())) || wanted != in_force,
            left_out: false,
            held: None,
        };
        rows.insert(row.name.clone(), row).ok()?;
    }
    for name in &own::NAMES {
        let setting = setting(&state.temper, name.name.as_bytes(), w::LimitKind::Temper, bounds).ok()?;
        let in_force = (name.value)(&state.in_force.temper);
        let wanted = match &setting {
            Some(setting) => setting.value,
            None => match &desired {
                Some(desired) => (name.value)(&desired.temper),
                None => (name.value)(&stock.temper),
            },
        };
        let row = w::LimitRow {
            name: name.name.as_bytes().into(),
            line: name.line.as_bytes().into(),
            unit: match name.unit {
                own::Unit::Count => w::LimitUnit::Count,
                own::Unit::Bytes => w::LimitUnit::Bytes,
                own::Unit::Milliseconds => w::LimitUnit::Milliseconds,
            },
            stock: Some((name.value)(&stock.temper)),
            setting,
            in_force: Some(in_force),
            applies: applies(name.applies),
            waiting: (state.held && has(&state.temper, name.name.as_bytes())) || wanted != in_force,
            left_out: false,
            held: if name.name == "transcript_memory" { Some(held) } else { None },
        };
        rows.insert(row.name.clone(), row).ok()?;
    }
    for (kind, values) in [(w::LimitKind::Jig, &state.jig), (w::LimitKind::Temper, &state.temper)] {
        for item in values {
            if rows.contains_key(item.name.as_ref()) {
                continue;
            }
            let row = w::LimitRow {
                name: item.name.clone(),
                line: Box::default(),
                unit: w::LimitUnit::Count,
                stock: None,
                setting: setting(values, &item.name, kind.clone(), bounds).ok()?,
                in_force: None,
                applies: w::LimitApplies::NextStart,
                waiting: state.held,
                left_out: true,
                held: None,
            };
            rows.insert(item.name.clone(), row).ok()?;
        }
    }
    let mut page = List::with_capacity(bounds.limits_answer_rows);
    let mut last = None;
    let mut next = None;
    for (name, row) in &rows {
        if let Some(after) = after
            && name.as_ref() <= after
        {
            continue;
        }
        if page.len() == page.capacity() {
            next = last;
            break;
        }
        last = Some(name.clone());
        page.push(row.clone()).ok()?;
    }
    let record = w::LimitsAnswer { rows: page, next, revision: state.revision };
    Some(wire::limits_answer(&record))
}
fn applies(applies: jig::Applies) -> w::LimitApplies {
    match applies {
        jig::Applies::AtOnce => w::LimitApplies::AtOnce,
        jig::Applies::Live => w::LimitApplies::Live,
        jig::Applies::NextStart => w::LimitApplies::NextStart,
    }
}
fn has(values: &[Override], name: &[u8]) -> bool {
    for item in values {
        if item.name.as_ref() == name {
            return true;
        }
    }
    false
}
fn setting(
    values: &[Override],
    name: &[u8],
    kind: w::LimitKind,
    bounds: &w::Limits,
) -> Result<Option<w::LimitSetting>, w::Problem> {
    for item in values {
        if item.name.as_ref() == name {
            let value = w::LimitSetting { kind, value: item.value, reason: item.reason.clone() };
            value.measure(bounds)?;
            return Ok(Some(value));
        }
    }
    Ok(None)
}
