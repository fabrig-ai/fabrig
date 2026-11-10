//! The binary's three directories (protocol/configuration/directories.md).
pub use temper_engine_shell::serve::program_environment;
pub use temper_service_shell::jig_directories;
pub use temper_shell::directories::*;
#[cfg(test)]
pub(crate) mod tests;
