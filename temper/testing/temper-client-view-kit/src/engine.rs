//! A wave-at-a-time engine double for linked page budgets (client/pages/work.md, 1).
use jig_client::{Event, Message, Request, Streamed};
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
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
        Some(number) => (w::Owner::Family(w::Family { number }), temper_client_protocol_wire::CLIENT_VERSION),
        None => (w::Owner::Jig, w::VERSION),
    };
    w::Owned { owner, version, record }
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
                    for (place, member) in batch.members.as_slice().iter().enumerate() {
                        let mut deliveries = Vec::new();
                        let reply = Self::member(member, &answer, &mut deliveries);
                        let part = w::BatchReply { member: u16::try_from(place).expect("member"), reply: Some(reply) };
                        events.push(Event::Part {
                            exchange: *exchange,
                            message: part.encode(&w::CEILINGS).expect("fixture encoding"),
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
                w::Owner::Family(w::Family { number: temper_client_protocol_wire::FORGE }),
                temper_client_protocol_wire::CLIENT_VERSION,
            ),
        ] {
            served.push(w::Served { owner, first: version, last: version }).expect("two owners");
        }
        let hello = w::Hello { stream: [7; 16], served, heartbeat: Duration::from_secs(30) };
        streamed(&jig_client_test_records::opened_with_layouts(
            hello,
            self.me.clone(),
            &[jig_client_test_records::family_layout(
                temper_client_protocol_wire::FORGE,
                temper_client_protocol_wire::CLIENT_LAYOUT,
            )],
        ))
    }
    fn member(member: &w::Member, answer: &impl Fn(&w::Owned, bool) -> w::Owned, events: &mut Vec<Event>) -> Box<[u8]> {
        match member {
            w::Member::Read(read) => {
                let reply = w::ReadAnswer::Value(answer(read, false));
                reply.encode(&w::CEILINGS).expect("fixture encoding")
            }
            w::Member::Open(open) => {
                let document = answer(&open.subject, true);
                let delivery = w::StreamEvent::Delivery(w::Delivery { watch: open.watch, document });
                events.push(streamed(&delivery));
                w::OpenAnswer::Admitted.encode(&w::CEILINGS).expect("fixture encoding")
            }
            w::Member::Close(_) => w::CloseAnswer::Closed.encode(&w::CEILINGS).expect("fixture encoding"),
        }
    }
}
#[must_use]
pub fn streamed(record: &w::StreamEvent) -> Event {
    Event::Streamed { message: Streamed::Record(record.encode(&w::CEILINGS).expect("fixture encoding")) }
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
