//! Durability before exchanges, and owner-versioned restoration (protocol/client/README.md, 7).
use crate::component::Gathered;
use crate::{Fact, Limits, Link, Message, Record, Request, Saved, translate};
use alloc::boxed::Box;
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use skein_lib::{List, Map, Queue, Reader, RequestKey, Token, Writer, bytes, length::len_u64};
#[derive(Debug)]
struct Held {
    exchange: Token,
    message: Message,
    credential: Option<skein_lib::Secret>,
}
#[derive(Debug)]
pub(super) struct Store {
    waiting: bool,
    held: Queue<Held>,
    unwritten: Map<RequestKey, ()>,
}
impl Store {
    pub(super) fn new() -> Self {
        Self { waiting: false, held: Queue::uncapped(), unwritten: Map::uncapped() }
    }
}
/// Whether these limits are representable: every edge bound they size
/// is finite and every capacity they derive fits (programming-model.md, 6.3).
#[must_use]
pub(crate) fn valid(limits: &Limits) -> bool {
    w::Keyed::encoded_bound(limits.wire()).is_some()
}

pub(super) fn key(record: &Saved) -> Box<[u8]> {
    bytes::copy_of(&record.record.key)
}
fn encoded(value: &Saved, limits: &Limits) -> Result<Box<[u8]>, w::Problem> {
    let table = translate::store::table(&value.record, limits)?;
    let mut writer = Writer::new(skein_lib::length::usize_of(table.measure(limits.wire())?));
    table.write(limits.wire(), &mut writer).expect("measured table record");
    let record = w::Kept { store_version: value.version, record: writer.finish() };
    if u64::from(record.measure(limits.wire())?) > limits.client().document_bytes {
        return Err(translate::too_large());
    }
    record.encode(limits.wire())
}
fn restore(value: w::Kept, storage_key: &[u8], limits: &Limits) -> Result<Saved, w::Problem> {
    let mut reader = Reader::new(&value.record);
    let record = translate::store::restore_table(w::TableEntry::decode(limits.wire(), &mut reader)?, limits)?;
    if reader.remaining() != 0 || record.key.as_slice() != storage_key {
        return Err(translate::piece_problem(skein_codec::Reason::Malformed));
    }
    Ok(Saved { version: value.store_version, record })
}
impl Link {
    /// Keeps keyed exchanges behind writes the composing client has already given.
    pub fn wait_for_store(&mut self) {
        self.store.waiting = true;
    }
    pub(super) fn exchange(&mut self, exchange: Token, message: Message, out: &mut Queue<Request>) {
        self.gather_exchange(exchange, self.credential.clone(), message, out);
    }
    fn gather_exchange(
        &mut self,
        exchange: Token,
        credential: Option<skein_lib::Secret>,
        message: Message,
        _out: &mut Queue<Request>,
    ) {
        let tracked = match &message {
            Message::Batch(_) => unreachable!("batches are built only at settling"),
            Message::Request(_) | Message::Read(_) | Message::Watch(_) => true,
            Message::Unwatch(_)
            | Message::StreamGone(_)
            | Message::SignInLink(_)
            | Message::SignInService
            | Message::SignOut
            | Message::Authorize(_)
            | Message::LinkAsk(_)
            | Message::ApprovalAsk(_)
            | Message::ApprovalRead(_)
            | Message::ApprovalDecide(_)
            | Message::SignInApproval(_) => false,
        };
        if !self.attempts.contains_key(&exchange) && tracked {
            return;
        }
        // A request carrying a value is never kept, so never waits on a write.
        let keyed = match &message {
            Message::Batch(_) => unreachable!("batches are built only at settling"),
            Message::Request(_) => match self.attempts.get(&exchange) {
                Some(attempt) => !attempt.carrying(),
                None => true,
            },
            Message::Read(_)
            | Message::Watch(_)
            | Message::Unwatch(_)
            | Message::StreamGone(_)
            | Message::SignInLink(_)
            | Message::SignInService
            | Message::SignOut
            | Message::Authorize(_)
            | Message::LinkAsk(_)
            | Message::ApprovalAsk(_)
            | Message::ApprovalRead(_)
            | Message::ApprovalDecide(_)
            | Message::SignInApproval(_) => false,
        };
        if !keyed {
            self.emit_exchange(exchange, credential, message);
            return;
        }
        if let Some(attempt) = self.attempts.get(&exchange)
            && let crate::component::ExchangeKind::Attempt { key, .. } = attempt.kind
            && self.store.unwritten.remove(&key).is_some()
        {
            self.attempts.remove(&exchange);
            return;
        }
        if self.store.waiting {
            self.store.held.push(Held { exchange, message, credential });
        } else {
            self.emit_exchange(exchange, credential, message);
        }
    }
    fn gather(&mut self, out: &mut Queue<Request>) {
        self.store.waiting = true;
        // A keyed duty still in the composer's output queue has not reached its transport.
        // Keep the write barrier ahead of it; batches contain no keyed requests.
        let mut pending = core::mem::replace(&mut self.gathered, Queue::uncapped());
        for _ in 0..out.len() {
            let value = out.pop().expect("counted composer outputs");
            match value {
                Request::Exchange { exchange, message: message @ Message::Request(_), credential } => {
                    self.transport.remove(&exchange);
                    self.emit_exchange(exchange, credential, message);
                }
                value @ (Request::Exchange { .. }
                | Request::Save { .. }
                | Request::Erase { .. }
                | Request::Kept { .. }
                | Request::Dropped
                | Request::Answer { .. }
                | Request::NotKept { .. }
                | Request::NotWritten { .. }
                | Request::OpenStream { .. }
                | Request::CloseStream
                | Request::StreamsFull { .. }
                | Request::Watch { .. }
                | Request::FamilyDelivery { .. }
                | Request::Ended { .. }
                | Request::Hello { .. }
                | Request::ReadAgain { .. }
                | Request::Read { .. }
                | Request::AskedLink(_)
                | Request::Approval(_)
                | Request::Authorized(_)
                | Request::Signing { .. }
                | Request::ServiceCredential { .. }
                | Request::Fact { .. }) => out.push(value),
            }
        }
        for _ in 0..pending.len() {
            let value = pending.pop().expect("counted pending transport");
            self.gathered.push(value);
        }
        let count = self.gathered.len();
        for _ in 0..count {
            match self.gathered.pop().expect("counted gathered transport") {
                Gathered::Exchange { exchange, message, credential } => match message {
                    Message::Batch(_) => unreachable!("batches are built only at settling"),
                    message @ (Message::LinkAsk(_)
                    | Message::ApprovalAsk(_)
                    | Message::ApprovalRead(_)
                    | Message::ApprovalDecide(_)
                    | Message::SignInApproval(_)) => self.emit_exchange(exchange, credential, message),
                    message @ (Message::Authorize(_)
                    | Message::Request(_)
                    | Message::Read(_)
                    | Message::Watch(_)
                    | Message::Unwatch(_)
                    | Message::StreamGone(_)
                    | Message::SignInLink(_)
                    | Message::SignInService
                    | Message::SignOut) => self.gather_exchange(exchange, credential, message, out),
                },
                value @ (Gathered::OpenStream { .. } | Gathered::CloseStream) => self.gathered.push(value),
            }
        }
    }

