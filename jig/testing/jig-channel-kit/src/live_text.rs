//! The process reply's prefixes precede its turn (protocol/channel/README.md, 11).
use crate::processes::Proc;
use jig_client::{Asker, Handle, contract as c};
use jig_client_protocol_kit::native::Received;

pub(crate) const RUN: Handle = Handle { asker: Asker(78), number: 71 };
const TASK: u64 = 1;
const TURN: u32 = 1;
// Three literal prefixes of the provider's reply, each observed before release.
pub(crate) const PIECES: usize = 3;
const PREFIXES: [&[u8]; PIECES] = [b"Hello", b"Hello from", b"Hello from worker"];
const HOLDS: [&[u8]; PIECES] = [b"watch-open", b"second-piece", b"third-piece"];

#[derive(Default)]
pub(crate) struct Reply {
    opened: bool,
    pub(crate) prefixes: Vec<Box<[u8]>>,
    pub(crate) committed: bool,
}

pub(crate) fn line() -> skein_fake_llm_domain::api::Line {
    use skein_fake_llm_domain::api::{Line, Piece};
    Line::Pieces {
        pieces: Box::new([
            Piece { text: b"Hello".as_slice().into(), hold: Some(HOLDS[0].into()) },
            Piece { text: b" from".as_slice().into(), hold: Some(HOLDS[1].into()) },
            Piece { text: b" worker".as_slice().into(), hold: Some(HOLDS[2].into()) },
        ]),
    }
}

impl Reply {
    pub(crate) fn act(&mut self, procs: &mut [Proc], store: &crate::world::Store) -> Result<(), String> {
        let client =
            procs.iter().find_map(|p| if let Proc::Client(c) = p { Some(c) } else { None }).ok_or("native watcher")?;
        let state = client.received.iter().rev().find_map(|r| {
            if let Received::Delivery(c::Delivery::RunState(state)) = r
                && state.task == TASK
            {
                Some(state)
            } else {
                None
            }
        });
        let mut release = None;
        if !self.opened && state.is_some() {
            self.opened = true;
            release = Some(HOLDS[0]);
        } else if self.prefixes.len() < PREFIXES.len() {
            if let Some(text) = client.text(RUN) {
                if self.prefixes.last().is_some_and(|previous| previous.as_ref() == text.text.bytes()) {
                    return Ok(());
                }
                {
                    let (observed_value, wanted) = (&(text.text.bytes()), &(PREFIXES[self.prefixes.len()]));
                    if observed_value != wanted {
                        return Err(format!(
                            "prefix before next release; assertion `left == right` failed: left: {observed_value:?}, right: {wanted:?}"
                        ));
                    }
                }
                {
                    let (observed_value, wanted) =
                        (&(text.turn.number, text.turn.retry, text.from, text.cut), &(TURN, 0, 0, false));
                    if observed_value != wanted {
                        return Err(format!(
                            "assertion `left == right` failed: left: {observed_value:?}, right: {wanted:?}"
                        ));
                    }
                }
                {
                    let (observed_value, wanted) = (&(text.through as usize), &(text.text.bytes().len()));
                    if observed_value != wanted {
                        return Err(format!(
                            "assertion `left == right` failed: left: {observed_value:?}, right: {wanted:?}"
                        ));
                    }
                }
                {
                    let (observed_value, wanted) = (&(text.attempt), &(text.turn.attempt));
                    if observed_value != wanted {
                        return Err(format!(
                            "assertion `left == right` failed: left: {observed_value:?}, right: {wanted:?}"
                        ));
                    }
                }
                {
                    let (observed_value, wanted) = (&(state.ok_or("opened run watch")?.committed_turn), &(0));
                    if observed_value != wanted {
                        return Err(format!(
                            "assertion `left == right` failed: left: {observed_value:?}, right: {wanted:?}"
                        ));
                    }
                }
                if text.call.is_some() {
                    return Err(String::from("assertion failed: text.call.is_none()"));
                }
                if !crate::world::turns(store).is_empty() {
                    return Err(String::from("each prefix precedes the durable turn"));
                }
                self.prefixes.push(text.text.bytes().into());
                release = HOLDS.get(self.prefixes.len()).copied();
            }
        } else if state.is_some_and(|state| state.committed_turn >= TURN) {
            if state.ok_or("committed state")?.writing.is_some() {
                return Err(String::from("committed reply has no writing"));
            }
            if client.text(RUN).is_some() {
                return Err(String::from("committed turn clears retained provisional text"));
            }
            self.committed = true;
        }
        if let Some(hold) = release {
            for proc in procs {
                if let Proc::Peer(peer) = proc {
                    peer.release(hold).map_err(|why| format!("named provider hold: {why:?}"))?;
                }
            }
        }
        Ok(())
    }
}
