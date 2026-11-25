use super::{BrowserStart, Host, Hosted, Time, Wall, kernel};
use crate::harness::{Harness, Names, Output, Printed, Sent, Setup};
use std::path::Path;

#[test]
fn chromium_waits_for_spawn_before_reporting_work() {
    let mut setup = Harness::new(Setup {
        server: "/test-server".into(),
        args: vec![],
        printed: Printed { sent: Sent::Page, prefix: b"listening ", on: Output::Stdout, names: Names::Address },
        restart: vec![],
        chromium: "/test-browser".into(),
        downloads: None,
        expected: vec![],
        origins: vec![],
    });
    // No ring enters: the spawn completions, and therefore pipes, are withheld.
    let start = BrowserStart::Chromium {
        profile: Path::new("/test-profile"),
        root: kernel::Fd::new(3),
        browser_root: kernel::Fd::new(4),
    };
    let mut run = Hosted::new(&mut setup, &mut [], &mut [], start);
    let now = Time::from_nanos(1);
    run.iterate(now, Wall::EPOCH);
    assert!(!run.submissions().is_empty(), "spawn operations were issued");
    assert!(!run.work_pending(now), "without pipes there is no browser work");
    assert!(run.next_deadline().is_some_and(|at| at > now), "the only wake is a future deadline or a completion");
}
