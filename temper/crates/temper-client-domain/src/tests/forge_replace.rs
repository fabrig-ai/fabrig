use crate::tests::{connect, fixtures as f};
use crate::{
    Domain, Event, Request,
    forge::{self, replace::Field},
};
use jig_client::{Down, Up};
use skein_lib::{List, Queue, Secret, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};
const FORGE: u16 = 3;
fn event(root: &mut Domain, value: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::step(root, &f::env(&f::limits()), value, &mut out);
    out
}
fn act(root: &mut Domain, act: forge::Act) -> Queue<Request> {
    event(root, Event::Forge(forge::Event::Act { act }))
}
fn paste(root: &mut Domain, field: Field, bytes: &[u8]) {
    event(root, Event::Forge(forge::Event::ReplaceValue { forge: FORGE, field, value: Secret::new(bytes.into()) }));
}
fn bytes(seed: u64, length: usize) -> Box<[u8]> {
    let mut rng = skein_lib::Rng::new(seed);
    let mut bytes = skein_lib::bytes::zeroed(length);
    for byte in &mut bytes {
        *byte = b'a'.checked_add(u8::try_from(rng.next_u64() % 26).unwrap()).unwrap();
    }
    bytes
}
fn row(kind: w::Kind, mode: w::Mode, connection: w::Connection, ended: bool) -> w::ForgeRow {
    w::ForgeRow::new(
        &w::CEILINGS,
        w::ForgeRowParts {
            number: FORGE,
            kind,
            mode,
            connection,
            offer: w::Offer::NotOffered,
            first: None,
            installations: List::with_capacity(0),
            ended: if ended {
                Some(
                    w::ConnectStopped::new(
                        &w::CEILINGS,
                        w::ConnectStoppedParts { why: b"tokens refused".as_slice().into() },
                    )
                    .unwrap(),
                )
            } else {
                None
            },
            app: None,
            web: None,
            repositories: List::with_capacity(0),
            owned: None,
            admin: false,
            read_only: false,
            ended_stop: None,
        },
    )
    .unwrap()
}
fn deliver(root: &mut Domain, watch: Token, row: w::ForgeRow) {
    let mut rows = List::with_capacity(1);
    rows.push(row).unwrap();
    deliver_rows(root, watch, rows);
}
fn deliver_rows(root: &mut Domain, watch: Token, rows: List<w::ForgeRow>) {
    event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::forges(&w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: rows }).unwrap()),
                    },
                },
            },
        },
    );
}
fn opened() -> (Domain, Token) {
    let (mut root, _, watch) = connect::opened(true);
    deliver(
        &mut root,
        watch,
        row(
            w::Kind::Forgejo,
            w::Mode::Connected,
            w::Connection::Connected(
                w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 7, login: b"builder".as_slice().into() })
                    .unwrap(),
            ),
            false,
        ),
    );
    act(&mut root, forge::Act::Forges(forge::forges::Act::OpenReplace { forge: FORGE }));
    (root, watch)
}
fn send(root: &mut Domain) -> Queue<Request> {
    act(root, forge::Act::Forges(forge::forges::Act::Replace { forge: FORGE }))
}
#[test]
fn replacement_bundles_carry_only_the_new_values() {
    for (kind, fields, codes) in [
        (w::Kind::Forgejo, &[Field::Git][..], &[3][..]),
        (w::Kind::Forgejo, &[Field::Api, Field::Git, Field::Webhook][..], &[2, 3, 5][..]),
        (w::Kind::Github, &[Field::Key][..], &[11][..]),
    ] {
        let (mut root, watch) = opened();
        deliver(
            &mut root,
            watch,
            row(
                kind,
                w::Mode::Connected,
                w::Connection::Stopped(
                    w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: Box::default() }).unwrap(),
                ),
                false,
            ),
        );
        let value = bytes(11, 32);
        for field in fields {
            paste(&mut root, *field, &value);
        }
        let mut sent = 0_u32;
        let mut out = send(&mut root);
        while let Some(request) = out.pop() {
            if let Request::Client {
                request:
                    jig_client::Request::Down {
                        down: Down::Send { request: jig_client::Ask::Family { record, .. }, value: Some(bundle), .. },
                    },
            } = request
            {
                let items = temper_protocol_records::decode_pasted(bundle.expose()).unwrap();
                assert_eq!(items.len(), codes.len());
                for ((code, _), expected) in items.iter().zip(codes) {
                    assert_eq!(code, expected);
                }
                for (_, secret) in &items {
                    assert!(secret.expose() == value.as_ref());
                }
                let w::Request::Replace(replace) =
                    wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request
                else {
                    panic!("replace")
                };
                assert_eq!(replace.forge(), FORGE);
                sent += 1;
            }
        }
        assert_eq!(sent, 1);
        for field in fields {
            assert!(!root.forge().replacement(FORGE).unwrap().has(*field));
        }
        assert!(send(&mut root).is_empty());
    }
}
#[test]
fn empty_and_oversized_replacements_send_nothing() {
    for fields in [&[][..], &[Field::Webhook][..], &[Field::Api, Field::Git][..]] {
        let (mut root, _) = opened();
        for field in fields {
            paste(&mut root, *field, &bytes(7, temper_protocol_records::PASTED_PAYLOAD));
        }
        assert!(send(&mut root).is_empty());
        assert!(root.forge().replacement(FORGE).unwrap().checked);
    }
}
#[test]
fn leaving_and_a_stop_clear_replacement_secrets() {
    let (mut root, watch) = opened();
    paste(&mut root, Field::Git, &bytes(7, 32));
    deliver(
        &mut root,
        watch,
        row(
            w::Kind::Forgejo,
            w::Mode::Connected,
            w::Connection::Stopped(w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: Box::default() }).unwrap()),
            true,
        ),
    );
    assert!(!root.forge().replacement(FORGE).unwrap().has(Field::Git));
    paste(&mut root, Field::Git, &bytes(7, 32));
    event(&mut root, Event::Forge(forge::Event::Left));
    assert!(root.forge().replacement(FORGE).is_none());
}

