//! Tagged wire fixtures shared by the process stories.
use skein_lib::Secret;
use smith_channel::{GrantValue, Scheme};

/// A host's bearer wire value.
#[must_use]
pub fn bearer(value: &[u8]) -> GrantValue {
    credential(value, Scheme::Bearer)
}

/// A host's key wire value.
#[must_use]
pub fn key(value: &[u8]) -> GrantValue {
    credential(value, Scheme::Key)
}

fn credential(value: &[u8], scheme: Scheme) -> GrantValue {
    GrantValue::Credential(smith_channel::CredentialValue { scheme, bytes: Secret::new(Box::from(value)) })
}

/// A host's program source wire value.
#[must_use]
pub fn program(name: &[u8]) -> GrantValue {
    GrantValue::Program(smith_channel::ProgramValue { name: Box::from(name) })
}

pub(crate) fn refresh(value: GrantValue) -> skein_channel::Frame {
    let record = smith_channel::GrantRefresh {
        grant: smith_channel::Grant { account: 0, generation: 2, valid: skein_lib::Duration::from_secs(7200), value },
    };
    crate::frame!(0x0104, record)
}
