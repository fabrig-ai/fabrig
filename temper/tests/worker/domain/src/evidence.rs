//! A resumed delivery's evidence, as temper's protocol layer is to encode
//! and decode it: smith's channel record of the delivery's terminal, which
//! must agree with the outcome the engine committed (`domain/worker.md`,
//! 4.1). temper has no protocol layer yet, so its codec lives with the world
//! that plays one.

use smith_channel as wire;
use temper_worker_domain::{Invalid, agent, host};

/// The delivery the evidence records, if it decodes and agrees with
/// `outcome`.
///
/// # Errors
///
/// [`Invalid::Evidence`] for evidence that does not decode, or that records
/// another outcome.
pub fn decode(outcome: host::DeliveryOutcome, evidence: &[u8]) -> Result<agent::Delivery, Invalid> {
    let delivery = delivery(evidence).ok_or(Invalid::Evidence)?;
    let recorded = match &delivery {
        agent::Delivery::Delivered(_) => host::DeliveryOutcome::Delivered,
        agent::Delivery::Nothing => host::DeliveryOutcome::Nothing,
        agent::Delivery::Stale => host::DeliveryOutcome::Stale,
        agent::Delivery::Refused(_) => host::DeliveryOutcome::Refused,
        agent::Delivery::Failed(_) => host::DeliveryOutcome::Failed,
    };
    if recorded == outcome { Ok(delivery) } else { Err(Invalid::Evidence) }
}

/// Encodes `delivery` as evidence for a settled call, if it fits smith's
/// channel record.
#[must_use]
pub fn encode(delivery: &agent::Delivery) -> Option<Box<[u8]>> {
    let limits = &wire::CEILINGS;
    let value = match delivery {
        agent::Delivery::Delivered(delivered) => {
            let mut receipts = skein_lib::List::with_capacity(limits.delivered_receipts);
            for receipt in delivered.receipts() {
                let parts = wire::ReceiptParts { directory: receipt.directory(), text: Box::from(receipt.text()) };
                receipts.push(wire::Receipt::new(limits, parts).ok()?).ok()?;
            }
            wire::Delivery::Delivered(wire::Delivered::new(limits, wire::DeliveredParts { receipts }).ok()?)
        }
        agent::Delivery::Nothing => wire::Delivery::Nothing,
        agent::Delivery::Stale => wire::Delivery::Stale,
        agent::Delivery::Refused(refusal) => {
            let marker = match refusal.marker() {
                Some(marker) => {
                    let parts = wire::MarkerParts { directory: marker.directory(), path: Box::from(marker.path()) };
                    Some(wire::Marker::new(limits, parts).ok()?)
                }
                None => None,
            };
            let parts = wire::DeliveryRefusalParts { marker, explanation: Box::from(refusal.explanation()) };
            wire::Delivery::Refused(wire::DeliveryRefusal::new(limits, parts).ok()?)
        }
        agent::Delivery::Failed(failure) => {
            let parts = wire::DeliveryFailureParts {
                directory: failure.directory,
                reason: encoded_reason(failure.reason),
                diagnostic: Box::from(failure.diagnostic.output()),
                dropped: failure.diagnostic.cut(),
            };
            wire::Delivery::Failed(wire::DeliveryFailure::new(limits, parts).ok()?)
        }
    };
    let record = wire::DeliveryReply::new(limits, wire::DeliveryReplyParts { value }).ok()?;
    let mut writer = skein_lib::Writer::new(usize::try_from(record.measure()).ok()?);
    record.encode(&mut writer).ok()?;
    Some(writer.finish())
}

fn delivery(evidence: &[u8]) -> Option<agent::Delivery> {
    let record = wire::DeliveryReply::decode(&wire::CEILINGS, &mut skein_lib::Reader::new(evidence)).ok()?;
    Some(match record.into_parts().value {
        wire::Delivery::Delivered(delivered) => {
            let mut receipts = Vec::new();
            for row in delivered.into_parts().receipts.into_boxed() {
                let parts = row.into_parts();
                receipts.push(agent::Receipt::new(parts.directory, parts.text)?);
            }
            agent::Delivery::Delivered(agent::Delivered::new(receipts.into_boxed_slice())?)
        }
        wire::Delivery::Nothing => agent::Delivery::Nothing,
        wire::Delivery::Stale => agent::Delivery::Stale,
        wire::Delivery::Refused(refusal) => {
            let parts = refusal.into_parts();
            let marker = match parts.marker {
                Some(marker) => {
                    let fields = marker.into_parts();
                    Some(agent::Marker::new(fields.directory, fields.path)?)
                }
                None => None,
            };
            agent::Delivery::Refused(agent::DeliveryRefusal::new(marker, parts.explanation)?)
        }
        wire::Delivery::Failed(failure) => {
            let parts = failure.into_parts();
            agent::Delivery::Failed(agent::DeliveryFailure {
                directory: parts.directory,
                reason: reason(&parts.reason),
                diagnostic: agent::Diagnostic::new(&parts.diagnostic, parts.dropped),
            })
        }
    })
}

