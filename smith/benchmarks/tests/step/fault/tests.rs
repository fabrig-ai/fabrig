use smith_bench::{End, FaultKind, isolate::SetupStep, run::Incomplete, testing};

#[test]
fn a_refused_seed_names_the_seed_step() {
    let fault = testing::verify_seed_digest(Ok("1".repeat(64)), &"0".repeat(64)).expect_err("wrong seed digest");
    assert_eq!(fault.kind, FaultKind::Setup(SetupStep::Seed));
}

#[test]
fn a_refused_attempt_keeps_the_seed_step() {
    let template = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/recorded/results/attempt.json");
    let mut identity = smith_bench::read_result(&template).expect("template").identity;
    identity.run = "refused-seed".into();
    identity.attempt = "one".into();
    let prepared = testing::request("task.toml".into(), identity);
    let fault = testing::verify_seed_digest(Ok("1".repeat(64)), &"0".repeat(64)).expect_err("wrong seed digest");
    let (incomplete, end) = testing::record_preparation_refusal(&prepared, fault);
    assert_eq!(incomplete, Incomplete::Setup(SetupStep::Seed));
    let End::Refused { setup } = end else { panic!("setup refusal") };
    assert!(setup.contains("seed digest differs"), "{setup}");
}
