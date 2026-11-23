//! Store key, interval and codec properties (protocol/store/README.md, section 9).
const COMPONENT_SCHEMA: &str = crate::STORE_FAMILIES[0].current;

mod admission;
mod census;
mod checkpoints;
mod commits;
mod dev_store;
mod directory;
mod erase_reads;
pub(crate) mod fixtures;
mod format;
mod frames;
mod generations;
mod gets;
mod history_logs;
mod history_writes;
mod intervals;
mod keys;
mod lifecycle;
mod live_parts;
mod open;
mod owners;
mod pages;
mod preopen;
mod readers;
mod reads;
mod rehearsal;
mod restore;
mod restoring;
mod secrets;
mod start_pages;
mod transcripts;
mod turns_logs;

mod settings;

mod operations;
mod phase;
mod retention;
mod words;

mod frame_memory;
mod frame_patches;
mod settings_patch;

#[test]
fn check_refuses_each_unrepresentable_limit() {
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
            crate::Limits {
                core: jig_domain_core_vocabulary::Limits { load_slots: u32::MAX, ..limits.core },
                ..limits
            },
            false,
        ),
    ] {
        assert_eq!(crate::check(&limits).is_some(), accepted, "{name}");
    }
}

#[test]
fn checked_bounds_are_kept_at_construction_and_reopen() {
    let stock = fixtures::store_limits();
    let mut custom = stock;
    custom.checkpoints.keep = 3;
    custom.core.tasks.tasks += 1;
    custom.reads += 1;
    for limits in [stock, custom] {
        let checked = crate::check(&limits).expect("admitted fixture limits");
        assert_eq!(checked.limits(), &limits);
        assert_eq!(Some(*checked.codec()), limits.codec_view());
        let store = crate::Store::over(
            &checked,
            fixtures::APPLICATION,
            &[],
            crate::Stores {
                store: skein_kv::Store::memory(&limits.store),
                secrets: skein_kv::Store::memory(&limits.secrets),
            },
        );
        assert_eq!(store.limits, limits);
        assert_eq!(&store.codec_limits, checked.codec());
        assert_eq!(store.checked(), checked);
        let mut reopened = crate::Store::over(&checked, fixtures::APPLICATION, &[], store.into_stores());
        let events = fixtures::run(
            &mut reopened,
            crate::Request::Open {
                purpose: crate::Purpose::Serve,
                owner: skein_lib::Token::new(1),
                root: fixtures::ROOT,
                installation: fixtures::INSTALLATION,
            },
        );
        let [crate::Event::Opened { last: 0, .. }, crate::Event::Census { .. }] = events.as_ref() else {
            panic!("reopened store: {events:?}");
        };
        assert_eq!(reopened.checked(), checked);
    }
}

mod sizes;
mod startup_limits;

#[test]
fn check_requires_room_for_transcript_work() {
    let limits = fixtures::store_limits();
    for (name, limits, accepted) in [
        ("empty queue", crate::Limits { queue: 0, ..limits }, false),
        ("queue exceeds store", crate::Limits { queue: limits.store.queued.checked_add(1).unwrap(), ..limits }, false),
        ("empty files", crate::Limits { files: 0, ..limits }, false),
        ("empty history fetch", crate::Limits { fetch: 0, ..limits }, false),
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
        assert_eq!(crate::check(&limits).is_some(), accepted, "{name}");
    }
}
