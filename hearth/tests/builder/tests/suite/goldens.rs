use hearth_builder_world::fixtures;
use hearth_test_render::golden;
#[test]
fn merge_renders_what_its_expected_files_and_witnesses_say() {
    fixtures::check_golden("merge", "", &[]);
}
#[test]
fn merge_files_show_each_layers_value_in_its_witness() {
    let golden = golden::load("merge");
    let verdict = golden.witnesses(&fixtures::golden_output("merge"));
    assert!(verdict.is_clean(), "{}", verdict.report());
}
#[test]
fn merge_removal_by_null_leaves_the_task_out_of_an_env() {
    let targets = golden::Golden::targets(&fixtures::golden_output("merge"));
    let gone: Vec<_> =
        targets.iter().filter(|(_, text)| text.contains("/etc/gone ")).map(|(name, _)| name.as_str()).collect();
    assert_eq!(gone, ["m-b", "m-c"]);
}

#[test]
fn templates_render_byte_for_byte_as_their_witnesses_say() {
    fixtures::check_golden("templates", "", &[]);
}
#[test]
fn the_fleet_renders_its_six_targets_as_expected() {
    fixtures::check_golden("fleet", "", &[]);
}
#[test]
fn a_lab_spec_does_not_change_when_a_production_file_does() {
    let repository = fixtures::golden_repository("fleet");
    let input = fixtures::render(&repository, &[b"carbon-lab"], hearth::Form::Text);
    drop(repository.next(&[fixtures::CARBON_PRODUCTION_EDIT], &[]));
    let after = fixtures::render(&repository, &[b"carbon-lab"], hearth::Form::Text);
    assert_eq!(after.stdout, input.stdout);
}
#[test]
fn the_container_has_no_hearth_task_and_its_host_has_one() {
    let repository = fixtures::golden_repository("fleet");
    drop(repository.next(&[], &["tasks/hearth/task.json"]));
    let rendered = fixtures::render(&repository, &[b"carbon-prod", b"chat-prod"], hearth::Form::Text);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["target ", "tasks/"]),
        "target       chat-prod   base chat   environment prod\n\
tasks/hearth/task.json: carbon-prod: no task hearth under tasks/\n"
    );
}
#[test]
fn the_cluster_renders_its_members_with_their_names() {
    fixtures::check_golden("cluster", "", &[]);
}
#[test]
fn a_base_env_file_removes_a_member_from_its_environment() {
    let targets = golden::Golden::targets(&fixtures::golden_output("cluster"));
    assert_eq!(
        targets.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>(),
        ["stoa-1-lab", "stoa-1-prod", "stoa-2-lab", "stoa-2-prod", "stoa-3-lab"]
    );
}

#[test]
fn the_vpn_and_mail_golden_render_with_their_peers_per_environment() {
    fixtures::check_golden("exports", "", &[]);
}
#[test]
fn each_error_case_prints_its_errors_after_the_healthy_spec() {
    for name in fixtures::ERROR_CASES {
        fixtures::check_golden(name, "", &[]);
    }
}
#[test]
fn an_error_case_exits_2() {
    let repository = fixtures::golden_repository("errors/schema");
    let rendered = fixtures::render(&repository, &[], hearth::Form::Text);
    assert_eq!(fixtures::exit_code(&rendered), Some(2));
}

#[test]
fn an_empty_store_prints_every_resolution_needed() {
    fixtures::check_loaded_golden(&fixtures::resolution_golden(), "", &[]);
}
#[test]
fn a_laid_record_prints_known_for_its_targets_alone() {
    let recorded = fixtures::Laid::Resolution(fixtures::test_min_recorded());
    fixtures::check_loaded_golden(&fixtures::resolution_golden(), "known", core::slice::from_ref(&recorded));
}
