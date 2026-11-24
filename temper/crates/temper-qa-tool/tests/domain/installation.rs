//! Installs share inodes or check cross-device copies (qa/places.md, 3).
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
};
use temper_qa_tool::{
    Refusal,
    faces::{DirectoryMode, FileEntry, FilesFace, Mode, PathFacts, RemoveMode, WriteMode, WriteOptions},
    installation::install,
    testing::{Connection, FileHandle, FileOp, Listener, Lock, Tree, World, id},
};

// Keep the install's checksum answers scripted, even though all its paths are in the tree.
struct Files {
    tree: Tree,
    hashes: Vec<(PathBuf, Vec<PathBuf>)>,
    links: Vec<(PathBuf, PathBuf)>,
    checks: Vec<(PathBuf, Vec<u8>)>,
    check_error: bool,
}
impl Files {
    fn new(tree: Tree) -> Self {
        Self { tree, hashes: Vec::new(), links: Vec::new(), checks: Vec::new(), check_error: false }
    }
}

impl FilesFace for Files {
    type Lock = Lock;
    type Listener = Listener;
    type Connection = Connection;
    type Reader = FileHandle;
    type Writer = FileHandle;

    fn read(&mut self, path: &Path) -> io::Result<Vec<u8>> {
        self.tree.read(path)
    }
    fn write(&mut self, path: &Path, bytes: &[u8], options: WriteOptions, sync: bool) -> io::Result<()> {
        self.tree.write(path, bytes, options, sync)
    }
    fn directory(&mut self, path: &Path, mode: DirectoryMode) -> io::Result<()> {
        self.tree.directory(path, mode)
    }
    fn facts(&mut self, path: &Path, follow: bool) -> io::Result<PathFacts> {
        self.tree.facts(path, follow)
    }
    fn list(&mut self, path: &Path) -> io::Result<Vec<FileEntry>> {
        self.tree.list(path)
    }
    fn open_read(&mut self, path: &Path) -> io::Result<FileHandle> {
        self.tree.open_read(path)
    }
    fn open_write(&mut self, path: &Path, options: WriteOptions) -> io::Result<FileHandle> {
        self.tree.open_write(path, options)
    }
    fn writer_permissions(&mut self, file: &FileHandle, mode: u32) -> io::Result<()> {
        self.tree.writer_permissions(file, mode)
    }
    fn remove(&mut self, path: &Path, mode: RemoveMode) -> io::Result<()> {
        self.tree.remove(path, mode)
    }
    fn rename(&mut self, source: &Path, destination: &Path) -> io::Result<()> {
        self.tree.rename(source, destination)
    }
    fn hash(&mut self, directory: &Path, files: &[PathBuf]) -> Result<Vec<u8>, Refusal> {
        self.hashes.push((directory.into(), files.into()));
        Ok(Vec::new())
    }
    fn check(&mut self, directory: &Path, manifest: &Path) -> Result<(), Refusal> {
        self.checks.push((directory.into(), self.tree.read(manifest).expect("checksum input")));
        if self.check_error { Err(Refusal("checksum check: bundle/assets/app.js: FAILED".into())) } else { Ok(()) }
    }
    fn link(&mut self, source: &Path, destination: &Path) -> io::Result<()> {
        self.links.push((source.into(), destination.into()));
        self.tree.link(source, destination)
    }
}

// The fixture's four bin files are linked; the bundle's are copied and checked.
const LINKED_FILES: usize = 4;
fn installation() -> (World, PathBuf, PathBuf) {
    let world = World::new();
    let build = world.installable_build(2);
    let place = world.home.directory.join("places/1");
    (world, build, place)
}

fn inode(tree: &mut Tree, path: &Path) -> u64 {
    tree.facts(path, true).expect("file").inode
}

