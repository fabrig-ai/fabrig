//! Git file steps and their settings for tests (temper/docs/design/protocol/git.md, 6).
use crate::{Limits, Settings, files};
use alloc::boxed::Box;
use skein_io::kernel;

pub mod fixtures;
pub use files::Action;

#[derive(Debug)]
pub struct Directories(files::Directories);

impl Directories {
    #[must_use]
    pub fn new(serial: u64) -> Self {
        Self(files::Directories::new(serial))
    }
}

#[derive(Debug)]
pub struct Task(files::Task);

impl Task {
    #[must_use]
    pub fn reds(directory: Box<[u8]>, content: Option<Box<[u8]>>, marker: bool, rerun: Option<u32>) -> Self {
        Self(files::Task::reds(directory, content, marker, rerun))
    }

    #[must_use]
    pub fn results(directory: Box<[u8]>) -> Self {
        Self(files::Task::results(directory))
    }

    pub fn first(&mut self, dirs: &mut Directories, settings: &Settings, limits: &Limits) -> Action {
        self.0.first(&mut dirs.0, settings, limits)
    }

    pub fn complete(
        &mut self,
        complete: kernel::Complete,
        dirs: &mut Directories,
        settings: &Settings,
        limits: &Limits,
        stop: bool,
    ) -> Action {
        self.0.complete(complete, &mut dirs.0, settings, limits, stop)
    }
}

#[must_use]
pub fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    crate::invocation::joined(parts)
}
