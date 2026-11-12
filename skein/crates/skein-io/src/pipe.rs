//! One-way pipes and adopted append files (runtime/io/sockets.md and runtime/io/files.md, 1 and runtime/io/processes.md).
//! Data-carrying states follow programming-model.md, 5.4. Output writes
//! keep the owner's deadline; graceful closing replaces it with io's.

use alloc::boxed::Box;
use core::mem;
use skein_lib::stream::{Down, Fault, OutputDown, OutputOutcome, Read, Up};
use skein_lib::{Duration, Env, Id, Intake, Queue, bytes, usize_of};

use crate::kernel::{self, Done, Fd, Op, Submit, Way};
use crate::layer::{Entity, Flight, Landed, Purpose, Tables, Timer};
use crate::limits::Limits;
use crate::output::Reservation;
use crate::records::Event;

#[derive(Debug)]
pub(crate) struct Pipe {
    child: Option<Id<Entity>>,
    way: Way,
    append: bool,
    write_timeout: Option<Duration>,
    independent: Reservation,
    state: State,
}
#[derive(Debug)]
enum State {
    Pending,
    Adopted { fd: Fd, demand: Option<(Read, u32)> },
    Reading(Reading),
    Writing(Writing),
    Flushing(Flushing),
    Settling { fd: Fd, read: Option<Id<Flight>>, cancels: u32 },
    Releasing { close: Id<Flight> },
    Closed,
}
#[derive(Debug)]
struct Reading {
    fd: Fd,
    intake: Intake,
    demand: Option<Read>,
    source: Source,
    paused: bool,
    cancel: bool,
}
impl Reading {
    fn quiet(&self) -> bool {
        let receiving = match self.source {
            Source::Reading(_) => true,
            Source::Idle | Source::Eof | Source::Ended => false,
        };
        !self.cancel && !receiving
    }
}

