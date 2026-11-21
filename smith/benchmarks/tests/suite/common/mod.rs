pub use smith_bench_fixtures::common::*;

#[cfg(target_os = "linux")]
use std::path::Path;

#[must_use]
pub fn unissued_auth() -> Vec<u8> {
    use base64::Engine;
    let claims = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b"{\"exp\":500}");
    serde_json::to_vec(&serde_json::json!({"auth_mode":"chatgpt","last_refresh":"2000-01-01T00:00:00Z",
        "tokens":{"id_token":"UNISSUED_ID","access_token":format!("header.{claims}.signature"),
        "account_id":"UNISSUED_ACCOUNT","refresh_token":"UNISSUED_REFRESH_MUST_NOT_COPY"}}))
    .expect("fixture")
}

#[cfg(target_os = "linux")]
#[must_use]
pub fn unexecuted_arm(directory: &Path, repository: &Path) -> smith_bench::arm::Build {
    use sha2::{Digest, Sha256};
    use smith_bench::arm::{ArmBinary, BuiltArm};
    // Replay fixtures select the product terminal invocation.
    let pin = smith_bench::read_configuration(&repository.join("benchmarks/agents/smith/standard.pin.toml"))
        .expect("standard Smith pin");
    let commit = pin.pin.minimum_commit.expect("minimum source identity");
    let ancestor = std::process::Command::new("git")
        .args(["-C", repository.to_str().expect("path"), "merge-base", "--is-ancestor", &commit, "HEAD"])
        .status()
        .expect("main ancestry");
    assert!(ancestor.success());
    let executable = b"unexecuted rendering fixture\n";
    let digest = format!("{:x}", Sha256::digest(executable));
    let path = directory.join(smith_bench::arm::arm_name(&commit, &digest).expect("fixture identity"));
    std::fs::create_dir(&path).expect("unexecuted arm directory");
    std::fs::write(path.join("smith"), executable).expect("unexecuted bytes");
    let arm = BuiltArm {
        version: 1,
        commit,
        sha256: digest.clone(),
        toolchain: "unexecuted test fixture".into(),
        binaries: vec![ArmBinary { name: "smith".into(), sha256: digest }],
    };
    std::fs::write(path.join("arm.json"), serde_json::to_vec(&arm).expect("manifest")).expect("fixture manifest");
    smith_bench::arm::read_build(&path.join("smith")).expect("manifest verification")
}
