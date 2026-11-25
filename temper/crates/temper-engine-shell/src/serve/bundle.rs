//! Reading `application.clients.bundle`'s directory whole at start, within `clients.bundle_files` and
//! `clients.bundle_bytes` (client/web/serving.md, 1). Copied from jig's testing/jig-test/src/bundle.rs, with
//! temper's bounds: the page counts among the files, each file is read at the bytes left, and a directory is
//! read in name order.
use skein_lib::Map;
use skein_shell::Unread;
use std::{
    ffi::{OsStr, OsString},
    fs, io,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};
use temper_engine_service::{
    clients::bundle::{Bundle, File, NAME_BYTES, compressed_of, kind},
    limits::Clients,
};

/// The digest of the bundle this binary holds (client/web/bundle.md, 1); `None` built with `TEMPER_INTERNAL_BUNDLE=none`.
pub const OWN: Option<&str> = temper_client_bundle::DIGEST;

fn refusal(shown: &Path, why: impl std::fmt::Display) -> String {
    format!("application.clients.bundle: {}: {why}", shown.display())
}
struct Entry<K> {
    name: OsString,
    // A kind is asked for only after the listing fits and the traversal reaches the entry.
    kind: K,
}

enum EntryKind {
    Directory,
    File,
    Other,
}
impl From<fs::FileType> for EntryKind {
    fn from(kind: fs::FileType) -> Self {
        if kind.is_dir() {
            Self::Directory
        } else if kind.is_file() {
            Self::File
        } else {
            Self::Other
        }
    }
}

/// The directory's entries in name order, at most twice `bundle_files` of them in all.
fn entries<K>(
    shown: &Path,
    listing: io::Result<impl Iterator<Item = io::Result<Entry<K>>>>,
    count: &mut u32,
    limits: Clients,
) -> Result<Vec<Entry<K>>, String> {
    let mut found = Vec::new();
    for entry in listing.map_err(|why| refusal(shown, why))? {
        let entry = entry.map_err(|why| refusal(shown, why))?;
        *count += 1;
        if *count > limits.bundle_files.saturating_mul(2) {
            return Err(refusal(&shown.join(&entry.name), "too many entries"));
        }
        found.push(entry);
    }
    found.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(found)
}
type Forms = Vec<(Box<[u8]>, Box<[u8]>)>;

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
    fn read(
        &mut self,
        shown: &Path,
        read: &impl Fn(&Path, u32) -> Result<Box<[u8]>, Unread>,
    ) -> Result<Box<[u8]>, String> {
        let left = self.left(shown)?;
        let content = read(shown, left).map_err(|why| match why {
            Unread::TooLarge { .. } => refusal(shown, "past clients.bundle_bytes"),
            why @ (Unread::Absent | Unread::NotAFile | Unread::Unsafe(_) | Unread::Kernel(_)) => {
                refusal(shown, super::unread(why))
            }
        })?;
        self.account(shown, content.len())?;
        Ok(content)
    }
}
/// The digest directory's name under `shown`, the only directory beside the page.
fn digest<K: FnOnce() -> io::Result<EntryKind>>(
    shown: &Path,
    listing: io::Result<impl Iterator<Item = io::Result<Entry<K>>>>,
    limits: Clients,
) -> Result<Box<[u8]>, String> {
    let mut found = None;
    let mut count = 0;
    for entry in entries(shown, listing, &mut count, limits)? {
        let at = shown.join(&entry.name);
        match (entry.kind)().map_err(|why| refusal(&at, why))? {
            EntryKind::Directory => choose_digest(&at, entry.name.as_bytes(), &mut found)?,
            EntryKind::File => (),
            EntryKind::Other => return Err(refusal(&at, "entry must be a regular file or directory")),
        }
    }
    found.ok_or_else(|| refusal(shown, "no digest directory"))
}

/// Read the configured path's bundle whole, within `limits`; `Err` is the refusal's line.
pub fn read(configured: &[u8], limits: Clients) -> Result<Bundle, String> {
    read_with(
        configured,
        limits,
        |path| {
            Ok(fs::read_dir(path)?.map(|entry| {
                entry.map(|entry| Entry {
                    name: entry.file_name(),
                    kind: move || entry.file_type().map(EntryKind::from),
                })
            }))
        },
        skein_shell::read_file,
    )
}

