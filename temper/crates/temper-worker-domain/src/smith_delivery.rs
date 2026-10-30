//! A delivery's commit fields and smith's delivery terminals (jig's
//! `domain/hosts.md`, sections 7 and 9). The fields arrive typed; this keeps
//! temper's policy over them, and translates a push's landings into smith's
//! terminal, commit heads as receipts and operator tails as data.

use alloc::boxed::Box;
use jig_domain_host as host;
use skein_lib::{Decimal, List, Writer};
use smith_host_domain as smith;

use crate::{Domain, wire, workspace};

pub(crate) struct Fields {
    pub(crate) title: Box<[u8]>,
    pub(crate) body: Box<[u8]>,
}

/// A delivery's commit message: exactly a title and a body, the title not
/// empty, the two within `limit` bytes; nothing otherwise.
pub(crate) fn fields(fields: smith::Fields, limit: u32) -> Option<Fields> {
    let mut title = None;
    let mut body = None;
    for field in fields.into_value().fields {
        match field.name.as_ref() {
            b"title" if title.is_none() => title = Some(field.value),
            b"body" if body.is_none() => body = Some(field.value),
            _ => return None,
        }
    }
    let title = title?;
    let body = body?;
    let bytes = u32::try_from(title.len().checked_add(body.len())?).ok()?;
    if bytes > limit || title.is_empty() {
        return None;
    }
    Some(Fields { title, body })
}

/// The hub's reply to a call, as smith takes it: a delivery's terminal is the
/// one the workspace made of its push.
pub(crate) fn reply(domain: &Domain, reply_bytes: u32, reply: host::Reply) -> smith::Reply {
    match reply {
        host::Reply::Relayed { error, text } => {
            if text.len() > usize::try_from(reply_bytes).expect("u32 fits usize") {
                match cut(&text, reply_bytes) {
                    Some(body) => smith::Reply::Host { error, body },
                    None => smith::Reply::TooLarge,
                }
            } else {
                smith::Reply::Host { error, body: text }
            }
        }
        host::Reply::Delivered(delivery) => smith::Reply::Delivery(workspace::smith_delivery(domain, delivery.left)),
        host::Reply::Unavailable => smith::Reply::Unavailable,
        host::Reply::Withdrawn => smith::Reply::Withdrawn,
        host::Reply::Busy => smith::Reply::Busy,
    }
}

/// Keep the answer's head and a counted marker inside the receiving room
/// (`temper/docs/design/domain/agent.md`, 4.3). Live and saved replies use
/// the same cut so retrying a call presents the same text.
pub(crate) fn cut(text: &[u8], reply_bytes: u32) -> Option<Box<[u8]>> {
    const PREFIX: &[u8] = b"\n[answer cut: ";
    const SUFFIX: &[u8] = b" bytes not shown; read a narrower part, e.g. read_file with max_bytes]";
    let room = usize::try_from(reply_bytes).expect("u32 fits usize");
    let mut head = room.min(text.len());
    // A u64 count has at most twenty decimal digits. Each repeat grows the
    // count's width; UTF-8 needs at most three continuation bytes removed.
    for _ in 0..21_u32 {
        for _ in 0..3_u32 {
            match text.get(head) {
                Some(byte) if byte & 0xc0 == 0x80 => head = head.checked_sub(1)?,
                Some(_) | None => break,
            }
        }
        let dropped = text.len().checked_sub(head)?;
        let count = Decimal::of(u64::try_from(dropped).ok()?);
        let marker = PREFIX.len().checked_add(count.as_bytes().len())?.checked_add(SUFFIX.len())?;
        let size = head.checked_add(marker)?;
        if size <= room {
            let mut writer = Writer::new(size);
            writer.put(text.get(..head)?).ok()?;
            writer.put(PREFIX).ok()?;
            writer.put(count.as_bytes()).ok()?;
            writer.put(SUFFIX).ok()?;
            return Some(writer.finish());
        }
        head = head.checked_sub(size.checked_sub(room)?)?;
    }
    None
}

