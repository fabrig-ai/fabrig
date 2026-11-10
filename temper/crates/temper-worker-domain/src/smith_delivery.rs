//! A delivery's commit fields and smith's delivery terminals (jig's
//! `domain/hosts/README.md`, sections 6 and 7). The fields arrive typed; this keeps
//! temper's policy over them, and translates a push's landings into smith's
//! terminal, push receipts in words (`temper/docs/design/domain/agent/README.md`,
//! section 5) and operator tails as data.

use alloc::boxed::Box;
use jig_domain_host as host;
use skein_lib::{List, Writer};
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
pub(crate) fn reply(domain: &Domain, reply_bytes: u32, reply: host::CallAnswer) -> smith::Reply {
    match reply {
        host::CallAnswer::Relayed { error, text } => {
            if text.len() > skein_lib::usize_of(reply_bytes) {
                smith::Reply::TooLarge
            } else {
                smith::Reply::Host { error, body: text }
            }
        }
        host::CallAnswer::Delivered(delivery) => {
            smith::Reply::Delivery(workspace::smith_delivery(domain, delivery.left))
        }
        host::CallAnswer::Unavailable => smith::Reply::Unavailable,
        host::CallAnswer::Withdrawn => smith::Reply::Withdrawn,
        host::CallAnswer::Busy => smith::Reply::Busy,
    }
}

pub(crate) fn landings(landings: &[wire::Landing], branches: &[Option<Box<[u8]>>], unchecked: bool) -> smith::Delivery {
    let mut receipts = List::with_capacity(u32::try_from(landings.len()).expect("bounded directories"));
    let mut first = None;
    let mut moved = false;
    for (directory, landing) in landings.iter().enumerate() {
        let directory = u32::try_from(directory).expect("bounded directories");
        let failure = match landing {
            wire::Landing::Landed { commit } => {
                let branch = branches
                    .get(skein_lib::usize_of(directory))
                    .expect("every item has a branch slot")
                    .as_deref()
                    .expect("a landed repository is writable");
                let text = receipt_text(commit, branch, unchecked);
                receipts
                    .push(smith::Receipt::new(directory, text).expect("bounded commit receipt"))
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

fn receipt_text(commit: &[u8; 32], branch: &[u8], unchecked: bool) -> Box<[u8]> {
    const TO_BRANCH: &[u8] = b" to branch ";
    let prefix: &[u8] = if unchecked { b"saved " } else { b"pushed " };
    let suffix: &[u8] = if unchecked {
        b", unchecked: no check ran; your checkout stays detached, as expected"
    } else {
        b"; your checkout stays detached, as expected"
    };
    let width = if commit.get(20..).expect("fixed id") == [0; 12] { 20_usize } else { 32 };
    let digits = commit.get(..width).expect("fixed id");
    let base = width
        .checked_mul(2)
        .expect("fixed commit digits")
        .checked_add(prefix.len())
        .expect("fixed receipt prefix")
        .checked_add(suffix.len())
        .expect("fixed receipt suffix");
    let room = smith::Receipt::CAPACITY
        .checked_sub(base)
        .expect("fixed receipt fits")
        .checked_sub(TO_BRANCH.len())
        .expect("branch prefix fits");
    let branch = if branch.len() <= room { Some(branch) } else { None };
    let len = match branch {
        Some(branch) => base
            .checked_add(TO_BRANCH.len())
            .expect("branch prefix fits")
            .checked_add(branch.len())
            .expect("checked branch fits"),
        None => base,
    };
    let mut text = Writer::new(len);
    text.put(prefix).expect("measured receipt prefix");
    for byte in digits {
        let high = *b"0123456789abcdef".get(usize::from(byte >> 4_u8)).expect("hex nibble");
        let low = *b"0123456789abcdef".get(usize::from(byte & 15)).expect("hex nibble");
        text.put(&[high, low]).expect("two characters per commit byte");
    }
    if let Some(branch) = branch {
        text.put(TO_BRANCH).expect("measured branch prefix");
        text.put(branch).expect("measured branch");
    }
    text.put(suffix).expect("measured receipt suffix");
    text.finish()
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
