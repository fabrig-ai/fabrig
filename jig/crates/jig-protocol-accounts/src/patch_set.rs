//! Jig's owner set (jig/docs/design/protocol/store/opening.md, 3).
use jig_protocol_store::{KeptFamily, KeptMap, PatchSet};

/// The component, core, children and secrets; Moved is held inside Carried.
pub const JIG_PATCH_SET: PatchSet = {
    let [component, openings, registry, core, tasks, people, notes, accounts, settings, secrets] =
        jig_protocol_store::JIG_STORE_FAMILIES;
    PatchSet {
        families: &[
            component,
            openings,
            registry,
            core,
            tasks,
            people,
            notes,
            accounts,
            settings,
            secrets,
            KeptFamily {
                current: crate::patches::kept::jig_accounts::TEXTS.last().expect("kept text"),
                types: &[],
                name: "jig-accounts",
                texts: crate::patches::kept::jig_accounts::TEXTS,
                map: KeptMap::Secrets,
                keys: &[],
                frames: None,
                flights: &[],
            },
        ],
        ..jig_protocol_store::JIG_STORE_PATCH_SET
    }
};
