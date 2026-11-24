//! Configuration discovery and machine facts (file.md, 6, steps 1–3).
use crate::{FILE, FILE_BYTES, FORMER, Facts, Read, Refusal, Shape, binary, parse};
use std::{io::ErrorKind, os::unix::ffi::OsStrExt, path::Path};
use temper_shell::directories::{Directories, unread};

pub fn read(directories: &Directories, shape: Shape) -> Result<Read, Refusal> {
    let config = Path::new(std::ffi::OsStr::from_bytes(&directories.config));
    read_with(
        shape,
        || std::fs::symlink_metadata(config.join(FORMER)).map(|_| ()),
        |max| skein_shell::read_file(&config.join(FILE), max),
        || facts(directories),
    )
}

/// Discover and read the configuration over supplied file and machine facts.
pub fn read_with(
    shape: Shape,
    former: impl FnOnce() -> std::io::Result<()>,
    current: impl FnOnce(u32) -> Result<Box<[u8]>, skein_shell::Unread>,
    machine_facts: impl FnOnce() -> Result<Facts, Refusal>,
) -> Result<Read, Refusal> {
    match former() {
        Ok(()) => {
            return Err(Refusal::one(format!(
                "{FORMER}: moved: the configuration is {FILE}; write it from {} help config, then remove {FORMER}",
                binary(shape)
            )));
        }
        Err(why) if why.kind() == ErrorKind::NotFound => {}
        Err(why) => return Err(Refusal::one(format!("{FORMER}: {why}"))),
    }
    let text = match current(u32::try_from(FILE_BYTES).expect("configuration bound fits u32")) {
        Ok(text) => text,
        Err(skein_shell::Unread::Absent) => {
            let words = if shape == Shape::Standalone {
                "temper-box init writes one".into()
            } else {
                format!("write it from {} help config", binary(shape))
            };
            return Err(Refusal::one(format!("{FILE}: absent; {words}")));
        }
        Err(why) => return Err(Refusal::one(format!("{FILE}: {}", unread(why)))),
    };
    parse(&text, shape, &machine_facts()?)
}
pub fn facts(directories: &Directories) -> Result<Facts, Refusal> {
    let login = login(skein_shell::effective_user()).map_err(Refusal::one)?;
    let hostname = skein_shell::host_name().map_err(|errno| Refusal::one(format!("hostname: errno {errno}")))?;
    let hostname = std::str::from_utf8(&hostname).map_err(|why| Refusal::one(format!("hostname: {why}")))?.to_owned();
    let data = Path::new(std::ffi::OsStr::from_bytes(&directories.data));
    let provider_key = match std::fs::symlink_metadata(data.join("provider.key")) {
        Ok(_) => true,
        Err(why) if why.kind() == ErrorKind::NotFound => false,
        Err(why) => return Err(Refusal::one(format!("provider.key: {why}"))),
    };
    Ok(Facts { login, hostname, provider_key })
}
fn login(user: u32) -> Result<String, String> {
    let bytes = skein_shell::login(user)?;
    String::from_utf8(bytes.into_vec()).map_err(|why| format!("user: {why}"))
}
