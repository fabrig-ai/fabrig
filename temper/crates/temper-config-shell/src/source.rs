//! File lines and the inverse mapping from a start's members (file.md, 2 and 5).
use crate::{FILE, rows};
use jig_protocol_config::{Path, Segment};
use std::fmt::{self, Write};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal {
    pub problems: Vec<String>,
}
impl Refusal {
    pub(crate) fn empty() -> Self {
        Self { problems: Vec::new() }
    }
    pub(crate) fn one(words: String) -> Self {
        Self { problems: vec![words] }
    }
    pub(crate) fn push(&mut self, key: &str, line: Option<usize>, words: &str) {
        self.problems.push(diagnostic(key, line, words, false));
    }
    #[must_use]
    pub fn lines(&self) -> Box<[u8]> {
        let mut text = self.problems.join("\n");
        text.push('\n');
        text.into_bytes().into_boxed_slice()
    }
}
impl fmt::Display for Refusal {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.write_str(&self.problems.join("\n"))
    }
}
impl std::error::Error for Refusal {}

pub(crate) fn diagnostic(key: &str, line: Option<usize>, words: &str, default: bool) -> String {
    let mut text = FILE.to_owned();
    if let Some(line) = line {
        write!(text, ":{line}").expect("string");
    }
    if !key.is_empty() {
        write!(text, ": {key}").expect("string");
        if default {
            text.push_str(" (its default)");
        }
    }
    write!(text, ": {words}").expect("string");
    text
}

#[derive(Debug, Default)]
pub struct Keys {
    pub(crate) sources: Vec<(String, usize)>,
    members: Vec<(String, String)>,
}
impl Keys {
    pub(crate) fn add(&mut self, member: &str, key: &str) {
        self.members.push((member.into(), key.into()));
    }
    pub(crate) fn line(&self, key: &str) -> Option<usize> {
        self.sources.iter().find(|(found, _)| found == key).map(|(_, line)| *line)
    }
    /// A file-only key, such as a program path, has no member in the start's document.
    #[must_use]
    pub fn render_key(&self, key: &str, words: &str) -> String {
        let line = self.line(key);
        diagnostic(key, line, words, line.is_none())
    }
    /// A member's path and the start's own words, without a final newline.
    #[must_use]
    pub fn render(&self, path: &Path, words: &str) -> String {
        let mut text = String::new();
        for segment in &path.segments {
            match segment {
                Segment::Member(name) => {
                    if !text.is_empty() {
                        text.push('.');
                    }
                    text.push_str(&String::from_utf8_lossy(name));
                }
                Segment::Index(index) => write!(text, "[{index}]").expect("string"),
            }
        }
        self.render_member(&text, words)
    }
    /// Render one already-spelled member path; paths outside the document are unchanged.
    #[must_use]
    pub fn render_member(&self, path: &str, words: &str) -> String {
        let found =
            self.members.iter().filter(|(member, _)| ancestor(member, path)).max_by_key(|(member, _)| member.len());
        match found {
            Some((_, key)) => {
                let line = self.line(key);
                diagnostic(key, line, words, line.is_none())
            }
            None => format!("{path}: {words}"),
        }
    }
    pub(crate) fn record_rows(&mut self, shape: crate::Shape, file: &crate::File, facts: &crate::Facts) {
        // Expand array templates from the keys written, including defaults of their fields.
        for row in rows::ROWS.iter().filter(|row| row.shapes.contains(&shape)) {
            if (row.key.starts_with("openid")
                && file.openid.is_none()
                && !(shape == crate::Shape::Standalone && facts.provider_key))
                || (row.key.starts_with("sign_in.") && file.sign_in.is_none())
                || (row.key.starts_with("push") && file.push.is_none())
                || (row.key.starts_with("test.github") && file.test.github.is_none())
                || (row.key.starts_with("test.google") && file.sign_in.is_none())
                || (row.key.starts_with("test.google.") && file.test.google.is_none())
                || (row.key.starts_with("test.push") && file.push.is_none())
                || (row.key == "environment" && file.environment.is_empty())
            {
                continue;
            }
            if row.form == rows::Form::Members || row.form == rows::Form::Records {
                let prefix = format!("{}[", row.key);
                let entries: Vec<_> = self
                    .sources
                    .iter()
                    .filter(|(key, _)| key.starts_with(&prefix) && key.ends_with(']'))
                    .map(|(key, _)| key.clone())
                    .collect();
                for key in entries {
                    for member in row.members {
                        self.add(&format!("{member}{}", key.strip_prefix(row.key).expect("array suffix")), &key);
                    }
                }
            }
            if let Some((parent, field)) = row.key.split_once("[].") {
                let prefix = format!("{parent}[");
                let entries: std::collections::BTreeSet<_> = self
                    .sources
                    .iter()
                    .filter_map(|(key, _)| {
                        let rest = key.strip_prefix(&prefix)?;
                        let (index, _) = rest.split_once(']')?;
                        Some(index.to_owned())
                    })
                    .collect();
                for index in entries {
                    let key = format!("{parent}[{index}].{field}");
                    for member in row.members {
                        self.add(&member.replace("[]", &format!("[{index}]")), &key);
                    }
                }
            } else {
                for member in row.members {
                    self.add(member, row.key);
                }
            }
        }
        for (index, (name, _)) in file.environment.iter().enumerate() {
            let key = format!("environment.{name}");
            self.members.push((format!("application.environment[{index}]"), key));
        }
    }
}
fn ancestor(parent: &str, child: &str) -> bool {
    child == parent || child.strip_prefix(parent).is_some_and(|rest| rest.starts_with('.') || rest.starts_with('['))
}

/// Index line breaks once; reading a span does not scan the file again.
pub(crate) struct Lines(Vec<usize>);
impl Lines {
    pub(crate) fn new(text: &str) -> Self {
        Self(text.match_indices('\n').map(|(offset, _)| offset).collect())
    }

    pub(crate) fn at(&self, offset: usize) -> usize {
        self.0.partition_point(|newline| *newline < offset) + 1
    }
}

pub(crate) fn at(text: &str, offset: usize) -> usize {
    text.get(..offset).expect("parser byte span").bytes().filter(|byte| *byte == b'\n').count() + 1
}
