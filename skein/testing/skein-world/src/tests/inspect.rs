use crate::inspect::{Entry, Inspection, Moment, Refused, Release, Table, Wait, Wake, check_tables, check_waits};
use skein_lib::Time;

// The finding fixture from inspection.md, 6.
const MOMENT: Moment = Moment { seed: 7, at: Time::from_nanos(120_000_000), iteration: 48 };
const BEFORE: Time = Time::from_nanos(119_999_999);
const AFTER: Time = Time::from_nanos(120_000_001);
const WATCH: &str = "its watch";
const ANSWER: &str = "an answer";
const TASK_ONE: &str = "task 1";
const TASK_TWO: &str = "task 2";
const TASK_FOUR: &str = "task 4";
const TASK_NINE: &str = "task 9";
const TASK_TWELVE: &str = "task 12";
const PERSON: &str = "person 1";
const TABLE: &str = "forge write entries";
const KEY: &str = "write 3";
const NEWCOMER: &str = "write 4";
const RULE: &str = "the write's settlement";
// One occupied slot is full; removing its entry leaves room (inspection.md, 4.2).
const CAPACITY: u64 = 1;

fn wait(waiter: &str, kind: &str, wakes: Vec<Wake>, bound: Option<Time>) -> Wait {
    Wait { waiter: waiter.into(), kind: kind.into(), wakes, bound }
}

fn engine() -> Inspection {
    Inspection::new("engine", &[WATCH, ANSWER])
}

fn cycle(extra_wakes: Vec<Wake>) -> Inspection {
    let mut inspection = engine();
    inspection.wait(wait(TASK_ONE, WATCH, vec![Wake::Waiter(TASK_TWO.into())], None));
    let mut wakes = vec![Wake::Waiter(TASK_ONE.into())];
    wakes.extend(extra_wakes);
    inspection.wait(wait(TASK_TWO, ANSWER, wakes, None));
    inspection
}

fn entries(release: Release) -> Vec<Entry> {
    vec![Entry { key: KEY.into(), rule: RULE, release }]
}

fn table(entries: Vec<Entry>, refused: Vec<Refused>) -> Inspection {
    let mut inspection = engine();
    inspection.table(Table { name: TABLE, capacity: CAPACITY, entries, refused });
    inspection
}

fn refused(bound: Option<Time>) -> Vec<Refused> {
    vec![Refused { key: NEWCOMER.into(), since: BEFORE, bound }]
}

#[test]
fn cycle_is_no_wake_and_names_the_other_wait() {
    let inspection = cycle(Vec::new());
    assert_eq!(
        check_waits(&[&inspection], MOMENT),
        Err("seed 7: at 120000000 ns, iteration 48: engine task 1: waits on its watch with no live wake: [waiter task 2 (waits: an answer)]".into()),
    );
}

#[test]
fn timer_makes_a_cycle_live() {
    let inspection = cycle(vec![Wake::Timer(AFTER)]);
    assert_eq!(check_waits(&[&inspection], MOMENT), Ok(()));
}

#[test]
fn ended_source_is_not_a_live_wake() {
    let mut inspection = engine();
    inspection.wait(wait(TASK_TWELVE, WATCH, vec![Wake::Ended(TASK_NINE.into())], None));
    assert_eq!(
        check_waits(&[&inspection], MOMENT),
        Err("seed 7: at 120000000 ns, iteration 48: engine task 12: waits on its watch with no live wake: [ended task 9]".into()),
    );
}

#[test]
fn undeclared_kind_is_the_first_wait_finding() {
    let mut inspection = engine();
    // Also lacks a live wake and exceeds its bound; the kind comes first.
    inspection.wait(wait(TASK_TWELVE, "an undocumented wait", Vec::new(), Some(BEFORE)));
    assert_eq!(
        check_waits(&[&inspection], MOMENT),
        Err("seed 7: at 120000000 ns, iteration 48: engine task 12: undeclared wait kind: an undocumented wait".into()),
    );
}

#[test]
fn wait_past_its_bound_is_a_finding() {
    let mut inspection = engine();
    inspection.wait(wait(TASK_TWELVE, WATCH, vec![Wake::Timer(AFTER)], Some(BEFORE)));
    assert_eq!(
        check_waits(&[&inspection], MOMENT),
        Err("seed 7: at 120000000 ns, iteration 48: engine task 12: waits on its watch past bound 119999999 ns".into()),
    );
}

