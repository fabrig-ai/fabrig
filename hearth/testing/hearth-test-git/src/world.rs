use hearth_protocol_git::{
    limits::Limits,
    listing::{self, Entry, Error, Event, Listing, Request},
};
use skein_lib::{
    Env, Queue, Time, Wall,
    stream::{Fault, Read, Up},
};
use skein_world::stream::Wire;

#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub seed: u64,
    pub whole: bool,
    pub end: Option<usize>,
    pub fail: bool,
    pub close: Option<u32>,
}
impl Settings {
    #[must_use]
    pub fn whole() -> Self {
        Self { whole: true, ..Self::cut(0) }
    }
    #[must_use]
    pub fn cut(seed: u64) -> Self {
        Self { seed, whole: false, end: None, fail: false, close: None }
    }
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Outcome {
    Done(Box<[Entry]>),
    Failed(Error),
    Closed,
}
/// Play a List and Close above and a seeded fragmented Wire below.
#[must_use]
pub fn list(stdout: &[u8], limits: &Limits, settings: &Settings) -> Outcome {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut machine = Listing::new(limits);
    let mut wire = Source::new(stdout, settings);
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    if settings.close == Some(0) {
        assert_closed(&mut machine, &env, &mut wire);
        return Outcome::Closed;
    }
    listing::down(&mut machine, &env, Request::List, &mut above, &mut below);
    let mut calls = 1;
    for _ in 0..stdout.len().saturating_mul(20).saturating_add(100) {
        while let Some(down) = below.pop() {
            wire.take(down);
        }
        if let Some(event) = above.pop() {
            let outcome = match event {
                Event::Listed(entries) => Outcome::Done(entries),
                Event::Failed(error) => Outcome::Failed(error),
                Event::Closed => Outcome::Closed,
            };
            assert_closed(&mut machine, &env, &mut wire);
            return outcome;
        }
        if settings.close == Some(calls) {
            assert_closed(&mut machine, &env, &mut wire);
            return Outcome::Closed;
        }
        if let Some(up) = wire.answer() {
            let up = match up {
                Up::End if settings.fail => Up::Failed(Fault::Reset),
                other @ (Up::Bytes(_) | Up::Room | Up::End | Up::Failed(_)) => other,
            };
            listing::up(&mut machine, &env, up, &mut above, &mut below);
            calls += 1;
        }
    }
    panic!("listing did not settle for {settings:?}")
}
fn assert_closed(machine: &mut Listing, env: &Env<Limits>, wire: &mut Source) {
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    listing::down(machine, env, Request::Close, &mut above, &mut below);
    assert_eq!(above.pop(), Some(Event::Closed), "Close answers Closed");
    while let Some(down) = below.pop() {
        assert_eq!(down, skein_lib::stream::Down::Demand { read: Read::Nothing, room: 0 }, "close only withdraws");
        wire.take(down);
    }
    assert_eq!(wire.answer(), None, "closed has no demand");
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Blob {
    pub oid: hearth_protocol_git::oid::Oid,
    pub bytes: Option<Box<[u8]>>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BatchOutcome {
    Done(Box<[Blob]>),
    Failed(hearth_protocol_git::batch::Error),
    Closed,
}
/// Play binary blobs through a Wire, delaying Next from the seed.
#[must_use]
pub fn read(
    stdout: &[u8],
    requests: &[hearth_protocol_git::batch::Request],
    limits: &Limits,
    settings: &Settings,
) -> BatchOutcome {
    use hearth_protocol_git::batch::{self, Batch, Event, Request};
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut machine = Batch::START;
    let mut wire = Source::new(stdout, settings);
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    let mut requests = requests.iter();
    let mut pending = requests.next().copied();
    let mut result = Vec::new();
    let mut bytes = Vec::new();
    let mut rng = skein_lib::Rng::new(settings.seed);
    let mut calls = 0;
    for _ in 0..stdout.len().saturating_mul(30).saturating_add(100) {
        while let Some(down) = below.pop() {
            wire.take(down);
        }
        if settings.close == Some(calls) {
            batch_closed(&mut machine, &env, &mut wire);
            return BatchOutcome::Closed;
        }
        if let Some(event) = above.pop() {
            match event {
                Event::Piece(piece) => {
                    assert!(piece.len() <= usize::try_from(limits.chunk).expect("chunk fits"), "bounded piece");
                    bytes.extend_from_slice(&piece);
                    pending = Some(Request::Next);
                }
                Event::Whole { oid, size } => {
                    assert_eq!(u64::try_from(bytes.len()).expect("body fits"), size, "whole body size");
                    result.push(Blob { oid, bytes: Some(core::mem::take(&mut bytes).into_boxed_slice()) });
                    pending = requests.next().copied();
                    if pending.is_none() {
                        batch_closed(&mut machine, &env, &mut wire);
                        return BatchOutcome::Done(result.into_boxed_slice());
                    }
                }
                Event::Missing { oid } => {
                    assert!(bytes.is_empty(), "missing has no body");
                    result.push(Blob { oid, bytes: None });
                    pending = requests.next().copied();
                    if pending.is_none() {
                        batch_closed(&mut machine, &env, &mut wire);
                        return BatchOutcome::Done(result.into_boxed_slice());
                    }
                }
                Event::Failed(error) => {
                    batch_closed(&mut machine, &env, &mut wire);
                    return BatchOutcome::Failed(error);
                }
                Event::Closed => return BatchOutcome::Closed,
            }
        }
        if let Some(request) = pending {
            if rng.chance(500) {
                pending = None;
                batch::down(&mut machine, &env, request, &mut above, &mut below);
                calls += 1;
            }
        } else if let Some(event) = wire.answer() {
            let event = match event {
                Up::End if settings.fail => Up::Failed(Fault::Reset),
                other @ (Up::Bytes(_) | Up::Room | Up::End | Up::Failed(_)) => other,
            };
            batch::up(&mut machine, &env, event, &mut above, &mut below);
            if above.iter().any(|event| matches!(event, Event::Piece(_))) {
                assert!(below.is_empty(), "a piece is taken before the next read is demanded");
            }
            calls += 1;
        }
    }
    panic!("batch did not settle for {settings:?}")
}
fn batch_closed(machine: &mut hearth_protocol_git::batch::Batch, env: &Env<Limits>, wire: &mut Source) {
    use hearth_protocol_git::batch::{self, Event, Request};
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    batch::down(machine, env, Request::Close, &mut above, &mut below);
    assert_eq!(above.pop(), Some(Event::Closed), "Close answers Closed");
    while let Some(down) = below.pop() {
        assert_eq!(down, skein_lib::stream::Down::Demand { read: Read::Nothing, room: 0 }, "close only withdraws");
        wire.take(down);
    }
    assert_eq!(wire.answer(), None, "closed has no demand");
}

pub use crate::replay::replay;

#[derive(Debug)]
pub(crate) enum Source {
    Cut(Wire),
    Quiet {
        input: skein_lib::Intake,
        pending: Box<[u8]>,
        cursor: usize,
        rng: skein_lib::Rng,
        demand: Option<Read>,
        ended: bool,
    },
    Whole {
        held: skein_lib::stream::Held,
        demand: Option<Read>,
    },
}
impl Source {
    pub(crate) fn quiet(stdout: &[u8], settings: &Settings) -> Self {
        if settings.whole {
            return Self::new(stdout, settings);
        }
        let bytes = &stdout[..settings.end.unwrap_or(stdout.len()).min(stdout.len())];
        Self::Quiet {
            input: skein_lib::Intake::with_capacity(32768),
            pending: bytes.into(),
            cursor: 0,
            rng: skein_lib::Rng::new(settings.seed),
            demand: None,
            ended: false,
        }
    }

    pub(crate) fn new(stdout: &[u8], settings: &Settings) -> Self {
        let bytes = &stdout[..settings.end.unwrap_or(stdout.len()).min(stdout.len())];
        if settings.whole {
            Self::Whole { held: skein_lib::stream::Held::new(bytes.into()), demand: None }
        } else {
            let mut wire = Wire::new(settings.seed);
            wire.write(bytes);
            wire.eof = true;
            Self::Cut(wire)
        }
    }
    pub(crate) fn take(&mut self, down: skein_lib::stream::Down) {
        match self {
            Self::Cut(wire) => wire.take(down),
            Self::Quiet { demand, .. } => match down {
                skein_lib::stream::Down::Demand { read: Read::Nothing, room: 0 } => *demand = None,
                skein_lib::stream::Down::Demand { read, room: 0 } => {
                    assert!(demand.is_none(), "one demand");
                    *demand = Some(read);
                }
                skein_lib::stream::Down::Demand { .. }
                | skein_lib::stream::Down::Send(_)
                | skein_lib::stream::Down::Finish => unreachable!("read-only quiet source"),
            },
            Self::Whole { demand, .. } => match down {
                skein_lib::stream::Down::Demand { read: Read::Nothing, room: 0 } => *demand = None,
                skein_lib::stream::Down::Demand { read, room: 0 } => *demand = Some(read),
                skein_lib::stream::Down::Demand { .. }
                | skein_lib::stream::Down::Send(_)
                | skein_lib::stream::Down::Finish => unreachable!("read-only source"),
            },
        }
    }
    pub(crate) fn answer(&mut self) -> Option<Up> {
        match self {
            Self::Cut(wire) => wire.answer(),
            Self::Quiet { input, pending, cursor, rng, demand, ended } => {
                if *cursor < pending.len() && rng.chance(700) {
                    let count =
                        usize::try_from(rng.between(1, 31)).expect("small fragment").min(pending.len() - *cursor);
                    input.append(&pending[*cursor..*cursor + count]).expect("bounded intake");
                    *cursor += count;
                }
                let read = (*demand)?;
                if let Some(bytes) = input.meet(read) {
                    *demand = None;
                    Some(Up::Bytes(bytes))
                } else if *cursor == pending.len() && !*ended {
                    *ended = true;
                    Some(Up::End)
                } else {
                    None
                }
            }
            Self::Whole { held, demand } => demand.take().and_then(|read| held.answer(read)),
        }
    }
}

pub use crate::hunks::{DiffOutcome, diff};
