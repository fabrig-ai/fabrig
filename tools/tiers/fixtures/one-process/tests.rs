//! Native libtest behaviours for tools/tiers/design.md, 11.10.
use std::time::Duration;

#[cfg(before_start)]
#[used]
#[unsafe(link_section = ".init_array")]
static BEFORE: extern "C" fn() = {
    extern "C" fn abort() {
        std::process::abort();
    }
    abort
};

fn scenario() -> String {
    std::env::var("FIXTURE_CASE").unwrap_or_default()
}

#[test]
fn a() {
    if scenario() == "unterminated" {
        print!("no newline");
    }
    if scenario() == "environment" {
        let cwd = std::env::current_dir().unwrap();
        assert_eq!(std::env::var("TIERS_ONE_PROCESS").unwrap(), "1");
        assert_eq!(std::env::var("NEXTEST_PROFILE").unwrap(), "default");
        assert!(std::env::var_os("NEXTEST").is_none());
        assert_eq!(std::env::var("CARGO_MANIFEST_DIR").unwrap(), cwd.to_str().unwrap());
        assert_eq!(std::env::var("CARGO_PKG_NAME").unwrap(), "fixture");
        assert_eq!(std::env::var("CARGO_BIN_EXE_helper").unwrap(), cwd.join("helper").to_str().unwrap());
        assert_eq!(std::env::var("NEXTEST_BIN_EXE_helper").unwrap(), cwd.join("helper").to_str().unwrap());
        assert_eq!(std::env::var("OUT_DIR").unwrap(), cwd.join("out").to_str().unwrap());
        assert!(
            std::env::split_paths(&std::env::var_os("LD_LIBRARY_PATH").unwrap())
                .any(|p| p == cwd.join("target/debug/deps"))
        );
        assert_eq!(std::env::var("CONFIGURED").unwrap(), "forced");
        assert_eq!(std::env::var("GLOBAL_CONFIGURED").unwrap(), "global");
        assert_eq!(std::env::var("KEPT").unwrap(), "caller");
        assert_eq!(std::env::var("RELATIVE").unwrap(), cwd.join("relative").to_str().unwrap());
    }
    if scenario() == "leak" {
        std::process::Command::new("sleep").arg("30").spawn().unwrap();
    }
    if scenario() == "starved" || scenario() == "starved-panic" {
        eprintln!("skein-world: starved: waited 1 ms runnable, fixture patience");
        if scenario() == "starved-panic" {
            panic!("output after starvation");
        }
        std::panic::resume_unwind(Box::new(()));
    }
    if scenario() == "first-failure" {
        panic!("first failure");
    }
    if scenario() == "slow" {
        std::thread::sleep(Duration::from_millis(80));
    }
}

#[test]
fn b() {
    match scenario().as_str() {
        "crash" => {
            eprintln!("crash output");
            std::process::abort();
        }
        "hang" => loop {
            std::thread::sleep(Duration::from_secs(60));
        },
        "slow" => std::thread::sleep(Duration::from_millis(80)),
        _ => (),
    }
}

#[test]
fn c() {
    if scenario() == "slow" {
        std::thread::sleep(Duration::from_millis(80));
    }
}
#[test]
fn d() {}
#[test]
#[ignore]
fn ignored() {}

#[test]
#[should_panic(expected = "expected panic")]
fn expected_panic() {
    panic!("expected panic");
}
