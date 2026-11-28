//! Seeded capacity admission checks (`skein/docs/foundation/programming-model.md`, 6.3).
//! A case takes the whole draw stream (`skein/docs/design/testing/sweeps.md`, 4.2).

use std::convert::Infallible;

use super::super::limits::{LAST_FLIGHT_SOCKETS, draw};
use skein_io::{Limits, valid};
use skein_lib::{Duration, Rng};
use skein_world::sweep::cases;
use skein_world::this_test;

#[test]
fn valid_checks_seeded_usable_limits_against_the_flight_capacity_edge() {
    // The flight table needs 8*sockets+4 slots below the file lane
    // (runtime/io/files.md); the other derived capacities fit whenever it does.
    // Alternate small and full-range socket counts to cover admission and refusal.
    const DRAWS: u32 = 4096;
    const SMALL: u32 = 255;
    let test = this_test!("valid_checks_seeded_usable_limits_against_the_flight_capacity_edge");
    cases::<_, Infallible, _>(test, [0x7067_7661_6c69_6401], |seed| {
        let mut rng = Rng::new(seed);
        for draw_index in 0..DRAWS {
            let sockets = if draw_index % 2 == 0 { draw(&mut rng, SMALL) } else { draw(&mut rng, u32::MAX) };
            let limits = Limits {
                sockets,
                names: draw(&mut rng, u32::MAX),
                refusals: draw(&mut rng, u32::MAX).max(1),
                intake: draw(&mut rng, u32::MAX).max(1),
                receive: draw(&mut rng, u32::MAX).max(1),
                output: draw(&mut rng, u32::MAX).max(1),
                sends: draw(&mut rng, SMALL).max(1),
                accepts: draw(&mut rng, u32::MAX).max(1),
                backlog: draw(&mut rng, u32::MAX),
                close_timeout: Duration::from_nanos(rng.next_u64()),
                retry: Duration::from_nanos(rng.next_u64().max(1)),
            };
            assert_eq!(valid(&limits), sockets <= LAST_FLIGHT_SOCKETS, "draw {draw_index}: {limits:?}");
        }
        Ok(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}
