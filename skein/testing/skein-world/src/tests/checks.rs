use crate::checks::count;
use skein_scratch::Scratch;
use std::{env, fs, process::Command};

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "a fresh test process selects its fixture without changing its environment"
)]
fn appends_counts_and_refuses_malformed_kinds() {
    const CHILD: &str = "SKEIN_CHECK_COUNTS_CHILD";
    const TEST: &str = "tests::checks::appends_counts_and_refuses_malformed_kinds";
    if let Some(mode) = env::var_os(CHILD) {
        if mode == "bad" {
            count("bad kind", 1);
            return;
        }
        count("seed", 1);
        count("mutation", u64::MAX);
        return;
    }

    let scratch = Scratch::new("check-counts");
    let path = scratch.path().join("checks");
    for (mode, destination) in [
        ("unset", None),
        ("empty", Some("".as_ref())),
        ("valid", Some(path.as_os_str())),
        ("bad", Some(path.as_os_str())),
    ] {
        let mut child = Command::new(env::current_exe().expect("test binary"));
        child.args(["--exact", TEST, "--nocapture"]).env(CHILD, mode).env_remove("TIERS_CHECKS_FILE");
        if let Some(destination) = destination {
            child.env("TIERS_CHECKS_FILE", destination);
        }
        if mode == "valid" {
            fs::write(&path, "seed 2\n").expect("prior process counts");
        }
        let result = child.output().expect("run isolated count test");
        if mode == "bad" {
            assert!(!result.status.success(), "malformed count kind must fail");
            assert!(String::from_utf8_lossy(&result.stderr).contains("invalid check kind"), "{result:?}");
        } else {
            assert!(result.status.success(), "{result:?}");
        }
        if destination.is_none_or(std::ffi::OsStr::is_empty) {
            assert_eq!(fs::read_dir(scratch.path()).expect("scratch directory").count(), 0, "{mode}");
        } else {
            assert_eq!(
                fs::read_to_string(&path).expect("check counts"),
                "seed 2\nseed 1\nmutation 18446744073709551615\n",
                "{mode}"
            );
        }
    }
}
