//! Reading `application.clients.bundle`'s directory whole at start, within `clients.bundle_files` and
//! `clients.bundle_bytes` (client/browser.md, 2.2). Copied from jig's testing/jig-test/src/bundle.rs, with
//! temper's bounds: the page counts among the files, each file is read at the bytes left, and a directory is
//! read in name order.
use skein_lib::Map;
use skein_shell::Unread;
use std::{
    fs,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};
use temper_service::{
    clients::bundle::{Bundle, File, NAME_BYTES, kind},
    limits::Clients,
};

/// The digest of the bundle this binary holds (client/browser.md, 2.6); `None` built with `TEMPER_BUNDLE=none`.
pub const OWN: Option<&str> = temper_client_bundle::DIGEST;

fn refusal(shown: &Path, why: impl std::fmt::Display) -> String {
    format!("application.clients.bundle: {}: {why}", shown.display())
}
/// The directory's entries in name order, at most twice `bundle_files` of them in all.
fn entries(shown: &Path, directory: &Path, count: &mut u32, limits: Clients) -> Result<Vec<fs::DirEntry>, String> {
    let mut found = Vec::new();
    for entry in fs::read_dir(directory).map_err(|why| refusal(shown, why))? {
        let entry = entry.map_err(|why| refusal(shown, why))?;
        *count += 1;
        if *count > limits.bundle_files.saturating_mul(2) {
            return Err(refusal(&shown.join(entry.file_name()), "too many entries"));
        }
        found.push(entry);
    }
    found.sort_by_key(fs::DirEntry::file_name);
    Ok(found)
}
struct Files {
    limits: Clients,
    bytes: u32,
    count: u32,
}
impl Files {
    fn left(&self, shown: &Path) -> Result<u32, String> {
        if self.count >= self.limits.bundle_files {
            return Err(refusal(shown, "past clients.bundle_files"));
        }
        Ok(self.limits.bundle_bytes.saturating_sub(self.bytes))
    }
    fn account(&mut self, shown: &Path, size: usize) -> Result<(), String> {
        let left = self.left(shown)?;
        let size = u32::try_from(size)
            .ok()
            .filter(|size| *size <= left)
            .ok_or_else(|| refusal(shown, "past clients.bundle_bytes"))?;
        self.bytes += size;
        self.count += 1;
        Ok(())
    }
    fn copy(&mut self, shown: &Path, content: &[u8]) -> Result<Box<[u8]>, String> {
        self.account(shown, content.len())?;
        Ok(Box::from(content))
    }
    /// One file, read at the bytes left; `shown` names it in a refusal.
    fn read(&mut self, shown: &Path, path: &Path) -> Result<Box<[u8]>, String> {
        let left = self.left(shown)?;
        let content = skein_shell::read_file(path, left).map_err(|why| match why {
            Unread::TooLarge { .. } => refusal(shown, "past clients.bundle_bytes"),
            why @ (Unread::Absent | Unread::NotAFile | Unread::Unsafe(_) | Unread::Kernel(_)) => {
                refusal(shown, crate::unread(why))
            }
        })?;
        self.account(shown, content.len())?;
        Ok(content)
    }
}
/// The digest directory's name under `shown`, the only directory beside the page.
fn digest(shown: &Path, root: &Path, limits: Clients) -> Result<Box<[u8]>, String> {
    let mut found = None;
    let mut count = 0;
    for entry in entries(shown, root, &mut count, limits)? {
        let at = shown.join(entry.file_name());
        let kind = entry.file_type().map_err(|why| refusal(&at, why))?;
        if kind.is_dir() {
            choose_digest(&at, entry.file_name().as_bytes(), &mut found)?;
        } else if !kind.is_file() {
            return Err(refusal(&at, "entry must be a regular file or directory"));
        }
    }
    found.ok_or_else(|| refusal(shown, "no digest directory"))
}