#[test]
fn install_links_files_and_an_unchanged_manifest_installs_nothing() {
    let (world, build, place) = installation();
    let mut tree = world.files();
    let mut face = Files::new(tree.clone());
    install(&build, &id(2), &place, temper_qa_tool::Kind::Box, &mut face).expect("installed");
    assert_eq!(face.links.len(), LINKED_FILES);
    assert_eq!(face.hashes.len(), 0);
    assert_eq!(face.checks.len(), 1, "the copied bundle is checked");
    assert_eq!(inode(&mut tree, &build.join("bin/temper-box")), inode(&mut tree, &place.join("bin/temper-box")));
    let bundle = place.join("bundle/assets/app.js");
    assert_ne!(inode(&mut tree, &build.join("bundle/assets/app.js")), inode(&mut tree, &bundle));
    assert_eq!(tree.facts(&bundle, true).expect("bundle file").links, 1, "the box takes one link alone");
    assert_eq!(tree.mode(place.join("manifest")).expect("receipt") & 0o777, 0o600);
    tree.write(&place.join("setup"), b"kept setup", WriteOptions::REPLACE, false).expect("setup");
    let receipt_inode = inode(&mut tree, &place.join("manifest"));
    install(&build, &id(2), &place, temper_qa_tool::Kind::Box, &mut face).expect("reused");
    assert_eq!(face.links.len(), LINKED_FILES);
    assert_eq!(face.checks.len(), 1);
    assert_eq!(inode(&mut tree, &place.join("manifest")), receipt_inode);
    assert_eq!(tree.read_to_string(&place.join("setup")).expect("untouched"), "kept setup");
    // A changed manifest must install again, even at the same commit.
    let mut manifest = tree.read(&build.join("manifest")).expect("manifest");
    manifest.extend_from_slice(b"\n");
    tree.write(&build.join("manifest"), &manifest, WriteOptions::REPLACE, false).expect("changed manifest");
    install(&build, &id(2), &place, temper_qa_tool::Kind::Box, &mut face).expect("reinstalled");
    assert_eq!(face.links.len(), LINKED_FILES * 2);
    assert!(!tree.exists(place.join("setup")));
    assert_eq!(tree.read(&place.join("manifest")).expect("receipt"), manifest);
    tree.remove(&build, RemoveMode::Tree).expect("released build deleted");
    assert_eq!(tree.read(&place.join("bundle/assets/app.js")).expect("link survives"), b"javascript");
}

#[test]
fn a_cross_device_install_copies_and_checks_the_installed_manifest() {
    let (world, build, place) = installation();
    let mut tree = world.files();
    // Kept directories outside bin/ and bundle/ are not installed or checked.
    tree.directory(&build.join("bundle-record"), DirectoryMode::One).expect("old bundle");
    tree.write(&build.join("bundle-record/index.html"), b"old", WriteOptions::REPLACE, false).expect("old file");
    writeln!(
        tree.open_write(
            &build.join("manifest"),
            WriteOptions { way: WriteMode::Append, mode: Mode::Default, read: false }
        )
        .expect("manifest"),
        "{}  bundle-record/index.html",
        "0".repeat(64)
    )
    .expect("old row");
    tree.directory(&place, DirectoryMode::Parents).expect("place");
    tree.set_device(&place, 2).expect("place on a different device");
    let mut face = Files::new(tree.clone());
    install(&build, &id(2), &place, temper_qa_tool::Kind::Box, &mut face).expect("copied");
    assert_eq!(face.links.len(), LINKED_FILES);
    assert_eq!(face.checks.len(), 1);
    let manifest = tree.read_to_string(&build.join("manifest")).expect("manifest");
    let expected = manifest
        .split_inclusive('\n')
        .skip(1)
        .filter(|line| !line.trim_end().ends_with("  bundle-record/index.html"))
        .collect::<String>();
    assert_eq!(face.checks[0], (place.clone(), expected.into_bytes()));
    for path in ["bin/temper-box", "bundle/assets/app.js"] {
        assert_ne!(inode(&mut tree, &build.join(path)), inode(&mut tree, &place.join(path)));
        assert_eq!(tree.read(&build.join(path)).expect("build"), tree.read(&place.join(path)).expect("copy"));
        assert_eq!(tree.mode(build.join(path)).expect("build"), tree.mode(place.join(path)).expect("copy"));
    }
    assert!(!tree.exists(place.join("manifest.tmp")));
    assert!(!tree.exists(place.join("bundle-record")));
}

#[test]
fn a_failed_copy_check_leaves_no_install_receipt() {
    let (world, build, place) = installation();
    let mut tree = world.files();
    let mut face = Files::new(tree.clone());
    install(&build, &id(2), &place, temper_qa_tool::Kind::Box, &mut face).expect("first install");
    let next = world.installable_build(3);
    tree.set_device(&next, 2).expect("next build on a different device");
    face.check_error = true;
    let why = install(&next, &id(3), &place, temper_qa_tool::Kind::Box, &mut face).expect_err("failed check");
    assert!(why.0.contains("checksum check"), "{why}");
    assert!(!tree.exists(place.join("manifest")));
    // The first install checked its copied bundle, and the second install's check failed.
    assert_eq!(face.checks.len(), 2);
    face.check_error = false;
    install(&next, &id(3), &place, temper_qa_tool::Kind::Box, &mut face).expect("retry succeeds");
    assert_eq!(tree.read(&place.join("manifest")).expect("receipt"), tree.read(&next.join("manifest")).expect("build"));
}

#[test]
fn a_non_device_link_error_refuses_without_copying() {
    let (world, build, place) = installation();
    let tree = world.files();
    tree.inject(build.join("bin/fake-claude"), FileOp::Link, io::ErrorKind::PermissionDenied);
    let mut face = Files::new(tree.clone());
    let why = install(&build, &id(2), &place, temper_qa_tool::Kind::Box, &mut face).expect_err("link refused");
    assert!(why.0.to_ascii_lowercase().contains("permission denied"), "{why}");
    assert!(!tree.exists(place.join("manifest")));
    assert_eq!(face.checks.len(), 0);
    assert_eq!(face.links.len(), 1);
}