fn reason(reason: &wire::DeliveryReason) -> agent::DeliveryReason {
    match *reason {
        wire::DeliveryReason::Unreachable => agent::DeliveryReason::Unreachable,
        wire::DeliveryReason::RefusedByTarget => agent::DeliveryReason::RefusedByTarget,
        wire::DeliveryReason::TimedOut => agent::DeliveryReason::TimedOut,
        wire::DeliveryReason::Broken => agent::DeliveryReason::Broken,
        wire::DeliveryReason::TooLarge => agent::DeliveryReason::TooLarge,
        wire::DeliveryReason::Missing => agent::DeliveryReason::Missing,
        wire::DeliveryReason::Busy => agent::DeliveryReason::Busy,
        wire::DeliveryReason::Unavailable => agent::DeliveryReason::Unavailable,
        wire::DeliveryReason::Cancelled => agent::DeliveryReason::Cancelled,
        wire::DeliveryReason::Unknown => agent::DeliveryReason::Unknown,
    }
}

fn encoded_reason(reason: agent::DeliveryReason) -> wire::DeliveryReason {
    match reason {
        agent::DeliveryReason::Unreachable => wire::DeliveryReason::Unreachable,
        agent::DeliveryReason::RefusedByTarget => wire::DeliveryReason::RefusedByTarget,
        agent::DeliveryReason::TimedOut => wire::DeliveryReason::TimedOut,
        agent::DeliveryReason::Broken => wire::DeliveryReason::Broken,
        agent::DeliveryReason::TooLarge => wire::DeliveryReason::TooLarge,
        agent::DeliveryReason::Missing => wire::DeliveryReason::Missing,
        agent::DeliveryReason::Busy => wire::DeliveryReason::Busy,
        agent::DeliveryReason::Unavailable => wire::DeliveryReason::Unavailable,
        agent::DeliveryReason::Cancelled => wire::DeliveryReason::Cancelled,
        agent::DeliveryReason::Unknown => wire::DeliveryReason::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::{decode, encode};
    use temper_worker_domain::agent::{Delivered, Delivery, DeliveryFailure, DeliveryReason, Diagnostic, Receipt};
    use temper_worker_domain::{Invalid, host::DeliveryOutcome};

    #[test]
    fn receipts_survive_a_resumed_delivery() {
        let receipt = Receipt::new(2, Box::from(&b"head:abc123"[..])).expect("bounded receipt");
        let delivered = Delivery::Delivered(Delivered::new(Box::new([receipt])).expect("one receipt"));
        let bytes = encode(&delivered).expect("codec accepts receipt");
        assert_eq!(decode(DeliveryOutcome::Delivered, &bytes), Ok(delivered));
    }

    #[test]
    fn failed_diagnostics_survive_a_resumed_delivery() {
        let failed = Delivery::Failed(DeliveryFailure {
            directory: 1,
            reason: DeliveryReason::TimedOut,
            diagnostic: Diagnostic::new(b"push timed out", 7),
        });
        let bytes = encode(&failed).expect("codec accepts diagnostic");
        assert_eq!(decode(DeliveryOutcome::Failed, &bytes), Ok(failed));
    }

    #[test]
    fn malformed_evidence_is_invalid() {
        assert_eq!(decode(DeliveryOutcome::Nothing, &[255, 0, 1]), Err(Invalid::Evidence));
    }

    #[test]
    fn evidence_that_disagrees_with_its_outcome_is_invalid() {
        let evidence = encode(&Delivery::Nothing).expect("codec accepts nothing");
        assert_eq!(decode(DeliveryOutcome::Delivered, &evidence), Err(Invalid::Evidence));
    }
}
