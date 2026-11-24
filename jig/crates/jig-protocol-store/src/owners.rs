//! Owners kept by the deployment (jig/docs/design/protocol/store/opening-owners.md, 1).
use crate::upgrade::sets::{Frames, KeptFamily, KeptMap, Owner, OwnerSet};

/// OAuth records are held inside the secrets family's Source, never direct map rows.
pub const SKEIN_SET: OwnerSet = OwnerSet {
    jobs: &[],
    owner: Owner::Skein,
    name: "skein",
    level: Some(0),
    families: &[KeptFamily {
        retired: &[],
        current: skein_oauth::SCHEMA,
        types: &[],
        name: "skein-oauth",
        map: KeptMap::Secrets,
        keys: &[],
        frames: None,
    }],
};

/// Turns are held in the core's transcript frames, never direct map rows.
pub const SMITH_SET: OwnerSet = OwnerSet {
    jobs: &[],
    owner: Owner::Smith,
    name: smith_transcript::OWNER,
    level: Some(0),
    families: &[KeptFamily {
        retired: &[],
        current: smith_transcript::patches::kept::smith_transcript::TEXTS.last().expect("kept text"),
        types: &[],
        name: smith_transcript::FAMILY,
        map: KeptMap::Store,
        keys: &[],
        frames: Some(Frames {
            root: smith_transcript::ROOT,
            texts: smith_transcript::patches::kept::smith_transcript::TEXTS,
            number: 0,
        }),
    }],
};
