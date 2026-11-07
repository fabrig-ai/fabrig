//! Every representable wall day swept through calendar and text round trips (testing-strategy.md, 8).

use skein_lib::Wall;
use skein_ui::date::{civil, format, parse, wall};

#[test]
fn every_wall_day_round_trips_at_midnight() {
    let day_nanos = 86_400_000_000_000;
    for elapsed in 0..=u64::MAX / day_nanos {
        let midnight = Wall::from_nanos(elapsed * day_nanos);
        let date = civil(midnight).unwrap();
        assert_eq!(wall(date), Some(midnight));
        assert_eq!(parse(&format(date)), Some(date));
        let end = Wall::from_nanos(midnight.as_nanos().saturating_add(day_nanos - 1));
        assert_eq!(civil(end), Some(date));
    }
}

