//! Convert previous branch rows during Link (temper/docs/design/domain/forge/writes.md, 7).
use crate::writes::{Operation, Sender, WRITES, WriteState, Writes, Written};
use crate::{Domain, Limits, Request, StartLink, Stored, What, Writer};
use skein_forge::api::Commit;
use skein_lib::{List, Map, Queue};

/// Prepare every record before changing any row or emitting any save.
pub(crate) fn prepare(d: &Domain, limits: &Limits) -> Result<List<Writes>, StartLink> {
    let mut branches = Map::uncapped();
    for (name, row) in &d.heads {
        if row.owned
            && let What::Branch(_) = name.what
            && !d.writes.contains_key(name)
        {
            branches.insert(name.clone(), Some(row.commit)).expect("uncapped conversion");
        }
    }
    for (name, _) in &d.holds {
        if let What::Branch(_) = name.what
            && !d.writes.contains_key(name)
            && !branches.contains_key(name)
        {
            branches.insert(name.clone(), None).expect("uncapped conversion");
        }
    }
    for resource in d.client.live_keys() {
        if let skein_forge_client::What::Branch(_) = resource.what
            && let Some(head) = d.client.expected(&resource)
            && let Some(name) = crate::domain::from_client(&resource, limits)
            && !d.writes.contains_key(&name)
        {
            branches.insert(name, Some(head)).expect("uncapped conversion");
        }
    }
    let mut converted = List::with_capacity(branches.len());
    for (name, head) in &branches {
        let mut entries = Entries { next: 1, rows: List::with_capacity(u32::try_from(WRITES).expect("stored limit")) };
        if let Some(hold) = d.holds.get(name) {
            let waiting = match hold.beside {
                Some(beside) => beside.waiting,
                None => None,
            };
            let slot = match hold.writer {
                Some(Writer::Run { task, attempt }) => Some((task, attempt, false)),
                Some(Writer::Lost { task, attempt }) => Some((task, attempt, true)),
                Some(Writer::Entry(_)) | None => None,
            };
            let count = hold
                .design_writers
                .len()
                .saturating_add(usize::from(slot.is_some()))
                .saturating_add(usize::from(waiting.is_some()));
            if count > WRITES {
                return Err(StartLink::BranchWrites(name.clone(), count));
            }
            if let Some(after) = waiting {
                entries.push(Operation::Compare { after }, Sender::Previous, WriteState::Uncertain);
            }
            if let Some((task, attempt, lost)) = slot {
                entries.writer(*head, task, attempt, lost);
            }
            for writer in &hold.design_writers {
                entries.writer(*head, writer.task, writer.attempt, writer.rereading);
            }
        }
        let record = Writes { name: name.clone(), head: *head, next: entries.next, entries: entries.rows.into_boxed() };
        converted.push(record).expect("one record per branch");
    }
    Ok(converted)
}

struct Entries {
    next: u64,
    rows: List<Written>,
}

impl Entries {
    fn push(&mut self, operation: Operation, sender: Sender, state: WriteState) {
        self.rows.push(Written { order: self.next, operation, sender, state }).expect("conversion checked the bound");
        self.next = self.next.checked_add(1).expect("orders bounded by WRITES");
    }

    fn writer(&mut self, before: Option<Commit>, task: u64, attempt: u64, lost: bool) {
        let state = if lost { WriteState::Uncertain } else { WriteState::Sent };
        self.push(Operation::Writer { before }, Sender::Attempt { task, attempt }, state);
    }
}

pub(crate) fn write(d: &mut Domain, converted: List<Writes>, out: &mut Queue<Request>) {
    for record in converted.into_boxed() {
        if let Some(hold) = d.holds.get_mut(&record.name)
            && hold.beside.take().is_some()
        {
            out.push(Request::Save { read_afresh: None, release: None, record: Stored::Hold(hold.clone()) });
        }
        d.writes.insert(record.name.clone(), record.clone()).expect("uncapped branch write records");
        out.push(Request::Save { read_afresh: None, release: None, record: Stored::Writes(record) });
    }
}
