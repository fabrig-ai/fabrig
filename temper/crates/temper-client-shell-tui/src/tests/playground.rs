//! A playground started elsewhere while the front is live (client/tui/box.md, The playground).
use super::*;
use temper_client_protocol_wire::{self as wire, v1 as f};

fn messages(duties: &Duties) -> Vec<(ReplySlot, Message)> {
    let mut messages = Vec::new();
    for request in &duties.asked {
        if let Request::Exchange { exchange, message, .. } = request {
            if let Message::Batch(bytes) = message {
                messages.extend(
                    batch_members(bytes)
                        .into_iter()
                        .map(|(member, message)| (ReplySlot { exchange: *exchange, member: Some(member) }, message)),
                );
            } else if let Message::Read(bytes) = message {
                messages.push((ReplySlot { exchange: *exchange, member: None }, Message::Read(bytes.clone())));
            } else if let Message::Watch(bytes) = message {
                messages.push((ReplySlot { exchange: *exchange, member: None }, Message::Watch(bytes.clone())));
            }
        }
    }
    messages
}

pub(super) fn forges_watch(duties: &Duties) -> u64 {
    messages(duties)
        .into_iter()
        .find_map(|(_, message)| {
            let Message::Watch(bytes) = message else { return None };
            let open = w::Open::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap();
            if open.subject.owner == w::Owner::Jig {
                return None;
            }
            let envelope =
                w::FamilyRecord::decode(&w::CEILINGS, &mut Reader::new(open.subject.record.as_ref())).unwrap();
            let watch = wire::decode_watching(envelope.record.as_ref(), &f::CEILINGS).unwrap();
            (watch.watch() == &f::Watch::Forges).then_some(open.watch)
        })
        .expect("session forges watch")
}

fn playground_read(duties: &Duties) -> ReplySlot {
    messages(duties)
        .into_iter()
        .find_map(|(slot, message)| {
            let Message::Read(bytes) = message else { return None };
            let owned = w::Owned::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap();
            if owned.owner == w::Owner::Jig {
                return None;
            }
            let envelope = w::FamilyRecord::decode(&w::CEILINGS, &mut Reader::new(owned.record.as_ref())).unwrap();
            let read = wire::decode_reading(envelope.record.as_ref(), &f::CEILINGS).unwrap();
            (read.read() == &f::Read::Playground).then_some(slot)
        })
        .expect("a first fake row refreshes the playground read")
}

#[test]
fn a_first_forges_delivery_with_the_playground_draws_it_without_reconnecting() {
    let mut host = host();
    start_local(&mut host, true, true, None, None);
    let signing = host.report(Report::Local { link: Some(local_link("127.0.0.1:8367", LOCAL_SECRET)) });
    let duties = reply(&mut host, local_exchange(&signing), sign_in_bytes(&signed_in_answer()));
    admit(&mut host, duties);
    let hello = hello_for_me(&host, launchpad_me(false));
    let duties =
        host.report(Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::Record(hello) }));
    let duties = admit(&mut host, duties);
    let watch = forges_watch(&duties);
    assert!(host.root.forge().playground_answered());
    assert!(host.root.forge().playground().is_none(), "the initial read preceded the other client's start");
    assert!(text(&mut host).contains("Welcome. Set temper up"));

    // The stream can coalesce away the empty forges document before this first delivery.
    let event = w::StreamEvent::Delivery(w::Delivery { watch, document: goal::family(goal::forges(f::Mode::Fake)) });
    let duties = host.report(Report::Heard(jig_client::Event::Streamed {
        message: jig_client::Streamed::Record(stream_bytes(&event)),
    }));
    let duties = admit_watches(&mut host, duties);
    let exchange = playground_read(&duties);
    let duties = delivery(&mut host, 0, &w::DeliveryValue::Me(launchpad_me(true)));
    admit(&mut host, duties);
    assert_eq!(host.root.pages().address(), &Address::Home { project: 3 });

    let record = wire::playground_answer(
        &f::PlaygroundAnswer::new(
            &f::CEILINGS,
            f::PlaygroundAnswerParts {
                state: f::PlaygroundState::Running,
                project: Some(3),
                forge: 2,
                provider: 3,
                owner: b"playground".as_slice().into(),
                name: b"widget".as_slice().into(),
                cues: List::with_capacity(0),
            },
        )
        .unwrap(),
    );
    let answer = w::ReadAnswer::Value(goal::family(record));
    let mut writer = Writer::new(
        usize::try_from(answer.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
    );
    answer.write(&jig_client_protocol_wire::CEILINGS, &mut writer).unwrap();
    let terminal = exchange.member.map(|_| exchange.exchange);
    let duties = reply_slot(&mut host, exchange, writer.finish());
    admit(&mut host, duties);
    if let Some(exchange) = terminal {
        let duties = host.report(Report::Heard(jig_client::Event::Done { exchange }));
        admit(&mut host, duties);
    }
    let frame = text(&mut host);
    let words = frame.lines().map(str::trim).collect::<Vec<_>>().join(" ");
    assert!(words.contains("This is the playground: a scripted model and in-memory forge."), "{frame}");
    assert!(frame.lines().last().unwrap().contains("playground"), "the live quiet line: {frame}");
    assert_eq!(host.root.client().link().state, jig_client::Link::Live);
}
