//! The core's store flights (`jig/docs/design/domain/engine/contracts.md`, 2).

use crate::{Core, Held, Limits, Request, engine::store, reads, routing::Work};
use alloc::boxed::Box;
use skein_lib::{Env, Id, Queue, Slab, Token};

#[derive(Hash, Debug)]
pub(crate) enum Flight {
    // A retired read remains charged to its lane until reclaim, but cannot take a terminal.
    Read(Option<reads::Pending>),
}

impl Flight {
    fn uses_load_slot(&self) -> bool {
        match self {
            Self::Read(_) => false,
        }
    }
}

#[derive(Hash, Debug)]
pub(crate) struct Loads {
    flights: Slab<Flight>,
}

impl Loads {
    pub(crate) fn new(read_slots: u32) -> Self {
        Self { flights: Slab::with_capacity(read_slots) }
    }

    pub(crate) fn reads(&self) -> u32 {
        self.flights.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.flights.is_empty()
    }

    pub(crate) fn read_slots_full(&self) -> bool {
        self.flights.is_full()
    }

    #[expect(clippy::result_large_err, reason = "refused reads return the original pending value without boxing it")]
    pub(crate) fn insert_read(&mut self, pending: reads::Pending) -> Result<Id<Flight>, reads::Pending> {
        match self.flights.insert(Flight::Read(Some(pending))) {
            Ok(id) => Ok(id),
            Err(Flight::Read(Some(pending))) => Err(pending),
            Err(Flight::Read(None)) => unreachable!("only live loads are inserted"),
        }
    }

    pub(crate) fn read(&self, id: Id<Flight>) -> Option<&reads::Pending> {
        let Flight::Read(pending) = self.flights.get(id)?;
        pending.as_ref()
    }

    pub(crate) fn read_mut(&mut self, id: Id<Flight>) -> Option<&mut reads::Pending> {
        let Flight::Read(pending) = self.flights.get_mut(id)?;
        pending.as_mut()
    }

    pub(crate) fn retire(&mut self, id: Id<Flight>) {
        let Flight::Read(pending) = self.flights.get_mut(id).expect("known flight");
        assert!(pending.take().is_some(), "one terminal per load");
        self.flights.retire(id);
    }

    pub(crate) fn reclaim(&mut self) {
        self.flights.reclaim();
    }

    /// Reads and retired loads do not occupy a slot in the engine's own load lane.
    #[expect(dead_code, reason = "the next load family uses the non-read lane")]
    pub(crate) fn room(&self, limits: &Limits) -> bool {
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
        Some(Flight::Read(None)) | None => false,
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
        Some(Flight::Read(None)) | None => Queue::uncapped(),
    }
}
