//! Signal intake and descriptor lifecycle (runtime/io/README.md, 4; programming-model.md,
//! 5.3 and 5.4). The owner decides what a `Shutdown` means.
//!
//! | State | Event | Next state and output |
//! |---|---|---|
//! | Open | resumed | Reading: submit `ReadSignal` |
//! | Open | Close or Abort | Releasing: submit `Close` |
//! | Reading | signal | Reading: `Shutdown`, submit another read |
//! | Reading | read error | Releasing: `Failed`, submit `Close` |
//! | Reading | Close or Abort | Settling: cancel the read |
//! | Settling | cancel unsubmitted | Settling: retry only while its read waits |
//! | Settling | read and cancel completed | Releasing: submit `Close` |
//! | Releasing | close completed | Closed: `Closed` |

use core::mem;

use skein_lib::{Id, Queue, Token};

use crate::kernel::{Done, Fd, Op, Submit};
use crate::layer::{Entity, Flight, Landed, Purpose, Tables};
use crate::records::{Error, Event};

#[derive(Debug)]
pub(crate) struct Signals {
    owner: Option<Token>,
    state: State,
}

#[derive(Debug)]
enum State {
    Open { fd: Fd },
    Reading { fd: Fd, read: Id<Flight> },
    Settling { fd: Fd, read: Option<Id<Flight>>, cancels: u32 },
    Releasing { close: Id<Flight> },
    Closed,
}

impl Signals {
    pub(crate) fn new(fd: Fd, owner: Option<Token>) -> Signals {
        Signals { owner, state: State::Open { fd } }
    }

    pub(crate) const fn is_closed(&self) -> bool {
        match self.state {
            State::Closed => true,
            State::Open { .. } | State::Reading { .. } | State::Settling { .. } | State::Releasing { .. } => false,
        }
    }
}

pub(crate) fn resume(signals: &mut Signals, id: Id<Entity>, tables: &mut Tables, subs: &mut Queue<Submit>) {
    follow(signals, id, tables, subs);
}

pub(crate) fn landed(
    signals: &mut Signals,
    id: Id<Entity>,
    landed: Landed,
    tables: &mut Tables,
    up: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) {
    let state = mem::replace(&mut signals.state, State::Closed);
    signals.state = match landed.purpose {
        Purpose::ReadSignal => match state {
            State::Reading { fd, read } => {
                assert!(read == landed.flight, "the signal read flight matches");
                read_landed(fd, signals.owner.unwrap_or(id.token()), landed, tables, up, subs)
            }
            State::Settling { fd, read, cancels } => {
                assert!(read == Some(landed.flight), "the signal read flight matches");
                State::Settling { fd, read: None, cancels }
            }
            State::Open { .. } | State::Releasing { .. } | State::Closed => unreachable!("no signal read in flight"),
        },
        Purpose::Cancel(target) => match state {
            State::Settling { fd, read, mut cancels } => {
                cancels = cancels.checked_sub(1).expect("one signal cancel in flight");
                // Retry an unsubmitted cancel only while its read waits (runtime/kernel.md, 5).
                if crate::layer::unsubmitted(landed.result) && read == Some(target) {
                    tables.cancel(subs, id, target);
                    cancels = cancels.checked_add(1).expect("one signal cancel per target");
                }
                State::Settling { fd, read, cancels }
            }
            State::Open { .. } | State::Reading { .. } | State::Releasing { .. } | State::Closed => {
                unreachable!("a signal cancel completes only while settling")
            }
        },
        Purpose::Close => match state {
            State::Releasing { close } => {
                assert!(close == landed.flight, "the signal close flight matches");
                up.push(Event::Closed { owner: signals.owner.unwrap_or(id.token()) });
                State::Closed
            }
            State::Open { .. } | State::Reading { .. } | State::Settling { .. } | State::Closed => {
                unreachable!("a signal close completes only while releasing")
            }
        },
        Purpose::Socket
        | Purpose::Bind
        | Purpose::Listen
        | Purpose::Accept
        | Purpose::Connect
        | Purpose::Recv
        | Purpose::Send
        | Purpose::Shutdown
        | Purpose::Discard
        | Purpose::Spawn
        | Purpose::Wait
        | Purpose::Signal
        | Purpose::Window { .. }
        | Purpose::Usage { .. }
        | Purpose::PipeRead
        | Purpose::PipeWrite => unreachable!("a signal source only reads, cancels and closes"),
    };
    follow(signals, id, tables, subs);
}

fn read_landed(
    fd: Fd,
    owner: Token,
    landed: Landed,
    tables: &mut Tables,
    up: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> State {
    match landed.result {
        Ok(Done::Signalled(crate::kernel::Signalled::Termination { signal, sender })) => {
            let mut from = None;
            if let Some(sender) = sender {
                for (child, identity) in &tables.senders {
                    if *identity == sender {
                        from = Some(*child);
                        break;
                    }
                }
            }
            up.push(Event::Shutdown { signal, from });
        }
        Ok(Done::Signalled(crate::kernel::Signalled::WindowChanged)) => up.push(Event::Resized),
        Err(_) => {
            up.push(Event::Failed { owner, error: Error::Other });
            return release(fd, landed.entity, tables, subs);
        }
        Ok(
            Done::Nothing
            | Done::Count(_)
            | Done::Fd(_)
            | Done::Accepted { .. }
            | Done::Bound(_)
            | Done::Stat(_)
            | Done::Spawned { .. }
            | Done::Exit(_)
            | Done::Window(_)
            | Done::Usage(_),
        ) => unreachable!("a signal read answers with a service signal"),
    }
    State::Open { fd }
}

pub(crate) fn close(signals: &mut Signals, id: Id<Entity>, tables: &mut Tables, subs: &mut Queue<Submit>) {
    let state = mem::replace(&mut signals.state, State::Closed);
    signals.state = match state {
        State::Open { fd } => release(fd, id, tables, subs),
        State::Reading { fd, read } => {
            tables.cancel(subs, id, read);
            State::Settling { fd, read: Some(read), cancels: 1 }
        }
        State::Settling { .. } | State::Releasing { .. } | State::Closed => state,
    };
    follow(signals, id, tables, subs);
}

/// The state's demand: arm an open source, or release it once settled.
fn follow(signals: &mut Signals, id: Id<Entity>, tables: &mut Tables, subs: &mut Queue<Submit>) {
    let state = mem::replace(&mut signals.state, State::Closed);
    signals.state = match state {
        State::Open { fd } => {
            let read = tables.submit(subs, id, Purpose::ReadSignal, Op::ReadSignal { fd });
            State::Reading { fd, read }
        }
        State::Settling { fd, read: None, cancels: 0 } => release(fd, id, tables, subs),
        State::Reading { .. } | State::Settling { .. } | State::Releasing { .. } | State::Closed => state,
    };
}

fn release(fd: Fd, id: Id<Entity>, tables: &mut Tables, subs: &mut Queue<Submit>) -> State {
    State::Releasing { close: tables.submit(subs, id, Purpose::Close, Op::Close { fd }) }
}
