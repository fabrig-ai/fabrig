//! Project charter files (temper/docs/design/protocol/temper-dir.md, 3.3, 4 and 7).
use crate::format::{TOML_LIMITS, opening, source_line, split_front};
use crate::{BLANK_LINE, CHARTER_BYTES, PART_HEADING, ReasonFault, checked_reason, heading, section_text, trim_text};
use alloc::boxed::Box;
#[expect(clippy::disallowed_types, reason = "the charter writer's return type is String")]
use alloc::string::String;
use core::fmt;
use skein_lib::{List, usize_of};
use skein_toml::{Document, Entry, Limit, ValueKind, Why};

#[derive(Debug, PartialEq, Eq)]
pub struct CharterFile {
    pub format: u32,
    pub settings: Option<Box<str>>,
    pub instructions: Box<str>,
    pub parts: Box<[FilePart]>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FilePart {
    pub line: u32,
    pub reasons: Box<[Box<str>]>,
    pub text: Box<str>,
}

/// The caller supplies jig's reason names in jig's order.
#[derive(Debug)]
pub struct WritePart<'a> {
    pub reasons: &'a [&'a str],
    pub text: &'a str,
}

/// A fault's line is one-based, or zero for the whole file. For the writer,
/// it locates a line in the instruction or part text, before trimming.
#[derive(Debug, PartialEq, Eq)]
#[expect(clippy::partial_pub_fields, reason = "callers use the line and render the private fault with Display")]
pub struct CharterFault {
    pub line: u32,
    kind: Fault,
}

#[derive(Debug, PartialEq, Eq)]
enum Fault {
    TooLong,
    Unclosed,
    Toml(Why),
    BadFormat,
    NewerFormat(i64),
    UnknownKey(Box<str>),
    SettingsType,
    SettingsName(Box<str>),
    MissingReason(Box<str>),
    EmptyReason(Box<str>),
    RepeatedReason { heading: Box<str>, name: Box<str> },
    UnknownReason { heading: Box<str>, name: Box<str> },
    InstructionHeading,
    PartHeading { part: usize },
}

#[expect(
    clippy::disallowed_macros,
    reason = "the charter fault renders its words, leaving the file path to the caller"
)]
impl fmt::Display for CharterFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            Fault::TooLong => f.write_str("past what temper reads, 65,536 bytes"),
            Fault::Unclosed => f.write_str("front matter: no closing `+++`"),
            Fault::Toml(why) => f.write_str(toml_words(*why)),
            Fault::BadFormat => f.write_str("format: a whole number from 1"),
            Fault::NewerFormat(n) => {
                write!(f, "format {n}; this temper-engine reads format 1: upgrade it, or write the file at format 1")
            }
            Fault::UnknownKey(key) => write!(
                f,
                "unknown key `{key}`: a charter's file holds words alone; models, tools, wakes and budgets are settings"
            ),
            Fault::SettingsType => f.write_str("`settings`: a charter name"),
            Fault::SettingsName(name) => {
                write!(f, "`{name}` is no name: 1 to 32 lowercase letters, digits and `-`, from a letter")
            }
            Fault::MissingReason(heading) => write!(f, "`{heading}`: a part needs at least one reason"),
            Fault::EmptyReason(heading) => write!(f, "`{heading}`: a reason is empty"),
            Fault::RepeatedReason { heading, name } => write!(f, "`{heading}`: `{name}` is repeated"),
            Fault::UnknownReason { heading, name } => {
                write!(f, "`{heading}`: `{name}` is no reason a part is selected by")
            }
            Fault::InstructionHeading => write!(f, "instructions: a line begins `{PART_HEADING}`"),
            Fault::PartHeading { part } => write!(f, "part {part}: a line begins `{PART_HEADING}`"),
        }
    }
}

fn toml_words(why: Why) -> &'static str {
    match why {
        Why::InvalidUtf8 => "not UTF-8",
        Why::Grammar => "does not parse as TOML",
        Why::KeyRedefined => "a TOML key is defined twice",
        Why::TableRedefined => "a TOML table is defined twice",
        Why::Range => "a TOML value is outside its range",
        Why::Limit(Limit::InputBytes) => "past what temper reads, 65,536 bytes",
        Why::Limit(Limit::Depth) => "past what temper reads: TOML nesting depth",
        Why::Limit(Limit::Entries) => "past what temper reads: TOML entries",
    }
}

