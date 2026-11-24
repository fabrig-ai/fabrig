use super::{
    deployment::{copy, deployment},
    shell,
};
use shell::{
    testing::scratch,
    upgrade::{Request, run},
};

mod tests;
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
        let (mut service, prepared_deployment) =
            shell::stopped::prepare("temper-engine", &directories).expect("read-only preparation with source held");
        assert_eq!(tree(data), before, "preparation with source held writes nothing under --data");
        service.engine_component().build_name("test-build");
        assert_eq!(
            shell::upgrade::run_prepared("temper-engine", &directories, &request, service, &prepared_deployment)
                .expect("prepared busy source report"),
            1,
            "the prepared run takes the checkpoint lock"
        );
        assert_eq!(tree(data), before, "a refused prepared rehearsal writes nothing under --data");
    }
}
