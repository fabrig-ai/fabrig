//! The reads of one immutable tree (temper/docs/design/domain/forge/temper-dir-contracts.md, 1.1).
use crate::{CHARTERS, DIRECTORIES, FILE_BYTES, FILES, HOOKS, ROOT, TOTAL_BYTES};
use alloc::boxed::Box;
use jig_domain_people::settings::is_name;
use skein_forge::api::{Answer, Commit, Entry, EntryKind, Error, Read};
use skein_lib::{List, Map, bytes, usize_of};
use temper_worker_domain_tooling::hooks;

/// A listed method file read at the commit; the path is repository-relative.
#[derive(Debug, PartialEq, Eq)]
pub struct MethodFile {
    pub path: Box<[u8]>,
    pub bytes: Box<[u8]>,
}

/// The result did not answer the pending read. The reading is unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Unread;

/// One caller-driven reading. The caller binds each result to its request.
#[derive(Hash, Debug)]
pub struct Reading {
    head: Commit,
    answer_bytes: u32,
    root_read: bool,
    pub(crate) root_cut: bool,
    pub(crate) entries: Map<Box<[u8]>, Kept>,
    asked: u32,
    bytes_read: u32,
}

#[derive(Hash, Debug)]
pub(crate) enum Kept {
    Directory { listed: bool, cut: bool },
    File { kind: Format, held: Held },
    Fault(Box<[u8]>),
}

#[derive(Hash, Debug)]
pub(crate) enum Held {
    Pending,
    Cut,
    Bytes(Box<[u8]>),
    OverBudget,
}

#[derive(Clone, Copy, Hash, Debug)]
pub(crate) enum Format {
    Offers,
    Way,
    Charter,
    Hooks,
}

impl Format {
    pub(crate) fn bound(self) -> u32 {
        match self {
            Self::Offers | Self::Way | Self::Charter => FILE_BYTES,
            Self::Hooks => u32::try_from(hooks::CONFIG_BYTES).expect("hook config limit fits a request"),
        }
    }
}

#[derive(Clone, Copy)]
enum Target {
    Directory,
    File(Format),
}

impl Target {
    fn bound(self) -> u32 {
        match self {
            Self::Directory => FILE_BYTES,
            Self::File(kind) => kind.bound(),
        }
    }
}

impl Reading {
    /// Start a reading at `head`, limiting each answer to the caller's budget.
    #[must_use]
    pub fn new(head: Commit, answer_bytes: u32) -> Self {
        // At most four listings, with at least one byte charged per entry by the forge.
        let entries = answer_bytes.min(FILE_BYTES).saturating_mul(4);
        Self {
            head,
            answer_bytes,
            root_read: false,
            root_cut: false,
            entries: Map::with_capacity(entries),
            asked: 0,
            bytes_read: 0,
        }
    }

    /// The next request, or nothing once the tree is read or its bounds are reached.
    #[must_use]
    pub fn next(&self) -> Option<Read> {
        let (path, target) = self.pending()?;
        Some(Read::File {
            head: self.head,
            path: bytes::copy_of(path),
            max_bytes: target.bound().min(self.answer_bytes),
        })
    }

