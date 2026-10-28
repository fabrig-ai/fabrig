//! A plain queue model of the journal's promises (lib.md, section 11).
//!
//! It stores writes and outputs in standard library queues, computes
//! releases from their contents, and answers commits in send order. Each
//! operation is compared with the public journal interface.

use std::collections::VecDeque;

use skein_lib::{Journal, Queue, Released, Rng};

struct Model {
    release: u32,
    outstanding: VecDeque<u64>,
    unsent: VecDeque<(u64, Vec<u32>)>,
    sent: VecDeque<u64>,
    held: VecDeque<(u64, u32)>,
    now: VecDeque<u32>,
    last: u64,
    durable: u64,
    stopped: bool,
}

impl Model {
    fn new(release: u32) -> Model {
        Model {
            release,
            outstanding: VecDeque::new(),
            unsent: VecDeque::new(),
            sent: VecDeque::new(),
            held: VecDeque::new(),
            now: VecDeque::new(),
            last: 0,
            durable: 0,
            stopped: false,
        }
    }

    fn accept(&mut self, writes: Vec<u32>, held: Vec<u32>) {
        if self.stopped {
            return;
        }
        if !writes.is_empty() {
            self.last += 1;
            self.outstanding.push_back(self.last);
            self.unsent.push_back((self.last, writes));
        }
        for output in held {
            self.held.push_back((self.last, output));
        }
    }

    fn now(&mut self, output: u32) -> Result<(), u32> {
        if self.stopped {
            return Err(output);
        }
        self.now.push_back(output);
        Ok(())
    }

    fn committed(&mut self, number: u64) {
        while self.sent.front().is_some_and(|&front| front <= number) {
            self.sent.pop_front();
        }
        while self.outstanding.front().is_some_and(|&front| front <= number) {
            self.outstanding.pop_front();
        }
        self.durable = number;
    }

    fn release(&mut self, capacity: u32) -> (Released, Vec<u32>) {
        if self.stopped {
            return (Released::Stopped, Vec::new());
        }
        let mut outputs = Vec::new();
        let n = self.release.min(capacity);
        for _ in 0..n {
            if let Some(output) = self.now.pop_front() {
                outputs.push(output);
            } else if self.held.front().is_some_and(|&(after, _)| after <= self.durable) {
                let (_, output) = self.held.pop_front().expect("ready front exists");
                outputs.push(output);
            } else {
                break;
            }
        }
        let status = if outputs.is_empty() { Released::None } else { Released::Some };
        (status, outputs)
    }
}

fn below(rng: &mut Rng, n: u32) -> u32 {
    u32::try_from(rng.below(u64::from(n))).expect("below u32")
}

/// Runs random decision, send, answer, failure, door, and release sequences.
pub fn check(seed: u64, rounds: u32) {
    // Two outputs may leave in each call.
    const RELEASE: u32 = 2;
    let mut rng = Rng::new(seed);
    for round in 0..rounds {
        let mut journal = Journal::<u32, u32>::uncapped(RELEASE, 0);
        let mut model = Model::new(RELEASE);
        for step in 0..60_u32 {
            let writes = below(&mut rng, 4);
            let held = below(&mut rng, 4);
            match below(&mut rng, 10) {
                0..=4 => {
                    let mut decision = journal.decide();
                    let write_count = below(&mut rng, writes + 1);
                    let held_count = below(&mut rng, held + 1);
                    let mut writes = Vec::new();
                    let mut held = Vec::new();
                    for _ in 0..write_count {
                        let write = rng.next_u64().to_le_bytes();
                        let value = u32::from_le_bytes([write[0], write[1], write[2], write[3]]);
                        decision.push_write(value);
                        writes.push(value);
                    }
                    for _ in 0..held_count {
                        let value = rng.next_u64().to_le_bytes();
                        let output = u32::from_le_bytes([value[0], value[1], value[2], value[3]]);
                        decision.push_held(output);
                        held.push(output);
                    }
                    journal.accept(decision);
                    model.accept(writes, held);
                }
                5 => {
                    let output = below(&mut rng, 1000);
                    assert_eq!(journal.now(output), model.now(output), "round {round}, step {step}: door");
                }
                6 => {
                    let actual = journal.commit();
                    let expected = model.unsent.pop_front();
                    match (actual, expected) {
                        (Some(mut commit), Some((number, writes))) => {
                            assert_eq!(commit.number, number, "round {round}, step {step}: number");
                            for write in writes {
                                assert_eq!(commit.writes.pop(), Some(write));
                            }
                            assert_eq!(commit.writes.pop(), None);
                            model.sent.push_back(number);
                        }
                        (None, None) => {}
                        _ => panic!("round {round}, step {step}: commit mismatch"),
                    }
                }
                7 => {
                    if !model.sent.is_empty() {
                        let number = if rng.chance(500) {
                            *model.sent.back().expect("sent is nonempty")
                        } else {
                            *model.sent.front().expect("sent is nonempty")
                        };
                        journal.committed(number);
                        model.committed(number);
                    }
                }
                8 => {
                    if !model.sent.is_empty() && rng.chance(30) {
                        let number = *model.sent.front().expect("sent is nonempty");
                        journal.failed(number);
                        model.stopped = true;
                    }
                }
                9 => {
                    let capacity = below(&mut rng, 4);
                    let mut out = Queue::with_capacity(capacity);
                    let actual = journal.release(&mut out);
                    let (expected, values) = model.release(capacity);
                    assert_eq!(actual, expected, "round {round}, step {step}: release status");
                    for value in values {
                        assert_eq!(out.pop(), Some(value), "round {round}, step {step}: release order");
                    }
                    assert_eq!(out.pop(), None);
                }
                _ => unreachable!("draw below ten"),
            }
            assert_eq!(journal.stopped(), model.stopped, "round {round}, step {step}: stopped");
            if model.stopped {
                let mut out = Queue::with_capacity(3);
                assert_eq!(journal.release(&mut out), Released::Stopped, "round {round}, step {step}: after failure");
                assert_eq!(out.pop(), None, "round {round}, step {step}: nothing after failure");
                break;
            }
        }
    }
}
