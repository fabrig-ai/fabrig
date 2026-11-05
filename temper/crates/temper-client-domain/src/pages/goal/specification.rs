//! Read cut goal words before finding their draft provenance (client/README.md, 5.8, 5.14).
use super::{Request, c, draft_of, handle, read};
use alloc::boxed::Box;
use jig_client::{Handle, Readout, Told};
use skein_lib::{List, Queue};

#[derive(Debug)]
pub(super) struct Specification {
    handle: Handle,
    bytes: List<u8>,
    total: Option<u64>,
    reading: bool,
}
impl Specification {
    pub(super) fn new(task: &c::Task, bound: u32, next: &mut u64, out: &mut Queue<Request>) -> Option<Self> {
        if !task.spec.specification.words.cut {
            return None;
        }
        let handle = handle(*next);
        *next = next.checked_add(1).expect("goal handles fit");
        read(handle, c::Read::TaskText { task: task.chip.task, field: c::TaskTextField::Specification, from: 0 }, out);
        Some(Self { handle, bytes: List::with_capacity(bound), total: None, reading: true })
    }
    pub(super) fn told(
        &mut self,
        told: &Told,
        task: u64,
        draft: &mut (Option<Box<[u8]>>, Option<u64>),
        out: &mut Queue<Request>,
    ) -> bool {
        let Told::Page { handle, page } = told else { return false };
        if *handle != self.handle {
            return false;
        }
        if !self.reading {
            return true;
        }
        self.reading = false;
        let Readout::Jig(c::ReadResult::Value(c::ReadValue::TaskText(page))) = page else { return true };
        let Some(end) = page.from.checked_add(skein_lib::len_u64(page.text.bytes().len())) else { return true };
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
            return true;
        }
        self.total = Some(page.total);
        for byte in page.text.bytes() {
            self.bytes.push(*byte).expect("validated source bound");
        }
        if let Some(from) = page.next {
            self.reading = true;
            read(self.handle, c::Read::TaskText { task, field: c::TaskTextField::Specification, from }, out);
        } else {
            *draft = draft_of(self.bytes.as_slice());
            self.bytes.clear();
        }
        true
    }
    pub(super) fn leave(&self, out: &mut Queue<Request>) {
        out.push(Request::Client(jig_client::Event::Unwatch { handle: self.handle }));
    }
}
