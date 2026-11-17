//! A checkpoint's independent reader lock and removal by name (checkpoints.md, 6).
use super::{CheckpointName, files};
use crate::{Arrived, Below, CheckpointRefusal, Event, Root, Store, directory, generations};
use alloc::boxed::Box;
use skein_io::{file, kernel::Error};
use skein_lib::{List, Map, Queue, Token};
pub(crate) const OWNER: Token = Token::new(0x434b_4e4d);

#[derive(Debug)]
struct Ask {
    owner: Token,
    root: Root,
    name: CheckpointName,
    remove: bool,
}
#[derive(Debug)]
enum Action {
    Ask(Ask),
    Close { owner: Token, root: Root },
}
#[derive(Debug)]
enum Step {
    Parent,
    Checkpoint,
    Lock,
    Remove(generations::Removing),
    Sync,
    Drain,
}
#[derive(Debug)]
struct Operation {
    owner: Token,
    name: Option<CheckpointName>,
    remove: bool,
    parent: Option<Root>,
    checkpoint: Option<Root>,
    handles: List<Token>,
    step: Step,
    terminal: Option<Event>,
    failure: Option<CheckpointRefusal>,
}
#[derive(Debug)]
pub(crate) struct Named {
    asks: Queue<Action>,
    active: Option<Operation>,
    held: Map<Token, Root>,
}
impl Named {
    pub fn new() -> Self {
        Self { asks: Queue::uncapped(), active: None, held: Map::uncapped() }
    }
    fn start(&mut self, below: &mut Below) {
        if self.active.is_some() {
            return;
        }
        let Some(action) = self.asks.pop() else { return };
        self.active = Some(match action {
            Action::Ask(ask) => {
                below.checkpoints.push(file::Request::OpenDirectory {
                    owner: OWNER,
                    root: ask.root.fd,
                    name: Box::from(super::DIRECTORY),
                    no_follow: true,
                });
                Operation {
                    owner: ask.owner,
                    name: Some(ask.name),
                    remove: ask.remove,
                    parent: None,
                    checkpoint: None,
                    handles: List::with_capacity(2),
                    step: Step::Parent,
                    terminal: None,
                    failure: None,
                }
            }
            Action::Close { owner, root } => {
                let mut handles = List::with_capacity(2);
                handles.push(root.token).expect("reader handle");
                let mut op = Operation {
                    owner,
                    name: None,
                    remove: false,
                    parent: None,
                    checkpoint: None,
                    handles,
                    step: Step::Drain,
                    terminal: Some(Event::Closed { owner, removed: false }),
                    failure: None,
                };
                assert!(!drain(&mut op, below), "reader close has a handle");
                op
            }
        });
    }
}

pub(crate) fn ask(
    store: &mut Store,
    owner: Token,
    root: Root,
    name: CheckpointName,
    remove: bool,
    above: &mut Queue<Event>,
    below: &mut Below,
) {
    if !store.directory.disk {
        above.push(Event::CheckpointRefused { owner, why: CheckpointRefusal::Unavailable });
        return;
    }
    store.named_checkpoints.asks.push(Action::Ask(Ask { owner, root, name, remove }));
    store.named_checkpoints.start(below);
}
/// A held reader closes without affecting the serving store.
pub(crate) fn close(store: &mut Store, owner: Token, below: &mut Below) -> bool {
    let Some(root) = store.named_checkpoints.held.remove(&owner) else { return false };
    store.named_checkpoints.asks.push(Action::Close { owner, root });
    store.named_checkpoints.start(below);
    true
}

