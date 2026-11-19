use super::*;
fn part(mount: &mut Mount, out: &mut Queue<Request>, bytes: &[u8]) {
    mount.down(
        &fixtures::env(),
        Event::Component(kit::ToTransport::Part { exchange: fixtures::EXCHANGE, message: Box::from(bytes) }),
    );
    fixtures::pump(mount, out);
}
fn done(mount: &mut Mount, out: &mut Queue<Request>) {
    mount.down(&fixtures::env(), Event::Component(kit::ToTransport::Done { exchange: fixtures::EXCHANGE }));
    fixtures::pump(mount, out);
}
#[test]
fn batch_frames_leave_before_done_and_the_call_lasts_through_the_body() {
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(256);
    let initial_room = mount.room();
    fixtures::call(&mut mount, &mut out, Method::Post, b"/jig/v1/batch", &[], &[0, 0, 0, 0]);
    let messages = fixtures::component(&mut out);
    assert_eq!(
        *messages,
        [kit::FromTransport::Exchange {
            exchange: fixtures::EXCHANGE,
            credential: None,
            browser: false,
            message: kit::Message::Batch(Box::new([0, 0, 0, 0])),
        }]
    );
    let part_record = w::BatchReply { member: 1, reply: Some(Box::new([3, 4])) };
    let mut bytes = skein_lib::Writer::new(skein_lib::length::usize_of(part_record.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")));
    part_record.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).expect("part bytes");
    let bytes = bytes.finish();
    assert_eq!(bytes.len(), 9);
    part(&mut mount, &mut out, &bytes);
    let (status, body, headers) = fixtures::answered(&mut mount, &mut out, fixtures::EXCHANGE);
    assert_eq!(status, 200);
    assert_eq!(body.as_ref(), skein_lib::bytes::concat(&[&[0, 0, 0, 9], &bytes]).as_ref());
    assert_eq!(fixtures::header(&headers, b"content-type"), Some(b"application/octet-stream".as_slice()));
    assert_eq!(fixtures::header(&headers, b"cache-control"), Some(b"no-store".as_slice()));
    assert_eq!(mount.room(), initial_room.checked_sub(1).expect("one batch call"));
    assert!(mount.calls.get(&fixtures::EXCHANGE).expect("call").reply_owed);
    // The delayed member can arrive after the first frame has been taken.
    part(&mut mount, &mut out, &[7]);
    done(&mut mount, &mut out);
    assert_eq!(mount.room(), initial_room.checked_sub(1).expect("body still writing"));
    let mut body = List::with_capacity(5);
    let mut finishes = 0_u32;
    while let Some(request) = out.pop() {
        match request {
            Request::Http { request: http::Request::Reply(Down::Demand { .. }), .. } => {
                fixtures::event(&mut mount, &mut out, fixtures::EXCHANGE, http::Event::Reply(Up::Room));
            }
            Request::Http { request: http::Request::Reply(Down::Send(bytes)), .. } => {
                for byte in bytes {
                    body.push(byte).expect("second frame");
                }
            }
            Request::Http { request: http::Request::Reply(Down::Finish), .. } => finishes += 1,
            other @ (Request::Http { .. } | Request::Component(_)) => panic!("unexpected batch output: {other:?}"),
        }
    }
    assert_eq!(body.into_boxed().as_ref(), &[0, 0, 0, 1, 7]);
    assert_eq!(finishes, 1);
    fixtures::event(&mut mount, &mut out, fixtures::EXCHANGE, http::Event::Done(http::Reuse::Keep));
    assert_eq!(mount.room(), initial_room);
    assert!(fixtures::component(&mut out).is_empty());
}
#[test]
fn a_batch_can_hold_all_64_frames_until_room_and_finish_only_after_the_last_piece() {
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(512);
    fixtures::call(&mut mount, &mut out, Method::Post, b"/jig/v1/batch", &[], &[0, 0, 0, 0]);
    fixtures::component(&mut out);
    for index in 0..64_u8 {
        part(&mut mount, &mut out, &[index]);
    }
    done(&mut mount, &mut out);
    assert!(mount.calls.contains_key(&fixtures::EXCHANGE));
    let mut body = List::with_capacity(320);
    let mut finishes = 0_u32;
    while let Some(request) = out.pop() {
        match request {
            Request::Http { request: http::Request::Reply(Down::Demand { .. }), .. } => {
                // Slice every frame into single bytes at the connection's edge.
                let mut env = fixtures::env();
                env.limits.send = 1;
                mount.down(&env, Event::Http { call: fixtures::EXCHANGE, event: http::Event::Reply(Up::Room) });
                while mount.up(&env, &mut out) {}
            }
            Request::Http { request: http::Request::Reply(Down::Send(bytes)), .. } => {
                assert_eq!(bytes.len(), 1);
                for byte in bytes {
                    body.push(byte).expect("64 frames");
                }
            }
            Request::Http { request: http::Request::Reply(Down::Finish), .. } => {
                assert_eq!(body.len(), 320, "finish follows every frame's last byte");
                finishes += 1;
            }
            Request::Http { request: http::Request::Respond(response), .. } => {
                assert_eq!(response.body, http::Body::Chunked);
            }
            Request::Http { request: http::Request::Discard, .. } => (),
            other @ (Request::Http { .. } | Request::Component(_)) => panic!("unexpected batch output: {other:?}"),
        }
    }
    assert_eq!(finishes, 1);
    for (index, frame) in body.into_boxed().as_chunks::<5>().0.iter().enumerate() {
        assert_eq!(frame, &[0, 0, 0, 1, u8::try_from(index).expect("64 frames")]);
    }
}
#[test]
fn only_an_unfinished_batch_says_exchange_gone_when_its_connection_ends() {
    for terminal in [false, true] {
        let mut mount = fixtures::mount();
        let mut out = Queue::with_capacity(32);
        fixtures::call(&mut mount, &mut out, Method::Post, b"/jig/v1/batch", &[], &[0, 0, 0, 0]);
        fixtures::component(&mut out);
        part(&mut mount, &mut out, &[7]);
        if terminal {
            done(&mut mount, &mut out);
        }
        fixtures::event(&mut mount, &mut out, fixtures::EXCHANGE, http::Event::Closed);
        let expected: Box<[kit::FromTransport]> = if terminal {
            Box::new([])
        } else {
            Box::new([kit::FromTransport::ExchangeGone { exchange: fixtures::EXCHANGE }])
        };
        assert_eq!(fixtures::component(&mut out), expected);
    }
}
#[test]
fn a_batchs_bad_record_and_oversize_answers_are_whole() {
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(32);
    let call = fixtures::call(&mut mount, &mut out, Method::Post, b"/jig/v1/batch", &[], &[255]);
    assert_eq!(fixtures::component(&mut out).len(), 1);
    fixtures::reply(&mut mount, &mut out, call, kit::Reply::NotARecord);
    let (status, bytes, _) = fixtures::answered(&mut mount, &mut out, call);
    assert_eq!((status, bytes.len()), (400, 0));
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(32);
    let length = u64::from(fixtures::env().limits.body).checked_add(1).expect("past body bound");
    mount.down(
        &fixtures::env(),
        Event::Call { call, head: fixtures::head(Method::Post, b"/jig/v1/batch", &[], http::Body::Length(length)) },
    );
    fixtures::pump(&mut mount, &mut out);
    let (status, bytes, _) = fixtures::answered(&mut mount, &mut out, call);
    assert_eq!((status, bytes.len()), (413, 0));
    assert!(fixtures::component(&mut out).is_empty());
}
#[test]
fn a_stream_takes_only_the_replaces_query_as_lowercase_hex() {
    for (query, name) in [
        (b"".as_slice(), Some(None)),
        (
            b"?replaces=00112233445566778899aabbccddeeff",
            Some(Some([0, 17, 34, 51, 68, 85, 102, 119, 136, 153, 170, 187, 204, 221, 238, 255])),
        ),
        (b"?", None),
        (b"?replaces=", None),
        (b"?replaces=00112233445566778899AABBCCDDEEFF", None),
        (b"?replaces=00112233445566778899aabbccddeef", None),
        (b"?replaces=00112233445566778899aabbccddeeff0", None),
        (b"?replaces=00112233445566778899aabbccddeefg", None),
        (b"?replaces=00112233445566778899aabbccddeeff&x=1", None),
        (b"?other=00112233445566778899aabbccddeeff", None),
    ] {
        let mut mount = fixtures::mount();
        let mut out = Queue::with_capacity(32);
        let target = skein_lib::bytes::concat(&[b"/jig/v1/stream", query]);
        fixtures::call(&mut mount, &mut out, Method::Get, &target, &[], &[]);
        if let Some(replaces) = name {
            assert_eq!(
                *fixtures::component(&mut out),
                [kit::FromTransport::StreamOpen {
                    stream: fixtures::EXCHANGE,
                    credential: None,
                    replaces,
                    version: None
                }]
            );
        } else {
            let (status, bytes, _) = fixtures::answered(&mut mount, &mut out, fixtures::EXCHANGE);
            assert_eq!((status, bytes.len()), (400, 0), "query {query:?}");
            assert!(fixtures::component(&mut out).is_empty());
        }
    }
}

#[test]
fn a_versioned_stream_replaces_its_predecessor_in_the_same_opening() {
    for query in [
        b"?version=32&replaces=00112233445566778899aabbccddeeff".as_slice(),
        b"?replaces=00112233445566778899aabbccddeeff&version=32".as_slice(),
    ] {
        let mut mount = fixtures::mount();
        let mut out = Queue::with_capacity(32);
        let target = skein_lib::bytes::concat(&[b"/jig/v1/stream", query]);
        fixtures::call(&mut mount, &mut out, Method::Get, &target, &[], &[]);
        assert_eq!(
            *fixtures::component(&mut out),
            [kit::FromTransport::StreamOpen {
                stream: fixtures::EXCHANGE,
                credential: None,
                version: Some(32),
                replaces: Some([0, 17, 34, 51, 68, 85, 102, 119, 136, 153, 170, 187, 204, 221, 238, 255]),
            }],
        );
    }
}
