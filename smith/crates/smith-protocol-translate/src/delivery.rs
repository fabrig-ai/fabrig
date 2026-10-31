//! Settled delivery translation (protocol/channel.md, section 5).
use crate::Error;
use alloc::boxed::Box;
use skein_lib::List;
use smith_channel as channel;
use smith_domain::run;

/// Decode a settled delivery into its bounded domain value.
pub fn saved_delivery(source: &channel::Delivery) -> Result<run::Delivery, Error> {
    let delivery = match source {
        channel::Delivery::Delivered(delivered) => {
            let mut receipts = List::with_capacity(delivered.receipts().len());
            for receipt in delivered.receipts() {
                let receipt = run::Receipt::new(receipt.directory(), Box::from(receipt.text()))
                    .ok_or(Error::InvalidSavedAnswer)?;
                if receipts.push(receipt).is_err() {
                    return Err(Error::InvalidSavedAnswer);
                }
            }
            let evidence = run::Delivered::new(receipts.into_boxed()).ok_or(Error::InvalidSavedAnswer)?;
            run::Delivery::Delivered(evidence)
        }
        channel::Delivery::Nothing => run::Delivery::Nothing,
        channel::Delivery::Refused(refused) => {
            let marker = match refused.marker() {
                Some(marker) => Some(
                    run::Marker::new(marker.directory(), Box::from(marker.path())).ok_or(Error::InvalidSavedAnswer)?,
                ),
                None => None,
            };
            let refusal =
                run::DeliveryRefusal::new(marker, Box::from(refused.explanation())).ok_or(Error::InvalidSavedAnswer)?;
            run::Delivery::Refused(refusal)
        }
        channel::Delivery::Failed(failed) => {
            let reason = match failed.reason() {
                channel::DeliveryReason::Unreachable => run::DeliveryReason::Unreachable,
                channel::DeliveryReason::RefusedByTarget => run::DeliveryReason::RefusedByTarget,
                channel::DeliveryReason::TimedOut => run::DeliveryReason::TimedOut,
                channel::DeliveryReason::Broken => run::DeliveryReason::Broken,
                channel::DeliveryReason::TooLarge => run::DeliveryReason::TooLarge,
                channel::DeliveryReason::Missing => run::DeliveryReason::Missing,
                channel::DeliveryReason::Busy => run::DeliveryReason::Busy,
                channel::DeliveryReason::Unavailable => run::DeliveryReason::Unavailable,
                channel::DeliveryReason::Cancelled => run::DeliveryReason::Cancelled,
                channel::DeliveryReason::Unknown => run::DeliveryReason::Unknown,
            };
            run::Delivery::Failed(run::DeliveryFailure {
                directory: failed.directory(),
                reason,
                diagnostic: run::Diagnostic::new(failed.diagnostic(), failed.dropped()),
            })
        }
        channel::Delivery::Stale => run::Delivery::Stale,
    };
    Ok(delivery)
}
