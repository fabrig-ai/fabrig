//! Native libtest behaviours for tools/tiers/design.md, 11.10.
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};

// These fixtures exercise cancellation with the process's SIGTERM disposition.
unsafe extern "C" {
    fn signal(signum: i32, handler: usize) -> usize;
}

static TERMINATED: AtomicBool = AtomicBool::new(false);
extern "C" fn terminated(_: i32) {
    TERMINATED.store(true, Ordering::Relaxed);
}

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
    if scenario() == "descriptors" {
        let output = std::process::Command::new("python3")
            .args(["-I", "-B"])
            .arg(std::env::var_os("FIXTURE_DESCRIPTOR_CHECK").unwrap())
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    }
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
        assert_eq!(std::env::var("CARGO_BIN_EXE_helper").unwrap(), cwd.join("target/debug/helper").to_str().unwrap());
        assert_eq!(std::env::var("NEXTEST_BIN_EXE_helper").unwrap(), cwd.join("target/debug/helper").to_str().unwrap());
        assert!(std::env::var_os("CARGO_BIN_EXE_library").is_none());
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
    if matches!(
        scenario().as_str(),
        "starved" | "starved-panic" | "starved-nonzero" | "starved-return" | "starved-not-last" | "starved-blanks"
    ) {
        eprintln!("skein-world: starved: waited 1 ms runnable, fixture patience");
        if scenario() == "starved-panic" {
            panic!("output after starvation");
        }
        if scenario() == "starved-return" {
            return;
        }
        if scenario() == "starved-not-last" {
            eprintln!("output after starvation");
        }
        if scenario() == "starved-blanks" {
            eprintln!("\n   ");
        }
        std::process::exit(if scenario() == "starved-nonzero" { 3 } else { 0 });
    }
    if matches!(scenario().as_str(), "first-failure" | "term-ignore" | "failure-then-starved") {
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
        "term-pass" => {
            unsafe { signal(15, terminated as usize); }
            while !TERMINATED.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        "term-ignore" => {
            unsafe { signal(15, 1); }
            loop {
                std::thread::sleep(Duration::from_secs(60));
            }
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
    if scenario() == "failure-then-starved" {
        eprintln!("skein-world: starved: waited 1 ms runnable, fixture patience");
        std::process::exit(0);
    }
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