fn drain(op: &mut Operation, below: &mut Below) -> bool {
    let Some(index) = op.handles.len().checked_sub(1) else { return true };
    let file = op.handles.swap_remove(index).expect("last owned checkpoint handle");
    below.checkpoints.push(file::Request::Close { owner: OWNER, file });
    op.step = Step::Drain;
    false
}
pub(crate) fn up(store: &mut Store, arrived: Arrived, above: &mut Queue<Event>, below: &mut Below) {
    let mut op = store.named_checkpoints.active.take().expect("named checkpoint ask in flight");
    let path = match op.name {
        Some(name) => files::join(b"checkpoints/", &name.bytes()),
        None => Box::from(super::DIRECTORY),
    };
    let why = files::failure(&path, &arrived.event);
    let step = core::mem::replace(&mut op.step, Step::Drain);
    let result = match step {
        Step::Parent => opened(&mut op, arrived, true, below),
        Step::Checkpoint => opened(&mut op, arrived, false, below),
        Step::Lock => {
            if let file::Event::Busy { .. } = arrived.event {
                Err(CheckpointRefusal::InUse)
            } else if let file::Event::Locked { .. } = arrived.event {
                let root = op.checkpoint.expect("locked checkpoint root");
                if op.remove {
                    let removing = generations::remove_checkpoint_owned(
                        op.parent.expect("parent open"),
                        Box::from(op.name.expect("named checkpoint").bytes()),
                        OWNER,
                        &mut below.checkpoints,
                    );
                    op.step = Step::Remove(removing);
                } else if store.named_checkpoints.held.contains_key(&op.owner) {
                    op.terminal = Some(Event::CheckpointRefused { owner: op.owner, why: CheckpointRefusal::InUse });
                } else {
                    let index = op.handles.len().checked_sub(1).expect("locked handle retained");
                    op.handles.swap_remove(index).expect("checkpoint handle");
                    store.named_checkpoints.held.insert(op.owner, root).expect("held readers uncapped");
                    op.terminal = Some(Event::CheckpointHeld { owner: op.owner });
                }
                Ok(())
            } else {
                Err(why)
            }
        }
        Step::Remove(mut removing) => {
            if let file::Event::Failed { .. } = &arrived.event
                && op.failure.is_none()
            {
                op.failure = Some(why.clone());
            }
            match generations::removed(&mut removing, arrived, &mut below.checkpoints) {
                generations::Progress::Pending => {
                    op.step = Step::Remove(removing);
                    Ok(())
                }
                generations::Progress::Done => {
                    below.checkpoints.push(file::Request::SyncDirectory {
                        owner: OWNER,
                        root: op.parent.expect("removal parent").fd,
                    });
                    op.step = Step::Sync;
                    Ok(())
                }
                generations::Progress::Failed => Err(op.failure.take().unwrap_or(why)),
            }
        }
        Step::Sync => {
            if let file::Event::Synced { .. } = arrived.event {
                op.terminal = Some(Event::CheckpointRemoved { owner: op.owner });
                Ok(())
            } else {
                Err(why)
            }
        }
        Step::Drain => Ok(()),
    };
    if let Err(why) = result {
        op.terminal = Some(Event::CheckpointRefused { owner: op.owner, why });
    }
    if op.terminal.is_some() && drain(&mut op, below) {
        above.push(op.terminal.take().expect("named operation terminal"));
        store.named_checkpoints.start(below);
    } else {
        store.named_checkpoints.active = Some(op);
    }
}
fn opened(op: &mut Operation, arrived: Arrived, parent: bool, below: &mut Below) -> Result<(), CheckpointRefusal> {
    if let file::Event::Failed { error: Error::NotFound, .. } = arrived.event {
        return Err(CheckpointRefusal::Missing);
    }
    let why =
        files::failure(&files::join(b"checkpoints/", &op.name.expect("named checkpoint").bytes()), &arrived.event);
    let root = directory::opened(arrived).ok_or(why)?;
    op.handles.push(root.token).expect("parent and checkpoint handle");
    if parent {
        op.parent = Some(root);
        below.checkpoints.push(file::Request::OpenDirectory {
            owner: OWNER,
            root: root.fd,
            name: Box::from(op.name.expect("named checkpoint").bytes()),
            no_follow: true,
        });
        op.step = Step::Checkpoint;
    } else {
        op.checkpoint = Some(root);
        below.checkpoints.push(file::Request::Lock { owner: OWNER, root: root.token });
        op.step = Step::Lock;
    }
    Ok(())
}
