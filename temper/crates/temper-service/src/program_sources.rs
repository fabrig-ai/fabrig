//! temper's program sources (jig/docs/design/protocol/service.md, 11.2): how its settings name a program.
use alloc::boxed::Box;
pub use jig_protocol_worker_agents::sources::{Component, Resources};
use smith_protocol_program::Programs;

#[must_use]
pub fn configured(program: &crate::settings::Program, environment: Box<[Box<[u8]>]>) -> Option<Resources> {
    match program.adapter {
        crate::settings::Adapter::ClaudeCode => Resources::new(
            Programs { claude: program.path.clone(), directory: program.directory.clone() },
            environment,
            crate::settings::PROGRAM_PATH_BYTES,
        ),
    }
}
