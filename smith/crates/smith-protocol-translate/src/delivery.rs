//! Settled delivery translation (smith/docs/design/protocol/channel.md, section 5).
use crate::{Error, generated::channel};
use smith_domain::run;

/// Decode a settled delivery into its bounded domain value.
pub fn saved_delivery(source: &smith_channel::Delivery) -> Result<run::Delivery, Error> {
    channel::delivery_to_domain(source.clone()).or(Err(Error::InvalidSavedAnswer))
}
