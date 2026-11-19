//! Static owner data handed in by the shell (protocol/store/records.md, 3, and protocol/store/opening.md, 3).

/// Whose patch levels the format row keeps, upstream first.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Owner {
    Skein,
    Smith,
    Jig,
    Application,
}

#[derive(Clone, Copy, Debug)]
pub struct OwnerSet {
    pub owner: Owner,
    pub name: &'static str,
    pub families: &'static [KeptFamily],
    /// The owner's level (protocol/store/records.md, 3).
    pub level: Option<u32>,
    pub jobs: &'static [crate::jobs::Job],
}
/// Which of the deployment's two maps a kept family's records live in (protocol/store/secrets.md, protocol/store/opening.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KeptMap {
    Store,
    Secrets,
}

#[derive(Clone, Copy, Debug)]
pub struct KeptFamily {
    pub name: &'static str,
    pub map: KeptMap,
    pub keys: &'static [Keys],
    /// The frames' family's texts and patches (protocol/store/transcripts.md, 6).
    pub frames: Option<Frames>,
    /// The binary's schema text.
    pub current: &'static str,
    /// Registered value types; empty for a family outside the registry (protocol/store/layouts.md, 2).
    pub types: &'static [ValueType],
    pub retired: &'static [Retired],
}

/// A frames' family's history, read by its machine alone (protocol/store/transcripts.md, 6).
#[derive(Clone, Copy, Debug)]
pub struct Frames {
    pub root: &'static str,
    pub texts: &'static [&'static str],
    pub patches: &'static [&'static str],
}

/// A value type whose ranges must hold nothing (protocol/store/layouts.md, 2).
#[derive(Clone, Copy, Debug)]
pub struct Retired {
    pub number: u16,
    pub keys: &'static [crate::KeyRange],
}

/// A stable kind of value, distinguished by its keys (protocol/store/layouts.md, 2).
#[derive(Clone, Copy, Debug)]
pub struct ValueType {
    pub number: u16,
    pub root: &'static str,
    /// A leading enumeration's variant, or none for the whole declaration.
    pub variant: Option<&'static str>,
    pub keys: &'static [crate::KeyRange],
}
#[derive(Clone, Copy, Debug)]
pub struct Keys {
    /// Inclusive; application's own bytes, whole map keys for other owners.
    pub from: &'static [u8],
    /// Exclusive, in the same key space as `from`.
    pub to: &'static [u8],
    /// Declaration of the records in this range.
    pub root: &'static str,
}
#[derive(Clone, Copy, Debug)]
pub struct OwnerSets {
    /// At most one set per owner.
    pub sets: &'static [OwnerSet],
}
impl OwnerSets {
    pub const NONE: OwnerSets = OwnerSets { sets: &[] };
}

/// jig's secrets rows, excluding the application's own rows (protocol/store/secrets.md, protocol/store/opening.md, 3).
pub const JIG: OwnerSet = OwnerSet {
    jobs: &[],
    owner: Owner::Jig,
    name: "jig",
    level: None,
    families: &[KeptFamily {
        retired: &[],
        current: {
            let [.., secrets] = crate::STORE_FAMILIES;
            secrets.current
        },
        types: &[],
        name: "jig-store-secrets",
        map: KeptMap::Secrets,
        frames: None,
        keys: &[
            Keys { from: &[1, 0], to: &[1, 2], root: "Digest" },
            Keys { from: &[3, 0], to: &[3, 1], root: "Source" },
            Keys { from: &[4, 0], to: &[4, 1], root: "Source" },
            Keys { from: &[2, 0, 0x80], to: &[2, 0, 0xc0], root: "Carried" },
            Keys { from: &[2, 1], to: &[2, 2], root: "NextCarried" },
        ],
    }],
};
