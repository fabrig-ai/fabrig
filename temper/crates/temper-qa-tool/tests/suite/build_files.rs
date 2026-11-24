//! Scripted build-file answers and real checksums (qa/builds.md, 3–4; qa/places.md, 8).
use sha2::{Digest, Sha256};
use skein_scratch::Scratch;
use std::{
    fs::{self, File},
    io,
    os::unix::net::{UnixListener, UnixStream},
    path::{Path, PathBuf},
};
use temper_qa_tool::{
    Refusal,
    faces::{DirectoryMode, FileEntry, FilesFace, LockMode, PathFacts, RealFiles, RemoveMode, WriteOptions},
};

#[derive(Default)]
pub struct Files {
    pub answer: Vec<u8>,
    pub hashes: Vec<(PathBuf, Vec<PathBuf>)>,
    pub links: Vec<(PathBuf, PathBuf)>,
    pub checks: Vec<(PathBuf, Vec<u8>)>,
    pub cross_device: bool,
    pub check_error: bool,
    pub link_error: bool,
}
impl FilesFace for Files {
    type Lock = File;
    type Listener = UnixListener;
    type Connection = UnixStream;
    type Reader = File;
    type Writer = File;

    fn read(&mut self, path: &Path) -> io::Result<Vec<u8>> {
        RealFiles.read(path)
    }
    fn write(&mut self, path: &Path, bytes: &[u8], options: WriteOptions, sync: bool) -> io::Result<()> {
        RealFiles.write(path, bytes, options, sync)
    }
    fn directory(&mut self, path: &Path, mode: DirectoryMode) -> io::Result<()> {
        RealFiles.directory(path, mode)
    }
    fn facts(&mut self, path: &Path, follow: bool) -> io::Result<PathFacts> {
        RealFiles.facts(path, follow)
    }
    fn list(&mut self, path: &Path) -> io::Result<Vec<FileEntry>> {
        RealFiles.list(path)
    }
    fn open_read(&mut self, path: &Path) -> io::Result<File> {
        RealFiles.open_read(path)
    }
    fn open_write(&mut self, path: &Path, options: WriteOptions) -> io::Result<File> {
        RealFiles.open_write(path, options)
    }
    fn writer_permissions(&mut self, file: &File, mode: u32) -> io::Result<()> {
        RealFiles.writer_permissions(file, mode)
    }
    fn remove(&mut self, path: &Path, mode: RemoveMode) -> io::Result<()> {
        RealFiles.remove(path, mode)
    }
    fn rename(&mut self, source: &Path, destination: &Path) -> io::Result<()> {
        RealFiles.rename(source, destination)
    }
    fn lock(&mut self, path: &Path, mode: LockMode, exact: bool) -> io::Result<Option<File>> {
        RealFiles.lock(path, mode, exact)
    }
    fn connect(&mut self, path: &Path) -> io::Result<Option<UnixStream>> {
        RealFiles.connect(path)
    }
    fn hash(&mut self, directory: &Path, files: &[PathBuf]) -> Result<Vec<u8>, Refusal> {
        self.hashes.push((directory.into(), files.into()));
        Ok(self.answer.clone())
    }
    fn check(&mut self, directory: &Path, manifest: &Path) -> Result<(), Refusal> {
        self.checks.push((directory.into(), fs::read(manifest).expect("checksum input")));
        if self.check_error { Err(Refusal("checksum check: bundle/assets/app.js: FAILED".into())) } else { Ok(()) }
    }
    fn link(&mut self, source: &Path, destination: &Path) -> std::io::Result<()> {
        self.links.push((source.into(), destination.into()));
        if self.link_error {
            Err(std::io::ErrorKind::PermissionDenied.into())
        } else if self.cross_device {
            Err(std::io::ErrorKind::CrossesDevices.into())
        } else {
            fs::hard_link(source, destination)
        }
    }
}

// APFS, macOS's file system, holds no file name that is not UTF-8 (temper/docs/design/testing/macos.md, 3).
#[cfg(target_os = "linux")]
#[test]
fn real_checksums_keep_a_non_utf8_path_as_its_bytes() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    let scratch = Scratch::new("qa-checksum-byte-path");
    fs::create_dir(scratch.path().join("bundle")).expect("bundle");
    let path = PathBuf::from(OsString::from_vec(b"bundle/weird\\\n\r\xff".to_vec()));
    fs::write(scratch.path().join(&path), b"asset").expect("byte path");
    let lines = RealFiles.hash(scratch.path(), &[path]).expect("checksum");
    let mut expected = format!("\\{:x}  bundle/weird\\\\\\n\\r", Sha256::digest(b"asset")).into_bytes();
    expected.extend_from_slice(b"\xff\n");
    assert_eq!(lines, expected);
    let manifest = scratch.path().join("manifest.tmp");
    fs::write(&manifest, &lines).expect("manifest");
    RealFiles.check(scratch.path(), &manifest).expect("raw byte path checked");
}
