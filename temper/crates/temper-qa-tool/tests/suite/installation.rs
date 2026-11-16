//! Installs share inodes or check cross-device copies (qa/places.md, 3).
use super::{
    build_files::Files,
    world::{World, id},
};
use std::{
    fs,
    io::Write,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
};
use temper_qa_tool::installation::install;

// The fixture's four bin files are linked; the bundle's are copied and checked.
const LINKED_FILES: usize = 4;
fn installation() -> (World, PathBuf, PathBuf) {
    let world = World::new();
    let build = world.installable_build(2);
    let place = world.home.directory.join("places/1");
    (world, build, place)
}

fn inode(path: &Path) -> u64 {
    fs::metadata(path).expect("file").ino()
}

#[test]
fn install_links_files_and_an_unchanged_manifest_installs_nothing() {
    let (_world, build, place) = installation();
    let mut face = Files::default();
    install(&build, &id(2), &place, &mut face).expect("installed");
    assert_eq!(face.links.len(), LINKED_FILES);
    assert_eq!(face.hashes.len(), 0);
    assert_eq!(face.checks.len(), 1, "the copied bundle is checked");
    assert_eq!(inode(&build.join("bin/temper-box")), inode(&place.join("bin/temper-box")));
    let bundle = place.join("bundle/assets/app.js");
    assert_ne!(inode(&build.join("bundle/assets/app.js")), inode(&bundle));
    assert_eq!(fs::metadata(&bundle).expect("bundle file").nlink(), 1, "the box takes one link alone");
    assert_eq!(fs::metadata(place.join("manifest")).expect("receipt").permissions().mode() & 0o777, 0o600);
    fs::write(place.join("setup"), "kept setup").expect("setup");
    let receipt_inode = inode(&place.join("manifest"));
    install(&build, &id(2), &place, &mut face).expect("reused");
    assert_eq!(face.links.len(), LINKED_FILES);
    assert_eq!(face.checks.len(), 1);
    assert_eq!(inode(&place.join("manifest")), receipt_inode);
    assert_eq!(fs::read_to_string(place.join("setup")).expect("untouched"), "kept setup");
    // A changed manifest must install again, even at the same commit.
    let mut manifest = fs::read(build.join("manifest")).expect("manifest");
    manifest.extend_from_slice(b"\n");
    fs::write(build.join("manifest"), &manifest).expect("changed manifest");
    install(&build, &id(2), &place, &mut face).expect("reinstalled");
    assert_eq!(face.links.len(), LINKED_FILES * 2);
    assert!(!place.join("setup").exists());
    assert_eq!(fs::read(place.join("manifest")).expect("receipt"), manifest);
    fs::remove_dir_all(&build).expect("released build deleted");
    assert_eq!(fs::read(place.join("bundle/assets/app.js")).expect("link survives"), b"javascript");
}

#[test]
fn a_cross_device_install_copies_and_checks_the_installed_manifest() {
    let (_world, build, place) = installation();
    // Kept directories outside bin/ and bundle/ are not installed or checked.
    fs::create_dir(build.join("bundle-record")).expect("old bundle");
    fs::write(build.join("bundle-record/index.html"), "old").expect("old file");
    writeln!(
        fs::OpenOptions::new().append(true).open(build.join("manifest")).expect("manifest"),
        "{}  bundle-record/index.html",
        "0".repeat(64)
    )
    .expect("old row");
    let mut face = Files { cross_device: true, ..Default::default() };
    install(&build, &id(2), &place, &mut face).expect("copied");
    assert_eq!(face.links.len(), LINKED_FILES);
    assert_eq!(face.checks.len(), 1);
    let manifest = fs::read_to_string(build.join("manifest")).expect("manifest");
    let expected = manifest
        .split_inclusive('\n')
        .skip(1)
        .filter(|line| !line.trim_end().ends_with("  bundle-record/index.html"))
        .collect::<String>();
    assert_eq!(face.checks[0], (place.clone(), expected.into_bytes()));
    for path in ["bin/temper-box", "bundle/assets/app.js"] {
        assert_ne!(inode(&build.join(path)), inode(&place.join(path)));
        assert_eq!(fs::read(build.join(path)).expect("build"), fs::read(place.join(path)).expect("copy"));
        assert_eq!(
            fs::metadata(build.join(path)).expect("build").permissions().mode(),
            fs::metadata(place.join(path)).expect("copy").permissions().mode()
        );
    }
    assert!(!place.join("manifest.tmp").exists());
    assert!(!place.join("bundle-record").exists());
}

#[test]
fn a_failed_copy_check_leaves_no_install_receipt() {
    let (world, build, place) = installation();
    let mut face = Files::default();
    install(&build, &id(2), &place, &mut face).expect("first install");
    let next = world.installable_build(3);
    face.cross_device = true;
    face.check_error = true;
    let why = install(&next, &id(3), &place, &mut face).expect_err("failed check");
    assert!(why.0.contains("sha256sum --check"), "{why}");
    assert!(!place.join("manifest").exists());
    // The first install checked its copied bundle, and the second install's check failed.
    assert_eq!(face.checks.len(), 2);
    face.check_error = false;
    install(&next, &id(3), &place, &mut face).expect("retry succeeds");
    assert_eq!(fs::read(place.join("manifest")).expect("receipt"), fs::read(next.join("manifest")).expect("build"));
}

#[test]
fn a_non_device_link_error_refuses_without_copying() {
    let (_world, build, place) = installation();
    let mut face = Files { link_error: true, ..Default::default() };
    let why = install(&build, &id(2), &place, &mut face).expect_err("link refused");
    assert!(why.0.to_ascii_lowercase().contains("permission denied"), "{why}");
    assert!(!place.join("manifest").exists());
    assert_eq!(face.checks.len(), 0);
    assert_eq!(face.links.len(), 1);
}
