//! Bounded source reads for task documents' asked words (client/tools.md, 6.4).
use super::reads::Reading;
use crate::jig::{Limits, render};
use jig_client::{Readout, contract as c};
use skein_lib::List;

/// A bounded source prefix and the continuation left after supplemental reads.
#[derive(Debug)]
pub(crate) struct Specification {
    /// The source prefix admitted under the call's text bounds.
    pub bytes: List<u8>,
    /// Source bytes remain beyond the retained prefix.
    pub more: bool,
    /// Source length when a supplemental read supplied it.
    pub total: Option<u64>,
    /// Retrieval cannot advance; the prefix keeps its continuation.
    pub stopped: bool,
}

/// Request one missing specification chunk for plain task show within its source bound.
pub(super) fn next(reading: &mut Reading, limits: &Limits) -> Option<c::Read> {
    if reading.called.call.tool != 3 || reading.called.json {
        return None;
    }
    if reading.specification.is_none() {
        let Readout::Jig(c::ReadResult::Value(value)) = reading.pages.get(&1)? else { return None };
        let (task, _) = render::read_task(value)?;
        reading.specification = Specification::new(task, limits);
    }
    reading.specification.as_ref()?.next(crate::jig::call::number(&reading.called.call, 0)?)
}

fn capacity(limits: &Limits) -> usize {
    let result = limits.result_bytes.saturating_sub(1024).checked_div(6).expect("nonzero divisor");
    usize::try_from(u64::from(limits.part.min(limits.text)).min(result)).expect("source limit fits")
}

/// Extend the matching prefix, stopping on a refused, inconsistent or nonadvancing chunk.
pub(super) fn accept(reading: &mut Reading, page: &Readout) {
    let task = crate::jig::call::number(&reading.called.call, 0).expect("task show");
    reading.specification.as_mut().expect("requested specification chunk").accept(task, page);
}

impl Specification {
    /// Retain the cut overview within the same source bound used by supplemental reads.
    pub(crate) fn new(task: &c::Task, limits: &Limits) -> Option<Self> {
        let words = &task.spec.specification.words;
        if !words.cut {
            return None;
        }
        let mut bytes = List::with_capacity(u32::try_from(capacity(limits)).expect("source bound"));
        for byte in skein_lib::bytes::utf8_prefix(words.bytes(), capacity(limits)) {
            bytes.push(*byte).expect("bounded overview");
        }
        Some(Self { bytes, more: true, total: None, stopped: false })
    }

    /// Request the next sequential chunk while source room remains and reads advance.
    pub(crate) fn next(&self, task: u64) -> Option<c::Read> {
        if !self.more || self.stopped || self.bytes.room() == 0 {
            return None;
        }
        Some(c::Read::TaskText { task, field: c::TaskTextField::Specification, from: u64::from(self.bytes.len()) })
    }

    /// Extend this task's prefix, stopping on refused, inconsistent or nonadvancing chunks.
    pub(crate) fn accept(&mut self, task: u64, page: &Readout) {
        // A refused, changed or nonadvancing source keeps the overview and a continuation.
        self.stopped = true;
        let Readout::Jig(c::ReadResult::Value(c::ReadValue::TaskText(page))) = page else { return };
        let from = u64::from(self.bytes.len());
        let Some(end) = page.from.checked_add(u64::try_from(page.text.bytes().len()).expect("chunk bound")) else {
            return;
        };
        let changed = match self.total {
            Some(total) => total != page.total,
            None => false,
        };
        let next = if end < page.total { Some(end) } else { None };
        if page.task != task
            || page.field != c::TaskTextField::Specification
            || page.from != from
            || end <= from
            || end > page.total
            || changed
            || page.next != next
        {
            return;
        }
        let room = usize::try_from(self.bytes.room()).expect("source bound fits");
        let bytes = skein_lib::bytes::utf8_prefix(page.text.bytes(), room);
        for byte in bytes {
            self.bytes.push(*byte).expect("bounded source");
        }
        self.total = Some(page.total);
        self.more = u64::from(self.bytes.len()) < page.total;
        self.stopped = bytes.len() != page.text.bytes().len();
    }
}
