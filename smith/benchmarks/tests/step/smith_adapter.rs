//! Replaying controls (smith/docs/design/testing.md, 2).

use smith_bench::Measure;
use std::path::{Path, PathBuf};

mod retried;

#[test]
fn every_smith_pin_is_a_product_settings_layer() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("agents/smith");
    for name in ["standard", "everything"] {
        let pin = smith_bench::read_configuration(&root.join(format!("{name}.pin.toml"))).expect("pin");
        assert_eq!(pin.pin.configuration, PathBuf::from(format!("{name}.toml")));
        let layer: toml::Value =
            toml::from_str(&std::fs::read_to_string(root.join(&pin.pin.configuration)).expect("TOML layer"))
                .expect("product TOML");
        assert_eq!(layer.as_table().expect("layer").keys().map(String::as_str).collect::<Vec<_>>(), ["trace"]);
        assert_eq!(layer["trace"]["capture"].as_str(), Some(if name == "standard" { "calls" } else { "everything" }));
    }
}

#[test]
fn a_time_breakdown_sums_to_task_wall_within_one_second() {
    let limits = {
        let mut fixture_limits = smith_events_fixtures::SMALL;
        fixture_limits.string = 524_288;
        fixture_limits.content = 4096;
        fixture_limits.items = 512;
        fixture_limits
    };
    let mut observer =
        smith_bench::agent::smith::events::Observer::with_ceiling(&limits, 1_048_576).expect("recorded stream bounds");
    observer.feed(include_bytes!("../recorded/smith-time.jsonl")).expect("recorded smith trace");
    observer.finish().expect("complete trace");
    let mut timing = smith_bench::parse_result(Path::new("fixture"), include_str!("../recorded/results/attempt.json"))
        .expect("result fixture")
        .timing;
    observer.timing(&mut timing);
    let record = &timing.record;
    let value = |measure: &Measure<u64>| match measure {
        Measure::Observed { value } => *value,
        other @ (Measure::LowerBound { .. } | Measure::Unavailable { .. }) => panic!("missing trace span: {other:?}"),
    };
    let sum = value(&record.requests_ms)
        + value(&record.compaction_ms)
        + value(&record.children_ms)
        + value(&record.rest_ms)
        + record.tools_ms.iter().map(|(_, span)| value(span)).sum::<u64>();
    assert!(sum.abs_diff(764_602) < 1000, "recorded task wall: {sum}");
    assert!((380_000..381_000).contains(&value(&record.requests_ms)));
    assert!((93_000..94_000).contains(&value(&record.compaction_ms)));
    assert!((32_000..33_000).contains(&value(&timing.first_byte_ms)));
    assert_eq!(timing.first_byte_ms, record.first_byte_ms);
    assert_eq!(timing.largest_gap_ms, record.largest_gap_ms);
    assert!(value(&timing.longest_completion_ms) > 10_000);
}