    /// Take the forge result for the request returned by `next`. `Missing` is
    /// tied to that request by the caller, since it carries no head or path.
    /// # Errors
    /// No request is left, or the answer's head, path or kind differs, or the call failed other than `Missing`.
    pub fn take(&mut self, answer: &Result<Answer, Error>) -> Result<(), Unread> {
        let (path, target) = self.pending().ok_or(Unread)?;
        match answer {
            Ok(answer) => match answer {
                Answer::Directory { head, path: found, entries, truncated } => {
                    if *head != self.head || found.as_ref() != path {
                        return Err(Unread);
                    }
                    match target {
                        Target::Directory => self.keep_directory(found, entries, *truncated),
                        Target::File(_) => return Err(Unread),
                    }
                }
                Answer::File { head, path: found, bytes, truncated } => {
                    if *head != self.head || found.as_ref() != path {
                        return Err(Unread);
                    }
                    match target {
                        Target::File(kind) => self.keep_file(found, kind, bytes, *truncated),
                        Target::Directory => return Err(Unread),
                    }
                }
                Answer::Located { .. }
                | Answer::Items { .. }
                | Answer::Item { .. }
                | Answer::Pull(_)
                | Answer::Reviews { .. }
                | Answer::Statuses { .. }
                | Answer::Remarks { .. }
                | Answer::Commit(_)
                | Answer::Branches(_)
                | Answer::PullFiles { .. }
                | Answer::Compare { .. }
                | Answer::Checks(_)
                | Answer::Job { .. }
                | Answer::JobState { .. }
                | Answer::Protection(_)
                | Answer::Settings(_)
                | Answer::Collaborators { .. }
                | Answer::Permission(_)
                | Answer::Created(_)
                | Answer::Commented(_)
                | Answer::Reviewed(_)
                | Answer::Merged(_)
                | Answer::Queued
                | Answer::Branch(_)
                | Answer::Runs { .. }
                | Answer::Run(_)
                | Answer::Tree { .. }
                | Answer::Commits { .. }
                | Answer::Done => return Err(Unread),
            },
            Err(error) => match error {
                Error::Missing => {
                    let path = bytes::copy_of(path);
                    self.gone(&path, target);
                }
                Error::Unavailable
                | Error::Timeout
                | Error::RateLimited { .. }
                | Error::Forbidden
                | Error::MissingJob
                | Error::TooLarge(_)
                | Error::Empty
                | Error::Full
                | Error::Exists
                | Error::NothingToMerge
                | Error::Closed
                | Error::Stale
                | Error::BaseMoved { .. }
                | Error::Conflict
                | Error::Protected
                | Error::Refused
                | Error::InvalidAnswer
                | Error::Busy
                | Error::NotConnected
                | Error::NotInstalled => return Err(Unread),
            },
        }
        Ok(())
    }

    /// The complete method files read, in byte order of path. Cut files and
    /// the hook config are not part of the revision's method files.
    #[must_use]
    pub fn method_files(&self) -> Box<[MethodFile]> {
        let mut kept = List::with_capacity(FILES);
        for (path, entry) in &self.entries {
            match entry {
                Kept::File { kind: Format::Offers | Format::Way | Format::Charter, held: Held::Bytes(body) } => {
                    let file = MethodFile { path: path.clone(), bytes: body.clone() };
                    kept.push(file).expect("the reading takes at most FILES files");
                }
                Kept::Directory { .. }
                | Kept::Fault(_)
                | Kept::File { kind: Format::Hooks, held: Held::Bytes(_) }
                | Kept::File { held: Held::Pending | Held::Cut | Held::OverBudget, .. } => {}
            }
        }
        kept.into_boxed()
    }

    fn pending(&self) -> Option<(&[u8], Target)> {
        if !self.root_read {
            return Some((ROOT, Target::Directory));
        }
        for path in DIRECTORIES {
            match self.entries.get(path) {
                Some(Kept::Directory { listed: false, .. }) => return Some((path, Target::Directory)),
                Some(Kept::Directory { listed: true, .. } | Kept::File { .. } | Kept::Fault(_)) | None => {}
            }
        }
        if self.at_bound() {
            return None;
        }
        let (path, kind) = self.next_file()?;
        Some((path, Target::File(kind)))
    }

    fn next_file(&self) -> Option<(&[u8], Format)> {
        for (path, entry) in &self.entries {
            match entry {
                Kept::File { kind, held: Held::Pending } => return Some((path, *kind)),
                Kept::File { held: Held::Bytes(_) | Held::Cut | Held::OverBudget, .. }
                | Kept::Directory { .. }
                | Kept::Fault(_) => {}
            }
        }
        None
    }

    const fn at_bound(&self) -> bool {
        self.asked == FILES || self.bytes_read > TOTAL_BYTES
    }

