//! Every worker restart kind at its named process boundary.
use skein_lib::{Duration, Time};
use skein_world::faults::Struck;
use temper_faults::restarts::{Point, Way, worker};
use temper_worker_domain_world::whole::restarts::{Recovery, at_point};

#[test]
fn workers_stop_at_each_turn_point_and_crashes_resume_on_same_or_another_worker() {
    for way in [Way::Drained, Way::Signalled, Way::Crashed] {
        for point in [Point::LlmCall, Point::ToolCall, Point::DeliveryCheck, Point::Save] {
            let recoveries = if way == Way::Crashed {
                &[Recovery::SameWorker, Recovery::AnotherWorker][..]
            } else {
                &[Recovery::SameWorker][..]
            };
            for &recovery in recoveries {
                let ledger = at_point(71, way, point, recovery);
                let [entry] = ledger.entries() else { panic!("one restart: {ledger:?}") };
                assert_eq!(entry.kind, worker(way, point));
                let from = Time::from_nanos(7_000_000_000);
                assert_eq!(entry.from, from);
                let until = match way {
                    Way::Drained => Some(from.saturating_add(if point == Point::DeliveryCheck {
                        temper_worker_domain_world::limits().drain
                    } else {
                        Duration::from_secs(1)
                    })),
                    Way::Signalled => None,
                    Way::Crashed => Some(from.saturating_add(Duration::from_secs(2))),
                };
                assert_eq!(entry.until, until, "{way:?} {point:?} {recovery:?}");
                assert_eq!(entry.struck, Struck::default());
            }
        }
    }
}
