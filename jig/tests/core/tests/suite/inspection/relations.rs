//! Which sides of jig's relations must agree at a quiet point.

use jig_inspect::{HOST_CALLS, PLACEMENTS};
use skein_lib::Time;
use skein_world::inspect::{Inspection, Moment, Related, Relation, check_relations};

const CALL: &str = "task 12 attempt 3 completion 2 position 1";
const PLACE: &str = "run 12 attempt 3";
const MOMENT: Moment = Moment { seed: 7, at: Time::ZERO, iteration: 48 };

fn record(component: &'static str, relation: &'static str, key: &str, holds: &'static str) -> Inspection {
    let mut inspection = Inspection::new(component, &[]);
    inspection.related(Related { relation, key: key.into(), holds });
    inspection
}

fn check(relation: &Relation, inspections: &[&Inspection]) -> Result<(), String> {
    check_relations(inspections, &[relation], MOMENT)
}

#[test]
fn host_calls_match_open_calls_or_named_skews_and_find_orphans() {
    const RELATION: &str = "a run's host calls";
    let open = record("smith", RELATION, CALL, "an open call");
    let call = record("jig", RELATION, CALL, "a call record");
    let answer = record("jig", RELATION, CALL, "an answer retained for replay");
    let cancelled = record("jig", RELATION, CALL, "a cancelled run's call");
    assert_eq!(check(&HOST_CALLS, &[&open, &call]), Ok(()));
    assert_eq!(check(&HOST_CALLS, &[&open, &answer]), Ok(()));
    assert_eq!(check(&HOST_CALLS, &[&answer]), Ok(()));
    assert_eq!(check(&HOST_CALLS, &[&cancelled]), Ok(()));
    for component in ["jig", "jig host", "links"] {
        let relay = record(component, RELATION, CALL, "a relay in flight");
        assert_eq!(check(&HOST_CALLS, &[&open, &relay]), Ok(()), "{component}");
        assert_eq!(check(&HOST_CALLS, &[&call, &relay]), Ok(()), "{component}");
    }
    let orphan = check(&HOST_CALLS, &[&open]).expect_err("an open call needs its matching side");
    assert_eq!(
        orphan,
        format!(
            "seed 7: at 0 ns, iteration 48: smith {RELATION} {CALL}: holds an open call, matched by none of \
            jig's a call record, jig's an answer retained for replay, jig's a relay in flight, \
            jig host's a relay in flight, links' a relay in flight"
        )
    );
    assert_eq!(
        check(&HOST_CALLS, &[&call]).expect_err("a call record needs its matching side"),
        format!(
            "seed 7: at 0 ns, iteration 48: jig {RELATION} {CALL}: holds a call record, matched by none of \
            smith's an open call, jig's a relay in flight, jig host's a relay in flight, links' a relay in flight"
        )
    );
    let other = record("jig", RELATION, "task 12 attempt 3 completion 2 position 2", "a call record");
    assert_eq!(check(&HOST_CALLS, &[&open, &other]), Err(orphan));
}

#[test]
fn placements_match_slots_terminals_or_relays_and_find_orphans() {
    const RELATION: &str = "a run's placement";
    let placed = record("jig", RELATION, PLACE, "a placement");
    let hosted = record("jig host", RELATION, PLACE, "a hosted slot");
    let terminal = record("jig host", RELATION, PLACE, "a terminal awaiting its acknowledgement");
    let relay = record("links", RELATION, PLACE, "a relay in flight");
    for inspections in [[&placed, &hosted], [&placed, &terminal], [&hosted, &relay], [&placed, &relay]] {
        assert_eq!(check(&PLACEMENTS, &inspections), Ok(()));
    }
    assert_eq!(check(&PLACEMENTS, &[&terminal]), Ok(()));
    assert_eq!(
        check(&PLACEMENTS, &[&placed]).expect_err("a placement needs its matching side"),
        format!(
            "seed 7: at 0 ns, iteration 48: jig {RELATION} {PLACE}: holds a placement, matched by none of \
            jig host's a hosted slot, jig host's a terminal awaiting its acknowledgement, links' a relay in flight"
        )
    );
    assert_eq!(
        check(&PLACEMENTS, &[&hosted]).expect_err("a hosted slot needs its matching side"),
        format!(
            "seed 7: at 0 ns, iteration 48: jig host {RELATION} {PLACE}: holds a hosted slot, matched by none of \
            jig's a placement, links' a relay in flight"
        )
    );
}
