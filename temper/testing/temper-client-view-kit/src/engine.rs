//! A wave-at-a-time engine double for linked page budgets (client/pages/work.md, 1).
use jig_client::{Event, Message, Request, Streamed};
use jig_client_protocol_wire as w;
use skein_lib::{Duration, List, Reader, Writer};

/// Encode typed fixture values as the engine does, without a transport.
pub fn encoded(size: u32, write: impl FnOnce(&mut Writer) -> Result<(), skein_lib::Overflow>) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(size).expect("fixture size"));
    write(&mut writer).expect("fixture encoding");
    writer.finish()
}

#[must_use]
pub fn owned(record: Box<[u8]>, family: Option<u16>) -> w::Owned {
    let (owner, version) = match family {
        Some(number) => (
            w::Owner::Family(w::Family::new(&w::CEILINGS, w::FamilyParts { number }).expect("family")),
            temper_client_protocol_wire::CLIENT_VERSION,
        ),
        None => (w::Owner::Jig, w::VERSION),
    };
    w::Owned::new(&w::CEILINGS, w::OwnedParts { owner, version, record }).expect("owned fixture")
}

/// The fake leaves keyed requests unanswered: the page must not depend on them.
#[derive(Debug)]
pub struct Engine {
    pub me: w::Me,
    pub batches: Vec<w::Batch>,
    pub keyed: Vec<w::Keyed>,
    pub replaces: Vec<Option<[u8; 16]>>,
}
impl Engine {
    #[must_use]
    pub const fn new(me: w::Me) -> Self {
        Self { me, batches: Vec::new(), keyed: Vec::new(), replaces: Vec::new() }
    }
    /// Answer all outstanding exchanges, including each watch's first value, in one iteration.
    pub fn wave(&mut self, sent: &[Request], answer: impl Fn(&w::Owned, bool) -> w::Owned) -> Vec<Event> {
        let mut events = Vec::new();
        for request in sent {
            match request {
                Request::OpenStream { replaces, version, .. } => {
                    assert_eq!(*version, w::VERSION);
                    self.replaces.push(*replaces);
                    events.push(self.hello());
                }
                Request::Exchange { exchange, message: Message::Batch(body), .. } => {
                    let batch = w::Batch::decode(&w::CEILINGS, &mut Reader::new(body)).expect("batch");
                    for (place, member) in batch.members().as_slice().iter().enumerate() {
                        let mut deliveries = Vec::new();
                        let reply = Self::member(member, &answer, &mut deliveries);
                        let part = w::BatchReply::new(
                            &w::CEILINGS,
                            w::BatchReplyParts { member: u16::try_from(place).expect("member"), reply: Some(reply) },
                        )
                        .expect("batch reply");
                        events.push(Event::Part {
                            exchange: *exchange,
                            message: encoded(part.measure(), |w| part.encode(w)),
                        });
                        events.extend(deliveries);
                    }
                    events.push(Event::Done { exchange: *exchange });
                    self.batches.push(batch);
                }
                Request::Exchange { exchange, message, .. } => {
                    let member = match message {
                        Message::Read(body) => {
                            w::Member::Read(w::Owned::decode(&w::CEILINGS, &mut Reader::new(body)).expect("read"))
                        }
                        Message::Watch(body) => {
                            w::Member::Open(w::Open::decode(&w::CEILINGS, &mut Reader::new(body)).expect("open"))
                        }
                        Message::Unwatch(body) => w::Member::Close(
                            w::WatchClose::decode(&w::CEILINGS, &mut Reader::new(body)).expect("close"),
                        ),
                        Message::Request(body) => {
                            self.keyed.push(w::Keyed::decode(&w::CEILINGS, &mut Reader::new(body)).expect("keyed"));
                            continue;
                        }
                        other => panic!("unexpected exchange: {other:?}"),
                    };
                    let mut deliveries = Vec::new();
                    let message = Self::member(&member, &answer, &mut deliveries);
                    events.push(Event::Reply { exchange: *exchange, message });
                    events.extend(deliveries);
                }
                Request::Save { .. } | Request::Erase { .. } => {}
                other => panic!("unexpected engine request: {other:?}"),
            }
        }
        events
    }
    #[must_use]
    pub fn hello(&self) -> Event {
        let mut served = List::with_capacity(2);
        for (owner, version) in [
            (w::Owner::Jig, w::VERSION),
            (
                w::Owner::Family(
                    w::Family::new(&w::CEILINGS, w::FamilyParts { number: temper_client_protocol_wire::FORGE })
                        .expect("family"),
                ),
                temper_client_protocol_wire::CLIENT_VERSION,
            ),
        ] {
            served
                .push(
                    w::Served::new(&w::CEILINGS, w::ServedParts { owner, first: version, last: version })
                        .expect("served"),
                )
                .expect("two owners");
        }
        let hello =
            w::Hello::new(&w::CEILINGS, w::HelloParts { stream: [7; 16], served, heartbeat: Duration::from_secs(30) })
                .expect("hello");
        streamed(&jig_client_test_records::opened(hello, self.me.clone()))
    }
    fn member(member: &w::Member, answer: &impl Fn(&w::Owned, bool) -> w::Owned, events: &mut Vec<Event>) -> Box<[u8]> {
        match member {
            w::Member::Read(read) => {
                let reply = w::ReadAnswer::Value(answer(read, false));
                encoded(reply.measure(), |w| reply.encode(w))
            }
            w::Member::Open(open) => {
                let document = answer(open.subject(), true);
                let delivery = w::StreamEvent::Delivery(
                    w::Delivery::new(&w::CEILINGS, w::DeliveryParts { watch: open.watch(), document })
                        .expect("delivery"),
                );
                events.push(streamed(&delivery));
                encoded(w::OpenAnswer::Admitted.measure(), |w| w::OpenAnswer::Admitted.encode(w))
            }
            w::Member::Close(_) => encoded(w::CloseAnswer::Closed.measure(), |w| w::CloseAnswer::Closed.encode(w)),
        }
    }
}
#[must_use]
pub fn streamed(record: &w::StreamEvent) -> Event {
    Event::Streamed { message: Streamed::Record(encoded(record.measure(), |w| record.encode(w))) }
}

/// Wire request count, excluding durable storage and kept keyed routes.
#[must_use]
pub fn page_requests(sent: &[Request]) -> u32 {
    sent.iter()
        .filter(|request| {
            matches!(
                request,
                Request::OpenStream { .. }
                    | Request::Exchange {
                        message: Message::Batch(_) | Message::Read(_) | Message::Watch(_) | Message::Unwatch(_),
                        ..
                    }
            )
        })
        .count()
        .try_into()
        .expect("request count")
}
