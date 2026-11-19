use super::{
    deployment::{copy, deployment},
    shell,
};
use shell::upgrade::{Request, run};
use std::{
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::Path,
};

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
fn stopped_preparation_names_bad_held_lines_without_opening_or_changing_the_store() {
    let (_scratch, directories) = deployment();
    let data = Path::new(std::ffi::OsStr::from_bytes(&directories.data));
    let source = std::fs::File::open(data.join("state")).expect("deployment directory");
    source.try_lock().expect("serving engine holds the store");
    for (body, prefix) in [
        ("not.a.limit 1\n", "limits-held line 1: not.a.limit is no limit of this release"),
        ("people.identity_bytes 4294967296\n", "limits-held line 1: people.identity_bytes (setting limits):"),
    ] {
        std::fs::write(data.join("limits-held"), body).expect("held input");
        let before = tree(data);
        let Err(why) = shell::stopped::prepare("temper-engine", &directories) else {
            panic!("stopped preparation ignored limits-held");
        };
        assert!(why.starts_with(prefix), "{why}");
        assert!(!why.contains("put limits-held"), "the held-file refusal names its own line: {why}");
        assert_eq!(tree(data), before, "held-file refusal opens and changes no store");
    }
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
        let (mut service, prepared_deployment) =
            shell::stopped::prepare("temper-engine", &directories).expect("read-only preparation");
        assert_eq!(tree(data), before, "preparation writes nothing under --data");
        service.engine_component().build_name("test-build");
        assert_eq!(
            shell::upgrade::run_prepared("temper-engine", &directories, &request, service, &prepared_deployment)
                .expect("checkpoint tool"),
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
