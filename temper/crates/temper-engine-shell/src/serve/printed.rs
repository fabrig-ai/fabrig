//! The words of a command for this machine (temper/docs/design/client/forge/records.md).

/// What a word is read from, given so that a test fakes the machine.
pub(crate) struct Machine<'a> {
    pub executable: Option<&'a [u8]>,
    pub path: Option<&'a [u8]>,
    pub home: Option<&'a [u8]>,
    /// The canonical path of an executable file; `None` for anything else.
    #[expect(clippy::type_complexity, reason = "the machine's canonical lookup takes and returns byte paths")]
    pub canonical: &'a dyn Fn(&[u8]) -> Option<Box<[u8]>>,
}
pub(crate) fn program_word(machine: &Machine<'_>) -> Box<[u8]> {
    let Some(executable) = machine.executable else { return b"temper-box".as_slice().into() };
    let executable = executable.strip_suffix(b" (deleted)").unwrap_or(executable);
    if let Some(path) = machine.path {
        let first = path.split(|byte| *byte == b':').filter(|entry| entry.starts_with(b"/")).find_map(|entry| {
            let mut candidate = entry.to_vec();
            if !candidate.ends_with(b"/") {
                candidate.push(b'/');
            }
            candidate.extend_from_slice(b"temper-box");
            (machine.canonical)(&candidate)
        });
        if let Some(first) = first
            && (machine.canonical)(executable).as_deref() == Some(first.as_ref())
        {
            return b"temper-box".as_slice().into();
        }
    }
    path_word(executable, machine.home)
}

#[must_use]
pub fn path_word(path: &[u8], home: Option<&[u8]>) -> Box<[u8]> {
    let path = collapsed(path);
    let home = home.map(collapsed);
    if let Some(home) = home {
        let home = home.as_slice();
        let home = home.strip_suffix(b"/").unwrap_or(home);
        if home.starts_with(b"/") && !home.is_empty() {
            if path == home {
                return b"~".as_slice().into();
            }
            if let Some(tail) = path.strip_prefix(home).and_then(|tail| tail.strip_prefix(b"/")) {
                let mut word = b"~/".to_vec();
                word.extend_from_slice(&quoted_if_needed(tail));
                return word.into_boxed_slice();
            }
        }
    }
    quoted_if_needed(&path)
}

fn collapsed(path: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    for byte in path {
        if *byte != b'/' || !result.ends_with(b"/") {
            result.push(*byte);
        }
    }
    result
}

fn quoted_if_needed(value: &[u8]) -> Box<[u8]> {
    if !value.is_empty() && value.iter().all(|byte| byte.is_ascii_alphanumeric() || b"_-./+,:@%".contains(byte)) {
        value.into()
    } else {
        temper_forge_setup::sign_in::quote(value)
    }
}

/// `program_word` of this process: `current_exe`, and `PATH` and `HOME` from ENVIRONMENT.
#[must_use]
pub fn running_word(environment: &[temper_shell::directories::EnvironmentEntry]) -> Box<[u8]> {
    use std::os::unix::{ffi::OsStrExt, fs::PermissionsExt};
    let executable = std::env::current_exe().ok();
    program_word(&Machine {
        executable: executable.as_ref().map(|path| path.as_os_str().as_bytes()),
        path: temper_shell::environment::PATH.value(environment),
        home: skein_shell::HOME.value(environment),
        canonical: &|bytes| {
            let path = std::path::Path::new(std::ffi::OsStr::from_bytes(bytes));
            let metadata = std::fs::metadata(path).ok()?;
            if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
                return None;
            }
            std::fs::canonicalize(path).ok().map(|path| Box::from(path.as_os_str().as_bytes()))
        },
    })
}

#[cfg(test)]
mod tests;
