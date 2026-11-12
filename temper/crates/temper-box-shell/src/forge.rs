//! Forge commands on the composition's checked shape (protocol/services.md, 7).
use std::ffi::OsString;
pub use temper_engine_shell::forge::*;
use temper_shell::directories::{Directories, EnvironmentEntry};
pub fn start(
    directories: &Directories,
    arguments: &[OsString],
    ssl_cert_file: Option<std::path::PathBuf>,
    environment: &[EnvironmentEntry],
) -> Result<u8, Refused> {
    temper_engine_shell::forge::start_with_reader(
        directories,
        arguments,
        ssl_cert_file,
        environment,
        crate::setup::engine,
    )
}
#[must_use]
pub fn run(
    binary: &str,
    directories: &Directories,
    arguments: &[OsString],
    ssl_cert_file: Option<std::path::PathBuf>,
    environment: &[EnvironmentEntry],
) -> u8 {
    temper_engine_shell::forge::run_with_reader(
        binary,
        directories,
        arguments,
        ssl_cert_file,
        environment,
        crate::setup::engine,
    )
}
pub fn command(binary: &str, command: Command) -> Result<u8, String> {
    temper_engine_shell::forge::command_with_reader(binary, command, crate::setup::engine)
}
