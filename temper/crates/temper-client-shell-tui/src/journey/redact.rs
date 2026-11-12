//! TUI originals and fields (temper/docs/design/client/tui/journeys.md, 5).
use super::{Framing, Kind, Line, Originals, Sent, Streamed, Unreadable};
use jig_client_protocol_wire as w;
use skein_lib::{
    Reader, Writer,
    base64::{self, Alphabet},
    bytes::find_from,
};
use skein_ui_screen_terminal::Key;
use std::collections::BTreeMap;
use temper_client_redaction::redact_field;
use temper_client_tui::BoxLine;

pub struct Redacted {
    pub recording: Box<[u8]>,
    pub originals: Originals,
    pub places: u32,
}

/// Every byte field, in record order, with the framing that counts its bytes.
#[must_use]
pub fn fields_mut(line: &mut Line) -> Vec<(&mut Box<[u8]>, Framing)> {
    use Framing::{Framed, Text};
    match line {
        Line::Reply { bytes, .. } | Line::Credential(bytes) | Line::Erase(bytes) => vec![(bytes, Framed)],
        Line::Stored { key, bytes } | Line::Save { key, bytes } => vec![(key, Framed), (bytes, Framed)],
        Line::Streamed(streamed) => match streamed {
            Streamed::Record(bytes) => vec![(bytes, Framed)],
            Streamed::Heartbeat | Streamed::Full { .. } | Streamed::SignedOut => Vec::new(),
        },
        Line::Send { message, credential, .. } => {
            let mut fields = match message {
                Sent::Authorize(bytes)
                | Sent::ApprovalAsk(bytes)
                | Sent::ApprovalRead(bytes)
                | Sent::ApprovalDecide(bytes)
                | Sent::SignInApproval(bytes)
                | Sent::Request(bytes)
                | Sent::Read(bytes)
                | Sent::Watch(bytes)
                | Sent::Unwatch(bytes)
                | Sent::StreamGone(bytes)
                | Sent::SignInLink(bytes) => vec![(bytes, Framed)],
                Sent::SignInService | Sent::SignOut => Vec::new(),
            };
            fields.extend(credential.iter_mut().map(|bytes| (bytes, Framed)));
            fields
        }
        Line::Stream(credential) | Line::Keep(credential) => {
            credential.iter_mut().map(|bytes| (bytes, Framed)).collect()
        }
        Line::Key { key, event } => {
            let mut fields = match key {
                Key::Paste(bytes) => vec![(bytes, Text)],
                Key::Char(_)
                | Key::Enter
                | Key::Tab
                | Key::BackTab
                | Key::Backspace
                | Key::Delete
                | Key::Up
                | Key::Down
                | Key::Left
                | Key::Right
                | Key::Home
                | Key::End
                | Key::PageUp
                | Key::PageDown
                | Key::Escape
                | Key::Control(_)
                | Key::Alt(_) => Vec::new(),
            };
            fields.extend(event.iter_mut().map(|bytes| (bytes, Text)));
            fields
        }
        Line::Found(bytes) => bytes.iter_mut().map(|bytes| (bytes, Text)).collect(),
        Line::BoxLine(bytes) | Line::Address(bytes) | Line::Signed(bytes) | Line::Browse(bytes) => vec![(bytes, Text)],
        Line::Mark(print) | Line::End(print) => print.iter_mut().map(|bytes| (bytes, Text)).collect(),
        Line::Journey { .. }
        | Line::Front
        | Line::Local
        | Line::Noted(_)
        | Line::Note(_)
        | Line::Read
        | Line::Seed(_)
        | Line::Clocks { .. }
        | Line::Screen { .. }
        | Line::Background(_)
        | Line::Box
        | Line::Turn { .. }
        | Line::Start
        | Line::Due
        | Line::Resize(_)
        | Line::Unanswered(_)
        | Line::Gone
        | Line::Saved
        | Line::Unsaved
        | Line::Kept { .. }
        | Line::Opened { .. }
        | Line::Unstream
        | Line::Quit
        | Line::Deadline(_) => Vec::new(),
    }
}