#[test]
fn wait_within_its_bound_passes() {
    for bound in [None, Some(MOMENT.at), Some(AFTER)] {
        let mut inspection = engine();
        inspection.wait(wait(TASK_TWELVE, WATCH, vec![Wake::Timer(AFTER)], bound));
        assert_eq!(check_waits(&[&inspection], MOMENT), Ok(()));
    }
}

#[test]
fn waiter_names_resolve_across_inspections_and_all_their_waits() {
    let mut engine = engine();
    engine.wait(wait(TASK_ONE, WATCH, vec![Wake::Waiter(TASK_FOUR.into())], None));
    engine.wait(wait(TASK_FOUR, WATCH, vec![Wake::Waiter(TASK_ONE.into())], None));
    let mut jig = Inspection::new("jig", &[ANSWER]);
    jig.wait(wait(TASK_FOUR, ANSWER, vec![Wake::Party(PERSON.into())], None));
    assert_eq!(check_waits(&[&engine, &jig], MOMENT), Ok(()));
}

#[test]
fn waiter_with_no_wait_is_live() {
    let mut inspection = engine();
    inspection.wait(wait(TASK_ONE, WATCH, vec![Wake::Waiter(TASK_FOUR.into())], None));
    assert_eq!(check_waits(&[&inspection], MOMENT), Ok(()));
}

#[test]
fn finding_lists_every_wake_and_referenced_wait_kind() {
    let mut engine = engine();
    engine.wait(wait(TASK_TWELVE, WATCH, vec![Wake::Waiter(TASK_FOUR.into()), Wake::Ended(TASK_NINE.into())], None));
    let mut jig = Inspection::new("jig", &[WATCH, ANSWER]);
    jig.wait(wait(TASK_FOUR, WATCH, vec![Wake::Waiter(TASK_TWELVE.into())], None));
    jig.wait(wait(TASK_FOUR, ANSWER, Vec::new(), None));
    assert_eq!(
        check_waits(&[&engine, &jig], MOMENT),
        Err("seed 7: at 120000000 ns, iteration 48: engine task 12: waits on its watch with no live wake: [waiter task 4 (waits: its watch, an answer); ended task 9]".into()),
    );
}

#[test]
fn findings_follow_inspection_then_wait_order() {
    let mut engine = engine();
    engine.wait(wait(TASK_TWELVE, WATCH, Vec::new(), None));
    engine.wait(wait(TASK_ONE, WATCH, Vec::new(), None));
    let mut jig = Inspection::new("jig", &[WATCH]);
    jig.wait(wait(TASK_NINE, WATCH, Vec::new(), None));
    assert_eq!(
        check_waits(&[&engine, &jig], MOMENT),
        Err("seed 7: at 120000000 ns, iteration 48: engine task 12: waits on its watch with no live wake: []".into()),
    );
}

#[test]
fn due_entry_names_table_key_and_release_rule() {
    let inspection = table(entries(Release::Due), Vec::new());
    assert_eq!(
        check_tables(&[&inspection], MOMENT),
        Err("seed 7: at 120000000 ns, iteration 48: engine forge write entries write 3: still held after release rule: the write's settlement".into()),
    );
}

#[test]
fn kept_and_pending_entries_pass() {
    for release in [Release::Kept, Release::Pending] {
        let inspection = table(entries(release), Vec::new());
        assert_eq!(check_tables(&[&inspection], MOMENT), Ok(()));
    }
}

#[test]
fn refused_newcomer_with_room_is_a_finding() {
    for bound in [None, Some(AFTER)] {
        let inspection = table(Vec::new(), refused(bound));
        assert_eq!(
            check_tables(&[&inspection], MOMENT),
            Err("seed 7: at 120000000 ns, iteration 48: engine forge write entries write 4: refused admission while the table has room".into()),
        );
    }
}

#[test]
fn refused_newcomer_past_its_bound_is_a_finding() {
    let inspection = table(entries(Release::Pending), refused(Some(BEFORE)));
    assert_eq!(
        check_tables(&[&inspection], MOMENT),
        Err("seed 7: at 120000000 ns, iteration 48: engine forge write entries write 4: refused admission past bound 119999999 ns".into()),
    );
}

#[test]
fn full_table_can_refuse_within_the_bound() {
    for bound in [None, Some(MOMENT.at), Some(AFTER)] {
        let inspection = table(entries(Release::Pending), refused(bound));
        assert_eq!(check_tables(&[&inspection], MOMENT), Ok(()));
    }
}

#[test]
fn finding_has_seed_nanoseconds_iteration_component_and_object() {
    assert_eq!(
        engine().finding(MOMENT, TASK_TWELVE, "no live wake"),
        "seed 7: at 120000000 ns, iteration 48: engine task 12: no live wake",
    );
}
