//! The executable's one-shot tools entry point (client/tools.md, 8.1).
#[cfg(test)]
pub(crate) use temper_client_shell_cli::start::parse;
pub use temper_client_shell_cli::start::run;
