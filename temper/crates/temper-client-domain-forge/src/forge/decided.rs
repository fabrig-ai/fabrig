//! A project's decisions (temper/docs/design/client/forge/records.md).
use temper_client_protocol_wire::v1::Record;
use super::{Limits, wire_limits};
use alloc::boxed::Box;
use jig_client::{Ask, Family, Watchable, contract as c};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[must_use]
pub fn watch(project: u32, limits: &Limits) -> Option<Watchable> {
    let bounds = wire_limits(limits)?;
    let subject = w::DecidedSubject { project };
    let watching = w::Watching { watch: w::Watch::Decided(subject) };
    Some(Watchable::Family {
        scope: c::Scope::Project(project),
        window: c::Window { rows: 0, after: None },
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: watching.encode(&bounds).ok()? },
    })
}

#[must_use]
pub fn document(record: &Family, limits: Limits) -> Option<w::Decisions> {
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    wire::decode_decisions(&record.bytes, &wire_limits(&limits)?)
}

#[must_use]
pub fn overturn(project: u32, number: u32, limits: &Limits) -> Option<Ask> {
    let bounds = wire_limits(limits)?;
    let overturn = w::Overturn { project, number };
    let record = w::Requesting { request: w::Request::Overturn(overturn) };
    Some(Ask::Family {
        scope: c::Scope::Project(project),
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: record.encode(&bounds).ok()? },
    })
}

#[must_use]
pub fn overturn_words(question: &[u8], took: &[u8]) -> Box<[u8]> {
    // The smallest deployed composer admits 512 bytes (client/view.md, 4).
    // Keep the choice and instruction whole, cutting the question at a character boundary.
    let lead = b"I overturn your decision. ";
    let between = b" You took ";
    let end = b". Ask me again with message, as a question, and wait for my answer.";
    let fixed = lead
        .len()
        .checked_add(between.len())
        .expect("fixed words fit")
        .checked_add(took.len())
        .expect("bounded choice fits")
        .checked_add(end.len())
        .expect("instruction fits");
    let question = skein_lib::bytes::utf8_prefix(question, 512_usize.saturating_sub(fixed));
    skein_lib::bytes::concat(&[lead, question, between, took, end])
}

#[must_use]
pub fn overturned(outcome: &jig_client::Outcome, limits: Limits) -> bool {
    let record = match outcome {
        jig_client::Outcome::Answered(jig_client::Answered::Family(record)) => record,
        jig_client::Outcome::Answered(jig_client::Answered::Jig(_))
        | jig_client::Outcome::Unknown
        | jig_client::Outcome::NotKept
        | jig_client::Outcome::Limit
        | jig_client::Outcome::Unconfirmed
        | jig_client::Outcome::Held
        | jig_client::Outcome::Unsent
        | jig_client::Outcome::Full { .. }
        | jig_client::Outcome::Lost => return false,
    };
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return false;
    }
    let Some(bounds) = wire_limits(&limits) else { return false };
    let mut reader = skein_lib::Reader::new(&record.bytes);
    let Ok(answer) = w::Answer::decode(&bounds, &mut reader) else { return false };
    reader.is_empty() && answer.outcome == w::Outcome::Overturned
}
