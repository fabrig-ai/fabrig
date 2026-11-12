#[test]
fn documentation_answer_bound_matches_the_engine() {
    assert_eq!(temper_service_core::DOC_ANSWER, temper_engine_smith::DOC_ANSWER);
}

#[test]
fn shared_override_names_match_engine_and_handed_owners() {
    use alloc::collections::BTreeSet;
    let mut expected = BTreeSet::new();
    for name in &crate::limits::NAMES { expected.insert(name.name); }
    for name in &temper_forge_setup::limits::NAMES {
        if crate::limits::handed_on(name.name.as_bytes()) { expected.insert(name.name); }
    }
    for name in &temper_forge_setup::limits::HOST_NAMES {
        if crate::limits::handed_on(name.name.as_bytes()) { expected.insert(name.name); }
    }
    for name in &temper_forge_setup::limits::FETCH_NAMES {
        if crate::limits::handed_on(name.name.as_bytes()) { expected.insert(name.name); }
    }
    for name in &temper_box_machine::limits::NAMES {
        assert!(crate::limits::handed_on(name.name.as_bytes()));
        expected.insert(name.name);
    }
    let mut actual = BTreeSet::new();
    for name in temper_service_core::limits::NAMES { actual.insert(*name); }
    assert_eq!(actual.len(), temper_service_core::limits::NAMES.len(), "no duplicate name");
    assert_eq!(actual, expected);
    for profile in [crate::limits::Profile::Small, crate::limits::Profile::Standard] {
        let engine = crate::limits::profile(profile);
        let shared = temper_service_core::limits::profile(profile);
        assert_eq!(crate::limits::worker(&engine), crate::worker_settings::Limits::shared(shared));
    }
}

#[test]
fn worker_close_kind_matches_engine() {
    assert_eq!(crate::worker_settings::CLOSE_KIND, temper_engine_domain_forge::CLOSE_KIND);
    assert_eq!(crate::worker_settings::close_bound(), crate::limits::close_bound());
}
