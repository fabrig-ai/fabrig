//! Seeded capacity admission checks (programming-model.md, 6.3).
use super::super::limits::draw;
use skein_io::{Limits, valid};
use skein_lib::{Duration, Rng};

#[test]
fn valid_checks_seeded_usable_limits_against_the_flight_capacity_edge() {
    // The flight table needs 8*sockets+4 slots in a u32; the other
    // derived capacities fit whenever it does. Alternate small and
    // full-range socket counts to cover admission and refusal.
    const LAST_FLIGHT_SOCKETS: u32 = 536_870_911;
    const DRAWS: u32 = 4096;
    const SMALL: u32 = 255;
    let mut rng = Rng::new(0x7067_7661_6c69_6401);
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
}
