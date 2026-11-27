//! What a run finds in its workspace before it opens its main conversation
//! (smith/docs/design/domain/run-start.md, section 1.3, and smith/docs/design/domain/run-delivery.md, section 1.1). The engine never reads a directory, so the
//! run looks for itself:
//!
//! - For each directory, the start of its selected guide, which the system text
//!   carries after the brief.
//! - For each writable directory, its two hooks where the selected hooks directory
//!   exists, or whether the legacy check path names an executable.
//!
//! One operation at a time, in the workspace's order, each with its deadline.
//! An unreadable guide or legacy check is absent; an unreadable hook is broken.

use alloc::boxed::Box;

use skein_lib::bytes::copy_of;
use skein_lib::{List, Time, Token};

use crate::boundary::{Hook, Hooks, Place, Read, Request};
use crate::charter::{Charter, count};
use crate::conventions;
use crate::limits::Limits;
use crate::workspace::{self, Directory, Workspace};

/// One thing a run looks for: in the directory at `directory` in its
/// workspace, its guide or its checks.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Step {
    pub(crate) directory: u32,
    pub(crate) look: Look,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Look {
    Guide,
    Hooks,
    Checks,
}

/// What a run found in its workspace.
#[derive(Debug)]
pub(crate) struct Found {
    /// The guides found, in the workspace's order.
    pub(crate) guides: List<Guide>,
    /// The repositories that have checks, by their place in the workspace, in
    /// its order.
    pub(crate) checks: List<Check>,
}

/// Checks found in one writable directory.
#[derive(Debug)]
pub(crate) struct Check {
    pub(crate) directory: u32,
    pub(crate) kind: Kind,
}

#[derive(Debug)]
pub(crate) enum Kind {
    Legacy,
    Hooks(Hooks),
}

/// The start of a directory's selected guide.
#[derive(Debug)]
pub(crate) struct Guide {
    /// The directory's place in the workspace.
    pub(crate) directory: u32,
    pub(crate) text: Box<[u8]>,
    /// Whether `text` is all of the file.
    pub(crate) whole: bool,
}

impl Found {
    /// Room for what a workspace of `repositories` may hold.
    pub(crate) fn with_capacity(repositories: u32) -> Found {
        Found { guides: List::with_capacity(repositories), checks: List::with_capacity(repositories) }
    }
}

/// The first step, or the step after `after`, or `None` once there is
/// nothing left to look for.
pub(crate) fn next(charter: &Charter, mounted: Option<&Workspace>, after: Option<Step>) -> Option<Step> {
    let repositories = count(workspace::directories(mounted).len());
    let step = match after {
        None => Step { directory: 0, look: Look::Guide },
        Some(Step { directory, look: Look::Guide }) if wants_checks(mounted, directory) => {
            Step { directory, look: if conventions::hooks(charter).is_some() { Look::Hooks } else { Look::Checks } }
        }
        Some(Step { directory, look: Look::Guide | Look::Hooks | Look::Checks }) => {
            Step { directory: directory.saturating_add(1), look: Look::Guide }
        }
    };
    (step.directory < repositories).then_some(step)
}

/// The request that takes `step`, for the run `owner`.
pub(crate) fn request(
    charter: &Charter,
    mounted: Option<&Workspace>,
    step: Step,
    owner: Token,
    now: Time,
    limits: &Limits,
    guide_bytes: u32,
) -> Request {
    let root = directory(mounted, step.directory).root;
    let deadline = now.saturating_add(limits.io_timeout);
    match step.look {
        Look::Guide => {
            let at = Place { root, path: copy_of(conventions::guide(charter)) };
            Request::Read { owner, at, max: guide_bytes, deadline }
        }
        Look::Hooks => {
            let at = Place { root, path: copy_of(conventions::hooks(charter).expect("hooks directory selected")) };
            Request::ProbeHooks { owner, at, deadline }
        }
        Look::Checks => {
            Request::Probe { owner, at: Place { root, path: copy_of(conventions::checks(charter)) }, deadline }
        }
    }
}

/// Keeps what reading the guide of `step` found.
pub(crate) fn guide(found: &mut Found, step: Step, read: Read, guide_bytes: u32) {
    assert!(step.look == Look::Guide, "a read answers a guide's step");
    match read {
        Read::Text { text, whole } => {
            assert!(count(text.len()) <= guide_bytes, "io reads no more than it is asked for");
            let guide = Guide { directory: step.directory, text, whole };
            found.guides.push(guide).expect("room for a guide per directory");
        }
        Read::Missing | Read::NotText | Read::Failed => {}
    }
}

/// Keeps what looking for the checks of `step` found.
pub(crate) fn checks(found: &mut Found, step: Step, executable: bool) {
    assert!(step.look == Look::Checks, "a probe answers a checks' step");
    if executable {
        found
            .checks
            .push(Check { directory: step.directory, kind: Kind::Legacy })
            .expect("room for checks per directory");
    }
}

/// Keeps the per-moment checks found, including broken hooks.
pub(crate) fn hooks(found: &mut Found, step: Step, hooks: Hooks) {
    assert!(step.look == Look::Hooks, "a hooks probe answers a hooks step");
    if hooks.deliver != Hook::Missing || hooks.finish != Hook::Missing {
        found
            .checks
            .push(Check { directory: step.directory, kind: Kind::Hooks(hooks) })
            .expect("room for checks per directory");
    }
}

/// Whether the run looks for the checks of the directory at `index`.
fn wants_checks(mounted: Option<&Workspace>, index: u32) -> bool {
    directory(mounted, index).writable
}

fn directory(mounted: Option<&Workspace>, index: u32) -> &Directory {
    let index = usize::try_from(index).expect("a u32 fits in a usize");
    workspace::directories(mounted).get(index).expect("steps are within the workspace")
}
