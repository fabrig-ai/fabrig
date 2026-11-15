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
pub struct PatchSet {
    pub owner: Owner,
    pub name: &'static str,
    pub families: &'static [KeptFamily],
    /// Patch texts in number order.
    pub patches: &'static [&'static str],
    /// The owner's mark across which windows start fresh.
    pub fresh: Option<&'static str>,
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
    /// Kept texts in version order.
    pub texts: &'static [&'static str],
    pub map: KeptMap,
    pub keys: &'static [Keys],
    /// The declaration used as a transcript frame root.
    pub frames: Option<&'static str>,
    /// Ranges whose every record is an effect in flight, its key naming
    /// it (protocol/store/opening.md 3), in `Keys`' form. Empty: none.
    pub flights: &'static [Keys],
    /// The binary's schema text; a patched family's latest kept text.
    pub current: &'static str,
    /// Registered value types; empty for a patched family (protocol/store/layouts.md, 2).
    pub types: &'static [ValueType],
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
pub struct PatchSets {
    /// At most one set per owner.
    pub sets: &'static [PatchSet],
}
impl PatchSets {
    pub const NONE: PatchSets = PatchSets { sets: &[] };
}

/// jig's secrets rows, excluding the application's own rows (protocol/store/secrets.md, protocol/store/opening.md, 3).
pub const JIG: PatchSet = PatchSet {
    jobs: &[],
    owner: Owner::Jig,
    name: "jig",
    patches: &[],
    fresh: None,
    families: &[KeptFamily {
        current: "",
        types: &[],
        name: "jig-store-secrets",
        texts: &[],
        map: KeptMap::Secrets,
        frames: None,
        flights: &[],
        keys: &[
            Keys { from: &[1, 0], to: &[1, 2], root: "Digest" },
            Keys { from: &[3, 0], to: &[3, 1], root: "Source" },
            Keys { from: &[4, 0], to: &[4, 1], root: "Source" },
            Keys { from: &[2, 0, 0x80], to: &[2, 0, 0xc0], root: "Carried" },
            Keys { from: &[2, 1], to: &[2, 2], root: "NextCarried" },
        ],
    }],
};
