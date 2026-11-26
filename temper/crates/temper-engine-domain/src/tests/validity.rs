use crate::{JournalLimits, loads};

// All journal fields at their type's maximum; validity allocates no state.
const MAXIMUM_JOURNAL_LIMITS: JournalLimits = JournalLimits {
    commits: u32::MAX,
    held: u32::MAX,
    writes: u32::MAX,
    deliveries: u32::MAX,
    transcript_bytes: u32::MAX,
    result_bytes: u32::MAX,
    run_bytes: u32::MAX,
};

#[test]
fn journal_validity_refuses_zero_entrance_limits() {
    let limits = super::LIMITS;
    for (name, limits) in [
        ("commits", JournalLimits { commits: 0, ..limits }),
        ("writes", JournalLimits { writes: 0, ..limits }),
        ("deliveries", JournalLimits { deliveries: 0, ..limits }),
        ("run bytes", JournalLimits { run_bytes: 0, ..limits }),
    ] {
        assert!(!crate::journal_valid(&limits), "{name}");
    }
}

#[test]
fn journal_validity_accepts_the_header_slot_and_large_counts() {
    let limits = super::LIMITS;
    for (name, limits) in [
        ("fixture", limits),
        ("header only", JournalLimits { writes: 1, held: 0, transcript_bytes: 0, result_bytes: 0, ..limits }),
        ("maximum fields", MAXIMUM_JOURNAL_LIMITS),
    ] {
        assert!(crate::journal_valid(&limits), "{name}");
    }
}

#[test]
fn root_validity_refuses_invalid_child_limits() {
    let limits = super::resume_fixture::limits();
    for (name, limits) in [
        ("journal", crate::Limits { journal: JournalLimits { writes: 0, ..limits.journal }, ..limits }),
        ("loads", crate::Limits { loads: loads::Limits { read_bytes: 0, ..limits.loads }, ..limits }),
        (
            "reply below prefix",
            crate::Limits { loads: loads::Limits { reply_bytes: limits.loads.bytes - 1, ..limits.loads }, ..limits },
        ),
        (
            "forge",
            crate::Limits { forge: temper_engine_domain_forge::Limits { repositories: 0, ..limits.forge }, ..limits },
        ),
    ] {
        assert!(!crate::valid(&limits), "{name}");
    }
}

#[test]
fn root_validity_accepts_dynamic_internal_counts() {
    let limits = super::resume_fixture::limits();
    for (name, limits) in [
        (
            "uncapped forge output",
            crate::Limits { forge: temper_engine_domain_forge::Limits { output: 0, ..limits.forge }, ..limits },
        ),
        ("maximum journal fields", crate::Limits { journal: MAXIMUM_JOURNAL_LIMITS, ..limits }),
    ] {
        assert!(crate::valid(&limits), "{name}");
    }
}

#[test]
fn forge_and_core_take_the_same_root_wait_figures() {
    use skein_lib::{Duration, Env, Time, Wall};

    let mut limits = super::resume_fixture::limits();
    limits.waits.step = Duration::from_secs(23);
    limits.waits.room = Duration::from_secs(47);
    limits.waits.sign_off = Duration::from_secs(61);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let core = crate::core_limits(&limits);
    let forge = crate::translate::environment_forge(&env);
    assert_eq!(core.waits, limits.waits);
    assert_eq!(forge.limits.waits, limits.waits);
}
