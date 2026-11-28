//! Ordering and wording the tree's faults (temper/docs/design/protocol/temper-dir.md, 8).
use crate::reading::{Format, Held, Kept, Reading};
use crate::{DeploymentCharters, FAULTS, Fault, Home, METHOD, READER, ROOT, Verdict, charter, ways};
use alloc::boxed::Box;
use skein_lib::{List, bytes};
use temper_protocol_hooks as hooks;

impl Reading {
    /// Judge the completed reading, including a reading stopped at a bound.
    #[must_use]
    pub fn judge(&self, home: Home, deployment: &DeploymentCharters) -> Verdict {
        let mut report = Report { faults: List::with_capacity(FAULTS), more: 0 };
        if self.root_cut {
            report.add(ROOT, 0, cut_words());
        }
        let mut charters = 0_u32;
        for (path, entry) in &self.entries {
            match entry {
                Kept::File { kind: Format::Charter, .. } => charters = charters.saturating_add(1),
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
                    Format::Charter => charter::check(path, body, deployment, &mut report),
                    Format::Hooks => hooks_config(path, body, &mut report),
                },
                Kept::File { held: Held::Pending | Held::OverBudget, .. } | Kept::Directory { .. } | Kept::Fault(_) => {
                }
            }
        }
        charter::check_count(charters, deployment, &mut report);
        let away = home_words(home);
        for path in METHOD {
            if self.entries.contains_key(path)
                && let Some(why) = &away
            {
                report.add(path, 0, why.clone());
            }
        }
        Verdict { method: self.has_method(), faults: report.faults.into_boxed(), more: report.more }
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

fn hooks_config(path: &[u8], body: &[u8], report: &mut Report) {
    if let Err(fault) = hooks::times(body) {
        report.add(path, fault.line(), fault.why(READER));
    }
}
