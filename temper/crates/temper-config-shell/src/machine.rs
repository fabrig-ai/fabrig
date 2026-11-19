//! Configuration discovery and machine facts (file.md, 6, steps 1–3).
use crate::{FILE, FORMER, Facts, Read, Refusal, Shape, binary, parse};
use std::{io::ErrorKind, os::unix::ffi::OsStrExt, path::Path};
use temper_shell::directories::{Directories, unread};

pub fn read(directories: &Directories, shape: Shape) -> Result<Read, Refusal> {
    let config = Path::new(std::ffi::OsStr::from_bytes(&directories.config));
    match std::fs::symlink_metadata(config.join(FORMER)) {
        Ok(_) => {
            return Err(Refusal::one(format!(
                "{FORMER}: moved: the configuration is {FILE}; write it from {} help config, then remove {FORMER}",
                binary(shape)
            )));
        }
        Err(why) if why.kind() == ErrorKind::NotFound => {}
        Err(why) => return Err(Refusal::one(format!("{FORMER}: {why}"))),
    }
    let text = match skein_shell::read_file(&config.join(FILE), 1 << 20) {
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
    parse(&text, shape, &facts(directories)?)
}
pub(crate) fn facts(directories: &Directories) -> Result<Facts, Refusal> {
    let login = login(skein_shell::effective_user()).map_err(Refusal::one)?;
    let hostname = skein_shell::read_file(Path::new("/proc/sys/kernel/hostname"), 4096)
        .map_err(|why| Refusal::one(format!("hostname: {}", unread(why))))?;
    let hostname = hostname.strip_suffix(b"\n").unwrap_or(&hostname);
    let hostname = std::str::from_utf8(hostname).map_err(|why| Refusal::one(format!("hostname: {why}")))?.to_owned();
    let data = Path::new(std::ffi::OsStr::from_bytes(&directories.data));
    let provider_key = match std::fs::symlink_metadata(data.join("provider.key")) {
        Ok(_) => true,
        Err(why) if why.kind() == ErrorKind::NotFound => false,
        Err(why) => return Err(Refusal::one(format!("provider.key: {why}"))),
    };
    Ok(Facts { login, hostname, provider_key })
}
fn login(user: u32) -> Result<String, String> {
    let bytes = if cfg!(target_os = "macos") {
        temper_forge_setup_shell::process::login_id()?
    } else {
        let passwd = skein_shell::read_file(Path::new("/etc/passwd"), 1 << 20)
            .map_err(|why| format!("user: {}", unread(why)))?;
        let found = passwd.split(|b| *b == b'\n').find_map(|line| {
            let fields: Vec<_> = line.split(|b| *b == b':').collect();
            if fields.get(2).and_then(|id| std::str::from_utf8(id).ok()).and_then(|id| id.parse::<u32>().ok())
                == Some(user)
            {
                fields.first().copied().map(Box::<[u8]>::from)
            } else {
                None
            }
        });
        found.ok_or_else(|| format!("user {user}: no login in /etc/passwd"))?
    };
    String::from_utf8(bytes.into_vec()).map_err(|why| format!("user: {why}"))
}
