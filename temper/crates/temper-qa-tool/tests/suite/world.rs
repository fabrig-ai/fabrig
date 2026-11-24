//! The shared QA world over scratch files and real program fallback (temper/docs/design/testing/qa/README.md, 6).
use skein_scratch::Scratch;
use std::ops::{Deref, DerefMut};
use temper_qa_tool::{faces::RealFiles, testing};

pub use testing::{Agent, Clock, id, land};
pub type Git = testing::Git<RealFiles>;
pub type Loop = testing::Loop<RealFiles>;

pub struct World {
    inner: testing::World<RealFiles>,
    _scratch: Scratch,
}

impl World {
    pub fn new() -> Self {
        let scratch = Scratch::new("qa-run");
        let inner = testing::World::on_disk(scratch.path());
        Self { inner, _scratch: scratch }
    }
}

impl Deref for World {
    type Target = testing::World<RealFiles>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for World {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub fn baseline() -> Git {
    testing::baseline_with(RealFiles)
}
