//! Compiled user topics, paging and search (protocol/documentation.md, sections 2 and 3).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

use skein_lib::List;

pub const TOPIC_BYTES: usize = 32_768;
pub const TOPICS_MAX: usize = 64;
pub const NAME_BYTES: usize = 32;
pub const TITLE_BYTES: usize = 80;
pub const SUMMARY_BYTES: usize = 200;
pub const HEADING_BYTES: usize = 80;
pub const LINE_BYTES: usize = 120;
pub const WORDS: usize = 8;
pub const WORD_BYTES: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Walk,
    Concept,
    Recipe,
    Limits,
}
impl Kind {
    /// The word of its kind line: "walk", "concept", "recipe", "limits".
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Self::Walk => "walk",
            Self::Concept => "concept",
            Self::Recipe => "recipe",
            Self::Limits => "limits",
        }
    }
}

#[derive(Debug)]
pub struct Topic {
    pub name: &'static str,
    pub kind: Kind,
    pub title: &'static str,
    pub summary: &'static str,
    pub text: &'static str,
}

/// Walks, concepts, recipes, limits; each kind by name.
#[expect(clippy::disallowed_macros, reason = "topics compile their single Markdown source as static data")]
pub static TOPICS: [Topic; 8] = [
    Topic {
        name: "box",
        kind: Kind::Walk,
        title: "Start a local temper box",
        summary: "Run temper, an owned Forgejo and its runner on one machine, on loopback, from a checkout of temper, then adopt a repository and ask for work.",
        text: include_str!("../../../docs/help/box.md"),
    },
    Topic {
        name: "forgejo",
        kind: Kind::Walk,
        title: "Connect Forgejo",
        summary: "Connect temper to a Forgejo as its bot, in one of three modes: connected, managed or owned.",
        text: include_str!("../../../docs/help/forgejo.md"),
    },
    Topic {
        name: "github",
        kind: Kind::Walk,
        title: "Connect GitHub",
        summary: "Connect temper to GitHub through a GitHub App, temper's own or one you make, and install it on the accounts whose repositories it works on.",
        text: include_str!("../../../docs/help/github.md"),
    },
    Topic {
        name: "server",
        kind: Kind::Walk,
        title: "Run temper on a server",
        summary: "Run temper as a long-lived deployment on a server, behind a TLS proxy, with Google sign-in, an owned Forgejo and its workers on another host.",
        text: include_str!("../../../docs/help/server.md"),
    },
    Topic {
        name: "budgets",
        kind: Kind::Concept,
        title: "Budgets",
        summary: "How temper counts what work spends, where a task's budget comes from, what happens when it runs out, and what you can do then.",
        text: include_str!("../../../docs/help/budgets.md"),
    },
    Topic {
        name: "charters",
        kind: Kind::Concept,
        title: "Charters, templates and policies",
        summary: "What decides how temper's agents work: a charter for each run, a template for each new task, and the policies and rules that bound what work may do.",
        text: include_str!("../../../docs/help/charters.md"),
    },
    Topic {
        name: "settings",
        kind: Kind::Concept,
        title: "Settings, revisions and proposals",
        summary: "How you read and change temper's settings with temper ctl, what a revision is, how a proposal asks the deployment's owners, and what still needs a restart.",
        text: include_str!("../../../docs/help/settings.md"),
    },
    Topic {
        name: "tasks",
        kind: Kind::Concept,
        title: "Tasks, goals and chats",
        summary: "What temper's work is made of, tasks with their delegates, goals and chats, and how you follow, answer, steer and stop them.",
        text: include_str!("../../../docs/help/tasks.md"),
    },
];

#[must_use]
#[expect(clippy::manual_find, reason = "the programming model uses bounded loops without closures")]
pub fn find(name: &[u8]) -> Option<&'static Topic> {
    for topic in &TOPICS {
        if topic.name.as_bytes() == name {
            return Some(topic);
        }
    }
    None
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Page {
    pub text: &'static str,
    pub next: Option<u32>,
}

