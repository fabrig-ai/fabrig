
use super::*;

#[test]
fn logical_calibration_requires_thirty_runs_but_one_suite_triplet_requires_three() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut suite = crate::read_suite(&root.join("suites/probes.toml")).expect("cache suite");
    suite.repetitions = 10;
    let result = crate::read_result(&root.join("tests/recorded/results/attempt.json")).expect("fixture");
    let requests: Vec<_> = (0..30)
        .map(|position| {
            let mut identity = result.identity.clone();
            identity.attempt = format!("{position:02}");
            Prepared {
                program: "/nonexecuted".into(),
                teardown_ms: 0,
                grace_ms: 1,
                settlement_ms: 1,
                ..crate::request_fixture::request(root.join("tasks/probes/cache-reuse/task.toml"), identity)
            }
        })
        .collect();
    assert_eq!(order(&suite, requests.clone(), 1, true).expect("ten triplets").len(), 30);
    assert!(order(&suite, requests[..29].to_vec(), 1, true).is_err());
    assert!(ordered(&suite, requests.clone(), 1).is_err(), "suite task's authored repetition count is three");
    assert_eq!(ordered(&suite, requests[..3].to_vec(), 1).expect("one triplet").len(), 3);
    suite.repetitions = 1;
    assert_eq!(order(&suite, requests[..3].to_vec(), 1, true).expect("one control triplet").len(), 3);
}
