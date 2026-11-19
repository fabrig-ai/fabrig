//! A linked client's next request must leave after a folded erase and save.
use super::*;
use client::{Ask, Asker, Event, Key, Message, Request, contract};
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::VERSION;
use skein_lib::{Duration, List, Wall};

const FIRST: Key = Key { asker: Asker::JIG, number: 1 };
const NEXT: Key = Key { asker: Asker::JIG, number: 2 };
const TASK: u64 = 9;
const PROJECT: u32 = 1;
// The terminal request and its successor can coexist before reclamation.
const REQUESTS: u32 = 2;
const ASK_BYTES: u64 = 4096;
// The fixture has one me watch; spare receiving counts stay tiny.
const RECEIVING_ITEMS: u32 = 2;
const TEXT_BYTES: u32 = 32;
pub(super) const STORE_BYTES: u32 = 4096;

pub(super) fn bounds() -> client::Limits {
    // Two requests, the client's own me watch, and small receiving bounds.
    client::Limits {
        exchanges: REQUESTS + RECEIVING_ITEMS,
        domain: client::DomainLimits {
            owners: &[],
            requests: skein_lib::RequestLimits {
                requests: REQUESTS,
                bytes: u64::from(REQUESTS) * ASK_BYTES,
                out: REQUESTS * 4, // Admission, send, answer and erase.
                first: Duration::from_millis(10),
                most: Duration::from_millis(80),
                margin: Duration::from_secs(1),
            },
            ask_bytes: ASK_BYTES,
            handles: 1,
            watches: RECEIVING_ITEMS,
            reads: 1,
            turn_bytes: TEXT_BYTES,
            document_bytes: u64::from(STORE_BYTES),
            backoff: client::Backoff { first: Duration::from_millis(10), most: Duration::from_millis(80) },
            rows: RECEIVING_ITEMS,
            accounts: client::AccountCounts { accounts: 1, sources: 1, sign_ins: 1 },
            segments: RECEIVING_ITEMS,
            authority: RECEIVING_ITEMS,
            text: TEXT_BYTES,
            long_text: TEXT_BYTES,
            turn_text: TEXT_BYTES,
            projects: RECEIVING_ITEMS,
            fields: RECEIVING_ITEMS,
            delegates: RECEIVING_ITEMS,
            records: RECEIVING_ITEMS,
            kept: RECEIVING_ITEMS,
            parts: RECEIVING_ITEMS,
            heartbeat: Duration::from_secs(1),
        },
    }
}
fn stream(machine: &mut client::Client, record: &w::StreamEvent, out: &mut Queue<Request>) {
    let mut writer = Writer::new(skein_lib::length::usize_of(record.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")));
    record.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("measured stream record");
    machine.step(Time::ZERO, Wall::EPOCH, Event::Streamed { message: client::Streamed::Record(writer.finish()) }, out);
    machine.settle(Time::ZERO, Wall::EPOCH, out);
}
fn send(key: Key) -> Event {
    Event::Send { key, name: None, request: Ask::Jig(Box::new(contract::Ask::Stop { project: PROJECT, task: TASK })) }
}
fn discard(out: &mut Queue<Request>) {
    while out.pop().is_some() {}
}
pub(super) fn mixed_turn() -> (client::Client, Queue<Request>) {
    let bounds = bounds();
    let mut machine = client::Client::new(&bounds, 7).expect("client bounds");
    let mut out = Queue::uncapped();
    for event in [
        Event::Start { kept: List::with_capacity(0) },
        Event::SignIn { with: client::With::Held(Secret::new(Box::new(TOKEN))) },
    ] {
        machine.step(Time::ZERO, Wall::EPOCH, event, &mut out);
        machine.settle(Time::ZERO, Wall::EPOCH, &mut out);
        discard(&mut out);
    }
    let mut served = List::with_capacity(1);
    served
        .push(
            w::Served { owner: w::Owner::Jig, first: VERSION, last: VERSION },
        )
        .expect("one family");
    stream(
        &mut machine,
        &jig_client_test_records::opened(
            w::Hello { stream: [7; 16], served, heartbeat: Duration::from_secs(1) },
            jig_client_test_records::frame_record(),
        ),
        &mut out,
    );
    let watch = 0;
    discard(&mut out);
    let mut parts = jig_client_test_records::frame_record();
    parts.retention = Duration::from_secs(60);
    let me = w::DeliveryValue::Me(parts);
    let mut writer = Writer::new(skein_lib::length::usize_of(me.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")));
    me.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("measured me");
    let document =
        w::Owned { owner: w::Owner::Jig, version: VERSION, record: writer.finish() };
    stream(
        &mut machine,
        &w::StreamEvent::Delivery(
            w::Delivery { watch, document },
        ),
        &mut out,
    );
    discard(&mut out);
    for event in [Event::Stored, send(FIRST), Event::Stored] {
        machine.step(Time::ZERO, Wall::EPOCH, event, &mut out);
        machine.settle(Time::ZERO, Wall::EPOCH, &mut out);
    }
    let exchange = out
        .iter()
        .find_map(|request| {
            let Request::Exchange { exchange, message: Message::Request(_), .. } = request else { return None };
            Some(*exchange)
        })
        .expect("first request sent");
    discard(&mut out);
    let done = w::Final::Done(
        w::Done {
                outcome: w::Outcome::Document(
                    w::OutcomeDocument { value: w::DocumentOutcome::AccountChanged },
                ),
            },
    );
    let mut writer = Writer::new(skein_lib::length::usize_of(done.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")));
    done.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("measured final answer");
    let owned =
        w::Owned { owner: w::Owner::Jig, version: VERSION, record: writer.finish() };
    let answer = w::KeyedAnswer::FinalAnswer(owned);
    let mut writer = Writer::new(skein_lib::length::usize_of(answer.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")));
    answer.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("measured reply");
    machine.step(Time::ZERO, Wall::EPOCH, Event::Reply { exchange, message: writer.finish() }, &mut out);
    machine.settle(Time::ZERO, Wall::EPOCH, &mut out);
    machine.step(Time::ZERO, Wall::EPOCH, send(NEXT), &mut out);
    machine.settle(Time::ZERO, Wall::EPOCH, &mut out);
    (machine, out)
}
