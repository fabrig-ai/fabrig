//! Project visibility (jig/docs/design/domain/people.md, 13.2).
use crate::Core;
use skein_lib::Map;

/// Which projects are public (jig/docs/design/domain/people.md, 13.2).
#[derive(Debug)]
pub(crate) struct Visibility {
    pub(crate) public: Map<u32, ()>,
}

impl Visibility {
    pub(crate) fn new(projects: u32) -> Self {
        Self { public: Map::with_capacity(projects) }
    }
}

impl Core {
    /// Public, open and not being deleted: the one test every public read and `me` makes.
    pub(crate) fn public_project(&self, project: u32) -> bool {
        self.open_project(project) && self.visibility.public.contains_key(&project)
    }
}
