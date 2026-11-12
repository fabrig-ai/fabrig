use super::{Encoded, Refusal, items::Target};
use crate::forgejo::{
    decode::{
        Malformed, Step,
        walk::{Field, Kind, Move, Read, Value, Walk},
    },
    document::{ChangedFile, Compare, Document, FileStatus, Row},
    id::{self, ObjectFormat, ObjectId},
    limits::Limits,
    request::{Method, Request},
};
use alloc::boxed::Box;
use skein_lib::{Writer, usize_of};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum FileKind {
    Range,
    Messages,
    Message,
    MessageBody,
    Parents,
    Parent,
    Tree,
    Entries,
    TreeEntry,
    DirectoryEntries,
    DirectoryEntry,
    File,
    Files,
    ComparedPath,
    ComparedPaths,
    Compare,
}
const fn field(key: &'static [u8], read: Read, required: bool) -> Field {
    Field { key, read, required }
}
pub(crate) fn fields(kind: FileKind) -> &'static [Field] {
    match kind {
        FileKind::DirectoryEntry => const { &[field(b"name", Read::Text, true), field(b"type", Read::Text, true)] },
        FileKind::Range => {
            const {
                &[
                    field(b"total_commits", Read::Number, true),
                    field(b"commits", Read::Array(Kind::File(FileKind::Messages)), true),
                ]
            }
        }
        FileKind::Message => {
            const {
                &[
                    field(b"sha", Read::Id { empty: false }, true),
                    field(b"parents", Read::Array(Kind::File(FileKind::Parents)), true),
                    field(b"commit", Read::Object(Kind::File(FileKind::MessageBody)), true),
                ]
            }
        }
        FileKind::MessageBody => const { &[field(b"message", Read::Body, true)] },
        FileKind::Tree => {
            const {
                &[
                    field(b"tree", Read::Array(Kind::File(FileKind::Entries)), true),
                    field(b"truncated", Read::Boolean, true),
                ]
            }
        }
        FileKind::TreeEntry => {
            const {
                &[
                    field(b"path", Read::Text, true),
                    field(b"mode", Read::Text, true),
                    field(b"type", Read::Text, true),
                    field(b"sha", Read::Id { empty: false }, true),
                ]
            }
        }
        FileKind::File => {
            const {
                &[
                    field(b"filename", Read::Text, true),
                    field(b"previous_filename", Read::Text, false),
                    field(b"status", Read::Text, true),
                ]
            }
        }
        FileKind::Parent => const { &[field(b"sha", Read::Id { empty: false }, true)] },
        FileKind::ComparedPath => const { &[field(b"filename", Read::Text, true)] },
        FileKind::Compare => {
            const {
                &[
                    field(b"total_commits", Read::Number, true),
                    field(b"files", Read::Array(Kind::File(FileKind::ComparedPaths)), true),
                ]
            }
        }
        FileKind::Messages
        | FileKind::Parents
        | FileKind::DirectoryEntries
        | FileKind::Entries
        | FileKind::Files
        | FileKind::ComparedPaths => &[],
    }
}
pub(crate) fn element(kind: FileKind) -> Option<Kind> {
    match kind {
        FileKind::Messages => Some(Kind::File(FileKind::Message)),
        FileKind::Parents => Some(Kind::File(FileKind::Parent)),
        FileKind::DirectoryEntries => Some(Kind::File(FileKind::DirectoryEntry)),
        FileKind::Entries => Some(Kind::File(FileKind::TreeEntry)),
        FileKind::Files => Some(Kind::File(FileKind::File)),
        FileKind::ComparedPaths => Some(Kind::File(FileKind::ComparedPath)),
        FileKind::Range
        | FileKind::Message
        | FileKind::MessageBody
        | FileKind::Parent
        | FileKind::DirectoryEntry
        | FileKind::Tree
        | FileKind::TreeEntry
        | FileKind::File
        | FileKind::ComparedPath
        | FileKind::Compare => None,
    }
}
pub(super) fn encode(request: &Request, limits: &Limits) -> Result<Encoded, Refusal> {
    let mut measure = Target { length: 0, writer: None };
    target(request, &mut measure);
    if measure.length > usize_of(limits.target_bytes) {
        return Err(Refusal::TooLong);
    }
    let mut write = Target { length: 0, writer: Some(Writer::new(measure.length)) };
    target(request, &mut write);
    Ok(Encoded { method: Method::Get, target: write.writer.expect("writing pass").finish(), body: None })
}
#[expect(clippy::too_many_lines, reason = "exhaustive request selection lists every dialect request")]
fn target(request: &Request, out: &mut Target) {
    match request {
        Request::Artifacts { at, run, name } => {
            out.numbered(at, b"/actions/runs/", *run);
            out.put(b"/artifacts");
            if let Some(name) = name {
                out.put(b"?name=");
                out.segment(name);
            }
        }
        Request::ArtifactZip { at, id } => {
            out.numbered(at, b"/actions/artifacts/", *id);
            out.put(b"/zip");
        }
        Request::Commits { at, before, after } => {
            out.repo(at);
            out.put(b"/compare/");
            out.put(&id::write(before));
            out.put(b"...");
            out.put(&id::write(after));
            out.put(b"?files=false&verification=false");
        }
        Request::Tree { at, object, page } => {
            out.repo(at);
            out.put(b"/git/trees/");
            out.put(&id::write(object));
            if let Some(n) = page.page {
                out.put(b"?page=");
                out.number(u64::from(n));
            }
            if let Some(n) = page.limit {
                out.put(b"&per_page=");
                out.number(u64::from(n));
            }
        }
        Request::WorkflowRuns { at, workflow, head_sha, page } => {
            out.repo(at);
            out.put(b"/actions/runs?workflow_id=");
            out.segment(workflow);
            out.put(b"&head_sha=");
            out.put(&id::write(head_sha));
            out.page(*page, true);
        }
        Request::PullFiles { at, number, page } => {
            out.numbered(at, b"/pulls/", *number);
            out.put(b"/files");
            out.page(*page, false);
        }
        Request::Compare { at, before, after, files } => {
            out.repo(at);
            out.put(b"/compare/");
            out.put(&id::write(before));
            out.put(b"...");
            out.put(&id::write(after));
            out.put(if *files { b"?verification=false" } else { b"?files=false&verification=false" });
        }
        Request::Contents { at, path, reference } => {
            out.repo(at);
            out.put(b"/contents");
            if !path.is_empty() {
                out.put(b"/");
                out.file(path);
            }
            out.put(b"?ref=");
            out.put(&id::write(reference));
        }
        Request::Raw { at, path, reference } => {
            out.repo(at);
            out.put(b"/raw/");
            out.file(path);
            out.put(b"?ref=");
            out.put(&id::write(reference));
        }
        Request::Runs { at, head_sha, run_number, page } => {
            out.repo(at);
            out.put(b"/actions/runs?head_sha=");
            out.put(&id::write(head_sha));
            out.put(b"&run_number=");
            out.number(*run_number);
            out.page(*page, true);
        }
        Request::Run { at, run } => out.numbered(at, b"/actions/runs/", *run),
        Request::Jobs { at, run } => {
            out.numbered(at, b"/actions/runs/", *run);
            out.put(b"/jobs");
        }
        Request::Job { at, job } => out.numbered(at, b"/actions/jobs/", *job),
        Request::JobLog { at, job, attempt, step } => {
            out.numbered(at, b"/actions/jobs/", *job);
            out.put(b"/logs?attempt=");
            out.number(u64::from(*attempt));
            if let Some(step) = step {
                out.put(b"&step=");
                out.number(u64::from(*step));
            }
        }
        Request::User
        | Request::ApiSettings
        | Request::RepositoryById { .. }
        | Request::Repository { .. }
        | Request::UserRepos { .. }
        | Request::RepoSearch { .. }
        | Request::UserSearch { .. }
        | Request::Issues { .. }
        | Request::Issue { .. }
        | Request::Comments { .. }
        | Request::Pull { .. }
        | Request::PullFor { .. }
        | Request::CombinedStatus { .. }
        | Request::Reviews { .. }
        | Request::ReviewComments { .. }
        | Request::Branch { .. }
        | Request::Branches { .. }
        | Request::Protection { .. }
        | Request::Protections { .. }
        | Request::Collaborators { .. }
        | Request::Permission { .. }
        | Request::Pulls { .. }
        | Request::CreateIssue { .. }
        | Request::OpenPull { .. }
        | Request::PostComment { .. }
        | Request::PostReview { .. }
        | Request::EditIssue { .. }
        | Request::RequestReviewers { .. }
        | Request::RemoveReviewers { .. }
        | Request::Dispatch { .. }
        | Request::Rerun { .. }
        | Request::Merge { .. }
        | Request::Update { .. }
        | Request::PostStatus { .. }
        | Request::CreateBranch { .. }
        | Request::Tokens { .. }
        | Request::CreateToken { .. }
        | Request::DeleteToken { .. }
        | Request::UserOf { .. }
        | Request::Org { .. }
        | Request::CreateOrg { .. }
        | Request::CreateFile { .. }
        | Request::Migrate { .. }
        | Request::DeleteRepository { .. }
        | Request::CreateRepository { .. }
        | Request::Teams { .. }
        | Request::CreateTeam { .. }
        | Request::EditTeam { .. }
        | Request::TeamMembers { .. }
        | Request::OrgHooks { .. }
        | Request::CreateOrgHook { .. }
        | Request::EditOrgHook { .. }
        | Request::DeleteOrgHook { .. }
        | Request::AddMember { .. }
        | Request::AddCollaborator { .. }
        | Request::RepoHooks { .. }
        | Request::CreateRepoHook { .. }
        | Request::EditRepoHook { .. }
        | Request::DeleteRepoHook { .. }
        | Request::CreateUser { .. }
        | Request::DeleteBranch { .. } => unreachable!("file encoder selection"),
    }
}
pub(crate) fn status(text: &[u8]) -> Option<FileStatus> {
    match text {
        b"added" => Some(FileStatus::Added),
        b"deleted" | b"removed" => Some(FileStatus::Deleted),
        b"modified" | b"changed" | b"unchanged" => Some(FileStatus::Modified),
        b"renamed" => Some(FileStatus::Renamed),
        b"copied" => Some(FileStatus::Copied),
        _ => None,
    }
}
fn file() -> ChangedFile {
    ChangedFile { filename: Box::from([]), previous_filename: None, status: FileStatus::Added }
}
#[derive(Debug)]
pub(super) struct Files {
    root: FileKind,
    name: Box<[u8]>,
    kind: Option<crate::api::EntryKind>,
    file: ChangedFile,
    commit: ObjectId,
    compare: Compare,
    entry: crate::forgejo::document::TreeEntry,
    truncated: bool,
    parents: skein_lib::List<ObjectId>,
    message: Box<[u8]>,
    parent: Option<ObjectId>,
}
impl Files {
    pub(super) fn new(root: FileKind, rows: u32) -> Self {
        Self {
            root,
            name: Box::from([]),
            kind: None,
            entry: entry(),
            truncated: false,
            parents: skein_lib::List::with_capacity(if root == FileKind::Range { rows } else { 0 }),
            message: Box::new([]),
            parent: None,
            file: file(),
            commit: ObjectId { format: ObjectFormat::Sha1, bytes: [0; 32] },
            compare: Compare { total_commits: 0 },
        }
    }
    pub(super) fn step(&mut self, movement: Move, walk: &Walk) -> Step {
        match movement {
            Move::Demand(demand) => Step::Demand(demand),
            Move::Scalar { kind, key, value } => match self.scalar(kind, key, value) {
                Ok(()) => Step::Demand(skein_json::tokenizer::Request::Next),
                Err(error) => Step::Malformed(error),
            },
            Move::End(Kind::File(FileKind::Message)) => {
                let parents = self.parents.as_slice().into();
                self.parents.clear();
                Step::Row(
                    Row::CommitMessage(crate::forgejo::document::CommitMessage {
                        id: self.commit,
                        parents,
                        message: core::mem::replace(&mut self.message, Box::new([])),
                    }),
                    skein_json::tokenizer::Request::Next,
                )
            }
            Move::End(Kind::File(FileKind::Parent)) => {
                let Some(parent) = self.parent.take() else {
                    return Step::Malformed(Malformed::Value { field: b"sha" });
                };
                if self.parents.push(parent).is_err() {
                    return Step::Malformed(Malformed::Bound { field: b"parents" });
                }
                Step::Demand(skein_json::tokenizer::Request::Next)
            }
            Move::End(Kind::File(FileKind::DirectoryEntry)) => Step::Row(
                Row::Entry { name: core::mem::replace(&mut self.name, Box::from([])), kind: self.kind.take() },
                skein_json::tokenizer::Request::Next,
            ),
            Move::End(Kind::File(FileKind::TreeEntry)) => Step::Row(
                Row::TreeEntry(core::mem::replace(&mut self.entry, entry())),
                skein_json::tokenizer::Request::Next,
            ),
            Move::End(Kind::File(FileKind::File | FileKind::ComparedPath)) => {
                Step::Row(Row::File(core::mem::replace(&mut self.file, file())), skein_json::tokenizer::Request::Next)
            }
            Move::End(Kind::File(
                FileKind::Range
                | FileKind::Messages
                | FileKind::MessageBody
                | FileKind::Parents
                | FileKind::DirectoryEntries
                | FileKind::Tree
                | FileKind::Entries
                | FileKind::Files
                | FileKind::ComparedPaths
                | FileKind::Compare,
            )) => Step::Demand(skein_json::tokenizer::Request::Next),
            Move::End(
                Kind::User
                | Kind::Reachable(_)
                | Kind::Error
                | Kind::Write(_)
                | Kind::Settings
                | Kind::Repository
                | Kind::Search
                | Kind::Setup(_)
                | Kind::Users
                | Kind::Item(_)
                | Kind::Review(_)
                | Kind::Delivery(_)
                | Kind::Branch(_)
                | Kind::Action(_),
            ) => unreachable!("file tables"),
            Move::Done => Step::Done(match self.root {
                FileKind::Tree => Document::Tree { truncated: self.truncated },
                FileKind::Entries | FileKind::TreeEntry => unreachable!("tree root selection"),
                FileKind::Range | FileKind::Compare => Document::Compare(self.compare),
                FileKind::DirectoryEntries | FileKind::Files => Document::Listed { skipped: walk.skipped() },
                FileKind::Messages
                | FileKind::Message
                | FileKind::MessageBody
                | FileKind::Parents
                | FileKind::Parent
                | FileKind::DirectoryEntry
                | FileKind::File
                | FileKind::ComparedPath
                | FileKind::ComparedPaths => {
                    unreachable!("file root selection")
                }
            }),
        }
    }
    fn scalar(&mut self, kind: Kind, key: &'static [u8], value: Value) -> Result<(), Malformed> {
        match value {
            Value::Input(_) => unreachable!("dispatch input table"),
            Value::Number(n) => self.compare.total_commits = n,
            Value::Id(id) => {
                let id = id.expect("required object id");
                if kind == Kind::File(FileKind::Parent) {
                    self.parent = Some(id);
                } else if self.root == FileKind::Tree {
                    self.entry.sha = id;
                } else {
                    self.commit = id;
                }
            }
            Value::Text(text) => match key {
                b"name" => self.name = text,
                b"message" => self.message = text,
                b"path" => self.entry.path = text,
                b"mode" => {
                    let mut mode = 0u32;
                    for byte in &text {
                        if !(b'0'..=b'7').contains(byte) {
                            return Err(Malformed::Value { field: key });
                        }
                        mode = mode
                            .checked_mul(8)
                            .ok_or(Malformed::Range { field: key })?
                            .checked_add(u32::from(byte.saturating_sub(b'0')))
                            .ok_or(Malformed::Range { field: key })?;
                    }
                    self.entry.mode = mode;
                }
                b"type" if self.root == FileKind::DirectoryEntries => self.kind = crate::directory::kind(&text),
                b"type" => {
                    self.entry.kind = match text.as_ref() {
                        b"blob" => crate::api::TreeEntryKind::Blob,
                        b"tree" => crate::api::TreeEntryKind::Tree,
                        b"commit" => crate::api::TreeEntryKind::Commit,
                        _ => return Err(Malformed::Value { field: key }),
                    }
                }
                b"filename" => self.file.filename = text,
                b"previous_filename" => self.file.previous_filename = Some(text),
                b"status" => self.file.status = status(&text).ok_or(Malformed::Value { field: key })?,
                _ => unreachable!("file text table"),
            },
            Value::Boolean(value) => self.truncated = value,
            Value::Body(crate::forgejo::document::Body::Kept(bytes)) => self.message = bytes,
            Value::Body(crate::forgejo::document::Body::Long(_)) => {
                return Err(Malformed::TooLong { field: key });
            }
            Value::Time(_) => unreachable!("file scalar table"),
        }
        Ok(())
    }
}

fn entry() -> crate::forgejo::document::TreeEntry {
    crate::forgejo::document::TreeEntry {
        path: Box::new([]),
        mode: 0,
        kind: crate::api::TreeEntryKind::Blob,
        sha: ObjectId { format: ObjectFormat::Sha1, bytes: [0; 32] },
    }
}
