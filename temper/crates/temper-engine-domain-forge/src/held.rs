//! Brief sections retained by the forge connector (jig's domain/connectors.md,
//! section 9). The parent sees a token and rendered size while the connector
//! owns the words. Taking or dropping a token ends that ownership.

use alloc::boxed::Box;
pub use temper_engine_domain_forge_vocabulary::held::*;

use skein_lib::{Decimal, Queue, Token, Writer};

use crate::brief_words::{bounds, checks, room};
use crate::{BriefRead, BriefSource, Domain, Request, brief};

#[derive(PartialEq, Eq, Hash, Debug)]
pub(crate) struct Planned {
    pub(crate) source: BriefSource,
    pub(crate) partner: Option<Token>,
}

#[derive(Hash, Debug)]
pub(crate) struct Pending {
    source: BriefSource,
    partner: Option<Token>,
    unread: bool,
    budget: u32,
    dropped: bool,
}

#[derive(Hash, Debug)]
pub(crate) struct Held {
    source: BriefSource,
    partner: Option<Token>,
    unread: bool,
    pub(crate) words: Box<[u8]>,
    ci: Option<crate::brief_ci::Section>,
    document: Option<crate::brief_document::Section>,
}

#[expect(clippy::too_many_arguments, reason = "one bounded section handoff names its source and read limits")]
pub(crate) fn gather(
    domain: &mut Domain,
    section: Token,
    source: BriefSource,
    partner: Option<Token>,
    parts: u32,
    bytes: u32,
    max_job_bytes: u32,
    limit: u32,
    out: &mut Queue<Request>,
) {
    if domain.brief_pending.contains_key(&section) || domain.brief_held.contains_key(&section) {
        out.push(Request::BriefSized { section, size: Err(bounds()) });
        return;
    }
    if domain.brief_pending.len().saturating_add(domain.brief_held.len()) >= domain.brief_held.capacity() {
        out.push(Request::BriefSized { section, size: Err(room()) });
        return;
    }
    domain
        .brief_pending
        .insert(section, Pending { source, partner, unread: false, budget: bytes, dropped: false })
        .expect("section room checked");
    brief::gather(domain, section, source, parts, bytes, max_job_bytes, limit, out);
}

/// Route a finished forge read to either the token owner or the old caller.
pub(crate) fn ready(domain: &mut Domain, section: Token, read: BriefRead, out: &mut Queue<Request>) {
    let Some(pending) = domain.brief_pending.remove(&section) else {
        out.push(Request::BriefReady { owner: section, read });
        return;
    };
    if pending.dropped {
        return;
    }
    let words = match read {
        BriefRead::Got { bytes, left } => render(pending.source, bytes, left, pending.budget).ok_or(bounds()),
        BriefRead::Failed(unread) => Err(unread),
    };
    match words {
        Ok(words) => {
            let size = u32::try_from(words.len()).expect("bounded rendered section");
            domain
                .brief_held
                .insert(
                    section,
                    Held {
                        source: pending.source,
                        partner: pending.partner,
                        unread: pending.unread,
                        words,
                        ci: None,
                        document: None,
                    },
                )
                .expect("pending reserved held room");
            out.push(Request::BriefSized { section, size: Ok(size) });
        }
        Err(unread) => out.push(Request::BriefSized { section, size: Err(unread) }),
    }
}

/// Retain CI structure so subsequent cuts follow the same whole-head rule.
pub(crate) fn ready_ci(
    domain: &mut Domain,
    section: Token,
    ci: crate::brief_ci::Section,
    budget: u32,
    out: &mut Queue<Request>,
) {
    let Some(pending) = domain.brief_pending.remove(&section) else {
        let read = match ci.render(budget) {
            Some(bytes) => BriefRead::Got { bytes, left: 0 },
            None => BriefRead::Failed(checks()),
        };
        out.push(Request::BriefReady { owner: section, read });
        return;
    };
    if pending.dropped {
        return;
    }
    match ci.render(pending.budget) {
        Some(words) => {
            let size = u32::try_from(words.len()).expect("bounded CI section");
            domain
                .brief_held
                .insert(
                    section,
                    Held {
                        source: pending.source,
                        partner: pending.partner,
                        unread: pending.unread,
                        words,
                        ci: Some(ci),
                        document: None,
                    },
                )
                .expect("pending reserved held room");
            out.push(Request::BriefSized { section, size: Ok(size) });
        }
        None => out.push(Request::BriefSized { section, size: Err(checks()) }),
    }
}

/// Keep the document's immutable words for every later line cut.
pub(crate) fn ready_document(
    domain: &mut Domain,
    section: Token,
    document: crate::brief_document::Section,
    out: &mut Queue<Request>,
) {
    let Some(pending) = domain.brief_pending.remove(&section) else { return };
    if pending.dropped {
        return;
    }
    match document.render(pending.budget) {
        Some(words) => {
            let size = u32::try_from(words.len()).expect("bounded document section");
            domain
                .brief_held
                .insert(
                    section,
                    Held {
                        source: pending.source,
                        partner: pending.partner,
                        unread: pending.unread,
                        words,
                        ci: None,
                        document: Some(document),
                    },
                )
                .expect("pending reserved held room");
            out.push(Request::BriefSized { section, size: Ok(size) });
        }
        None => out.push(Request::BriefSized { section, size: Err(bounds()) }),
    }
}

