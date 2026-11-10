//! Decode the chat's document write (temper/docs/design/domain/agent/ask-temper.md, 3).

use crate::{Problem, json::Value};
use temper_engine_domain::{DocumentAction, Tool};

pub(crate) fn decode(value: &Value) -> Result<Tool, Problem> {
    let Value::Object(fields) = value else { return Err(Problem::Type(b"document")) };
    for (name, _) in fields {
        if name.as_ref() != b"action" && name.as_ref() != b"body" {
            return Err(Problem::Range(b"document"));
        }
    }
    let action = match value.required(b"action")?.text(b"action")?.as_ref() {
        b"write" => DocumentAction::Write,
        b"append" => DocumentAction::Append,
        b"replace" => DocumentAction::Replace,
        _ => return Err(Problem::Range(b"action")),
    };
    Ok(Tool::Document { action, body: value.required(b"body")?.text(b"body")? })
}

/// Render the root's joined answer in the document call's words (domain/agent/ask-temper.md, 3).
pub(crate) fn render(
    text: &mut crate::answers::Text,
    documented: &temper_engine_domain::Documented,
) -> Result<bool, Problem> {
    use crate::answers::note_written;
    use temper_engine_domain::DocumentOutcome;
    let action = documented.action;
    let heading = &documented.heading;
    match &documented.outcome {
        DocumentOutcome::Written { name, revision, lines, added, removed, first } => {
            note_written(
                text,
                *name,
                *revision,
                jig_domain_notes::LineChange { lines: *lines, added: *added, removed: *removed, first: *first },
            )?;
            text.add(b"\ndocument: ")?;
            named(text, action, heading)?;
            match action {
                DocumentAction::Write => {
                    if *revision == 1 {
                        text.add(b" written, ")?;
                    } else {
                        text.add(b" written whole again, ")?;
                    }
                }
                DocumentAction::Append => text.add(b" appended, ")?,
                DocumentAction::Replace => text.add(b" replaced, ")?,
            }
            text.number(u64::from(*lines))?;
            text.add(b" lines.")?;
            Ok(false)
        }
        DocumentOutcome::Refused(why) => {
            refusal(text, action, heading, why)?;
            Ok(true)
        }
    }
}

fn refusal(
    text: &mut crate::answers::Text,
    action: DocumentAction,
    heading: &[u8],
    why: &temper_engine_domain::DocumentRefusal,
) -> Result<(), Problem> {
    use crate::answers::authority_findings;
    use temper_engine_domain::{DocumentRefusal, NoteFamily};
    text.add(b"document: ")?;
    match why {
        DocumentRefusal::NoTitle => {
            text.add(b"write takes the whole document, its first line # TITLE; nothing changed.")?;
        }
        DocumentRefusal::TitlePastBound { bytes, bound } => {
            text.add(b"the title is ")?;
            bound_text(text, *bytes, *bound)?;
        }
        DocumentRefusal::NotOneSection => {
            match action {
                DocumentAction::Write => text.add(b"write")?,
                DocumentAction::Append => text.add(b"append")?,
                DocumentAction::Replace => text.add(b"replace")?,
            }
            text.add(b" takes one section, its first line ## HEADING and no other heading; nothing changed.")?;
        }
        DocumentRefusal::NoDocument => {
            text.add(b"no document yet; write it whole first, its first line # TITLE.")?;
        }
        DocumentRefusal::NoSection { headings, more } => {
            text.add(b"no section ## ")?;
            text.add(heading)?;
            if headings.is_empty() && !*more {
                text.add(b"; it has none. Nothing changed.")?;
            } else {
                text.add(b"; its sections: ")?;
                for (at, heading) in headings.iter().enumerate() {
                    if at > 0 {
                        text.literal(" · ".as_bytes())?;
                    }
                    text.add(b"## ")?;
                    text.add(heading)?;
                }
                if *more {
                    if !headings.is_empty() {
                        text.literal(" · ".as_bytes())?;
                    }
                    text.literal("…".as_bytes())?;
                }
                text.add(b". Nothing changed.")?;
            }
        }
        DocumentRefusal::SectionThere => {
            text.add(b"## ")?;
            text.add(heading)?;
            text.add(b" is there already; replace it. Nothing changed.")?;
        }
        DocumentRefusal::PastBound { bytes, bound } => {
            text.add(b"the document would be ")?;
            bound_text(text, *bytes, *bound)?;
        }
        DocumentRefusal::Moved => {
            text.add(b"the document changed while ")?;
            named(text, action, heading)?;
            text.add(b" was written; nothing changed. Call again: it applies to the newest.")?;
        }
        DocumentRefusal::Busy => text.add(b"busy; nothing changed. Call again.")?,
        DocumentRefusal::Unread => text.add(b"the document could not be read; nothing changed.")?,
        DocumentRefusal::Denied { family, answer, findings } => {
            let tool = match family {
                NoteFamily::Note => b"note".as_slice(),
                NoteFamily::Recall => b"recall".as_slice(),
            };
            authority_findings(text, *answer, findings, tool, false)?;
        }
        DocumentRefusal::Note(why) => {
            let (body, _error) =
                crate::answers::answer(&temper_engine_domain::CallAnswer::NoteRefused(*why)).into_parts();
            text.literal(&body)?;
        }
    }
    Ok(())
}

fn named(text: &mut crate::answers::Text, action: DocumentAction, heading: &[u8]) -> Result<(), Problem> {
    match action {
        DocumentAction::Write => {
            text.add(b"\"")?;
            text.add(heading)?;
            text.add(b"\"")
        }
        DocumentAction::Append | DocumentAction::Replace => {
            text.add(b"## ")?;
            text.add(heading)
        }
    }
}

fn bound_text(text: &mut crate::answers::Text, bytes: u32, bound: u32) -> Result<(), Problem> {
    text.number(u64::from(bytes))?;
    text.add(b" bytes, past its bound of ")?;
    text.number(u64::from(bound))?;
    text.add(b"; nothing changed.")
}
