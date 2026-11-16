//! Named refs are resolved by the server's store before reading their commit (skein/docs/design/testing/fakes/forge.md, 4).
use crate::forgejo::{
    id::ObjectId,
    request::{Page, Repo, Request},
};
use alloc::boxed::Box;

#[derive(Debug)]
pub enum ReferenceRead {
    Contents { path: Box<[u8]> },
    CombinedStatus { page: Page },
}
#[derive(Debug)]
pub struct Reference {
    pub at: Repo,
    pub name: Box<[u8]>,
    pub read: ReferenceRead,
}
impl Reference {
    #[must_use]
    pub fn lookup(&self) -> Request {
        Request::Branch { at: self.at.clone(), name: self.name.clone() }
    }
    #[must_use]
    pub fn resolve(self, commit: ObjectId) -> Request {
        match self.read {
            ReferenceRead::Contents { path } => Request::Contents { at: self.at, path, reference: commit },
            ReferenceRead::CombinedStatus { page } => Request::CombinedStatus { at: self.at, commit, page },
        }
    }
}
