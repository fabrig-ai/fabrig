//! A set of charter files (temper/docs/design/domain/agent/charter.md, 1).
//! The borrowed set and caller's reason map are an assembly boundary, not stored state.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::boxed::Box;
use core::fmt;
use skein_lib::{List, Writer};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum File {
    Common,
    Goal,
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
            Self::Designer => "designer.md",
            Self::Coordinator => "coordinator.md",
            Self::Implementer => "implementer.md",
        }
    }
}
/// Identify a directory entry without reading it.
/// # Errors
/// Refuses a Markdown name outside the five files.
#[expect(clippy::case_sensitive_file_extension_comparisons, reason = "the set recognizes names ending .md exactly")]
pub fn file(name: &str) -> Result<Option<File>, Refusal> {
    let value = match name {
        "common.md" => File::Common,
        "goal.md" => File::Goal,
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
        let heading = line.trim_end_matches('\n').strip_prefix("## part, on:");
        if let Some(names) = heading {
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
    Ok(GoalAgent { instructions: joined(set.common, set.goal), parts: parts.into_boxed() })
}
#[expect(clippy::disallowed_types, reason = "the caller's reason map is the assembly boundary")]
fn reasons<R>(names: &str, line: usize, reason: &impl Fn(&str) -> Option<R>) -> Result<Box<[R]>, Refusal> {
    let mut seen = List::uncapped();
    let mut values = List::uncapped();
    for name in names.split(',') {
        let name = name.trim_matches(' ');
        let file = File::Coordinator;
        if name.is_empty() {
            return Err(Refusal::EmptyReason { file, line });
        }
        for earlier in &seen {
            if *earlier == name {
                return Err(Refusal::RepeatedReason { file, line, name: name.into() });
            }
        }
        let Some(value) = reason(name) else {
            return Err(Refusal::UnknownReason { file, line, name: name.into() });
        };
        seen.push(name).expect("reason count fits u32");
        assert!(values.push(value).is_ok(), "reason count fits u32");
    }
    Ok(values.into_boxed())
}
fn part<R>(reasons: Box<[R]>, text: &str, start: usize, end: usize) -> Part<R> {
    let text = text.get(start..end).expect("section offsets are line boundaries").trim_matches('\n');
    Part { reasons, text: text.as_bytes().into() }
}
/// Assemble the implementer's instructions; absent and blank files contribute nothing.
#[must_use]
pub fn implementer(set: &Set<'_>) -> Box<[u8]> {
    joined(set.common, set.implementer)
}
fn trimmed(text: Option<&str>) -> &str {
    text.unwrap_or("").trim_matches('\n')
}
fn joined(first: Option<&str>, second: Option<&str>) -> Box<[u8]> {
    let first = trimmed(first);
    let second = trimmed(second);
    let separator = if first.is_empty() || second.is_empty() { b"".as_slice() } else { b"\n\n" };
    let length = first.len().checked_add(separator.len()).expect("first and separator length");
    let length = length.checked_add(second.len()).expect("joined text length");
    let mut text = Writer::new(length);
    text.put(first.as_bytes()).expect("first counted");
    text.put(separator).expect("separator counted");
    text.put(second.as_bytes()).expect("second counted");
    text.finish()
}