/// Read sign-in records through the wire, leaving refused bodies to the last scan.
#[must_use]
pub fn originals(lines: &[Line], given: &[Box<[u8]>]) -> Originals {
    let mut candidates = Vec::new();
    for line in lines {
        match line {
            Line::Credential(bytes) | Line::Keep(Some(bytes)) => candidates.push((Kind::Credential, bytes.clone())),
            Line::Send { message, credential, .. } => {
                match message {
                    Sent::SignInApproval(bytes) => {
                        if let Ok(value) = w::ApprovalSignIn::decode(&w::CEILINGS, &mut Reader::new(bytes)) {
                            candidates.push((Kind::Approval, value.into_parts().secret.into_exposed()));
                        }
                    }
                    Sent::SignInLink(bytes) => {
                        if let Ok(value) = w::LinkSignIn::decode(&w::CEILINGS, &mut Reader::new(bytes)) {
                            candidates.push((Kind::Link, value.into_parts().secret.into_exposed()));
                        }
                    }
                    Sent::Authorize(_)
                    | Sent::ApprovalAsk(_)
                    | Sent::ApprovalRead(_)
                    | Sent::ApprovalDecide(_)
                    | Sent::Request(_)
                    | Sent::Read(_)
                    | Sent::Watch(_)
                    | Sent::Unwatch(_)
                    | Sent::StreamGone(_)
                    | Sent::SignInService
                    | Sent::SignOut => {}
                }
                if let Some(bytes) = credential {
                    candidates.push((Kind::Bearer, bytes.clone()));
                }
            }
            Line::Stream(Some(bytes)) => candidates.push((Kind::Bearer, bytes.clone())),
            Line::Found(Some(bytes)) => {
                candidates.extend(link_original(bytes));
            }
            Line::BoxLine(bytes) => {
                if let Some(BoxLine::Link { url }) = BoxLine::read(bytes) {
                    candidates.extend(link_original(url.expose()));
                }
            }
            Line::Found(None)
            | Line::Journey { .. }
            | Line::Front
            | Line::Local
            | Line::Noted(_)
            | Line::Note(_)
            | Line::Read
            | Line::Seed(_)
            | Line::Clocks { .. }
            | Line::Screen { .. }
            | Line::Background(_)
            | Line::Box
            | Line::Address(_)
            | Line::Stored { .. }
            | Line::Turn { .. }
            | Line::Start
            | Line::Due
            | Line::Key { .. }
            | Line::Resize(_)
            | Line::Reply { .. }
            | Line::Unanswered(_)
            | Line::Streamed(_)
            | Line::Gone
            | Line::Saved
            | Line::Unsaved
            | Line::Kept { .. }
            | Line::Opened { .. }
            | Line::Stream(None)
            | Line::Unstream
            | Line::Save { .. }
            | Line::Erase(_)
            | Line::Keep(None)
            | Line::Signed(_)
            | Line::Browse(_)
            | Line::Quit
            | Line::Deadline(_)
            | Line::Mark(_)
            | Line::End(_) => {}
        }
    }
    Originals::new(candidates, given)
}

fn link_original(url: &[u8]) -> Option<(Kind, Box<[u8]>)> {
    let mapping = jig_client_protocol_http::Mapping::new(b"/jig").expect("mapping prefix");
    jig_client_protocol_http::printed(&mapping, url).ok().map(|link| (Kind::Link, link.secret.into_exposed()))
}

/// Replace every original's forms and digests in every field, then write the lines.
pub fn redact(recording: &[u8], given: &[Box<[u8]>]) -> Result<Redacted, Unreadable> {
    let mut lines = super::lines(recording)?;
    let originals = originals(&lines, given);
    let forms = link_forms(&originals);
    let mut places = 0_u32;
    let mut recording = Vec::new();
    for line in &mut lines {
        for (field, framing) in fields_mut(line) {
            places = places.checked_add(redact_links(field, &forms)).expect("replacement count fits u32");
            places = places.checked_add(redact_field(field, framing, &originals)).expect("replacement count fits u32");
        }
        line.write(&mut recording);
    }
    Ok(Redacted { recording: recording.into(), originals, places })
}

/// A link's canonical URL-base64 and its Typed event's decimal byte elements.
pub(super) struct LinkForm {
    bytes: Box<[u8]>,
    hidden: Box<[u8]>,
    decimal: bool,
}

pub(super) fn link_forms(originals: &Originals) -> Vec<LinkForm> {
    let mut forms = Vec::new();
    for original in originals.sought() {
        if original.kind() == Kind::Link {
            let bytes = url_base64(original.forms().first().expect("an original has its bytes"));
            let hidden = url_base64(&vec![b'x'; original.len()]);
            forms.push(LinkForm { bytes: decimal(&bytes), hidden: decimal(&hidden), decimal: true });
            forms.push(LinkForm { bytes, hidden, decimal: false });
        }
    }
    forms
}

fn url_base64(bytes: &[u8]) -> Box<[u8]> {
    let len = base64::encoded_len(Alphabet::Url, u32::try_from(bytes.len()).expect("original length fits u32"))
        .expect("encoded original length fits u32");
    let mut writer = Writer::new(usize::try_from(len).expect("encoded length fits usize"));
    base64::encode(Alphabet::Url, bytes, &mut writer).expect("measured URL-base64");
    writer.finish()
}

fn decimal(bytes: &[u8]) -> Box<[u8]> {
    bytes.iter().map(u8::to_string).collect::<Vec<_>>().join(", ").into_bytes().into()
}

impl LinkForm {
    pub(super) fn occurrences(&self, field: &[u8]) -> Vec<(usize, usize)> {
        let mut found = Vec::new();
        let mut from = 0;
        while let Some(at) = find_from(field, &self.bytes, from) {
            let end = at.checked_add(self.bytes.len()).expect("found form fits field");
            if !self.decimal
                || ((field[..at].ends_with(b"[") || field[..at].ends_with(b", "))
                    && (field[end..].starts_with(b"]") || field[end..].starts_with(b",")))
            {
                found.push((at, end));
            }
            from = at.checked_add(1).expect("found form is nonempty");
        }
        found
    }
}

fn redact_links(field: &mut Box<[u8]>, links: &[LinkForm]) -> u32 {
    let mut replacements = BTreeMap::new();
    for form in links {
        for (at, end) in form.occurrences(field) {
            if replacements.range(..end).next_back().is_none_or(|(_, (prior_end, _))| *prior_end <= at) {
                replacements.insert(at, (end, &form.hidden));
            }
        }
    }
    let places = u32::try_from(replacements.len()).expect("field replacement count fits u32");
    if places != 0 {
        let mut output = Vec::with_capacity(field.len());
        let mut from = 0;
        for (at, (end, hidden)) in replacements {
            output.extend_from_slice(&field[from..at]);
            output.extend_from_slice(hidden);
            from = end;
        }
        output.extend_from_slice(&field[from..]);
        *field = output.into();
    }
    places
}

#[cfg(test)]
pub(super) mod tests;
