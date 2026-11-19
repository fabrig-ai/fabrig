use crate::limits::{self, Applies, NAMES, Unit};

#[test]
fn the_source_figures_have_their_stock_units_and_lines() {
    let stock = limits::stock();
    for (name, unit, value) in [
        ("live_goals", Unit::Count, 40),
        ("tool_payload", Unit::Bytes, 63_740),
        ("transcript_memory", Unit::Bytes, 1_073_741_824),
    ] {
        let row = limits::find(name.as_bytes()).unwrap();
        assert_eq!(row.unit, unit);
        assert_eq!((row.value)(&stock), value, "{name}");
        assert_ne!(row.line, "");
        let mut raised = stock;
        limits::apply(&mut raised, name.as_bytes(), value * 2).unwrap();
        assert_eq!((row.value)(&raised), value * 2);
    }
}

#[test]
fn only_the_live_limits_take_a_raise_at_once() {
    let mut names = skein_lib::List::with_capacity(197);
    for row in &NAMES {
        if row.applies == Applies::AtOnce {
            names.push(row.name).unwrap();
        }
    }
    let mut names = names.into_boxed();
    names.sort_unstable();
    assert_eq!(
        names.as_ref(),
        ["forge.client.entries", "forge.entries", "forge.holds", "forge.tasks", "transcript_memory"]
    );
}

#[test]
fn jig_and_temper_names_are_disjoint() {
    for name in &NAMES {
        assert!(jig_protocol_config::names::find(name.name.as_bytes()).is_none(), "{}", name.name);
    }
}
