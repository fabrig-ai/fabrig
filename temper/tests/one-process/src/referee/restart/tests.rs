use crate::{fixtures, referee::restart};

#[test]
fn a_restart_showing_part_of_a_commit_fails_the_referee() {
    let records = fixtures::two_forge_records();
    let uncertain: restart::Records =
        records.iter().map(|row| (temper_engine_domain_forge::stored_key(row), row.clone())).collect();
    let partial: restart::Records =
        records.iter().take(1).map(|row| (temper_engine_domain_forge::stored_key(row), row.clone())).collect();
    assert_eq!(
        restart::check(&restart::Records::new(), Some(&uncertain), &partial),
        Err(temper_engine_domain_forge::Key::Names(2))
    );
}
