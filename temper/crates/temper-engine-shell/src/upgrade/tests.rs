//! The upgrade command on a deployment copy (protocol/binaries.md, 1).
use super::{Request, run};
use std::{
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::Path,
};
use temper_shell::directories::Directories;

// A SHA-256 digest has 64 hexadecimal digits.
const DIGEST_BYTES: usize = 64;

fn copy(source: &Path, target: &Path) {
    std::fs::create_dir(target).expect("copy directory");
    std::fs::set_permissions(target, std::fs::Permissions::from_mode(0o700)).expect("private directory");
    for entry in std::fs::read_dir(source).expect("source directory") {
        let entry = entry.expect("source entry");
        let target = target.join(entry.file_name());
        if entry.file_type().expect("entry type").is_dir() {
            copy(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("copy fixture file");
            std::fs::set_permissions(target, std::fs::Permissions::from_mode(0o600)).expect("private file");
        }
    }
}

fn deployment() -> (skein_scratch::Scratch, Directories) {
    let scratch = skein_scratch::Scratch::new_in(&skein_scratch::paths::target_tmp(), "upgrade-deployment");
    let config = scratch.path().join("config");
    let data = scratch.path().join("data");
    let cache = scratch.path().join("cache");
    for directory in [&config, &cache] {
        std::fs::create_dir(directory).expect("tool directory");
    }
    copy(&skein_scratch::paths::tree().join("tests/one-process/golden/baseline/state"), &data);
    let mut document: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../../../docs/examples/owned.json")).expect("owned example");
    document["workers"] = serde_json::json!([]);
    document["application"]["workers"] = serde_json::json!([]);
    document["engine"]["deployment"]["directory"] = data.to_str().expect("data path").into();
    document["engine"]["hosts"]["listener"] = serde_json::json!([{"socket":"127.0.0.1:7103"}]);
    let mut workers = Vec::new();
    for (name, digit) in [("worker-a", '0'), ("worker-b", '1'), ("worker-c", '2')] {
        let digest = config.join(format!("{name}.digest"));
        std::fs::write(&digest, digit.to_string().repeat(DIGEST_BYTES)).expect("fixture digest");
        std::fs::set_permissions(&digest, std::fs::Permissions::from_mode(0o600)).expect("private digest");
        workers.push(serde_json::json!({"name":name,"digest":digest}));
    }
    document["engine"]["hosts"]["workers"] = workers.into();
    let registration = serde_json::json!({
        "issuer":"http://127.0.0.1:7104", "client":"test-client", "scopes":["openid","read"],
        "authorization":"http://127.0.0.1:7104/authorize", "token":"http://127.0.0.1:7104/token",
        "keys":"http://127.0.0.1:7104/keys", "transport":"plaintext", "must_grant":["read"],
        "redirect_path":"callback"
    });
    document["engine"]["llm"]["providers"][0]["sources"]["sign_in"] = registration;
    document["engine"]["llm"]["providers"][0]["manage_usage"] = "https://accounts.example.invalid/usage".into();
    let endpoint = &mut document["engine"]["llm"]["endpoints"][0];
    endpoint.as_object_mut().expect("endpoint").remove("server_name");
    endpoint["address"] = serde_json::json!({"socket":"127.0.0.1:1"});
    endpoint["transport"] = "plaintext".into();
    let mut endpoint = endpoint.clone();
    endpoint["name"] = "second".into();
    endpoint["number"] = 2.into();
    endpoint["provider"] = "second".into();
    document["engine"]["llm"]["endpoints"].as_array_mut().expect("endpoints").push(endpoint);
    let mut provider = document["engine"]["llm"]["providers"][0].clone();
    provider["name"] = "second".into();
    provider["number"] = 2.into();
    provider["endpoint"] = "second".into();
    document["engine"]["llm"]["providers"].as_array_mut().expect("providers").push(provider);
    document["application"]["machine"]["github"] = serde_json::json!({
        "api":{"socket":"127.0.0.1:1"}, "transport":"plaintext",
        "remote":"http://127.0.0.1:1", "web":"http://127.0.0.1:1", "git":"/usr/bin/git"
    });
    std::fs::write(config.join("temper.json"), serde_json::to_vec(&document).expect("configuration bytes"))
        .expect("configuration file");
    let directories = Directories {
        config: config.as_os_str().as_bytes().into(),
        data: data.as_os_str().as_bytes().into(),
        cache: cache.as_os_str().as_bytes().into(),
        origins: [skein_shell::Origin::Flag; 3],
    };
    (scratch, directories)
}

#[test]
fn deployment_rehearsal_with_accounts_workers_and_forge_reaches_its_report() {
    let (_scratch, directories) = deployment();
    let code = run("temper-engine", &directories, &Request::Check(directories.data.clone()))
        .expect("upgrade reaches its report");
    assert_eq!(code, 0, "golden patches and restores pass");
}

#[test]
fn deployment_back_with_accounts_workers_and_forge_reports_no_previous_generation() {
    let (_scratch, directories) = deployment();
    let code = run("temper-engine", &directories, &Request::Back).expect("back reaches its report");
    assert_eq!(code, 1, "baseline has no previous generation");
}
