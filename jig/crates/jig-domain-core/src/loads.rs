//! The core's store flights (`jig/docs/design/domain/engine/vocabulary.md`, 2.4).

use crate::{Core, Held, Limits, Request, engine::store, reads, routing::Work};
use alloc::boxed::Box;
use jig_domain_notes as notes;
use skein_lib::{Env, Id, Queue, Slab, Token};

pub(crate) mod point;
use point::Point;

#[derive(Hash, Debug)]
#[expect(clippy::large_enum_variant, reason = "retained reads keep their pending value inline in the load table")]
pub(crate) enum Flight {
    // A retired read remains charged to its lane until reclaim, but cannot take a terminal.
    Read(Option<reads::Pending>),
    Point(Option<Point>),
    Window(Option<u64>),
    Notes(Option<Token>),
}

impl Flight {
    fn uses_load_slot(&self) -> bool {
        match self {
            Self::Read(_) | Self::Point(None) | Self::Window(None) | Self::Notes(None) => false,
            Self::Point(Some(_)) | Self::Window(Some(_)) | Self::Notes(Some(_)) => true,
        }
    }
}

#[derive(Hash, Debug)]
pub(crate) struct Loads {
    flights: Slab<Flight>,
    read_slots: u32,
    reads: u32,
    retired_reads: u32,
    waiting: Queue<Point>,
    // Project deletion retries the page the child asked for after a local refusal.
    note_retry: Option<(Token, notes::Range)>,
}

impl Loads {
    pub(crate) fn new(read_slots: u32, result_flights: u32) -> Self {
        Self {
            flights: Slab::with_capacity(u32::MAX),
            read_slots,
            reads: 0,
            retired_reads: 0,
            waiting: Queue::with_capacity(result_flights),
            note_retry: None,
        }
    }

    pub(crate) fn reads(&self) -> u32 {
        self.reads
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.flights.is_empty() && self.waiting.is_empty() && self.note_retry.is_none()
    }

    pub(crate) fn delegate_waiting(&self, key: crate::CallKey) -> bool {
        for (_, flight) in &self.flights {
            if let Flight::Point(Some(Point::DelegateInput(read))) = flight
                && read.key == key
            {
                return true;
            }
        }
        false
    }

    pub(crate) fn read_slots_full(&self) -> bool {
        self.reads() == self.read_slots
    }

    #[expect(clippy::result_large_err, reason = "refused reads return the original pending value without boxing it")]
    pub(crate) fn insert_read(&mut self, pending: reads::Pending) -> Result<Id<Flight>, reads::Pending> {
        if self.read_slots_full() {
            Err(pending)
        } else {
            let id = self.flights.insert(Flight::Read(Some(pending))).expect("read room reserved");
            self.reads = self.reads.checked_add(1).expect("bounded read flights");
            Ok(id)
        }
    }

    pub(crate) fn read(&self, id: Id<Flight>) -> Option<&reads::Pending> {
        let Flight::Read(pending) = self.flights.get(id)? else { return None };
        pending.as_ref()
    }

    pub(crate) fn read_mut(&mut self, id: Id<Flight>) -> Option<&mut reads::Pending> {
        let Flight::Read(pending) = self.flights.get_mut(id)? else { return None };
        pending.as_mut()
    }

    pub(crate) fn retire(&mut self, id: Id<Flight>) {
        let Flight::Read(pending) = self.flights.get_mut(id).expect("known flight") else {
            unreachable!("retiring a read")
        };
        assert!(pending.take().is_some(), "one terminal per load");
        self.retired_reads = self.retired_reads.checked_add(1).expect("bounded read flights");
        self.flights.retire(id);
    }

    pub(crate) fn reclaim(&mut self) {
        self.flights.reclaim();
        self.reads = self.reads.checked_sub(self.retired_reads).expect("retired reads were charged");
        self.retired_reads = 0;
    }

    /// Reads and retired loads do not occupy a slot in the engine's own load lane.
    pub(crate) fn room(&self, limits: &Limits) -> bool {
        self.waiting.is_empty() && self.has_slot(limits)
    }

    pub(crate) fn window(&mut self, task: u64, from: (u64, u32)) -> Request {
        let load = self.flights.insert(Flight::Window(Some(task))).expect("window load room reserved").token();
        Request::Held(Box::new(Held::Store(store::Out::Load { load, range: store::Range::Window { task, from } })))
    }

    pub(crate) fn notes(&mut self, owner: Token, range: notes::Range) -> Request {
        let load = self.flights.insert(Flight::Notes(Some(owner))).expect("notes page room reserved").token();
        Request::Held(Box::new(Held::Store(store::Out::Load { load, range: store::Range::Notes(range) })))
    }

    pub(crate) fn retry_notes(&mut self, owner: Token, range: notes::Range) {
        assert!(self.note_retry.replace((owner, range)).is_none(), "one notes operation");
    }

    pub(crate) fn cancel_notes_retry(&mut self) {
        self.note_retry = None;
    }

    pub(crate) fn cancel_window(&mut self, task: u64) {
        let mut name = None;
        for (id, flight) in &self.flights {
            if let Flight::Window(Some(owner)) = flight
                && *owner == task
            {
                name = Some(id);
                break;
            }
        }
        if let Some(id) = name {
            self.finish_window(id);
        }
    }

    fn finish_window(&mut self, id: Id<Flight>) {
        let Flight::Window(task) = self.flights.get_mut(id).expect("known window flight") else {
            unreachable!("only retiring a window")
        };
        assert!(task.take().is_some(), "one terminal per window");
        self.flights.retire(id);
    }

