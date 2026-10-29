// temper_engine_domain::Fact (temper/crates/temper-engine-domain/src/boundary.rs) and jig_domain_tasks::Problem (jig/crates/jig-domain-tasks/src/boundary.rs) contain only numeric identities/capacities and closed tags, with no credential-bearing bytes.
#[test]
fn refused_and_replayed_pastes_keep_fake_issued_keys_off_public_surfaces() {
    let counts = temper_one_process_world::m15::refused_replayed_pastes();
    assert!(counts.tokens >= 2);
    assert_eq!(counts.checkpoints, 3);
    assert!(counts.files > 0 && counts.logs > 0 && counts.client_files > 0);
    assert!(counts.wire_replies > 0 && counts.pages >= 8);
}

#[test]
fn installation_tokens_stay_out_of_saved_state_pages_context_and_delivered_git_files() {
    let counts = temper_one_process_world::m15::delivery(false);
    assert_eq!(counts.tokens, 2);
    assert_eq!(counts.checkpoints, 4);
    assert!(counts.contexts >= 5);
    assert!(counts.files > 0 && counts.logs > 0 && counts.transcripts > 0);
    assert!(counts.client_files > 0 && counts.worker_files > 0);
    assert!(counts.wire_replies > 0 && counts.pages >= 11);
}
#[test]
fn whole_run_and_renewal_scan_every_fake_issued_token_byte() {
    let counts = temper_one_process_world::m15::delivery(true);
    assert_eq!(counts.tokens, 4);
    assert_eq!(counts.checkpoints, 5);
    assert!(counts.contexts >= 6);
    assert!(counts.files > 0 && counts.logs > 0 && counts.transcripts > 0);
    assert!(counts.client_files > 0 && counts.worker_files > 0);
    assert!(counts.wire_replies > 0 && counts.pages >= 11);
}