/// Supply entries in listing order, deferring kind reads until after the cap and sort.
fn read_with<I, K>(
    configured: &[u8],
    limits: Clients,
    list: impl Fn(&Path) -> io::Result<I>,
    read_file: impl Fn(&Path, u32) -> Result<Box<[u8]>, Unread>,
) -> Result<Bundle, String>
where
    I: Iterator<Item = io::Result<Entry<K>>>,
    K: FnOnce() -> io::Result<EntryKind>,
{
    let shown = Path::new(OsStr::from_bytes(configured));
    let mut state = Files { limits, bytes: 0, count: 0 };
    let page = state.read(&shown.join("index.html"), &read_file)?;
    let digest = digest(shown, list(shown), limits)?;
    let shown_tree = shown.join(OsStr::from_bytes(&digest));
    let mut files = Map::with_capacity(limits.bundle_files);
    let mut forms = Forms::new();
    let mut pending = vec![PathBuf::new()];
    let mut count = 0;
    while let Some(relative) = pending.pop() {
        let mut directories = Vec::new();
        let directory = shown_tree.join(&relative);
        for entry in entries(&directory, list(&directory), &mut count, limits)? {
            let name = relative.join(&entry.name);
            let at = shown_tree.join(&name);
            plain(&at, name.as_os_str().as_bytes())?;
            match (entry.kind)().map_err(|why| refusal(&at, why))? {
                EntryKind::Directory => directories.push(name),
                EntryKind::File => {
                    let key = name.to_str().ok_or_else(|| refusal(&at, "path is not UTF-8"))?.as_bytes();
                    let typed = file_kind(&at, compressed_of(key).unwrap_or(key))?;
                    let content = state.read(&at, &read_file)?;
                    if compressed_of(key).is_some() {
                        forms.push((Box::from(key), content));
                    } else {
                        files
                            .insert(Box::from(key), File { kind: typed, bytes: content, gzip: None })
                            .expect("file bound checked");
                    }
                }
                EntryKind::Other => return Err(refusal(&at, "entry must be a regular file or directory")),
            }
        }
        pending.extend(directories.into_iter().rev());
    }
    pair(&shown_tree, &mut files, forms)?;
    Ok(finish(page, digest, files))
}

/// Read the chosen bundle and add the start's metadata before adopting it.
pub(super) fn prepare(
    configured: Option<&[u8]>,
    limits: Clients,
    google: bool,
    record: bool,
    read: impl FnOnce(&[u8], Clients) -> Result<Bundle, String>,
) -> Result<Option<Bundle>, String> {
    let mut bundle = match configured {
        Some(path) => Some(read(path, limits)?),
        None => own(limits)?,
    };
    if let Some(bundle) = &mut bundle {
        let missing = |kind| {
            let path = configured.map_or("embedded".into(), String::from_utf8_lossy);
            format!("application.clients.bundle: {path}: the page has no </head> for its {kind} metadata")
        };
        bundle.fill_sign_in(google).ok_or_else(|| missing("sign-in"))?;
        if record {
            bundle.fill_record().ok_or_else(|| missing("recording"))?;
        }
    }
    Ok(bundle)
}

/// Attach forms only after all plain files have been read, independent of entry order.
fn pair(shown: &Path, files: &mut Map<Box<[u8]>, File>, forms: Forms) -> Result<(), String> {
    for (name, bytes) in forms {
        let at = shown.join(std::ffi::OsStr::from_bytes(&name));
        let plain = compressed_of(&name).expect("a gathered compressed form");
        let file = files.get_mut(plain).ok_or_else(|| refusal(&at, "a compressed form with no file"))?;
        file.gzip = Some(bytes);
    }
    Ok(())
}

fn finish(page: Box<[u8]>, digest: Box<[u8]>, files: Map<Box<[u8]>, File>) -> Bundle {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in &page {
        hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
    }
    Bundle {
        page,
        forge_meta_bytes: 0,
        etag: format!("\"{hash:016x}\"").into_bytes().into_boxed_slice(),
        digest,
        files,
    }
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

/// Read the binary's files through the directory read's checks (client/web/bundle.md, 1).
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
    let mut forms = Forms::new();
    for (name, bytes) in files {
        let Some((_, relative)) = name.split_once('/') else { continue };
        let at = shown.join(name);
        let key = relative.as_bytes();
        plain(&at, key)?;
        let typed = file_kind(&at, compressed_of(key).unwrap_or(key))?;
        let bytes = state.copy(&at, bytes)?;
        if compressed_of(key).is_some() {
            forms.push((Box::from(key), bytes));
        } else {
            contents.insert(Box::from(key), File { kind: typed, bytes, gzip: None }).expect("file bound checked");
        }
    }
    pair(&shown.join(std::ffi::OsStr::from_bytes(&digest)), &mut contents, forms)?;
    Ok(finish(page, digest, contents))
}

/// The binary's own bundle, absent when built without its browser (client/web/bundle.md, 1).
pub fn own(limits: Clients) -> Result<Option<Bundle>, String> {
    if temper_client_bundle::FILES.is_empty() {
        Ok(None)
    } else {
        embedded(temper_client_bundle::FILES, limits).map(Some)
    }
}

#[cfg(test)]
mod tests;
