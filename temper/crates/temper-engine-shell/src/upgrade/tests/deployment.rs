//! A stopped deployment copied from the floor store for upgrade command tests.
use std::{
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::Path,
};
use temper_shell::directories::Directories;

pub(super) fn copy(source: &Path, target: &Path) {
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

pub(crate) fn deployment() -> (skein_scratch::Scratch, Directories) {
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
        serde_json::from_slice(include_bytes!("../../../../../docs/examples/owned.json")).expect("owned example");
    document["workers"] = serde_json::json!([]);

    document["engine"]["hosts"]["listener"] = serde_json::json!([{"socket":"127.0.0.1:7103"}]);
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

pub(crate) fn tree(root: &std::path::Path) -> Vec<(std::path::PathBuf, u32, Vec<u8>)> {
    use std::os::unix::fs::PermissionsExt;
    let mut result = Vec::new();
    let mut entries =
        std::fs::read_dir(root).expect("data directory").map(|entry| entry.expect("data entry")).collect::<Vec<_>>();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let metadata = entry.metadata().expect("data metadata");
        let bytes = if metadata.is_file() { std::fs::read(entry.path()).expect("data file") } else { Vec::new() };
        result.push((entry.path(), metadata.permissions().mode(), bytes));
        if metadata.is_dir() {
            result.extend(tree(&entry.path()));
        }
    }
    result
}
