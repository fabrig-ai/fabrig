//! A set of charter files (temper/docs/design/domain/agent/charter.md, 1).
//! The borrowed set and caller's reason map are an assembly boundary, not stored state.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::boxed::Box;
use core::fmt;
use skein_lib::{List, Writer};

mod charter;
mod format;
pub use charter::{CharterFault, CharterFile, FilePart, WritePart, read_charter, write_charter};
pub use format::{FormatFault, read_file};

const PART_HEADING: &str = "## part, on:";
const BLANK_LINE: &str = "\n\n";
const CHARTER_BYTES: u32 = 65_536;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum File {
    Common,
    Goal,
    Design,
    Designer,
    Coordinator,
    Implementer,
}
impl File {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Common => "common.md",
            Self::Goal => "goal.md",
            Self::Design => "design.md",
            Self::Designer => "designer.md",
            Self::Coordinator => "coordinator.md",
            Self::Implementer => "implementer.md",
        }
    }
}
/// Identify a directory entry without reading it.
/// # Errors
/// Refuses a Markdown name outside the six files.
#[expect(clippy::case_sensitive_file_extension_comparisons, reason = "the set recognizes names ending .md exactly")]
pub fn file(name: &str) -> Result<Option<File>, Refusal> {
    let value = match name {
        "common.md" => File::Common,
        "goal.md" => File::Goal,
        "design.md" => File::Design,
        "designer.md" => File::Designer,
        "coordinator.md" => File::Coordinator,
        "implementer.md" => File::Implementer,
        _ => return if name.ends_with(".md") { Err(Refusal::UnknownFile { name: name.into() }) } else { Ok(None) },
    };
    Ok(Some(value))
}
#[derive(Clone, Copy, Debug)]
pub struct Set<'a> {
    pub common: Option<&'a str>,
    pub goal: Option<&'a str>,
    pub design: Option<&'a str>,
    pub designer: Option<&'a str>,
    pub coordinator: Option<&'a str>,
    pub implementer: Option<&'a str>,
}
#[derive(Debug, PartialEq, Eq)]
pub struct GoalAgent<R> {
    pub instructions: Box<[u8]>,
    pub parts: Box<[Part<R>]>,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Part<R> {
    pub reasons: Box<[R]>,
    pub text: Box<[u8]>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    UnknownFile { name: Box<str> },
    FrontMatter { file: File },
    BadFormat { file: File },
    NewerFormat { file: File, number: Box<str> },
    Settings { file: File, line: u32 },
    BeforeSection { file: File, line: usize },
    EmptyReason { file: File, line: usize },
    RepeatedReason { file: File, line: usize, name: Box<str> },
    UnknownReason { file: File, line: usize, name: Box<str> },
}
// Refusals carry their diagnostic words across the assembly boundary.
#[expect(clippy::disallowed_macros, reason = "a refusal renders its file, line and name at the assembly boundary")]
impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownFile { name } => write!(f, "{name}: unknown Markdown file"),
            Self::FrontMatter { file } => write!(f, "{}: front matter holds format = N alone", file.name()),
            Self::BadFormat { file } => write!(f, "{}: format: a whole number from 1", file.name()),
            Self::NewerFormat { file, number } => write!(
                f,
                "{}: format {number}; this temper ctl reads format 1: upgrade it, or write the file at format 1",
                file.name()
            ),
            Self::Settings { file, line } => write!(f, "{}:{line}: {}", file.name(), format::SETTINGS_WORDS),
            Self::BeforeSection { file, line } => write!(f, "{}:{line}: text before first section", file.name()),
            Self::EmptyReason { file, line } => write!(f, "{}:{line}: empty reason", file.name()),
            Self::RepeatedReason { file, line, name } => write!(f, "{}:{line}: repeated reason {name}", file.name()),
            Self::UnknownReason { file, line, name } => write!(f, "{}:{line}: unknown reason {name}", file.name()),
        }
    }
}
/// Assemble the goal agent's instructions and ordered parts.
/// # Errors
/// Refuses text before a coordinator section, or empty, repeated or unknown reasons.
#[expect(
    clippy::impl_trait_in_params,
    clippy::disallowed_types,
    reason = "the assembly boundary maps names to the caller's reason type"
)]
pub fn goal_agent<R>(set: &Set<'_>, reason: impl Fn(&str) -> Option<R>) -> Result<GoalAgent<R>, Refusal> {
    let mut parts = List::uncapped();
    let designer = trimmed(set.designer);
    if !designer.is_empty() {
        let Some(first) = reason("first") else {
            return Err(Refusal::UnknownReason { file: File::Designer, line: 1, name: "first".into() });
        };
        assert!(
            parts.push(Part { reasons: Box::new([first]), text: designer.as_bytes().into() }).is_ok(),
            "part count fits u32"
        );
    }
    let coordinator = set.coordinator.unwrap_or("");
    let mut start = 0_usize;
    let mut offset = 0_usize;
    let mut current = None;
    for (index, line) in coordinator.split_inclusive('\n').enumerate() {
        if let Some(names) = heading(line) {
            if let Some(reasons) = current.take() {
                assert!(parts.push(part(reasons, coordinator, start, offset)).is_ok(), "part count fits u32");
            }
            let number = index.checked_add(1).expect("line within text");
            current = Some(reasons(names, number, &reason)?);
            start = offset.checked_add(line.len()).expect("section within text");
        } else if current.is_none() && line != "\n" {
            return Err(Refusal::BeforeSection {
                file: File::Coordinator,
                line: index.checked_add(1).expect("line within text"),
            });
        }
        offset = offset.checked_add(line.len()).expect("line within text");
    }
    if let Some(reasons) = current {
        assert!(parts.push(part(reasons, coordinator, start, coordinator.len())).is_ok(), "part count fits u32");
    }
    Ok(GoalAgent { instructions: joined(&[set.common, set.goal, set.design]), parts: parts.into_boxed() })
}
#[expect(clippy::disallowed_types, reason = "the caller's reason map is the assembly boundary")]
fn reasons<R>(names: &str, line: usize, reason: &impl Fn(&str) -> Option<R>) -> Result<Box<[R]>, Refusal> {
    let mut seen = List::uncapped();
    let mut values = List::uncapped();
    for name in names.split(',') {
        let file = File::Coordinator;
        let name = match checked_reason(name, seen.as_slice()) {
            Ok(name) => name,
            Err(ReasonFault::Empty) => return Err(Refusal::EmptyReason { file, line }),
            Err(ReasonFault::Repeated(name)) => return Err(Refusal::RepeatedReason { file, line, name }),
        };
        let Some(value) = reason(name) else {
            return Err(Refusal::UnknownReason { file, line, name: name.into() });
        };
        seen.push(name).expect("reason count fits u32");
        assert!(values.push(value).is_ok(), "reason count fits u32");
    }
    Ok(values.into_boxed())
}
fn part<R>(reasons: Box<[R]>, text: &str, start: usize, end: usize) -> Part<R> {
    Part { reasons, text: section_text(text, start, end).as_bytes().into() }
}