#[derive(Debug)]
enum Source {
    Idle,
    Reading(Id<Flight>),
    Eof,
    Ended,
}
// Every live output byte is counted until its write completes.
#[derive(Debug)]
struct Output {
    fd: Fd,
    queue: Queue<Box<[u8]>>,
    bytes: u32,
    write: Option<Id<Flight>>,
}
#[derive(Debug)]
struct Writing {
    output: Output,
    granted: u32,
    demand: Option<u32>,
    finishing: Finish,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Finish {
    More,
    Finishing,
}
#[derive(Debug)]
struct Flushing {
    output: Output,
    cancels: u32,
    flush: Flush,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Flush {
    Whole,
    Spill,
}

impl Pipe {
    pub(crate) fn pending(way: Way) -> Pipe {
        match way {
            Way::In | Way::Out => {}
            Way::Given(_) | Way::Null => unreachable!("a given or null descriptor has no io pipe entity"),
        }
        Pipe {
            child: None,
            way,
            append: false,
            write_timeout: None,
            independent: Reservation::Idle,
            state: State::Pending,
        }
    }
    pub(crate) fn activate(&mut self, child: Id<Entity>, fd: Fd, way: Way) {
        let state = mem::replace(&mut self.state, State::Closed);
        self.state = match state {
            State::Pending => {
                assert_eq!(self.way, way, "a pending pipe activates once");
                self.child = Some(child);
                State::Adopted { fd, demand: None }
            }
            State::Adopted { .. }
            | State::Reading(_)
            | State::Writing(_)
            | State::Flushing(_)
            | State::Settling { .. }
            | State::Releasing { .. }
            | State::Closed => unreachable!("a pending pipe activates once"),
        };
    }
    pub(crate) fn inherited(fd: Fd, way: Way, write_timeout: Option<Duration>) -> Pipe {
        let mut pipe = Self::pending(way);
        pipe.state = State::Adopted { fd, demand: None };
        pipe.write_timeout = write_timeout;
        pipe
    }
    pub(crate) fn append(fd: Fd, write_timeout: Duration) -> Pipe {
        let mut pipe = Self::inherited(fd, Way::In, Some(write_timeout));
        pipe.append = true;
        pipe
    }
    pub(crate) fn window_fd(&self) -> Option<Fd> {
        assert_eq!(self.way, Way::Out, "Window names a readable terminal stream");
        match &self.state {
            State::Adopted { fd, .. } | State::Reading(Reading { fd, .. }) => Some(*fd),
            State::Pending
            | State::Writing(_)
            | State::Flushing(_)
            | State::Settling { .. }
            | State::Releasing { .. }
            | State::Closed => None,
        }
    }
    pub(crate) fn read_cancelled(&self) -> bool {
        match &self.state {
            State::Reading(read) => read.paused && read.quiet(),
            State::Pending
            | State::Adopted { .. }
            | State::Writing(_)
            | State::Flushing(_)
            | State::Settling { .. }
            | State::Releasing { .. }
            | State::Closed => false,
        }
    }
    pub(crate) const fn is_closed(&self) -> bool {
        match self.state {
            State::Closed => true,
            State::Pending
            | State::Adopted { .. }
            | State::Reading(_)
            | State::Writing(_)
            | State::Flushing(_)
            | State::Settling { .. }
            | State::Releasing { .. } => false,
        }
    }
    pub(crate) const fn child(&self) -> Option<Id<Entity>> {
        self.child
    }
}

pub(crate) fn resume(
    pipe: &mut Pipe,
    id: Id<Entity>,
    env: &Env<Limits>,
    tables: &mut Tables,
    up: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) {
    let state = mem::replace(&mut pipe.state, State::Closed);
    let state = match state {
        State::Adopted { fd, demand } => {
            let (read, room) = demand.unzip();
            match pipe.way {
                Way::Out => State::Reading(Reading {
                    fd,
                    intake: Intake::with_capacity(env.limits.intake),
                    demand: read,
                    source: Source::Idle,
                    paused: false,
                    cancel: false,
                }),
                Way::In => State::Writing(Writing {
                    output: Output { fd, queue: Queue::with_capacity(env.limits.sends), bytes: 0, write: None },
                    granted: 0,
                    demand: room,
                    finishing: Finish::More,
                }),
                Way::Given(_) | Way::Null => unreachable!("a given or null descriptor has no io pipe entity"),
            }
        }
        State::Pending
        | State::Reading(_)
        | State::Writing(_)
        | State::Flushing(_)
        | State::Settling { .. }
        | State::Releasing { .. }
        | State::Closed => state,
    };
    pipe.state = follow(state, pipe, id, env, tables, up, subs);
}
/// Pauses a readable pipe without relinquishing its descriptor.
pub(crate) fn cancel_read(
    pipe: &mut Pipe,
    id: Id<Entity>,
    env: &Env<Limits>,
    tables: &mut Tables,
    subs: &mut Queue<Submit>,
) {
    if pipe.way != Way::Out {
        return;
    }
    let state = mem::replace(&mut pipe.state, State::Closed);
    pipe.state = match state {
        State::Adopted { fd, .. } => {
            tables.ready.mark(id);
            State::Reading(Reading {
                fd,
                intake: Intake::with_capacity(env.limits.intake),
                demand: None,
                source: Source::Idle,
                paused: true,
                cancel: false,
            })
        }
        State::Reading(mut read) => {
            if !read.paused {
                read.paused = true;
                match read.source {
                    Source::Reading(flight) => {
                        read.cancel = true;
                        tables.cancel(subs, id, flight);
                    }
                    Source::Idle | Source::Eof | Source::Ended => {}
                }
                tables.ready.mark(id);
            }
            State::Reading(read)
        }
        State::Pending
        | State::Writing(_)
        | State::Flushing(_)
        | State::Settling { .. }
        | State::Releasing { .. }
        | State::Closed => state,
    };
}

pub(crate) fn request(
    pipe: &mut Pipe,
    id: Id<Entity>,
    down: Down,
    env: &Env<Limits>,
    tables: &mut Tables,
    subs: &mut Queue<Submit>,
) {
    let state = mem::replace(&mut pipe.state, State::Closed);
    pipe.state = match state {
        State::Adopted { fd, demand } => match down {
            Down::Demand { read, room } => {
                State::Adopted { fd, demand: asked(pipe, demand.is_none(), read, room, env, tables, id) }
            }
            Down::Finish => {
                assert_eq!(pipe.way, Way::In, "only a write pipe finishes");
                retire(pipe, id, tables);
                release(fd, id, tables, subs)
            }
            Down::Send(_) => unreachable!("a send stays within granted room"),
        },
        State::Reading(mut reading) => match down {
            Down::Demand { read, room } => {
                let (demand, _) = asked(pipe, reading.demand.is_none(), read, room, env, tables, id).unzip();
                reading.demand = demand;
                if demand.is_some() {
                    assert!(!reading.paused || reading.quiet(), "a withdrawn read settles before reading again");
                    reading.paused = false;
                }
                State::Reading(reading)
            }
            Down::Send(_) => unreachable!("only an unfinished write pipe sends"),
            Down::Finish => unreachable!("only a write pipe finishes"),
        },
        State::Writing(mut write) => {
            match down {
                Down::Demand { read, room } => {
                    let (_, demand) = asked(pipe, write.demand.is_none(), read, room, env, tables, id).unzip();
                    write.demand = demand;
                }
                Down::Send(bytes) => {
                    assert!(write.finishing == Finish::More, "only an unfinished write pipe sends");
                    assert!(pipe.independent.idle(), "classic Send cannot spend an independent grant");
                    let size = u32::try_from(bytes.len()).expect("a send fits u32");
                    assert!(size <= write.granted, "a send stays within granted room");
                    write.granted = 0;
                    if !bytes.is_empty() {
                        write.output.bytes = write.output.bytes.checked_add(size).expect("bounded output");
                        assert!(write.output.bytes <= env.limits.output, "output fits its cap");
                        write.output.queue.push(bytes);
                        if write.output.write.is_none()
                            && let Some(bytes) = write.output.queue.pop()
                        {
                            arm_write(pipe.write_timeout, id, env, tables);
                            write.output.write = Some(submit_write(pipe, write.output.fd, bytes, 0, id, tables, subs));
                        }
                    }
                }
                Down::Finish => {
                    write.finishing = Finish::Finishing;
                    write.granted = 0;
                    retire(pipe, id, tables);
                }
            }
            settled(State::Writing(write), id, tables, subs)
        }
        State::Pending | State::Flushing(_) | State::Settling { .. } | State::Releasing { .. } | State::Closed => state,
    };
}

fn asked(
    pipe: &Pipe,
    vacant: bool,
    read: Read,
    room: u32,
    env: &Env<Limits>,
    tables: &mut Tables,
    id: Id<Entity>,
) -> Option<(Read, u32)> {
    if read == Read::Nothing && room == 0 {
        return None;
    }
    match pipe.way {
        Way::In => assert!(read == Read::Nothing, "a parent write pipe has no read side"),
        Way::Out => assert!(room == 0, "a parent read pipe has no write side"),
        Way::Given(_) | Way::Null => unreachable!("a given or null descriptor has no io pipe entity"),
    }
    assert!(room == 0 || pipe.independent.idle(), "classic room cannot overlap independent output");
    assert!(vacant, "one demand at a time");
    assert!(room <= env.limits.output, "demand fits the output cap");
    match read {
        Read::Nothing => {}
        Read::Fill(n) => assert!(n <= env.limits.intake, "fill within intake"),
        Read::Scan { until, max } => {
            assert!(max <= env.limits.intake && usize_of(max) >= until.as_bytes().len(), "scan within intake");
        }
        Read::Line { max } => assert!(max > 0 && max <= env.limits.intake, "line within intake"),
        Read::Arrived { max } => assert!(max > 0, "a read of what has arrived has a positive maximum"),
    }
    tables.ready.mark(id);
    Some((read, room))
}

pub(crate) fn landed(
    pipe: &mut Pipe,
    landed: Landed,
    env: &Env<Limits>,
    tables: &mut Tables,
    up: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) {
    let id = landed.entity;
    emit_terminal(pipe, id, up);
    let state = mem::replace(&mut pipe.state, State::Closed);
    let state = match landed.purpose {
        Purpose::PipeRead => read_landed(state, pipe, landed, up),
        Purpose::PipeWrite => write_landed(state, pipe, landed, env, tables, up, subs),
        Purpose::Close => match state {
            State::Releasing { close } => {
                assert_eq!(close, landed.flight, "close flight matches");
                up.push(Event::Closed { owner: id.token() });
                State::Closed
            }
            State::Pending
            | State::Adopted { .. }
            | State::Reading(_)
            | State::Writing(_)
            | State::Flushing(_)
            | State::Settling { .. }
            | State::Closed => unreachable!("close completes while releasing"),
        },
        Purpose::Cancel(target) => match state {
            State::Settling { fd, read, cancels } => {
                State::Settling { fd, read, cancels: cancels.checked_sub(1).expect("cancel in flight") }
            }
            State::Flushing(mut flush) => {
                if pipe.write_timeout.is_some()
                    && crate::layer::unsubmitted(landed.result)
                    && flush.output.write == Some(target)
                {
                    tables.cancel(subs, landed.entity, target);
                } else {
                    flush.cancels = flush.cancels.checked_sub(1).expect("cancel in flight");
                }
                State::Flushing(flush)
            }
            State::Reading(mut read) => {
                assert!(read.cancel, "read cancellation is in flight");
                if crate::layer::unsubmitted(landed.result) {
                    match read.source {
                        Source::Reading(flight) => tables.cancel(subs, id, flight),
                        Source::Idle | Source::Eof | Source::Ended => read.cancel = false,
                    }
                } else {
                    read.cancel = false;
                }
                State::Reading(read)
            }
            State::Pending | State::Adopted { .. } | State::Writing(_) | State::Releasing { .. } | State::Closed => {
                unreachable!("a cancel completes while settling, flushing or withdrawing a read")
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
        | Purpose::ReadSignal => unreachable!("a pipe only reads, writes, cancels and closes"),
    };
    pipe.state = follow(state, pipe, id, env, tables, up, subs);
}

fn read_landed(state: State, pipe: &mut Pipe, landed: Landed, up: &mut Queue<Event>) -> State {
    match state {
        State::Reading(mut read) => {
            match read.source {
                Source::Reading(read) => assert_eq!(read, landed.flight, "read flight matches"),
                Source::Idle | Source::Eof | Source::Ended => unreachable!("a read completes while reading"),
            }
            let Op::PipeRead { buf, .. } = landed.kind else { unreachable!("a pipe read returns its buffer") };
            read.source = match count(landed.result) {
                Ok(0) => Source::Eof,
                Ok(n) => {
                    read.intake.append(buf.get(..usize_of(n)).expect("count fits buffer")).expect("read within room");
                    Source::Idle
                }
                Err(kernel::Error::Cancelled) if read.paused => Source::Idle,
                Err(error) => {
                    failed(pipe, landed.entity, error, up);
                    Source::Ended
                }
            };
            State::Reading(read)
        }
        State::Settling { fd, read, cancels } => {
            assert_eq!(read, Some(landed.flight), "read flight matches");
            State::Settling { fd, read: None, cancels }
        }
        State::Pending
        | State::Adopted { .. }
        | State::Writing(_)
        | State::Flushing(_)
        | State::Releasing { .. }
        | State::Closed => unreachable!("a pipe read completes while reading or settling"),
    }
}

pub(crate) fn close(
    pipe: &mut Pipe,
    id: Id<Entity>,
    abort: bool,
    env: &Env<Limits>,
    tables: &mut Tables,
    subs: &mut Queue<Submit>,
) {
    retire(pipe, id, tables);
    let state = mem::replace(&mut pipe.state, State::Closed);
    let state = match state {
        State::Pending => unreachable!("only an activated pipe has a token to close"),
        State::Adopted { fd, .. } => release(fd, id, tables, subs),
        State::Reading(reading) => {
            let read = match reading.source {
                Source::Reading(read) => Some(read),
                Source::Idle | Source::Eof | Source::Ended => None,
            };
            let cancels = if reading.cancel {
                1
            } else if let Some(read) = read {
                tables.cancel(subs, id, read);
                1
            } else {
                0
            };
            State::Settling { fd: reading.fd, read, cancels }
        }
        State::Writing(write) => {
            let mut flush = flushing(write);
            if abort {
                flush = spill(flush, id, tables, subs);
            } else if pipe.write_timeout.is_some() {
                tables
                    .deadlines
                    .arm((id, Timer::Close), env.now.saturating_add(env.limits.close_timeout))
                    .expect("one close timer fits its entity slots");
            }
            State::Flushing(flush)
        }
        State::Flushing(flush) => {
            let flush = match flush.flush {
                Flush::Whole if abort => spill(flush, id, tables, subs),
                Flush::Whole | Flush::Spill => flush,
            };
            State::Flushing(flush)
        }
        State::Settling { .. } | State::Releasing { .. } | State::Closed => state,
    };
    pipe.state = settled(state, id, tables, subs);
}
fn flushing(write: Writing) -> Flushing {
    Flushing { output: write.output, cancels: 0, flush: Flush::Whole }
}

fn spill(mut flush: Flushing, id: Id<Entity>, tables: &mut Tables, subs: &mut Queue<Submit>) -> Flushing {
    flush.flush = Flush::Spill;
    flush.output.queue.clear();
    if let Some(write) = flush.output.write {
        tables.cancel(subs, id, write);
        flush.cancels = flush.cancels.checked_add(1).expect("one cancel per flight");
    }
    flush
}

fn follow(
    state: State,
    pipe: &mut Pipe,
    id: Id<Entity>,
    env: &Env<Limits>,
    tables: &mut Tables,
    up: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> State {
    emit_terminal(pipe, id, up);
    match state {
        State::Reading(mut read) => {
            if let Some(demand) = read.demand
                && let Some(bytes) = read.intake.meet(demand)
            {
                read.demand = None;
                up.push(Event::Stream { owner: id.token(), up: Up::Bytes(bytes) });
            }
            if read.paused && read.quiet() {
                read.demand = None;
            }
            read.source = match read.source {
                Source::Eof if read.intake.is_empty() || read.demand.is_some() => {
                    up.push(Event::Stream { owner: id.token(), up: Up::End });
                    Source::Ended
                }
                Source::Idle => {
                    let room = read.intake.room().min(env.limits.receive);
                    if room > 0 && !read.paused {
                        Source::Reading(tables.submit(
                            subs,
                            id,
                            Purpose::PipeRead,
                            Op::PipeRead { fd: read.fd, buf: bytes::zeroed(usize_of(room)) },
                        ))
                    } else {
                        Source::Idle
                    }
                }
                Source::Reading(_) | Source::Eof | Source::Ended => read.source,
            };
            State::Reading(read)
        }
        State::Writing(mut write) => {
            if let Some(room) = write.demand {
                if room > 0
                    && write.output.bytes.saturating_add(room) <= env.limits.output
                    && write.output.queue.room() > 0
                {
                    write.granted = room;
                    write.demand = None;
                    up.push(Event::Stream { owner: id.token(), up: Up::Room });
                } else if room == 0 {
                    write.demand = None;
                }
            }
            if let Some((_, bytes)) = pipe.independent.wanted()
                && write.finishing == Finish::More
                && bytes <= env.limits.output.saturating_sub(write.output.bytes)
                && write.output.queue.room() > 0
            {
                let terminal = pipe.independent.grant();
                up.push(Event::Output { owner: id.token(), up: terminal });
            }
            settled(State::Writing(write), id, tables, subs)
        }
        State::Flushing(mut flush) => {
            if flush.output.write.is_none()
                && let Some(bytes) = flush.output.queue.pop()
            {
                flush.output.write = Some(submit_write(pipe, flush.output.fd, bytes, 0, id, tables, subs));
            }
            settled(State::Flushing(flush), id, tables, subs)
        }
        State::Settling { fd, read, cancels } => settled(State::Settling { fd, read, cancels }, id, tables, subs),
        State::Pending | State::Adopted { .. } | State::Releasing { .. } | State::Closed => state,
    }
}

fn arm_write(timeout: Option<Duration>, id: Id<Entity>, env: &Env<Limits>, tables: &mut Tables) {
    if let Some(timeout) = timeout {
        tables
            .deadlines
            .arm((id, Timer::Close), env.now.saturating_add(timeout))
            .expect("one write timer fits its entity slots");
    }
}
fn submit_write(
    pipe: &Pipe,
    fd: Fd,
    bytes: Box<[u8]>,
    from: u32,
    id: Id<Entity>,
    tables: &mut Tables,
    subs: &mut Queue<Submit>,
) -> Id<Flight> {
    let operation = if pipe.append { Op::Append { fd, bytes, from } } else { Op::PipeWrite { fd, bytes, from } };
    tables.submit(subs, id, Purpose::PipeWrite, operation)
}

pub(crate) fn expired(
    pipe: &mut Pipe,
    id: Id<Entity>,
    env: &Env<Limits>,
    tables: &mut Tables,
    up: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) {
    assert!(pipe.write_timeout.is_some(), "only output streams with a timeout arm a pipe deadline");
    let state = mem::replace(&mut pipe.state, State::Closed);
    let state = match state {
        State::Writing(write) => {
            failed(pipe, id, kernel::Error::TimedOut, up);
            State::Flushing(spill(flushing(write), id, tables, subs))
        }
        State::Flushing(flush) => match flush.flush {
            Flush::Whole => State::Flushing(spill(flush, id, tables, subs)),
            Flush::Spill => State::Flushing(flush),
        },
        State::Pending
        | State::Adopted { .. }
        | State::Reading(_)
        | State::Settling { .. }
        | State::Releasing { .. }
        | State::Closed => state,
    };
    pipe.state = follow(state, pipe, id, env, tables, up, subs);
}

fn settled(state: State, id: Id<Entity>, tables: &mut Tables, subs: &mut Queue<Submit>) -> State {
    let fd = match &state {
        State::Writing(write) if write.finishing == Finish::Finishing && write.output.bytes == 0 => write.output.fd,
        State::Flushing(flush) if flush.output.write.is_none() && flush.cancels == 0 => flush.output.fd,
        State::Settling { fd, read: None, cancels: 0 } => *fd,
        State::Pending
        | State::Adopted { .. }
        | State::Reading(_)
        | State::Writing(_)
        | State::Flushing(_)
        | State::Settling { .. }
        | State::Releasing { .. }
        | State::Closed => return state,
    };
    release(fd, id, tables, subs)
}
fn release(fd: Fd, id: Id<Entity>, tables: &mut Tables, subs: &mut Queue<Submit>) -> State {
    State::Releasing { close: tables.submit(subs, id, Purpose::Close, Op::Close { fd }) }
}
fn retire(pipe: &mut Pipe, id: Id<Entity>, tables: &mut Tables) {
    let pending = pipe.independent.wanted().is_some();
    pipe.independent.retire(OutputOutcome::Cancelled);
    if pending {
        tables.ready.mark(id);
    }
}
fn failed(pipe: &mut Pipe, id: Id<Entity>, error: kernel::Error, up: &mut Queue<Event>) {
    let fault = if error == kernel::Error::BrokenPipe
        || error == kernel::Error::Reset
        || error == kernel::Error::NotConnected
    {
        Fault::Reset
    } else {
        Fault::Other
    };
    pipe.independent.retire(OutputOutcome::Failed(fault));
    emit_terminal(pipe, id, up);
    up.push(Event::Stream { owner: id.token(), up: Up::Failed(fault) });
}
fn emit_terminal(pipe: &mut Pipe, id: Id<Entity>, up: &mut Queue<Event>) {
    if let Some(terminal) = pipe.independent.take_terminal() {
        up.push(Event::Output { owner: id.token(), up: terminal });
    }
}
pub(crate) fn output_request(
    pipe: &mut Pipe,
    id: Id<Entity>,
    down: OutputDown,
    env: &Env<Limits>,
    tables: &mut Tables,
    subs: &mut Queue<Submit>,
) {
    let (room, granted) = match &pipe.state {
        State::Adopted { demand, .. } if pipe.way == Way::In => (demand.unzip().1, 0),
        State::Writing(write) if write.finishing == Finish::More => (write.demand, write.granted),
        State::Pending
        | State::Adopted { .. }
        | State::Reading(_)
        | State::Writing(_)
        | State::Flushing(_)
        | State::Settling { .. }
        | State::Releasing { .. }
        | State::Closed => return,
    };
    match down {
        OutputDown::Room { right, bytes } => {
            if bytes == 0 || bytes > env.limits.output || !pipe.independent.idle() || room.is_some() || granted > 0 {
                return;
            }
            pipe.independent.admit(right, bytes);
            tables.ready.mark(id);
        }
        OutputDown::Cancel { right } => {
            if pipe.independent.cancel(right) {
                tables.ready.mark(id);
            }
        }
        OutputDown::Send { right, bytes } => {
            let Some(granted) = pipe.independent.granted(right) else { return };
            let length = u32::try_from(bytes.len()).expect("a matching independent pipe Send fits u32");
            assert!(length <= granted, "a matching independent pipe Send fits its grant");
            pipe.independent.release(right);
            match &mut pipe.state {
                State::Writing(write) => write.granted = granted,
                State::Pending
                | State::Adopted { .. }
                | State::Reading(_)
                | State::Flushing(_)
                | State::Settling { .. }
                | State::Releasing { .. }
                | State::Closed => unreachable!("only a write pipe grants output"),
            }
            request(pipe, id, Down::Send(bytes), env, tables, subs);
        }
        OutputDown::Release { right } => pipe.independent.release(right),
    }
}
fn count(result: Result<Done, kernel::Error>) -> Result<u32, kernel::Error> {
    let Done::Count(n) = result? else { unreachable!("a pipe read or write answers with a count") };
    Ok(n)
}
fn write_landed(
    mut state: State,
    pipe: &mut Pipe,
    landed: Landed,
    env: &Env<Limits>,
    tables: &mut Tables,
    up: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> State {
    let (output, whole, timeout) = match &mut state {
        State::Writing(write) => (&mut write.output, Flush::Whole, pipe.write_timeout),
        State::Flushing(flush) => (&mut flush.output, flush.flush, None),
        State::Pending
        | State::Adopted { .. }
        | State::Reading(_)
        | State::Settling { .. }
        | State::Releasing { .. }
        | State::Closed => unreachable!("a write completes while writing or flushing"),
    };
    assert_eq!(output.write.take(), Some(landed.flight), "write flight matches");
    let id = landed.entity;
    let (Op::PipeWrite { bytes, from, .. } | Op::Append { bytes, from, .. }) = landed.kind else {
        unreachable!("a stream write returns its bytes")
    };
    if timeout.is_some() {
        tables.deadlines.cancel((id, Timer::Close));
    }
    let total = u32::try_from(bytes.len()).expect("queued write fits u32");
    match count(landed.result) {
        Ok(n) if whole == Flush::Whole && from.saturating_add(n) < total => {
            let from = from.checked_add(n).expect("a short write stays within its buffer");
            arm_write(timeout, id, env, tables);
            output.write = Some(submit_write(pipe, output.fd, bytes, from, id, tables, subs));
        }
        Ok(n) if n > 0 => {
            output.bytes = output.bytes.checked_sub(total).expect("queued bytes include write");
            if let Some(bytes) = output.queue.pop() {
                arm_write(timeout, id, env, tables);
                output.write = Some(submit_write(pipe, output.fd, bytes, 0, id, tables, subs));
            }
        }
        Err(error) => {
            if timeout.is_some() || whole == Flush::Whole {
                output.bytes = output.bytes.checked_sub(total).expect("queued bytes include write");
            }
            let flush = match state {
                State::Writing(write) => {
                    failed(pipe, id, error, up);
                    let finish = write.finishing;
                    let mut flush = flushing(write);
                    flush.flush = match finish {
                        Finish::More => Flush::Spill,
                        Finish::Finishing => Flush::Whole,
                    };
                    if pipe.write_timeout.is_some() { spill(flush, id, tables, subs) } else { flush }
                }
                State::Flushing(flush) => {
                    if pipe.write_timeout.is_some() && flush.flush == Flush::Whole {
                        spill(flush, id, tables, subs)
                    } else {
                        flush
                    }
                }
                State::Pending
                | State::Adopted { .. }
                | State::Reading(_)
                | State::Settling { .. }
                | State::Releasing { .. }
                | State::Closed => unreachable!("a write fails while writing or flushing"),
            };
            return State::Flushing(flush);
        }
        Ok(_) => unreachable!("a pipe write counts positive bytes"),
    }
    state
}
