//! Bounded credential forms below the domain (smith/docs/design/protocol/channel.md, section 6).
//! Conversion retains no state. Callers store values below the domain and
//! expose only grant names, validity and renewal policy above it.

use alloc::boxed::Box;
use core::fmt;

use crate::Error;
use smith_channel as channel;

/// A host's tagged grant, retained only below the domain.
#[derive(Clone, PartialEq, Eq)]
pub enum GrantValue {
    /// A host's bearer credential.
    Bearer(Box<[u8]>),
    /// A host's key credential.
    Key(Box<[u8]>),
    /// A host's program source directory name.
    Program(Box<[u8]>),
    /// A withdrawn credential carrying its account id and bearer.
    Withdrawn { account_id: Box<[u8]>, bearer: Box<[u8]> },
}

impl fmt::Debug for GrantValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[redacted grant value]")
    }
}

/// Check a program source's directory name at either channel boundary.
pub fn program_name(name: &[u8]) -> Result<&[u8], Error> {
    if name.is_empty() || name.len() > 32 {
        return Err(Error::Grants);
    }
    for byte in name {
        if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && *byte != b'-' {
            return Err(Error::Grants);
        }
    }
    Ok(name)
}

impl GrantValue {
    /// Copy one checked wire credential into its redacted service-owned form.
    pub fn from_wire(value: &smith_channel::GrantValue) -> Result<GrantValue, Error> {
        Ok(match value {
            smith_channel::GrantValue::Credential(value) => match value.scheme() {
                smith_channel::Scheme::Bearer => GrantValue::Bearer(Box::from(value.bytes().expose())),
                smith_channel::Scheme::Key => GrantValue::Key(Box::from(value.bytes().expose())),
            },
            smith_channel::GrantValue::Program(value) => {
                let name = program_name(value.name())?;
                GrantValue::Program(Box::from(name))
            }
            smith_channel::GrantValue::Withdrawn(value) => {
                GrantValue::Withdrawn { account_id: Box::from(value.id()), bearer: Box::from(value.bearer().expose()) }
            }
        })
    }
}

/// Renewal policy already carried by the existing wire form (smith/docs/design/protocol/channel.md, section 6).
pub(crate) fn renewable(value: &channel::GrantValue) -> bool {
    match value {
        channel::GrantValue::Credential(credential) => match credential.scheme() {
            channel::Scheme::Key => false,
            channel::Scheme::Bearer => true,
        },
        channel::GrantValue::Program(_) | channel::GrantValue::Withdrawn(_) => true,
    }
}

/// Refresh conversion used after the channel retains the value (smith/docs/design/protocol/channel.md, section 6).
/// Expose only a refreshed grant name and validity to the domain.
#[must_use]
pub fn refreshed_grant(value: &channel::Grant) -> smith_domain::Grant {
    smith_domain::Grant {
        name: smith_domain::GrantName { account: value.account(), generation: value.generation() },
        valid: value.valid(),
        renewable: renewable(value.value()),
    }
}

#[cfg(test)]
mod tests {
    use super::program_name;
    use crate::Error;
    // protocol/channel.md, section 6 bounds a program source name at 32 bytes.
    const PROGRAM_NAME_BYTES: usize = 32;

    #[test]
    fn a_program_name_outside_its_charset_is_refused() {
        let rows = [
            (b"".as_slice(), Some(Error::Grants)),
            (&[b'a'; PROGRAM_NAME_BYTES + 1], Some(Error::Grants)),
            (b"Upper", Some(Error::Grants)),
            (b"under_score", Some(Error::Grants)),
            (b"\xc3\xa9", Some(Error::Grants)),
        ];
        for (index, (name, expected)) in rows.into_iter().enumerate() {
            assert_eq!(program_name(name).err(), expected, "row {index}");
        }
    }

    #[test]
    fn a_program_name_at_both_length_bounds_is_accepted() {
        let rows = [(b"a".as_slice(), b"a".as_slice()), (&[b'a'; PROGRAM_NAME_BYTES], &[b'a'; PROGRAM_NAME_BYTES])];
        for (index, (name, expected)) in rows.into_iter().enumerate() {
            assert_eq!(program_name(name), Ok(expected), "row {index}");
        }
    }
}
