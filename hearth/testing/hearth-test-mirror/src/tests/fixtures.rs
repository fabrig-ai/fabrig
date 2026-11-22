pub(super) use crate::testing::{PIN, SNAPSHOT, XZ_SHA256, laid};
use skein_scratch::paths;
use std::{fs, path::Path};

pub fn repository(name: &str) -> std::path::PathBuf {
    paths::tree().join("testing/repos").join(name)
}
pub fn files(root: &Path) -> Vec<String> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else {
                files.push(path.strip_prefix(root).unwrap().to_str().unwrap().to_owned());
            }
        }
    }
    files.sort();
    files
}
