use crate::{
    errors::{Kind, Rule},
    tests::fixtures::{self, SixLayers, only},
};
type PresenceCase = (&'static str, &'static [(&'static [u8], bool)]);
const PRESENCE: &[PresenceCase] = &[("metal", &[(b"hearth", true)]), ("container", &[])];
#[test]
fn hearth_is_present_except_on_a_container() {
    for (row, &(placement, expected)) in PRESENCE.iter().enumerate() {
        let (documents, target) = fixtures::target_with(placement, SixLayers::NONE);
        assert_eq!(
            fixtures::hearth_task_names(&fixtures::hearth_settings(&documents, &target)).as_ref(),
            expected,
            "row {row}"
        );
    }
}
const SETTINGS: &[SixLayers<'_>] = &[
    SixLayers { fleet: Some("settings"), ..SixLayers::NONE },
    SixLayers { environment: Some("settings"), ..SixLayers::NONE },
];
#[test]
fn fleet_and_environment_reach_hearth_as_settings() {
    for (row, &layers) in SETTINGS.iter().enumerate() {
        let (documents, target) = fixtures::target_with("metal", layers);
        let settings = fixtures::hearth_settings(&documents, &target);
        let crate::compose::Composite::Map(pairs) = &only(&settings.tasks).settings.kind else { panic!("settings") };
        assert_eq!(only(pairs).value.kind, crate::compose::Composite::Int(42), "row {row}");
    }
}
#[test]
fn container_settings_keys_are_dropped_silently() {
    for (row, &layers) in SETTINGS.iter().enumerate() {
        let (documents, target) = fixtures::target_with("container", layers);
        assert_eq!(fixtures::hearth_settings(&documents, &target).tasks.as_ref(), [], "row {row}");
        assert_eq!(fixtures::hearth_errors(&documents, &target).as_ref(), [], "row {row}");
    }
}
const CONTAINER_LAYERS: &[SixLayers<'_>] = &[
    SixLayers { base: Some("{}"), ..SixLayers::NONE },
    SixLayers { base: Some("null"), ..SixLayers::NONE },
    SixLayers { member: Some("{}"), ..SixLayers::NONE },
    SixLayers { member: Some("null"), ..SixLayers::NONE },
    SixLayers { base_env: Some("{}"), ..SixLayers::NONE },
    SixLayers { base_env: Some("null"), ..SixLayers::NONE },
    SixLayers { base_env_member: Some("{}"), ..SixLayers::NONE },
    SixLayers { base_env_member: Some("null"), ..SixLayers::NONE },
];
#[test]
fn a_container_refuses_hearth_in_each_presence_layer() {
    for (row, &layers) in CONTAINER_LAYERS.iter().enumerate() {
        let (documents, target) = fixtures::target_with("container", layers);
        let errors = fixtures::hearth_errors(&documents, &target);
        assert_eq!(only(&errors).kind, fixtures::hearth_fault(Rule::HearthContainer), "row {row}");
        assert_eq!(only(&errors).task.as_deref(), Some(b"hearth".as_slice()), "row {row}");
    }
}
const NULLS: &[SixLayers<'_>] = &[
    SixLayers { fleet: Some("null"), ..SixLayers::NONE },
    SixLayers { base: Some("null"), ..SixLayers::NONE },
    SixLayers { member: Some("null"), ..SixLayers::NONE },
    SixLayers { environment: Some("null"), ..SixLayers::NONE },
    SixLayers { base_env: Some("null"), ..SixLayers::NONE },
    SixLayers { base_env_member: Some("null"), ..SixLayers::NONE },
];
#[test]
fn null_hearth_is_reserved_alone_in_every_layer() {
    for (row, &layers) in NULLS.iter().enumerate() {
        let (documents, target) = fixtures::target_with("metal", layers);
        assert_eq!(
            only(&fixtures::hearth_errors(&documents, &target)).kind,
            fixtures::hearth_fault(Rule::HearthNull),
            "row {row}"
        );
    }
}
#[test]
fn named_hearth_is_one_task_and_is_explicit() {
    let (documents, target) = fixtures::target_with("metal", SixLayers { base: Some("{}"), ..SixLayers::NONE });
    assert_eq!(
        fixtures::hearth_task_names(&fixtures::hearth_settings(&documents, &target)).as_ref(),
        [(b"hearth".as_slice(), false)]
    );
}
#[test]
fn a_container_needs_no_hearth_declaration() {
    let (documents, target) = fixtures::target_without_hearth("container");
    assert_eq!(fixtures::hearth_errors(&documents, &target).as_ref(), []);
}
#[test]
fn missing_hearth_fails_a_target_without_a_position() {
    let (documents, target) = fixtures::target_without_hearth("metal");
    let errors = fixtures::hearth_errors(&documents, &target);
    let error = only(&errors);
    assert_eq!(error.kind, Kind::Composition(crate::errors::Composition::Task));
    assert_eq!(error.at.as_ref().expect("task path").position, None);
    assert_eq!(error.at.as_ref().expect("task path").path.as_ref(), b"tasks/hearth/task.json");
}
