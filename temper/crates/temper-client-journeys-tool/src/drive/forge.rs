//! Forge line policy; credentials never cross this boundary (client/drive.md, 3.5).
use super::{Action, Event, Line, Machine, Phase, Record, STEP};
use crate::line::Forge;
use skein_forge::forgejo::{
    client,
    request::{FileChange, Repo, Request},
};
use skein_http::Method;
use skein_json::{Document, Kind, document, tokenizer, writer};
use skein_lib::{Duration, Time};
use std::{os::unix::ffi::OsStrExt, path::PathBuf};

const CHECKS: Duration = Duration::from_secs(300);
const POLL: Duration = Duration::from_secs(2);
pub(super) const RESPONSE_BYTES: u32 = 4 * 1024 * 1024;
const JSON: tokenizer::Limits =
    tokenizer::Limits { depth: 32, string: RESPONSE_BYTES, number: 32, chunk: 4096, length: RESPONSE_BYTES };
#[derive(PartialEq, Eq)]
enum Stage {
    Log,
    Files,
    Repository,
    File,
    Commit,
    Checks,
    Poll,
}
pub(super) struct Pending {
    token_file: PathBuf,
    directory: Option<PathBuf>,
    api: Box<[u8]>,
    repository: Box<[u8]>,
    message: Option<Box<[u8]>>,
    files: Vec<FileChange>,
    index: usize,
    branch: Box<[u8]>,
    stage: Stage,
    method: Method,
    path: Box<[u8]>,
    last: String,
}
impl Machine {
    pub(super) fn forge_start(&mut self, now: Time, out: &mut Vec<Action>) {
        let Some(Record::Forge(record)) = self.current.as_ref().map(Line::record) else { unreachable!("forge line") };
        let (repository, message, files) = match record {
            Forge::Commit { repository, message, paths } => (
                repository.clone(),
                Some(message.clone()),
                paths.iter().map(|path| FileChange { path: path.clone(), content: Box::new([]), sha: None }).collect(),
            ),
            Forge::Checks { repository } => (repository.clone(), None, vec![]),
        };
        let log = self.plan.options.scratch.join("session/box.log");
        self.wait(Phase::Forge, now, if message.is_some() { STEP } else { CHECKS });
        self.forge = Some(Pending {
            token_file: PathBuf::new(),
            directory: self.plan.options.files.clone(),
            api: Box::new([]),
            repository,
            message,
            files,
            index: 0,
            branch: Box::new([]),
            stage: Stage::Log,
            method: Method::Get,
            path: Box::new([]),
            last: "no status read".into(),
        });
        out.push(Action::ReadBoxLog { path: log });
    }
    pub(super) fn forge_event(&mut self, event: Event, now: Time, out: &mut Vec<Action>) {
        let Some(mut pending) = self.forge.take() else { return };
        let result = match event {
            Event::BoxLog(result) if pending.stage == Stage::Log => {
                result.and_then(|bytes| pending.log(&bytes)).map(|()| None)
            }
            Event::ForgeFiles(result) if pending.stage == Stage::Files => result.map(|contents| {
                for (file, content) in pending.files.iter_mut().zip(contents) {
                    file.content = content;
                }
                pending.stage = Stage::Repository;
                None
            }),
            Event::ForgeResponse { status, body } => pending.response(status, &body),
            Event::ForgeFailure(why) => Err(why),
            _ => {
                self.forge = Some(pending);
                return;
            }
        };
        match result {
            Err(why) => self.answer(Some(why), out),
            Ok(Some(shown)) => {
                self.shown.extend(shown);
                self.answer(None, out);
            }
            Ok(None) => {
                if pending.stage == Stage::Poll {
                    self.poll = Some(now.saturating_add(POLL));
                } else {
                    match pending.request(self.until.expect("forge bound")) {
                        Ok(action) => out.push(action),
                        Err(why) => {
                            self.answer(Some(why), out);
                            return;
                        }
                    }
                }
                self.forge = Some(pending);
            }
        }
    }
    pub(super) fn forge_deadline(&mut self, now: Time, out: &mut Vec<Action>) -> bool {
        if self.forge.is_none() {
            return false;
        }
        if self.until.is_some_and(|until| now >= until) {
            let pending = self.forge.take().expect("forge line");
            out.push(Action::CancelForge);
            self.answer(Some(format!("forge deadline passed; {}", pending.last)), out);
        } else if self.poll.is_some_and(|poll| now >= poll) {
            self.poll = None;
            let pending = self.forge.as_mut().expect("forge poll");
            pending.stage = Stage::Checks;
            match pending.request(self.until.expect("forge bound")) {
                Ok(action) => out.push(action),
                Err(why) => self.answer(Some(why), out),
            }
        }
        true
    }
}
impl Pending {
    fn log(&mut self, bytes: &[u8]) -> Result<(), String> {
        let git = super::values::read(bytes, crate::line::BoxValue::Git)?;
        self.api = [git.as_ref(), b"/api/v1"].concat().into();
        self.token_file =
            PathBuf::from(std::ffi::OsStr::from_bytes(&super::values::read(bytes, crate::line::BoxValue::Admin)?));
        self.stage = if self.message.is_some() { Stage::Files } else { Stage::Repository };
        Ok(())
    }
    fn repo_path(&self) -> String {
        format!("/repos/{}", encoded(&self.repository, true))
    }
    fn request(&mut self, until: Time) -> Result<Action, String> {
        if self.stage == Stage::Files {
            let directory = self.directory.clone().ok_or("forge commit: no files directory (--files DIR)")?;
            return Ok(Action::ReadForgeFiles {
                directory,
                paths: self.files.iter().map(|file| file.path.clone()).collect(),
            });
        }
        let repo = self.repo_path();
        let mut body: Box<[u8]> = Box::new([]);
        let path = match self.stage {
            Stage::Repository => repo,
            Stage::File => format!(
                "{repo}/contents/{}?ref={}",
                encoded(&self.files[self.index].path, true),
                encoded(&self.branch, false)
            ),
            Stage::Checks => format!("{repo}/commits/{}/status", encoded(&self.branch, false)),
            Stage::Commit => {
                let mut parts = self.repository.split(|byte| *byte == b'/');
                let owner = parts.next().expect("OWNER");
                let name = parts.next().expect("NAME");
                let request = Request::ChangeFiles {
                    at: Repo { owner: owner.into(), name: name.into() },
                    branch: self.branch.clone(),
                    message: self.message.clone().expect("commit message"),
                    files: self.files.clone().into(),
                };
                let limits = skein_forge::forgejo::limits::Limits {
                    json: JSON,
                    writer: writer::Limits { depth: 8, length: u32::MAX },
                    string_bytes: RESPONSE_BYTES,
                    body_bytes: RESPONSE_BYTES,
                    inner: 16,
                    target_bytes: u32::MAX,
                };
                body = client::encode(&request, &limits)
                    .map_err(|why| format!("forge commit JSON: {why:?}"))?
                    .body
                    .expect("contents body");
                format!("{repo}/contents")
            }
            Stage::Log | Stage::Files | Stage::Poll => unreachable!("only ready requests"),
        };
        self.method = if self.stage == Stage::Commit { Method::Post } else { Method::Get };
        self.path = path.into_bytes().into();
        Ok(Action::ForgeRequest {
            api: self.api.clone(),
            token_file: self.token_file.clone(),
            method: self.method,
            path: self.path.clone(),
            body,
            until,
        })
    }
    fn response(&mut self, status: u16, body: &[u8]) -> Result<Option<Vec<String>>, String> {
        if !((200..300).contains(&status) || self.stage == Stage::File && status == 404) {
            return Err(format!(
                "{} {}: HTTP {status}: {}",
                if self.method == Method::Post { "POST" } else { "GET" },
                String::from_utf8_lossy(&self.path),
                String::from_utf8_lossy(body.get(..4096).unwrap_or(body))
            ));
        }
        if self.stage == Stage::File && status == 404 {
            self.next_file();
            return Ok(None);
        }
        let doc = skein_json::parse(body, &JSON, &document::Limits { tokens: RESPONSE_BYTES, text: RESPONSE_BYTES })
            .map_err(|why| format!("forge response JSON: {why:?}"))?;
        match self.stage {
            Stage::Repository => {
                self.branch = text(&doc, 0, b"default_branch")?.into();
                if self.branch.is_empty() {
                    return Err("repository lacks default_branch".into());
                }
                self.stage = if self.message.is_some() { Stage::File } else { Stage::Checks };
            }
            Stage::File => {
                self.files[self.index].sha = Some(text(&doc, 0, b"sha")?.into());
                self.next_file();
            }
            Stage::Commit => {
                let commit = doc.member(0, b"commit").ok_or("forge response lacks commit")?;
                return Ok(Some(vec![String::from_utf8_lossy(text(&doc, commit, b"sha")?).into_owned()]));
            }
            Stage::Checks => return self.checks(&doc),
            Stage::Log | Stage::Files | Stage::Poll => return Err("unexpected forge response".into()),
        }
        Ok(None)
    }
    fn next_file(&mut self) {
        self.index += 1;
        self.stage = if self.index == self.files.len() { Stage::Commit } else { Stage::File };
    }
    fn checks(&mut self, doc: &Document) -> Result<Option<Vec<String>>, String> {
        let state = text(doc, 0, b"state")?;
        let mut shown = vec![String::from_utf8_lossy(text(doc, 0, b"sha")?).into_owned()];
        let statuses = doc.member(0, b"statuses").ok_or("forge response lacks statuses")?;
        if doc.token(statuses).is_none_or(|token| token.kind != Kind::ArrayStart) {
            return Err("expected statuses array".into());
        }
        let mut cursor = statuses + 1;
        while doc.token(cursor).is_some_and(|token| token.kind != Kind::ArrayEnd) {
            shown.push(format!(
                "{} {}",
                String::from_utf8_lossy(text(doc, cursor, b"context")?),
                String::from_utf8_lossy(text(doc, cursor, b"status")?)
            ));
            cursor = doc.value_end(cursor).ok_or("expected status object")?;
        }
        self.last = format!(
            "state {}; {}{}",
            String::from_utf8_lossy(state),
            shown.join("; "),
            if shown.len() == 1 { "; no statuses" } else { "" }
        );
        match state {
            b"success" => Ok(Some(shown)),
            b"failure" | b"error" => Err(format!("forge checks: {}", self.last)),
            _ => {
                self.stage = Stage::Poll;
                Ok(None)
            }
        }
    }
}
fn text<'a>(doc: &'a Document, object: u32, key: &[u8]) -> Result<&'a [u8], String> {
    doc.member(object, key)
        .and_then(|index| doc.token(index))
        .filter(|token| token.kind == Kind::String)
        .and_then(|token| doc.text(token))
        .ok_or_else(|| format!("forge response lacks {} string", String::from_utf8_lossy(key)))
}
fn encoded(bytes: &[u8], slash: bool) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    for byte in bytes {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(byte) || slash && *byte == b'/' {
            out.push(char::from(*byte));
        } else {
            write!(out, "%{byte:02X}").expect("String");
        }
    }
    out
}