    pub(crate) fn exceeded(&self) -> bool {
        self.bytes_read > TOTAL_BYTES || (self.asked == FILES && self.next_file().is_some())
    }

    fn gone(&mut self, path: &[u8], target: Target) {
        match target {
            Target::Directory => {
                if path == ROOT {
                    self.root_read = true;
                    return;
                }
            }
            Target::File(_) => self.asked = self.asked.saturating_add(1),
        }
        self.entries.remove(path).expect("only a listed path can be pending below the root");
    }

    fn keep_directory(&mut self, path: &[u8], entries: &[Entry], truncated: bool) {
        let mut room = usize_of(self.answer_bytes.min(FILE_BYTES));
        let mut cut = truncated;
        for entry in entries {
            let charge = entry.name.len().saturating_add(1);
            let Some(left) = room.checked_sub(charge) else {
                cut = true;
                break;
            };
            room = left;
            let Some(kept) = classify(path, entry) else { continue };
            let path = bytes::concat(&[path, b"/", &entry.name]);
            let _previous = self.entries.insert(path, kept).expect("listings fit the four forge answers");
        }
        if path == ROOT {
            self.root_read = true;
            self.root_cut = cut;
        } else {
            let slot = self.entries.get_mut(path).expect("the directory was listed");
            *slot = Kept::Directory { listed: true, cut };
        }
    }

    fn keep_file(&mut self, path: &[u8], kind: Format, body: &[u8], cut: bool) {
        self.asked = self.asked.saturating_add(1);
        let length = u32::try_from(body.len()).unwrap_or(u32::MAX);
        self.bytes_read = self.bytes_read.saturating_add(length);
        let held = if cut || length > kind.bound().min(self.answer_bytes) {
            Held::Cut
        } else if self.bytes_read > TOTAL_BYTES {
            Held::OverBudget
        } else {
            Held::Bytes(bytes::copy_of(body))
        };
        let slot = self.entries.get_mut(path).expect("the file was listed");
        *slot = Kept::File { kind, held };
    }
}

fn classify(parent: &[u8], entry: &Entry) -> Option<Kept> {
    if parent == ROOT {
        return match entry.name.as_ref() {
            b"charters" | b"hooks" | b"ways" => Some(if entry.kind == EntryKind::Directory {
                Kept::Directory { listed: false, cut: false }
            } else {
                wrong_kind(entry.kind, b"temper reads a directory here")
            }),
            b"ways.toml" => Some(regular(entry.kind, Format::Offers)),
            _ => None,
        };
    }
    if parent == HOOKS {
        if entry.name.as_ref() == b"config.toml" {
            return Some(regular(entry.kind, Format::Hooks));
        }
        return None;
    }
    let (suffix, kind, shape) = if parent == CHARTERS {
        (b".md".as_slice(), Format::Charter, b"temper reads a NAME.md regular file here".as_slice())
    } else {
        (b".toml".as_slice(), Format::Way, b"temper reads a NAME.toml regular file here".as_slice())
    };
    if entry.kind != EntryKind::File {
        return Some(wrong_kind(entry.kind, b"temper reads a regular file here"));
    }
    let Some(stem) = entry.name.strip_suffix(suffix) else {
        return Some(wrong_kind(entry.kind, shape));
    };
    if !is_name(stem) {
        return Some(Kept::Fault(crate::judge::name_words(stem)));
    }
    Some(regular(entry.kind, kind))
}

fn regular(kind: EntryKind, format: Format) -> Kept {
    if kind == EntryKind::File {
        Kept::File { kind: format, held: Held::Pending }
    } else {
        wrong_kind(kind, b"temper reads a regular file here")
    }
}

fn wrong_kind(kind: EntryKind, expected: &[u8]) -> Kept {
    let found: &[u8] = match kind {
        EntryKind::File => b"a regular file",
        EntryKind::Directory => b"a directory",
        EntryKind::Link => b"a symbolic link",
        EntryKind::Submodule => b"a submodule",
    };
    Kept::Fault(bytes::concat(&[found, b"; ", expected]))
}
