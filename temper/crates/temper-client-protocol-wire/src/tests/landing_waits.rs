//! The added wait fields default when a client reads the former landing layouts.
use crate::{CLIENT_SCHEMA, v1 as w};
use skein_codec::{
    layout::{Mode, Plan, Workspace},
    schema::{self, Schema},
};
use skein_lib::{Buffer, List};

#[test]
fn the_previous_landing_layouts_read_without_wait_timing_or_a_line_wait() {
    let mut previous = Buffer::with_capacity(u32::try_from(CLIENT_SCHEMA.len()).unwrap());
    for line in CLIENT_SCHEMA.lines() {
        let added = if line.starts_with("record LandingWait {") {
            line.split_once(" since:")
        } else if line.starts_with("record LandingLine {") {
            line.split_once(" waits:")
        } else {
            None
        };
        if let Some((fields, _)) = added {
            previous.put(fields.as_bytes()).unwrap();
            previous.put(b" }\n").unwrap();
        } else {
            previous.put(line.as_bytes()).unwrap();
            previous.put(b"\n").unwrap();
        }
    }
    let old = schema::parse(previous.as_slice()).expect("previous wire");
    let current = schema::parse(CLIENT_SCHEMA.as_bytes()).expect("current wire");
    let wait = w::LandingWait {
        on: w::LandingWaitFor::Ready,
        change: Some(51),
        gate: None,
        run: None,
        landing: None,
        since: Some(2),
        bound: Some(w::LandingWaitBound::None(w::LandingWaitNone {
            reason: w::LandingWaitUnbounded::Behind(w::LandingWaitBehind { task: 51 }),
        })),
    };
    let mut panel = w::LandingPanel {
        waits: Some(wait.clone()),
        task: 61,
        project: 5,
        phase: w::LandingPhase::Waiting,
        hold: None,
        members: List::with_capacity(0),
        preparation: 1,
        stale_proofs: 0,
        turn: None,
        train: None,
        round: None,
    };
    let line = w::LandingLine {
        waits: Some(wait),
        step: Some(w::LandingStep::Waiting),
        last_event: Some(1),
        task: 61,
        phase: w::LandingPhase::Waiting,
        changes: List::from_boxed(Box::new([51])),
        repositories: List::with_capacity(0),
        landed: List::with_capacity(0),
        train: None,
    };
    let mut list = w::LandingsPanel {
        now: 3,
        turn_since: None,
        project: 5,
        turn: None,
        landings: List::from_boxed(Box::new([line])),
    };

    let answer = from_previous(b"LandingPanel", &old, &current, &crate::landing_panel(&panel));
    let wait = panel.waits.as_mut().unwrap();
    wait.since = None;
    wait.bound = None;
    assert_eq!(crate::decode_landing_panel(answer.as_slice(), &w::CEILINGS), Some(panel));

    let answer = from_previous(b"LandingsPanel", &old, &current, &crate::landings_panel(&list));
    list.landings.get_mut(0).unwrap().waits = None;
    assert_eq!(crate::decode_landings_panel(answer.as_slice(), &w::CEILINGS), Some(list));
}

fn from_previous(root: &[u8], old: &Schema, current: &Schema, bytes: &[u8]) -> Buffer {
    let old_root = old.root(root, None).unwrap();
    let current_root = current.root(root, None).unwrap();
    let writing = Plan::between(Mode::Read, current, current_root, &[], old, old_root, &[]).unwrap();
    let reading = Plan::between(Mode::Read, old, old_root, &[], current, current_root, &[]).unwrap();
    let mut work = Workspace::new();
    let mut previous = Buffer::with_capacity(256);
    assert!(work.fit(&writing));
    writing.convert(bytes, &mut previous, &mut work).unwrap();
    assert!(previous.as_slice().len() < bytes.len());
    assert!(work.fit(&reading));
    let mut answer = Buffer::with_capacity(256);
    reading.convert(previous.as_slice(), &mut answer, &mut work).unwrap();
    answer
}
