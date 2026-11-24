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
        assert_eq!(
            prepared_rehearsal(
                &directories,
                &request,
                data,
                &before,
                "read-only preparation",
                "preparation writes nothing under --data",
                "checkpoint tool",
            ),
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
        assert_eq!(
            prepared_rehearsal(
                &directories,
                &request,
                data,
                &before,
                "read-only preparation with source held",
                "preparation with source held writes nothing under --data",
                "prepared busy source report",
            ),
            1,
            "the prepared run takes the checkpoint lock"
        );
        assert_eq!(tree(data), before, "a refused prepared rehearsal writes nothing under --data");
    }
}

fn prepared_rehearsal(
    directories: &shell::directories::Directories,
    request: &Request,
    data: &Path,
    before: &[(std::path::PathBuf, u32, Vec<u8>)],
    prepare_message: &str,
    unchanged_message: &str,
    run_message: &str,
) -> u8 {
    // Preparation reads an existing --data; make_private checks its metadata alone.
    let (mut service, deployment) = shell::stopped::prepare("temper-engine", directories).expect(prepare_message);
    assert_eq!(tree(data), before, "{unchanged_message}");
    service.engine_component().build_name("test-build");
    shell::upgrade::run_prepared("temper-engine", directories, request, service, &deployment).expect(run_message)
}
