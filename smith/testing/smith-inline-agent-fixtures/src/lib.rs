//! Small slot bounds over a world's root configuration (smith/docs/design/domain/host.md, section 9.4).
//! Shared fixtures: smith/docs/design/testing.md, section 7.
#![no_std]

use skein_lib::Duration;
use smith_domain as smith;
use smith_inline_agent::Limits;

/// Two slots with eight maximum turns of credit, using the supplied root's bounds.
#[must_use]
pub fn small(smith: smith::Limits) -> Limits {
    Limits {
        slots: 2,
        smith,
        window: smith::Window {
            turns: 8,
            bytes: smith::max_turn_bytes(&smith).expect("root bound").checked_mul(8).expect("small window"),
        },
        wall_time: Duration::from_secs(3600),
        cancel_grace: Duration::from_secs(5),
    }
}
