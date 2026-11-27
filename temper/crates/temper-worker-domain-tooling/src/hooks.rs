//! Job hook paths and failure lines (temper/docs/design/domain/worker/jobs.md, 2).
use alloc::boxed::Box;
use skein_lib::{Writer, bytes};

/// Splits a hook command into its repository name and its path in that repository.
#[must_use]
pub fn hook_path(command: &[u8]) -> Option<(&[u8], &[u8])> {
    let slash = bytes::find(command, b"/")?;
    let (repository, path) = command.split_at_checked(slash)?;
    let path = path.strip_prefix(b"/")?;
    if repository.is_empty() || !path.starts_with(b".temper/hooks/") { None } else { Some((repository, path)) }
}

/// The one-line tail of a command that could not run its hook.
#[must_use]
pub fn hook_failure(path: &[u8], why: &[u8]) -> Box<[u8]> {
    bytes::concat(&[b"hook `", &one_line(path), b"`: ", why])
}

/// Why a link that does not name a sibling was refused.
#[must_use]
pub fn hook_link_target(target: &[u8]) -> Box<[u8]> {
    bytes::concat(&[b"a link to `", &one_line(target), b"`, not to an executable file beside it"])
}

fn one_line(input: &[u8]) -> Box<[u8]> {
    let mut len = 0_usize;
    for byte in input {
        len = len.checked_add(byte.escape_ascii().len()).expect("bounded hook path");
    }
    let mut text = Writer::new(len);
    for byte in input {
        for ascii in byte.escape_ascii() {
            text.put(core::slice::from_ref(&ascii)).expect("measured diagnostic");
        }
    }
    text.finish()
}
