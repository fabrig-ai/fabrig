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

#[test]
fn an_existing_scratch_root_is_refused_and_never_deleted() {
    use skein_fake_machine::Item;
    use smith_bench::files::Files;
    use smith_bench::guard::Guard;
    use smith_bench_fixtures::files::Tree;
    use std::path::Path;

    let template = include_str!("../../recorded/results/attempt.json");
    let mut identity = smith_bench::parse_result(Path::new("attempt.json"), template).expect("template").identity;
    identity.run = "existing".into();
    identity.attempt = "one".into();
    let prepared = testing::request("/task/task.toml".into(), identity);
    let tree = Tree::new(&[
        Item::directory(b"bench").mode(0o700),
        Item::directory(b"bench/scratch").mode(0o700),
        Item::directory(b"bench/scratch/existing-one").mode(0o700),
        Item::file(b"bench/scratch/existing-one/sentinel", b"keep"),
    ]);
    let guard =
        Guard::new_with(&tree, Path::new("/user"), Path::new("/config"), Path::new("/state"), &[]).expect("tree guard");
    let (incomplete, end) = testing::record_scratch_work_with(&tree, &guard, Path::new("/bench"), &prepared, |_, _| {
        panic!("attempt work must not run over a preexisting root")
    })
    .expect("record setup refusal");
    assert!(matches!(end, End::Refused { .. }));
    assert_eq!(incomplete, Incomplete::Setup(SetupStep::Scratch));
    assert_eq!(tree.read(Path::new("/bench/scratch/existing-one/sentinel")).expect("retained original"), b"keep");
    assert!(tree.writes().is_empty(), "no writes or deletion of the preexisting root");
}
