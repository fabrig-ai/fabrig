//! A staged generation is removed under its opened parent, as a stray is (restoring.md, 4.4 and 4.8).
use crate::{
    Arrived, Root,
    generations::{self, Progress, Removing},
};
use alloc::boxed::Box;
use skein_io::{file, kernel::Error};
use skein_lib::{Decimal, Queue, Token};
#[derive(Debug)]
enum Step {
    Parent,
    Removing(Removing),
    Close(Progress),
    Finished,
}
#[derive(Debug)]
pub(super) struct Generation {
    number: u64,
    owner: Token,
    parent: Option<Root>,
    step: Step,
    error: Option<Error>,
}
impl Generation {
    pub fn begin(root: Root, number: u64, owner: Token, queue: &mut Queue<file::Request>) -> Self {
        queue.push(file::Request::OpenDirectory {
            owner,
            root: root.fd,
            name: Box::from(generations::GENERATIONS),
            no_follow: true,
        });
        Self { number, owner, parent: None, step: Step::Parent, error: None }
    }
    pub fn error(&self) -> Option<Error> {
        self.error
    }
    pub fn arrived(&mut self, arrived: Arrived, queue: &mut Queue<file::Request>) -> Progress {
        if let Some(error) = super::error(&arrived.event)
            && error != Error::NotFound
            && self.error.is_none()
        {
            self.error = Some(error);
        }
        match core::mem::replace(&mut self.step, Step::Finished) {
            Step::Parent => {
                if let file::Event::Failed { error: Error::NotFound, .. } = arrived.event {
                    return Progress::Done;
                }
                let Some(parent) = crate::directory::opened(arrived) else {
                    return Progress::Failed;
                };
                self.parent = Some(parent);
                self.step = Step::Removing(generations::remove_checkpoint_owned(
                    parent,
                    Box::from(Decimal::of(self.number).as_bytes()),
                    self.owner,
                    queue,
                ));
            }
            Step::Removing(mut removing) => match generations::removed(&mut removing, arrived, queue) {
                Progress::Pending => self.step = Step::Removing(removing),
                progress @ (Progress::Done | Progress::Failed) => {
                    queue.push(file::Request::Close {
                        owner: self.owner,
                        file: self.parent.take().expect("generation parent open").token,
                    });
                    self.step = Step::Close(progress);
                }
            },
            Step::Close(progress) => {
                let file::Event::Closed { .. } = arrived.event else {
                    return Progress::Failed;
                };
                return progress;
            }
            Step::Finished => return Progress::Failed,
        }
        Progress::Pending
    }
}
