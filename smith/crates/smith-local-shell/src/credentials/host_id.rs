//! The installation UUID, drawn only by login (shell/credentials.md, 2).
//! Absent-file stores choose one winner; every other process reads that winner.
use super::{Directory, private_file};
use skein_io::file::Expect;
use std::fmt::Write;

pub(super) fn read_or_draw(directory: &Directory, entropy: &[u8; 32]) -> Result<String, String> {
    let file = directory.host_id();
    read_or_draw_with(
        entropy,
        || skein_shell::read_private_file(&file, 45, skein_shell::effective_user()),
        |bytes| private_file::store(&file, bytes, Expect::Absent, entropy).map(|_| ()),
    )
}

fn read_or_draw_with(
    entropy: &[u8; 32],
    mut read_bytes: impl FnMut() -> Result<skein_lib::Secret, skein_shell::Unread>,
    publish: impl FnOnce(Box<[u8]>) -> Result<(), String>,
) -> Result<String, String> {
    if let Some(value) = read(read_bytes())? {
        return Ok(value);
    }
    let mut bytes: [u8; 16] = entropy.get(..16).expect("UUID entropy").try_into().expect("sixteen bytes");
    *bytes.get_mut(6).expect("version byte") = (bytes.get(6).expect("version byte") & 0x0f) | 0x40;
    *bytes.get_mut(8).expect("variant byte") = (bytes.get(8).expect("variant byte") & 0x3f) | 0x80;
    let mut value = String::from("urn:uuid:");
    for (index, byte) in bytes.iter().enumerate() {
        if [4, 6, 8, 10].contains(&index) {
            value.push('-');
        }
        write!(value, "{byte:02x}").expect("UUID string");
    }
    publish(value.into_bytes().into_boxed_slice())?;
    read(read_bytes())?.ok_or_else(|| "unreadable host-id".into())
}

fn read(bytes: Result<skein_lib::Secret, skein_shell::Unread>) -> Result<Option<String>, String> {
    match bytes {
        Ok(bytes) => {
            let value = std::str::from_utf8(bytes.expose()).map_err(|_| "unreadable host-id")?;
            let uuid = value.strip_prefix("urn:uuid:").ok_or("unreadable host-id")?;
            if uuid.len() != 36
                || !uuid.bytes().enumerate().all(|(index, byte)| {
                    if [8, 13, 18, 23].contains(&index) { byte == b'-' } else { byte.is_ascii_hexdigit() }
                })
            {
                return Err("unreadable host-id".into());
            }
            Ok(Some(value.into()))
        }
        Err(skein_shell::Unread::Absent) => Ok(None),
        Err(_) => Err("unreadable host-id".into()),
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod process_tests;
