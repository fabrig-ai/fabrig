//! Seed outputs owned by one fake-peer start (temper/docs/design/protocol/setup/box-backends.md, 1).
use super::disk::{Disk, Native, UNSAFE_OUTPUT, opened};
use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct Created {
    pub files: Vec<PathBuf>,
    directories: Vec<PathBuf>,
}
impl Created {
    pub fn output(&mut self, path: &Path, expected: &[u8]) -> Result<(), String> {
        self.output_with(&mut Native, path, expected)
    }
    pub fn file(&mut self, path: &Path, bytes: &[u8]) -> Result<File, String> {
        self.file_with(&mut Native, path, bytes)
    }
    pub fn rollback(self) {
        self.rollback_with(&mut Native);
    }
    pub(super) fn output_with(&mut self, disk: &mut impl Disk, path: &Path, expected: &[u8]) -> Result<(), String> {
        let named = |why| format!("{}: {why}", path.display());
        let metadata = match disk.metadata(path) {
            Ok(metadata) => metadata,
            Err(why) if why.kind() == io::ErrorKind::NotFound => {
                self.file_with(disk, path, expected)?;
                return Ok(());
            }
            Err(why) => return Err(named(why.to_string())),
        };
        if !metadata.safe(disk.user()) {
            return Err(named(UNSAFE_OUTPUT.into()));
        }
        let (file, _) = opened(disk, path, metadata).map_err(named)?;
        let bound = skein_lib::len_u64(expected.len()).checked_add(1).expect("output bound");
        let content = disk.read(file, bound).map_err(|why| named(why.to_string()))?;
        if content.as_ref() != expected {
            return Err(named("retained output differs from expected seeded bytes".into()));
        }
        Ok(())
    }
    pub(super) fn file_with<D: Disk>(&mut self, disk: &mut D, path: &Path, bytes: &[u8]) -> Result<D::File, String> {
        let mut file = disk.create(path, 0o600).map_err(|why| format!("{}: {why}", path.display()))?;
        self.files.push(path.into());
        disk.write(&mut file, bytes).map_err(|why| format!("{}: {why}", path.display()))?;
        Ok(file)
    }
    pub(super) fn directory(&mut self, disk: &mut impl Disk, path: &Path) -> Result<(), String> {
        let mut missing = Vec::new();
        for parent in path.ancestors() {
            match disk.metadata(parent) {
                Ok(_) => break,
                Err(why) if why.kind() == io::ErrorKind::NotFound => missing.push(parent.to_path_buf()),
                Err(why) => return Err(format!("{}: {why}", parent.display())),
            }
        }
        // Record before creation, so a partially successful mkdir is also undone.
        self.directories.extend(missing.into_iter().rev());
        match disk.metadata(path) {
            Ok(_) => {}
            Err(why) if why.kind() == io::ErrorKind::NotFound => {
                disk.create_directories(path, 0o700).map_err(|why| format!("{}: {why}", path.display()))?;
            }
            Err(why) => return Err(format!("{}: {why}", path.display())),
        }
        temper_shell::directories::private_root_with(disk.user(), disk.metadata(path).map(|metadata| metadata.stat))
            .map_err(|why| format!("{}: {why}", path.display()))
    }
    pub(super) fn rollback_with(self, disk: &mut impl Disk) {
        for path in self.files.into_iter().rev() {
            let _removed = disk.remove_file(&path);
        }
        for path in self.directories.into_iter().rev() {
            let _removed = disk.remove_directory(&path);
        }
    }
}
