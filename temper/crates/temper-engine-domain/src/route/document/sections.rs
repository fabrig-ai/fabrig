//! Pure section assembly (temper/docs/design/domain/agent/ask-temper.md, 3).
use crate::{DocumentAction, DocumentRefusal};
use alloc::boxed::Box;
use skein_lib::{List, Writer};

use super::Made;

/// Make the whole note from the document read and one section call.
pub(crate) fn made(
    read: Option<(&[u8], &[u8])>,
    action: DocumentAction,
    body: &[u8],
    limits: &jig_domain_notes::Limits,
) -> Result<Made, DocumentRefusal> {
    let (description, body) = match action {
        DocumentAction::Write => {
            let first = first_line(body);
            let title = first.strip_prefix(b"# ").ok_or(DocumentRefusal::NoTitle)?.trim_ascii();
            if title.is_empty() || title.starts_with(b"work:") {
                return Err(DocumentRefusal::NoTitle);
            }
            if title.len() > skein_lib::usize_of(limits.description_bytes) {
                return Err(DocumentRefusal::TitlePastBound {
                    bytes: count(title.len()),
                    bound: limits.description_bytes,
                });
            }
            (title.into(), joined(&[], body, &[], limits)?)
        }
        DocumentAction::Append | DocumentAction::Replace => {
            let sections = sections_of(body);
            if !body.starts_with(b"## ") || sections.len() != 1 {
                return Err(DocumentRefusal::NotOneSection);
            }
            let (description, old) = read.ok_or(DocumentRefusal::NoDocument)?;
            let named = heading(action, body);
            let old_sections = sections_of(old);
            let mut found = None;
            for section in &old_sections {
                let title = first_line(old.get(section.start..section.end).expect("section range"))
                    .strip_prefix(b"## ")
                    .expect("section heading");
                if title.trim_ascii().eq_ignore_ascii_case(named.trim_ascii()) && found.is_none() {
                    found = Some(section);
                }
            }
            let body = match action {
                DocumentAction::Append => {
                    if found.is_some() {
                        return Err(DocumentRefusal::SectionThere);
                    }
                    let old = old.get(..text_end(old)).expect("last text line");
                    let separator = if old.is_empty() { b"".as_slice() } else { b"\n\n".as_slice() };
                    joined_append(old, separator, body, limits)?
                }
                DocumentAction::Replace => {
                    let Some(section) = found else { return Err(no_section(old, &old_sections)) };
                    let before = old.get(..section.start).expect("section start");
                    let section_body = old.get(section.start..section.end).expect("old section");
                    let end = section.start.checked_add(text_end(section_body)).expect("body offset");
                    let after = old.get(end..).expect("empty lines after the replaced section");
                    joined(before, trim_newlines(body), after, limits)?
                }
                DocumentAction::Write => unreachable!("section action"),
            };
            (description.into(), body)
        }
    };
    Ok(Made { description, body })
}

/// The title or section heading as the caller gave it, without the Markdown prefix.
pub(crate) fn heading(action: DocumentAction, body: &[u8]) -> Box<[u8]> {
    let prefix = match action {
        DocumentAction::Write => b"# ".as_slice(),
        DocumentAction::Append | DocumentAction::Replace => b"## ".as_slice(),
    };
    first_line(body).strip_prefix(prefix).unwrap_or_default().into()
}

fn first_line(body: &[u8]) -> &[u8] {
    let end = skein_lib::bytes::find(body, b"\n").unwrap_or(body.len());
    body.get(..end).expect("first line range")
}
fn count(bytes: usize) -> u32 {
    u32::try_from(bytes).unwrap_or(u32::MAX)
}
fn trim_newlines(body: &[u8]) -> &[u8] {
    let mut end = body.len();
    for byte in body.iter().rev() {
        if *byte != b'\n' {
            break;
        }
        end = end.checked_sub(1).expect("trailing newline offset");
    }
    body.get(..end).expect("body without trailing newlines")
}

#[derive(Debug)]
struct Section {
    start: usize,
    end: usize,
}
fn sections_of(body: &[u8]) -> List<Section> {
    let mut sections: List<Section> =
        List::with_capacity(skein_lib::bytes::count(body, b"\n", u32::MAX).saturating_add(1));
    let mut start: usize = 0;
    let mut fenced = false;
    for (end, byte) in body.iter().chain(core::iter::once(&b'\n')).enumerate() {
        if *byte != b'\n' || start == body.len() {
            continue;
        }
        let line = body.get(start..end).expect("line range");
        if line.starts_with(b"```") || line.starts_with(b"~~~") {
            fenced = !fenced;
        } else if !fenced && line.starts_with(b"## ") {
            if let Some(section) = sections.get_mut(sections.len().saturating_sub(1)) {
                section.end = start;
            }
            sections.push(Section { start, end: body.len() }).expect("at most one section per line");
        }
        start = end.checked_add(1).expect("body offset");
    }
    sections
}
fn text_end(body: &[u8]) -> usize {
    let mut start: usize = 0;
    let mut end = 0;
    for (at, byte) in body.iter().chain(core::iter::once(&b'\n')).enumerate() {
        if *byte != b'\n' || start == body.len() {
            continue;
        }
        let text = body.get(start..at).expect("line range");
        if !text.trim_ascii().is_empty() {
            end = at;
        }
        start = at.checked_add(1).expect("body offset");
    }
    end
}
fn no_section(body: &[u8], sections: &List<Section>) -> DocumentRefusal {
    const MOST: usize = 1024;
    let mut headings = List::with_capacity(sections.len());
    let mut bytes: usize = 0;
    let mut more = false;
    for section in sections {
        let heading = first_line(body.get(section.start..section.end).expect("section range"))
            .strip_prefix(b"## ")
            .expect("heading");
        let separator = if headings.is_empty() { 0 } else { " · ".len() };
        let size = bytes.saturating_add(separator).saturating_add(b"## ".len()).saturating_add(heading.len());
        if size > MOST {
            more = true;
            break;
        }
        bytes = size;
        headings.push(heading.into()).expect("bounded section headings");
    }
    DocumentRefusal::NoSection { headings: headings.into_boxed(), more }
}
fn joined_append(
    before: &[u8],
    separator: &[u8],
    body: &[u8],
    limits: &jig_domain_notes::Limits,
) -> Result<Box<[u8]>, DocumentRefusal> {
    let size = before.len().saturating_add(separator.len());
    let mut prefix = Writer::new(size);
    prefix.put(before).expect("prefix size");
    prefix.put(separator).expect("separator size");
    joined(&prefix.finish(), body, &[], limits)
}
fn joined(
    before: &[u8],
    body: &[u8],
    after: &[u8],
    limits: &jig_domain_notes::Limits,
) -> Result<Box<[u8]>, DocumentRefusal> {
    let tail = if after.is_empty() { trim_newlines(body) } else { body };
    let after = trim_newlines(after);
    let size = before.len().saturating_add(tail.len()).saturating_add(after.len()).saturating_add(1);
    if size > skein_lib::usize_of(limits.body_bytes) {
        return Err(DocumentRefusal::PastBound { bytes: count(size), bound: limits.body_bytes });
    }
    let mut out = Writer::new(size);
    out.put(before).expect("joined size");
    out.put(tail).expect("joined size");
    out.put(after).expect("joined size");
    out.put(b"\n").expect("joined size");
    Ok(out.finish())
}
