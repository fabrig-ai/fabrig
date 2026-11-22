//! Agent launch fixture and path view (tools/crates/design.md, 3.2).
use std::{os::unix::ffi::OsStrExt, path::Path};

#[must_use]
pub fn path(bytes: &[u8]) -> &Path {
    crate::path(bytes)
}

#[must_use]
pub fn launch(program: &Path, configuration: &Path, words: &[&[u8]]) -> jig_protocol_config::Agent {
    jig_protocol_config::Agent {
        program: program.as_os_str().as_bytes().into(),
        configuration: configuration.as_os_str().as_bytes().into(),
        arguments: words.iter().map(|word| Box::from(*word)).collect(),
    }
}
