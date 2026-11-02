//! A kept forge is reconciled by the service (temper/docs/design/protocol/forge.md, 2.6).
#[test]
fn reconcile_directs_to_the_service_without_creating_forge_files() {
    let scratch = skein_scratch::Scratch::new("forge-reconcile-service");
    let (_, bytes) = crate::tests::configuration::documents(scratch.path(), "/fixture/temper");
    std::fs::write(scratch.path().join("temper.json"), bytes).expect("document");
    let refused = super::start(
        &crate::tests::configuration::directories(scratch.path()),
        &["reconcile".into(), "1".into()],
        None,
    )
    .expect_err("service owns reconciliation");
    assert_eq!(
        refused.line(),
        "temper forge reconcile: forge 1: forge 1 is kept by the service: connect it from the client\n"
    );
    assert!(!scratch.path().join("forge-1").exists());
}