fn heading(line: &str) -> Option<&str> {
    line.trim_end_matches('\n').strip_prefix(PART_HEADING)
}

enum ReasonFault {
    Empty,
    Repeated(Box<str>),
}

fn checked_reason<'a>(name: &'a str, seen: &[&str]) -> Result<&'a str, ReasonFault> {
    let name = name.trim_matches(' ');
    if name.is_empty() {
        return Err(ReasonFault::Empty);
    }
    if seen.contains(&name) {
        return Err(ReasonFault::Repeated(name.into()));
    }
    Ok(name)
}

fn section_text(text: &str, start: usize, end: usize) -> &str {
    trim_text(text.get(start..end).expect("section offsets are line boundaries"))
}

fn trim_text(text: &str) -> &str {
    text.trim_matches('\n')
}
/// Assemble the implementer's instructions; absent and blank files contribute nothing.
#[must_use]
pub fn implementer(set: &Set<'_>) -> Box<[u8]> {
    joined(&[set.common, set.implementer])
}
fn trimmed(text: Option<&str>) -> &str {
    trim_text(text.unwrap_or(""))
}
fn joined(files: &[Option<&str>]) -> Box<[u8]> {
    let mut length = 0_usize;
    for file in files {
        let file = trimmed(*file);
        if !file.is_empty() {
            if length != 0 {
                length = length.checked_add(BLANK_LINE.len()).expect("separator length");
            }
            length = length.checked_add(file.len()).expect("joined text length");
        }
    }
    let mut text = Writer::new(length);
    let mut first = true;
    for file in files {
        let file = trimmed(*file);
        if !file.is_empty() {
            if !first {
                text.put(BLANK_LINE.as_bytes()).expect("separator counted");
            }
            text.put(file.as_bytes()).expect("file counted");
            first = false;
        }
    }
    text.finish()
}
