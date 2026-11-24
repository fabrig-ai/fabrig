//! Observe the simulated hook listener's accepted streams and completed writes
//! (temper/docs/design/protocol/forge/README.md, 4).
use skein_io::kernel::{Complete, Done, Fd, Op};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct HookReceipt {
    pub accepted: u32,
    /// Each HTTP status and the accept ordinal it was written to.
    pub responses: Vec<(u32, u16)>,
    pub signed_push: bool,
}

#[derive(Default)]
pub(super) struct Hooks {
    streams: BTreeMap<Fd, (u32, Vec<u8>)>,
    pub(super) receipt: HookReceipt,
}
impl Hooks {
    pub(super) fn complete(&mut self, listener: Fd, complete: &Complete) {
        match (&complete.kind, &complete.result) {
            (Op::Accept { fd: at }, Ok(Done::Accepted { fd, .. })) if *at == listener => {
                self.receipt.accepted += 1;
                assert!(self.streams.insert(*fd, (self.receipt.accepted, Vec::new())).is_none());
            }
            (Op::Send { fd, bytes, from }, Ok(Done::Count(count))) => {
                if let Some((connection, pending)) = self.streams.get_mut(fd) {
                    pending.extend_from_slice(&bytes[*from as usize..(*from + *count) as usize]);
                    // Hook answers are bodyless. Count complete heads even if a send
                    // ends mid-head or carries more than one.
                    while let Some(end) = skein_lib::bytes::find(pending, b"\r\n\r\n") {
                        let head = pending.drain(..end + 4).collect::<Vec<_>>();
                        let status = std::str::from_utf8(&head)
                            .expect("HTTP head")
                            .strip_prefix("HTTP/1.1 ")
                            .expect("HTTP response")
                            .split_whitespace()
                            .next()
                            .expect("HTTP status")
                            .parse()
                            .expect("numeric HTTP status");
                        self.receipt.responses.push((*connection, status));
                    }
                    assert!(pending.len() <= 4096, "bounded hook response head");
                }
            }
            (Op::Close { fd }, Ok(Done::Nothing)) => {
                self.streams.remove(fd);
            }
            _ => {}
        }
    }
}
