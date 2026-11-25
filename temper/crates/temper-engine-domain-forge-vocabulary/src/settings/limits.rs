//! Limit overrides at the client boundary (protocol/configuration/limit-settings.md, 2).
use crate::generated::client_wire::{limit_override_to_domain, limit_override_to_record};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::settings::{OVERRIDES, Override, OverrideFault};
use skein_lib::{Decimal, List};
use temper_client_protocol_wire::v1 as w;

#[must_use]
pub fn limits_record(values: &[Override]) -> w::LimitOverrides {
    let mut overrides = List::with_capacity(OVERRIDES);
    for value in values {
        overrides.push(limit_override_to_record(value)).expect("admitted override count");
    }
    w::LimitOverrides { overrides }
}

#[must_use]
pub fn limits_from(record: w::LimitOverrides) -> Box<[Override]> {
    let mut overrides = List::with_capacity(OVERRIDES);
    for value in record.overrides.into_boxed() {
        overrides.push(limit_override_to_domain(value)).expect("decoded override count");
    }
    overrides.into_boxed()
}

pub(super) fn fault_words(fault: OverrideFault) -> Box<[u8]> {
    let (at, rule): (Option<u32>, &[u8]) = match fault {
        OverrideFault::Count => (None, b"more than 256 overrides"),
        OverrideFault::Name { at } => (Some(at), b"name must be 1 to 64 bytes of a-z, 0-9, _ or ."),
        OverrideFault::Order { at } => (Some(at), b"names must ascend"),
        OverrideFault::Reason { at } => (Some(at), b"reason must be 1 to 120 UTF-8 bytes"),
    };
    let mut words = List::uncapped();
    if let Some(at) = at {
        let number = Decimal::of(u64::from(at).checked_add(1).expect("bounded override position"));
        for part in [b"override ".as_slice(), number.as_bytes(), b": "] {
            for &byte in part {
                words.push(byte).expect("bounded fault words");
            }
        }
    }
    for &byte in rule {
        words.push(byte).expect("bounded fault words");
    }
    words.into_boxed()
}
