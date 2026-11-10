//! Ordered authentication, payload and capacity checks (protocol/channel.md, 4.1).
use crate::Component;
use alloc::boxed::Box;
use jig_protocol_worker_wire::{Credential, Reason};
use skein_io::digest::{Digest, digest};
use skein_lib::{Reader, Token};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Verdict {
    Accept { worker: u32 },
    Refuse(Reason),
    OutputRoom { worker: u32, output: u32 },
    Replace { worker: u32, older: Token },
}
impl Component {
    pub(crate) fn judge(&self, credential: &[u8]) -> Verdict {
        let Ok(credential) = Credential::decode(&self.codec, &mut Reader::new(credential)) else {
            return Verdict::Refuse(Reason::Unauthorized);
        };
        let supplied = digest(credential.secret().expose());
        let mut found = None;
        for (index, worker) in self.workers.iter().enumerate() {
            if worker.name.as_ref() == credential.name() && same(&worker.digest, &supplied) {
                found = Some(u32::try_from(index).expect("bounded workers"));
            }
        }
        let Some(worker) = found else { return Verdict::Refuse(Reason::Unauthorized) };
        if credential.payload() != self.payload {
            return Verdict::Refuse(Reason::Application);
        }
        if credential.output() != self.limits.room {
            return Verdict::OutputRoom { worker, output: credential.output() };
        }
        let mut open = 0_u32;
        for (token, connection) in &self.connections {
            if connection.worker == Some(worker) {
                return Verdict::Replace { worker, older: *token };
            }
            if connection.worker.is_some() {
                open = open.saturating_add(1);
            }
        }
        if open >= self.limits.workers { Verdict::Refuse(Reason::Busy) } else { Verdict::Accept { worker } }
    }
}
/// The authenticated opening's unequal rooms (protocol/channel.md, 4.1).
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
