//! The domain-face machine's disk layout (temper/docs/design/testing/fakes.md, 3).

use skein_lib::Token;

/// Where the machine keeps a workspace.
#[must_use]
pub fn dir(workspace: Token) -> Vec<u8> {
    format!("ws/{}", workspace.raw()).into_bytes()
}

/// Where the machine keeps a repository in a workspace.
#[must_use]
pub fn path(workspace: Token, repository: &[u8]) -> Vec<u8> {
    [dir(workspace).as_slice(), b"/", repository].concat()
}

/// Where the machine keeps a shared bare repository.
#[must_use]
pub fn store_path(store: Token) -> Vec<u8> {
    format!("stores/{}", store.raw()).into_bytes()
}

/// A file's workspace and its path within that workspace.
#[must_use]
pub fn workspace_file(path: &[u8]) -> Option<(Token, &[u8])> {
    named_file(path.strip_prefix(b"ws/")?)
}

/// A file's store and its path within that store.
#[must_use]
pub fn store_file(path: &[u8]) -> Option<(Token, &[u8])> {
    named_file(path.strip_prefix(b"stores/")?)
}

/// The workspace named by a store's per-workspace reference.
#[must_use]
pub fn reference_workspace(path: &[u8]) -> Option<Token> {
    let reference = path.strip_prefix(b"refs/temper/")?;
    let name = reference.split(|byte| *byte == b'/').next()?;
    Some(Token::new(std::str::from_utf8(name).ok()?.parse().ok()?))
}

fn named_file(path: &[u8]) -> Option<(Token, &[u8])> {
    let slash = path.iter().position(|byte| *byte == b'/')?;
    let token = Token::new(std::str::from_utf8(path.get(..slash)?).ok()?.parse().ok()?);
    Some((token, path.get(slash + 1..)?))
}