/// Read the bundle in `configured` (a path under `directory`) whole, within `limits`; `Err` is the refusal's
/// line.
pub fn read(configured: &[u8], limits: Clients) -> Result<Bundle, String> {
    let shown = PathBuf::from(std::ffi::OsStr::from_bytes(configured));
    let root = shown.clone();
    let mut state = Files { limits, bytes: 0, count: 0 };
    let page = state.read(&shown.join("index.html"), &root.join("index.html"))?;
    let digest = digest(&shown, &root, limits)?;
    let tree = std::ffi::OsStr::from_bytes(&digest);
    let (shown_tree, root_tree) = (shown.join(tree), root.join(tree));
    let mut files = Map::with_capacity(limits.bundle_files);
    let mut pending = vec![PathBuf::new()];
    let mut count = 0;
    while let Some(relative) = pending.pop() {
        let mut directories = Vec::new();
        for entry in entries(&shown_tree.join(&relative), &root_tree.join(&relative), &mut count, limits)? {
            let name = relative.join(entry.file_name());
            let at = shown_tree.join(&name);
            plain(&at, name.as_os_str().as_bytes())?;
            let file_type = entry.file_type().map_err(|why| refusal(&at, why))?;
            if file_type.is_dir() {
                directories.push(name);
            } else if file_type.is_file() {
                let key = name.to_str().ok_or_else(|| refusal(&at, "path is not UTF-8"))?.as_bytes();
                let typed = file_kind(&at, key)?;
                let content = state.read(&at, &root_tree.join(&name))?;
                files.insert(Box::from(key), File { kind: typed, bytes: content }).expect("file bound checked");
            } else {
                return Err(refusal(&at, "entry must be a regular file or directory"));
            }
        }
        pending.extend(directories.into_iter().rev());
    }
    Ok(finish(page, digest, files))
}

fn finish(page: Box<[u8]>, digest: Box<[u8]>, files: Map<Box<[u8]>, File>) -> Bundle {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in &page {
        hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
    }
    Bundle { page, etag: format!("\"{hash:016x}\"").into_bytes().into_boxed_slice(), digest, files }
}

/// Check each digest name once, shared by directory and embedded reads.
fn choose_digest(shown: &Path, name: &[u8], found: &mut Option<Box<[u8]>>) -> Result<(), String> {
    if found.is_some() {
        return Err(refusal(shown, "more than one digest directory"));
    }
    if name.len() != 16 || !name.iter().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) {
        return Err(refusal(shown, "digest must be 16 lowercase hex digits"));
    }
    *found = Some(Box::from(name));
    Ok(())
}
fn plain(shown: &Path, name: &[u8]) -> Result<(), String> {
    if name.len() > NAME_BYTES as usize
        || name.split(|b| *b == b'/').any(|part| part.is_empty() || part == b"." || part == b".." || part.contains(&0))
    {
        return Err(refusal(shown, "path must be plain and at most 1024 bytes"));
    }
    Ok(())
}
fn file_kind(shown: &Path, name: &[u8]) -> Result<&'static [u8], String> {
    kind(name).ok_or_else(|| refusal(shown, "file type is not on the closed list"))
}

/// Read the binary's files through the directory read's checks (client/browser.md, 2.6).
pub fn embedded(files: &[(&str, &[u8])], limits: Clients) -> Result<Bundle, String> {
    embedded_read(files, limits).map_err(|why| {
        let rest = why.strip_prefix("application.clients.bundle: bundle").expect("embedded refusal root");
        match rest.strip_prefix('/') {
            Some(path) => format!("application.clients.bundle: the binary's bundle: {path}"),
            None => format!("application.clients.bundle: the binary's bundle{rest}"),
        }
    })
}
fn embedded_read(files: &[(&str, &[u8])], limits: Clients) -> Result<Bundle, String> {
    let shown = Path::new("bundle");
    let mut state = Files { limits, bytes: 0, count: 0 };
    let page = files
        .iter()
        .find(|(name, _)| *name == "index.html")
        .ok_or_else(|| refusal(&shown.join("index.html"), "absent"))?;
    let page = state.copy(&shown.join("index.html"), page.1)?;
    let mut digest = None;
    for (name, _) in files {
        if let Some((tree, _)) = name.split_once('/')
            && digest.as_deref() != Some(tree.as_bytes())
        {
            choose_digest(&shown.join(tree), tree.as_bytes(), &mut digest)?;
        }
    }
    let digest = digest.ok_or_else(|| refusal(shown, "no digest directory"))?;
    let mut contents = Map::with_capacity(limits.bundle_files);
    for (name, bytes) in files {
        let Some((_, relative)) = name.split_once('/') else { continue };
        let at = shown.join(name);
        let key = relative.as_bytes();
        plain(&at, key)?;
        let typed = file_kind(&at, key)?;
        let bytes = state.copy(&at, bytes)?;
        contents.insert(Box::from(key), File { kind: typed, bytes }).expect("file bound checked");
    }
    Ok(finish(page, digest, contents))
}

/// The binary's own bundle, absent when built without its browser (client/browser.md, 2.6).
pub fn own(limits: Clients) -> Result<Option<Bundle>, String> {
    if temper_client_bundle::FILES.is_empty() {
        Ok(None)
    } else {
        embedded(temper_client_bundle::FILES, limits).map(Some)
    }
}
