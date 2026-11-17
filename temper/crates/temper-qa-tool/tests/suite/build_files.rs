//! Scripted checksum answers and link errors (qa/README.md, 6).
use std::{
    fs,
    path::{Path, PathBuf},
};
use temper_qa_tool::{Refusal, faces::BuildFilesFace};

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
impl BuildFilesFace for Files {
    fn hash(&mut self, directory: &Path, files: &[PathBuf]) -> Result<Vec<u8>, Refusal> {
        self.hashes.push((directory.into(), files.into()));
        Ok(self.answer.clone())
    }
    fn check(&mut self, directory: &Path, manifest: &Path) -> Result<(), Refusal> {
        self.checks.push((directory.into(), fs::read(manifest).expect("checksum input")));
        if self.check_error { Err(Refusal("sha256sum --check: bundle/assets/app.js: FAILED".into())) } else { Ok(()) }
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

#[test]
fn a_checksum_check_keeps_failed_paths_from_stdout() {
    use skein_scratch::Scratch;
    use std::os::unix::{ffi::OsStrExt, fs::PermissionsExt};
    use temper_qa_tool::faces::RealBuildFiles;
    let scratch = Scratch::new("qa-checksum-face");
    let bin = scratch.path().join("tools");
    let place = scratch.path().join("place");
    fs::create_dir(&bin).expect("fake command directory");
    fs::create_dir(&place).expect("place");
    let command = bin.join("sha256sum");
    fs::write(
        &command,
        br#"#!/bin/sh
if test "$1" != --check || test "$2" != -- || test "$PWD" != "$CHECK_DIRECTORY" || test ! -f "$3"; then
  printf 'unexpected checksum command\n' >&2
  exit 2
fi
printf 'bundle/assets/app.js: FAILED\n'
printf 'sha256sum: WARNING: 1 computed checksum did NOT match\n' >&2
exit 1
"#,
    )
    .expect("fake checksum command");
    fs::set_permissions(&command, fs::Permissions::from_mode(0o700)).expect("executable");
    let manifest = place.join("manifest.tmp");
    fs::write(&manifest, b"checksums").expect("checksum input");
    let mut face = RealBuildFiles {
        environment: vec![
            (b"PATH".as_slice().into(), bin.as_os_str().as_bytes().into()),
            (b"CHECK_DIRECTORY".as_slice().into(), place.as_os_str().as_bytes().into()),
        ],
    };
    let why = face.check(&place, &manifest).expect_err("failed check");
    assert!(why.0.contains("bundle/assets/app.js: FAILED"), "{why}");
    assert!(why.0.contains("1 computed checksum did NOT match"), "{why}");
}
