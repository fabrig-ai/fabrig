//! Checked saved answers at agent admission (smith/docs/design/protocol/channel.md, section 5).
use crate::generated::channel;
use skein_codec::Reason;
use skein_lib::List;
use smith_channel::v14 as wire;
use smith_domain::run;

pub(crate) fn delivered_to_domain(record: wire::Delivered) -> Result<run::Delivered, wire::Problem> {
    let rows = record.receipts;
    let mut receipts = List::with_capacity(rows.len());
    let path = wire::paths::DELIVERED_RECEIPTS;
    for row in rows.into_boxed() {
        let receipt = run::Receipt::new(row.directory, row.text).ok_or(path.problem(Reason::Malformed))?;
        receipts.push(receipt).or(Err(path.problem(Reason::Bound)))?;
    }
    run::Delivered::new(receipts.into_boxed()).ok_or(path.problem(Reason::Malformed))
}

pub(crate) fn delivery_refusal_to_domain(record: wire::DeliveryRefusal) -> Result<run::DeliveryRefusal, wire::Problem> {
    let marker = match record.marker {
        Some(marker) => Some(
            run::Marker::new(marker.directory, marker.path)
                .ok_or(wire::paths::DELIVERY_REFUSAL_MARKER.problem(Reason::Malformed))?,
        ),
        None => None,
    };
    run::DeliveryRefusal::new(marker, record.explanation)
        .ok_or(wire::paths::DELIVERY_REFUSAL_EXPLANATION.problem(Reason::Malformed))
}

pub(crate) fn delivery_failure_to_domain(record: wire::DeliveryFailure) -> run::DeliveryFailure {
    run::DeliveryFailure {
        directory: record.directory,
        reason: channel::delivery_reason_to_domain(record.reason),
        diagnostic: run::Diagnostic::new(&record.diagnostic, record.dropped),
    }
}

pub(crate) fn host_reply_to_domain(record: wire::HostReply) -> Result<run::HostAnswer, wire::Problem> {
    run::HostAnswer::new(record.text, record.error).ok_or(wire::paths::HOST_REPLY_TEXT.problem(Reason::Bound))
}
