//! Replaying controls (smith/docs/design/testing.md, 2).

use crate::common;
use std::fmt::Write;

#[test]
fn oversized_calls_exceed_the_bound_and_repairs_fit() {
    let task = common::task("probes/large-write");
    let variant = task.with_variant("oversized").expect("small-payload variant");
    let lines = |first, last| {
        let mut result = String::new();
        for index in first..=last {
            writeln!(&mut result, "Line {index:04}: The quick brown fox jumps over the lazy dog.").expect("line");
        }
        result
    };
    let first = lines(1, 280);
    let input = serde_json::to_vec(&serde_json::json!({"path":"payload.txt","content":first})).expect("JSON framing");
    assert_eq!(input.len(), 15995);
    assert!(input.len() * 100 >= 13848 * 115);
    assert_eq!(lines(1, 60).len(), 3360);
    assert_eq!(lines(1, 30).len(), 1680);
    assert!(lines(30, 60).len() <= 2048);
    assert!(!variant.deciding.iter().any(|name| name == "file-digest:payload.txt"));
}
