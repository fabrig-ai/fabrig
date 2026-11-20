//! Limits show the running snapshot, not merely the newest setting.
use super::{LimitSettings, inputs, item};
use crate::limit_settings::{reading, resolve_limits};
use temper_client_protocol_wire::{self as wire, v1 as w};

pub(super) fn page(state: &LimitSettings, after: Option<&[u8]>) -> w::LimitsAnswer {
    let encoded = reading::answer(state, after, 1234, &w::CEILINGS).unwrap();
    let answer = wire::decode_limits_answer(&encoded, &w::CEILINGS).unwrap();
    assert_eq!(answer.revision, state.revision);
    answer
}

pub(super) fn read_named(state: &LimitSettings, name: &[u8]) -> w::LimitRow {
    // Start the page immediately before the named row, not at the first limit.
    let (_, after) = name.split_last().expect("nonempty limit name");
    named(page(state, Some(after)).rows.as_slice(), name).clone()
}

fn rows(state: &LimitSettings) -> Vec<w::LimitRow> {
    let mut rows = Vec::new();
    let mut after = None;
    for _ in 0_u32..32 {
        let answer = page(state, after.as_deref());
        assert!(answer.rows.len() <= 32);
        let w::LimitsAnswer { rows: page, next, .. } = answer;
        rows.extend(page.into_boxed());
        if next.is_none() {
            return rows;
        }
        assert!(next > after);
        after = next;
    }
    panic!("finite names table pages")
}
fn named<'a>(rows: &'a [w::LimitRow], name: &[u8]) -> &'a w::LimitRow {
    rows.iter().find(|row| row.name.as_ref() == name).unwrap()
}
#[test]
fn the_limits_read_unifies_names_and_preserves_stock_setting_and_running_figures() {
    let inputs = inputs();
    let temper: Box<[jig_domain_core::settings::Override]> =
        Box::new([item(b"forge.client.entries", 16), item(b"forge.entries", 16)]);
    let in_force = resolve_limits(&inputs, &[], &temper).unwrap();
    let mut state = LimitSettings {
        inputs,
        jig: Box::new([]),
        jig_taken: Box::new([]),
        temper_taken: temper.clone(),
        temper,
        in_force,
        revision: 7,
        held: false,
    };
    state.temper[1].reason = b"dev's outbox".as_slice().into();
    let read = rows(&state);
    assert_eq!(read.len(), jig_protocol_config::names::NAMES.len() + crate::limits::NAMES.len());
    assert!(read.windows(2).all(|pair| pair[0].name < pair[1].name));
    let entries = named(&read, b"forge.entries");
    assert_eq!(&entries.stock, &Some(8));
    assert_eq!(&entries.in_force, &Some(16));
    assert_eq!(&entries.applies, &w::LimitApplies::AtOnce);
    let setting = entries.setting.as_ref().unwrap();
    assert_eq!(
        (&setting.kind, setting.value, setting.reason.as_ref()),
        (&w::LimitKind::Temper, 16, b"dev's outbox".as_slice())
    );
    assert!(!entries.waiting);
    assert_eq!(entries.line.as_ref(), crate::limits::find(b"forge.entries").unwrap().line.as_bytes());
    for (name, applies) in [
        (b"tasks.tasks".as_slice(), w::LimitApplies::NextStart),
        (b"checkout.remote_timeout_ms", w::LimitApplies::Live),
        (b"link.stall_ms", w::LimitApplies::Live),
    ] {
        assert_eq!(named(&read, name).applies, applies, "{}", String::from_utf8_lossy(name));
    }
    let transcript = named(&read, b"transcript_memory");
    assert_eq!(&transcript.held, &Some(1234));
    assert_eq!(&transcript.unit, &w::LimitUnit::Bytes);
    state.temper[1].value = 32;
    let read = rows(&state);
    let entries = named(&read, b"forge.entries");
    assert_eq!(&entries.in_force, &Some(16));
    assert_eq!(entries.setting.as_ref().unwrap().value, 32);
    assert!(entries.waiting);
}
#[test]
fn a_held_start_shows_even_invalid_and_dropped_settings_as_waiting() {
    let inputs = inputs();
    let in_force = resolve_limits(&inputs, &[], &[]).unwrap();
    let state = LimitSettings {
        inputs,
        jig: Box::new([item(b"tasks.tasks", 0)]),
        jig_taken: Box::new([]),
        temper_taken: Box::new([]),
        temper: Box::new([item(b"forge.entries", 8), item(b"old.limit", 17)]),
        in_force,
        revision: 7,
        held: true,
    };
    let read = rows(&state);
    for name in [b"tasks.tasks".as_slice(), b"forge.entries", b"old.limit"] {
        assert!(named(&read, name).waiting);
    }
    let dropped = named(&read, b"old.limit");
    assert!(dropped.left_out);
    assert_eq!(&dropped.stock, &None);
    assert_eq!(&dropped.in_force, &None);
    assert_eq!(dropped.setting.as_ref().unwrap().value, 17);
    assert_eq!(&dropped.setting.as_ref().unwrap().kind, &w::LimitKind::Temper);
}
