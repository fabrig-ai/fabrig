//! Landings and the page the writer and refresh share (temper/docs/design/client/journey-writer.md, 6 and 6.3).

use std::{collections::BTreeMap, fmt::Write as _, path::Path};

use temper_client_walk_lines::layout::{TUI, WEB};

use crate::{host::Host, look_compare::request::blob_at};

mod render;
pub(crate) use render::Render;

pub(crate) const LANDINGS: &str = "--format=%H%n%(trailers:key=Look-landed,valueonly,unfold)";
const RESULTS: &[&str] = &["identical", "differs", "not compared", "nothing to render"];
const NOT_CHECKED: &str = "not checked";
const FIELDS: [&str; 5] =
    ["data-look-request", "data-look-commit", "data-look-landed", "data-look-tip", "data-look-result"];
const TITLE: &str = "Look requests rendered · temper";

pub(crate) fn valid_name(name: &str) -> bool {
    let word_byte = |byte: u8| byte.is_ascii_lowercase() || byte.is_ascii_digit();
    let mut bytes = name.bytes();
    bytes.next().is_some_and(word_byte) && bytes.all(|byte| word_byte(byte) || byte == b'-')
}

pub(crate) fn hash(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

pub(crate) fn root(tui: bool) -> &'static str {
    if tui { TUI } else { WEB }
}

pub(crate) fn path(tui: bool) -> String {
    format!("{}/looks.html", root(tui))
}

pub(crate) fn is_tui(name: &str) -> bool {
    name.starts_with("tui-")
}

pub(crate) struct Landing {
    pub(crate) name: String,
    pub(crate) commit: String,
}

/// Git supplies commit records newest first in topological order, including merged side branches.
pub(crate) fn landings(bytes: &[u8]) -> Result<Vec<Landing>, String> {
    let log = std::str::from_utf8(bytes).map_err(|_| "Look-landed history is not UTF-8")?;
    let mut newest = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for commit in log.split_terminator('\0') {
        let (commit, names) = commit.split_once('\n').ok_or("malformed Look-landed history")?;
        if !hash(commit) {
            return Err("malformed Look-landed commit".into());
        }
        for name in names.lines() {
            if valid_name(name) && seen.insert(name) {
                newest.push(Landing { name: name.into(), commit: commit.into() });
            }
        }
    }
    Ok(newest)
}

#[derive(Default)]
pub(crate) struct Page {
    bytes: Box<[u8]>,
    entries: BTreeMap<(String, String), String>,
    first_section: Option<Insertion>,
    after_heading: Option<Insertion>,
}

struct Insertion {
    offset: usize,
    inline: bool,
}

impl Page {
    pub(crate) fn at(
        host: &mut Host,
        root: Option<&Path>,
        tip: &str,
        path: &str,
        caller: &str,
    ) -> Result<Self, String> {
        Ok(blob_at(host, root, tip, path)?.map_or_else(Self::default, |bytes| Self::read(bytes, path, caller, host)))
    }

    fn read(bytes: Box<[u8]>, path: &str, caller: &str, host: &mut Host) -> Self {
        let mut page = Self { bytes, ..Self::default() };
        let mut offset = 0;
        let mut lines = page.bytes.split_inclusive(|&byte| byte == b'\n').enumerate().peekable();
        while let Some((index, line)) = lines.next() {
            let start = offset;
            offset += line.len();
            let next_closes = lines.peek().is_some_and(|(_, line)| line.starts_with(b">"));
            let line = line.trim_ascii_end();
            if line.ends_with(b"</h1>") {
                page.after_heading.get_or_insert(Insertion { offset, inline: false });
            } else if line.ends_with(b"</h1") && next_closes {
                page.after_heading.get_or_insert(Insertion { offset: offset + 1, inline: true });
            }
            let leading = line.len() - line.trim_ascii_start().len();
            let line = line.trim_ascii_start();
            let (line, inline) = line.strip_prefix(b">").map_or((line, false), |rest| (rest, true));
            let Some(rest) = line.strip_prefix(b"<section") else { continue };
            if !rest.first().is_none_or(|byte| byte.is_ascii_whitespace() || *byte == b'>') {
                continue;
            }
            let position = if inline { start + leading + 1 } else { start };
            page.first_section.get_or_insert(Insertion { offset: position, inline });
            match std::str::from_utf8(rest).ok().and_then(|rest| entry(rest, next_closes)) {
                Some([name, _, landed, _, result]) => {
                    page.entries.entry((name.into(), landed.into())).or_insert_with(|| result.into());
                }
                None => host.err(format!("{caller}: ignoring malformed verdict at {path}:{}\n", index + 1).as_bytes()),
            }
        }
        page
    }

