//! The reference worker's program sources (jig/docs/design/protocol/service.md, 11.2):
//! its bound on a path, and its startup's paths.
use alloc::boxed::Box;
pub use jig_protocol_worker_agents::sources::{Component, FileStage, Resources};
use smith_protocol_program::Programs;

/// The reference worker's bound on a program path.
pub const PATH_BYTES: usize = 1024;

/// Startup's already parsed agent paths and explicit environment.
#[must_use]
pub fn from_paths(
    directory: Box<[u8]>,
    claude: Box<[u8]>,
    environment: Box<[Box<[u8]>]>,
    user: u32,
    parent: Option<skein_io::kernel::Fd>,
) -> Option<Resources> {
    Resources::new(Programs { directory, claude }, environment, PATH_BYTES, user, parent)
}
