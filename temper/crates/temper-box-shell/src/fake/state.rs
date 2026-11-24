//! Retained forge state (temper/docs/design/testing/fakes.md, section 2).
use super::disk::{Disk, UNSAFE_OUTPUT, opened};
use skein_fake_peers::forge::Peer;
use std::{io, path::Path};

fn read(disk: &mut impl Disk, path: &Path) -> Result<Option<Box<[u8]>>, String> {
    let metadata = match disk.metadata(path) {
        Ok(metadata) => metadata,
        Err(why) if why.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(why) => return Err(why.to_string()),
    };
    if !metadata.safe(disk.user()) {
        return Err(UNSAFE_OUTPUT.into());
    }
    let (file, opened) = opened(disk, path, metadata)?;
    // Reader's address space bounds an image. Read one extra byte to reject growth.
    let length = u32::try_from(opened.stat.size).map_err(|_| "image length".to_string())?;
    let bytes = disk.read(file, u64::from(length) + 1).map_err(|why| why.to_string())?;
    if skein_lib::len_u64(bytes.len()) != opened.stat.size {
        return Err("output changed while reading".into());
    }
    Ok(Some(bytes))
}
pub(super) fn restore(disk: &mut impl Disk, path: &Path, peer: &mut Peer) -> Result<(), String> {
    let refused = |why| format!("state {} refused: {why}", path.display());
    if let Some(bytes) = read(disk, path).map_err(refused)? {
        peer.restore(&bytes).map_err(|why| refused(format!("{why:?}")))?;
    }
    Ok(())
}
pub(super) fn store(disk: &mut impl Disk, path: &Path, peer: &Peer, process: u32) -> Result<(), String> {
    let image = peer.image().map_err(|why| format!("state {}: {why:?}", path.display()))?;
    let mut name =
        path.file_name().ok_or_else(|| format!("state {}: expected a file name", path.display()))?.to_os_string();
    name.push(format!(".{process}.tmp"));
    let temporary = path.with_file_name(name);
    let mut file = disk.create(&temporary, 0o600).map_err(|why| format!("state {}: {why}", path.display()))?;
    let result =
        disk.write(&mut file, &image).and_then(|()| disk.sync(&file)).and_then(|()| disk.rename(&temporary, path));
    if let Err(why) = result {
        let _removed = disk.remove_file(&temporary);
        return Err(format!("state {}: {why}", path.display()));
    }
    Ok(())
}