    fn has_slot(&self, limits: &Limits) -> bool {
        let mut free = limits.load_slots;
        for (_, flight) in &self.flights {
            if flight.uses_load_slot() {
                free = free.saturating_sub(1);
            }
        }
        free > 0
    }
}

pub(crate) fn read(load: Token, range: reads::Range) -> Request {
    Request::Held(Box::new(Held::Store(store::Out::Load { load, range: store::Range::Read(Box::new(range)) })))
}

pub(crate) fn awaited(core: &Core, load: Token) -> bool {
    match core.loads.flights.get(Id::from_token(load)) {
        Some(Flight::Read(Some(pending))) => pending.awaiting_store(),
        Some(Flight::Point(Some(_)) | Flight::Window(Some(_)) | Flight::Notes(Some(_))) => true,
        Some(Flight::Read(None) | Flight::Point(None) | Flight::Window(None) | Flight::Notes(None)) | None => false,
    }
}

pub(crate) fn loaded(core: &mut Core, env: &Env<Limits>, work: &mut Queue<Work>, input: store::In) -> Queue<Request> {
    let store::In::Loaded { load, result } = input;
    let name = Id::from_token(load);
    match core.loads.flights.get(name) {
        Some(Flight::Read(Some(_))) => {
            let result = match result {
                store::Answer::Rows(store::Rows::Read { .. })
                | store::Answer::Failed(
                    store::Failure::Busy
                    | store::Failure::Gone
                    | store::Failure::TooLinked { .. }
                    | store::Failure::TurnProblem { .. }
                    | store::Failure::Failed,
                ) => result,
                store::Answer::Rows(store::Rows::Records { .. } | store::Rows::Turns { .. })
                | store::Answer::Cut { .. }
                | store::Answer::Failed(store::Failure::Unread) => store::Answer::Failed(store::Failure::Failed),
            };
            reads::loaded(core, env, work, name, result)
        }
        Some(Flight::Point(Some(_))) => {
            let Flight::Point(point) = core.loads.flights.get_mut(name).expect("known point load") else {
                unreachable!("point load flight")
            };
            let point = point.take().expect("one terminal per point load");
            core.loads.flights.retire(name);
            point::loaded(core, env, work, point, result)
        }
        Some(Flight::Window(Some(task))) => {
            let task = *task;
            let more = match &result {
                store::Answer::Rows(store::Rows::Turns { more, .. }) | store::Answer::Cut { more, .. } => *more,
                store::Answer::Rows(store::Rows::Read { .. } | store::Rows::Records { .. })
                | store::Answer::Failed(_) => false,
            };
            if !more || core.stopping || core.settling {
                core.loads.finish_window(name);
            }
            if core.stopping || core.settling {
                return Queue::uncapped();
            }
            crate::routing::run::preparation::transcript_loaded(core, env, work, task, result)
        }
        Some(Flight::Notes(Some(owner))) => {
            let owner = *owner;
            let Flight::Notes(pending) = core.loads.flights.get_mut(name).expect("known notes flight") else {
                unreachable!("notes flight")
            };
            *pending = None;
            core.loads.flights.retire(name);
            crate::routing::effect::note::loaded(core, env, work, owner, result)
        }
        Some(Flight::Read(None) | Flight::Point(None) | Flight::Window(None) | Flight::Notes(None)) | None => {
            Queue::uncapped()
        }
    }
}

/// A result read already owns one of the bounded result flights.
pub(crate) fn wait_point(core: &mut Core, request: Token, task: u64) {
    core.loads.waiting.push(Point::Result { request, task });
}

/// Drop a cancelled result's queued get. An issued get owns its slot until
/// the store terminal; its result route is already gone.
pub(crate) fn cancel_waiting_result(core: &mut Core, request: Token) {
    for _ in 0..core.loads.waiting.len() {
        let point = core.loads.waiting.pop().expect("waiting point count");
        if !point.is_result(request) {
            core.loads.waiting.push(point);
        }
    }
}

pub(crate) fn discard_waiting_points(core: &mut Core) {
    for _ in 0..core.loads.waiting.len() {
        let Point::Result { request, .. } = core.loads.waiting.pop().expect("waiting point count") else {
            unreachable!("only ended task loads wait")
        };
        crate::routing::run::results::discard(core, request);
    }
}

pub(crate) fn resume_waiting(core: &mut Core, limits: &Limits, out: &mut Queue<Request>) {
    if core.stopping() {
        discard_waiting_points(core);
        return;
    }
    for _ in 0..core.loads.waiting.len() {
        if !core.loads.has_slot(limits) {
            break;
        }
        let point = core.loads.waiting.pop().expect("waiting point count");
        send_point(core, out, point);
    }
    if core.loads.room(limits)
        && let Some((owner, range)) = core.loads.note_retry.take()
    {
        out.push(core.loads.notes(owner, range));
    }
}

/// Immediate loads cannot overtake a queued ended task.
pub(crate) fn try_point(core: &mut Core, limits: &Limits, out: &mut Queue<Request>, point: Point) -> bool {
    if !core.loads.room(limits) {
        return false;
    }
    send_point(core, out, point);
    true
}

fn send_point(core: &mut Core, out: &mut Queue<Request>, point: Point) {
    let range = point.range();
    let load = core.loads.flights.insert(Flight::Point(Some(point))).expect("point load room reserved").token();
    out.push(Request::Held(Box::new(Held::Store(store::Out::Load { load, range }))));
}
