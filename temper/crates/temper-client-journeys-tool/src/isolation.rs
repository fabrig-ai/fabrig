//! Recording children keep their homes and browser openers in scratch (client/web/journeys.md, 8.1).
use std::{
    ffi::{OsStr, OsString},
    fs,
    os::unix::{ffi::OsStrExt, fs::DirBuilderExt},
    path::{Path, PathBuf},
};

pub(crate) fn environment(
    given: &[crate::EnvironmentEntry],
    scratch: &Path,
) -> Result<Vec<(OsString, OsString)>, String> {
    let environment = crate::tui_driver::environment(given, scratch)
        .map_err(|why| format!("cannot build recording environment: {why}"))?;
    for directory in ["home", "state", "bin", "tmp", "config", "data", "cache", "runtime", "tui-runtime"] {
        let path = scratch.join(directory);
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&path)
            .map_err(|why| format!("cannot create {}: {why}", path.display()))?;
    }
    let true_program = program("true", given).ok_or("cannot find true on PATH")?;
    for opener in ["xdg-open", "open"] {
        let path = scratch.join("bin").join(opener);
        std::os::unix::fs::symlink(&true_program, &path)
            .map_err(|why| format!("cannot create {}: {why}", path.display()))?;
    }
    Ok(environment)
}

pub(crate) fn program(name: &str, given: &[crate::EnvironmentEntry]) -> Option<PathBuf> {
    let path = OsStr::from_bytes(crate::table::PATH.value(given).unwrap_or_default());
    std::env::split_paths(path)
        .filter(|path| path.is_absolute())
        .map(|path| path.join(name))
        .find(|path| path.is_file())
}

/// The harness clears its server's environment; pass only the isolated paths.
pub(crate) fn server_arguments(environment: &[(OsString, OsString)], scratch: &Path) -> Vec<OsString> {
    environment
        .iter()
        .filter(|(name, _)| {
            [
                "PATH",
                "HOME",
                "TMPDIR",
                "XDG_STATE_HOME",
                "XDG_CONFIG_HOME",
                "XDG_DATA_HOME",
                "XDG_CACHE_HOME",
                "XDG_RUNTIME_DIR",
            ]
            .iter()
            .any(|forward| name == forward)
        })
        .map(|(name, value)| {
            let mut entry = name.clone();
            entry.push("=");
            if name == "XDG_RUNTIME_DIR" {
                entry.push(scratch.join("runtime"));
            } else {
                entry.push(value);
            }
            entry
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_recording_server_has_an_isolated_home_and_openers() {
        let scratch = skein_scratch::Scratch::new("recording-isolation");
        let given = vec![
            (b"PATH".as_slice().into(), b"/usr/bin:/bin".as_slice().into()),
            (b"HOME".as_slice().into(), b"/outside-home".as_slice().into()),
            (b"XDG_RUNTIME_DIR".as_slice().into(), b"/outside-runtime".as_slice().into()),
            (b"UNRELATED".as_slice().into(), b"not-for-the-server".as_slice().into()),
        ];
        let isolated = environment(&given, scratch.path()).expect("isolated environment");
        let server: std::collections::BTreeMap<_, _> = server_arguments(&isolated, scratch.path())
            .into_iter()
            .map(|argument| {
                let argument = argument.into_string().expect("test paths are UTF-8");
                let (name, value) = argument.split_once('=').expect("environment assignment");
                (name.to_owned(), value.to_owned())
            })
            .collect();
        assert_eq!(server.get("HOME").map(Path::new), Some(scratch.path().join("home").as_path()));
        assert_eq!(server.get("XDG_RUNTIME_DIR").map(Path::new), Some(scratch.path().join("runtime").as_path()));
        assert!(!server.contains_key("UNRELATED"), "only the server's path environment is forwarded");
        for (name, value) in &server {
            if name != "PATH" {
                assert!(Path::new(value).starts_with(scratch.path()), "{name} stays in scratch");
                assert!(Path::new(value).is_dir(), "{name} names a prepared directory");
            }
        }
        let path = server.get("PATH").expect("server PATH");
        let first = std::env::split_paths(path).next().expect("opener directory");
        assert_eq!(first, scratch.path().join("bin"));
        for opener in ["xdg-open", "open"] {
            assert_eq!(
                fs::read_link(first.join(opener)).expect("no-op opener"),
                program("true", &given).expect("true")
            );
        }
        assert_ne!(
            isolated.iter().find(|(name, _)| name == "XDG_RUNTIME_DIR").expect("terminal runtime").1,
            OsString::from(server.get("XDG_RUNTIME_DIR").expect("server runtime")),
            "terminal and server runtime directories stay separate"
        );
    }
}
