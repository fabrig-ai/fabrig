//! Host policy for relative workspace guide/check paths (smith/docs/design/domain/run-start.md, sections 1.1 and 1.3, and smith/docs/design/domain/run-delivery.md, section 1.1, and smith/docs/design/domain/run.md, section 11
//! ). Admitted charters keep up to three owning paths;
//! the absent policy keeps no path allocation and selects Smith defaults.
//! Discovery, system text and exclusive checks borrow the same selection.
//! This module knows no host workflow or filesystem; IO confines resolution,
//! including symlinks, beneath the supplied root. Validation precedes effects.

use alloc::boxed::Box;

use crate::charter::Charter;
use crate::delivery::{Marker, relative_path};

/// Host-supplied workspace conventions, replacing both default relative paths.
/// The host attests UTF-8, as for the charter's text. Admission refuses unsafe
/// or oversized spellings before IO; accepted paths are retained unchanged
/// within Charter's aggregate byte cap. Only the declared paths are read.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Conventions {
    /// Relative guide path read in every mount and named in its system heading.
    /// Missing/nontext/failed reads yield no guide; at most `guide_bytes` are read.
    pub guide: Box<[u8]>,

    /// Relative executable path probed and run only in writable mounts when
    /// a Change or separately granted delivery requires checks. No executable
    /// means no checks; a failing check returns bounded feedback.
    pub checks: Box<[u8]>,

    /// Relative hooks directory; absent selects the check path at both moments.
    pub hooks: Option<Box<[u8]>>,
}

impl Conventions {
    /// Maximum bytes per path, sharing the bounded relative marker vocabulary.
    /// All paths also count against receiving `Limits.run_bytes`; their inline
    /// Box wrappers are part of Charter and the domain's slab bound.
    pub const PATH_CAPACITY: usize = Marker::CAPACITY;
}

pub(crate) fn valid(conventions: &Conventions) -> bool {
    let hooks = match &conventions.hooks {
        Some(path) => valid_path(path),
        None => true,
    };
    valid_path(&conventions.guide) && valid_path(&conventions.checks) && hooks
}

fn valid_path(path: &[u8]) -> bool {
    if path.len() > Conventions::PATH_CAPACITY || !relative_path(path) {
        return false;
    }
    for byte in path {
        // Portable relative spelling and safe single-line prompt labels.
        if *byte < b' ' || *byte == 127 || *byte == b'\\' {
            return false;
        }
    }
    true
}

pub(crate) const GUIDE: &[u8] = b"AGENTS.md";

pub(crate) fn guide(charter: &Charter) -> &[u8] {
    match &charter.conventions {
        Some(conventions) => &conventions.guide,
        None => GUIDE,
    }
}

pub(crate) fn checks(charter: &Charter) -> &[u8] {
    match &charter.conventions {
        Some(conventions) => &conventions.checks,
        None => b".smith/check",
    }
}

/// The optional hooks directory, looked for before the legacy check.
pub(crate) fn hooks(charter: &Charter) -> Option<&[u8]> {
    charter.conventions.as_ref()?.hooks.as_deref()
}
