//! One whole borrowed input driven one request at a time (data/json.md, 5.3).

use skein_lib::stream::{Down, Slice};
use skein_lib::{Env, Queue, Time, Wall};

use super::{Error, Event, Limits, Request, Tokenizer, down, tokenizer_bound, up};

/// A tokenizer and its side below over one borrowed input (data/json.md, 5.3).
#[derive(Debug)]
pub struct Feed<'a> {
    machine: Tokenizer,
    env: Env<Limits>,
    input: Slice<'a>,
    above: Queue<Event>,
    below: Queue<Down>,
    /// Steps left across all answers, bounded by the input length.
    ticks: usize,
}

impl<'a> Feed<'a> {
    /// As for [`Tokenizer::new`], `limits.chunk` must be positive and
    /// `limits.length` must be less than `u32::MAX`.
    #[must_use]
    pub fn new(input: &'a [u8], limits: &Limits) -> Feed<'a> {
        Feed {
            machine: Tokenizer::new(limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits },
            input: Slice::new(input),
            above: Queue::with_capacity(1),
            below: Queue::with_capacity(1),
            ticks: input.len().saturating_mul(4).saturating_add(16),
        }
    }

    /// Bytes delivered, including the delivery that passed the document's length bound.
    #[must_use]
    pub fn delivered(&self) -> u64 {
        self.machine.delivered()
    }

    /// States `request`, then meets each demand until the tokenizer answers.
    /// Returns `Failed(Unexpected)` past the step bound or on a demand the
    /// input cannot meet. Requests keep the tokenizer's lifecycle contract.
    pub fn answer(&mut self, request: Request) -> Event {
        down(&mut self.machine, &self.env, request, &mut self.above, &mut self.below);
        for _tick in 0..self.ticks {
            self.ticks = self.ticks.saturating_sub(1);
            if let Some(event) = self.above.pop() {
                return event;
            }
            match self.below.pop() {
                Some(Down::Demand { read, room: 0 }) => {
                    let Some(delivery) = self.input.answer(read) else {
                        return Event::Failed(Error::Unexpected);
                    };
                    up(&mut self.machine, &self.env, delivery, &mut self.above, &mut self.below);
                }
                Some(Down::Demand { .. } | Down::Send(_) | Down::Finish) | None => {
                    return Event::Failed(Error::Unexpected);
                }
            }
        }
        Event::Failed(Error::Unexpected)
    }
}

/// The tokenizer and the feed's two one-entry queues (programming-model.md, 6.3).
#[must_use]
pub fn feed_bound(limits: &Limits) -> Option<u64> {
    tokenizer_bound(limits)?.checked_add(Queue::<Event>::price(1)?)?.checked_add(Queue::<Down>::price(1)?)
}
