//! Checkpoint temporaries, independently of the opening lane (protocol/store/checkpoints.md, 7).
use super::{CheckpointName, DIRECTORY};
use crate::{
    Arrived, Root,
    directory::OWNER,
    generations::{self, PAGE, Progress, Removing},
};
use alloc::boxed::Box;
use skein_io::{
    file,
    kernel::{Error, Kind},
};
use skein_lib::Queue;

#[derive(Debug)]
enum Step {
    Opening,
    Listing,
    File,
    Tree(Removing),
    Closing { restart: bool },
    Syncing,
    Finished,
}
/// One page and one bounded removal tree, with one file request in flight.
#[derive(Debug)]
pub(crate) struct Sweep {
    deployment: Root,
    root: Option<Root>,
    entries: Box<[file::Entry]>,
    next: usize,
    dirty: bool,
    page_dirty: bool,
    step: Step,
}
/// Open the deployment's checkpoints directory; absence is an empty sweep.
pub(crate) fn begin(deployment: Root, queue: &mut Queue<file::Request>) -> Sweep {
    let mut sweep = Sweep {
        deployment,
        root: None,
        entries: Box::new([]),
        next: 0,
        dirty: false,
        page_dirty: false,
        step: Step::Opening,
    };
    open(&mut sweep, queue);
    sweep
}
fn open(sweep: &mut Sweep, queue: &mut Queue<file::Request>) {
    queue.push(file::Request::OpenDirectory {
        owner: OWNER,
        root: sweep.deployment.fd,
        name: Box::from(DIRECTORY),
        no_follow: true,
    });
    sweep.step = Step::Opening;
}
fn close(sweep: &mut Sweep, restart: bool, queue: &mut Queue<file::Request>) {
    let root = sweep.root.take().expect("sweep directory open");
    queue.push(file::Request::Close { owner: OWNER, file: root.token });
    sweep.step = Step::Closing { restart };
}
fn next(sweep: &mut Sweep, queue: &mut Queue<file::Request>) -> Progress {
    let root = sweep.root.expect("listed checkpoint directory open");
    for _ in 0..PAGE {
        let Some(entry) = sweep.entries.get(sweep.next) else {
            break;
        };
        sweep.next = sweep.next.checked_add(1).expect("bounded page index");
        if CheckpointName::parse_temporary(&entry.name).is_none() {
            continue;
        }
        sweep.dirty = true;
        sweep.page_dirty = true;
        if entry.kind == Kind::Directory {
            let removing = generations::remove_checkpoint(root, entry.name.clone(), queue);
            sweep.step = Step::Tree(removing);
        } else {
            queue.push(file::Request::Remove {
                owner: OWNER,
                root: root.fd,
                name: entry.name.clone(),
                directory: false,
            });
            sweep.step = Step::File;
        }
        return Progress::Pending;
    }
    if sweep.page_dirty {
        close(sweep, true, queue);
    } else if !sweep.entries.is_empty() {
        sweep.entries = Box::new([]);
        queue.push(file::Request::ListPage { owner: OWNER, directory: root.token, max: PAGE });
        sweep.step = Step::Listing;
    } else if sweep.dirty {
        queue.push(file::Request::SyncDirectory { owner: OWNER, root: root.fd });
        sweep.step = Step::Syncing;
    } else {
        close(sweep, false, queue);
    }
    Progress::Pending
}
pub(crate) fn arrived(sweep: &mut Sweep, arrived: Arrived, queue: &mut Queue<file::Request>) -> Progress {
    match core::mem::replace(&mut sweep.step, Step::Finished) {
        Step::Opening => {
            if let file::Event::Failed { error: Error::NotFound, .. } = arrived.event {
                return Progress::Done;
            }
            let Some(root) = crate::directory::opened(arrived) else {
                return Progress::Failed;
            };
            sweep.root = Some(root);
            queue.push(file::Request::ListPage { owner: OWNER, directory: root.token, max: PAGE });
            sweep.step = Step::Listing;
        }
        Step::Listing => {
            let file::Event::Listed { entries, .. } = arrived.event else {
                return Progress::Failed;
            };
            sweep.entries = entries;
            sweep.next = 0;
            sweep.page_dirty = false;
            return next(sweep, queue);
        }
        Step::File => {
            let file::Event::Removed { .. } = arrived.event else {
                return Progress::Failed;
            };
            return next(sweep, queue);
        }
        Step::Tree(mut removing) => match generations::removed(&mut removing, arrived, queue) {
            Progress::Pending => sweep.step = Step::Tree(removing),
            Progress::Done => return next(sweep, queue),
            Progress::Failed => return Progress::Failed,
        },
        Step::Syncing => {
            let file::Event::Synced { .. } = arrived.event else {
                return Progress::Failed;
            };
            close(sweep, false, queue);
        }
        Step::Closing { restart } => {
            let file::Event::Closed { .. } = arrived.event else {
                return Progress::Failed;
            };
            if restart {
                open(sweep, queue);
            } else {
                return Progress::Done;
            }
        }
        Step::Finished => return Progress::Failed,
    }
    Progress::Pending
}
