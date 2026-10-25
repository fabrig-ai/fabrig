//! Input is visible through the native client while the engine's store is held.
use crate::processes::Proc;
use jig_client::{Asker, Handle, contract as c};
use jig_client_protocol_world::native::Received;

const RUN: Handle = Handle { asker: Asker(79), number: 72 };
const HOLD: &[u8] = b"input-open";
#[derive(Default)]
pub(crate) struct Input {
    pub(crate) opened: bool,
    pub(crate) seen: bool,
    pub(crate) committed: bool,
}
pub(crate) fn arguments() -> Box<[u8]> {
    let mut bytes = br#"{"form":"report","text":"Hello","fields":{}}"#.to_vec();
    bytes.extend_from_slice(&[b' '; 32]);
    assert_eq!(bytes.len(), 76);
    bytes.into_boxed_slice()
}
pub(crate) fn turn() -> skein_fake_llm_domain::api::Turn {
    use skein_fake_llm_domain::api::{Finish, Line, Piece, Turn};
    Turn {
        lines: Box::new([
            Line::Pieces {
                pieces: Box::new([Piece { text: Box::from(b" ".as_slice()), hold: Some(Box::from(HOLD)) }]),
            },
            Line::Call { name: Box::from(b"finish".as_slice()), arguments: arguments() },
        ]),
        finish: Finish::ToolCalls,
        tokens: 1,
    }
}
impl Input {
    pub(crate) fn act(&mut self, procs: &mut [Proc], store: &crate::world::Store) {
        let client = procs
            .iter()
            .find_map(|proc| if let Proc::Client(client) = proc { Some(client) } else { None })
            .expect("native watcher");
        let state = client.received.iter().rev().find_map(|received| {
            if let Received::Delivery(c::Delivery::RunState(state)) = received {
                if state.task == 1 { Some(state) } else { None }
            } else {
                None
            }
        });
        if !self.opened && state.is_some() {
            self.opened = true;
            for proc in procs {
                if let Proc::Peer(peer) = proc {
                    peer.release(HOLD).expect("input hold");
                }
            }
            return;
        }
        if !self.seen {
            if let Some(text) = client.text(RUN)
                && let Some(writing) = &text.writing
            {
                assert_eq!(writing.tool.bytes(), b"finish");
                assert!(!writing.id.bytes().is_empty());
                assert_eq!(writing.input.bytes(), arguments().as_ref());
                assert_eq!((writing.from, writing.through, writing.cut), (0, 76, false));
                assert!(text.call.is_none());
                assert_eq!(state.expect("opened run").committed_turn, 0);
                assert!(crate::world::turns(store).is_empty(), "input is visible before its durable turn");
                self.seen = true;
            }
        } else if state.is_some_and(|state| state.committed_turn >= 1) {
            assert!(client.text(RUN).is_none(), "commit clears provisional input");
            self.committed = true;
        }
    }
    pub(crate) fn watch(procs: &mut [Proc]) {
        for proc in procs {
            if let Proc::Client(client) = proc {
                client.account_request(jig_client::Event::Watch {
                    handle: RUN,
                    subject: jig_client::Watchable::Jig(c::Subject::Run { task: 1, turn: None, offset: 0 }),
                });
            }
        }
    }
}
