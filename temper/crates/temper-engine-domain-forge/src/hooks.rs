//! A repository's commands at an immutable head (temper/docs/design/protocol/hooks.md, 4.2 and 5).
use alloc::boxed::Box;
use skein_forge::api::{Answer, Commit, Error, Read};
use skein_lib::bytes;

pub(crate) const DIRECTORY: &[u8] = b".temper/hooks";
pub const CLOSE_HOOK: &[u8] = b".temper/hooks/close";
const MERGE_HOOK: &[u8] = b".temper/hooks/merge";

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Hooks {
    Legacy,
    Directory { merge: bool, close: bool },
}

impl Hooks {
    pub(crate) fn merge(self, legacy: Option<&[u8]>) -> Option<&[u8]> {
        match self {
            Self::Legacy => legacy,
            Self::Directory { merge: true, .. } => Some(MERGE_HOOK),
            Self::Directory { merge: false, .. } => None,
        }
    }

    pub(crate) fn missing_merge(self, head: Commit) -> Box<[u8]> {
        let why = match self {
            Self::Legacy => b"no `.temper/hooks/`, and the project names no `merge_job`".as_slice(),
            Self::Directory { .. } => b"`.temper/hooks/` names no `merge`".as_slice(),
        };
        bytes::concat(&[b"change's merge: no merge command at ", &crate::commit_text(&head), b": ", why])
    }
}

pub(crate) fn read(head: Commit, max_bytes: u32) -> Read {
    Read::File { head, path: Box::from(DIRECTORY), max_bytes }
}

pub(crate) fn answered(at: Commit, result: Result<Answer, Error>) -> Result<Hooks, Error> {
    if let Err(Error::Missing) = result {
        return Ok(Hooks::Legacy);
    }
    let answer = result?;
    if let Answer::Directory { head, path, entries, truncated } = answer {
        if head != at || path.as_ref() != DIRECTORY {
            return Err(Error::InvalidAnswer);
        }
        let mut merge = truncated;
        let mut close = truncated;
        for entry in &entries {
            merge |= entry.name.as_ref() == b"merge";
            close |= entry.name.as_ref() == b"close";
        }
        return Ok(Hooks::Directory { merge, close });
    }
    if let Answer::File { head, path, .. } = answer
        && head == at
        && path.as_ref() == DIRECTORY
    {
        return Ok(Hooks::Legacy);
    }
    Err(Error::InvalidAnswer)
}
