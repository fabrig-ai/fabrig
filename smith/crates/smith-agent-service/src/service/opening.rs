//! A Start retains its attached directories until its result contract and delivery
//! grant are known.
//! No contract is known while idle or attaching. `attach`, `attached` and `take`
//! move the Start; `follow` enters it after every mount, and `contract` serves effects.
//! Contract: smith/docs/design/protocol/agent.md, sections 4–6;
//! smith/docs/design/domain/run.md, sections 3.1 and 8.4.

use super::{Frame, Stages};
use alloc::boxed::Box;
use core::mem;
use skein_lib::{List, Token};
use smith_domain::run;
use smith_protocol_translate::DecodedStart;

/// The Start waiting on mounts, or the result contract of the started run.
#[derive(Debug)]
pub(super) enum Opening {
    /// No Start waits, and no contract is known.
    Idle,
    /// A Start waits for its mounts, with attached directories in Start order.
    Attaching { start: Box<DecodedStart>, directories: List<run::Directory> },
    /// The run's result contract and independent delivery grant are known.
    Started { outcome: run::outcome::OutcomeSpec, deliver: Option<run::outcome::ChangeSpec> },
}

impl Opening {
    pub(super) fn attach(&mut self, start: Box<DecodedStart>, roots: u32) {
        let state = mem::replace(self, Self::Idle);
        *self = match state {
            Self::Idle => Self::Attaching { start, directories: List::with_capacity(roots) },
            Self::Attaching { .. } | Self::Started { .. } => unreachable!("one Start per channel"),
        };
    }

    pub(super) fn next_mount(&self) -> Option<&[u8]> {
        match self {
            Self::Idle | Self::Started { .. } => None,
            Self::Attaching { start, directories } => {
                let mounts = start.mounts.as_ref()?;
                let index = usize::try_from(directories.len()).expect("root index fits usize");
                match mounts.get(index) {
                    Some(mount) => Some(&mount.path),
                    None => None,
                }
            }
        }
    }

    pub(super) fn attached(&mut self, host: Box<[u8]>, root: Token) {
        match self {
            Self::Attaching { start, directories } => {
                let mounts = start.mounts.as_ref().expect("a requested root belongs to a workspace");
                let index = usize::try_from(directories.len()).expect("root index fits usize");
                let mount = mounts.get(index).expect("one mount requested at a time");
                directories
                    .push(run::Directory {
                        host,
                        name: mount.name.clone(),
                        root,
                        writable: mount.writable,
                        git: mount.git,
                        conflicts: mount.conflicts.clone(),
                    })
                    .expect("root count was checked before opening");
            }
            Self::Idle | Self::Started { .. } => unreachable!("a root belongs to a pending Start"),
        }
    }

    pub(super) fn take(&mut self) -> (Box<DecodedStart>, List<run::Directory>) {
        let state = mem::replace(self, Self::Idle);
        match state {
            Self::Attaching { start, directories } => (start, directories),
            Self::Idle | Self::Started { .. } => unreachable!("all roots opened for pending Start"),
        }
    }

    pub(super) fn refuse(&mut self) {
        *self = Self::Idle;
    }

    pub(super) fn contract(&mut self, outcome: run::outcome::OutcomeSpec, deliver: Option<run::outcome::ChangeSpec>) {
        let state = mem::replace(self, Self::Idle);
        *self = match state {
            Self::Idle | Self::Started { .. } => Self::Started { outcome, deliver },
            Self::Attaching { .. } => unreachable!("effects hold no Start"),
        };
    }

    pub(super) fn outcome(&self) -> &run::outcome::OutcomeSpec {
        match self {
            Self::Started { outcome, .. } => outcome,
            Self::Idle | Self::Attaching { .. } => unreachable!("an admitted Start retains its result contract"),
        }
    }

    pub(super) fn deliver(&self) -> Option<&run::outcome::ChangeSpec> {
        match self {
            Self::Started { deliver, .. } => deliver.as_ref(),
            Self::Idle | Self::Attaching { .. } => unreachable!("an admitted Start retains its delivery grant"),
        }
    }
}

/// A Start with every mount attached enters the domain.
pub(super) fn follow(service: &mut Stages, frame: &mut Frame) {
    match &service.opening {
        Opening::Attaching { .. } => {
            if service.opening.next_mount().is_none() {
                service.start_ready(frame);
            }
        }
        Opening::Idle | Opening::Started { .. } => {}
    }
}
