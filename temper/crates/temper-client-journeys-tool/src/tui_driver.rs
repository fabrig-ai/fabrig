//! Shared terminal walk mechanics (client/tui/journeys.md, 8.1–8.2).
use skein_ui_screen_terminal::Key;
use std::{
    ffi::{OsStr, OsString},
    fs::File,
    io::{self, Read},
    os::unix::{ffi::OsStrExt, fs::MetadataExt},
    path::Path,
};
pub(crate) mod terminal;
/// Isolated terminal settings without choosing a fixed start (client/drive.md, 5).
pub(crate) fn unfixed_environment(
    given: &[crate::EnvironmentEntry],
    scratch: &Path,
) -> Result<Vec<(OsString, OsString)>, std::env::JoinPathsError> {
    let mut environment: Vec<_> = given
        .iter()
        .filter(|(name, _)| !name.starts_with(b"SSH_") && name.as_ref() != b"COLORFGBG")
        .map(|(name, value)| (OsStr::from_bytes(name).to_os_string(), OsStr::from_bytes(value).to_os_string()))
        .collect();
    let path = OsStr::from_bytes(crate::table::PATH.value(given).unwrap_or_default());
    let paths = std::iter::once(scratch.join("bin")).chain(std::env::split_paths(&path));
    let mut replacements = vec![
        ("PATH", std::env::join_paths(paths)?),
        ("TERM", "xterm-256color".into()),
        (temper_shell::environment::INTERNAL_RECORD.name, "1".into()),
    ];
    for (name, directory) in [
        ("HOME", "home"),
        ("XDG_STATE_HOME", "state"),
        ("XDG_CONFIG_HOME", "config"),
        ("XDG_DATA_HOME", "data"),
        ("XDG_CACHE_HOME", "cache"),
        ("XDG_RUNTIME_DIR", "tui-runtime"),
        ("TMPDIR", "tmp"),
    ] {
        replacements.push((name, scratch.join(directory).into_os_string()));
    }
    if !given.iter().any(|(name, _)| name.as_ref() == b"DISPLAY") {
        replacements.push(("DISPLAY", ":0".into()));
    }
    for (name, value) in replacements {
        environment.retain(|(key, _)| key != name);
        environment.push((name.into(), value));
    }
    Ok(environment)
}

pub(crate) fn key_bytes(key: &Key) -> Vec<u8> {
    match key {
        Key::Enter => b"\r".to_vec(),
        Key::Tab => b"\t".to_vec(),
        Key::BackTab => b"\x1b[Z".to_vec(),
        Key::Backspace => b"\x7f".to_vec(),
        Key::Delete => b"\x1b[3~".to_vec(),
        Key::Up => b"\x1b[A".to_vec(),
        Key::Down => b"\x1b[B".to_vec(),
        Key::Right => b"\x1b[C".to_vec(),
        Key::Left => b"\x1b[D".to_vec(),
        Key::Home => b"\x1b[H".to_vec(),
        Key::End => b"\x1b[F".to_vec(),
        Key::PageUp => b"\x1b[5~".to_vec(),
        Key::PageDown => b"\x1b[6~".to_vec(),
        Key::Escape => b"\x1b".to_vec(),
        Key::Control(letter) => vec![letter.to_ascii_lowercase().saturating_sub(b'a').saturating_add(1)],
        Key::Char(character) => character.to_string().into_bytes(),
        Key::Alt(character) => {
            let mut bytes = vec![0x1b];
            bytes.extend_from_slice(character.to_string().as_bytes());
            bytes
        }
        Key::Paste(text) => {
            let mut bytes = b"\x1b[200~".to_vec();
            bytes.extend_from_slice(text);
            bytes.extend_from_slice(b"\x1b[201~");
            bytes
        }
    }
}

pub(crate) fn optional_file(path: &Path) -> io::Result<Option<File>> {
    match File::open(path) {
        Ok(file) => Ok(Some(file)),
        Err(why) if why.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(why) => Err(why),
    }
}
pub(crate) fn replaced(path: &Path, earlier: Option<&File>) -> io::Result<Option<Vec<u8>>> {
    let Some(mut file) = optional_file(path)? else { return Ok(None) };
    let current = file.metadata()?;
    if let Some(earlier) = earlier {
        let earlier = earlier.metadata()?;
        if current.dev() == earlier.dev() && current.ino() == earlier.ino() {
            return Ok(None);
        }
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(Some(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_lib::Queue;
    use skein_scratch::Scratch;
    use skein_ui_screen_terminal::Decoder;
    use std::fs;
    #[test]
    fn walk_keys_are_terminal_bytes() {
        let cases: &[(Key, &[u8])] = &[
            (Key::Enter, b"\r"),
            (Key::Tab, b"\t"),
            (Key::BackTab, b"\x1b[Z"),
            (Key::Backspace, b"\x7f"),
            (Key::Delete, b"\x1b[3~"),
            (Key::Up, b"\x1b[A"),
            (Key::Down, b"\x1b[B"),
            (Key::Right, b"\x1b[C"),
            (Key::Left, b"\x1b[D"),
            (Key::Home, b"\x1b[H"),
            (Key::End, b"\x1b[F"),
            (Key::PageUp, b"\x1b[5~"),
            (Key::PageDown, b"\x1b[6~"),
            (Key::Escape, b"\x1b"),
            (Key::Char('a'), b"a"),
            (Key::Char('λ'), "λ".as_bytes()),
            (Key::Control(b'c'), b"\x03"),
            (Key::Alt('M'), b"\x1bM"),
            (Key::Alt('S'), b"\x1bS"),
            (Key::Paste(b"one\ntwo".as_slice().into()), b"\x1b[200~one\ntwo\x1b[201~"),
        ];
        for (key, expected) in cases {
            let bytes = key_bytes(key);
            assert_eq!(bytes, *expected, "{key:?}");
            let mut decoder = Decoder::new(100);
            let mut keys = Queue::with_capacity(2);
            decoder.read(&bytes, &mut keys);
            assert_eq!(keys.pop().as_ref(), Some(key), "decoder sees {key:?}");
            assert!(keys.is_empty());
        }
    }

    #[test]
    fn a_save_is_a_replacement_even_when_its_bytes_are_unchanged() {
        let scratch = Scratch::new("tui-again-save");
        let path = scratch.path().join("recording-20261115T090032Z");
        assert!(replaced(&path, None).expect("no first save").is_none());
        fs::write(&path, b"same save").expect("first save");
        let earlier = File::open(&path).expect("hold earlier inode");
        assert!(replaced(&path, Some(&earlier)).expect("unchanged file").is_none());
        let next = scratch.path().join("next");
        fs::write(&next, b"same save").expect("next save");
        fs::rename(next, &path).expect("replace save");
        assert_eq!(replaced(&path, Some(&earlier)).expect("replacement"), Some(b"same save".to_vec()));
    }
}