    pub(super) fn save(&mut self, record: Saved, out: &mut Queue<Request>) {
        self.gather(out);
        let key = key(&record);
        match encoded(&record, &self.limits) {
            Ok(bytes) => out.push(Request::Save { record: Record { key, bytes } }),
            Err(_) => {
                self.store.unwritten.insert(record.record.key, ()).expect("uncapped unwritten keys");
                out.push(Request::NotWritten { key: record.record.key });
            }
        }
    }
    pub(super) fn erase(&mut self, key: RequestKey, out: &mut Queue<Request>) {
        self.gather(out);
        self.store.unwritten.remove(&key);
        out.push(Request::Erase { key: bytes::copy_of(&key) });
    }
    pub(super) fn written(&mut self) {
        self.store.waiting = false;
        // Durability releases held exchanges in FIFO order, skipping stale attempts.
        while let Some(value) = self.store.held.pop() {
            if self.attempts.contains_key(&value.exchange) {
                self.emit_exchange(value.exchange, value.credential, value.message);
            }
        }
    }
    pub(super) fn write_failed(&mut self, out: &mut Queue<Request>) {
        self.store.waiting = false;
        while let Some(value) = self.store.held.pop() {
            if let Some(attempt) = self.attempts.remove(&value.exchange) {
                out.push(Request::NotKept { attempt: attempt.token });
            }
        }
    }
    pub(super) fn save_stream(&mut self, stream: [u8; 16], out: &mut Queue<Request>) -> Result<(), crate::Undecodable> {
        let Ok(value) = (w::Kept { store_version: jig_client_domain::TABLE_VERSION, record: Box::from(stream) })
            .encode(self.limits.wire())
        else {
            return Err(crate::Undecodable::TooLarge);
        };
        out.push(Request::Save { record: Record { key: bytes::copy_of(crate::STREAM_KEY), bytes: value } });
        Ok(())
    }
    pub(super) fn start_store(&mut self, kept: List<Record>, out: &mut Queue<Request>) {
        let mut bound = self.limits.client().requests.requests;
        for record in &kept {
            if record.key.as_ref() == crate::STREAM_KEY {
                bound = bound.saturating_add(1);
                break;
            }
        }
        if kept.len() > bound {
            out.push(Request::Fact { fact: Fact::UndecodableStore });
            return;
        }
        for entry in kept.into_boxed() {
            let mut reader = Reader::new(&entry.bytes);
            let result = if (entry.key.len() != 16 && entry.key.as_ref() != crate::STREAM_KEY)
                || len_u64(entry.bytes.len()) > self.limits.client().document_bytes
            {
                Err(translate::too_large())
            } else {
                w::Kept::decode(self.limits.wire(), &mut reader)
            };
            let Ok(value) = result else {
                out.push(Request::Dropped);
                out.push(Request::Fact { fact: Fact::UndecodableStore });
                continue;
            };
            if reader.remaining() != 0 || value.store_version != jig_client_domain::TABLE_VERSION {
                out.push(Request::Dropped);
                continue;
            }
            if entry.key.as_ref() == crate::STREAM_KEY {
                match <[u8; 16]>::try_from(value.record.as_ref()) {
                    Ok(stream) => self.predecessor = Some(stream),
                    Err(_) => {
                        out.push(Request::Dropped);
                        out.push(Request::Fact { fact: Fact::UndecodableStore });
                    }
                }
                continue;
            }
            match restore(value, &entry.key, &self.limits) {
                Ok(record) => out.push(Request::Kept { record }),
                Err(_) => {
                    out.push(Request::Dropped);
                    out.push(Request::Fact { fact: Fact::UndecodableStore });
                }
            }
        }
    }
}
