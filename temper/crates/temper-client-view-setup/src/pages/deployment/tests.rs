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
    for (value, waiting) in [(16, false), (32, true)] {
        let row = w::LimitRow {
                name: b"forge.entries".as_slice().into(),
                line: b"Bounds the forge outbox.".as_slice().into(),
                unit: w::LimitUnit::Count,
                stock: Some(8),
                setting: Some(
                    w::LimitSetting {
                            kind: w::LimitKind::Temper,
                            value,
                            reason: b"dev's outbox".as_slice().into(),
                        },
                ),
                in_force: Some(16),
                applies: w::LimitApplies::AtOnce,
                waiting,
                left_out: false,
                held: None,
            };
        let mut builder = Builder::new(&f::tree_limits());
        limit_row(&mut builder, &words, &row);
        let tree = builder.finish();
        let raw = f::text(&tree);
        let text = core::str::from_utf8(&raw).unwrap();
        for expected in [
            "forge.entries",
            "Stock: 8",
            "In force: 16",
            "temper-limits: dev's outbox",
            "Raises apply at once",
            "count",
        ] {
            assert!(text.contains(expected), "missing {expected}: {text}");
        }
        assert!(text.contains(if waiting {
            "32 (temper-limits: dev's outbox)"
        } else {
            "16 (temper-limits: dev's outbox)"
        }));
        assert_eq!(text.contains("Waits for the next start"), waiting);
    }
}
