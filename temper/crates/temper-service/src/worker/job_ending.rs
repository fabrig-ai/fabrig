//! The worker's job vocabulary (protocol/services.md, 5).
use alloc::boxed::Box;
/// A worker job's ending, in the host's words (protocol/services.md, 5).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum JobEnding {
    Failed { failure: jig_domain_host::JobFailure, detail: Box<[u8]> },
    Refused(jig_domain_host::JobRefusal),
}