fn connected() -> w::Connection {
    w::Connection::Connected(
        w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 7, login: b"builder".as_slice().into() }).unwrap(),
    )
}
fn attempt(out: &Queue<Request>) -> Token {
    for request in out {
        if let Request::Client { request: jig_client::Request::Down { down: Down::Send { attempt, .. } } } = request {
            return *attempt;
        }
    }
    panic!("sent request")
}
fn answer(root: &mut Domain, attempt: Token, outcome: w::Outcome) {
    event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Family(jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap()),
                    })),
                    why: None,
                },
            },
        },
    );
}
#[test]
fn replacements_decode_connecting_refused_and_unknown_answers() {
    use forge::github::Answer;
    for (outcome, expected) in [
        (w::Outcome::Connecting, Answer::Connecting),
        (w::Outcome::Unknown, Answer::Unknown),
        (
            w::Outcome::ConnectRefused(
                w::ConnectRefused::new(&w::CEILINGS, w::ConnectRefusedParts { why: w::ForgeRefusal::NotConnected })
                    .unwrap(),
            ),
            Answer::Refused(w::ForgeRefusal::NotConnected),
        ),
    ] {
        let (mut root, _) = opened();
        paste(&mut root, Field::Git, &bytes(1, 32));
        let out = send(&mut root);
        answer(&mut root, attempt(&out), outcome);
        assert_eq!(root.forge().replacement(FORGE).unwrap().answer, Some(expected));
        assert!(!root.forge().replacement(FORGE).unwrap().has(Field::Git));
    }
}
#[test]
fn replacement_checks_the_rows_offer_again_before_sending() {
    for (kind, mode, connection) in [
        (w::Kind::Forgejo, w::Mode::Managed, connected()),
        (w::Kind::Forgejo, w::Mode::Owned, connected()),
        (w::Kind::Forgejo, w::Mode::Connected, w::Connection::NotConnected),
        (w::Kind::Github, w::Mode::Connected, w::Connection::NotConnected),
        (w::Kind::Forgejo, w::Mode::Connected, w::Connection::Connecting),
    ] {
        let (mut root, watch) = opened();
        paste(&mut root, Field::Git, &bytes(1, 32));
        deliver(&mut root, watch, row(kind, mode, connection, false));
        assert!(send(&mut root).is_empty());
    }
}
#[test]
fn replacement_bundle_bound_is_checked_with_its_framing() {
    let overhead = temper_protocol_records::encode_pasted(&[(2, &[1]), (3, &[2])]).unwrap().len() - 2;
    for (extra, accepted) in [(0, true), (1, false)] {
        let (mut root, _) = opened();
        paste(&mut root, Field::Api, &bytes(1, temper_protocol_records::PASTED_BYTES - overhead - 1 + extra));
        paste(&mut root, Field::Git, &bytes(2, 1));
        assert_eq!(!send(&mut root).is_empty(), accepted);
    }
}
#[test]
fn a_key_replacement_restores_armour_and_refuses_empty_keys() {
    let (mut root, watch) = opened();
    deliver(&mut root, watch, row(w::Kind::Github, w::Mode::Connected, connected(), false));
    for value in [b"".as_slice(), b" \t\r\n"] {
        paste(&mut root, Field::Key, value);
        assert!(send(&mut root).is_empty());
    }
    let body = bytes(7, 32);
    let mut input = skein_lib::Writer::new(
        b"-----BEGIN RSA PRIVATE KEY-----".len() + body.len() + b"-----END RSA PRIVATE KEY-----".len(),
    );
    input.put(b"-----BEGIN RSA PRIVATE KEY-----").unwrap();
    input.put(&body).unwrap();
    input.put(b"-----END RSA PRIVATE KEY-----").unwrap();
    paste(&mut root, Field::Key, &input.finish());
    let mut out = send(&mut root);
    let mut count = 0_u32;
    while let Some(request) = out.pop() {
        if let Request::Client { request: jig_client::Request::Down { down: Down::Send { value: Some(value), .. } } } =
            request
        {
            let items = forge::forges::decode_pasted(value.expose()).unwrap();
            assert_eq!(items.len(), 1);
            for (code, secret) in &items {
                assert_eq!(*code, 11);
                assert!(secret.expose().starts_with(b"-----BEGIN RSA PRIVATE KEY-----\n"));
                assert!(secret.expose().ends_with(b"\n-----END RSA PRIVATE KEY-----"));
                assert!(skein_lib::bytes::find(secret.expose(), &body).is_some());
            }
            count += 1;
        }
    }
    assert_eq!(count, 1);
}
#[test]
fn closing_replacement_erases_its_fields_and_reopens_fresh() {
    let (mut root, _) = opened();
    paste(&mut root, Field::Git, &bytes(7, 32));
    act(&mut root, forge::Act::Forges(forge::forges::Act::CloseReplace { forge: FORGE }));
    assert!(root.forge().replacement(FORGE).is_none());
    act(&mut root, forge::Act::Forges(forge::forges::Act::OpenReplace { forge: FORGE }));
    assert!(!root.forge().replacement(FORGE).unwrap().has(Field::Git));
}

