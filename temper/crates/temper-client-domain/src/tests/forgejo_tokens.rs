use crate::tests::{connect, fixtures as f};
use crate::{
    Choice, Domain, Event, Request,
    forge::{self, tokens::Field},
};
use jig_client::{Down, Up};
use skein_lib::{List, Queue, Secret, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};
const FORGE: u16 = 3;
const OTHER_FORGE: u16 = 4;
fn event(root: &mut Domain, value: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::step(root, &f::env(&f::limits()), value, &mut out);
    out
}
fn act(root: &mut Domain, act: forge::Act) -> Queue<Request> {
    event(root, Event::Forge(forge::Event::Act { act }))
}
fn paste(root: &mut Domain, field: Field, bytes: &[u8]) {
    event(root, Event::Forge(forge::Event::ForgejoToken { field, value: Secret::new(bytes.into()) }));
}
fn bytes(seed: u64, length: usize) -> Box<[u8]> {
    let mut rng = skein_lib::Rng::new(seed);
    let mut bytes = skein_lib::bytes::zeroed(length);
    for byte in &mut bytes {
        *byte = rng.next_u64().to_le_bytes()[0];
    }
    bytes
}
fn row(number: u16, kind: w::Kind, mode: w::Mode, connection: w::Connection, ended: Option<&[u8]>) -> w::ForgeRow {
    let mut stopped = None;
    if let Some(why) = ended {
        stopped = Some(w::ConnectStopped::new(&w::CEILINGS, w::ConnectStoppedParts { why: why.into() }).unwrap());
    }
    w::ForgeRow::new(
        &w::CEILINGS,
        w::ForgeRowParts {
            number,
            kind,
            mode,
            connection,
            offer: w::Offer::NotOffered,
            first: None,
            installations: List::with_capacity(0),
            ended: stopped,
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
fn deliver(root: &mut Domain, watch: Token, delivered: &[w::ForgeRow]) {
    let mut rows = List::with_capacity(u32::try_from(delivered.len()).unwrap());
    for row in delivered {
        rows.push(row.clone()).unwrap();
    }
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
    deliver(&mut root, watch, &[row(FORGE, w::Kind::Forgejo, w::Mode::Connected, w::Connection::NotConnected, None)]);
    (root, watch)
}
fn send(root: &mut Domain) -> Queue<Request> {
    act(root, forge::Act::ConnectTokens { forge: FORGE })
}
#[test]
#[expect(clippy::disallowed_macros, reason = "capture Debug to check secret redaction")]
fn tokens_are_bundled_carried_once_and_cleared_at_send() {
    for webhook in [false, true] {
        let (mut root, _) = opened();
        let values = [bytes(11, 32), bytes(12, 32), bytes(13, 32)];
        for (field, value) in [(Field::Api, &values[0]), (Field::Git, &values[1]), (Field::Webhook, &values[2])] {
            if field != Field::Webhook || webhook {
                paste(&mut root, field, value);
            }
        }
        for value in &values {
            assert!(skein_lib::bytes::find(format!("{root:?}").as_bytes(), value).is_none());
        }
        let written = root.forge().tokens_form().written;
        let mut out = send(&mut root);
        let mut sent = 0_u32;
        while let Some(request) = out.pop() {
            for value in &values {
                assert!(skein_lib::bytes::find(format!("{request:?}").as_bytes(), value).is_none());
            }
            if let Request::Client {
                request:
                    jig_client::Request::Down {
                        down: Down::Send { request: jig_client::Ask::Family { record, .. }, value: Some(value), .. },
                    },
            } = request
            {
                let items = temper_protocol_records::decode_pasted(value.expose()).unwrap();
                assert_eq!(items.len(), if webhook { 3 } else { 2 });
                for (index, (code, secret)) in items.iter().enumerate() {
                    assert_eq!(*code, [2, 3, 5][index]);
                    assert!(secret.expose() == values[index].as_ref());
                }
                let w::Request::Connect(connect) =
                    wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request
                else {
                    panic!("connect")
                };
                let w::How::ForgejoTokens(_) = connect.how() else { panic!("tokens") };
                let w::Making::Kept(kept) = connect.forge() else { panic!("kept") };
                assert_eq!(kept.forge(), FORGE);
                sent += 1;
            }
        }
        assert_eq!(sent, 1);
        for field in [Field::Api, Field::Git, Field::Webhook] {
            assert!(!root.forge().tokens_form().has(field));
            assert!(root.forge().tokens_form().written[field.index()] > written[field.index()]);
        }
        assert!(send(&mut root).is_empty());
    }
}
#[test]
fn both_tokens_are_required_but_the_webhook_is_optional() {
    for fields in [&[][..], &[Field::Api][..], &[Field::Git][..], &[Field::Webhook][..]] {
        let (mut root, _) = opened();
        for field in fields {
            paste(&mut root, *field, &bytes(1, 16));
        }
        assert!(send(&mut root).is_empty());
        assert!(root.forge().tokens_form().checked);
    }
}
#[test]
fn the_aggregate_bound_includes_each_items_framing() {
    let overhead = temper_protocol_records::encode_pasted(&[(2, &[1]), (3, &[2])]).unwrap().len() - 2;
    for (extra, accepted) in [(0, true), (1, false)] {
        let (mut root, _) = opened();
        paste(&mut root, Field::Api, &bytes(1, temper_protocol_records::PASTED_BYTES - overhead - 1 + extra));
        paste(&mut root, Field::Git, &bytes(2, 1));
        assert_eq!(root.forge().tokens_form().too_long, !accepted);
        assert_eq!(send(&mut root).is_empty(), !accepted);
        if !accepted {
            paste(&mut root, Field::Api, &bytes(3, 16));
            assert!(!root.forge().tokens_form().too_long);
            assert!(!send(&mut root).is_empty());
        }
    }
}
#[test]
fn only_disconnected_forgejos_of_the_connected_mode_send() {
    for (kind, mode, connection, allowed) in [
        (w::Kind::Forgejo, w::Mode::Connected, w::Connection::NotConnected, true),
        (
            w::Kind::Forgejo,
            w::Mode::Connected,
            w::Connection::Stopped(
                w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: b"stopped".as_slice().into() }).unwrap(),
            ),
            true,
        ),
        (w::Kind::Forgejo, w::Mode::Connected, w::Connection::Connecting, false),
        (
            w::Kind::Forgejo,
            w::Mode::Connected,
            w::Connection::Connected(
                w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 7, login: b"bot".as_slice().into() }).unwrap(),
            ),
            false,
        ),
        (w::Kind::Forgejo, w::Mode::Managed, w::Connection::NotConnected, false),
        (w::Kind::Forgejo, w::Mode::Owned, w::Connection::NotConnected, false),
        (w::Kind::Github, w::Mode::Connected, w::Connection::NotConnected, false),
    ] {
        let (mut root, watch) = opened();
        deliver(&mut root, watch, &[row(FORGE, kind, mode, connection, None)]);
        paste(&mut root, Field::Api, &bytes(1, 16));
        paste(&mut root, Field::Git, &bytes(2, 16));
        assert_eq!(!send(&mut root).is_empty(), allowed);
    }
}
#[test]
fn a_stop_clears_all_fields_and_returns_to_details_before_the_answer() {
    let (mut root, watch) = opened();
    paste(&mut root, Field::Api, &bytes(1, 16));
    paste(&mut root, Field::Git, &bytes(2, 16));
    send(&mut root);
    let written = root.forge().tokens_form().written;
    deliver(
        &mut root,
        watch,
        &[row(FORGE, w::Kind::Forgejo, w::Mode::Connected, w::Connection::NotConnected, Some(b"tokens refused"))],
    );
    for field in [Field::Api, Field::Git, Field::Webhook] {
        assert!(!root.forge().tokens_form().has(field));
        assert!(root.forge().tokens_form().written[field.index()] > written[field.index()]);
    }
    assert_eq!(
        forge::connect::step(Some(Choice::Forge(FORGE)), root.forges(), None),
        forge::connect::Shown::Details(Choice::Forge(FORGE))
    );
    paste(&mut root, Field::Api, &bytes(1, 16));
    assert!(root.forge().tokens_form().has(Field::Api));
    let written = root.forge().tokens_form().written;
    let stopped =
        row(FORGE, w::Kind::Forgejo, w::Mode::Connected, w::Connection::NotConnected, Some(b"tokens refused"));
    deliver(&mut root, watch, core::slice::from_ref(&stopped));
    assert!(root.forge().tokens_form().has(Field::Api));
    assert_eq!(root.forge().tokens_form().written, written);
    deliver(
        &mut root,
        watch,
        &[stopped, row(OTHER_FORGE, w::Kind::Forgejo, w::Mode::Connected, w::Connection::Connecting, None)],
    );
    assert!(root.forge().tokens_form().has(Field::Api));
    assert_eq!(root.forge().tokens_form().written, written);
    paste(&mut root, Field::Git, &bytes(2, 16));
    assert!(!send(&mut root).is_empty());
    let written = root.forge().tokens_form().written;
    deliver(
        &mut root,
        watch,
        &[row(FORGE, w::Kind::Forgejo, w::Mode::Connected, w::Connection::NotConnected, Some(b"git token refused"))],
    );
    for field in [Field::Api, Field::Git, Field::Webhook] {
        assert!(!root.forge().tokens_form().has(field));
        assert!(root.forge().tokens_form().written[field.index()] > written[field.index()]);
    }
    assert_eq!(
        forge::connect::step(Some(Choice::Forge(FORGE)), root.forges(), None),
        forge::connect::Shown::Details(Choice::Forge(FORGE))
    );
    paste(&mut root, Field::Api, &bytes(1, 16));
    event(&mut root, Event::SignedOut);
    for field in [Field::Api, Field::Git, Field::Webhook] {
        assert!(!root.forge().tokens_form().has(field));
    }
}
#[test]
fn a_stopped_row_redelivered_keeps_a_fresh_paste() {
    for (connection, ended) in [
        (w::Connection::NotConnected, Some(b"tokens refused".as_slice())),
        (
            w::Connection::Stopped(
                w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: b"stopped".as_slice().into() }).unwrap(),
            ),
            None,
        ),
    ] {
        let (mut root, watch) = opened();
        paste(&mut root, Field::Api, &bytes(1, 16));
        paste(&mut root, Field::Git, &bytes(2, 16));
        send(&mut root);
        let written = root.forge().tokens_form().written;
        let stopped = row(FORGE, w::Kind::Forgejo, w::Mode::Connected, connection, ended);
        deliver(&mut root, watch, core::slice::from_ref(&stopped));
        for field in [Field::Api, Field::Git, Field::Webhook] {
            assert!(!root.forge().tokens_form().has(field));
            assert!(root.forge().tokens_form().written[field.index()] > written[field.index()]);
        }
        paste(&mut root, Field::Api, &bytes(1, 16));
        let written = root.forge().tokens_form().written;
        deliver(&mut root, watch, &[stopped]);
        assert!(root.forge().tokens_form().has(Field::Api));
        assert_eq!(root.forge().tokens_form().written, written);
    }
}
#[test]
fn an_unrelated_stopped_forgejo_leaves_a_new_forges_draft() {
    for (connection, ended) in [
        (w::Connection::NotConnected, Some(b"tokens refused".as_slice())),
        (
            w::Connection::Stopped(
                w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: b"stopped".as_slice().into() }).unwrap(),
            ),
            None,
        ),
    ] {
        let (mut root, _, watch) = connect::opened(true);
        let stopped = row(FORGE, w::Kind::Forgejo, w::Mode::Connected, connection.clone(), ended);
        deliver(&mut root, watch, core::slice::from_ref(&stopped));
        paste(&mut root, Field::Api, &bytes(1, 16));
        paste(&mut root, Field::Git, &bytes(2, 16));
        let written = root.forge().tokens_form().written;
        deliver(&mut root, watch, &[stopped]);
        assert!(root.forge().tokens_form().has(Field::Api));
        assert!(root.forge().tokens_form().has(Field::Git));
        assert_eq!(root.forge().tokens_form().written, written);
        deliver(
            &mut root,
            watch,
            &[row(FORGE, w::Kind::Forgejo, w::Mode::Connected, connection, Some(b"git token refused"))],
        );
        assert!(root.forge().tokens_form().has(Field::Api));
        assert!(root.forge().tokens_form().has(Field::Git));
        assert_eq!(root.forge().tokens_form().written, written);
        act(
            &mut root,
            forge::Act::Forges(forge::forges::Act::PlaceWrite {
                field: forge::admin::PlaceField::Host,
                text: b"forge.example".as_slice().into(),
            }),
        );
        let mut sent = 0_u32;
        for request in &act(&mut root, forge::Act::Forges(forge::forges::Act::ConnectTokens)) {
            if let Request::Client {
                request:
                    jig_client::Request::Down { down: Down::Send { request: jig_client::Ask::Family { record, .. }, .. } },
            } = request
            {
                let w::Request::Connect(connect) =
                    wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request
                else {
                    panic!("connect")
                };
                let w::Making::New(_) = connect.forge() else { panic!("new") };
                let w::How::ForgejoTokens(_) = connect.how() else { panic!("tokens") };
                sent += 1;
            }
        }
        assert_eq!(sent, 1);
    }
}
#[test]
fn a_stop_of_another_forge_leaves_the_sent_connects_draft() {
    let (mut root, watch) = opened();
    paste(&mut root, Field::Api, &bytes(1, 16));
    paste(&mut root, Field::Git, &bytes(2, 16));
    let mut attempt = None;
    for request in &send(&mut root) {
        if let Request::Client { request: jig_client::Request::Down { down: Down::Send { attempt: value, .. } } } =
            request
        {
            attempt = Some(*value);
        }
    }
    let answer = w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome: w::Outcome::Connecting }).unwrap();
    event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::Answer {
                    attempt: attempt.unwrap(),
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Family(jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::answer(&answer),
                    })),
                    why: None,
                },
            },
        },
    );
    paste(&mut root, Field::Api, &bytes(1, 16));
    let written = root.forge().tokens_form().written;
    deliver(
        &mut root,
        watch,
        &[
            row(FORGE, w::Kind::Forgejo, w::Mode::Connected, w::Connection::Connecting, None),
            row(
                OTHER_FORGE,
                w::Kind::Forgejo,
                w::Mode::Connected,
                w::Connection::NotConnected,
                Some(b"tokens refused"),
            ),
        ],
    );
    assert!(root.forge().tokens_form().has(Field::Api));
    assert_eq!(root.forge().tokens_form().written, written);
    deliver(
        &mut root,
        watch,
        &[row(FORGE, w::Kind::Forgejo, w::Mode::Connected, w::Connection::NotConnected, Some(b"tokens refused"))],
    );
    for field in [Field::Api, Field::Git, Field::Webhook] {
        assert!(!root.forge().tokens_form().has(field));
        assert!(root.forge().tokens_form().written[field.index()] > written[field.index()]);
    }
}
#[test]
fn new_connected_forgejo_uses_the_shared_place_and_needs_an_address() {
    let (mut root, _, _) = connect::opened(true);
    paste(&mut root, Field::Api, &bytes(1, 16));
    paste(&mut root, Field::Git, &bytes(2, 16));
    assert!(act(&mut root, forge::Act::Forges(forge::forges::Act::ConnectTokens)).is_empty());
    act(
        &mut root,
        forge::Act::Forges(forge::forges::Act::PlaceWrite {
            field: forge::admin::PlaceField::Host,
            text: b"forge.example".as_slice().into(),
        }),
    );
    let mut sent = 0_u32;
    for request in &act(&mut root, forge::Act::Forges(forge::forges::Act::ConnectTokens)) {
        if let Request::Client {
            request:
                jig_client::Request::Down { down: Down::Send { request: jig_client::Ask::Family { record, .. }, .. } },
        } = request
        {
            let w::Request::Connect(connect) =
                wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request
            else {
                panic!("connect")
            };
            let w::Making::New(new) = connect.forge() else { panic!("new") };
            let w::ForgeAsk::Connected(place) = new.forge() else { panic!("connected forge") };
            assert_eq!(place.host(), b"forge.example");
            assert_eq!(place.port(), 443);
            sent += 1;
        }
    }
    assert_eq!(sent, 1);
}

