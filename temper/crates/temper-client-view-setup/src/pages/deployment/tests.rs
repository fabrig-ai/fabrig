#![expect(clippy::disallowed_methods, reason = "ordinary tests inspect the limit row's semantic text")]
use super::*;
use crate::tests::fixtures as f;
use temper_client_protocol_wire::v1 as w;

#[test]
fn limit_rows_distinguish_the_stock_override_running_figure_and_wait() {
    let words = f::words();
    let long_reason = [b'r'; 120];
    let setting = words.say(
        Word::LimitsSetting,
        &[Hole::Number(16), Hole::Name(b"temper-limits".as_slice().into()), Hole::Name(long_reason.as_slice().into())],
    );
    assert!(setting.windows(long_reason.len()).any(|part| part == long_reason));
    for (name, value, waiting, applies, when) in [
        ("forge.entries", 16, false, w::LimitApplies::AtOnce, "Raises apply at once"),
        ("forge.entries", 32, true, w::LimitApplies::AtOnce, "Raises apply at once"),
        ("checkout.preparations", 16, false, w::LimitApplies::Live, "Changes apply live"),
    ] {
        let row = w::LimitRow {
            name: name.as_bytes().into(),
            line: b"Bounds concurrent work.".as_slice().into(),
            unit: w::LimitUnit::Count,
            stock: Some(8),
            setting: Some(w::LimitSetting {
                kind: w::LimitKind::Temper,
                value,
                reason: b"dev's capacity".as_slice().into(),
            }),
            in_force: Some(16),
            applies,
            waiting,
            left_out: false,
            held: None,
        };
        let mut builder = Builder::new(&f::tree_limits());
        limit_row(&mut builder, &words, &row);
        let tree = builder.finish();
        let raw = f::text(&tree);
        let text = core::str::from_utf8(&raw).unwrap();
        for expected in [name, "Stock: 8", "In force: 16", "temper-limits: dev's capacity", when, "count"] {
            assert!(text.contains(expected), "missing {expected}: {text}");
        }
        assert!(text.contains(if waiting {
            "32 (temper-limits: dev's capacity)"
        } else {
            "16 (temper-limits: dev's capacity)"
        }));
        assert_eq!(text.contains("Waits for the next start"), waiting);
    }
}