#[expect(clippy::manual_map, reason = "explicit option match without closures in step code")]
pub(crate) fn landings(landings: &[wire::Landing]) -> smith::Delivery {
    let mut receipts = List::with_capacity(u32::try_from(landings.len()).expect("bounded directories"));
    let mut first = None;
    let mut moved = false;
    for (directory, landing) in landings.iter().enumerate() {
        let directory = u32::try_from(directory).expect("bounded directories");
        let failure = match landing {
            wire::Landing::Landed { commit } => {
                let mut text = Writer::new(64);
                for byte in commit {
                    let high = *b"0123456789abcdef".get(usize::from(byte >> 4_u8)).expect("hex nibble");
                    let low = *b"0123456789abcdef".get(usize::from(byte & 15)).expect("hex nibble");
                    text.put(&[high, low]).expect("two characters per commit byte");
                }
                receipts
                    .push(smith::Receipt::new(directory, text.finish()).expect("bounded commit receipt"))
                    .expect("room for changed directories");
                None
            }
            wire::Landing::Moved => {
                moved = true;
                None
            }
            wire::Landing::Unchanged => None,
            wire::Landing::Conflicted { files } => {
                let marker = match files.first() {
                    Some(path) => Some(smith::Marker::new(directory, path.clone()).expect("validated conflict path")),
                    None => None,
                };
                Some(smith::Delivery::Refused(
                    smith::DeliveryRefusal::new(marker, Box::from(&b"resolve conflict markers before delivering"[..]))
                        .expect("bounded conflict feedback"),
                ))
            }
            wire::Landing::Refused => {
                Some(failed(directory, smith::DeliveryReason::RefusedByTarget, &smith::Diagnostic::empty()))
            }
            wire::Landing::Failed => {
                Some(failed(directory, smith::DeliveryReason::Unknown, &smith::Diagnostic::empty()))
            }
            wire::Landing::Explained { failure } => Some(failed(
                directory,
                reason(failure.reason),
                &smith::Diagnostic::new(failure.diagnostic.output(), failure.diagnostic.cut()),
            )),
        };
        if first.is_none() {
            first = failure;
        }
    }
    if moved {
        smith::Delivery::Stale
    } else if let Some(failed) = first {
        failed
    } else if receipts.is_empty() {
        smith::Delivery::Nothing
    } else {
        smith::Delivery::Delivered(
            smith::Delivered::new(receipts.into_boxed()).expect("one unique receipt per changed directory"),
        )
    }
}

fn failed(directory: u32, reason: smith::DeliveryReason, diagnostic: &smith::Diagnostic) -> smith::Delivery {
    smith::Delivery::Failed(smith::DeliveryFailure { directory, reason, diagnostic: *diagnostic })
}

fn reason(reason: wire::PushReason) -> smith::DeliveryReason {
    match reason {
        wire::PushReason::MissingRepository | wire::PushReason::MissingBranch | wire::PushReason::MissingCommit => {
            smith::DeliveryReason::Missing
        }
        wire::PushReason::Refused => smith::DeliveryReason::RefusedByTarget,
        wire::PushReason::Unreachable => smith::DeliveryReason::Unreachable,
        wire::PushReason::Broken => smith::DeliveryReason::Broken,
        wire::PushReason::TimedOut => smith::DeliveryReason::TimedOut,
        wire::PushReason::Cancelled => smith::DeliveryReason::Cancelled,
        wire::PushReason::Unavailable => smith::DeliveryReason::Unavailable,
        wire::PushReason::Busy => smith::DeliveryReason::Busy,
        wire::PushReason::TooLarge => smith::DeliveryReason::TooLarge,
        wire::PushReason::Nothing | wire::PushReason::Unknown => smith::DeliveryReason::Unknown,
    }
}
