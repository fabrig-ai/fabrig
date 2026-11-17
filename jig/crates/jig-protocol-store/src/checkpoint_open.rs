//! A checkpoint-only open drains its owned file handles on Close (checkpoints.md, 5).
use crate::{Below, Event, Io, Root, Store, file_handles::Owned};
use skein_lib::{Queue, Token};

#[derive(Debug)]
pub(crate) struct State {
    files: Owned,
    root: Option<Token>,
    close: Option<Token>,
}
impl State {
    pub fn new(root: Root) -> Self {
        Self { files: Owned::new(), root: Some(root.token), close: None }
    }
}
pub(crate) fn close(store: &mut Store, owner: Token, above: &mut Queue<Event>) {
    let state = store.checkpoint_open.as_mut().expect("checkpoint open close");
    state.close = Some(owner);
    store.phase = crate::Phase::Stopped;
    crate::checkpoints::taking::cancel(store, false, above);
}
pub(crate) fn closing(store: &Store) -> bool {
    match &store.checkpoint_open {
        Some(state) => state.close.is_some(),
        None => false,
    }
}
pub(crate) fn arriving(store: &mut Store, io: &Io) {
    if let Some(state) = &mut store.checkpoint_open {
        state.files.arriving(io);
    }
}
pub(crate) fn issued(store: &mut Store, below: &Below, lengths: [u32; 4]) {
    if let Some(state) = &mut store.checkpoint_open {
        state.files.issued(below, lengths);
    }
}
pub(crate) fn advance(store: &mut Store, above: &mut Queue<Event>, below: &mut Below) {
    let Some(state) = &mut store.checkpoint_open else { return };
    let Some(owner) = state.close else { return };
    if state.files.drain(below) {
        if let Some(file) = state.root.take() {
            below.store.push(skein_io::file::Request::Close { owner: crate::directory::OWNER, file });
        } else {
            store.checkpoint_open = None;
            above.push(Event::Closed { owner, removed: false });
        }
    }
}
