//! A question's line form (temper/docs/design/domain/agent.md, 8.2).
use alloc::boxed::Box;
use skein_lib::{List, Writer, bytes};

pub const CHOICES: u32 = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    /// What the button says, and the answer's words: no prefix or recommendation ending.
    pub words: Box<[u8]>,
    pub recommended: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parsed {
    pub question: Box<[u8]>,
    /// Empty where there is none.
    pub context: Box<[u8]>,
    pub saw: Option<Box<[u8]>>,
    /// Empty on the fallback: answered by a reply alone.
    pub choices: Box<[Choice]>,
}

impl Parsed {
    /// The choice drawn primary: the recommended one, else the first.
    #[must_use]
    pub fn primary(&self) -> Option<usize> {
        for (index, choice) in self.choices.iter().enumerate() {
            if choice.recommended {
                return Some(index);
            }
        }
        if self.choices.is_empty() { None } else { Some(0) }
    }
}

/// `cut` is the read's `question.words.cut`.
#[must_use]
pub fn parse(words: &[u8], cut: bool) -> Parsed {
    let lines = lines(words);
    let lines = lines.as_slice();
    if cut {
        return fallback(lines);
    }
    let mut trailing = lines.len();
    for (index, line) in lines.iter().enumerate().skip(1).rev() {
        if !line.trim_ascii_end().is_empty() && !line.starts_with(b"Saw: ") && !line.starts_with(b"Choice: ") {
            break;
        }
        trailing = index;
    }
    let mut choices = List::uncapped();
    let mut saw = List::uncapped();
    let mut marked = false;
    let mut choice_lines = 0_u32;
    for line in lines.get(trailing..).expect("trailing begins at a line boundary") {
        if let Some(words) = line.strip_prefix(b"Choice: ") {
            choice_lines = choice_lines.checked_add(1).expect("bounded question lines");
            if choice_lines > CHOICES {
                return fallback(lines);
            }
            let words = words.trim_ascii_end();
            let ending = words.strip_suffix(b" (recommended)");
            let words = ending.unwrap_or(words).trim_ascii();
            if words.is_empty()
                || words.len() > usize::try_from(temper_client_protocol_wire::CUE_BYTES).expect("cue bound fits usize")
            {
                continue;
            }
            let recommended = ending.is_some() && !marked;
            marked |= recommended;
            choices
                .push(Choice { words: bytes::copy_of(words), recommended })
                .expect("choices fit the question's line count");
        } else if let Some(words) = line.strip_prefix(b"Saw: ") {
            saw.push(words.trim_ascii_end()).expect("saw lines fit the question's line count");
        }
    }
    if choices.is_empty() {
        return fallback(lines);
    }
    Parsed {
        question: bytes::copy_of(lines.first().expect("even empty words have a first line").trim_ascii_end()),
        context: join(lines.get(1..trailing).expect("the first line is never trailing")),
        saw: if saw.is_empty() { None } else { Some(join_all(saw.as_slice())) },
        choices: choices.into_boxed(),
    }
}

fn lines(words: &[u8]) -> List<&[u8]> {
    let mut lines = List::uncapped();
    let mut start = 0;
    for end in 0..=words.len() {
        if end == words.len() || words.get(end) == Some(&b'\n') {
            let line = words.get(start..end).expect("line boundaries come from the input");
            lines.push(line).expect("line count fits the question's byte bound");
            start = end.saturating_add(1);
        }
    }
    lines
}

fn fallback(lines: &[&[u8]]) -> Parsed {
    let mut paragraph = lines.len();
    for (index, line) in lines.iter().enumerate() {
        if line.trim_ascii_end().is_empty() {
            paragraph = index;
            break;
        }
    }
    Parsed {
        question: join(lines.get(..paragraph).expect("paragraph ends at a line boundary")),
        context: join(lines.get(paragraph..).expect("context begins at a line boundary")),
        saw: None,
        choices: Box::from([]),
    }
}

fn join(lines: &[&[u8]]) -> Box<[u8]> {
    let mut start = lines.len();
    let mut end = 0;
    for (index, line) in lines.iter().enumerate() {
        if !line.trim_ascii_end().is_empty() {
            start = start.min(index);
            end = index.checked_add(1).expect("a line follows its index");
        }
    }
    let lines = lines.get(start..end).unwrap_or(&[]);
    join_all(lines)
}

fn join_all(lines: &[&[u8]]) -> Box<[u8]> {
    let mut size = lines.len().saturating_sub(1);
    for line in lines {
        size = size.checked_add(line.trim_ascii_end().len()).expect("joined lines fit the input's length");
    }
    let mut out = Writer::new(size);
    for (index, line) in lines.iter().enumerate() {
        if index != 0 {
            out.put(b"\n").expect("separators were counted");
        }
        out.put(line.trim_ascii_end()).expect("line lengths were counted");
    }
    out.finish()
}

/// A person task a goal asks in its words: addressed to a person, a report its contract
/// (temper/docs/design/client/README.md, 3.5; temper/docs/design/domain/agent.md, 8.2).
#[must_use]
pub fn in_words(task: &jig_client::contract::PersonTask) -> bool {
    use jig_client::contract::{PersonAddress, TaskContractDocument};
    match task.address {
        PersonAddress::Party(_) => match task.contract {
            TaskContractDocument::Report { .. } => true,
            TaskContractDocument::Approval { .. }
            | TaskContractDocument::Verdict { .. }
            | TaskContractDocument::Change { .. }
            | TaskContractDocument::Job { .. } => false,
        },
        PersonAddress::Role(_) => false,
    }
}
