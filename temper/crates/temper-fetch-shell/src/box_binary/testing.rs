use super::{Candidate, Event, Failed, Path, PathBuf, Request};

#[must_use]
pub fn absent() -> Candidate {
    super::absent()
}

pub fn cached(file: &Path) -> Result<Event, Failed> {
    super::cached(file)
}

#[must_use]
pub fn identity(file: &Path) -> Candidate {
    super::identity(file)
}

#[must_use]
pub fn suffix(file: &Path, name: &str) -> PathBuf {
    super::suffix(file, name)
}

#[derive(Default)]
pub struct Files(super::Files);

impl Files {
    pub fn answer(&mut self, request: Request) -> Result<Event, Failed> {
        self.0.answer(request)
    }

    #[must_use]
    pub fn is_held(&self) -> bool {
        self.0.held.is_some()
    }
}
