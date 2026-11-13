//! Ops's observability words and delegates (jig/docs/design/examples.md, 2 and 4).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;
use jig_domain_core_vocabulary as core;
use jig_ops_domain_observability as observability;
use jig_protocol_tools::Limits;

/// Render the alert identity and severity as subscriber words.
#[must_use]
pub fn alert_words(alert: &observability::Alert) -> Box<[u8]> {
    let bytes = b"alert "
        .len()
        .checked_add(skein_lib::Decimal::of(alert.number).as_bytes().len())
        .expect("alert number")
        .checked_add(b" severity ".len())
        .expect("severity heading")
        .checked_add(skein_lib::Decimal::of(u64::from(alert.severity)).as_bytes().len())
        .expect("severity");
    let mut writer = skein_lib::Writer::new(bytes);
    writer.put(b"alert ").expect("alert heading");
    writer.put(skein_lib::Decimal::of(alert.number).as_bytes()).expect("alert identity");
    writer.put(b" severity ").expect("severity heading");
    writer.put(skein_lib::Decimal::of(u64::from(alert.severity)).as_bytes()).expect("severity");
    writer.finish()
}
/// Render a service's health as subscriber words.
#[must_use]
pub fn health_words(healthy: bool) -> Box<[u8]> {
    if healthy { Box::from(&b"healthy"[..]) } else { Box::from(&b"unhealthy"[..]) }
}
/// Decode one triage delegate and append the alert batch to its words.
#[must_use]
pub fn triage(delegate: &[u8], alerts: &[u64], limits: &Limits) -> Option<Box<[core::Delegate]>> {
    let mut batch = jig_protocol_tools::batch(delegate, limits).ok()?;
    if batch.len() != 1 {
        return None;
    }
    for member in &mut batch {
        let mut room = member.spec.words.len().checked_add(b"\nAlerts:".len())?;
        for alert in alerts {
            room = room.checked_add(1)?.checked_add(skein_lib::Decimal::of(*alert).as_bytes().len())?;
        }
        let mut words = skein_lib::Writer::new(room);
        words.put(&member.spec.words).ok()?;
        words.put(b"\nAlerts:").ok()?;
        for alert in alerts {
            words.put(b" ").ok()?;
            words.put(skein_lib::Decimal::of(*alert).as_bytes()).ok()?;
        }
        member.spec.words = words.finish();
    }
    Some(batch)
}

/// The owner templates always request one delegate, matching the outcome's one child.
#[must_use]
pub fn lead(delegate: &[u8], limits: &Limits) -> Option<Box<[core::Delegate]>> {
    let batch = jig_protocol_tools::batch(delegate, limits).ok()?;
    if batch.len() != 1 {
        return None;
    }
    Some(batch)
}

#[cfg(test)]
mod tests;
