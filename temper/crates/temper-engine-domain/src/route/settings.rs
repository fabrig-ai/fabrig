//! Volatile bounded proposal transport; no fragment applies settings
//! (temper/docs/design/domain/agent.md, 8.1).
use super::{CallKey, Domain, ReplyTo, Work, host_route};
use alloc::boxed::Box;
use jig_domain_core::{self as core, settings as s};
use skein_lib::{Decimal, Writer};

const BYTES: usize = 65_536; // The shared host-input and settings JSON decoder ceiling.

/// One task attempt's unvalidated proposal text at its base revision.
#[derive(Debug)]
pub(crate) struct Assembly {
    task: u64,
    attempt: u64,
    against: u32,
    pub(crate) bytes: Box<[u8]>,
}

/// Discard unfinished text when its base, proof or root lifecycle no longer holds.
pub(crate) fn sweep(domain: &mut Domain) {
    if let Some(value) = &domain.settings_assembly
        && (domain.settling.active()
            || value.against != domain.core.current.newest()
            || !super::current_proof(domain, value.task, value.attempt))
    {
        domain.settings_assembly = None;
    }
}

fn answer(domain: &mut Domain, to: ReplyTo, error: bool, body: &[u8]) {
    host_route::rendered(domain, to, core::SettledAnswer::Host { error, body: body.into() });
}

/// Retain a contiguous fragment or finalize its exact acknowledged byte count.
pub(super) fn fragment(
    domain: &mut Domain,
    to: ReplyTo,
    key: CallKey,
    against: u32,
    offset: u32,
    text: Option<Box<[u8]>>,
) {
    sweep(domain);
    if against != domain.core.current.newest() {
        answer(domain, to, true, b"assembly stale; read settings again");
        return;
    }
    if let Some(value) = &domain.settings_assembly
        && (value.task != key.task || value.attempt != key.attempt || value.against != against)
    {
        answer(domain, to, true, b"settings assembly busy; retry after its task finishes");
        return;
    }
    match text {
        Some(text) => append(domain, to, key, against, offset, &text),
        None => finish(domain, to, key, against, offset),
    }
}

fn append(domain: &mut Domain, to: ReplyTo, key: CallKey, against: u32, offset: u32, text: &[u8]) {
    let offset = skein_lib::usize_of(offset);
    let previous = match &domain.settings_assembly {
        Some(value) => value.bytes.as_ref(),
        None => b"".as_slice(),
    };
    let end = match offset.checked_add(text.len()) {
        Some(end) if end <= BYTES => end,
        Some(_) | None => {
            answer(domain, to, true, b"settings assembly exceeds 65536 bytes");
            return;
        }
    };
    // Identical retransmission is harmless; offset zero with different text starts afresh.
    let repeated = !text.is_empty() && previous.get(offset..end) == Some(text);
    if offset != 0 && offset != previous.len() && !repeated {
        answer(domain, to, true, b"settings fragment offset differs; append at the acknowledged byte count");
        return;
    }
    let length = if repeated { previous.len() } else { end };
    if length > previous.len()
        && domain.held_bytes().saturating_add(u64::try_from(length).expect("bounded assembly"))
            > domain.limits.memory_bytes
    {
        answer(domain, to, true, b"settings assembly memory busy");
        return;
    }
    if !repeated {
        let mut bytes = Writer::new(length);
        if offset != 0 {
            bytes.put(previous).expect("contiguous admitted fragment");
        }
        bytes.put(text).expect("bounded fragment");
        domain.settings_assembly = if length == 0 {
            None
        } else {
            Some(Assembly { task: key.task, attempt: key.attempt, against, bytes: bytes.finish() })
        };
    }
    let header = b"settings fragment retained; next offset ";
    let offset = Decimal::of(u64::try_from(length).expect("bounded length"));
    let mut words = Writer::new(header.len().checked_add(offset.as_bytes().len()).expect("bounded answer"));
    words.put(header).expect("fixed answer");
    words.put(offset.as_bytes()).expect("bounded offset");
    answer(domain, to, false, &words.finish());
}

fn finish(domain: &mut Domain, to: ReplyTo, key: CallKey, against: u32, length: u32) {
    let Some(value) = &domain.settings_assembly else {
        answer(domain, to, true, b"no settings assembly; start at offset 0");
        return;
    };
    if value.bytes.len() != skein_lib::usize_of(length) {
        answer(domain, to, true, b"settings assembly length differs; finish at the acknowledged byte count");
        return;
    }
    let value = domain.settings_assembly.take().expect("checked assembly");
    host_route::decode_settings(domain, to, key, against, value.bytes);
}

/// Route whole validated values through the existing owner proposal decision.
pub(super) fn propose(
    domain: &mut Domain,
    to: ReplyTo,
    key: CallKey,
    against: u32,
    items: Box<[s::Item]>,
    reason: Box<[u8]>,
    kinds: Box<[crate::KindValue]>,
) {
    if let Some(jig_domain_people::Refusal::Judged { why, .. }) = super::room_refusal(domain.host_part, &items) {
        answer(domain, to, true, &why);
        return;
    }
    domain.kinds = kinds;
    domain.work.push(Work::Core(core::Event::NamedAction {
        to,
        key,
        action: core::NamedAction::ProposeSettings { against, items, reason },
    }));
}
