//! Ordering and wording the tree's faults (temper/docs/design/protocol/temper-dir.md, 8).
use crate::reading::{Format, Held, Kept, Reading};
use crate::{FAULTS, Fault, Home, METHOD, READER, ROOT, Verdict, ways};
use alloc::boxed::Box;
use skein_lib::{List, bytes};
use temper_charter_set::FormatFault;
use temper_worker_domain_tooling::hooks;

impl Reading {
    /// Judge the completed reading, including a reading stopped at a bound.
    #[must_use]
    pub fn judge(&self, home: Home) -> Verdict {
        let mut report = Report { faults: List::with_capacity(FAULTS), more: 0 };
        let mut method = false;
        if self.root_cut {
            report.add(ROOT, 0, cut_words());
        }
        for (path, entry) in &self.entries {
            match entry {
                Kept::Fault(why) => report.add(path, 0, why.clone()),
                Kept::Directory { cut: true, .. } => report.add(path, 0, cut_words()),
                Kept::Directory { cut: false, .. } | Kept::File { .. } => {}
            }
        }
        if self.exceeded() {
            report.add(b".temper/", 0, bytes::copy_of(b"past what temper reads: 64 files"));
        }
        for (path, entry) in &self.entries {
            match entry {
                Kept::File { held: Held::Cut, .. } => report.add(path, 0, cut_words()),
                Kept::File { kind, held: Held::Bytes(body) } => match kind {
                    Format::Offers => ways::check(self, path, body, true, &mut report),
                    Format::Way => ways::check(self, path, body, false, &mut report),
                    Format::Charter => charter(path, body, &mut report),
                    Format::Hooks => hooks_config(path, body, &mut report),
                },
                Kept::File { held: Held::Pending | Held::OverBudget, .. } | Kept::Directory { .. } | Kept::Fault(_) => {
                }
            }
        }
        let away = home_words(home);
        for path in METHOD {
            if self.entries.contains_key(path) {
                method = true;
                if let Some(why) = &away {
                    report.add(path, 0, why.clone());
                }
            }
        }
        Verdict { method, faults: report.faults.into_boxed(), more: report.more }
    }
}

pub(crate) struct Report {
    faults: List<Fault>,
    pub(crate) more: u32,
}

impl Report {
    pub(crate) fn add(&mut self, path: &[u8], line: u32, why: Box<[u8]>) {
        if self.faults.room() == 0 {
            self.more = self.more.saturating_add(1);
        } else {
            self.faults.push(Fault { path: bytes::copy_of(path), line, why }).expect("checked fault room");
        }
    }
}

fn cut_words() -> Box<[u8]> {
    bytes::copy_of(b"past what temper reads, 65,536 bytes")
}

fn home_words(home: Home) -> Option<Box<[u8]>> {
    const START: &[u8] = b"a project's ways live in its home repository, ";
    match home {
        Home::Here => None,
        Home::Elsewhere { name } => Some(bytes::concat(&[START, &name])),
        Home::None => Some(bytes::concat(&[START, b"and it has none"])),
    }
}

pub(crate) fn name_words(name: &[u8]) -> Box<[u8]> {
    bytes::concat(&[b"`", name, b"` is no name: 1 to 32 lowercase letters, digits and `-`, from a letter"])
}

pub(crate) const BAD_FORMAT: &[u8] = b"format: a whole number from 1";

pub(crate) fn newer(written: &[u8]) -> Box<[u8]> {
    bytes::concat(&[
        b"format ",
        written,
        b"; this ",
        READER,
        b" reads format 1: upgrade it, or write the file at format 1",
    ])
}

#[expect(clippy::disallowed_methods, reason = "the protocol decodes charter text and locates invalid UTF-8")]
fn charter(path: &[u8], body: &[u8], report: &mut Report) {
    let text = match core::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            let prefix = body.get(..error.valid_up_to()).expect("the UTF-8 error gives its valid prefix");
            let before = bytes::count(prefix, b"\n", crate::FILE_BYTES);
            report.add(path, before.saturating_add(1), bytes::copy_of(b"not UTF-8"));
            return;
        }
    };
    match temper_charter_set::read_file(text) {
        Ok(_) => {}
        Err(FormatFault::FrontMatter { line }) => {
            report.add(path, line, bytes::copy_of(b"front matter holds format = N alone"));
        }
        Err(FormatFault::BadFormat) => report.add(path, 2, bytes::copy_of(BAD_FORMAT)),
        Err(FormatFault::NewerFormat { number }) => report.add(path, 2, newer(number.as_bytes())),
    }
}

fn hooks_config(path: &[u8], body: &[u8], report: &mut Report) {
    if let Err(fault) = hooks::times(body) {
        report.add(path, fault.line(), fault.why(READER));
    }
}