#[test]
fn each_row_keeps_its_replacement_and_its_request_name() {
    const OTHER: u16 = FORGE + 1;
    let (mut root, watch) = opened();
    let mut rows = List::with_capacity(2);
    for number in [FORGE, OTHER] {
        let mut parts = row(w::Kind::Forgejo, w::Mode::Connected, connected(), false).into_parts();
        parts.number = number;
        rows.push(w::ForgeRow::new(&w::CEILINGS, parts).unwrap()).unwrap();
    }
    deliver_rows(&mut root, watch, rows);
    act(&mut root, forge::Act::Forges(forge::forges::Act::OpenReplace { forge: OTHER }));
    for number in [FORGE, OTHER] {
        event(
            &mut root,
            Event::Forge(forge::Event::ReplaceValue {
                forge: number,
                field: Field::Git,
                value: Secret::new(bytes(u64::from(number), 32)),
            }),
        );
    }
    let mut names = List::with_capacity(2);
    for number in [FORGE, OTHER] {
        let mut out = act(&mut root, forge::Act::Forges(forge::forges::Act::Replace { forge: number }));
        while let Some(request) = out.pop() {
            if let Request::Client {
                request: jig_client::Request::Down { down: Down::Send { key: name, value: Some(value), .. } },
            } = request
            {
                names.push(name).unwrap();
                for (code, value) in &forge::forges::decode_pasted(value.expose()).unwrap() {
                    assert_eq!(*code, 3);
                    assert!(value.expose() == bytes(u64::from(number), 32).as_ref());
                }
            }
        }
    }
    assert_eq!(names.len(), 2);
    assert_ne!(names.get(0), names.get(1));
}
#[test]
fn one_key_fits_at_the_payload_bound_and_refuses_one_past() {
    for (extra, accepted) in [(0, true), (1, false)] {
        let (mut root, watch) = opened();
        deliver(&mut root, watch, row(w::Kind::Github, w::Mode::Connected, connected(), false));
        paste(&mut root, Field::Key, &bytes(7, forge::forges::PASTED_PAYLOAD + extra));
        assert_eq!(!send(&mut root).is_empty(), accepted);
    }
}

#[test]
fn a_connecting_answer_yields_to_the_completed_watch() {
    let (mut root, watch) = opened();
    paste(&mut root, Field::Git, &bytes(7, 32));
    let out = send(&mut root);
    answer(&mut root, attempt(&out), w::Outcome::Connecting);
    assert!(root.forge().replacement(FORGE).unwrap().connecting());
    deliver(&mut root, watch, row(w::Kind::Forgejo, w::Mode::Connected, w::Connection::Connecting, false));
    deliver(&mut root, watch, row(w::Kind::Forgejo, w::Mode::Connected, connected(), false));
    assert!(!root.forge().replacement(FORGE).unwrap().connecting());
}
