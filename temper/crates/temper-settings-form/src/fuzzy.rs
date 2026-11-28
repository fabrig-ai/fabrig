use crate::{
    Limits,
    fields::{
        self, Shape,
        tests::{Edit, generated},
    },
    file::{format_tests::TEST, read, read_with, render_with},
};
use alloc::format;

#[test]
#[expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "generate whole legacy and current fixtures as JSON text"
)]
fn generated_settings_round_trip_through_format_two() {
    let limits = Limits::standard();
    let edit = fields::member(Shape::Item, b"edit").expect("edit row");
    let mut seed = skein_lib::Rng::new(0x052c_7ae2);
    for edit in [Edit::Base, Edit::Word(edit, 0), Edit::Word(edit, 1)] {
        let (base, _) = generated(edit);
        let base = core::str::from_utf8(&base).expect("generated JSON").strip_suffix("]}").expect("generated file");
        for _ in 0..12_u32 {
            let bits = seed.next_u64();
            let shell = bits & 1 == 0;
            let (agents, legacy) = if bits & 2 == 0 { (false, "no") } else { (true, "yes") };
            let old = format!(
                r#"{base},{{"setting":{{"kind":"charter","number":5}},"overlay":{{"instructions":"Note {bits}","legacy-shell":{shell},"legacy-mode":{bits},"legacy-agents":"{legacy}","legacy-check":true}}}}]}}"#
            );
            let current = format!(
                r#"{base},{{"setting":{{"kind":"charter","number":5}},"overlay":{{"instructions":"Note {bits}","shell":{shell},"agents":{agents}}}}}]}}"#
            );
            let expected = read(current.as_bytes(), &limits).expect("generated items in current form");
            let moved = read_with(old.as_bytes(), &limits, TEST).expect("convert generated settings");
            assert_eq!(moved.format, 1);
            assert_eq!(moved.items, expected.items);
            assert_eq!(moved.notices.len(), 4);
            let upgraded = render_with(moved.items.as_slice(), &limits, TEST).expect("print newest format");
            let reread = read_with(&upgraded, &limits, TEST).expect("reread upgraded settings");
            assert_eq!(reread.format, 2);
            assert!(reread.notices.is_empty());
            assert_eq!(reread.items, expected.items);
        }
    }
}
