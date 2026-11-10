//! The worker's half of a start and program-login, for every binary that runs a worker (temper/docs/design/protocol/binaries.md, 1 and 5).
#[cfg(test)]
extern crate self as temper;
use std::{
    ffi::OsString,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};
mod binary;
pub use binary::main;
pub mod agent;
pub mod program_login;
pub mod worker;
pub use program_login::PROGRAM_LOGIN;
#[cfg(test)]
pub use temper_shell::directories;
#[expect(
    clippy::disallowed_methods,
    reason = "cli C6: TEMPER_WORKSPACE_REUSE selects worker reuse; temper-worker's and temper-box's tables will pass it"
)]
#[must_use]
pub fn workspace_reuse() -> bool {
    std::env::var_os("TEMPER_WORKSPACE_REUSE").is_some()
}
/// Where a worker's agents keep each model request (temper/docs/design/draft/dogfood-hacks.md, 2.13).
#[expect(
    clippy::disallowed_methods,
    reason = "cli C6: TEMPER_DUMP_LLM selects the agent request dump directory; temper-worker's and temper-box's tables will pass it"
)]
#[must_use]
pub fn dump_llm() -> Option<PathBuf> {
    dump_directory(std::env::var_os("TEMPER_DUMP_LLM"))
}
fn dump_directory(value: Option<OsString>) -> Option<PathBuf> {
    let value = value?;
    if value.is_empty() {
        return None;
    }
    match std::path::absolute(&value) {
        Ok(directory) => Some(directory),
        Err(why) => {
            use std::io::Write;
            drop(writeln!(std::io::stderr(), "temper serve: TEMPER_DUMP_LLM: {why}"));
            None
        }
    }
}
pub const DUMP_BOUND: u64 = 4_294_967_296;
fn path(bytes: &[u8]) -> &Path {
    Path::new(std::ffi::OsStr::from_bytes(bytes))
}
#[cfg(test)]
#[path = "../../temper/tests/support/configuration.rs"]
pub mod test_configuration;
#[cfg(test)]
mod tests;
