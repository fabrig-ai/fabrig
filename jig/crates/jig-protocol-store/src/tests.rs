//! Store key, interval and codec properties (protocol/store/README.md, section 9).
mod codec;
mod commits;
mod directory;
mod erase_reads;
pub(crate) mod fixtures;
mod format;
mod frames;
mod generations;
mod gets;
mod intervals;
mod keys;
mod open;
mod pages;
mod patching;
mod preopen;
mod readers;
mod reads;
mod restore;
mod secrets;
mod start_pages;
mod transcripts;

mod settings;

mod operations;
mod phase;
mod words;

mod frame_memory;
mod frame_patches;
mod settings_layout;
mod settings_patch;

#[test]
fn valid_refuses_each_unrepresentable_limit() {
    let limits = fixtures::store_limits();
    for (name, limits, accepted) in [
        ("fixture", limits, true),
        ("store validity", crate::Limits { store: skein_kv::Limits { key: 0, ..limits.store }, ..limits }, false),
        ("secret validity", crate::Limits { secrets: skein_kv::Limits { key: 0, ..limits.secrets }, ..limits }, false),
        (
            "decoded codec",
            crate::Limits {
                core: jig_domain_core_vocabulary::Limits {
                    tasks: jig_domain_tasks::Limits { spec_bytes: u32::MAX, ..limits.core.tasks },
                    ..limits.core
                },
                ..limits
            },
            false,
        ),
        ("reservation addition", crate::Limits { queue: u32::MAX, ..limits }, false),
        (
            "reservation product",
            crate::Limits { commits: crate::sizes::Commits { writes: u32::MAX, ..limits.commits }, ..limits },
            false,
        ),
        ("operation capacity", crate::Limits { orphan_erases: u32::MAX, ..limits }, false),
        (
            "question ranges",
            crate::Limits {
                core: jig_domain_core_vocabulary::Limits {
                    tasks: jig_domain_tasks::Limits { tasks: u32::MAX, inbox_messages: 2, ..limits.core.tasks },
                    ..limits.core
                },
                ..limits
            },
            false,
        ),
        (
            "reader addition",
            crate::Limits { core: jig_domain_core_vocabulary::Limits { load_slots: u32::MAX, ..limits.core }, ..limits },
            false,
        ),
    ] {
        assert_eq!(crate::valid(&limits), accepted, "{name}");
    }
}

mod sizes;

#[test]
fn valid_requires_room_for_transcript_work() {
    let limits = fixtures::store_limits();
    for (name, limits, accepted) in [
        ("empty queue", crate::Limits { queue: 0, ..limits }, false),
        ("queue exceeds store", crate::Limits { queue: limits.store.queued.checked_add(1).unwrap(), ..limits }, false),
        ("empty files", crate::Limits { files: 0, ..limits }, false),
        ("empty orphan erases", crate::Limits { orphan_erases: 0, ..limits }, false),
        (
            "too few readers",
            crate::Limits {
                reads: limits.core.load_slots.checked_add(limits.core.tasks.tasks).unwrap().checked_sub(1).unwrap(),
                ..limits
            },
            false,
        ),
        (
            "exact readers",
            crate::Limits { reads: limits.core.load_slots.checked_add(limits.core.tasks.tasks).unwrap(), ..limits },
            true,
        ),
    ] {
        assert_eq!(crate::valid(&limits), accepted, "{name}");
    }
}

#[test]
#[should_panic(expected = "the store's limits are valid")]
fn over_refuses_insufficient_transcript_readers() {
    let mut limits = fixtures::store_limits();
    limits.reads = limits
        .core
        .load_slots
        .checked_add(limits.core.tasks.tasks)
        .expect("fixture reader capacity")
        .checked_sub(1)
        .expect("fixture has transcript readers");
    let _store = crate::Store::over(&limits, fixtures::APPLICATION, &[], fixtures::stores());
}

mod catch_up;

mod policy_patch;
