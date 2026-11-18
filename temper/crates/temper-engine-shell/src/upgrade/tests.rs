//! The upgrade command on a deployment copy (protocol/binaries.md, 1).
use super::{Request, run};
use std::{
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::Path,
};
use temper_shell::directories::Directories;

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
    for directory in [&config, &data, &cache] {
        std::fs::create_dir(directory).expect("tool directory");
    }
    let floor = skein_scratch::paths::tree().join("tests/one-process/golden/floor");
    copy(&floor.join("state"), &data.join("state"));
    // Git omits empty directories; the manifest keeps the deployment's transcripts root.
    for line in std::fs::read_to_string(floor.join("modes.txt")).expect("floor modes").lines() {
        let fields: Vec<_> = line.split(' ').collect();
        if fields[0] == "d"
            && let Some(path) = fields[3].strip_prefix("state/")
        {
            let path = data.join("state").join(path);
            std::fs::create_dir_all(&path).expect("kept deployment directory");
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).expect("private directory");
        }
    }
    let mut document: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../../../docs/examples/owned.json")).expect("owned example");
    document["workers"] = serde_json::json!([]);

    document["engine"]["hosts"]["listener"] = serde_json::json!([{"socket":"127.0.0.1:7103"}]);
    let registration = serde_json::json!({
        "issuer":"http://127.0.0.1:7104", "client":"test-client", "scopes":["openid","read"],
        "authorization":"http://127.0.0.1:7104/authorize", "token":"http://127.0.0.1:7104/token",
        "keys":"http://127.0.0.1:7104/keys", "transport":"plaintext", "must_grant":["read"],
        "redirect_path":"callback"
    });
    let mut providers = temper_fixtures::providers::stock();
    providers["providers"].as_array_mut().expect("providers").truncate(1);
    providers["endpoints"].as_array_mut().expect("endpoints").truncate(1);
    providers["providers"][0]["sources"]["sign_in"] = registration;
    providers["providers"][0]["manage_usage"] = "https://accounts.example.invalid/usage".into();
    let endpoint = &mut providers["endpoints"][0];
    endpoint.as_object_mut().expect("endpoint").remove("server_name");
    endpoint["address"] = serde_json::json!({"socket":"127.0.0.1:1"});
    endpoint["transport"] = "plaintext".into();
    let mut endpoint = endpoint.clone();
    endpoint["name"] = "second".into();
    endpoint["number"] = 2.into();
    endpoint["provider"] = "second".into();
    providers["endpoints"].as_array_mut().expect("endpoints").push(endpoint);
    let mut provider = providers["providers"][0].clone();
    provider["name"] = "second".into();
    provider["number"] = 2.into();
    provider["endpoint"] = "second".into();
    providers["providers"].as_array_mut().expect("providers").push(provider);
    temper_fixtures::providers::write(&data, &providers);
    document["application"]["test"]["github"] = serde_json::json!({
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
    let active_path = Path::new(std::ffi::OsStr::from_bytes(&directories.data)).join("state/active");
    let active = std::fs::read(&active_path).expect("floor active generation");
    let code = run(
        "temper-engine",
        &directories,
        &Request::Check([directories.data.as_ref(), b"/state"].concat().into()),
        "test-build",
    )
    .expect("upgrade reaches its report");
    assert_eq!(code, 0, "the floor rehearses through restores");
    assert_eq!(std::fs::read(active_path).expect("active after rehearsal"), active);
}

#[test]
fn deployment_back_with_accounts_workers_and_forge_reports_no_previous_generation() {
    let (_scratch, directories) = deployment();
    let code = run("temper-engine", &directories, &Request::Back, "test-build").expect("back reaches its report");
    assert_eq!(code, 1, "floor has no previous generation");
}
