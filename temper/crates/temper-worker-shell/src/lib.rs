//! The worker's half of a start and program-login, for every binary that runs a worker (temper/docs/design/protocol/binaries.md, 1 and 5).
#[cfg(test)]
extern crate self as temper_box_shell;
use std::{
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};
mod binary;
pub mod lines;
pub mod served;
pub use binary::{WORKER, main};
use temper_shell::directories::EnvironmentEntry;
pub mod agent;
pub mod program_login;
pub mod worker;
pub use program_login::PROGRAM_LOGIN;
#[cfg(test)]
pub use temper_shell::directories;

/// The worker's workspace reuse switch (temper/docs/design/cli/README.md, 8).
pub const INTERNAL_WORKSPACE_REUSE: skein_shell::Variable = skein_shell::Variable {
    name: "TEMPER_INTERNAL_WORKSPACE_REUSE",
    set: skein_shell::Set::Internal,
    form: "ANY",
    way: skein_shell::Way::Reads,
    line: "Reuse worker workspaces across restarts.",
    absent: "Start with fresh worker workspaces.",
    notes: "Any nonempty value enables reuse; an empty value leaves it off. Never a shipped default.",
};

#[must_use]
pub fn workspace_reuse(environment: &[EnvironmentEntry]) -> bool {
    INTERNAL_WORKSPACE_REUSE.value(environment).is_some()
}
/// Where a worker's agents keep each model request (temper/docs/design/draft/dogfood-hacks.md, 2.13).
#[must_use]
pub fn dump_llm(environment: &[EnvironmentEntry]) -> Option<PathBuf> {
    let value = temper_shell::environment::INTERNAL_DUMP_LLM.value(environment)?;
    match std::path::absolute(path(value)) {
        Ok(directory) => Some(directory),
        Err(why) => {
            use std::io::Write;
            drop(writeln!(std::io::stderr(), "temper serve: TEMPER_INTERNAL_DUMP_LLM: {why}"));
            None
        }
    }
}
fn path(bytes: &[u8]) -> &Path {
    Path::new(std::ffi::OsStr::from_bytes(bytes))
}
#[cfg(test)]
#[path = "../../../testing/temper-box-kit/src/configuration.rs"]
pub mod test_configuration;
#[cfg(test)]
mod tests;
