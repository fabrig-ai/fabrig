//! Reports, resets and wall-clock jumps keep bounded, ordered memory and a total rate.
use crate::rates::{ACCOUNT, LABEL, MINUTE, fresh, report, value};
use skein_lib::{Duration, Rng, Wall};

#[test]
fn seeded_window_reports_keep_bounded_ordered_readings_and_saturating_rates() {
    for seed in 0..32 {
        let mut rng = Rng::new(seed);
        let mut world = fresh(None);
        let mut at = MINUTE;
        for _ in 0..256 {
            at = match rng.below(8) {
                0 => rng.next_u64(),
                1 => at.saturating_sub(rng.below(120) * MINUTE),
                _ => at.saturating_add(rng.below(15) * MINUTE),
            };
            let mut window = value(Some(u16::try_from(rng.below(10_001)).unwrap()));
            window.length = if rng.chance(20) { None } else { Some(Duration::from_nanos(rng.next_u64())) };
            window.resets = if rng.chance(20) { None } else { Some(Wall::from_nanos(rng.next_u64())) };
            report(&mut world, at, window);
            let readings = world.domain.window_readings(ACCOUNT, LABEL);
            assert!(!readings.is_empty(), "seed {seed}");
            assert!(readings.len() <= 13, "seed {seed}");
            for pair in readings.windows(2) {
                assert!(pair[1].at.as_nanos() - pair[0].at.as_nanos() >= 5 * MINUTE, "seed {seed}");
                assert!(pair[1].share >= pair[0].share, "seed {seed}");
            }
            let newest = readings.last().unwrap();
            let oldest = readings.first().unwrap();
            let span = newest.at.as_nanos() - oldest.at.as_nanos();
            assert!(span <= 60 * MINUTE, "seed {seed}");
            if span >= 30 * MINUTE {
                let expected = u128::from(newest.share - oldest.share) * u128::from(60 * MINUTE) / u128::from(span);
                assert_eq!(
                    world.domain.window_rate(ACCOUNT, LABEL),
                    Some(u32::try_from(expected).unwrap()),
                    "seed {seed}"
                );
            }
        }
    }
}
