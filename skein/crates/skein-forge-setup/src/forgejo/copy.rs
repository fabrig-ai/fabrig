//! Copies a public repository with the shared creation steps (skein/docs/design/forge/setup/forgejo.md, 1.10).
use crate::{
    create::{self, Create},
    given::{Given, Refused},
    host::File,
    reconcile::{Event, Need, Why},
};
use alloc::boxed::Box;
use skein_lib::Queue;

/// One copy, holding its administrator only until the run ends.
#[derive(Debug)]
pub struct Copy {
    machine: Create,
}
impl Copy {
    /// The host has checked the plan against its admitted bounds before constructing it.
    pub fn new(
        given: &Given,
        org: Box<[u8]>,
        repository: Box<[u8]>,
        from: Box<[u8]>,
        files: Box<[File]>,
    ) -> Result<Self, Refused> {
        Ok(Self { machine: Create::copying(given, org, repository, from, files)? })
    }
}
/// One event in, at most one need out; an ended run ignores later events.
pub fn step(machine: &mut Copy, event: Event, out: &mut Queue<Need>) {
    create::step(&mut machine.machine, event, out);
}
/// Stops at the outstanding stage, releasing every held credential.
pub fn halt(machine: &mut Copy, why: Why, out: &mut Queue<Need>) {
    create::halt(&mut machine.machine, why, out);
}
