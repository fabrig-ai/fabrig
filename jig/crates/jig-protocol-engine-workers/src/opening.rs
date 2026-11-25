//! Ordered authentication, wires and link-only room and capacity checks (protocol/channel/README.md, 4.1).
use crate::{
    Component, Statement,
    statement::{self, Fitted},
};
use alloc::boxed::Box;
use jig_protocol_worker_wire::Record;
use jig_protocol_worker_wire::{Credential, Opening, Reason};
use skein_lib::sha256;
use skein_lib::{Reader, Token};
pub(crate) enum Verdict {
    Accept { worker: u32, opening: Opening, older: Option<Token>, fitted: Box<Fitted>, statement: Statement },
    Refuse { worker: Option<u32>, reason: Reason, text: Box<[u8]> },
    OutputRoom { worker: u32, output: u32, text: Box<[u8]> },
}
impl Component {
    pub(crate) fn judge(&mut self, credential: &[u8], in_process: bool) -> Verdict {
        let Ok(credential) = Credential::decode(&self.codec, &mut Reader::new(credential)) else {
            return Verdict::Refuse { worker: None, reason: Reason::Unauthorized, text: Box::new([]) };
        };
        let supplied = sha256::digest(credential.secret.expose());
        let mut found = None;
        for worker in &self.workers {
            let authorized = match &worker.digest {
                Some(digest) => same(digest, &supplied),
                None => in_process,
            };
            if !worker.withdrawn && worker.name.as_ref() == credential.name.as_ref() && authorized {
                found = worker.number.checked_sub(1);
            }
        }
        let Some(worker) = found else {
            return Verdict::Refuse { worker: None, reason: Reason::NotAdmitted, text: Box::new([]) };
        };
        let opening = credential.opening.clone();
        let statement = Statement::credential(credential);
        let fitted = match statement::wires(&mut self.plans, &self.wires, &self.codec, &statement.wires) {
            Ok(fitted) => Box::new(fitted),
            Err(text) => return Verdict::Refuse { worker: Some(worker), reason: Reason::Wires, text },
        };
        if opening == Opening::Figures {
            return if self.figures.is_some() {
                Verdict::Accept { worker, opening, older: None, fitted, statement }
            } else {
                Verdict::Refuse { worker: Some(worker), reason: Reason::Starting, text: Box::new([]) }
            };
        }
        if let Err(text) = statement::output(self.limits.room, statement.output) {
            return Verdict::OutputRoom { worker, output: statement.output, text };
        }
        let mut open = 0_u32;
        for (token, connection) in &self.connections {
            if connection.worker == Some(worker) && connection.opening == Opening::Link {
                return Verdict::Accept { worker, opening, older: Some(*token), fitted, statement };
            }
            if connection.worker.is_some() && connection.opening == Opening::Link {
                open = open.saturating_add(1);
            }
        }
        if open >= self.limits.workers {
            Verdict::Refuse { worker: Some(worker), reason: Reason::Busy, text: Box::new([]) }
        } else {
            Verdict::Accept { worker, opening, older: None, fitted, statement }
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
pub(crate) fn same(left: &[u8; 32], right: &[u8; 32]) -> bool {
    let mut different = 0_u8;
    for (left, right) in left.iter().zip(right) {
        different |= left ^ right;
    }
    different == 0
}
