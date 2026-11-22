use std::{
    fs::{self, OpenOptions},
    os::unix::fs::{MetadataExt, PermissionsExt},
};
use temper_box_shell::{directories, testing::Created};

#[test]
fn retained_seed_outputs_survive_later_refusal_and_rollback() {
    let directory = std::env::temp_dir().join(format!("temper-seed-reuse-{}", std::process::id()));
    directories::make_private(&directory, skein_shell::effective_user()).expect("private fixture");
    let retained = directory.join("retained");
    let new = directory.join("new");
    let observations = directory.join("observations");
    let mut first = Created::default();
    first.output(&retained, b"seeded bytes").expect("new output");
    first.file(&observations, b"observation bytes").expect("observation fixture");
    let inode = fs::metadata(&retained).expect("retained metadata").ino();
    let mut second = Created::default();
    second.output(&retained, b"seeded bytes").expect("matching retained output");
    second.output(&new, b"new bytes").expect("new invocation's output");
    assert_eq!(fs::metadata(&new).expect("new output").mode() & 0o777, 0o600);
    assert!(
        second
            .file(&observations, b"replacement")
            .expect_err("observations remain create-new")
            .contains("observations")
    );
    second.rollback();
    assert_eq!(fs::read(&retained).expect("retained output survives"), b"seeded bytes");
    assert_eq!(fs::metadata(&retained).expect("retained metadata").ino(), inode);
    assert_eq!(fs::read(&observations).expect("observations untouched"), b"observation bytes");
    assert!(!new.exists());
    fs::remove_dir_all(directory).expect("remove fixture");
}

#[test]
fn unsafe_or_different_retained_seed_outputs_are_named_and_refused() {
    for kind in ["bytes", "symlink", "hardlink", "mode", "oversize", "directory"] {
        let directory = std::env::temp_dir().join(format!("temper-seed-refusal-{}-{kind}", std::process::id()));
        directories::make_private(&directory, skein_shell::effective_user()).expect("private fixture");
        let path = directory.join("output");
        let mut created = Created::default();
        match kind {
            "directory" => fs::create_dir(&path).expect("directory fixture"),
            "symlink" | "hardlink" => {
                let target = directory.join("target");
                created.output(&target, b"seed").expect("link target");
                if kind == "symlink" {
                    std::os::unix::fs::symlink(&target, &path).expect("symlink fixture");
                } else {
                    fs::hard_link(&target, &path).expect("hardlink fixture");
                }
            }
            _ => {
                created.output(&path, b"seed").expect("private output fixture");
                match kind {
                    "bytes" => fs::write(&path, b"other").expect("different bytes"),
                    "mode" => fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).expect("public mode"),
                    "oversize" => OpenOptions::new()
                        .write(true)
                        .open(&path)
                        .expect("fixture")
                        .set_len(1 << 20)
                        .expect("oversized output"),
                    _ => unreachable!("fixture kind"),
                }
            }
        }
        let mut reused = Created::default();
        let error = reused.output(&path, b"seed").expect_err("unsafe output refused");
        assert!(error.contains(path.to_str().expect("fixture path")), "{kind}: {error}");
        assert!(reused.files.is_empty(), "refused retained files are never owned for rollback");
        fs::remove_dir_all(directory).expect("remove fixture");
    }
}
