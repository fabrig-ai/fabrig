// temper_engine_domain::Fact (temper/crates/temper-engine-domain/src/boundary.rs) and jig_domain_tasks::Problem (jig/crates/jig-domain-tasks/src/boundary.rs) contain only numeric identities/capacities and closed tags, with no credential-bearing bytes.
#[test]
fn refused_and_replayed_pastes_keep_fake_issued_keys_off_public_surfaces() {
    let counts = temper_one_process_world::m15::refused_replayed_pastes();
    assert!(counts.checkpoints >= 2);
    assert!(counts.files > 0 && counts.logs > 0 && counts.client_files > 0);
    assert!(counts.wire_replies > 0 && counts.pages >= 8);
}
