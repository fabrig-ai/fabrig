use super::fixtures;

#[test]
fn workspace_reuse_is_off_until_requested() {
    let mut stand = fixtures::system::one_process();
    assert!(!stand.service.worker_domain().expect("one worker").checkout().reuses_workspaces());
    assert!(!stand.service.worker.as_ref().expect("one worker").keeps_workspaces());
    stand.service.reuse_workspaces();
    assert!(stand.service.worker_domain().expect("one worker").checkout().reuses_workspaces());
    assert!(stand.service.worker.as_ref().expect("one worker").keeps_workspaces());
}

#[test]
fn workspace_reuse_on_an_engine_alone_does_nothing() {
    let mut service = crate::startup::with_setup(
        &fixtures::engine_alone_document(),
        &jig_protocol_config::Directories::working(),
        &fixtures::files(),
        None,
        &fixtures::seeds(),
        None,
        None,
        fixtures::forge_runtime(),
        None,
        None,
    )
    .expect("an engine alone starts");
    assert!(service.worker_domain().is_none());
    service.reuse_workspaces();
    assert!(service.worker_domain().is_none());
}
