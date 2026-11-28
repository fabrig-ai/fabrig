//! The core's store flights (`jig/docs/design/domain/engine/vocabulary.md`, 2.4).

use crate::{Core, Held, Limits, Request, engine::store, reads, routing::Work};
use alloc::boxed::Box;
use jig_domain_notes as notes;
use skein_lib::{Env, Map, Queue, Token};

pub(crate) mod point;
use point::Point;

#[derive(Hash, Debug)]
enum WorkFlight {
    Point(Point),
    Window(u64),
    Notes(Token),
}

#[derive(Hash, Debug)]
pub(crate) struct Loads {
    read_flights: Map<Token, reads::Pending>,
    work_flights: Map<Token, WorkFlight>,
    // One sequence for both lanes, so an old token cannot reach another flight.
    next_token: u64,
    retired_reads: u32,
    retired_work: bool,
    waiting: Queue<Point>,
    // Project deletion retries the page the child asked for after a local refusal.
    note_retry: Option<(Token, notes::Range)>,
}

impl Loads {
    pub(crate) fn new(read_slots: u32, load_slots: u32, result_flights: u32) -> Self {
        // The two lanes price the table: read room (including retired reads)
        // plus load_slots. Work terminals free storage before any continuation.
        Self {
            read_flights: Map::with_capacity(read_slots),
            work_flights: Map::with_capacity(load_slots),
            next_token: 0,
            retired_reads: 0,
            retired_work: false,
            waiting: Queue::with_capacity(result_flights),
            note_retry: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn capacity(&self) -> u64 {
        u64::from(self.read_flights.capacity())
            .checked_add(u64::from(self.work_flights.capacity()))
            .expect("both lane capacities are u32")
    }

    pub(crate) fn reads(&self) -> u32 {
        self.read_flights.len().checked_add(self.retired_reads).expect("read room includes retired reads")
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.reads() == 0
            && self.work_flights.is_empty()
            && !self.retired_work
            && self.waiting.is_empty()
            && self.note_retry.is_none()
    }

    pub(crate) fn read_slots_full(&self) -> bool {
        self.reads() == self.read_flights.capacity()
    }

    pub(crate) fn name(&mut self) -> Token {
        let load = Token::new(self.next_token);
        self.next_token = self.next_token.checked_add(1).expect("store load tokens are never reused");
        load
    }

    #[expect(clippy::result_large_err, reason = "refused reads return the original pending value without boxing it")]
    pub(crate) fn insert_read(&mut self, pending: reads::Pending) -> Result<Token, reads::Pending> {
        if self.read_slots_full() {
            return Err(pending);
        }
        let load = self.name();
        let previous = self.read_flights.insert(load, pending).expect("read load room reserved");
        assert!(previous.is_none(), "fresh read load token");
        Ok(load)
    }

    pub(crate) fn read(&self, load: Token) -> Option<&reads::Pending> {
        self.read_flights.get(&load)
    }

    pub(crate) fn read_mut(&mut self, load: Token) -> Option<&mut reads::Pending> {
        self.read_flights.get_mut(&load)
    }

    pub(crate) fn retire(&mut self, load: Token) {
        drop(self.read_flights.remove(&load).expect("one terminal per read load"));
        self.retired_reads = self.retired_reads.checked_add(1).expect("bounded read flights");
    }

    pub(crate) fn reclaim(&mut self) {
        self.retired_reads = 0;
        self.retired_work = false;
    }

    /// Reads and retired loads do not occupy a slot in the engine's own load lane.
    pub(crate) fn room(&self, limits: &Limits) -> bool {
        self.waiting.is_empty() && self.has_slot(limits)
    }

    pub(crate) fn window(&mut self, task: u64, from: (u64, u32)) -> Request {
        let load = self.insert_work(WorkFlight::Window(task));
        Request::Held(Box::new(Held::Store(store::Out::Load { load, range: store::Range::Window { task, from } })))
    }

    pub(crate) fn notes(&mut self, owner: Token, range: notes::Range) -> Request {
        let load = self.insert_work(WorkFlight::Notes(owner));
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
        for (load, flight) in &self.work_flights {
            if let WorkFlight::Window(owner) = flight
                && *owner == task
            {
                name = Some(*load);
                break;
            }
        }
        if let Some(load) = name {
            let _window = self.finish_work(load);
        }
    }

    fn insert_work(&mut self, flight: WorkFlight) -> Token {
        let load = self.name();
        let previous = self.work_flights.insert(load, flight).expect("work load room reserved");
        assert!(previous.is_none(), "fresh work load token");
        load
    }

    fn finish_work(&mut self, load: Token) -> WorkFlight {
        let flight = self.work_flights.remove(&load).expect("one terminal per work load");
        // Quiescence still waits for the iteration's reclaim point.
        self.retired_work = true;
        flight
    }

    fn has_slot(&self, limits: &Limits) -> bool {
        self.work_flights.len() < limits.load_slots.min(self.work_flights.capacity())
    }
}

pub(crate) fn read(load: Token, range: reads::Range) -> Request {
    Request::Held(Box::new(Held::Store(store::Out::Load { load, range: store::Range::Read(Box::new(range)) })))
}

pub(crate) fn awaited(core: &Core, load: Token) -> bool {
    if let Some(pending) = core.loads.read(load) {
        pending.awaiting_store()
    } else {
        core.loads.work_flights.contains_key(&load)
    }
}

pub(crate) fn loaded(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    load: Token,
    result: store::Answer,
) -> Queue<Request> {
    if core.loads.read(load).is_some() {
        let result = match result {
            store::Answer::Rows(store::Rows::Read { .. })
            | store::Answer::Failed(
                store::Failure::Busy
                | store::Failure::Gone
                | store::Failure::TooLinked { .. }
                | store::Failure::TurnProblem { .. }
                | store::Failure::Failed,
            ) => result,
            store::Answer::Rows(
                store::Rows::Records { .. } | store::Rows::Turns { .. } | store::Rows::Start { .. },
            )
            | store::Answer::Cut { .. }
            | store::Answer::Failed(store::Failure::Unread) => store::Answer::Failed(store::Failure::Failed),
        };
        return reads::loaded(core, env, work, load, result);
    }
    match core.loads.work_flights.get(&load) {
        Some(WorkFlight::Point(_)) => {
            let WorkFlight::Point(point) = core.loads.finish_work(load) else { unreachable!("point load flight") };
            point::loaded(core, env, work, point, result)
        }
        Some(WorkFlight::Window(task)) => {
            let task = *task;
            let more = match &result {
                store::Answer::Rows(store::Rows::Turns { more, .. }) | store::Answer::Cut { more, .. } => *more,
                store::Answer::Rows(
                    store::Rows::Read { .. } | store::Rows::Records { .. } | store::Rows::Start { .. },
                )
                | store::Answer::Failed(_) => false,
            };
            if !more || core.stopping || core.settling {
                let _window = core.loads.finish_work(load);
            }
            if core.stopping || core.settling {
                return Queue::uncapped();
            }
            crate::routing::run::preparation::transcript_loaded(core, env, work, task, result)
        }
        Some(WorkFlight::Notes(owner)) => {
            let owner = *owner;
            let _notes = core.loads.finish_work(load);
            crate::routing::effect::note::loaded(core, env, work, owner, result)
        }
        None => Queue::uncapped(),
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
    let load = core.loads.insert_work(WorkFlight::Point(point));
    out.push(Request::Held(Box::new(Held::Store(store::Out::Load { load, range }))));
}
