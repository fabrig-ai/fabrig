//! Configured endpoint identities (smith/docs/design/protocol/charter.md, section 5).
use alloc::boxed::Box;
use skein_lib::List;

/// One configured endpoint name and its domain-visible identities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Endpoint {
    /// Byte name carried by charters and transcripts.
    pub name: Box<[u8]>,
    /// Domain-visible endpoint identity.
    pub number: u32,
    /// Opaque replay dialect identity.
    pub dialect: u32,
}

/// Bounded endpoint names resolved before an agent starts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Endpoints {
    entries: List<Endpoint>,
}

impl Endpoints {
    /// Retain the checked startup endpoint table.
    #[must_use]
    pub fn new(entries: List<Endpoint>) -> Endpoints {
        Endpoints { entries }
    }

    /// Number of names retained by the agent.
    #[must_use]
    pub(crate) fn len(&self) -> u32 {
        self.entries.len()
    }

    /// Whether the configured names fit the channel's endpoint allowance.
    #[must_use]
    pub fn fits(&self, count: u32, name_bytes: u32) -> bool {
        if self.len() > count {
            return false;
        }
        for endpoint in &self.entries {
            match u32::try_from(endpoint.name.len()) {
                Ok(length) if length <= name_bytes => {}
                Ok(_) | Err(_) => return false,
            }
        }
        true
    }

    /// Find a charter name without exposing endpoint addresses.
    #[must_use]
    #[expect(clippy::manual_find, reason = "production steps do not use closures")]
    pub fn resolve(&self, name: &[u8]) -> Option<&Endpoint> {
        for endpoint in &self.entries {
            if endpoint.name.as_ref() == name {
                return Some(endpoint);
            }
        }
        None
    }

    /// Find the startup wire name by its unique configured endpoint number.
    #[must_use]
    pub fn name_for(&self, number: u32) -> Option<&[u8]> {
        for endpoint in &self.entries {
            if endpoint.number == number {
                return Some(&endpoint.name);
            }
        }
        None
    }

    /// Find the configured wire name for one domain endpoint and dialect.
    #[must_use]
    pub fn name_of(&self, number: u32, dialect: u32) -> Option<&[u8]> {
        for endpoint in &self.entries {
            if endpoint.number == number && endpoint.dialect == dialect {
                return Some(&endpoint.name);
            }
        }
        None
    }
}
