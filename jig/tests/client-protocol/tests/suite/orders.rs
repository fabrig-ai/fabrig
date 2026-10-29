//! Durability and exchange order in a world with a scripted parent and slow store.
//! See skein/docs/design/draft/dynamic-memory.md, 4.6.
use jig_client_domain::{Ask, Asker, Held, Key, Readable, contract};
use jig_client_protocol_link::{Event, Link, Message, Record, Request, Saved};
use jig_client_protocol_wire::v19 as w;
use jig_client_protocol_world::fixtures;
use skein_lib::{Duration, Queue, Reader, RequestKey, RequestRecord, Time, Token, Writer};
use std::collections::BTreeMap;

// One occupied read plus three keyed exchanges fills this transport edge,
// within the parent fixture's four-request table, including the replacement.
const EXCHANGES: u32 = 4;
const BLOCKER: Token = Token::new(100);
const PROJECT: u32 = 1;
const FIRST: u8 = 2;
const CANCELLED: u8 = 3;
const LAST: u8 = 4;
const REPLACEMENT: u8 = 5;
const STORE_DELAY: Duration = Duration::from_millis(5);
// Three original writes and the replacement's write await the same store barrier.
const PENDING_WRITES: usize = 4;
const OCCUPIED_READS: u32 = 1;

struct World {
    link: Link,
    now: Time,
    out: Queue<Request>,
    pending: BTreeMap<Box<[u8]>, Record>,
    durable: BTreeMap<Box<[u8]>, Record>,
    sent: Vec<RequestKey>,
    exchanges: Vec<Token>,
    terminals: Vec<(Token, &'static str)>,
    loads: Vec<(u32, usize)>,
}

impl World {
    fn loaded() -> Self {
        let limits =
            jig_client_protocol_link::Limits::new(fixtures::domain_limits(), EXCHANGES).expect("fixture exchange edge");
        let mut world = Self {
            link: Link::new(&limits).expect("fixture link"),
            now: Time::ZERO,
            out: Queue::uncapped(),
            pending: BTreeMap::new(),
            durable: BTreeMap::new(),
            sent: vec![],
            exchanges: vec![],
            terminals: vec![],
            loads: vec![],
        };
        world.event(Event::Read { read: BLOCKER, what: Readable::Jig(contract::Read::Me) });
        // A composing parent's turn holds its outputs until every record is
        // gathered. Observe the issued exchange name before Save gathers it;
        // cancellation never guesses a private exchange identity.
        for number in FIRST..=LAST {
            world.link.step(world.now, send(number), &mut world.out);
        }
        let mut cancelled = None;
        for request in &world.out {
            if let Request::Exchange { exchange, message: Message::Request(bytes), .. } = request {
                let keyed = w::Keyed::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("keyed output");
                if keyed.key() == &[CANCELLED; 16] {
                    cancelled = Some(*exchange);
                }
            }
        }
        for number in FIRST..=LAST {
            world.link.step(world.now, Event::Save { record: saved(number) }, &mut world.out);
        }
        world.drain();
        world.event(Event::Lost { exchange: cancelled.expect("issued cancelled exchange") });
        world.event(Event::Save { record: saved(REPLACEMENT) });
        world.event(send(REPLACEMENT));
        world.now = world.now.saturating_add(STORE_DELAY);
        world.link.fire(world.now, &mut world.out);
        world.drain();
        world.loads.push((world.link.in_flight(), world.pending.len()));
        world
    }

    fn event(&mut self, event: Event) {
        self.link.step(self.now, event, &mut self.out);
        self.drain();
    }

