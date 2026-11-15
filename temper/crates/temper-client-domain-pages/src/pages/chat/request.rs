//! A chat's first message read whole within `long_text` (temper/docs/design/client/view.md, 4).
use super::{Request, c};
use crate::pages::activity::{Asking, read_end};
use jig_client::{Client, Handle, Readout};
use skein_lib::{List, Queue, Time};

#[derive(Debug, PartialEq, Eq)]
enum State {
    Initial,
    Reading,
    Stopped,
    Whole,
}

#[derive(Debug)]
pub(super) struct Reader {
    pub(super) handle: Option<Handle>,
    bytes: List<u8>,
    total: Option<u64>,
    state: State,
    asking: Asking,
}
impl Reader {
    pub(super) fn new(bound: u32) -> Self {
        Self {
            handle: None,
            bytes: List::with_capacity(bound),
            total: None,
            state: State::Initial,
            asking: Asking::new(),
        }
    }
    pub(super) fn start(&mut self, task: &c::Task, next: &mut u64, out: &mut Queue<Request>) {
        if self.state != State::Initial || !task.spec.specification.words.cut {
            return;
        }
        let handle = super::handle(*next);
        *next = next.checked_add(1).expect("chat handles fit");
        self.handle = Some(handle);
        self.state = State::Reading;
        self.asking.ask(
            handle,
            c::Read::TaskText { task: task.chip.task, field: c::TaskTextField::Specification, from: 0 },
            out,
        );
    }
    pub(super) fn words<'a>(&'a self, document: &'a c::DocumentText) -> (&'a [u8], bool) {
        match self.state {
            State::Initial => (document.bytes(), !document.cut),
            State::Reading | State::Stopped => {
                if self.bytes.is_empty() {
                    (document.bytes(), false)
                } else {
                    (self.bytes.as_slice(), false)
                }
            }
            State::Whole => (self.bytes.as_slice(), true),
        }
    }
    pub(super) fn page(&mut self, now: Time, client: &Client, task: u64, page: Readout, out: &mut Queue<Request>) {
        if self.state != State::Reading {
            return;
        }
        let Readout::Jig(c::ReadResult::Value(c::ReadValue::TaskText(page))) = page else {
            if read_end(client, &page).transient() {
                self.asking.ended(now);
            } else {
                self.asking.answered();
                self.state = State::Stopped;
            }
            return;
        };
        self.asking.answered();
        self.state = State::Stopped;
        let Some(end) = page.from.checked_add(skein_lib::len_u64(page.text.bytes().len())) else { return };
        if page.task != task
            || page.field != c::TaskTextField::Specification
            || page.from != u64::from(self.bytes.len())
            || page.total > u64::from(self.bytes.capacity())
            || end > page.total
            || match self.total {
                Some(total) => total != page.total,
                None => false,
            }
            || match page.next {
                Some(next) => next != end || end <= page.from || end >= page.total,
                None => end != page.total,
            }
        {
            return;
        }
        self.total = Some(page.total);
        for byte in page.text.bytes() {
            self.bytes.push(*byte).expect("validated request bound");
        }
        if let Some(from) = page.next {
            self.state = State::Reading;
            self.asking.ask(
                self.handle.expect("reading request has a handle"),
                c::Read::TaskText { task, field: c::TaskTextField::Specification, from },
                out,
            );
        } else {
            self.state = State::Whole;
        }
    }
    pub(super) fn fire(&mut self, now: Time, out: &mut Queue<Request>) {
        if let Some(handle) = self.handle {
            self.asking.fire(now, handle, out);
        }
    }
    pub(super) const fn next_deadline(&self) -> Option<Time> {
        self.asking.next_deadline()
    }
    pub(super) fn leave(&self, out: &mut Queue<Request>) {
        if let Some(handle) = self.handle {
            super::unwatch(handle, out);
        }
    }
}
