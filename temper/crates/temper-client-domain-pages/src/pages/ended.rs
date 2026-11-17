//! Ended groups reread only as their mark grows (client/forge/parts.md, 6).
use jig_client::{Client, Handle, Latest, contract as c};

#[derive(Debug)]
pub(super) struct EndedRead {
    held: u64,
    seen: u64,
    reading: bool,
}
impl EndedRead {
    pub(super) const fn new() -> Self {
        Self { held: 0, seen: 0, reading: false }
    }
    pub(super) const fn reading(&self) -> bool {
        self.reading
    }
    pub(super) fn start(&mut self) {
        self.reading = true;
    }
    pub(super) fn changed(&mut self, client: &Client, live: Handle) -> bool {
        if let Some(document) = client.document(live)
            && let Some(Latest::Jig(c::Delivery::Tasks(tasks))) = document.latest
        {
            self.seen = self.seen.max(tasks.ended_mark);
        }
        !self.reading && self.seen > self.held
    }
    pub(super) fn answered(&mut self, mark: Option<u64>) -> bool {
        self.reading = false;
        if let Some(mark) = mark {
            self.held = mark;
            self.seen > self.held
        } else {
            false
        }
    }
}