    fn drain(&mut self) {
        while let Some(request) = self.out.pop() {
            match request {
                Request::Save { record } => {
                    self.pending.insert(record.key.clone(), record);
                }
                Request::Exchange { exchange, message: Message::Request(bytes), .. } => {
                    let keyed = w::Keyed::decode(&w::CEILINGS, &mut Reader::new(&bytes)).expect("keyed wire bytes");
                    assert!(self.durable.contains_key(keyed.key().as_slice()), "no keyed send before durability");
                    self.sent.push(*keyed.key());
                    self.exchanges.push(exchange);
                }
                Request::Exchange { message: Message::Read(_), .. } => {}
                Request::Answer { attempt, .. } => self.terminals.push((attempt, "answer")),
                Request::NotKept { attempt } => self.terminals.push((attempt, "not kept")),
                other @ (Request::Exchange { .. }
                | Request::Authorized(_)
                | Request::Erase { .. }
                | Request::Kept { .. }
                | Request::Dropped
                | Request::ServiceCredential { .. }
                | Request::OpenStream { .. }
                | Request::Signing { .. }
                | Request::CloseStream
                | Request::Watch { .. }
                | Request::FamilyDelivery { .. }
                | Request::Ended { .. }
                | Request::Hello { .. }
                | Request::Read { .. }
                | Request::Fact { .. }) => panic!("unplanned neighbor request: {other:?}"),
            }
        }
    }

    fn stored(&mut self) {
        self.durable.append(&mut self.pending);
        self.event(Event::Stored);
    }

    fn answer(&mut self, exchange: Token) {
        let answer = w::KeyedAnswer::Busy;
        let mut bytes = Writer::new(usize::try_from(answer.measure()).expect("busy answer size"));
        answer.encode(&mut bytes).expect("measured busy answer");
        self.event(Event::Reply { exchange, message: bytes.finish() });
    }
}

fn send(number: u8) -> Event {
    Event::Send {
        attempt: Token::new(u64::from(number)),
        key: [number; 16],
        ask: Ask::Jig(Box::new(contract::Ask::Stop { project: PROJECT, task: u64::from(number) })),
        value: None,
    }
}

fn saved(number: u8) -> Saved {
    let Event::Send { key, ask, .. } = send(number) else { unreachable!("send fixture") };
    Saved {
        version: jig_client_domain::TABLE_VERSION,
        record: RequestRecord {
            key,
            scope: u64::from(PROJECT),
            first_sent: None,
            request: Held { key: Key { asker: Asker::JIG, number: u64::from(number) }, request: ask },
        },
    }
}

fn terminals(outcome: &'static str) -> Vec<(Token, &'static str)> {
    vec![(Token::new(3), "answer"), (Token::new(2), outcome), (Token::new(4), outcome), (Token::new(5), outcome)]
}

/// `request_records_durable` and `held_exchanges_fifo`: with all exchange
/// credit occupied, delayed persistence releases survivors once, in arrival
/// order, skipping a cancelled attempt even when its credit is reused.
#[test]
fn held_exchanges_wait_for_durability_and_leave_in_order_under_load() {
    let mut world = World::loaded();
    assert!(world.sent.is_empty(), "all admitted keyed requests remain held during the delay");
    assert_eq!(world.loads, [(EXCHANGES, PENDING_WRITES)]);
    world.stored();
    for exchange in world.exchanges.clone() {
        world.answer(exchange);
        world.answer(exchange);
    }
    world.event(Event::Stored);
    world.event(Event::NotStored);
    assert_eq!(world.sent, [[2; 16], [4; 16], [5; 16]]);
    assert_eq!(world.terminals, terminals("answer"));
    // Only the intentionally unanswered read keeps its exchange credit.
    assert_eq!(world.link.in_flight(), OCCUPIED_READS);
}

/// `request_records_durable`: failure terminates the surviving held attempts
/// without sending them; late store completions cannot release stale entries.
#[test]
fn failed_persistence_never_releases_held_exchanges_under_load() {
    let mut world = World::loaded();
    world.event(Event::NotStored);
    world.stored();
    world.event(Event::NotStored);
    assert!(world.sent.is_empty());
    assert_eq!(world.loads, [(EXCHANGES, PENDING_WRITES)]);
    assert_eq!(world.terminals, terminals("not kept"));
    assert_eq!(world.link.in_flight(), OCCUPIED_READS);
}
