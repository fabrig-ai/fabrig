//! The upgrade command on a deployment copy (protocol/binaries.md, 1).
use super::{Request, run};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
<<<<<<< HEAD
use temper_deployment_fixtures::deployment;
=======
mod deployment;
pub(crate) use deployment::deployment;
use deployment::{copy, tree};
>>>>>>> refs/manual/8753

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

<<<<<<< HEAD
=======
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

>>>>>>> refs/manual/8753
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
