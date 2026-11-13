use crate::{Stated, Statement, Unfit, judge};
use jig_protocol_worker_wire::{Agreed, Spoken};
use skein_channel::{Direction, Kind, Schema, Term, Version};
use skein_lib::List;

const ROOM: u32 = 4096;
const CHANNEL: u16 = 42;

fn spoken() -> List<Spoken> {
    List::from_boxed(Box::new([
        Spoken::current("smith-charter", 7, [0xab; 32]),
        Spoken { name: "jig-worker-wire", lowest: 41, layouts: List::from_boxed(Box::new([[1; 32], [2; 32]])) },
    ]))
}

fn schema() -> Schema {
    Schema {
        magic: *b"test",
        versions: List::from_boxed(Box::new([Version {
            version: CHANNEL,
            kinds: List::from_boxed(Box::new([Kind { kind: 0x0180, direction: Direction::FromResponder, largest: 8 }])),
        }])),
    }
}

fn statement() -> Statement {
    Statement {
        wires: List::from_boxed(Box::new([
            Stated {
                name: "jig-worker-wire".into(),
                lowest: 41,
                layouts: List::from_boxed(Box::new([[1; 32], [2; 32]])),
            },
            Stated { name: "smith-charter".into(), lowest: 7, layouts: List::from_boxed(Box::new([[0xab; 32]])) },
        ])),
        output: ROOM,
        terms: List::from_boxed(Box::new([Term { kind: 0x0180, largest: 8 }])),
    }
}

#[test]
fn judge_agrees_in_statement_order_at_the_highest_common_versions() {
    assert_eq!(
        judge(&spoken(), ROOM, &schema(), &statement()),
        Ok(List::from_boxed(Box::new([
            Agreed { name: "jig-worker-wire", version: CHANNEL },
            Agreed { name: "smith-charter", version: 7 },
        ])))
    );
}

#[test]
fn judge_names_both_layouts_for_a_common_version() {
    let mut statement = statement();
    *statement.wires.get_mut(1).expect("charter").layouts.get_mut(0).expect("layout") = [0xcd; 32];
    assert_eq!(
        judge(&spoken(), ROOM, &schema(), &statement),
        Err(Unfit::Wires(Box::from(
            b"wire smith-charter version 7: the engine's layout abababababababab, the worker's cdcdcdcdcdcdcdcd"
                .as_slice()
        )))
    );
}

#[test]
fn judge_names_disjoint_ranges() {
    let mut statement = statement();
    statement.wires.get_mut(1).expect("charter").lowest = 6;
    assert_eq!(
        judge(&spoken(), ROOM, &schema(), &statement),
        Err(Unfit::Wires(Box::from(b"wire smith-charter: the engine speaks 7..7, the worker 6..6".as_slice())))
    );
}

#[test]
fn judge_names_a_missing_wire() {
    let mut statement = statement();
    statement.wires.truncate(1);
    assert_eq!(
        judge(&spoken(), ROOM, &schema(), &statement),
        Err(Unfit::Wires(Box::from(b"wire smith-charter: the worker states none".as_slice())))
    );
}

#[test]
fn judge_names_a_wire_stated_twice() {
    let mut statement = statement();
    let charter = statement.wires.get(1).expect("charter").clone();
    statement.wires = List::from_boxed(Box::new([charter.clone(), charter]));
    assert_eq!(
        judge(&spoken(), ROOM, &schema(), &statement),
        Err(Unfit::Wires(Box::from(b"wire smith-charter: stated twice".as_slice())))
    );
}

#[test]
fn judge_refuses_an_empty_or_overflowing_range() {
    for layouts in [List::with_capacity(0), List::from_boxed(Box::new([[0xab; 32], [0xab; 32]]))] {
        let mut statement = statement();
        let charter = statement.wires.get_mut(1).expect("charter");
        charter.lowest = u16::MAX;
        charter.layouts = layouts;
        assert_eq!(
            judge(&spoken(), ROOM, &schema(), &statement),
            Err(Unfit::Wires(Box::from(b"wire smith-charter: no such range".as_slice())))
        );
    }
}

#[test]
fn judge_names_unequal_output_rooms_and_the_setting() {
    let mut statement = statement();
    statement.output = 4095;
    assert_eq!(
        judge(&spoken(), ROOM, &schema(), &statement),
        Err(Unfit::Limits(Box::from(
            b"engine io.output 4096 bytes differs from worker io.output 4095 bytes; change io.output to agree"
                .as_slice()
        )))
    );
}

#[test]
fn judge_names_a_missing_or_short_term_at_the_agreed_channel_version() {
    for terms in [List::with_capacity(0), List::from_boxed(Box::new([Term { kind: 0x0180, largest: 7 }]))] {
        let mut statement = statement();
        statement.terms = terms;
        assert_eq!(judge(&spoken(), ROOM, &schema(), &statement), Err(Unfit::Limits(Box::from(b"384".as_slice()))));
    }
}

#[test]
fn judge_checks_wires_before_output_before_terms() {
    let mut statement = statement();
    statement.wires.get_mut(1).expect("charter").lowest = 6;
    statement.output = 4095;
    statement.terms = List::with_capacity(0);
    assert_eq!(
        judge(&spoken(), ROOM, &schema(), &statement),
        Err(Unfit::Wires(Box::from(b"wire smith-charter: the engine speaks 7..7, the worker 6..6".as_slice())))
    );
    statement.wires.get_mut(1).expect("charter").lowest = 7;
    assert_eq!(
        judge(&spoken(), ROOM, &schema(), &statement),
        Err(Unfit::Limits(Box::from(
            b"engine io.output 4096 bytes differs from worker io.output 4095 bytes; change io.output to agree"
                .as_slice()
        )))
    );
    statement.output = ROOM;
    assert_eq!(judge(&spoken(), ROOM, &schema(), &statement), Err(Unfit::Limits(Box::from(b"384".as_slice()))));
}
