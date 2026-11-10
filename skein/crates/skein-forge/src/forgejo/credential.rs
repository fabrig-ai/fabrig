//! Forgejo's authorization field (forge/forgejo.md, section 1.1, and forge/README.md, section 7).
use crate::credential::Credential;
use skein_lib::{Secret, Writer};

/// Renders a credential with the dialect's prefix, retaining its redaction.
#[must_use]
pub fn authorization(credential: &Credential) -> Option<Secret> {
    let token = match credential {
        Credential::Token(token) => token,
        // A Forgejo forge never holds an App: the connection refuses it (runtime 20).
        Credential::App { .. } => return None,
    };
    let prefix = b"token ";
    let mut out = Writer::new(prefix.len().checked_add(token.len())?);
    out.put(prefix).ok()?;
    out.put(token.expose()).ok()?;
    Some(Secret::new(out.finish()))
}
