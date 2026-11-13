//! A delivery's commit fields and smith's delivery terminals (jig's
//! `domain/hosts/README.md`, sections 6 and 7). The fields arrive typed; this keeps
//! temper's policy over them, and translates a push's landings into smith's
//! terminal, push receipts in words (`temper/docs/design/domain/agent/README.md`,
//! section 5) and operator tails as data.

use alloc::boxed::Box;
use jig_domain_host as host;
use skein_lib::{List, Writer};
use smith_host_domain as smith;

use crate::{Domain, checkout, wire, workspace};

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

pub(crate) fn landings(
    landings: &[checkout::Landing],
    branches: &[Option<Box<[u8]>>],
    unchecked: bool,
) -> smith::Delivery {
    let mut receipts = List::with_capacity(u32::try_from(landings.len()).expect("bounded directories"));
    let mut first = None;
    let mut moved = false;
    for (directory, landing) in landings.iter().enumerate() {
        let directory = u32::try_from(directory).expect("bounded directories");
        let failure = match landing {
            checkout::Landing::Landed { commit } => {
                receipts
                    .push(receipt(directory, &commit.raw(), branches, unchecked, None))
                    .expect("room for changed directories");
                None
            }
            checkout::Landing::Prefixes { commit, branch } => {
                let receipt = smith::Receipt::new(directory, receipt_text(&commit.raw(), branch, false, None));
                receipts
                    .push(receipt.expect("prepare branch and commit validated at admission"))
                    .expect("one receipt per directory");
                None
            }
            checkout::Landing::DesignMerged { commit, tip } => {
                receipts
                    .push(receipt(directory, &commit.raw(), branches, unchecked, Some(&tip.raw())))
                    .expect("room for changed directories");
                None
            }
            checkout::Landing::Moved => {
                moved = true;
                None
            }
            checkout::Landing::Unchanged => None,
            checkout::Landing::Conflicted { files } => {
                Some(refused(directory, first_path(files), b"resolve conflict markers before delivering"))
            }
            checkout::Landing::DesignConflict { files } => {
                Some(refused(directory, first_path(files), b"settle the files in conflict and deliver again"))
            }
            checkout::Landing::DesignCode { path } => Some(refused(
                directory,
                Some(path),
                b"a delivery to the goal branch carries Markdown only; code goes through a change",
            )),
            checkout::Landing::DesignMoving => {
                Some(refused(directory, None, b"the goal branch kept moving; deliver again"))
            }
            checkout::Landing::DesignUnchecked => {
                Some(refused(directory, None, b"the goal branch takes checked deliveries alone; deliver instead"))
            }
            checkout::Landing::Refused => {
                Some(failed(directory, smith::DeliveryReason::RefusedByTarget, &smith::Diagnostic::empty()))
            }
            checkout::Landing::Failed | checkout::Landing::Aborted => {
                Some(failed(directory, smith::DeliveryReason::Unknown, &smith::Diagnostic::empty()))
            }
            checkout::Landing::Explained { fault, diagnostic } => Some(failed(
                directory,
                reason(crate::translate::push_reason(*fault)),
                &smith::Diagnostic::new(diagnostic.output(), diagnostic.cut()),
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

fn first_path(files: &[Box<[u8]>]) -> Option<&[u8]> {
    match files.first() {
        Some(path) => Some(path),
        None => None,
    }
}

fn refused(directory: u32, path: Option<&[u8]>, words: &[u8]) -> smith::Delivery {
    let marker = match path {
        Some(path) => Some(smith::Marker::new(directory, path.into()).expect("validated checkout path")),
        None => None,
    };
    smith::Delivery::Refused(smith::DeliveryRefusal::new(marker, words.into()).expect("bounded delivery feedback"))
}

fn receipt(
    directory: u32,
    commit: &[u8; 32],
    branches: &[Option<Box<[u8]>>],
    unchecked: bool,
    merged: Option<&[u8; 32]>,
) -> smith::Receipt {
    let branch = branches
        .get(skein_lib::usize_of(directory))
        .expect("every item has a branch slot")
        .as_deref()
        .expect("a landed repository is writable");
    smith::Receipt::new(directory, receipt_text(commit, branch, unchecked, merged)).expect("bounded commit receipt")
}

fn head_width(commit: &[u8; 32]) -> usize {
    if commit.get(20..).expect("fixed id") == [0; 12] { 20 } else { 32 }
}

fn put_head(text: &mut Writer, commit: &[u8; 32]) {
    for byte in commit.get(..head_width(commit)).expect("fixed id") {
        let high = *b"0123456789abcdef".get(usize::from(byte >> 4_u8)).expect("hex nibble");
        let low = *b"0123456789abcdef".get(usize::from(byte & 15)).expect("hex nibble");
        text.put(&[high, low]).expect("two characters per commit byte");
    }
}

fn receipt_text(commit: &[u8; 32], branch: &[u8], unchecked: bool, merged: Option<&[u8; 32]>) -> Box<[u8]> {
    const TO_BRANCH: &[u8] = b" to branch ";
    let prefix: &[u8] = if unchecked { b"saved " } else { b"pushed " };
    let suffix: &[u8] = if unchecked {
        b", unchecked: no check ran; your checkout stays detached, as expected"
    } else {
        b"; your checkout stays detached, as expected"
    };
    let width = head_width(commit);
    let merged_size = match merged {
        Some(tip) => b", merging its tip "
            .len()
            .checked_add(head_width(tip).checked_mul(2).expect("fixed digits"))
            .expect("fixed merge prefix")
            .checked_add(b" first".len())
            .expect("fixed merge suffix"),
        None => 0,
    };
    let base = width
        .checked_mul(2)
        .expect("fixed commit digits")
        .checked_add(prefix.len())
        .expect("fixed receipt prefix")
        .checked_add(suffix.len())
        .expect("fixed receipt suffix")
        .checked_add(merged_size)
        .expect("fixed merged tip");
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
    put_head(&mut text, commit);
    if let Some(branch) = branch {
        text.put(TO_BRANCH).expect("measured branch prefix");
        text.put(branch).expect("measured branch");
    }
    if let Some(tip) = merged {
        text.put(b", merging its tip ").expect("measured merge prefix");
        put_head(&mut text, tip);
        text.put(b" first").expect("measured merge suffix");
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
