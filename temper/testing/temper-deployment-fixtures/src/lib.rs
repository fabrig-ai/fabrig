//! Loaded golden deployments for the simulated world and shell command tests.
mod frame_registry;
pub mod log;

pub use frame_registry::frame_registry_commit;
use std::{
    io::Write as _,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::Path,
};
use temper_shell::directories::Directories;

pub fn copy(source: &Path, target: &Path) {
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

#[must_use]
pub fn deployment() -> (skein_scratch::Scratch, Directories) {
    let scratch = skein_scratch::Scratch::new_in(&skein_scratch::paths::target_tmp(), "upgrade-deployment");
    let config = scratch.path().join("config");
    let data = scratch.path().join("data");
    let cache = scratch.path().join("cache");
    for directory in [&config, &data, &cache] {
        std::fs::create_dir(directory).expect("tool directory");
        std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700)).expect("private directory");
    }
    let floor = skein_scratch::paths::tree().join("tests/one-process/golden/floor");
    let state = data.join("state");
    copy(&floor.join("state"), &state);
    // Git omits empty directories; the manifest keeps the deployment's transcripts root.
    for line in std::fs::read_to_string(floor.join("modes.txt")).expect("floor modes").lines() {
        let fields: Vec<_> = line.split(' ').collect();
        if fields[0] == "d"
            && let Some(path) = fields[3].strip_prefix("state/")
        {
            let path = state.join(path);
            std::fs::create_dir_all(&path).expect("kept deployment directory");
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).expect("private directory");
        }
    }
    let active = std::fs::read(state.join("active")).expect("active generation");
    let path = state.join(log::initial_path(&active));
    let bytes = std::fs::read(&path).expect("copied store log");
    let commit = frame_registry_commit(&bytes);
    let mut log = std::fs::OpenOptions::new().append(true).open(path).expect("copied store log");
    log.write_all(&commit).expect("append frame registry to the copy");

    let mut file = temper_config_shell::File {
        listen: Some("127.0.0.1:7100".parse().expect("clients listener")),
        workers: Some("127.0.0.1:7103".parse().expect("workers listener")),
        owner: Some("a@b".into()),
        ..temper_config_shell::File::default()
    };
    let registration = serde_json::json!({
        "issuer":"http://127.0.0.1:7104", "client":"test-client", "scopes":["openid","read"],
        "authorization":"http://127.0.0.1:7104/authorize", "token":"http://127.0.0.1:7104/token",
        "keys":"http://127.0.0.1:7104/keys", "transport":"plaintext", "must_grant":["read"],
        "redirect_path":"callback"
    });
    let mut providers = temper_agent_fixtures::providers::stock();
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
    temper_agent_fixtures::providers::write(&data, &providers);
    file.test.github = Some(temper_config_shell::TestGitHub {
        api: "127.0.0.1:1".into(),
        transport: temper_config_shell::Transport::Plaintext,
        remote: "http://127.0.0.1:1".into(),
        web: "http://127.0.0.1:1".into(),
        git: Some("/usr/bin/git".into()),
    });
    std::fs::write(
        config.join(temper_config_shell::FILE),
        temper_config_shell::render(&file, temper_config_shell::Shape::EngineAlone),
    )
    .expect("configuration file");
    let directories = Directories {
        config: config.as_os_str().as_bytes().into(),
        data: data.as_os_str().as_bytes().into(),
        cache: cache.as_os_str().as_bytes().into(),
        origins: [skein_shell::Origin::Flag; 3],
    };
    (scratch, directories)
}

pub fn tree(root: &std::path::Path) -> Vec<(std::path::PathBuf, u32, Vec<u8>)> {
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
