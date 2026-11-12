//! The same empty-deployment boundary for the two upgrade binaries.
use std::os::unix::fs::PermissionsExt;
pub fn empty_directory(binary: &str) {
    let scratch = skein_scratch::Scratch::new("upgrade-empty");
    let config = scratch.path().join("config");
    let data = scratch.path().join("data");
    let cache = scratch.path().join("cache");
    std::fs::create_dir(&config).expect("configuration directory");
    std::fs::create_dir(&data).expect("empty stopped deployment directory");
    let mut document: serde_json::Value = serde_json::from_slice(include_bytes!("../../../../docs/examples/owned.json")).expect("example JSON");
    document["engine"]["llm"]["providers"][0]["sources"].as_object_mut().expect("sources").remove("sign_in");
    document["engine"]["llm"]["endpoints"][0].as_object_mut().expect("endpoint").remove("server_name");
    document["engine"]["llm"]["endpoints"][0]["address"] = serde_json::json!({"socket":"127.0.0.1:1"});
    document["engine"]["llm"]["endpoints"][0]["transport"] = "plaintext".into();
    document["engine"]["llm"]["providers"][0]["signed_in_plan"] = false.into();
    document["engine"]["deployment"]["directory"] = data.to_str().expect("fixture path").into();
    document["workers"] = serde_json::json!([]);
    document["application"]["workers"] = serde_json::json!([]);
    std::fs::create_dir(config.join("secrets")).expect("fixture secrets directory");
    let digest = config.join("secrets/worker.digest");
    document["engine"]["hosts"]["workers"][0]["digest"] = digest.to_str().expect("fixture path").into();
    std::fs::write(&digest, "0".repeat(64)).expect("fixture digest");
    std::fs::set_permissions(&digest, std::fs::Permissions::from_mode(0o600)).expect("private digest");
    document["engine"]["hosts"]["listener"] = serde_json::json!([{"socket":"127.0.0.1:7102"}]);
    std::fs::write(config.join("temper.json"), serde_json::to_vec(&document).expect("document bytes")).expect("configuration");
    for mode in ["--check", "--back"] {
        let mut command = std::process::Command::new(skein_scratch::paths::binary(binary));
        command.args(["upgrade", mode]).env_clear().env("TEMPER_CONFIG", &config).env("TEMPER_DATA", &data).env("TEMPER_CACHE", &cache).stdin(std::process::Stdio::null());
        if mode == "--check" { command.arg(&data); }
        let output = command.output().expect("run upgrade");
        let out = String::from_utf8_lossy(&output.stdout);
        let err = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(1), "{out}\n{err}");
        assert!(err.is_empty(), "{err}");
        let refusal = if mode == "--check" { "does not name a generation" } else { "deployment could not be read" };
        assert!(out.contains(refusal), "{out}");
        assert!(std::fs::read_dir(&data).expect("source listing").next().is_none(), "no deployment was created");
    }
}
