//! The deployment's build places (temper/docs/design/testing/worker.md, 2.2–2.3).
use std::collections::{BTreeSet, VecDeque};

use skein_lib::{Time, Token};
use skein_world::inspect::{Entry, Inspection, Refused, Release, Table, Wait, Wake};

const KIND: &str = "a check waiting for a build place";

pub struct Builder {
    places: u32,
    holders: Vec<Token>,
    waiting: VecDeque<(Token, Time)>,
    ended: BTreeSet<Token>,
}

impl Builder {
    #[must_use]
    pub fn new(places: u32) -> Self {
        Self { places, holders: Vec::new(), waiting: VecDeque::new(), ended: BTreeSet::new() }
    }

    /// True if the check immediately holds a place; otherwise it waits FIFO.
    pub fn ask(&mut self, run: Token, at: Time) -> bool {
        assert!(!self.holders.contains(&run) && !self.waiting.iter().any(|(other, _)| *other == run));
        if self.holders.len() < usize::try_from(self.places).expect("places fit usize") {
            self.holders.push(run);
            true
        } else {
            self.waiting.push_back((run, at));
            false
        }
    }

    /// A check ends; its place passes directly to the oldest waiting check.
    pub fn finish(&mut self, run: Token) -> Option<Token> {
        let index = self.holders.iter().position(|holder| *holder == run).expect("check holds a place");
        self.holders.remove(index);
        let (next, _) = self.waiting.pop_front()?;
        self.holders.push(next);
        Some(next)
    }

    pub fn ended(&mut self, run: Token) {
        self.ended.insert(run);
    }

    #[must_use]
    pub fn inspection(&self) -> Inspection {
        let mut found = Inspection::new("builder", &[KIND]);
        for (run, _) in &self.waiting {
            found.wait(Wait {
                waiter: name(*run),
                kind: KIND.into(),
                wakes: self.holders.iter().map(|holder| Wake::Waiter(name(*holder))).collect(),
                bound: None,
            });
        }
        found.table(Table {
            name: "build places",
            capacity: u64::from(self.places),
            entries: self
                .holders
                .iter()
                .map(|run| Entry {
                    key: name(*run),
                    rule: "the check's end",
                    release: if self.ended.contains(run) { Release::Due } else { Release::Pending },
                })
                .collect(),
            refused: self
                .waiting
                .iter()
                .map(|(run, since)| Refused { key: name(*run), since: *since, bound: None })
                .collect(),
        });
        found
    }
}

fn name(run: Token) -> String {
    format!("run {}", run.raw())
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_world::inspect::{Moment, check_tables};

    #[test]
    fn a_check_waiting_beside_a_free_build_place_is_a_finding() {
        let mut builder = Builder::new(1);
        assert!(builder.ask(Token::new(1), Time::ZERO));
        assert!(!builder.ask(Token::new(2), Time::ZERO));
        // A broken grant path left the queue asleep despite newly free capacity.
        builder.places = 2;
        let inspection = builder.inspection();
        let finding = check_tables(&[&inspection], Moment { seed: 2, at: Time::ZERO, iteration: 1 })
            .expect_err("free capacity must admit the waiting check");
        assert!(finding.contains("builder build places"), "{finding}");
    }
}
