//! Checked delivery evidence and list failures that have no structural channel map
//! (smith/docs/design/protocol/channel.md, sections 3–5; protocol/host.md, section 2).
use crate::generated::channel;
use alloc::boxed::Box;
use skein_codec::Reason;
use skein_lib::List;
use smith_channel::v14 as wire;
use smith_host_domain as host;

pub(crate) fn list_failure_to_domain(record: wire::ListFailure) -> host::ListFailure {
    match record.failure {
        wire::ListFailed::Refused(refused) => host::ListFailure::Refused { status: refused.status },
        wire::ListFailed::Unreachable => host::ListFailure::Unreachable,
        wire::ListFailed::Unreadable => host::ListFailure::Unreadable,
        wire::ListFailed::Deadline => host::ListFailure::Deadline,
    }
}

pub(crate) fn delivered_to_record(
    delivered: &host::Delivered,
    limits: &wire::Limits,
) -> Result<wire::Delivered, wire::Problem> {
    let mut receipts = List::with_capacity(limits.delivered_receipts);
    for receipt in delivered.receipts() {
        let record = wire::Receipt { directory: receipt.directory(), text: Box::from(receipt.text()) };
        receipts.push(record).or(Err(wire::paths::DELIVERED_RECEIPTS.problem(Reason::Bound)))?;
    }
    Ok(wire::Delivered { receipts })
}

pub(crate) fn delivery_refusal_to_record(value: &host::DeliveryRefusal) -> wire::DeliveryRefusal {
    let marker = match value.marker() {
        Some(marker) => Some(wire::Marker { directory: marker.directory(), path: Box::from(marker.path()) }),
        None => None,
    };
    wire::DeliveryRefusal { marker, explanation: Box::from(value.explanation()) }
}

pub(crate) fn delivery_failure_to_record(value: &host::DeliveryFailure) -> wire::DeliveryFailure {
    wire::DeliveryFailure {
        directory: value.directory,
        reason: channel::delivery_reason_to_record(&value.reason),
        diagnostic: Box::from(value.diagnostic.output()),
        dropped: value.diagnostic.cut(),
    }
}
