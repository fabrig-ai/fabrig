//! temper's program sources (jig/docs/design/protocol/service.md, 11.2): how its settings name a program.
use alloc::boxed::Box;
pub use jig_protocol_worker_agents::sources::{Component, FileStage, Resources};
use smith_protocol_program::Programs;

/// Bind configured paths to the shell user and the opened programs parent.
#[must_use]
pub fn configured(
    program: &crate::settings::Program,
    environment: Box<[Box<[u8]>]>,
    user: u32,
    parent: Option<skein_io::kernel::Fd>,
) -> Option<Resources> {
    match program.adapter {
        crate::settings::Adapter::ClaudeCode => Resources::new(
            Programs { claude: program.path.clone(), directory: program.directory.clone() },
            environment,
            crate::settings::PROGRAM_PATH_BYTES,
            user,
            parent,
            None,
        ),
    }
}
