//! Child replies resume the continuation named by their handle
//! (`jig/docs/design/domain/engine.md`, sections 4.3 and 4.4).

use crate::Core;
use skein_lib::{Id, Token};

#[derive(Debug)]
pub(crate) enum Flight {
    Moving { request: Token, task: u64 },
    Retired,
}

impl Core {
    pub(crate) fn fly(&mut self, flight: Flight) -> Token {
        self.flights.insert(flight).expect("continuations grow with admitted work").token()
    }

    pub(crate) fn land(&mut self, token: Token) -> Option<Flight> {
        let id = Id::from_token(token);
        let flight = core::mem::replace(self.flights.get_mut(id)?, Flight::Retired);
        self.flights.retire(id);
        self.flights.reclaim();
        Some(flight)
    }
}
