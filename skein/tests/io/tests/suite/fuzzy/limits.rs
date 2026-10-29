//! Agreement outside semantic refusals and heap-total overflow (programming-model.md, 6.3).
use super::super::limits::draw;
use skein_io::{Limits, valid};
use skein_lib::{Duration, Rng};

#[test]
fn valid_and_heap_bound_agree_over_seeded_usable_limits() {
    // Bound sends and entities together so their heap product cannot overflow u64.
    // Alternate full-range sockets to cover derived-capacity refusals as well.
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
        assert_eq!(valid(&limits), super::super::heap_bound(&limits).is_some(), "draw {draw_index}: {limits:?}");
    }
}
