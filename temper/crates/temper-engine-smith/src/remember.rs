//! Decode and render memory calls (temper/docs/design/domain/agent.md, 8.4.4).
use crate::{Problem, answers::Text, json::Value};
use temper_engine_domain::{RememberAction, RememberDid, RememberRefusal, Remembered, Tool};

pub(crate) fn decode(value: &Value) -> Result<Tool, Problem> {
    let Value::Object(fields) = value else { return Err(Problem::Type(b"remember")) };
    for (name, _) in fields {
        if name.as_ref() != b"action" && name.as_ref() != b"line" && name.as_ref() != b"text" {
            return Err(Problem::Range(b"remember"));
        }
    }
    let action = match value.required(b"action")?.text(b"action")?.as_ref() {
        b"add" => {
            if value.field(b"line")?.is_some() {
                return Err(Problem::Range(b"action"));
            }
            RememberAction::Add { text: value.required(b"text")?.text(b"text")? }
        }
        b"change" => RememberAction::Change { line: line(value)?, text: value.required(b"text")?.text(b"text")? },
        b"drop" => {
            if value.field(b"text")?.is_some() {
                return Err(Problem::Range(b"action"));
            }
            RememberAction::Drop { line: line(value)? }
        }
        _ => return Err(Problem::Range(b"action")),
    };
    Ok(Tool::Remember { action })
}

fn line(value: &Value) -> Result<u32, Problem> {
    let line = value.required(b"line")?.small(b"line")?;
    if line == 0 {
        return Err(Problem::Range(b"line"));
    }
    Ok(line)
}

pub(crate) fn render(text: &mut Text, remembered: &Remembered) -> Result<bool, Problem> {
    text.add(b"remember: ")?;
    match remembered {
        Remembered::Done { did, line, lines, bound } => {
            text.add(b"line ")?;
            text.number(u64::from(*line))?;
            text.add(b" ")?;
            did_text(text, *did)?;
            text.add(b"; ")?;
            text.number(u64::from(lines.len()))?;
            text.add(b" of ")?;
            text.number(u64::from(*bound))?;
            text.add(b" lines.")?;
            if lines.is_empty() {
                text.add(b"\nNone kept.")?;
            }
            for (at, line) in lines.iter().enumerate() {
                text.add(b"\n")?;
                text.number(skein_lib::len_u64(at).checked_add(1).ok_or(Problem::TooLarge)?)?;
                text.add(b". ")?;
                text.add(line)?;
            }
            Ok(false)
        }
        Remembered::Refused(why) => {
            refusal(text, why)?;
            Ok(true)
        }
    }
}

fn did_text(text: &mut Text, did: RememberDid) -> Result<(), Problem> {
    text.add(match did {
        RememberDid::Kept => b"kept",
        RememberDid::Changed => b"changed",
        RememberDid::Dropped => b"dropped",
    })
}

fn refusal(text: &mut Text, why: &RememberRefusal) -> Result<(), Problem> {
    match why {
        RememberRefusal::NotOneLine => text.add(b"a line is one line of text, not empty; nothing changed."),
        RememberRefusal::TooLong { bytes, bound } => {
            text.add(b"the line is ")?;
            text.number(u64::from(*bytes))?;
            text.add(b" bytes, past its bound of ")?;
            text.number(u64::from(*bound))?;
            text.add(b"; nothing changed.")
        }
        RememberRefusal::Full { bound } => {
            text.add(b"the list is full, ")?;
            text.number(u64::from(*bound))?;
            text.add(b" lines; ask which line to change or drop. Nothing changed.")
        }
        RememberRefusal::NoLine { line, lines } => {
            text.add(b"no line ")?;
            text.number(u64::from(*line))?;
            if *lines == 0 {
                text.add(b"; the list is empty. Nothing changed.")
            } else {
                text.add(b"; the list has ")?;
                text.number(u64::from(*lines))?;
                text.add(b". Nothing changed.")
            }
        }
        RememberRefusal::Moved { action, line } => {
            text.add(b"the list changed while ")?;
            match action {
                RememberDid::Kept => text.add(b"a line")?,
                RememberDid::Changed | RememberDid::Dropped => {
                    text.add(b"line ")?;
                    text.number(u64::from(*line))?;
                }
            }
            text.add(b" was ")?;
            did_text(text, *action)?;
            text.add(b"; nothing changed. Call again.")
        }
        RememberRefusal::NoPerson => text.add(b"this chat keeps nothing for a person; nothing changed."),
        RememberRefusal::Busy => text.add(b"busy; nothing changed. Call again."),
        RememberRefusal::Unread => text.add(b"the list could not be read; nothing changed."),
        RememberRefusal::Note(why) => {
            let (body, _error) =
                crate::answers::answer(&temper_engine_domain::CallAnswer::NoteRefused(*why)).into_parts();
            text.literal(&body)
        }
    }
}
