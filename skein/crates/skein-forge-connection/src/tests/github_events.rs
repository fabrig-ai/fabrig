//! Typed completion events for the component's GitHub step tests.
use super::super::*;
use crate::{
    Component,
    component::{Dialect, entry::ForgeEntry},
};
use alloc::boxed::Box;

impl Component {
    fn test_progress(&mut self, progress: Progress, env: &Env<Limits>, up: &mut Queue<Event>) {
        let ForgeEntry { base, dialect } = self.forges.get_mut(0).expect("forge");
        let Dialect::GitHub(gh) = dialect else { unreachable!("GitHub fixture") };
        self.common.github_progress(base, gh, progress, env, up);
    }
    pub(crate) fn test_minted(
        &mut self,
        answer: Result<document::Minted, api::Error>,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
    ) {
        self.test_progress(Progress::Minted(answer), env, up);
    }
    pub(crate) fn test_paged(
        &mut self,
        rows: Box<[document::Installation]>,
        next: bool,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
    ) {
        self.test_progress(Progress::Paged(Ok(InstallationPage { rows, next })), env, up);
    }
    pub(crate) fn test_identified(&mut self, viewer: document::Viewer, env: &Env<Limits>, up: &mut Queue<Event>) {
        self.test_progress(Progress::Identified(Ok(viewer)), env, up);
    }
    pub(crate) fn test_local_step(&mut self, env: &Env<Limits>, up: &mut Queue<Event>) {
        let ForgeEntry { base, dialect } = self.forges.get_mut(0).expect("forge");
        let Dialect::GitHub(gh) = dialect else { unreachable!("GitHub fixture") };
        // The bounded installation table and its pending outputs end this drain.
        while self.common.github_local(base, gh, env, up) {}
    }
}
