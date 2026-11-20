//! The walk's owner argument stands in for root on either system.

use super::pass_links;
use skein_scratch::Scratch;
use std::fs;
use std::os::unix::{
    ffi::OsStrExt,
    fs::{PermissionsExt, symlink},
};
use std::path::{Path, PathBuf};

fn layout(mode: u32) -> Scratch {
    let scratch = Scratch::new("system-links");
    fs::set_permissions(scratch.path(), fs::Permissions::from_mode(mode)).expect("explicit holding-directory mode");
    fs::create_dir(scratch.path().join("real")).expect("real directory");
    fs::set_permissions(scratch.path().join("real"), fs::Permissions::from_mode(0o755))
        .expect("explicit leaf-link directory mode");
    fs::write(scratch.path().join("real/f"), b"file").expect("real file");
    scratch
}

fn parent_link(mode: u32) -> Scratch {
    let scratch = layout(mode);
    symlink("real", scratch.path().join("sys")).expect("parent link");
    scratch
}

fn walk(path: &Path, owner: u32) -> Option<Box<[u8]>> {
    pass_links(libc::AT_FDCWD, path.as_os_str().as_bytes(), owner)
}

#[test]
fn owned_parent_links_are_passed_without_leaving_a_link() {
    let scratch = parent_link(0o755);
    let expected = scratch.path().join("real/f");
    let answer = walk(&scratch.path().join("sys/f"), crate::effective_user()).expect("owned parent passed");
    assert_eq!(&*answer, expected.as_os_str().as_bytes(), "relative link target replaces its name");
    let mut prefix = PathBuf::new();
    for component in expected.components() {
        prefix.push(component);
        assert!(!fs::symlink_metadata(&prefix).expect("walked prefix").file_type().is_symlink(), "no remaining link");
    }
}

#[test]
fn a_parent_link_is_passed_beneath_a_descriptor() {
    let scratch = parent_link(0o755);
    let directory = crate::open_root(scratch.path()).expect("borrowed directory descriptor");
    let answer = pass_links(directory.raw(), b"sys/f", crate::effective_user()).expect("owned parent passed");
    crate::close_startup_fd(directory);
    assert_eq!(&*answer, b"real/f", "the answer stays relative to the descriptor");
}

#[test]
fn another_owners_link_is_not_passed() {
    let scratch = parent_link(0o755);
    let user = crate::effective_user();
    let other = u32::from(user == 0);
    assert_eq!(walk(&scratch.path().join("sys/f"), other), None, "the link must belong to the named owner");
}

#[test]
fn links_in_writable_directories_are_not_passed() {
    for mode in [0o777, 0o775] {
        let scratch = parent_link(mode);
        assert_eq!(
            walk(&scratch.path().join("sys/f"), crate::effective_user()),
            None,
            "mode {mode:o} permits other writers"
        );
    }
}

#[test]
fn a_leaf_link_is_not_passed() {
    let scratch = layout(0o755);
    symlink("f", scratch.path().join("real/leaf")).expect("leaf link");
    assert_eq!(
        walk(&scratch.path().join("real/leaf"), crate::effective_user()),
        None,
        "the last component is left for the open"
    );
}

#[test]
fn dotdot_stops_before_a_link() {
    let scratch = parent_link(0o755);
    assert_eq!(walk(&scratch.path().join("real/../sys/f"), crate::effective_user()), None, "dotdot ends the walk");
}

#[test]
fn absolute_link_targets_restart_at_root() {
    let scratch = layout(0o755);
    let real = scratch.path().join("real");
    symlink(&real, scratch.path().join("abs")).expect("absolute target");
    let answer = walk(&scratch.path().join("abs/f"), crate::effective_user()).expect("absolute link passed");
    assert_eq!(&*answer, real.join("f").as_os_str().as_bytes(), "an absolute target replaces the whole prefix");
    assert!(answer.starts_with(b"/"), "the answer starts again at root");
}

#[test]
fn eight_links_are_passed_and_the_ninth_is_left() {
    let scratch = layout(0o755);
    for number in 1_u8..=9 {
        let target =
            if number == 9 { "real".into() } else { format!("l{}", number.checked_add(1).expect("small chain")) };
        symlink(target, scratch.path().join(format!("l{number}"))).expect("chain link");
    }
    let answer = walk(&scratch.path().join("l1/f"), crate::effective_user()).expect("eight links passed");
    assert_eq!(&*answer, scratch.path().join("l9/f").as_os_str().as_bytes(), "the ninth link is left as written");
}

#[test]
fn resolving_system_links_preserves_a_users_link_in_a_writable_directory() {
    let scratch = parent_link(0o777);
    let path = scratch.path().join("sys/f");
    assert_eq!(crate::resolve_system_links(&path), path, "a caller's check still sees the unsafe link");
}
