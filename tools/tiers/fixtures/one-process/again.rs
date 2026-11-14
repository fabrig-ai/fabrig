//! One-process reruns keep the plan's environment and process (design.md, 11.10).
use std::sync::atomic::{AtomicBool, Ordering};

static RAN: AtomicBool = AtomicBool::new(false);

#[test]
fn a_passes() {
    assert_eq!(std::env::var("TIERS_ONE_PROCESS").unwrap(), "1");
    assert_eq!(std::env::var("NEXTEST_PROFILE").unwrap(), "default");
    assert_eq!(std::env::var("CARGO_PKG_NAME").unwrap(), "fixture");
    assert_eq!(
        std::env::current_dir().unwrap(),
        std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
    );
    assert!(std::env::var_os("NEXTEST").is_none());
    RAN.store(true, Ordering::Relaxed);
}

#[test]
fn b_fails() {
    panic!("again fixture failure");
}

#[test]
fn c_reads() {
    assert!(RAN.load(Ordering::Relaxed), "a_passes must run in this process first");
}