#[test]
fn the_new_forge_follows_its_number_in_either_watch_answer_order() {
    for watch_first in [true, false] {
        let (mut root, _, watch) = connect::opened(true);
        event(
            &mut root,
            Event::Went { place: crate::Place::Address(crate::Address::ConnectForge { forge: Some(Choice::Managed) }) },
        );
        act(
            &mut root,
            forge::Act::Forges(forge::forges::Act::PlaceWrite {
                field: forge::admin::PlaceField::Host,
                text: b"forge.example".as_slice().into(),
            }),
        );
        paste(&mut root, Field::Api, &bytes(1, 16));
        paste(&mut root, Field::Git, &bytes(2, 16));
        let out = act(&mut root, forge::Act::Forges(forge::forges::Act::ConnectTokens));
        let mut attempt = None;
        for request in &out {
            if let Request::Client { request: jig_client::Request::Down { down: Down::Send { attempt: value, .. } } } =
                request
            {
                attempt = Some(*value);
            }
        }
        if watch_first {
            deliver(
                &mut root,
                watch,
                &[row(FORGE, w::Kind::Forgejo, w::Mode::Connected, w::Connection::Connecting, None)],
            );
        }
        let answer = w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome: w::Outcome::Connecting }).unwrap();
        let out = event(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: Up::Answer {
                        attempt: attempt.unwrap(),
                        envelope: skein_lib::RequestEnvelope::Final,
                        answer: Some(jig_client::Answered::Family(jig_client::Family {
                            family: wire::FORGE,
                            version: wire::CLIENT_VERSION,
                            bytes: wire::answer(&answer),
                        })),
                        why: None,
                    },
                },
            },
        );
        if watch_first {
            let mut followed = false;
            for request in &out {
                if *request
                    == (Request::Show {
                        address: crate::Address::ConnectForge { forge: Some(Choice::Forge(FORGE)) },
                        replace: true,
                    })
                {
                    followed = true;
                }
            }
            assert!(followed);
        } else {
            deliver(
                &mut root,
                watch,
                &[row(FORGE, w::Kind::Forgejo, w::Mode::Connected, w::Connection::Connecting, None)],
            );
        }
        assert_eq!(root.forge().tokens().unwrap().forge, FORGE);
        assert!(root.forge().setup_form().making.is_none());
    }
}
