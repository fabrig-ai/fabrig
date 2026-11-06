//! Delivery-log admission bounds.
use super::fixtures as f;
use crate::Limits;
use skein_lib::Duration;

#[test]
fn log_limits_are_positive_and_smallest_values_are_admitted() {
    for limits in [
        Limits { log_poll: Duration::ZERO, ..f::limits() },
        Limits { log_catch_up: Duration::ZERO, ..f::limits() },
        Limits { log_pages: 0, ..f::limits() },
        Limits { delivery_bytes: 0, ..f::limits() },
    ] {
        f::admission(f::github_settings(), &limits, Err(crate::Unusable::Zero), b"zero log limit");
    }
    let limits = Limits {
        log_poll: Duration::from_nanos(1),
        log_catch_up: Duration::from_nanos(1),
        log_pages: 1,
        delivery_bytes: 1,
        ..f::limits()
    };
    f::admission(f::github_settings(), &limits, Ok(()), b"positive log limits");
}
