//! `temper help` (protocol/documentation.md, section 4).
pub use temper_shell::help::{Request, request};

/// What it prints on stdout, or its error line for stderr.
pub fn answer(request: &Request) -> Result<String, String> {
    if let Request::Page(page) = request
        && page.as_ref() == b"commands"
    {
        return Ok(temper_shell::table::commands_page(&crate::TABLE));
    }
    temper_shell::help::answer("temper-box", request)
}
