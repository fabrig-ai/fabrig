//! Bounded source reads for task documents' asked words (client/ctl/results.md, 4).
use super::reads::Reading;
use crate::jig::{Limits, render};
use alloc::boxed::Box;
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
    /// The source field kept by these sequential reads.
    pub field: c::TaskTextField,
    /// The source offset at which the retained prefix starts.
    pub from: u64,
}

/// Request the next bounded source chunk for task text or plain task show.
pub(super) fn next(reading: &mut Reading, limits: &Limits) -> Option<c::Read> {
    if reading.called.call.tool == crate::jig::rows::TASK_TEXT_AT {
        return next_text(reading, limits);
    }
    if (reading.called.call.tool != 3 && reading.called.call.tool != crate::jig::rows::ROLE_SHOW_AT)
        || reading.called.json
    {
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
    let task = crate::jig::call::number(&reading.called.call, 0).expect("task source read");
    reading.specification.as_mut().expect("requested specification chunk").accept(task, page);
}

impl Specification {
    fn offset(&self) -> u64 {
        self.from.checked_add(u64::from(self.bytes.len())).expect("source offset")
    }

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
        Some(Self { bytes, more: true, total: None, stopped: false, field: c::TaskTextField::Specification, from: 0 })
    }

    /// Request the next sequential chunk while source room remains and reads advance.
    pub(crate) fn next(&self, task: u64) -> Option<c::Read> {
        if !self.more || self.stopped || self.bytes.room() == 0 {
            return None;
        }
        Some(c::Read::TaskText { task, field: self.field, from: self.offset() })
    }

    /// Extend this task's prefix, stopping on refused, inconsistent or nonadvancing chunks.
    pub(crate) fn accept(&mut self, task: u64, page: &Readout) {
        // A refused, changed or nonadvancing source keeps the overview and a continuation.
        self.stopped = true;
        let Readout::Jig(c::ReadResult::Value(c::ReadValue::TaskText(page))) = page else { return };
        let from = self.offset();
        let Some(end) = page.from.checked_add(skein_lib::len_u64(page.text.bytes().len())) else {
            return;
        };
        let changed = match self.total {
            Some(total) => total != page.total,
            None => false,
        };
        let next = if end < page.total { Some(end) } else { None };
        if page.task != task
            || page.field != self.field
            || page.from != from
            || (end <= from && end != page.total)
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
        self.more = self.offset() < page.total;
        self.stopped = bytes.len() != page.text.bytes().len();
    }
}

fn next_text(reading: &mut Reading, limits: &Limits) -> Option<c::Read> {
    let task = crate::jig::call::number(&reading.called.call, 0)?;
    if reading.specification.is_none() {
        let Readout::Jig(c::ReadResult::Value(c::ReadValue::TaskText(page))) = reading.pages.get(&1)? else {
            return None;
        };
        let mut source = Specification {
            bytes: List::with_capacity(u32::try_from(render::task_source::capacity(limits)).expect("text capacity")),
            more: true,
            total: None,
            stopped: false,
            field: page.field,
            from: page.from,
        };
        source.accept(task, reading.pages.get(&1).expect("first text page"));
        reading.specification = Some(source);
    }
    let source = reading.specification.as_ref().expect("text source");
    if let Some(total) = source.total {
        let page = c::TaskText {
            task,
            field: source.field,
            from: source.from,
            total,
            next: if source.more { Some(source.offset()) } else { None },
            text: c::Text::new(skein_lib::bytes::copy_of(source.bytes.as_slice()), source.bytes.len())
                .expect("joined UTF-8"),
        };
        reading.pages.insert(1, Readout::Jig(c::ReadResult::Value(c::ReadValue::TaskText(Box::new(page)))));
    }
    source.next(task)
}
