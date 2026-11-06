use crate::Stage;
use crate::long_pass::{LONG_PASS, LongPass, Part, Watch};
use skein_lib::{Duration, Time};

fn at(ms: u64) -> Time {
    Time::ZERO.saturating_add(Duration::from_millis(ms))
}
fn pass(watch: &mut Watch, start: u64, took: u64, part: Part) -> Option<LongPass> {
    watch.begin(at(start));
    watch.ended(part, at(start).saturating_add(Duration::from_millis(took)));
    watch.pass_ended()
}
fn engine(took: u64, more: u32) -> LongPass {
    LongPass {
        took: Duration::from_millis(took),
        part: Part::Service(Stage::Engine),
        part_took: Duration::from_millis(took),
        more,
    }
}
fn after_first_line() -> Watch {
    let mut watch = Watch::new();
    assert_eq!(pass(&mut watch, 0, 1250, Part::Service(Stage::Engine)), Some(engine(1250, 0)));
    watch
}

#[test]
fn a_pass_of_the_bound_writes_nothing() {
    let mut watch = Watch::new();
    watch.begin(Time::ZERO);
    watch.ended(Part::Service(Stage::Engine), Time::ZERO.saturating_add(LONG_PASS));
    assert_eq!(watch.pass_ended(), None);
    assert_eq!(watch.held(), None);
}
#[test]
fn a_pass_past_the_bound_names_its_longest_part() {
    let mut watch = Watch::new();
    watch.begin(Time::ZERO);
    watch.ended(Part::Service(Stage::Io), at(10));
    watch.ended(Part::Service(Stage::Route), at(20));
    watch.ended(Part::Service(Stage::Engine), at(1220));
    watch.ended(Part::Service(Stage::Worker), at(1230));
    watch.ended(Part::Service(Stage::Out), at(1240));
    watch.ended(Part::Names, at(1245));
    watch.ended(Part::Drain, at(1250));
    let line = watch.pass_ended().expect("the first long pass writes at once");
    assert_eq!(
        line,
        LongPass { took: Duration::from_millis(1250), part_took: Duration::from_millis(1200), ..engine(0, 0) }
    );
    assert_eq!(line.part.name(), "engine");
    assert_eq!(watch.held(), None);

    let mut watch = Watch::new();
    watch.begin(at(50));
    watch.ended(Part::Names, at(50).saturating_add(LONG_PASS).saturating_add(Duration::from_nanos(1)));
    assert_eq!(watch.pass_ended().expect("one nanosecond past the bound writes").part.name(), "names");
}
#[test]
fn long_passes_in_the_window_are_held_as_the_longest_and_a_count() {
    let mut watch = after_first_line();
    for (start, took) in [(1500, 1400), (3000, 1250), (4500, 1600), (6500, 1100)] {
        assert_eq!(pass(&mut watch, start, took, Part::Service(Stage::Engine)), None);
    }
    assert_eq!(watch.held(), Some(engine(1600, 3)));
}
#[test]
fn the_first_pass_past_the_window_writes_what_is_held_long_or_not() {
    for (took, expected) in [(1, engine(1400, 0)), (1200, engine(1400, 1)), (1500, engine(1500, 1))] {
        let mut watch = after_first_line();
        assert_eq!(pass(&mut watch, 1500, 1400, Part::Service(Stage::Engine)), None);
        assert_eq!(pass(&mut watch, 11249, took, Part::Service(Stage::Engine)), Some(expected));
        assert_eq!(watch.held(), None);
        assert_eq!(pass(&mut watch, 13000, 1100, Part::Service(Stage::Engine)), None);
        assert_eq!(watch.held(), Some(engine(1100, 0)), "the returned line restarts the window");
    }
}
#[test]
fn what_is_held_can_be_read_at_the_end() {
    let mut watch = after_first_line();
    assert_eq!(pass(&mut watch, 2000, 1250, Part::Drain), None);
    let held = LongPass { part: Part::Drain, ..engine(1250, 0) };
    assert_eq!(watch.held(), Some(held));
    assert_eq!(watch.held(), Some(held), "reading at exit does not consume the line");
    watch.begin(at(4000));
    assert_eq!(watch.held(), Some(held), "begin keeps the held line");
}
#[test]
fn a_short_pass_holds_nothing() {
    let meter = skein_world::domain::heap::Meter::new();
    meter.start();
    let mut watch = Watch::new();
    assert_eq!(pass(&mut watch, 0, 999, Part::Service(Stage::Engine)), None);
    assert_eq!(watch.held(), None);
    assert_eq!(pass(&mut watch, 11000, 1, Part::Drain), None);
    assert_eq!(watch.held(), None);
    let measured = meter.end();
    assert_eq!(measured.peak(), 0, "short passes allocate nothing");
    assert_eq!(
        pass(&mut watch, 12000, 1250, Part::Service(Stage::Engine)),
        Some(engine(1250, 0)),
        "short passes do not start the window"
    );
}
