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
        serde_json::from_slice(include_bytes!("../../../../docs/examples/owned.json")).expect("owned example");
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

#[test]
fn workers_arguments_are_distinct_and_modes_exclude_each_other() {
    let environment = [(Box::from(b"HOME".as_slice()), Box::from(b"/home/test".as_slice()))];
    let words = |words: &[&[u8]]| words.iter().map(|word| Box::from(*word)).collect::<Vec<_>>();
    for mode in ["--check", "--workers"] {
        let (directories, request) =
            super::configure(mode, &words(&[b"--data", b"state", b"backup"]), &environment, b"/work")
                .expect("one directory");
        assert_eq!(directories.data.as_ref(), b"/work/state");
        assert_eq!(
            request,
            if mode == "--check" {
                Request::Check(b"/work/backup".as_slice().into())
            } else {
                Request::Workers(b"/work/backup".as_slice().into())
            }
        );
        for invalid in [
            words(&[]),
            words(&[b""]),
            words(&[b"one", b"two"]),
            words(&[b"--check", b"one"]),
            words(&[b"--workers", b"one"]),
            words(&[b"--back"]),
            words(&[b"one", b"--back"]),
        ] {
            assert!(super::configure(mode, &invalid, &environment, b"/work").is_err(), "{mode}: {invalid:?}");
        }
    }
    assert_eq!(super::configure("--back", &[], &environment, b"/work").unwrap().1, Request::Back);
    for invalid in [words(&[b"one"]), words(&[b"--workers", b"one"]), words(&[b"--check", b"one"])] {
        assert!(super::configure("--back", &invalid, &environment, b"/work").is_err());
    }
    let arguments = words(&[b"upgrade", b"--workers", b"backup"]);
    let skein_shell::Reached::Command(row, rest) = crate::ENGINE.reach(b"temper-engine", &arguments) else {
        panic!("workers row")
    };
    assert_eq!(row, &crate::commands::UPGRADE_WORKERS);
    assert_eq!(
        super::configure("--workers", rest, &environment, b"/work").unwrap().1,
        Request::Workers(b"/work/backup".as_slice().into())
    );
}

fn tree(root: &Path) -> Vec<(std::path::PathBuf, u32, Vec<u8>)> {
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
    let data = Path::new(std::ffi::OsStr::from_bytes(&directories.data));
    let deployment = data.join("state");
    let checkpoints = deployment.join("checkpoints");
    std::fs::create_dir(&checkpoints).unwrap();
    let checkpoint = checkpoints.join("00000000000000000001-00000000000000ff");
    // The stopped floor and its checkpoint hold the same generation's maps.
    let staged = data.join("checkpoint-copy");
    copy(&deployment, &staged);
    std::fs::rename(staged, &checkpoint).unwrap();
    let before = tree(data);
    let held = std::fs::File::open(&deployment).unwrap();
    held.try_lock().expect("serving engine's deployment lock");
    for request in [
        Request::Check(checkpoint.as_os_str().as_bytes().into()),
        Request::Workers(checkpoint.as_os_str().as_bytes().into()),
    ] {
        // Preparation reads an existing --data; make_private checks its metadata alone.
        let (service, _) = crate::stopped::prepare("temper-engine", &directories).expect("read-only preparation");
        assert_eq!(tree(data), before, "preparation writes nothing under --data");
        drop(service);
        assert_eq!(
            run("temper-engine", &directories, &request, "test-build").expect("checkpoint tool"),
            0,
            "deployment lock is not taken"
        );
        assert_eq!(tree(data), before, "the deployment and checkpoint stay unchanged");
        let source_lock = std::fs::File::open(&checkpoint).unwrap();
        source_lock.try_lock().expect("hold the checkpoint's source lock");
        assert_eq!(
            run("temper-engine", &directories, &request, "test-build").expect("busy source report"),
            1,
            "the checkpoint lock is taken"
        );
        assert_eq!(tree(data), before, "a refused rehearsal writes nothing under --data");
    }
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
