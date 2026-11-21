//! Real-file configuration controls and the shipped smith check (smith/docs/design/testing.md, 2.3).

pub use smith_end_to_end_kit as support;
pub use support::live;

mod live_tests;

#[test]
fn the_refresh_report_names_facts_without_token_bytes() {
    use skein_lib::Wall;
    use smith_local_shell::credentials::{RefreshFacts, env_record, refresh_facts, token_limits};
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    const ACCESS: &str = "synthetic-refresh-report-access";
    const REFRESH: &str = "synthetic-refresh-report-refresh";
    let scratch = skein_scratch::Scratch::new("smith-refresh-report");
    let now = Wall::from_nanos(10_000_500_000_000);
    let mut record = env_record(ACCESS.as_bytes(), now).expect("synthetic record").record;
    record.generation = 7;
    record.refresh_token = Some(skein_lib::Secret::new(REFRESH.as_bytes().into()));
    record.expires_at = Wall::from_nanos(13_127_900_000_000);
    record.earliest_refresh_at = Some(Wall::from_nanos(11_840_900_000_000));
    let bytes = skein_oauth::encode_record(&record, &token_limits()).expect("bounded synthetic record");
    let path = scratch.path().join("record");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)
        .expect("private synthetic record");
    file.write_all(&bytes).expect("write synthetic record");
    let before = refresh_facts(&path).expect("readable synthetic record").expect("synthetic record exists");
    let after = RefreshFacts {
        generation: 8,
        refreshable: false,
        expires_at: Wall::from_nanos(9_988_900_000_000),
        earliest_refresh_at: None,
    };
    let report = live::refresh_report(&before, &after, now);
    assert_eq!(
        report,
        "record before: generation=7 refreshable=true expires_in=3127s earliest_in=1840s; after: generation=8 refreshable=false expires_in=-11s earliest_in=none",
        "the report names only fixed words and whole seconds relative to the story clock"
    );
    for token in [ACCESS, REFRESH] {
        assert!(!report.contains(token), "token bytes stay out of the refresh report");
    }
}
