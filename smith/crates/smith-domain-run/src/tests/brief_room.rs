//! Brief capacity uses admission's host measure and the literal section renderer.
//! No state or text meaning is retained; the tests hold domain/run.md, section 4.

use super::{LIMITS, bytes, charter, coding, coding_workspace};
use crate::outcome::{ChangeSpec, OutcomeSpec, TextSpec};
use crate::{Brief, Charter, Invalid, Limits, Mode, SchemaCosts, Section, brief_bytes, brief_room};
use alloc::boxed::Box;

const ROOM: u32 = 32;
const COSTS: SchemaCosts = SchemaCosts {
    finish_text: 11,
    deliver_text: 13,
    finish: 17,
    deliver: 19,
    change: 23,
    report: 29,
    field_first: 31,
    field_next: 37,
    escaped: [2; 256],
    ..SchemaCosts::NONE
};

fn policies() -> [Charter; 3] {
    let mut change = coding();
    change.grants.deliver = Some(ChangeSpec { checks_must_pass: true, fields: Box::default() });
    let mut report = charter();
    report.outcome = OutcomeSpec {
        change: None,
        verdicts: Box::default(),
        report: Some(TextSpec { max: 32, fields: Box::default() }),
        failure: None,
    };
    let open = Charter { mode: Mode::Open, ..charter() };
    [change, report, open]
}

fn brief() -> Brief {
    Brief {
        sections: Box::new([
            Section { title: bytes(b""), text: bytes(b"") },
            Section { title: bytes(b"Context"), text: bytes(b"line\n") },
            Section { title: bytes(b"Context"), text: bytes(b"work") },
        ]),
    }
}

#[test]
fn brief_room_and_bytes_match_admission_at_the_bound_and_one_byte_past() {
    let mounted = coding_workspace();
    for host_tools in [false, true] {
        for mut selected in policies() {
            if !host_tools {
                selected.grants.host_tools = Box::default();
            }
            selected.instructions = bytes(b"Role\n");
            let limits = Limits { schema_costs: COSTS, ..LIMITS };
            let non_brief = limits.host_part - u64::from(brief_room(&selected, Some(&mounted), &limits).unwrap());
            let limits = Limits { host_part: non_brief + u64::from(ROOM), ..limits };
            assert_eq!(brief_room(&selected, Some(&mounted), &limits), Some(ROOM));
            // Five heading bytes and two paragraph bytes leave 25 payload bytes.
            selected.brief = Brief { sections: Box::new([Section { title: bytes(b""), text: bytes(&[b'x'; 25]) }]) };
            assert_eq!(brief_bytes(&selected.brief), Some(u64::from(ROOM)));
            assert_eq!(crate::charter::check(&selected, Some(&mounted), &limits), Ok(0));
            selected.brief.sections[0].text = bytes(&[b'x'; 26]);
            assert_eq!(brief_bytes(&selected.brief), Some(u64::from(ROOM) + 1));
            assert_eq!(brief_room(&selected, Some(&mounted), &limits), Some(ROOM));
            assert_eq!(
                crate::charter::check(&selected, Some(&mounted), &limits),
                Err(Invalid::HostPart { bound: limits.host_part })
            );
        }
    }
}

#[test]
fn declarations_past_the_host_bound_leave_no_brief_room() {
    let mounted = coding_workspace();
    for mut selected in policies() {
        selected.brief.sections = Box::default();
        let limits = Limits { schema_costs: COSTS, ..LIMITS };
        let non_brief = limits.host_part - u64::from(brief_room(&selected, Some(&mounted), &limits).unwrap());
        let exact = Limits { host_part: non_brief, ..limits };
        assert_eq!(brief_room(&selected, Some(&mounted), &exact), Some(0));
        assert_eq!(crate::charter::check(&selected, Some(&mounted), &exact), Ok(0));
        let short = Limits { host_part: non_brief - 1, ..limits };
        assert_eq!(brief_room(&selected, Some(&mounted), &short), None);
        assert_eq!(
            crate::charter::check(&selected, Some(&mounted), &short),
            Err(Invalid::HostPart { bound: short.host_part })
        );
    }
}

#[test]
fn brief_bytes_are_additive_with_empty_and_literal_sections_in_both_modes() {
    let mounted = coding_workspace();
    assert_eq!(brief_bytes(&Brief { sections: Box::default() }), Some(0));
    // Empty payloads still frame a section; one trailing newline saves one byte.
    assert_eq!(brief_bytes(&brief()), Some(43));
    for host_tools in [false, true] {
        for mut selected in policies() {
            if !host_tools {
                selected.grants.host_tools = Box::default();
            }
            for instructions in [b"".as_slice(), b"Role", b"Role\n"] {
                selected.instructions = bytes(instructions);
                for workspace in [None, Some(&mounted)] {
                    for guide_bytes in [0, LIMITS.guide_bytes] {
                        selected.brief.sections = Box::default();
                        let empty = crate::prompt::host_bytes(&selected, workspace, guide_bytes).unwrap();
                        selected.brief = brief();
                        let populated = crate::prompt::host_bytes(&selected, workspace, guide_bytes).unwrap();
                        assert_eq!(populated, empty + brief_bytes(&selected.brief).unwrap());
                    }
                }
            }
        }
    }
}

#[test]
fn brief_room_returns_none_on_count_or_room_conversion_overflow() {
    let selected = charter();
    let overflowing = Limits { schema_costs: SchemaCosts { finish: u64::MAX, ..COSTS }, ..LIMITS };
    assert_eq!(brief_room(&selected, None, &overflowing), None);
    assert_eq!(
        crate::charter::check(&selected, None, &overflowing),
        Err(Invalid::HostPart { bound: overflowing.host_part })
    );
    let non_brief = LIMITS.host_part - u64::from(brief_room(&selected, None, &LIMITS).unwrap());
    let maximum = Limits { host_part: non_brief + u64::from(u32::MAX), ..LIMITS };
    assert_eq!(brief_room(&selected, None, &maximum), Some(u32::MAX));
    let overflowing = Limits { host_part: maximum.host_part + 1, ..LIMITS };
    assert_eq!(brief_room(&selected, None, &overflowing), None);
}