    pub(crate) fn result(&self, name: &str, landed: &str) -> Option<&str> {
        self.entries.get(&(name.into(), landed.into())).map(String::as_str)
    }

    pub(crate) fn prepend(self, entries: &[String]) -> Box<[u8]> {
        if self.bytes.is_empty() {
            let heading = tag_lines(&format!(
                "<html lang=\"en\"><head><meta charset=\"utf-8\"><title>{TITLE}</title></head><body><h1>{TITLE}</h1>"
            ));
            return [
                b"<!doctype html>\n".as_slice(),
                heading.as_bytes(),
                entries.concat().as_bytes(),
                b"</body\n></html\n>\n",
            ]
            .concat()
            .into_boxed_slice();
        }
        let Insertion { offset, inline } =
            self.first_section.or(self.after_heading).unwrap_or(Insertion { offset: 0, inline: false });
        let entries = if inline { entries.concat() } else { format!("{}\n", entries.join("\n")) };
        [&self.bytes[..offset], entries.as_bytes(), &self.bytes[offset..]].concat().into_boxed_slice()
    }
}

fn entry(mut rest: &str, next_closes: bool) -> Option<[&str; 5]> {
    let mut fields = [None; 5];
    loop {
        rest = rest.trim_ascii_start();
        if rest.starts_with('>') || (rest.is_empty() && next_closes) {
            break;
        }
        let (key, value) = rest.split_once('=')?;
        let value = value.trim_ascii_start();
        let quote = match value.as_bytes().first()? {
            b'"' => '"',
            b'\'' => '\'',
            _ => return None,
        };
        let (value, tail) = value[1..].split_once(quote)?;
        if let Some(index) = FIELDS.iter().position(|field| *field == key.trim_ascii_end())
            && fields[index].replace(value).is_some()
        {
            return None;
        }
        rest = tail;
    }
    let [Some(name), Some(commit), Some(landed), Some(tip), Some(result)] = fields else { return None };
    (valid_name(name)
        && hash(commit)
        && hash(landed)
        && hash(tip)
        && (RESULTS.contains(&result) || result == NOT_CHECKED))
        .then_some([name, commit, landed, tip, result])
}

pub(crate) enum Verdict {
    Checked(Render),
    NotChecked(String),
}

impl Verdict {
    pub(crate) fn result(&self) -> &str {
        match self {
            Self::Checked(render) => &render.result,
            Self::NotChecked(_) => NOT_CHECKED,
        }
    }

    pub(crate) fn section(&self, landing: &Landing, commit: &str, tip: &str) -> String {
        let Landing { name, commit: landed } = landing;
        let result = self.result();
        let [request_attr, commit_attr, landed_attr, tip_attr, result_attr] = FIELDS;
        let mut html = format!(
            "<section {request_attr}=\"{name}\" {commit_attr}=\"{commit}\" {landed_attr}=\"{landed}\" {tip_attr}=\"{tip}\" {result_attr}=\"{result}\"><h2>{name}</h2><p>Request {commit}, landed by {landed}, printed at {tip}: {result}.</p>"
        );
        match self {
            Self::Checked(render) => render.html(&mut html, is_tui(name)),
            Self::NotChecked(why) => write!(html, "<p>{}</p>", escaped(why)).expect("write to String"),
        }
        html.push_str("<p>States the request names in words aren't rendered: its landing's account covers them.</p>");
        if is_tui(name) {
            html.push_str("<p>These are the browser's renders of the TUI's frames, not a terminal's.</p>");
        }
        html.push_str("</section>");
        tag_lines(&html)
    }
}

// Put the break inside the tag, as in the journey print, so it changes no text.
fn tag_lines(html: &str) -> String {
    html.replace('>', "\n>")
}

fn escaped(value: &str) -> String {
    let mut text = String::new();
    for character in value.chars() {
        match character {
            '<' => text.push_str("&lt;"),
            '>' => text.push_str("&gt;"),
            '"' => text.push_str("&quot;"),
            '&' => text.push_str("&amp;"),
            other => text.push(other),
        }
    }
    text
}
