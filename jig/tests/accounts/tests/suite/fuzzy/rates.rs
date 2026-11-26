//! Reports, resets and wall-clock jumps keep bounded, ordered memory and a total rate.
use crate::rates::{ACCOUNT, LABEL, MINUTE, fresh, read_report, report, value};
use jig_domain_accounts::{ORIGIN_PROGRAM, ORIGIN_USAGE_READ};
use skein_lib::{Duration, Rng, Wall};

#[test]
fn seeded_window_reports_keep_bounded_ordered_readings_and_saturating_rates() {
    for seed in 0..32 {
        let mut rng = Rng::new(seed);
        let mut world = fresh(None);
        let mut at = MINUTE;
        for step in 0..256 {
            at = match rng.below(8) {
                0 => rng.next_u64(),
                1 => at.saturating_sub(rng.below(120) * MINUTE),
                _ => at.saturating_add(rng.below(15) * MINUTE),
            };
            let mut window = value(Some(u16::try_from(rng.below(10_001)).expect("the share fits")));
            window.length = if rng.chance(20) { None } else { Some(Duration::from_nanos(rng.next_u64())) };
            window.resets = if rng.chance(20) { None } else { Some(Wall::from_nanos(rng.next_u64())) };
            if step % 8 == 7 {
                window.used = window.used.map(|share| share / 100 * 100);
                read_report(&mut world, (step / 8 + 1) * 30, at, window);
            } else {
                report(&mut world, at, window);
            }
            let readings = world.domain.window_readings(ACCOUNT, LABEL);
            let newest = readings.last().expect("the first report establishes a reading");
            assert!(readings.len() <= 76, "seed {seed} step {step}");
            assert!(readings.is_sorted_by_key(|reading| reading.at), "seed {seed} step {step}");
            let oldest = &readings[0];
            assert!(newest.at.as_nanos() - oldest.at.as_nanos() <= 185 * MINUTE, "seed {seed} step {step}");
            for origin in [ORIGIN_PROGRAM, ORIGIN_USAGE_READ] {
                let mut same = readings.iter().filter(|reading| reading.origin == origin);
                assert!(same.clone().count() <= 38, "seed {seed} step {step} origin {origin}");
                if let Some(mut prior) = same.next() {
                    for reading in same {
                        assert!(reading.at.as_nanos() - prior.at.as_nanos() >= 5 * MINUTE, "seed {seed} step {step}");
                        assert!(reading.share >= prior.share, "seed {seed} step {step}");
                        if let (Some(before), Some(after)) = (prior.resets, reading.resets) {
                            assert_eq!(before, after, "seed {seed} step {step}");
                        }
                        prior = reading;
                    }
                }
            }
            let candidates = readings.iter().filter_map(|reading| {
                let age = newest.at.as_nanos() - reading.at.as_nanos();
                (reading.origin == newest.origin && age >= 60 * MINUTE).then_some((age, reading.share))
            });
            if let Some((age, share)) = candidates.min_by_key(|&(age, _)| (age.abs_diff(180 * MINUTE), age)) {
                let expected = u64::from(newest.share - share) * (60 * MINUTE) / age;
                assert_eq!(
                    world.domain.window_rate(ACCOUNT, LABEL),
                    Some(u32::try_from(expected).expect("an hour's rate fits")),
                    "seed {seed} step {step}"
                );
            }
        }
    }
}
