//! Seed outputs owned by one fake-peer start (temper/docs/design/protocol/setup/box.md, 5).
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct Created {
    pub files: Vec<PathBuf>,
    directories: Vec<PathBuf>,
}
impl Created {
    pub fn output(&mut self, path: &Path, expected: &[u8]) -> Result<(), String> {
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(why) if why.kind() == io::ErrorKind::NotFound => {
                self.file(path, expected)?;
                return Ok(());
            }
            Err(why) => return Err(format!("{}: {why}", path.display())),
        };
        let safe = |metadata: &fs::Metadata| {
            metadata.is_file()
                && !metadata.file_type().is_symlink()
                && metadata.uid() == skein_shell::effective_user()
                // No group or other permission bits.
                && metadata.mode().trailing_zeros() >= 6
                && metadata.nlink() == 1
        };
        if !safe(&metadata) {
            return Err(format!("{}: expected this user's private single-link regular output", path.display()));
        }
        let file = File::open(path).map_err(|why| format!("{}: {why}", path.display()))?;
        let opened = file.metadata().map_err(|why| format!("{}: {why}", path.display()))?;
        let named = fs::symlink_metadata(path).map_err(|why| format!("{}: {why}", path.display()))?;
        if !safe(&opened)
            || !safe(&named)
            || opened.dev() != named.dev()
            || opened.ino() != named.ino()
            || opened.dev() != metadata.dev()
            || opened.ino() != metadata.ino()
        {
            return Err(format!("{}: output changed while opening", path.display()));
        }
        let bound = skein_lib::len_u64(expected.len()).checked_add(1).expect("output bound");
        let mut content = Vec::new();
        file.take(bound).read_to_end(&mut content).map_err(|why| format!("{}: {why}", path.display()))?;
        if content != expected {
            return Err(format!("{}: retained output differs from expected seeded bytes", path.display()));
        }
        Ok(())
    }
    pub fn file(&mut self, path: &Path, bytes: &[u8]) -> Result<File, String> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .map_err(|why| format!("{}: {why}", path.display()))?;
        self.files.push(path.into());
        file.write_all(bytes).map_err(|why| format!("{}: {why}", path.display()))?;
        Ok(file)
    }
    pub(super) fn directory(&mut self, path: &Path) -> Result<(), String> {
        let mut missing = Vec::new();
        for parent in path.ancestors() {
            match fs::symlink_metadata(parent) {
                Ok(_) => break,
                Err(why) if why.kind() == io::ErrorKind::NotFound => missing.push(parent.to_path_buf()),
                Err(why) => return Err(format!("{}: {why}", parent.display())),
            }
        }
        // Record before creation, so a partially successful mkdir is also undone.
        self.directories.extend(missing.into_iter().rev());
        crate::directories::make_private(path, skein_shell::effective_user())
    }
    pub fn rollback(self) {
        for path in self.files.into_iter().rev() {
            let _removed = fs::remove_file(path);
        }
        for path in self.directories.into_iter().rev() {
            let _removed = fs::remove_dir(path);
        }
    }
}
