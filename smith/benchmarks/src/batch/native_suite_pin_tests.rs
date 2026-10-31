use super::*;
use sha2::Digest;

#[test]
fn native_built_suite_pin_verifies_full_arm_commit_and_binary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pin_file = root.join("agents/smith/everything.pin.toml");
    let pin = crate::read_configuration(&pin_file).expect("native built-arm pin");
    let task_file = root.join("tasks/probes/first-request/task.toml");
    let task = crate::read_task(&task_file).expect("frozen author input");
    let mut suite = crate::read_suite(&root.join("suites/calibrate.toml")).expect("suite");
    suite.tasks = vec!["probes/first-request".into()];
    let mut identity =
        crate::read_result(&root.join("tests/recorded/results/attempt.json")).expect("identity fixture").identity;
    let commit = "a".repeat(40);
    let bytes = b"synthetic identity fixture; never executed\n";
    let digest = format!("{:x}", sha2::Sha256::digest(bytes));
    let scratch = skein_scratch::Scratch::new("bench-batch");
    let directory = scratch
        .path()
        .to_path_buf()
        .join(format!("smith-native-suite-pin-{}", std::process::id()))
        .join(crate::arm::arm_name(&commit, &digest).expect("arm name"));
    fs::create_dir_all(&directory).expect("isolated identity fixture");
    fs::write(directory.join("smith"), bytes).expect("non-executed fixture");
    let arm = crate::arm::BuiltArm {
        version: 1,
        commit: commit.clone(),
        sha256: digest.clone(),
        toolchain: "synthetic identity control".into(),
        binaries: vec![crate::arm::ArmBinary { name: "smith".into(), sha256: digest.clone() }],
    };
    fs::write(directory.join("arm.json"), serde_json::to_vec(&arm).expect("arm manifest"))
        .expect("private test manifest");
    identity.agent = crate::Agent::Smith;
    identity.provider = crate::Provider::Codex;
    identity.agent_version = commit;
    identity.binary_sha256 = digest.clone();
    identity.configuration_sha256 = pin.sha256.clone();
    identity.task = "probes/first-request".into();
    identity.task_version = task.version;
    identity.seed_sha256 = task.seed_sha256;
    identity.model.requested_model = "gpt-6-luna".into();
    identity.model.requested_effort = "low".into();
    let mut request = Request {
        program: directory.join("smith"),
        configuration: Some(pin_file),
        teardown_ms: 1,
        grace_ms: 1,
        settlement_ms: 1,
        max_stdout_bytes: 1,
        max_stderr_bytes: 1,
        disk_estimate_bytes: 1,
        ..crate::request_fixture::request(task_file, identity)
    };
    let suite_file = root.join("suites/calibrate.toml");
    let valid = check_requests(&suite_file, &suite, std::slice::from_ref(&request));
    // Do not execute or read credentials; this is the real admission seam.
    assert!(valid.is_ok(), "verified native commit must match built-arm pin: {valid:?}");
    request.identity.binary_sha256 = "0".repeat(64);
    assert_eq!(configuration_matches(&pin, &request), Ok(false), "changed binary identity refused by admission");
    request.identity.binary_sha256 = digest;
    unknown_model_admission(root, &mut request);
}

fn unknown_model_admission(root: &Path, request: &mut Request) {
    let suite_file = root.join("suites/probes.toml");
    let mut suite = probe_suite(&suite_file, "probes/failures/unknown-model", 1).expect("failure suite");
    suite.tasks = vec!["probes/failures/unknown-model".into()];
    let pin_file = root.join("agents/smith/everything.pin.toml");
    let pin = crate::read_configuration(&pin_file).expect("failure pin");
    request.task = root.join("tasks/probes/failures/task.toml");
    let task = crate::read_task(&request.task).expect("failure task").with_variant("unknown-model").expect("variant");
    request.configuration = Some(pin_file);
    request.identity.configuration_sha256 = pin.sha256;
    request.identity.task = "probes/failures/unknown-model".into();
    request.identity.task_version = task.version;
    request.identity.seed_sha256 = task.seed_sha256;
    // The closed admission seam refuses a tier-labelled secret override.
    assert!(check_requests(&suite_file, &suite, std::slice::from_ref(request)).is_err());
    request.identity.model.requested_model = "codex/smith-bench-nonexistent-model-2026-10-11".into();
    let valid = check_requests(&suite_file, &suite, std::slice::from_ref(request));
    assert!(valid.is_ok(), "truthful authored provider rejection identity admitted: {valid:?}");
}
