//! Retained forge state (temper/docs/design/testing/fakes.md, section 2).
use skein_fake_peers::forge::Peer;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
};

fn safe(metadata: &fs::Metadata) -> bool {
    metadata.is_file()
        && !metadata.file_type().is_symlink()
        && metadata.uid() == skein_shell::effective_user()
        && metadata.mode().trailing_zeros() >= 6
        && metadata.nlink() == 1
}
fn read(path: &Path) -> Result<Option<Box<[u8]>>, String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(why) if why.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(why) => return Err(why.to_string()),
    };
    if !safe(&metadata) {
        return Err("expected this user's private single-link regular output".into());
    }
    let file = File::open(path).map_err(|why| why.to_string())?;
    let opened = file.metadata().map_err(|why| why.to_string())?;
    let named = fs::symlink_metadata(path).map_err(|why| why.to_string())?;
    if !safe(&opened)
        || !safe(&named)
        || opened.dev() != named.dev()
        || opened.ino() != named.ino()
        || opened.dev() != metadata.dev()
        || opened.ino() != metadata.ino()
    {
        return Err("output changed while opening".into());
    }
    // Reader's address space bounds an image. Read one extra byte to reject growth.
    let length = u32::try_from(opened.len()).map_err(|_| "image length".to_string())?;
    let mut bytes = Vec::new();
    file.take(u64::from(length).saturating_add(1)).read_to_end(&mut bytes).map_err(|why| why.to_string())?;
    if skein_lib::len_u64(bytes.len()) != opened.len() {
        return Err("output changed while reading".into());
    }
    Ok(Some(bytes.into_boxed_slice()))
}
pub(super) fn restore(path: &Path, peer: &mut Peer) -> Result<(), String> {
    let refused = |why| format!("state {} refused: {why}", path.display());
    if let Some(bytes) = read(path).map_err(refused)? {
        peer.restore(&bytes).map_err(|why| refused(format!("{why:?}")))?;
    }
    Ok(())
}
pub(super) fn store(path: &Path, peer: &Peer) -> Result<(), String> {
    let image = peer.image().map_err(|why| format!("state {}: {why:?}", path.display()))?;
    let mut name =
        path.file_name().ok_or_else(|| format!("state {}: expected a file name", path.display()))?.to_os_string();
    name.push(format!(".{}.tmp", temper_forge_setup_shell::process::id()));
    let temporary = path.with_file_name(name);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)
        .map_err(|why| format!("state {}: {why}", path.display()))?;
    let result = file.write_all(&image).and_then(|()| file.sync_all()).and_then(|()| fs::rename(&temporary, path));
    if let Err(why) = result {
        let _removed = fs::remove_file(&temporary);
        return Err(format!("state {}: {why}", path.display()));
    }
    Ok(())
}
