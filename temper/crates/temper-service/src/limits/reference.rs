//! Named limits as checked-in reference pages (temper/docs/design/protocol/documentation.md, 8).
use super::{NAMES, Profile, Unit, profile};
use skein_lib::{Decimal, Overflow, Writer};
use temper_forge_setup::limits as setup;

/// Temper's, the setup's and the box's override names at the small profile's values.
pub fn reference(out: &mut Writer) -> Result<(), Overflow> {
    out.put(b"# Limits\n\nKind: reference\n\nEvery limit application.limits.overrides takes: its unit, the small profile's value and what it bounds.\n")?;
    let limits = profile(Profile::Small);
    let mut owner = "";
    for name in &NAMES {
        if name.name.starts_with("setup.") {
            continue;
        }
        row(name.name, unit(name.unit), (name.value)(&limits), name.line, &mut owner, out)?;
    }
    let limits = setup::small();
    for name in &setup::NAMES {
        // The shared forge client name is already rendered from the service's profile.
        if name.name.starts_with("setup.") {
            row(name.name, setup_unit(name.unit), (name.value)(&limits), name.line, &mut owner, out)?;
        }
    }
    let limits = setup::host();
    for name in &setup::HOST_NAMES {
        let mut shared = false;
        for plan in &setup::NAMES {
            shared |= plan.name == name.name;
        }
        if !shared {
            row(name.name, setup_unit(name.unit), (name.value)(&limits), name.line, &mut owner, out)?;
        }
    }
    for name in &setup::FETCH_NAMES {
        row(name.name, setup_unit(name.unit), (name.value)(&setup::FETCH_BYTES), name.line, &mut owner, out)?;
    }
    let limits = temper_box::limits::small();
    for name in &temper_box::limits::NAMES {
        let unit = match name.unit {
            temper_box::limits::Unit::Count => "count",
            temper_box::limits::Unit::Bytes => "bytes",
            temper_box::limits::Unit::Milliseconds => "milliseconds",
        };
        row(name.name, unit, (name.value)(&limits), name.line, &mut owner, out)?;
    }
    Ok(())
}

/// Jig's engine and worker override names at the small profile's values.
pub fn engine_reference(out: &mut Writer) -> Result<(), Overflow> {
    out.put(b"# Engine limits\n\nKind: reference\n\nEvery limit engine.limits.overrides and a worker's limits.overrides take: its unit, the small profile's value and what it bounds.\n")?;
    let limits = jig_protocol_config::profile(jig_protocol_config::Profile::Small);
    let mut owner = "";
    for name in &jig_protocol_config::names::NAMES {
        let unit = match name.unit {
            jig_protocol_config::names::Unit::Count => "count",
            jig_protocol_config::names::Unit::Bytes => "bytes",
            jig_protocol_config::names::Unit::Milliseconds => "milliseconds",
        };
        row(name.name, unit, (name.value)(&limits), name.line, &mut owner, out)?;
    }
    Ok(())
}

fn unit(unit: Unit) -> &'static str {
    match unit {
        Unit::Count => "count",
        Unit::Bytes => "bytes",
        Unit::Milliseconds => "milliseconds",
    }
}

fn setup_unit(unit: setup::Unit) -> &'static str {
    match unit {
        setup::Unit::Count => "count",
        setup::Unit::Bytes => "bytes",
        setup::Unit::Milliseconds => "milliseconds",
    }
}

fn row(
    name: &'static str,
    unit: &str,
    value: u64,
    line: &str,
    owner: &mut &'static str,
    out: &mut Writer,
) -> Result<(), Overflow> {
    let prefix = name.split('.').next().expect("a dotted limit has an owner");
    if prefix != *owner {
        out.put(b"\n## ")?;
        out.put(prefix.as_bytes())?;
        out.put(b"\n\n")?;
        *owner = prefix;
    }
    out.put(b"- `")?;
    out.put(name.as_bytes())?;
    out.put(b"`: ")?;
    out.put(unit.as_bytes())?;
    out.put(b", ")?;
    let digits = Decimal::of(value);
    let digits = digits.as_bytes();
    for (index, digit) in digits.iter().enumerate() {
        if index != 0 && digits.len().saturating_sub(index).is_multiple_of(3) {
            out.put(b",")?;
        }
        out.put(core::slice::from_ref(digit))?;
    }
    out.put(b". ")?;
    out.put(line.as_bytes())?;
    out.put(b"\n")
}
