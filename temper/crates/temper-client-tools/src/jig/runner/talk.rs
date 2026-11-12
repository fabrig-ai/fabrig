//! A talk's reply is the committed turns after its admitted words, through the wait's delivery.
use crate::jig::{Limits, Printed, render};
use alloc::{boxed::Box, collections::BTreeMap};
use jig_client::{contract as c, thread::Thread};

#[derive(Debug)]
pub(crate) struct Reply {
    message: u64,
    through: Option<u64>,
    pub(crate) met: bool,
    pub(crate) chunks: BTreeMap<u64, Box<[u8]>>,
    bytes: u64,
}
impl Reply {
    pub(crate) fn new(message: u64, through: Option<u64>, met: bool) -> Self {
        Self { message, through, met, chunks: BTreeMap::new(), bytes: 0 }
    }
    /// Keep text, not an unbounded activity history, while the thread reads back to the words.
    pub(crate) fn page(
        &mut self,
        thread: &mut Thread,
        page: c::ActivityPage,
        limits: &Limits,
    ) -> Result<Option<c::Read>, Printed> {
        let mut from = None;
        let mut first = u64::MAX;
        for row in &page.rows {
            first = first.min(row.source.position);
            if let c::Activity::Words { message, .. } = row.source.line
                && message == self.message
            {
                from = Some(row.source.position);
            }
        }
        let cut = page.cut;
        drop(thread.accept(page));
        let chunk = render::roles::reply_page(thread, from.unwrap_or(0), self.through, limits)?;
        self.bytes = self.bytes.saturating_add(skein_lib::len_u64(chunk.len()));
        if self.bytes > limits.result_bytes {
            return Err(render::failure(b"result limit", false, 1));
        }
        if !chunk.is_empty() {
            self.chunks.insert(first, chunk);
        }
        // Pages are rendered before the next bounded window replaces them.
        thread.lines.rows.clear();
        if from.is_some() {
            return Ok(None);
        }
        let next = thread.older();
        if cut || next.is_none() {
            return Err(render::failure(b"talk words not found in activity", false, 1));
        }
        Ok(next)
    }
}
