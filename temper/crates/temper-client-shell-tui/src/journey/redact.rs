//! TUI originals and fields (temper/docs/design/client/tui/journeys.md, 5).
use super::{Framing, Kind, Line, Originals, Sent, Streamed, Unreadable};
use jig_client_protocol_wire as w;
use skein_lib::Reader;
use skein_ui_screen_terminal::Key;
use temper_client_shell_web::journey::redact_field;

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
        Line::Address(bytes) | Line::Signed(bytes) | Line::Browse(bytes) => vec![(bytes, Text)],
        Line::Mark(print) | Line::End(print) => print.iter_mut().map(|bytes| (bytes, Text)).collect(),
        Line::Journey { .. }
        | Line::Seed(_)
        | Line::Clocks { .. }
        | Line::Screen { .. }
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
            Line::Journey { .. }
            | Line::Seed(_)
            | Line::Clocks { .. }
            | Line::Screen { .. }
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

/// Replace every original's forms and digests in every field, then write the lines.
pub fn redact(recording: &[u8], given: &[Box<[u8]>]) -> Result<Redacted, Unreadable> {
    let mut lines = super::lines(recording)?;
    let originals = originals(&lines, given);
    let mut places = 0_u32;
    let mut recording = Vec::new();
    for line in &mut lines {
        for (field, framing) in fields_mut(line) {
            places = places.checked_add(redact_field(field, framing, &originals)).expect("replacement count fits u32");
        }
        line.write(&mut recording);
    }
    Ok(Redacted { recording: recording.into(), originals, places })
}

#[cfg(test)]
pub(super) mod tests;
