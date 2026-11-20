use super::fixtures as f;
use crate::{Event, FromTransport, Message, Reply, Request, ToTransport};
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary as core;
use skein_lib::{List, Reader, ReplyTo, Secret, Token, Writer};
fn read() -> w::Owned {
    w::Owned { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: f::encoded_read(&w::Read::Me) }
}
fn opening(stream: [u8; 16], watch: u64) -> w::Member {
    w::Member::Open(w::Open { stream, watch, subject: f::owned_subject(&w::Subject::Me) })
}
fn closing(stream: [u8; 16], watch: u64) -> w::Member {
    w::Member::Close(w::WatchClose { stream, watch })
}
fn encoded(members: &[w::Member]) -> Box<[u8]> {
    let mut list = List::with_capacity(u32::try_from(members.len()).expect("members"));
    for member in members {
        list.push(member.clone()).expect("member");
    }
    let value = w::Batch { members: list };
    let mut bytes = Writer::new(skein_lib::length::usize_of(
        value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
    ));
    value.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).expect("batch bytes");
    bytes.finish()
}
fn send(world: &mut f::Exchange, members: &[w::Member]) -> Token {
    let owner = world.send(Message::Batch(encoded(members)));
    world.authenticate();
    owner
}
fn pump(world: &mut f::Exchange) {
    while world.component.up(&world.env, &mut world.out) {}
}
fn take_read(world: &mut f::Exchange) -> ReplyTo {
    let Some(Request::Core(event)) = world.out.pop() else { panic!("read request") };
    let core::Event::Read { reply, .. } = *event else { panic!("read") };
    reply
}
fn part(world: &mut f::Exchange, owner: Token, member: u16) -> Option<Box<[u8]>> {
    assert!(world.component.up(&world.env, &mut world.out));
    let Some(Request::Transport(ToTransport::Part { exchange, message })) = world.out.pop() else { panic!("part") };
    assert_eq!(exchange, owner);
    let value = w::BatchReply::decode(&f::wire(), &mut Reader::new(&message)).expect("batch reply");
    assert_eq!(value.member, member);
    value.reply
}
fn done(world: &mut f::Exchange, owner: Token) {
    assert!(world.component.up(&world.env, &mut world.out));
    let Some(Request::Transport(ToTransport::Done { exchange })) = world.out.pop() else { panic!("Done") };
    assert_eq!(exchange, owner);
}
fn read_answer(bytes: Option<Box<[u8]>>) -> w::ReadAnswer {
    w::ReadAnswer::decode(&f::wire(), &mut Reader::new(&bytes.expect("read record"))).expect("read answer")
}
fn open_answer(bytes: Option<Box<[u8]>>) -> w::OpenAnswer {
    w::OpenAnswer::decode(&f::wire(), &mut Reader::new(&bytes.expect("opening record"))).expect("opening answer")
}
fn busy_open() -> w::OpenAnswer {
    w::OpenAnswer::Refused(w::OpenAnswerRefused { why: w::WatchRefusal::Busy })
}
#[test]
fn parts_follow_release_order_and_hold_one_exchange_through_done() {
    let mut world = f::Exchange::new();
    let owner = send(&mut world, &[w::Member::Read(read()), w::Member::Read(read())]);
    let a = take_read(&mut world);
    let b = take_read(&mut world);
    assert_eq!(world.component.admitted(), 1);
    assert_eq!(world.component.active(f::NUMBER), 1);
    assert!(!world.component.up(&world.env, &mut world.out), "A and B are held");
    world.release_read(b, core::reads::Answer::None(core::reads::Why::Gone));
    assert_eq!(read_answer(part(&mut world, owner, 1)), f::read_none(w::ReadNone::Gone));
    assert_eq!(world.component.admitted(), 1, "one member is still held");
    world.release_read(a, core::reads::Answer::None(core::reads::Why::Unknown));
    assert_eq!(read_answer(part(&mut world, owner, 0)), f::read_none(w::ReadNone::Unknown));
    assert_eq!(world.component.active(f::NUMBER), 1, "last part precedes Done");
    done(&mut world, owner);
    assert_eq!(world.component.admitted(), 0);
    assert_eq!(world.component.active(f::NUMBER), 0);
}
#[test]
fn reads_alone_and_in_batches_share_the_sign_ins_read_room() {
    let mut world = f::Exchange::new();
    world.env.limits.reads_per_client = 2;
    let (_, a, _) = world.read(&w::Read::Me);
    let owner = send(&mut world, &[w::Member::Read(read()), w::Member::Read(read())]);
    let b = take_read(&mut world);
    assert_eq!(read_answer(part(&mut world, owner, 1)), f::read_none(w::ReadNone::Busy));
    world.env.limits.exchanges_per_client = 8;
    let lone = world.send(Message::Read(f::owned_read(
        w::Owner::Jig,
        jig_client_protocol_wire::VERSION,
        f::encoded_read(&w::Read::Me),
    )));
    world.authenticate();
    assert_eq!(world.read_answer(lone), f::read_none(w::ReadNone::Busy));
    world.release_read(a, core::reads::Answer::None(core::reads::Why::Gone));
    world.release_read(b, core::reads::Answer::None(core::reads::Why::Gone));
    assert_eq!(read_answer(part(&mut world, owner, 0)), f::read_none(w::ReadNone::Gone));
    done(&mut world, owner);
}
#[test]
fn openings_alone_and_in_batches_share_the_sign_ins_opening_room() {
    let mut world = f::Exchange::new();
    let stream = world.open_stream();
    world.out.clear();
    world.env.limits.openings_per_client = 2;
    world.env.limits.exchanges_per_client = 8;
    world.open_record(stream, 1, f::owned_subject(&w::Subject::Me));
    let a = world.take_watch();
    let owner = send(&mut world, &[opening(stream, 2), opening(stream, 3)]);
    let b = world.take_watch();
    assert_eq!(open_answer(part(&mut world, owner, 1)), busy_open());
    let lone = world.open_record(stream, 4, f::owned_subject(&w::Subject::Me));
    assert_eq!(w::OpenAnswer::decode(&f::wire(), &mut Reader::new(world.response(lone).1)), Ok(busy_open()));
    world.component.down(&world.env, Event::Admitted { watcher: a }, &mut world.out);
    world.component.down(&world.env, Event::Admitted { watcher: b }, &mut world.out);
    assert_eq!(open_answer(part(&mut world, owner, 0)), w::OpenAnswer::Admitted);
    done(&mut world, owner);
}
#[test]
fn a_batch_without_exchange_room_refuses_reads_and_openings_but_takes_closings() {
    for global in [false, true] {
        let mut world = f::Exchange::new();
        let stream = world.open_stream();
        world.out.clear();
        world.open_record(stream, 1, f::owned_subject(&w::Subject::Me));
        let watcher = world.take_watch();
        world.component.down(&world.env, Event::Admitted { watcher }, &mut world.out);
        pump(&mut world);
        world.out.clear();
        let (_, held, _) = world.read(&w::Read::Me);
        if global {
            world.env.limits.exchanges = 1;
        } else {
            world.env.limits.exchanges_per_client = 1;
        }
        let owner = send(&mut world, &[w::Member::Read(read()), opening(stream, 2), closing(stream, 1)]);
        let Some(Request::Core(event)) = world.out.pop() else { panic!("unwatch admitted despite exchange pressure") };
        let core::Event::Unwatch { watcher: got } = *event else { panic!("unwatch") };
        assert_eq!(got, watcher);
        assert_eq!(read_answer(part(&mut world, owner, 0)), f::read_none(w::ReadNone::Busy));
        assert_eq!(open_answer(part(&mut world, owner, 1)), busy_open());
        world.component.down(
            &world.env,
            Event::WatchEnded { watcher, why: jig_domain_views::End::Unwatched },
            &mut world.out,
        );
        let bytes = part(&mut world, owner, 2).expect("closed record");
        assert_eq!(w::CloseAnswer::decode(&f::wire(), &mut Reader::new(&bytes)), Ok(w::CloseAnswer::Closed));
        done(&mut world, owner);
        world.release_read(held, core::reads::Answer::None(core::reads::Why::Gone));
    }
}
#[test]
fn malformed_members_and_conflicting_closings_have_no_record() {
    let mut world = f::Exchange::new();
    let stream = world.open_stream();
    world.out.clear();
    world.open_record(stream, 1, f::owned_subject(&w::Subject::Me));
    let watcher = world.take_watch();
    world.component.down(&world.env, Event::Admitted { watcher }, &mut world.out);
    pump(&mut world);
    world.out.clear();
    let family = w::Owner::Family(w::Family { number: 9 });
    let bad = w::Owned { owner: family, version: 2, record: Box::new([255]) };
    let owner = send(&mut world, &[closing(stream, 1), closing(stream, 1), w::Member::Read(bad)]);
    world.out.pop().expect("unwatch");
    assert_eq!(part(&mut world, owner, 1), None);
    assert_eq!(part(&mut world, owner, 2), None);
    world.component.down(
        &world.env,
        Event::WatchEnded { watcher, why: jig_domain_views::End::Unwatched },
        &mut world.out,
    );
    assert!(part(&mut world, owner, 0).is_some());
    done(&mut world, owner);
}
#[test]
fn a_bad_batch_is_refused_whole_before_authentication_or_work() {
    let members = [w::Member::Read(read())];
    let bytes = encoded(&members);
    let trailing = skein_lib::bytes::concat(&[&bytes, &[0]]);
    let valid = encoded(&members);
    let short = u32::try_from(valid.len()).expect("batch bytes").checked_sub(1).expect("batch header");
    for (body, limit) in [(Box::from([0, 0, 0, 65]), 256), (Box::from([255]), 256), (trailing, 256), (valid, short)] {
        let mut world = f::Exchange::new();
        world.env.limits.message_bytes = limit;
        world.out.clear();
        let owner = world.send(Message::Batch(body));
        assert_eq!(world.response(owner).0, 400);
        assert!(world.out.is_empty(), "no credential read or work");
        assert!(world.component.batches.is_empty());
        assert!(world.component.pending.exchanges.is_empty());
    }
    let mut world = f::Exchange::new();
    world.env.limits.message_bytes = 4096;
    let mut members = List::with_capacity(64);
    for index in 0..64 {
        members.push(closing([0; 16], index)).expect("64 members");
    }
    let owner = send(&mut world, &members.into_boxed());
    for index in 0..64 {
        let bytes = part(&mut world, owner, index).expect("closed record");
        assert_eq!(w::CloseAnswer::decode(&f::wire(), &mut Reader::new(&bytes)), Ok(w::CloseAnswer::Closed));
    }
    done(&mut world, owner);
}
#[test]
fn an_empty_or_signed_out_batch_is_still_an_exchange_with_parts_and_done() {
    let mut world = f::Exchange::new();
    let owner = world.send(Message::Batch(encoded(&[])));
    assert!(!world.component.up(&world.env, &mut world.out), "credential check precedes Done even when empty");
    world.authenticate();
    done(&mut world, owner);
    let owner = Token::new(900);
    world.component.down(
        &world.env,
        Event::Transport(FromTransport::Exchange {
            exchange: owner,
            credential: None,
            via: crate::Via::Listener,
            source: None,
            browser: false,
            message: Message::Batch(encoded(&[w::Member::Read(read()), opening([0; 16], 1), closing([0; 16], 1)])),
        }),
        &mut world.out,
    );
    assert!(world.component.up(&world.env, &mut world.out));
    assert_eq!(read_answer(part(&mut world, owner, 0)), f::read_none(w::ReadNone::SignedOut));
    let signed_out = w::OpenAnswer::Refused(w::OpenAnswerRefused { why: w::WatchRefusal::SignedOut });
    assert_eq!(open_answer(part(&mut world, owner, 1)), signed_out);
    let bytes = part(&mut world, owner, 2).expect("signed out closing");
    assert_eq!(w::CloseAnswer::decode(&f::wire(), &mut Reader::new(&bytes)), Ok(w::CloseAnswer::SignedOut));
    done(&mut world, owner);
}
#[test]
fn a_gone_batchs_work_settles_without_parts_or_holding_the_drain() {
    let mut world = f::Exchange::new();
    world.component.down(
        &world.env,
        Event::Secrets(crate::SecretsAnswer::Got {
            owner: crate::SecretOwner::PublicNames,
            value: Some(crate::Kept {
                digest: Secret::new(Box::new([7; 32])),
                expires: skein_lib::Wall::from_nanos(u64::MAX),
            }),
        }),
        &mut world.out,
    );
    let owner = send(&mut world, &[w::Member::Read(read()), w::Member::Read(read())]);
    let a = take_read(&mut world);
    let b = take_read(&mut world);
    world.release_read(b, core::reads::Answer::None(core::reads::Why::Gone));
    world.component.down(&world.env, Event::Transport(FromTransport::ExchangeGone { exchange: owner }), &mut world.out);
    world.release_read(a, core::reads::Answer::None(core::reads::Why::Gone));
    assert!(world.component.pending.exchanges.is_empty());
    assert!(world.component.members.is_empty());
    world.component.down(&world.env, Event::Close, &mut world.out);
    pump(&mut world);
    let Some(Request::Closed) = world.out.pop() else { panic!("closed") };
    assert!(world.out.is_empty(), "even the queued part is dropped");
}
#[test]
fn a_stream_replaces_only_a_live_stream_of_its_sign_in_before_admission() {
    for (foreign, absent, valid_credential) in
        [(false, false, true), (true, false, true), (false, true, true), (false, false, false)]
    {
        let mut env = f::env();
        env.limits.streams = 1;
        let mut world = f::Exchange::with_env(env);
        let old_name = world.open_stream();
        let old_owner = *world.component.streams.first().expect("old stream").0;
        world.out.clear();
        world.open_record(old_name, 1, f::owned_subject(&w::Subject::Me));
        let watcher = world.take_watch();
        if foreign {
            world.component.streams.get_mut(&old_owner).expect("stream").sign_in = 42;
        }
        let owner = Token::new(900);
        let credential = if valid_credential { Some(Secret::new(world.token.clone())) } else { None };
        world.component.down(
            &world.env,
            Event::Transport(FromTransport::StreamOpen {
                version: Some(jig_client_protocol_wire::VERSION),
                stream: owner,
                credential,
                replaces: Some(if absent { [0; 16] } else { old_name }),
            }),
            &mut world.out,
        );
        if valid_credential {
            world.authenticate();
        }
        if !foreign && !absent && valid_credential {
            assert!(!world.component.streams.contains_key(&old_owner));
            assert_eq!(
                world.component.connections.get(&old_owner).expect("old transport").ended,
                Some(crate::Ended::Stopping)
            );
            assert!(world.component.streams.contains_key(&owner), "replacement admitted at the stream bound");
            let Some(Request::Core(event)) = world.out.pop() else { panic!("replacement unwatch") };
            let core::Event::Unwatch { watcher: got } = *event else { panic!("unwatch") };
            assert_eq!(got, watcher);
        } else {
            assert!(world.component.streams.contains_key(&old_owner));
            if valid_credential {
                let Some(Reply::StreamsFull { size }) =
                    world.component.connections.get(&owner).expect("caller").response
                else {
                    panic!(
                        "streams full: foreign={foreign}, absent={absent}, response={:?}",
                        world.component.connections.get(&owner).expect("caller")
                    )
                };
                assert_eq!(size, 1);
            }
            assert!(world.out.is_empty());
        }
    }
}
