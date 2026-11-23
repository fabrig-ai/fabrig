use crate::{MutationBatch, count, mutation};
use std::{env, fs, io::ErrorKind, os::unix::fs::DirBuilderExt, path::PathBuf, process::Command};

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let base = env::temp_dir();
        for attempt in 0..1000 {
            let path = base.join(format!("skein-checks-{}-{attempt}", std::process::id()));
            match fs::DirBuilder::new().mode(0o700).create(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
                Err(error) => panic!("create check-count fixture {}: {error}", path.display()),
            }
        }
        panic!("no free check-count fixture name beneath {}", base.display());
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.0)
            && !std::thread::panicking()
        {
            panic!("remove check-count fixture {}: {error}", self.0.display());
        }
    }
}

#[test]
#[ignore = "selected by the isolated check-count regression"]
fn append_counts_in_child() {
    count("seed", 1);
    count("mutation", u64::MAX);
    let mut batch = MutationBatch::default();
    let mut candidates = [mutation(1), mutation(2), mutation(3)].into_iter();
    let selected = candidates.nth(1).expect("select exactly one candidate");
    assert_eq!(batch.judge(selected, |value| value), 2);
    assert_eq!(batch.judge(4.into(), |value: u32| value), 4);
    let pair = mutation(5).zip(mutation(6));
    assert_eq!(batch.judge(pair, |(left, right)| left + right), 11);
    batch.finish();
    MutationBatch::default().finish();
}

#[test]
#[ignore = "selected by the isolated check-count regression to exercise fail-stop"]
fn malformed_kind_in_child() {
    count("bad kind", 1);
}

#[test]
fn appends_counts_and_refuses_malformed_kinds() {
    let scratch = Scratch::new();
    let path = scratch.0.join("checks");
    for (mode, destination) in [
        ("unset", None),
        ("empty", Some("".as_ref())),
        ("valid", Some(path.as_os_str())),
        ("bad", Some(path.as_os_str())),
    ] {
        let test = if mode == "bad" { "tests::malformed_kind_in_child" } else { "tests::append_counts_in_child" };
        let mut child = Command::new(env::current_exe().expect("test binary"));
        child.args(["--exact", "--ignored", "--nocapture", test]).env_remove("TIERS_CHECKS_FILE");
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
            assert_eq!(fs::read_dir(&scratch.0).expect("scratch directory").count(), 0, "{mode}");
        } else {
            assert_eq!(
                fs::read_to_string(&path).expect("check counts"),
                "seed 2\nseed 1\nmutation 18446744073709551615\nmutation 3\n",
                "{mode}"
            );
        }
    }
}
