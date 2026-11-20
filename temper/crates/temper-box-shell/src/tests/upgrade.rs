//! Checkpoint readers take only their source's lock (protocol/binaries.md, 1;
//! jig/docs/design/protocol/store/checkpoints.md, 6).
use std::{
    ffi::OsStr,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::{Path, PathBuf},
};
use temper_engine_shell::upgrade::Request;
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
    let scratch = skein_scratch::Scratch::new_in(&skein_scratch::paths::target_tmp(), "box-upgrade-deployment");
    let config = scratch.path().join("config");
    let data = scratch.path().join("data");
    let cache = scratch.path().join("cache");
    for directory in [&config, &data, &cache] {
        std::fs::create_dir(directory).expect("tool directory");
        std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700)).expect("private directory");
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
    let mut file = temper_config_shell::File {
        listen: Some("127.0.0.1:7100".parse().expect("clients")),
        webhooks: Some("127.0.0.1:7101".parse().expect("webhooks")),
        ..Default::default()
    };
    // The floor keeps accounts at both providers; all endpoints remain inert and local.
    let mut providers = temper_agent_fixtures::providers::stock();
    providers["providers"].as_array_mut().expect("stock providers").truncate(1);
    providers["endpoints"].as_array_mut().expect("stock endpoints").truncate(1);
    providers["providers"][0]["sources"]["sign_in"] = serde_json::json!({
        "issuer":"http://127.0.0.1:7104", "client":"test-client", "scopes":["openid","read"],
        "authorization":"http://127.0.0.1:7104/authorize", "token":"http://127.0.0.1:7104/token",
        "keys":"http://127.0.0.1:7104/keys", "transport":"plaintext", "must_grant":["read"],
        "redirect_path":"callback"
    });
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
    temper_service_shell::providers::write(
        &data.join("providers.setting"),
        &temper_agent_fixtures::providers::values(&providers),
    )
    .expect("fixture providers");
    file.test.github = Some(temper_config_shell::TestGitHub {
        api: "127.0.0.1:1".into(),
        transport: temper_config_shell::Transport::Plaintext,
        remote: "http://127.0.0.1:1".into(),
        web: "http://127.0.0.1:1".into(),
        git: Some("/usr/bin/git".into()),
    });
    std::fs::write(
        config.join(temper_config_shell::FILE),
        temper_config_shell::render(&file, temper_config_shell::Shape::Standalone),
    )
    .expect("configuration file");
    std::fs::create_dir(data.join("secrets")).expect("fixture secrets directory");
    std::fs::write(data.join("worker.secret"), b"box-upgrade-test-worker-secret").expect("fixture worker secret");
    std::fs::set_permissions(data.join("worker.secret"), std::fs::Permissions::from_mode(0o600))
        .expect("private fixture worker secret");
    let directories = Directories {
        config: config.as_os_str().as_bytes().into(),
        data: data.as_os_str().as_bytes().into(),
        cache: cache.as_os_str().as_bytes().into(),
        origins: [skein_shell::Origin::Flag; 3],
    };
    (scratch, directories)
}

fn tree(root: &Path) -> Vec<(PathBuf, u32, Vec<u8>)> {
    let mut result = Vec::new();
    let mut entries = std::fs::read_dir(root).unwrap().map(|entry| entry.unwrap()).collect::<Vec<_>>();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let metadata = entry.metadata().unwrap();
        let bytes = if metadata.is_file() { std::fs::read(entry.path()).unwrap() } else { Vec::new() };
        result.push((entry.path(), metadata.permissions().mode(), bytes));
        if metadata.is_dir() {
            result.extend(tree(&entry.path()));
        }
    }
    result
}

#[test]
fn checkpoint_checks_take_only_the_checkpoint_lock_and_leave_data_unchanged() {
    let (_scratch, directories) = deployment();
    let data = Path::new(OsStr::from_bytes(&directories.data));
    let deployment = data.join("state");
    let checkpoints = deployment.join("checkpoints");
    std::fs::create_dir(&checkpoints).unwrap();
    let checkpoint = checkpoints.join("00000000000000000001-00000000000000ff");
    // The stopped box and its checkpoint hold the same generation's maps.
    let staged = data.join("checkpoint-copy");
    copy(&deployment, &staged);
    std::fs::rename(staged, &checkpoint).unwrap();
    let before = tree(data);
    let held = std::fs::File::open(&deployment).unwrap();
    held.try_lock().expect("serving box's deployment lock");
    for (mode, request) in [
        ("--check", Request::Check(checkpoint.as_os_str().as_bytes().into())),
        ("--workers", Request::Workers(checkpoint.as_os_str().as_bytes().into())),
    ] {
        let (service, _) = crate::upgrade::prepare(&directories).expect("read-only box preparation");
        assert!(tree(data) == before, "{mode}: preparation writes nothing under --data");
        drop(service);
        assert_eq!(
            crate::upgrade::run(&directories, &request).expect("checkpoint tool"),
            0,
            "{mode}: deployment lock is not taken"
        );
        assert!(tree(data) == before, "{mode}: the deployment and checkpoint stay unchanged");
        let source_lock = std::fs::File::open(&checkpoint).unwrap();
        source_lock.try_lock().expect("hold the checkpoint's source lock");
        assert_eq!(
            crate::upgrade::run(&directories, &request).expect("busy source report"),
            1,
            "{mode}: the checkpoint lock is taken"
        );
        assert!(tree(data) == before, "{mode}: a refused rehearsal writes nothing under --data");
    }
}
