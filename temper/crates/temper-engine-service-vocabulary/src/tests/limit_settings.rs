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

fn temper_names(applies: Applies) -> Box<[&'static str]> {
    let mut names = skein_lib::List::with_capacity(u32::try_from(NAMES.len()).unwrap());
    for row in &NAMES {
        if row.applies == applies {
            names.push(row.name).unwrap();
        }
    }
    let mut names = names.into_boxed();
    names.sort_unstable();
    names
}

#[test]
fn only_the_live_limits_take_a_raise_at_once() {
    assert_eq!(
        temper_names(Applies::AtOnce).as_ref(),
        ["forge.client.entries", "forge.entries", "forge.holds", "forge.tasks", "transcript_memory"]
    );
}

#[test]
fn live_rows_match_the_design_for_temper_and_jig() {
    // configuration/limit-settings.md, 5.2: eleven temper rows and ten jig rows.
    assert_eq!(
        temper_names(Applies::Live).as_ref(),
        [
            "checkout.local_timeout_ms",
            "checkout.message_bytes",
            "checkout.preparations",
            "checkout.reds_read_ms",
            "checkout.remote_timeout_ms",
            "checkout.rerun_processes",
            "git_component.read_bytes",
            "git_component.stderr_bytes",
            "git_component.stdout_bytes",
            "git_component.sweep_bytes",
            "git_component.sweep_entries",
        ]
    );
    let rows = &jig_protocol_config::names::NAMES;
    let mut jig = skein_lib::List::with_capacity(u32::try_from(rows.len()).unwrap());
    for row in rows {
        if row.applies == Applies::Live {
            jig.push(row.name).unwrap();
        }
    }
    let mut jig = jig.into_boxed();
    jig.sort_unstable();
    assert_eq!(
        jig.as_ref(),
        [
            "hub.accounts",
            "hub.event_bytes",
            "hub.turn_queue_bytes",
            "link.backoff_ms",
            "link.busy_ms",
            "link.ceiling_ms",
            "link.handshake_ms",
            "link.hello_ms",
            "link.proved_ms",
            "link.stall_ms",
        ]
    );
}

#[test]
fn jig_and_temper_names_are_disjoint() {
    for name in &NAMES {
        assert!(jig_protocol_config::names::find(name.name.as_bytes()).is_none(), "{}", name.name);
    }
}
