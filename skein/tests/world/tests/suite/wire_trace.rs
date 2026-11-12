//! A wire's steps retain their text, progress, replay and heap contracts
//! (testing-strategy.md, sections 2.5 and 6).

use skein_heap::Meter;
use skein_lib::stream::{Down, Read, Up};
use skein_world::stream::Wire;

// Two fragments, then a delayed room before its grant.
const SEED: u64 = 1034;

fn answer(wire: &mut Wire, inspect: fn(&Wire)) -> Up {
    for _ in 0..1000 {
        let answer = wire.answer();
        inspect(wire);
        if let Some(answer) = answer {
            return answer;
        }
    }
    panic!("the seeded demand is answered")
}

fn scenario(bytes: &[u8], inspect: fn(&Wire)) -> Wire {
    let mut wire = Wire::new(SEED);
    inspect(&wire);
    wire.write(bytes);
    wire.take(Down::Demand { read: Read::Fill(3), room: 0 });
    inspect(&wire);
    drop(answer(&mut wire, inspect));
    wire.take(Down::Demand { read: Read::Nothing, room: 4 });
    inspect(&wire);
    drop(answer(&mut wire, inspect));
    wire.take(Down::Send(Box::from([1, 2])));
    inspect(&wire);
    wire.take(Down::Finish);
    inspect(&wire);
    wire.eof = true;
    wire.take(Down::Demand { read: Read::Arrived { max: 1 }, room: 0 });
    inspect(&wire);
    drop(answer(&mut wire, inspect));
    wire.trace.push("owner closed".to_owned());
    inspect(&wire);
    wire
}

#[test]
fn a_wire_prints_each_step_as_it_went() {
    let wire = scenario(b"abc", |_| {});
    assert_eq!(
        wire.trace.lines(),
        [
            "down Demand { read: Fill(3), room: 0 }",
            "fragment [97, 98]",
            "fragment [99]",
            "up Bytes([97, 98, 99])",
            "down Demand { read: Nothing, room: 4 }",
            "room delayed",
            "up Room",
            "down Send([1, 2])",
            "down Finish",
            "down Demand { read: Arrived { max: 1 }, room: 0 }",
            "up End",
            "owner closed",
        ]
    );
    assert_eq!(format!("{:?}", wire.trace), format!("{:?}", wire.trace.lines()));
}

#[test]
fn a_trace_counts_compares_and_appends_its_steps() {
    let inspect = |wire: &Wire| {
        let lines = wire.trace.lines();
        assert_eq!(wire.trace.len(), lines.len());
        assert_eq!(wire.trace.is_empty(), lines.is_empty());
    };
    let mut trace = scenario(b"abc", inspect).trace;
    assert_eq!(trace, scenario(b"abc", inspect).trace);
    let mut other = scenario(b"abd", inspect).trace;
    assert_ne!(trace, other, "replay compares byte values, even at equal lengths");
    let expected = [trace.lines(), other.lines()].concat();
    trace.append(&mut other);
    assert!(other.is_empty());
    assert_eq!(other.lines().len(), 0);
    assert_eq!(trace.lines(), expected);
    assert_eq!(trace.len(), expected.len());
    let cloned = trace.clone();
    trace.clear();
    assert!(trace.is_empty());
    assert_eq!(trace.lines().len(), 0);
    assert_eq!(cloned.lines(), expected);
}

#[test]
fn a_cleared_trace_holds_no_heap() {
    let mut wire = Wire::new(SEED);
    let meter = Meter::new();
    wire.take(Down::Demand { read: Read::Nothing, room: 4 });
    drop(answer(&mut wire, |_| {}));
    wire.take(Down::Send(Box::from([1, 2])));
    wire.trace.push("owner closed".to_owned());
    wire.trace.clear();
    assert_eq!(meter.held(), u64::try_from(wire.received.capacity()).expect("capacity fits"));
}
