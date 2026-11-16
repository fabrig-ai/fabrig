//! Jig's owner set (jig/docs/design/protocol/store/opening.md, 3).
use jig_protocol_store::{KeptFamily, KeptMap, OwnerSet};

/// The component, core, children and secrets; Moved is held inside Carried.
pub const JIG_SET: OwnerSet = {
    let [component, openings, registry, core, tasks, people, notes, accounts, settings, secrets] =
        jig_protocol_store::STORE_FAMILIES;
    OwnerSet {
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
                retired: &[],
                current: crate::patches::kept::jig_accounts::TEXTS.last().expect("kept text"),
                types: &[],
                name: "jig-accounts",
                map: KeptMap::Secrets,
                keys: &[],
                frames: None,
            },
        ],
        ..jig_protocol_store::JIG_STORE_SET
    }
};