#[must_use]
pub fn page(topic: &Topic, from: u32, bytes: u32) -> Page {
    assert!(bytes >= 2, "a page must fit every rendered byte");
    let from = usize::try_from(from).expect("offset fits usize");
    if from >= topic.text.len() {
        return Page { text: "", next: None };
    }
    let mut end = from;
    let mut line_end = from;
    let mut rendered = 0_u32;
    for byte in topic.text.as_bytes().get(from..).expect("offset within text") {
        let size = match byte {
            b'"' | b'\\' | b'\n' => 2,
            _ => 1,
        };
        if size > bytes.checked_sub(rendered).expect("page within budget") {
            break;
        }
        rendered = rendered.checked_add(size).expect("page within budget");
        end = end.checked_add(1).expect("topic offset fits usize");
        if *byte == b'\n' {
            line_end = end;
        }
    }
    if end == topic.text.len() {
        return Page { text: topic.text.get(from..).expect("ASCII topic"), next: None };
    }
    if line_end > from {
        end = line_end;
    }
    Page {
        text: topic.text.get(from..end).expect("ASCII topic"),
        next: Some(u32::try_from(end).expect("topic offset fits u32")),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hit {
    pub topic: &'static str,
    pub heading: &'static str,
    pub offset: u32,
    pub line: &'static str,
}
#[derive(Debug)]
pub struct Hits {
    pub hits: List<Hit>,
    pub more: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    NoWords,
    TooManyWords,
    WordTooLong,
}
impl Refused {
    /// "nothing to search for", "at most 8 words", "a word is at most 32 bytes".
    #[must_use]
    pub fn says(self) -> &'static str {
        match self {
            Self::NoWords => "nothing to search for",
            Self::TooManyWords => "at most 8 words",
            Self::WordTooLong => "a word is at most 32 bytes",
        }
    }
}

// A section begins outside a fence; its opening heading is part of it.
fn section_end(text: &str, from: usize) -> usize {
    let mut at = from;
    let mut fenced = false;
    for line in text.get(from..).expect("section in topic").split_inclusive('\n') {
        if line.starts_with("```") {
            fenced = !fenced;
        } else if at != from && !fenced && line.starts_with("## ") {
            return at;
        }
        at = at.checked_add(line.len()).expect("topic offset fits usize");
    }
    at
}

fn contains(text: &str, word: &[u8]) -> bool {
    for window in text.as_bytes().windows(word.len()) {
        if window.eq_ignore_ascii_case(word) {
            return true;
        }
    }
    false
}

fn matching_line(section: &'static str, word: &[u8]) -> &'static str {
    for line in section.split('\n') {
        if contains(line, word) {
            return line.get(..line.len().min(LINE_BYTES)).expect("ASCII topic");
        }
    }
    unreachable!("matching section has a line holding its first word")
}

fn next_word(words: &[u8], mut at: usize) -> Option<(&[u8], usize)> {
    while let Some(byte) = words.get(at) {
        if !byte.is_ascii_whitespace() {
            break;
        }
        at = at.checked_add(1).expect("word offset fits usize");
    }
    let start = at;
    while let Some(byte) = words.get(at) {
        if byte.is_ascii_whitespace() {
            break;
        }
        at = at.checked_add(1).expect("word offset fits usize");
    }
    if at == start { None } else { Some((words.get(start..at).expect("word in input"), at)) }
}

pub fn search(table: &'static [Topic], words: &[u8], limit: u32) -> Result<Hits, Refused> {
    let mut asked = List::with_capacity(u32::try_from(WORDS).expect("word count fits u32"));
    let mut at = 0_usize;
    while let Some((word, next)) = next_word(words, at) {
        at = next;
        if asked.len() == u32::try_from(WORDS).expect("word count fits u32") {
            return Err(Refused::TooManyWords);
        }
        if word.len() > WORD_BYTES {
            return Err(Refused::WordTooLong);
        }
        asked.push(word).expect("word count checked");
    }
    let Some(first) = asked.as_slice().first() else { return Err(Refused::NoWords) };
    let mut hits = List::with_capacity(limit);
    for topic in table {
        let mut from = 0_usize;
        while from < topic.text.len() {
            let end = section_end(topic.text, from);
            let section = topic.text.get(from..end).expect("section within topic");
            let mut matches = true;
            for word in asked.as_slice() {
                if !contains(section, word) {
                    matches = false;
                    break;
                }
            }
            if matches {
                if hits.len() == limit {
                    return Ok(Hits { hits, more: true });
                }
                let heading = if from == 0 {
                    topic.title
                } else {
                    section.lines().next().expect("section heading").strip_prefix("## ").expect("section heading")
                };
                hits.push(Hit {
                    topic: topic.name,
                    heading,
                    offset: u32::try_from(from).expect("topic offset fits u32"),
                    line: matching_line(section, first),
                })
                .expect("hit count checked");
            }
            from = end;
        }
    }
    Ok(Hits { hits, more: false })
}

#[cfg(test)]
mod tests;
