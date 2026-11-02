use crate::{Profile, profile};

#[test]
fn supplied_profiles_have_valid_domain_limits() {
    for selected in [Profile::Small, Profile::Standard] {
        let limits = profile(selected);
        assert_eq!(limits.core.tasks.charter_documents, 2);
        assert_eq!(limits.core.brief.sections, 8);
        assert!(jig_domain_core::valid(&limits.core), "{selected:?} core");
        assert!(jig_domain_host::valid_hub(&limits.hub), "{selected:?} hub");
    }
}

#[test]
fn child_validity_keeps_semantic_and_count_refusals() {
    let limits = profile(Profile::Small).core;
    let mut brief = limits.brief;
    brief.briefs = 0;
    assert!(!jig_domain_brief::valid(&brief));
    brief = limits.brief;
    brief.sections = 0;
    assert!(!jig_domain_brief::valid(&brief));
    brief = limits.brief;
    brief.read_bytes = 0;
    assert!(!jig_domain_brief::valid(&brief));
    brief = limits.brief;
    brief.brief_bytes = 0;
    assert!(!jig_domain_brief::valid(&brief));
    let mut notes = limits.notes;
    notes.scopes = 0;
    assert!(!jig_domain_notes::valid(&notes));
    notes = limits.notes;
    notes.entries_per_scope = 0;
    assert!(!jig_domain_notes::valid(&notes));
    notes = limits.notes;
    notes.load_rows = 0;
    assert!(!jig_domain_notes::valid(&notes));
    notes = limits.notes;
    notes.recalled = 0;
    assert!(!jig_domain_notes::valid(&notes));
    notes = limits.notes;
    notes.scopes = u32::MAX;
    assert!(!jig_domain_notes::valid(&notes));
    let mut views = limits.views;
    views.runs = 0;
    assert!(!jig_domain_views::valid(&views));
    views = limits.views;
    views.watchers = 0;
    assert!(!jig_domain_views::valid(&views));
    views = limits.views;
    views.watchers_per_party = 0;
    assert!(!jig_domain_views::valid(&views));
    views = limits.views;
    views.subjects_per_change = 0;
    assert!(!jig_domain_views::valid(&views));
    views = limits.views;
    views.text_bytes = 0;
    assert!(!jig_domain_views::valid(&views));
    views = limits.views;
    views.watchers = u32::MAX;
    assert!(!jig_domain_views::valid(&views));
    let mut fleet = limits.fleet;
    fleet.worker_name_bytes = jig_domain_fleet::WORKER_NAME + 1;
    assert!(!jig_domain_fleet::valid(&fleet));
    fleet = limits.fleet;
    fleet.workers = u32::MAX;
    fleet.engine_slots = 1;
    assert!(!jig_domain_fleet::valid(&fleet));
    fleet = limits.fleet;
    fleet.history = u32::MAX;
    assert!(!jig_domain_fleet::valid(&fleet));
}

#[test]
fn configured_read_room_is_smaller_than_the_maximum_frame() {
    // The frame bound is TurnRecord::encoded_bound + 12, including
    // version, optional read, transcript length, checksum and both lengths.
    const MAX_FRAME: u64 = 8_388_672;
    // The map pages include job specs, task/run revisions and placed references;
    // transcript framing and direct read room are unchanged.
    for (kind, request, page) in [(Profile::Small, 65_536, 1_606_795), (Profile::Standard, 262_144, 10_769_483)] {
        let configured = profile(kind);
        let codec = jig_protocol_store::codec::limits(&configured.core).expect("profile codec");
        assert_eq!(u64::from(configured.core.resume_bytes), request);
        assert_eq!(configured.store.store.page.bytes, page);
        assert_eq!(jig_protocol_store::core_v6::TurnRecord::encoded_bound(&codec.core).expect("body") + 12, MAX_FRAME);
        assert!(request.min(u64::from(page)) < MAX_FRAME);
    }
}
