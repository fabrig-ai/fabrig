//! A person task's question read whole (temper/docs/design/client/pages/README.md, Whole).
use super::{Request, client};
use jig_client::{Handle, Readable, Readout, contract as c};
use skein_lib::{List, Queue};

#[derive(Clone, Debug)]
pub struct Whole {
    pub(super) handle: Handle,
    bytes: List<u8>,
    total: Option<u64>,
    pub(super) reading: bool,
}
impl Whole {
    pub(super) fn start(task: u64, handle: Handle, out: &mut Queue<Request>) -> Self {
        read(task, handle, 0, out);
        Self { handle, bytes: List::with_capacity(0), total: None, reading: true }
    }
    /// Takes one page, asking for the next.
    pub(super) fn told(&mut self, task: u64, bound: u32, page: Readout, out: &mut Queue<Request>) {
        if !self.reading {
            return;
        }
        self.reading = false;
        let Readout::Jig(c::ReadResult::Value(c::ReadValue::TaskText(page))) = page else { return };
        let Some(end) = page.from.checked_add(skein_lib::len_u64(page.text.bytes().len())) else { return };
        if page.task != task
            || page.field != c::TaskTextField::Specification
            || page.from != u64::from(self.bytes.len())
            || page.total > u64::from(bound)
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
        if self.total.is_none() {
            self.bytes = List::with_capacity(u32::try_from(page.total).expect("validated whole bound"));
        }
        self.total = Some(page.total);
        for byte in page.text.bytes() {
            self.bytes.push(*byte).expect("validated whole bound");
        }
        if let Some(from) = page.next {
            self.reading = true;
            read(task, self.handle, from, out);
        }
    }
    /// The whole, once it has it all.
    #[must_use]
    pub fn words(&self) -> Option<&[u8]> {
        if !self.reading && self.total == Some(u64::from(self.bytes.len())) {
            Some(self.bytes.as_slice())
        } else {
            None
        }
    }
}
fn read(task: u64, handle: Handle, from: u64, out: &mut Queue<Request>) {
    client(
        out,
        jig_client::Event::Read {
            handle,
            read: Readable::Jig(c::Read::TaskText { task, field: c::TaskTextField::Specification, from }),
        },
    );
}
