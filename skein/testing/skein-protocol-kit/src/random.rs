//! Seeded draws below protocol machines (skein/docs/design/testing/README.md, 6).

use skein_lib::Rng;
use skein_lib::stream::Fault;

/// One draw among reset, invalid and other stream faults.
#[must_use]
pub fn fault(rng: &mut Rng) -> Fault {
    [Fault::Reset, Fault::Invalid, Fault::Other][usize::try_from(rng.below(3)).expect("fits a usize")]
}

/// One draw between the inclusive bounds.
#[must_use]
pub fn draw(rng: &mut Rng, low: u32, high: u32) -> u32 {
    u32::try_from(rng.between(u64::from(low), u64::from(high))).expect("fits a u32")
}
