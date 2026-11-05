pub use hearth_test_git::world::*;

use hearth_protocol_git::{
    limits::Limits,
    listing::{self, Event, Listing, Request},
};
use skein_lib::{
    Env, Queue, Time, Wall,
    stream::{Read, Up},
};

/// Admit stderr in seeded cuts, without involving another decoder.
#[must_use]
pub fn tail(stderr: &[u8], limits: &Limits, seed: u64) -> Box<[Box<[u8]>]> {
    let mut machine = hearth_protocol_git::tail::Tail::new(limits);
    let mut rng = skein_lib::Rng::new(seed);
    let mut rest = stderr;
    while !rest.is_empty() {
        let count = usize::try_from(rng.between(1, u64::from(limits.chunk.max(1)))).expect("cut fits").min(rest.len());
        let (cut, remaining) = rest.split_at(count);
        machine.admit(cut);
        rest = remaining;
    }
    machine.lines()
}
/// A held-input driver with exactly one decoder entry point per call.
/// `pending` prepares deliveries outside the meter; `drop_events` hands outputs out.
#[derive(Debug)]
pub struct Stepped {
    env: Env<Limits>,
    decoder: hearth_protocol_git::limits::Decoder,
    listing: Option<Listing>,
    batch: Option<hearth_protocol_git::batch::Batch>,
    diff: Option<hearth_protocol_git::hunks::Diff>,
    tail: Option<hearth_protocol_git::tail::Tail>,
    diff_events: Queue<hearth_protocol_git::hunks::Event>,
    held: skein_lib::stream::Held,
    feed: Box<[u8]>,
    requests: std::vec::IntoIter<hearth_protocol_git::batch::Request>,
    listing_events: Queue<Event>,
    batch_events: Queue<hearth_protocol_git::batch::Event>,
    below: Queue<skein_lib::stream::Down>,
    read: Option<Read>,
    action: Option<Action>,
    closed: bool,
}
#[derive(Debug)]
enum Action {
    Diff,
    Feed,
    List,
    Batch(hearth_protocol_git::batch::Request),
    Up(Up),
    Close,
}
impl Stepped {
    #[must_use]
    pub fn new(case: &crate::fixtures::Case, limits: &Limits) -> Self {
        Self {
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits },
            decoder: case.decoder,
            listing: None,
            batch: None,
            diff: None,
            tail: None,
            diff_events: Queue::uncapped(),
            held: skein_lib::stream::Held::new(case.stdout.clone().into_boxed_slice()),
            feed: case.stdout.clone().into_boxed_slice(),
            requests: Vec::from(case.requests.clone()).into_iter(),
            listing_events: Queue::uncapped(),
            batch_events: Queue::uncapped(),
            below: Queue::uncapped(),
            read: None,
            action: None,
            closed: false,
        }
    }
    pub fn open(&mut self) {
        use hearth_protocol_git::{batch::Batch, limits::Decoder};
        match self.decoder {
            Decoder::Tail => {
                self.tail = Some(hearth_protocol_git::tail::Tail::new(&self.env.limits));
                self.action = Some(Action::Feed);
            }
            Decoder::Diff => {
                self.diff = Some(hearth_protocol_git::hunks::Diff::new(&self.env.limits));
                self.action = Some(Action::Diff);
            }
            Decoder::Listing => {
                self.listing = Some(Listing::new(&self.env.limits));
                self.action = Some(Action::List);
            }
            Decoder::Batch => {
                self.batch = Some(Batch::START);
                self.action = Some(Action::Batch(self.requests.next().expect("batch case has request")));
            }
        }
    }
    pub fn pending(&mut self) -> bool {
        if self.closed {
            return false;
        }
        if self.action.is_none() {
            let read = self.read.take().expect("unfinished machine demands input");
            self.action = Some(Action::Up(self.held.answer(read).expect("active held demand")));
        }
        true
    }
    pub fn call(&mut self) {
        use hearth_protocol_git::{batch, limits::Decoder};
        let action = self.action.take().expect("prepared call");
        match self.decoder {
            Decoder::Tail => {
                self.tail.as_mut().expect("open tail").admit(&self.feed);
                self.closed = true;
            }
            Decoder::Diff => {
                use hearth_protocol_git::hunks::{self, Request};
                let machine = self.diff.as_mut().expect("open diff");
                match action {
                    Action::Diff => {
                        hunks::down(machine, &self.env, Request::Read, &mut self.diff_events, &mut self.below);
                    }
                    Action::Close => {
                        hunks::down(machine, &self.env, Request::Close, &mut self.diff_events, &mut self.below);
                    }
                    Action::Up(up) => hunks::up(machine, &self.env, up, &mut self.diff_events, &mut self.below),
                    Action::Feed | Action::List | Action::Batch(_) => unreachable!("diff driver"),
                }
            }
            Decoder::Listing => {
                let machine = self.listing.as_mut().expect("open listing");
                match action {
                    Action::List => {
                        listing::down(machine, &self.env, Request::List, &mut self.listing_events, &mut self.below);
                    }
                    Action::Close => {
                        listing::down(machine, &self.env, Request::Close, &mut self.listing_events, &mut self.below);
                    }
                    Action::Up(up) => listing::up(machine, &self.env, up, &mut self.listing_events, &mut self.below),
                    Action::Feed | Action::Batch(_) | Action::Diff => unreachable!("listing driver"),
                }
            }
            Decoder::Batch => {
                let machine = self.batch.as_mut().expect("open batch");
                match action {
                    Action::Batch(request) => {
                        batch::down(machine, &self.env, request, &mut self.batch_events, &mut self.below);
                    }
                    Action::Close => {
                        batch::down(machine, &self.env, batch::Request::Close, &mut self.batch_events, &mut self.below);
                    }
                    Action::Up(up) => batch::up(machine, &self.env, up, &mut self.batch_events, &mut self.below),
                    Action::Feed | Action::List | Action::Diff => unreachable!("batch driver"),
                }
            }
        }
    }
    pub fn drop_events(&mut self) {
        use hearth_protocol_git::batch;
        if let Some(event) = self.diff_events.pop() {
            match event {
                hearth_protocol_git::hunks::Event::Hunks(_) | hearth_protocol_git::hunks::Event::Failed(_) => {
                    self.action = Some(Action::Close);
                }
                hearth_protocol_git::hunks::Event::Closed => self.closed = true,
            }
            drop(event);
        }
        if let Some(event) = self.listing_events.pop() {
            match event {
                Event::Listed(_) | Event::Failed(_) => self.action = Some(Action::Close),
                Event::Closed => self.closed = true,
            }
            drop(event);
        }
        if let Some(event) = self.batch_events.pop() {
            match event {
                batch::Event::Piece(_) => self.action = Some(Action::Batch(batch::Request::Next)),
                batch::Event::Whole { .. } | batch::Event::Missing { .. } => {
                    self.action = Some(match self.requests.next() {
                        Some(request) => Action::Batch(request),
                        None => Action::Close,
                    });
                }
                batch::Event::Failed(_) => self.action = Some(Action::Close),
                batch::Event::Closed => self.closed = true,
            }
            drop(event);
        }
        if let Some(down) = self.below.pop() {
            match down {
                skein_lib::stream::Down::Demand { read: Read::Nothing, room: 0 } => self.read = None,
                skein_lib::stream::Down::Demand { read, room: 0 } => self.read = Some(read),
                skein_lib::stream::Down::Demand { .. }
                | skein_lib::stream::Down::Send(_)
                | skein_lib::stream::Down::Finish => unreachable!("read only"),
            }
        }
    }
}
