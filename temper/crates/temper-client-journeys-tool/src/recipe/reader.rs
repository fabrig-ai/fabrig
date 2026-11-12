//! Only the reader touches the directory; the grammar and hash take its bytes.
use super::Refusal;
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    io::Read,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};
pub(super) type CommitFiles = BTreeMap<Box<[u8]>, Vec<u8>>;

pub(super) enum Reader<'a> {
    Directory(&'a Path),
    #[cfg(test)]
    Memory(&'a std::collections::BTreeMap<String, std::collections::BTreeMap<OsString, Vec<u8>>>),
}
impl Reader<'_> {
    pub(super) fn files(&self, name: &str) -> Result<Vec<OsString>, String> {
        match self {
            Self::Directory(setups) => fs::read_dir(setups.join(name))
                .map_err(|error| error.to_string())?
                .map(|entry| entry.map(|entry| entry.file_name()).map_err(|error| error.to_string()))
                .collect(),
            #[cfg(test)]
            Self::Memory(files) => files
                .get(name)
                .map(|files| {
                    files
                        .keys()
                        .map(|file| Path::new(file).components().next().expect("file name").as_os_str().to_owned())
                        .collect::<std::collections::BTreeSet<_>>()
                        .into_iter()
                        .collect()
                })
                .ok_or("missing directory".into()),
        }
    }
    pub(super) fn bytes(&self, name: &str, file: &str) -> Result<Vec<u8>, String> {
        match self {
            Self::Directory(setups) => {
                let mut bytes = vec![];
                let file_handle = fs::File::open(setups.join(name).join(file)).map_err(|error| error.to_string())?;
                if file == "recipe" {
                    file_handle.take(65537).read_to_end(&mut bytes).map_err(|error| error.to_string())?;
                } else {
                    let mut file_handle = file_handle;
                    file_handle.read_to_end(&mut bytes).map_err(|error| error.to_string())?;
                }
                Ok(bytes)
            }
            #[cfg(test)]
            Self::Memory(files) => files
                .get(name)
                .and_then(|files| files.get(std::ffi::OsStr::new(file)))
                .cloned()
                .ok_or("missing file".into()),
        }
    }
    pub(super) fn commit_files(&self, name: &str) -> Result<CommitFiles, Refusal> {
        let mut files = CommitFiles::new();
        match self {
            Self::Directory(setups) => {
                let root = setups.join(name).join("files");
                if root.exists() {
                    directory_files(name, &root, &mut files)?;
                }
            }
            #[cfg(test)]
            Self::Memory(recipes) => {
                for (path, bytes) in recipes.get(name).expect("recipe read") {
                    if let Some(path) = path.as_bytes().strip_prefix(b"files/") {
                        insert(name, path, bytes.clone(), &mut files)?;
                    }
                }
            }
        }
        Ok(files)
    }
    pub(super) fn missing(&self, name: &str) -> bool {
        match self {
            Self::Directory(setups) => !setups.join(name).join("recipe").exists(),
            #[cfg(test)]
            Self::Memory(files) => {
                !files.get(name).is_some_and(|files| files.contains_key(std::ffi::OsStr::new("recipe")))
            }
        }
    }
}

fn insert(name: &str, path: &[u8], bytes: Vec<u8>, files: &mut CommitFiles) -> Result<(), Refusal> {
    let file = format!("files/{}", String::from_utf8_lossy(path));
    if files.len() >= 64 {
        return Err(Refusal::file(name, &file, "more than 64 files"));
    }
    if bytes.len() > 65536 {
        return Err(Refusal::file(name, &file, "more than 64 KiB"));
    }
    files.insert(path.into(), bytes);
    Ok(())
}
fn directory_files(name: &str, root: &Path, files: &mut CommitFiles) -> Result<(), Refusal> {
    let mut pending = vec![PathBuf::new()];
    while let Some(relative) = pending.pop() {
        let directory = root.join(&relative);
        let entries = fs::read_dir(&directory)
            .map_err(|why| Refusal::file(name, &format!("files/{}", relative.display()), why.to_string()))?;
        for entry in entries {
            let entry = entry.map_err(|why| Refusal::file(name, "files", why.to_string()))?;
            let relative = relative.join(entry.file_name());
            let file = format!("files/{}", relative.display());
            let refuse = |why: std::io::Error| Refusal::file(name, &file, why.to_string());
            let kind = entry.file_type().map_err(refuse)?;
            if kind.is_dir() {
                pending.push(relative);
                continue;
            }
            if !kind.is_file() {
                return Err(Refusal::file(name, &file, "expected a regular file"));
            }
            let mut bytes = vec![];
            fs::File::open(entry.path()).map_err(refuse)?.take(65537).read_to_end(&mut bytes).map_err(refuse)?;
            insert(name, relative.as_os_str().as_bytes(), bytes, files)?;
        }
    }
    Ok(())
}
