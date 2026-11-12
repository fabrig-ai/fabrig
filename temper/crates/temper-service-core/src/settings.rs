//! Shared settings (protocol/services.md, 3).
use alloc::boxed::Box;

pub use jig_protocol_config::Files;

/// The adapters recognized by smith, separate from a program source's name.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Adapter {
    ClaudeCode,
}
impl Adapter {
    #[must_use]
    pub fn name(self) -> &'static [u8] {
        match self {
            Self::ClaudeCode => b"claude_code",
        }
    }
}
/// Smith's checked bound for each program path.
pub const PROGRAM_PATH_BYTES: usize = 4096;
/// Secret-free declaration, retained once by the worker; never a login.
#[derive(Clone, Debug)]
pub struct Program {
    pub adapter: Adapter,
    pub path: Box<[u8]>,
    pub directory: Box<[u8]>,
}