/// Read one project's charter file. `reasons` lists the names a part can name.
/// # Errors
/// Returns the first fault in the front matter or body; judges the format before the other keys.
pub fn read_charter(text: &str, reasons: &[&str]) -> Result<CharterFile, CharterFault> {
    if text.len() > usize_of(CHARTER_BYTES) {
        return Err(CharterFault { line: 0, kind: Fault::TooLong });
    }
    let mut charter = CharterFile { format: 1, settings: None, instructions: "".into(), parts: Box::new([]) };
    let (body, first_line) = if let Some(rest) = opening(text) {
        let (front, body, first_line) = split_front(rest).ok_or(CharterFault { line: 1, kind: Fault::Unclosed })?;
        charter.settings = front_settings(front)?;
        (body, first_line)
    } else {
        (text, 1)
    };
    read_body(&mut charter, body, first_line, reasons)?;
    Ok(charter)
}

fn front_settings(front: &str) -> Result<Option<Box<str>>, CharterFault> {
    let doc = match skein_toml::parse(front.as_bytes(), &TOML_LIMITS) {
        Ok(doc) => doc,
        Err(fault) => return Err(CharterFault { line: source_line(fault.line), kind: Fault::Toml(fault.why) }),
    };
    let root = doc.root();
    if let Some(entry) = doc.get(root, b"format") {
        check_format(&doc, entry)?;
    }
    let mut settings = None;
    for index in 0..doc.table_len(root).expect("the root is a table") {
        let entry = doc.entry(root, index).expect("the index is in the root table");
        let key = toml_text(&doc, entry.key);
        match key {
            "format" => {}
            "settings" => settings = Some(read_settings(&doc, entry)?),
            _ => return Err(CharterFault { line: source_line(entry.line), kind: Fault::UnknownKey(key.into()) }),
        }
    }
    Ok(settings)
}

fn check_format(doc: &Document, entry: Entry) -> Result<(), CharterFault> {
    let kind = match entry.value.kind {
        ValueKind::Integer(n) if n > 0 && doc.integer_radix(doc.root(), b"format") == Some(10) => {
            if n == 1 {
                return Ok(());
            }
            Fault::NewerFormat(n)
        }
        ValueKind::Integer(_)
        | ValueKind::String(_)
        | ValueKind::Float(_)
        | ValueKind::Boolean(_)
        | ValueKind::DateTime { .. }
        | ValueKind::Array(_)
        | ValueKind::Table(_)
        | ValueKind::ArrayOfTables(_) => Fault::BadFormat,
    };
    Err(CharterFault { line: source_line(entry.line), kind })
}

fn read_settings(doc: &Document, entry: Entry) -> Result<Box<str>, CharterFault> {
    let line = source_line(entry.line);
    let name = match entry.value.kind {
        ValueKind::String(text) => toml_text(doc, text),
        ValueKind::Integer(_)
        | ValueKind::Float(_)
        | ValueKind::Boolean(_)
        | ValueKind::DateTime { .. }
        | ValueKind::Array(_)
        | ValueKind::Table(_)
        | ValueKind::ArrayOfTables(_) => return Err(CharterFault { line, kind: Fault::SettingsType }),
    };
    if !is_name(name.as_bytes()) {
        return Err(CharterFault { line, kind: Fault::SettingsName(name.into()) });
    }
    Ok(name.into())
}

#[expect(clippy::disallowed_methods, reason = "skein-toml has validated and decoded its text as UTF-8")]
fn toml_text(doc: &Document, text: skein_toml::Text) -> &str {
    core::str::from_utf8(doc.text(text).expect("the text belongs to its document")).expect("TOML text is UTF-8")
}

fn is_name(bytes: &[u8]) -> bool {
    let Some((first, rest)) = bytes.split_first() else { return false };
    if !first.is_ascii_lowercase() || bytes.len() > 32 {
        return false;
    }
    for byte in rest {
        if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && *byte != b'-' {
            return false;
        }
    }
    true
}

