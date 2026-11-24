//! Offline command controls use synthetic credentials and an unissued replay arm.

use crate::common;

use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root(name: &str) -> skein_scratch::Scratch {
    skein_scratch::Scratch::new(name)
}

// The probe's plan reads /proc, its attempts use GNU cp --reflink and /proc/self, and its arm needs source history.
#[cfg(target_os = "linux")]
#[test]
fn a_replay_arm_runs_through_the_probe_command_and_prints_one_line() {
    let scratch = root("replay");
    let directory = scratch.path().join("root");
    let arm = replay_arm(&directory);
    let shared = directory.join("shared-tmp");
    std::fs::create_dir(&shared).expect("shared temporary directory");
    let login = directory.join("state/smith-bench/logins/smith");
    std::fs::create_dir_all(login.join("config/smith")).expect("synthetic configuration");
    std::fs::write(login.join("config/smith/config.toml"), []).expect("setup marker");
    std::fs::write(login.join("host-id"), []).expect("host marker");
    std::fs::write(login.join("tokens/chatgpt"), []).expect("record marker");
    let output = Command::new(env!("CARGO_BIN_EXE_smith-bench"))
        .args(["probe", "probes/smoke-tools", "--arm"])
        .arg(&arm)
        .arg("--shared-temporary")
        .arg(&shared)
        .env("HOME", &directory)
        .env("XDG_CONFIG_HOME", directory.join("config"))
        .env("XDG_STATE_HOME", directory.join("state"))
        .output()
        .expect("probe process");
    let text = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        text.lines().filter(|line| line.contains("/01 end=completed ")).count(),
        1,
        "{text} {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let verdict = match text.lines().last().expect("summary").split("verdict=").last().expect("verdict") {
        "pass" => 0,
        "fail" => 1,
        "incomplete" => 2,
        other => panic!("unknown verdict {other}"),
    };
    assert_eq!(output.status.code(), Some(verdict));
    assert!(login.join("tokens/chatgpt").is_file());
}

#[cfg(target_os = "linux")]
fn replay_arm(directory: &Path) -> PathBuf {
    use sha2::{Digest, Sha256};
    use std::os::unix::fs::PermissionsExt;
    std::fs::DirBuilder::new().mode(0o700).create(directory).expect("private replay root");
    std::fs::create_dir_all(directory.join("state/smith-bench/logins/smith/tokens")).expect("synthetic home");
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("repository");
    let mut build = common::unexecuted_arm(directory, repository);
    let settings = build.directory.join("recording.jsonl");
    std::fs::write(&settings, common::events::bytes(&common::events::balanced())).expect("wire recording");
    let executable = std::fs::read(env!("CARGO_BIN_EXE_test-peer")).expect("Rust replay peer");
    let program = build.directory.join("smith");
    std::fs::write(&program, &executable).expect("replay executable");
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).expect("execute");
    let digest = format!("{:x}", Sha256::digest(&executable));
    build.arm.sha256.clone_from(&digest);
    build.arm.binaries[0].sha256.clone_from(&digest);
    std::fs::write(build.directory.join("arm.json"), serde_json::to_vec(&build.arm).expect("manifest"))
        .expect("manifest");
    let arm = directory.join(smith_bench::arm::arm_name(&build.arm.commit, &digest).expect("arm name"));
    std::fs::rename(&build.directory, &arm).expect("rename frozen arm");
    arm
}

#[test]
fn a_missing_smoke_login_refuses_before_calibration() {
    let scratch = root("missing-smoke-login");
    let directory = scratch.path().join("root");
    std::fs::DirBuilder::new().mode(0o700).create(&directory).expect("private root");
    let output = Command::new(env!("CARGO_BIN_EXE_smith-bench"))
        .args(["suite", "smoke"])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", &directory)
        .env("XDG_CONFIG_HOME", directory.join("config"))
        .env("XDG_STATE_HOME", directory.join("state"))
        .output()
        .expect("offline smoke refusal");
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8(output.stderr).expect("diagnostic").trim(),
        format!(
            "no smith login for the harness: run `XDG_CONFIG_HOME={} smith login chatgpt`",
            directory.join("state/smith-bench/logins/smith/config").display()
        )
    );
}
