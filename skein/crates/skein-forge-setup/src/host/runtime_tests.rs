//! Runtime membership (skein/docs/design/forge-setup.md, 6.3).
use super::*;

fn unknown(run: u64, forge: u16) -> Event {
    Event::Stop {
        run: Token::new(run),
        failure: Failure { forge, step: Stage::Admin, subject: b"reconcile".as_slice().into(), why: Why::Unexpected },
    }
}

#[test]
fn an_added_forge_serves_a_run() {
    for initial in [settings(&[]), settings(&[1])] {
        let mut w = World::with(limits(), initial);
        w.down(Request::Add(forge(2)));
        w.run(1, 2);
        assert_eq!(w.up.pop(), Some(owner_read(1)));
        w.answer(1, Answer::Read(Some(fixtures::secret(fixtures::ADMIN))));
        w.connect();
        assert!(w.up.is_empty());
    }
}

#[test]
fn remove_ends_its_runs_with_their_stop() {
    let l = Limits { runs: 3, ..limits() };
    let mut w = World::with(l, settings(&[1, 2]));
    w.run(1, 1);
    w.plan(2, reachable(1));
    w.run(3, 2);
    w.up.clear();
    w.answer(1, Answer::Read(Some(fixtures::secret(fixtures::ADMIN))));
    let owner = w.connect();
    w.event(skein_io::Event::Connecting { owner, socket: owner });
    w.down(Request::Remove { forge: 1 });
    assert_eq!(w.up.pop(), Some(stop(1, 1, Why::Down)));
    assert_eq!(w.up.pop(), Some(stop(2, 1, Why::Down)));
    assert!(w.up.is_empty());
    assert_eq!(w.io.pop(), Some(skein_io::Request::Abort { entity: owner }));
    w.event(skein_io::Event::Closed { owner });
    w.answer(1, Answer::Read(None));
    w.answer(2, Answer::Read(None));
    w.down(Request::Remove { forge: 1 });
    assert!(w.up.is_empty(), "each removed run ends once");
    w.answer(3, Answer::Read(None));
    assert_eq!(w.up.pop(), Some(stop(3, 2, Why::Missing)), "the other forge keeps serving its run");
    w.fire(120_000);
    assert!(w.up.is_empty());
    assert!(w.io.is_empty());
    assert_eq!(w.host.next_deadline(), None);
    w.run(4, 1);
    assert_eq!(w.up.pop(), Some(unknown(4, 1)));
    w.down(Request::Add(forge(1)));
    w.run(5, 1);
    assert_eq!(w.up.pop(), Some(owner_read(5)), "removal frees the forge slot");
}

#[test]
fn a_run_for_a_forge_not_held_ends_at_once() {
    let mut w = World::new();
    w.run(1, 9);
    assert_eq!(w.up.pop(), Some(unknown(1, 9)));
    w.answer(1, Answer::Read(None));
    w.fire(120_000);
    assert!(w.up.is_empty());
    assert!(w.io.is_empty());
    assert_eq!(w.host.next_deadline(), None);
}

#[test]
fn an_add_past_the_bound_is_ignored() {
    let mut w = World::with(limits(), settings(&[1]));
    w.down(Request::Add(forge(2)));
    w.down(Request::Add(forge(3)));
    w.run(1, 3);
    assert_eq!(w.up.pop(), Some(unknown(1, 3)));
    w.run(2, 1);
    w.run(3, 2);
    assert_eq!(w.up.pop(), Some(owner_read(2)));
    assert_eq!(w.up.pop(), Some(owner_read(3)));
    assert!(w.up.is_empty());
    assert!(w.io.is_empty());
}

#[test]
fn adding_a_held_number_keeps_its_settings_and_its_room() {
    let mut w = World::new();
    let mut replacement = forge(1);
    replacement.address = skein_io::Addr::Ip(([127, 0, 0, 1], 9999).into());
    w.down(Request::Add(replacement));
    w.down(Request::Add(forge(2)));
    w.send();
    w.connect();
    w.run(2, 2);
    assert_eq!(w.up.pop(), Some(owner_read(2)));
}

#[test]
fn removal_aborts_a_connection_before_io_names_its_socket() {
    let mut w = World::new();
    w.send();
    let owner = w.connect();
    w.down(Request::Remove { forge: 1 });
    assert_eq!(w.up.pop(), Some(stop(1, 1, Why::Down)));
    assert!(w.io.is_empty());
    w.event(skein_io::Event::Connecting { owner, socket: owner });
    assert_eq!(w.io.pop(), Some(skein_io::Request::Abort { entity: owner }));
    w.event(skein_io::Event::Closed { owner });
    w.fire(120_000);
    assert!(w.up.is_empty());
    assert!(w.io.is_empty());
}

#[test]
fn removal_aborts_a_settling_connection_whose_run_has_ended() {
    use skein_lib::stream::{Down, Up};
    let mut w = World::new();
    w.send();
    let owner = w.connect();
    w.event(skein_io::Event::Connecting { owner, socket: owner });
    w.event(skein_io::Event::Connected { owner });
    let Some(skein_io::Request::Stream { down: Down::Demand { .. }, .. }) = w.io.pop() else {
        panic!("stream demand");
    };
    w.event(skein_io::Event::Stream { owner, up: Up::Room });
    let Some(skein_io::Request::Stream { down: Down::Send(_), .. }) = w.io.pop() else {
        panic!("request head");
    };
    for line in [b"HTTP/1.1 401 Unauthorized\r\n".as_slice(), b"Content-Length: 0\r\n", b"\r\n"] {
        let Some(skein_io::Request::Stream { down: Down::Demand { .. }, .. }) = w.io.pop() else {
            panic!("stream demand");
        };
        w.event(skein_io::Event::Stream { owner, up: Up::Bytes(line.into()) });
    }
    assert_eq!(w.up.pop(), Some(stop(1, 1, Why::Refused(401))));
    assert_eq!(w.io.pop(), Some(skein_io::Request::Close { entity: owner }));
    w.down(Request::Remove { forge: 1 });
    assert_eq!(w.io.pop(), Some(skein_io::Request::Abort { entity: owner }));
    w.event(skein_io::Event::Closed { owner });
    assert!(w.io.is_empty());
    assert!(w.up.is_empty(), "the ended run gets no second terminal event");
}

#[test]
fn an_added_forge_with_invalid_settings_is_ignored() {
    let mut w = World::new();
    let mut invalid = forge(2);
    invalid.address = skein_io::Addr::Ip(([192, 0, 2, 1], 3000).into());
    w.down(Request::Add(invalid));
    w.run(1, 2);
    assert_eq!(w.up.pop(), Some(unknown(1, 2)));
    w.down(Request::Add(forge(2)));
    w.run(2, 2);
    assert_eq!(w.up.pop(), Some(owner_read(2)), "refusal leaves room for valid settings");
}