fn read_body(charter: &mut CharterFile, text: &str, first_line: usize, reasons: &[&str]) -> Result<(), CharterFault> {
    let mut parts = List::uncapped();
    let mut current: Option<FilePart> = None;
    let mut start = 0_usize;
    let mut offset = 0_usize;
    for (index, line) in text.split_inclusive('\n').enumerate() {
        let end = offset.checked_add(line.len()).expect("line within text");
        if let Some(names) = heading(line) {
            let body = section_text(text, start, offset);
            if let Some(mut part) = current.take() {
                part.text = body.into();
                parts.push(part).expect("part count fits u32");
            } else {
                charter.instructions = body.into();
            }
            let at = index.checked_add(first_line).expect("line within file");
            let at = u32::try_from(at).expect("a charter file is bounded to 65,536 bytes");
            current = Some(read_part(at, line.trim_end_matches('\n'), names, reasons)?);
            start = end;
        }
        offset = end;
    }
    let body = section_text(text, start, text.len());
    if let Some(mut part) = current {
        part.text = body.into();
        parts.push(part).expect("part count fits u32");
    } else {
        charter.instructions = body.into();
    }
    charter.parts = parts.into_boxed();
    Ok(())
}

fn read_part(line: u32, heading: &str, names: &str, reasons: &[&str]) -> Result<FilePart, CharterFault> {
    if names.trim_matches(' ').is_empty() {
        return Err(CharterFault { line, kind: Fault::MissingReason(heading.into()) });
    }
    let mut seen = List::uncapped();
    let mut owned = List::uncapped();
    for name in names.split(',') {
        let name = match checked_reason(name, seen.as_slice()) {
            Ok(name) => name,
            Err(ReasonFault::Empty) => return Err(CharterFault { line, kind: Fault::EmptyReason(heading.into()) }),
            Err(ReasonFault::Repeated(name)) => {
                return Err(CharterFault { line, kind: Fault::RepeatedReason { heading: heading.into(), name } });
            }
        };
        if !reasons.contains(&name) {
            return Err(CharterFault {
                line,
                kind: Fault::UnknownReason { heading: heading.into(), name: name.into() },
            });
        }
        seen.push(name).expect("reason count fits u32");
        owned.push(Box::<str>::from(name)).expect("reason count fits u32");
    }
    Ok(FilePart { line, reasons: owned.into_boxed(), text: "".into() })
}

/// Write a project's charter without front matter. Each part supplies its
/// nonempty list of distinct reason names in jig's order.
/// # Errors
/// Refuses a part heading inside instructions or part text, or a file over 65,536 bytes.
#[expect(clippy::disallowed_types, reason = "the charter file writer returns String at its text boundary")]
pub fn write_charter(instructions: &str, parts: &[WritePart<'_>]) -> Result<String, CharterFault> {
    let instructions = writable_text(instructions, None)?;
    let mut out = String::new();
    // A leading blank line keeps a literal delimiter in the instruction body.
    if opening(instructions).is_some() || instructions == "+++\r" {
        put(&mut out, "\n")?;
    }
    put(&mut out, instructions)?;
    for (index, part) in parts.iter().enumerate() {
        let text = writable_text(part.text, Some(index.checked_add(1).expect("part within slice")))?;
        if !out.is_empty() {
            put(&mut out, BLANK_LINE)?;
        }
        put(&mut out, PART_HEADING)?;
        put(&mut out, " ")?;
        for (index, name) in part.reasons.iter().enumerate() {
            if index != 0 {
                put(&mut out, ", ")?;
            }
            put(&mut out, name)?;
        }
        if !text.is_empty() {
            put(&mut out, "\n")?;
            put(&mut out, text)?;
        }
    }
    Ok(out)
}

#[expect(clippy::disallowed_types, reason = "the charter writer grows its returned String only within the file bound")]
fn put(out: &mut String, text: &str) -> Result<(), CharterFault> {
    if text.len() > usize_of(CHARTER_BYTES).checked_sub(out.len()).expect("output is already within the file bound") {
        return Err(CharterFault { line: 0, kind: Fault::TooLong });
    }
    out.push_str(text);
    Ok(())
}

fn writable_text(text: &str, part: Option<usize>) -> Result<&str, CharterFault> {
    for (index, line) in text.split_inclusive('\n').enumerate() {
        if heading(line).is_some() {
            let line = index.checked_add(1).expect("line within text");
            let Ok(line) = u32::try_from(line) else {
                return Err(CharterFault { line: 0, kind: Fault::TooLong });
            };
            let kind = match part {
                Some(part) => Fault::PartHeading { part },
                None => Fault::InstructionHeading,
            };
            return Err(CharterFault { line, kind });
        }
    }
    Ok(trim_text(text))
}
