//! Terminal result words; record details remain in the JSON projection.
use super::Text;
use crate::jig::render::Parts;
use jig_client::contract as c;

pub(super) fn text(result: &c::Result, vocabulary: &dyn Parts, out: &mut Text) {
    out.put(b"result: ");
    match &result.ending {
        c::Ending::Done => done(result.value.as_ref(), vocabulary, out),
        c::Ending::Task(ending) => match ending.as_ref() {
            c::OwnerChipEnding::Done(value) => done(Some(value), vocabulary, out),
            c::OwnerChipEnding::Failed(reason) => said(b"Failed", Some(reason.bytes()), out),
            c::OwnerChipEnding::Cancelled { reason, .. } => said(b"Cancelled", Some(reason.bytes()), out),
        },
        c::Ending::Failed { reason } => said(b"Failed", reason_bytes(reason.as_ref()), out),
        c::Ending::Cancelled { reason, .. } => said(b"Cancelled", reason_bytes(reason.as_ref()), out),
        c::Ending::Withdrawn { reason } => said(b"Cancelled", Some(reason.bytes()), out),
        c::Ending::Deleted => ended(result.value.as_ref(), vocabulary, out),
        c::Ending::Decided { how, .. } => {
            out.put(b"Ended");
            if let Some(value) = &result.value {
                value_words(value, vocabulary, out);
            } else if let Some(how) = how {
                let mut measure = Text::measure();
                decision(how, &mut measure);
                let mut words = Text::writing_exact(measure);
                decision(how, &mut words);
                out.result_words(&words.finish());
            }
        }
    }
    out.put(b"\n");
}
fn said(state: &[u8], words: Option<&[u8]>, out: &mut Text) {
    out.put(state);
    if let Some(words) = words {
        out.result_words(words);
    }
}
fn done(value: Option<&c::ResultValue>, vocabulary: &dyn Parts, out: &mut Text) {
    if let Some(c::ResultValue::Change { kind, part, words }) = value {
        vocabulary.change(*kind, part, words, out);
    } else {
        out.put(b"Completed");
        if let Some(value) = value {
            value_words(value, vocabulary, out);
        }
    }
}
fn ended(value: Option<&c::ResultValue>, vocabulary: &dyn Parts, out: &mut Text) {
    out.put(b"Ended");
    if let Some(value) = value {
        value_words(value, vocabulary, out);
    }
}
fn value_words(value: &c::ResultValue, vocabulary: &dyn Parts, out: &mut Text) {
    let mut measure = Text::measure();
    value_text(value, vocabulary, &mut measure);
    let mut words = Text::writing_exact(measure);
    value_text(value, vocabulary, &mut words);
    out.result_words(&words.finish());
}
fn value_text(value: &c::ResultValue, vocabulary: &dyn Parts, out: &mut Text) {
    match value {
        c::ResultValue::Report(words)
        | c::ResultValue::OwnerVerdict { words, .. }
        | c::ResultValue::Verdict { words, .. }
        | c::ResultValue::Change { words, .. }
        | c::ResultValue::Failure(words)
        | c::ResultValue::Words(words) => out.put(words.bytes()),
        c::ResultValue::Approved { state, words } => {
            out.put(state);
            if let Some(words) = words {
                separated(words.bytes(), !state.is_empty(), out);
            }
        }
        c::ResultValue::Choice { option, reason, .. } => {
            out.put(b"option ");
            out.number(u64::from(*option));
            separated(reason.bytes(), true, out);
        }
        c::ResultValue::Connector { summary, part } => {
            out.put(summary.bytes());
            if !summary.bytes().is_empty() {
                out.put(b" \xc2\xb7 ");
            }
            vocabulary.part(part, out);
        }
    }
}
fn separated(words: &[u8], prior: bool, out: &mut Text) {
    if !words.is_empty() {
        if prior {
            out.put(b" \xc2\xb7 ");
        }
        out.put(words);
    }
}
fn decision(how: &c::Decision, out: &mut Text) {
    match how {
        c::Decision::Accepted => out.put(b"accepted"),
        c::Decision::Rejected { reason } => {
            out.put(b"rejected");
            separated(reason.bytes(), true, out);
        }
        c::Decision::PassedUp => out.put(b"passed up"),
        c::Decision::Released { again } => {
            out.put(b"released");
            match again {
                Some(true) => out.put(b" \xc2\xb7 try again"),
                Some(false) => out.put(b" \xc2\xb7 do not try again"),
                None => {}
            }
        }
        c::Decision::LeftHeld { reason } => {
            out.put(b"left held");
            separated(reason.bytes(), true, out);
        }
    }
}

fn reason_bytes(reason: Option<&c::Text>) -> Option<&[u8]> {
    match reason {
        Some(reason) => Some(reason.bytes()),
        None => None,
    }
}
