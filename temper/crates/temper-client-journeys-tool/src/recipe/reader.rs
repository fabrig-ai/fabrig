//! Only the reader touches the directory; the grammar and hash take its bytes.
use std::{ffi::OsString, fs, io::Read, path::Path};

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
            Self::Memory(files) => files.get(name).map(|files| files.keys().cloned().collect()).ok_or("missing directory".into()),
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
            Self::Memory(files) => files.get(name).and_then(|files| files.get(std::ffi::OsStr::new(file))).cloned().ok_or("missing file".into()),
        }
    }
    pub(super) fn missing(&self, name: &str) -> bool {
        match self {
            Self::Directory(setups) => !setups.join(name).join("recipe").exists(),
            #[cfg(test)]
            Self::Memory(files) => !files.get(name).is_some_and(|files| files.contains_key(std::ffi::OsStr::new("recipe"))),
        }
    }
}
