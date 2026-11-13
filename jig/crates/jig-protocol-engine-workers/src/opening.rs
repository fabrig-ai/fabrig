//! Ordered authentication, wires and capacity checks (protocol/channel/wires.md, 3).
use crate::{Component, Statement};
use alloc::boxed::Box;
use jig_protocol_worker_wire::{Agreed, Credential, Reason};
use skein_io::digest::{Digest, digest};
use skein_lib::{List, Reader, Token};
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Verdict {
    Accept { worker: u32, wires: List<Agreed>, statement: Statement },
    Refuse { worker: Option<u32>, reason: Reason, text: Box<[u8]> },
    OutputRoom { worker: u32, output: u32, text: Box<[u8]> },
    Replace { worker: u32, older: Token, wires: List<Agreed>, statement: Statement },
}
impl Component {
    pub(crate) fn judge(&self, credential: &[u8]) -> Verdict {
        let Ok(credential) = Credential::decode(&self.codec, &mut Reader::new(credential)) else {
            return Verdict::Refuse { worker: None, reason: Reason::Unauthorized, text: Box::new([]) };
        };
        let supplied = digest(credential.secret().expose());
        let mut found = None;
        for (index, worker) in self.workers.iter().enumerate() {
            if worker.name.as_ref() == credential.name() && same(&worker.digest, &supplied) {
                found = Some(u32::try_from(index).expect("bounded workers"));
            }
        }
        let Some(worker) = found else {
            return Verdict::Refuse { worker: None, reason: Reason::Unauthorized, text: Box::new([]) };
        };
        let statement = Statement::credential(&credential);
        let wires = match crate::wires::agree(&self.wires, &statement.wires) {
            Ok(wires) => wires,
            Err(text) => return Verdict::Refuse { worker: Some(worker), reason: Reason::Wires, text },
        };
        if let Err(text) = crate::statement::output(self.limits.room, statement.output) {
            return Verdict::OutputRoom { worker, output: statement.output, text };
        }
        let mut open = 0_u32;
        for (token, connection) in &self.connections {
            if connection.worker == Some(worker) {
                return Verdict::Replace { worker, older: *token, wires, statement };
            }
            if connection.worker.is_some() {
                open = open.saturating_add(1);
            }
        }
        if open >= self.limits.workers {
            Verdict::Refuse { worker: Some(worker), reason: Reason::Busy, text: Box::new([]) }
        } else {
            Verdict::Accept { worker, wires, statement }
        }
    }
}
/// The authenticated opening's unequal rooms (protocol/channel/README.md, 4.1).
pub(crate) fn output_room_words(engine: u32, worker: u32) -> Box<[u8]> {
    use skein_lib::{Decimal, Writer};
    let engine = Decimal::of(u64::from(engine));
    let worker = Decimal::of(u64::from(worker));
    let parts: [&[u8]; 5] = [
        b"engine io.output ",
        engine.as_bytes(),
        b" bytes differs from worker io.output ",
        worker.as_bytes(),
        b" bytes; change io.output to agree",
    ];
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded diagnostic");
    }
    let mut words = Writer::new(length);
    for part in parts {
        words.put(part).expect("measured diagnostic");
    }
    words.finish()
}
pub(crate) fn same(left: &Digest, right: &Digest) -> bool {
    let mut different = 0_u64;
    for index in 0..4 {
        let left = left.0.get(index).expect("four digest words");
        let right = right.0.get(index).expect("four digest words");
        different |= left ^ right;
    }
    different == 0
}
