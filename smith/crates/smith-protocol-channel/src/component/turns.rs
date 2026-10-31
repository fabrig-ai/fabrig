//! Durable turn credit retained through the final answer (smith/docs/design/protocol/channel.md, section 7).
use crate::Error;
use skein_lib::{List, Map};

#[derive(Debug)]
pub(super) struct Turns {
    window: smith_domain::Window,
    held: Map<u32, u64>,
    last: u32,
    bytes: u64,
}

impl Turns {
    pub(super) fn new(window: smith_domain::Window, capacity: u32) -> Turns {
        Turns { window, held: Map::with_capacity(capacity), last: 0, bytes: 0 }
    }

    pub(super) fn next(&self) -> Option<u32> {
        self.last.checked_add(1)
    }

    pub(super) fn check(&self, number: u32, bytes: u64) -> Result<(), Error> {
        let held = self.bytes.checked_add(bytes).ok_or(Error::Window)?;
        if self.next() != Some(number) || self.held.len() >= self.window.turns || held > self.window.bytes {
            return Err(Error::Window);
        }
        Ok(())
    }

    pub(super) fn keep(&mut self, number: u32, bytes: u64) -> Result<(), Error> {
        self.check(number, bytes)?;
        let held = self.bytes.checked_add(bytes).ok_or(Error::Window)?;
        self.held.insert(number, bytes).or(Err(Error::Window))?;
        self.last = number;
        self.bytes = held;
        Ok(())
    }

    pub(super) fn acknowledge(&mut self, number: u32) -> Result<(), Error> {
        if number == 0 || number > self.last {
            return Err(Error::Window);
        }
        let mut released = 0_u64;
        let mut old = List::with_capacity(self.held.len());
        for (turn, bytes) in &self.held {
            if *turn <= number {
                released = released.checked_add(*bytes).ok_or(Error::Window)?;
                old.push(*turn).or(Err(Error::Window))?;
            }
        }
        for turn in &old {
            self.held.remove(turn);
        }
        self.bytes = self.bytes.checked_sub(released).ok_or(Error::Window)?;
        Ok(())
    }
}
