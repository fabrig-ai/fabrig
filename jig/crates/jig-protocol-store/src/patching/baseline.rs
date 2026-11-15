//! Owners kept by the deployment (jig/docs/design/protocol/store/opening.md, 3).
use super::sets::{KeptFamily, KeptMap, Owner, PatchSet};

/// OAuth records are held inside the secrets family's Source, never direct map rows.
pub const SKEIN_PATCH_SET: PatchSet = PatchSet {
    jobs: &[],
    owner: Owner::Skein,
    name: "skein",
    level: Some(0),
    patches: skein_oauth::patches::PATCHES,
    fresh: None,
    families: &[KeptFamily {
        retired: &[],
        current: skein_oauth::patches::kept::skein_oauth::TEXTS.last().expect("kept text"),
        types: &[],
        name: "skein-oauth",
        texts: skein_oauth::patches::kept::skein_oauth::TEXTS,
        map: KeptMap::Secrets,
        keys: &[],
        frames: None,
        flights: &[],
    }],
};

/// Turns are held in the core's transcript frames, never direct map rows.
pub const SMITH_PATCH_SET: PatchSet = PatchSet {
    jobs: &[],
    owner: Owner::Smith,
    name: smith_transcript::OWNER,
    level: Some(0),
    patches: smith_transcript::patches::PATCHES,
    fresh: Some(smith_transcript::FRESH_WINDOWS),
    families: &[KeptFamily {
        retired: &[],
        current: smith_transcript::patches::kept::smith_transcript::TEXTS.last().expect("kept text"),
        types: &[],
        name: smith_transcript::FAMILY,
        texts: smith_transcript::patches::kept::smith_transcript::TEXTS,
        map: KeptMap::Store,
        keys: &[],
        frames: Some(smith_transcript::ROOT),
        flights: &[],
    }],
};
