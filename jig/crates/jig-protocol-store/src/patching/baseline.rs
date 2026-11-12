//! Owners kept by the deployment (temper/docs/design/protocol/baseline.md, 4;
//! jig/docs/design/protocol/store/opening.md, 3).
use super::sets::{KeptFamily, KeptMap, Keys, Owner, PatchSet};
use crate::patches;

/// OAuth records are held inside the secrets family's Source, never direct map rows.
pub const SKEIN_PATCH_SET: PatchSet = PatchSet {
    owner: Owner::Skein,
    name: "skein",
    patches: skein_oauth::patches::PATCHES,
    fresh: None,
    families: &[KeptFamily {
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
    owner: Owner::Smith,
    name: smith_transcript::OWNER,
    patches: smith_transcript::patches::PATCHES,
    fresh: Some(smith_transcript::FRESH_WINDOWS),
    families: &[KeptFamily {
        name: smith_transcript::FAMILY,
        texts: smith_transcript::patches::kept::smith_transcript::TEXTS,
        map: KeptMap::Store,
        keys: &[],
        frames: Some(smith_transcript::ROOT),
        flights: &[],
    }],
};

/// The component, core, children and secrets; Moved is held inside Carried.
pub const JIG_PATCH_SET: PatchSet = PatchSet {
    owner: Owner::Jig,
    name: "jig",
    patches: patches::PATCHES,
    fresh: None,
    families: &[
        KeptFamily {
            name: "jig-store-component",
            texts: patches::kept::jig_store_component::TEXTS,
            map: KeptMap::Store,
            keys: &[
                Keys { from: &[0, 1], to: &[0, 2], root: "Transcript" },
                Keys { from: &[0, 2], to: &[0, 3], root: "Attempt" },
            ],
            frames: None,
            flights: &[],
        },
        KeptFamily {
            name: "jig-store-core",
            texts: patches::kept::jig_store_core::TEXTS,
            map: KeptMap::Store,
            keys: &[Keys { from: &[1, 0, 0], to: &[1, 0, 8], root: "Row" }],
            frames: Some("TurnRecord"),
            flights: &[],
        },
        KeptFamily {
            name: "jig-store-tasks",
            texts: patches::kept::jig_store_tasks::TEXTS,
            map: KeptMap::Store,
            keys: &[Keys { from: &[1, 1], to: &[1, 2], root: "Row" }],
            frames: None,
            flights: &[],
        },
        KeptFamily {
            name: "jig-store-people",
            texts: patches::kept::jig_store_people::TEXTS,
            map: KeptMap::Store,
            keys: &[Keys { from: &[1, 2], to: &[1, 3], root: "Row" }],
            frames: None,
            flights: &[],
        },
        KeptFamily {
            name: "jig-store-notes",
            texts: patches::kept::jig_store_notes::TEXTS,
            map: KeptMap::Store,
            keys: &[Keys { from: &[1, 3], to: &[1, 4], root: "Row" }],
            frames: None,
            flights: &[],
        },
        KeptFamily {
            name: "jig-store-accounts",
            texts: patches::kept::jig_store_accounts::TEXTS,
            map: KeptMap::Store,
            keys: &[
                Keys { from: &[1, 4, 0], to: &[1, 4, 3], root: "Row" },
                Keys { from: &[1, 4, 4], to: &[1, 4, 6], root: "Row" },
            ],
            frames: None,
            flights: &[],
        },
        KeptFamily {
            name: "jig-store-settings",
            texts: patches::kept::jig_store_settings::TEXTS,
            map: KeptMap::Store,
            keys: &[
                Keys { from: &[1, 0, 8, 0], to: &[1, 0, 8, 1], root: "Default" },
                Keys { from: &[1, 0, 8, 1, 0], to: &[1, 0, 8, 1, 4], root: "Changed" },
                Keys { from: &[1, 0, 8, 1, 4], to: &[1, 0, 8, 1, 5], root: "ApplicationState" },
                Keys { from: &[1, 0, 8, 1, 5], to: &[1, 0, 8, 2], root: "Changed" },
                Keys { from: &[1, 0, 8, 2], to: &[1, 0, 8, 3], root: "Revision" },
                Keys { from: &[1, 0, 8, 3], to: &[1, 0, 8, 4], root: "CharterVersion" },
                Keys { from: &[1, 0, 8, 4], to: &[1, 0, 8, 5], root: "Proposal" },
                Keys { from: &[1, 0, 8, 5], to: &[1, 0, 8, 6], root: "Extent" },
                Keys { from: &[1, 0, 9], to: &[1, 0, 10], root: "Revision" },
            ],
            frames: None,
            flights: &[],
        },
        KeptFamily {
            name: "jig-store-secrets",
            texts: patches::kept::jig_store_secrets::TEXTS,
            map: KeptMap::Secrets,
            keys: &[
                Keys { from: &[1, 0], to: &[1, 2], root: "Digest" },
                Keys { from: &[2, 0, 0x80], to: &[2, 0, 0xc0], root: "Carried" },
                Keys { from: &[2, 1], to: &[2, 2], root: "NextCarried" },
                Keys { from: &[3, 0], to: &[3, 1], root: "Source" },
                Keys { from: &[4, 0], to: &[4, 1], root: "Source" },
            ],
            frames: None,
            flights: &[],
        },
        KeptFamily {
            name: "jig-accounts",
            texts: jig_protocol_accounts::patches::kept::jig_accounts::TEXTS,
            map: KeptMap::Secrets,
            keys: &[],
            frames: None,
            flights: &[],
        },
    ],
};
