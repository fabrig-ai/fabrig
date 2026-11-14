use jig_client_views_world::World;
use jig_domain_views::{Event, Follow, Request, TurnName};
use skein_lib::Token;
#[test]
fn a_second_delivery_in_flight_fails_the_referee() {
    let mut world = World::default();
    world.watch(1, 7);
    world.referee.check_holding(1).expect("valid delivery in flight");
    world.referee.saw(&[Request::Deliver { watcher: Token::new(1) }]);
    let why = world.referee.check_holding(1).expect_err("second delivery in flight");
    assert!(why.contains("one document in flight"), "{why}");
}
#[test]
fn ending_before_the_delivery_terminal_fails_the_referee() {
    let mut world = World::default();
    world.watch(1, 7);
    world.referee.check_holding(1).expect("valid delivery in flight");
    world.referee.saw(&[Request::Ended { watcher: Token::new(1), end: jig_domain_views::End::Gone }]);
    let why = world.referee.check_holding(1).expect_err("end before delivery terminal");
    assert!(why.contains("end follows every outstanding terminal"), "{why}");
}
#[test]
fn a_document_without_an_owner_construction_fails_the_referee() {
    let mut world = World::default();
    world.watch(1, 7);
    world.delivered(1);
    world.referee.check_holding(1).expect("valid completed delivery");
    world.referee.saw(&[Request::Deliver { watcher: Token::new(1) }]);
    let why = world.referee.check_holding(1).expect_err("delivery without owner construction");
    assert!(why.contains("owner made a document"), "{why}");
}
#[test]
fn a_missed_final_change_fails_the_referee() {
    let mut world = World::default();
    world.watch(1, 7);
    world.referee.check_caught_up().expect("initial delivery is current");
    world.changed(7);
    world.referee.check_holding(1).expect("valid pending final change");
    let why = world.referee.check_caught_up().expect_err("missed final change");
    assert!(why.contains("last delivery follows the final subject change"), "{why}");
}
#[test]
fn corrupted_provisional_bytes_fail_the_referee() {
    let mut world = World::default();
    let turn = TurnName { attempt: 1, turn: 1, retry: 0 };
    world.watch_from(1, 7, Some(Follow { run: Token::new(7), turn: None, offset: 0 }));
    world.text(7, turn, b"abc", None);
    world.stream_delivered(1);
    world.referee.before(&Event::Text {
        run: Token::new(7),
        turn,
        offset: 3,
        text: b"def".as_slice().into(),
        call: None,
    });
    world.referee.check_holding(1).expect("valid provisional source and completed stream");
    world.referee.saw(&[Request::Stream {
        writing: None,
        watcher: Token::new(1),
        turn,
        from: 3,
        to: 6,
        text: b"xyz".as_slice().into(),
        cut: false,
        call: None,
    }]);
    let why = world.referee.check_holding(1).expect_err("corrupted provisional bytes");
    assert!(why.contains("provisional bytes are a prefix-consistent source copy"), "{why}");
}
