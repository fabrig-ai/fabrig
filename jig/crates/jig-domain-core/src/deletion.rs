//! Kept project deletion (jig/docs/design/domain/people.md, 5.2).
use crate::{Core, Limits, Request, Requests};
use skein_lib::{Env, Queue, Token};

/// Kept deletions' run-time state.
#[derive(Debug)]
pub(crate) struct Deletions {}

impl Core {
    /// Whether a project takes requests (jig/docs/design/domain/people.md, 5.2, "Closed at once").
    #[must_use]
    pub fn open_project(&self, project: u32) -> bool {
        self.projects.as_slice().contains(&project)
    }
}

/// Continue a project's deletion once its tasks are erased whole.
pub(crate) fn tasks_erased(_core: &mut Core, _env: &Env<Limits>, _out: &mut Queue<Request>, _project: u32) {}

/// Continue a project's deletion once its notes are dropped.
pub(crate) fn notes_dropped(_core: &mut Core, _owner: Token) -> Requests {
    Requests::Out(Queue::uncapped())
}
