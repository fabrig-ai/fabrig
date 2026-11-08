//! `temper help` (protocol/documentation.md, section 4).
pub use temper_shell::help::{Request, request};

/// What it prints on stdout, or its error line for stderr.
pub fn answer(request: &Request) -> Result<String, String> {
    temper_shell::help::answer("temper", request)
}