pub(crate) fn cut(domain: &mut Domain, section: Token, bytes: u32, out: &mut Queue<Request>) {
    let Some(held) = domain.brief_held.remove(&section) else {
        out.push(Request::BriefSized { section, size: Err(bounds()) });
        return;
    };
    if bytes >= u32::try_from(held.words.len()).expect("bounded section") {
        let size = u32::try_from(held.words.len()).expect("bounded section");
        domain.brief_held.insert(section, held).expect("same held room");
        out.push(Request::BriefSized { section, size: Ok(size) });
        return;
    }
    let unread = if held.ci.is_some() { checks() } else { bounds() };
    let rendered = match &held.document {
        Some(document) => document.render(bytes),
        None => match &held.ci {
            Some(ci) => ci.render(bytes),
            None => render(held.source, held.words.clone(), 0, bytes),
        },
    };
    match rendered {
        Some(words) => {
            let size = u32::try_from(words.len()).expect("bounded cut");
            domain
                .brief_held
                .insert(
                    section,
                    Held {
                        source: held.source,
                        partner: held.partner,
                        unread: held.unread,
                        words,
                        ci: held.ci,
                        document: held.document,
                    },
                )
                .expect("same held room");
            out.push(Request::BriefSized { section, size: Ok(size) });
        }
        None => {
            domain.brief_held.insert(section, held).expect("same held room");
            out.push(Request::BriefSized { section, size: Err(unread) });
        }
    }
}

pub(crate) fn take(domain: &mut Domain, section: Token, out: &mut Queue<Request>) {
    let bytes = match domain.brief_held.remove(&section) {
        Some(held) => Some(held.words),
        None => None,
    };
    out.push(Request::BriefTaken { section, bytes });
}

pub(crate) fn drop_section(domain: &mut Domain, section: Token) {
    domain.brief_held.remove(&section);
    if let Some(pending) = domain.brief_pending.get_mut(&section) {
        pending.dropped = true;
    }
}

fn render(_source: BriefSource, text: Box<[u8]>, left: u64, budget: u32) -> Option<Box<[u8]>> {
    // CI is rendered from its retained check/log structure in ready_ci.
    render_edge(text, left, budget, false)
}

pub(crate) const LOG_CUT: &[u8] = b"[log bytes cut]\n";

pub(crate) fn render_log(text: Box<[u8]>, truncated: bool, budget: u32) -> Option<Box<[u8]>> {
    let room = usize::try_from(budget).ok()?;
    if !truncated && text.len() <= room {
        return Some(text);
    }
    let kept = edge(&text, room.checked_sub(LOG_CUT.len())?.min(text.len()), true);
    let mut writer = Writer::new(LOG_CUT.len().checked_add(kept.len())?);
    writer.put(LOG_CUT).expect("measured log marker");
    writer.put(kept).expect("measured log tail");
    Some(writer.finish())
}

fn render_edge(text: Box<[u8]>, left: u64, budget: u32, tail: bool) -> Option<Box<[u8]>> {
    let room = usize::try_from(budget).ok()?;
    if left == 0 && text.len() <= room {
        return Some(text);
    }
    let mut low = 0_usize;
    let mut high = text.len().min(room);
    let minimum = marker(left.checked_add(skein_lib::len_u64(text.len()))?, false);
    if minimum > room {
        return None;
    }
    for _ in 0..usize::BITS {
        if low >= high {
            break;
        }
        let mid = low.saturating_add(high.saturating_sub(low).div_ceil(2));
        let kept = edge(&text, mid, tail);
        let lost = left.checked_add(u64::try_from(text.len().saturating_sub(kept.len())).ok()?)?;
        if kept.len().saturating_add(marker(lost, !kept.is_empty())) <= room {
            low = mid;
        } else {
            high = mid.saturating_sub(1);
        }
    }
    let kept = edge(&text, low, tail);
    let lost = left.checked_add(u64::try_from(text.len().saturating_sub(kept.len())).ok()?)?;
    if lost == 0 {
        return Some(Box::from(kept));
    }
    let digits = Decimal::of(lost);
    let size = kept.len().saturating_add(marker(lost, !kept.is_empty()));
    let mut writer = Writer::new(size);
    writer.put(kept).expect("measured section");
    if !kept.is_empty() {
        writer.put(b"\n").expect("measured break");
    }
    writer.put(b"[").expect("measured marker");
    writer.put(digits.as_bytes()).expect("measured count");
    writer.put(b" bytes cut]\n").expect("measured marker");
    Some(writer.finish())
}

fn marker(lost: u64, leading: bool) -> usize {
    usize::from(leading)
        .saturating_add(b"[".len())
        .saturating_add(Decimal::of(lost).as_bytes().len())
        .saturating_add(b" bytes cut]\n".len())
}

/// A section may stand on its partner once; both unread sections fail the brief.
pub(crate) fn stand_on_partner(domain: &mut Domain, section: Token) -> bool {
    let Some(pending) = domain.brief_pending.get(&section) else { return false };
    let Some(partner) = pending.partner else { return false };
    let readable = match domain.brief_pending.get(&partner) {
        Some(partner) => !partner.unread && !partner.dropped,
        None => match domain.brief_held.get(&partner) {
            Some(partner) => !partner.unread,
            None => domain.brief_planned.contains_key(&partner),
        },
    };
    if readable {
        domain.brief_pending.get_mut(&section).expect("pending section").unread = true;
    }
    readable
}
