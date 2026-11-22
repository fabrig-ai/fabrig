//! Answer-table entries in their owner's order, without copying their payloads.
use super::{Domain, requests};
use crate::{ActKey, RequestKey};
use skein_lib::{Time, length::usize_of};

impl Domain {
    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_answer_count(&self) -> u32 {
        self.answers.len()
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_act_answer_count(&self) -> u32 {
        self.acted.len()
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_request_count(&self) -> u32 {
        self.flights.len()
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_act_count(&self) -> u32 {
        self.act_flights.len()
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_answer_capacity(&self) -> u32 {
        self.answers.capacity()
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_answer(&self, index: u32) -> Option<(&RequestKey, Time)> {
        let (key, answer) = self.answers.iter().nth(usize_of(index))?;
        Some((key, answer.due))
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_act_answer(&self, index: u32) -> Option<(&ActKey, Time)> {
        let (key, answer) = self.acted.iter().nth(usize_of(index))?;
        Some((key, answer.due()))
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_answer_reservation(&self, index: u32) -> Option<(&RequestKey, u32)> {
        let (key, id) = self.flights.iter().nth(usize_of(index))?;
        let count = match self.pending.get(*id)? {
            requests::Pending::Party(party) => requests::answer_places(&party.ask),
            requests::Pending::MoveOut { .. } => 1,
            requests::Pending::Act(_) => 0,
        };
        Some((key, count))
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_act_reservation(&self, index: u32) -> Option<&ActKey> {
        let (key, _) = self.act_flights.iter().nth(usize_of(index))?;
        Some(key)
    }
}
