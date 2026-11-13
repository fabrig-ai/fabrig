//! The named opening refusal keeps its diagnostic words (redeploy.md, 4.7).
use temper_redeploy_tool::openings::{Event, parse};

#[test]
fn a_prefixed_opening_refusal_keeps_the_worker_and_its_text() {
    let line = "Nov 13 engine temper-engine serve: worker w2 refused at its opening: wires: jig-worker-wire: kept 2, speaks 3..4\n";
    assert_eq!(
        parse(line),
        Some(Event::Refused { worker: "w2".into(), text: "wires: jig-worker-wire: kept 2, speaks 3..4".into() })
    );
    assert_eq!(parse("temper-engine serve: a worker refused at its opening: wires: unknown"), None);
    assert_eq!(parse("temper-engine serve: worker lost worker=w2 channel=1: closed"), None);
    assert_eq!(parse("temper-engine serve: worker w2 opened: wire no-version"), None);
}
