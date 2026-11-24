//! Durable credit for interleaved parts and turns (smith/docs/design/domain/turn-parts.md, section 3).
//! Pending records retain their bodies until the host has room. Acknowledgements
//! release their own prefix; the answer follows every pending record. A lost
//! channel discards its unsent records and lets the runtime take the answer.

use crate::{Request, Window};
use skein_lib::Queue;

#[derive(Debug)]
pub(crate) struct Flow {
    window: Window,
    track: bool,
    disconnected: bool,
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
    pub(crate) fn new(window: Window, track: bool) -> Self {
        Self {
            window,
            track,
            disconnected: false,
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

    pub(crate) const fn disconnected(&self) -> bool {
        self.disconnected
    }

    pub(crate) fn push(&mut self, record: Record, request: Request, out: &mut Queue<Request>) {
        if self.disconnected {
            match record {
                Record::Answer => {
                    self.done = true;
                    out.push(request);
                }
                Record::Part { .. } | Record::Turn { .. } => {}
            }
            return;
        }
        if !self.track {
            match record {
                Record::Answer => self.done = true,
                Record::Part { .. } | Record::Turn { .. } => {}
            }
            out.push(request);
            return;
        }
        self.pending.push((record, request));
        self.flush(out);
    }

    pub(crate) fn disconnect(&mut self, out: &mut Queue<Request>) {
        self.disconnected = true;
        self.parts = Queue::uncapped();
        self.turns = Queue::uncapped();
        self.bytes = 0;
        for _ in 0..self.pending.len() {
            let (record, request) = self.pending.pop().expect("pending record");
            self.push(record, request, out);
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use skein_lib::Token;

    #[test]
    fn a_disconnected_host_cannot_hold_the_terminal_behind_unacknowledged_records() {
        let mut flow = Flow::new(Window { turns: 1, bytes: 1 }, true);
        let mut out = Queue::uncapped();
        flow.push(Record::Part { number: 1, bytes: 1 }, Request::LongDone { host_run: Token::new(1) }, &mut out);
        assert!(out.pop().is_some());
        flow.push(Record::Turn { number: 1, bytes: 1 }, Request::LongDone { host_run: Token::new(2) }, &mut out);
        flow.push(Record::Answer, Request::LongDone { host_run: Token::new(3) }, &mut out);
        assert!(out.is_empty());
        flow.disconnect(&mut out);
        let Some(Request::LongDone { host_run }) = out.pop() else { panic!("terminal bypasses lost credit") };
        assert_eq!(host_run, Token::new(3));
        assert!(out.is_empty(), "unsent turn is discarded with the channel");
        assert!(flow.done());
    }

    #[test]
    fn whole_turn_runs_leave_no_second_unacknowledged_queue() {
        let mut flow = Flow::new(Window { turns: 1, bytes: 1 }, false);
        let mut out = Queue::uncapped();
        for number in 1..=16 {
            flow.push(Record::Turn { number, bytes: 1 }, Request::LongDone { host_run: Token::new(1) }, &mut out);
        }
        flow.push(Record::Answer, Request::LongDone { host_run: Token::new(1) }, &mut out);
        assert_eq!(out.len(), 17);
        assert!(flow.turns.is_empty());
        assert!(flow.pending.is_empty());
        assert_eq!(flow.bytes, 0);
        assert!(flow.done());
    }
}
