//! Durable credit for interleaved parts and turns (smith/docs/design/domain/turn-parts.md, section 3).
//! Pending records retain their bodies until the host has room. Acknowledgements
//! release their own prefix; the answer follows every pending record.

use crate::{Request, Window};
use skein_lib::Queue;

#[derive(Debug)]
pub(crate) struct Flow {
    window: Window,
    parts: Queue<(u32, u64)>,
    turns: Queue<(u32, u64)>,
    bytes: u64,
    pending: Queue<(Record, Request)>,
    done: bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Record {
    Part { number: u32, bytes: u64 },
    Turn { number: u32, bytes: u64 },
    Answer,
}

impl Flow {
    pub(crate) fn new(window: Window) -> Self {
        Self {
            window,
            parts: Queue::uncapped(),
            turns: Queue::uncapped(),
            bytes: 0,
            pending: Queue::uncapped(),
            done: false,
        }
    }

    pub(crate) const fn done(&self) -> bool {
        self.done
    }

    pub(crate) fn push(&mut self, record: Record, request: Request, out: &mut Queue<Request>) {
        self.pending.push((record, request));
        self.flush(out);
    }

    pub(crate) fn keep(&mut self, part: bool, number: u32, out: &mut Queue<Request>) {
        let held = if part { &mut self.parts } else { &mut self.turns };
        for _ in 0..held.len() {
            let Some((first, _)) = held.iter().next() else { break };
            if *first > number {
                break;
            }
            let (_, bytes) = held.pop().expect("acknowledged record");
            self.bytes = self.bytes.checked_sub(bytes).expect("record's reserved bytes");
        }
        self.flush(out);
    }

    fn flush(&mut self, out: &mut Queue<Request>) {
        for _ in 0..self.pending.len() {
            let record = self.pending.iter().next().expect("pending record").0;
            let bytes = match record {
                Record::Part { bytes, .. } | Record::Turn { bytes, .. } => bytes,
                Record::Answer => 0,
            };
            let full = match record {
                Record::Part { .. } | Record::Turn { .. } => {
                    self.parts.len().saturating_add(self.turns.len()) >= self.window.turns
                        || bytes > self.window.bytes.saturating_sub(self.bytes)
                }
                Record::Answer => false,
            };
            if full {
                break;
            }
            let (_, request) = self.pending.pop().expect("pending record");
            match record {
                Record::Part { number, bytes } => self.parts.push((number, bytes)),
                Record::Turn { number, bytes } => self.turns.push((number, bytes)),
                Record::Answer => self.done = true,
            }
            self.bytes = self.bytes.checked_add(bytes).expect("admitted byte credit");
            out.push(request);
        }
    }
}
