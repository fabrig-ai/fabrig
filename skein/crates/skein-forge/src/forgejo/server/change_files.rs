//! The existing contents commit API, decoded as one bounded batch (skein/docs/design/testing/fakes/forge.md, 4).
use super::writes::WriteKind;
use crate::forgejo::{
    decode::{
        Malformed,
        walk::{Field, Kind, Read},
    },
    limits::Limits,
    request::{FileChange, Request},
};
use alloc::boxed::Box;
use skein_lib::{
    List,
    base64::{self, Alphabet},
};

pub(super) fn fields(kind: WriteKind) -> &'static [Field] {
    match kind {
        WriteKind::ChangeFiles => {
            const {
                &[
                    Field { key: b"branch", read: Read::Text, required: true },
                    Field { key: b"new_branch", read: Read::Text, required: false },
                    Field { key: b"message", read: Read::Body, required: true },
                    Field { key: b"files", read: Read::Array(Kind::Write(WriteKind::Changes)), required: true },
                ]
            }
        }
        WriteKind::Change => {
            const {
                &[
                    Field { key: b"operation", read: Read::Text, required: true },
                    Field { key: b"path", read: Read::Text, required: true },
                    Field { key: b"content", read: Read::Body, required: true },
                    Field { key: b"sha", read: Read::Text, required: false },
                ]
            }
        }
        WriteKind::Dispatch
        | WriteKind::Inputs
        | WriteKind::Issue
        | WriteKind::NewUser
        | WriteKind::NewToken
        | WriteKind::Scopes
        | WriteKind::Scope
        | WriteKind::NewHook
        | WriteKind::Collaborator
        | WriteKind::HookConfig
        | WriteKind::NewOrg
        | WriteKind::NewRepository
        | WriteKind::Migration
        | WriteKind::NewFile
        | WriteKind::Changes
        | WriteKind::Author
        | WriteKind::Committer
        | WriteKind::NewTeam
        | WriteKind::Pull
        | WriteKind::Comment
        | WriteKind::Review
        | WriteKind::Edit
        | WriteKind::Reviewers
        | WriteKind::Logins
        | WriteKind::Login
        | WriteKind::Merge
        | WriteKind::Status
        | WriteKind::Branch => &[],
    }
}
#[derive(Debug)]
pub(super) struct Changes {
    files: List<FileChange>,
    file: FileChange,
    update: bool,
}
fn file() -> FileChange {
    FileChange { path: Box::new([]), content: Box::new([]), sha: None }
}
impl Changes {
    pub(super) fn new(limits: &Limits) -> Self {
        Self { files: List::with_capacity(limits.inner), file: file(), update: false }
    }
    pub(super) fn text(
        &mut self,
        kind: Kind,
        key: &'static [u8],
        text: Box<[u8]>,
        request: &mut Request,
        limits: &Limits,
    ) -> Result<(), Malformed> {
        if kind == Kind::Write(WriteKind::ChangeFiles) {
            let Request::ChangeFiles { branch, new_branch, message, .. } = request else {
                unreachable!("contents body")
            };
            if key == b"branch" {
                *branch = text;
            } else if key == b"new_branch" {
                *new_branch = Some(text);
            } else {
                *message = text;
            }
            return Ok(());
        }
        match key {
            b"path" => self.file.path = text,
            b"sha" => self.file.sha = Some(text),
            b"content" => {
                self.file.content = base64::decode(Alphabet::Standard, &text, limits.body_bytes)
                    .or(Err(Malformed::Value { field: key }))?;
            }
            b"operation" => {
                self.update = match text.as_ref() {
                    b"create" => false,
                    b"update" => true,
                    _ => return Err(Malformed::Value { field: key }),
                };
            }
            _ => unreachable!("contents fields"),
        }
        Ok(())
    }
    pub(super) fn end(&mut self, kind: Kind, request: &mut Request) -> Result<(), Malformed> {
        if kind == Kind::Write(WriteKind::Change) {
            if self.update != self.file.sha.is_some() || self.file.path.is_empty() {
                return Err(Malformed::Value { field: b"sha" });
            }
            for held in &self.files {
                if held.path == self.file.path {
                    return Err(Malformed::Value { field: b"path" });
                }
            }
            let file = core::mem::replace(&mut self.file, file());
            self.files.push(file).or(Err(Malformed::Bound { field: b"files" }))?;
            self.update = false;
        }
        if kind == Kind::Write(WriteKind::Changes) {
            if self.files.is_empty() {
                return Err(Malformed::Value { field: b"files" });
            }
            let Request::ChangeFiles { files, .. } = request else { unreachable!("contents body") };
            *files = core::mem::replace(&mut self.files, List::with_capacity(0)).into_boxed();
        }
        Ok(())
    }
}
