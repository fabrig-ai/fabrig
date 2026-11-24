//! Render refusals and diagnostics over fake git's cut pipes (hearth/docs/design/testing/README.md, 2.5).
use hearth::Form;
use hearth_builder_world::{
    fixtures::{self, File, Scenario},
    protocol,
};

const BROKEN_ENVIRONMENTS: u32 = 300;

fn render(files: &[File], form: Form) -> protocol::Run {
    protocol::run(&Scenario { files, form, ..fixtures::thin() }, fixtures::SIMULATION_SEED)
}

fn thin(overrides: &[File]) -> protocol::Run {
    render(&fixtures::thin_with(overrides), Form::Text)
}

#[test]
fn a_broken_document_prints_its_error_after_the_specs_that_rendered() {
    let rendered = thin(fixtures::BROKEN_LAB);
    assert_eq!(String::from_utf8_lossy(&rendered.stdout), fixtures::BROKEN_LAB_TEXT);
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn a_broken_task_json_prints_its_error_with_its_task() {
    let rendered = render(&fixtures::broken_task(), Form::Text);
    assert_eq!(
        String::from_utf8_lossy(&rendered.stdout),
        "tasks/motd/task.json:1:10: motd: duplicate key \"a\" (tasks/motd/task.json:1:2)\n"
    );
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn a_task_json_out_of_shape_prints_its_fault_and_fails_its_targets() {
    let rendered = thin(&[fixtures::MISKEYED]);
    assert_eq!(
        String::from_utf8_lossy(&rendered.stdout),
        "tasks/motd/task.json:1:2: motd: no such key in a declaration\n"
    );
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn a_read_of_a_setting_its_task_lacks_prints_and_fails_its_targets() {
    let rendered = thin(fixtures::MISREAD);
    assert_eq!(
        String::from_utf8_lossy(&rendered.stdout),
        "tasks/motd/task.json:1:34: motd: reads core.domian, which core does not declare; core declares domain\n"
    );
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn a_default_without_its_type_prints_and_fails_its_targets() {
    let rendered = thin(&[fixtures::MISTYPED]);
    assert_eq!(String::from_utf8_lossy(&rendered.stdout), "tasks/motd/task.json:1:54: motd: not an int\n");
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn hearth_render_prints_256_errors_then_the_count() {
    let rendered = render(&fixtures::broken_env_files(BROKEN_ENVIRONMENTS), Form::Text);
    let stdout = fixtures::lines_starting(&rendered.stdout, &["envs/", "44 "]);
    let mut lines = stdout.lines();
    assert_eq!(lines.next(), Some("envs/e000.json:1:14: duplicate key \"motd\" (envs/e000.json:1:2)"));
    assert_eq!(lines.next_back(), Some("44 more errors"));
    assert_eq!(lines.next_back(), Some("envs/e255.json:1:14: duplicate key \"motd\" (envs/e255.json:1:2)"));
    assert_eq!(lines.count(), 254);
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn hearth_render_json_gives_left_out_44() {
    let rendered = render(&fixtures::broken_env_files(BROKEN_ENVIRONMENTS), Form::JsonLines);
    assert_eq!(
        String::from_utf8_lossy(&rendered.stdout).lines().last(),
        Some("{\"record\":\"outcome\",\"outcome\":\"failed\",\"left_out\":44}")
    );
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn a_stray_file_in_a_task_prints_a_repository_error() {
    let rendered = thin(&[fixtures::STRAY]);
    assert_eq!(
        String::from_utf8_lossy(&rendered.stdout),
        "tasks/motd/tre/etc/motd: motd: a path protocol/format/README.md 2.1 does not allow\n"
    );
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn a_template_that_does_not_compile_prints_its_decode_error_at_its_line() {
    let rendered = thin(&[fixtures::BROKEN_TEMPLATE]);
    assert_eq!(
        String::from_utf8_lossy(&rendered.stdout),
        "tasks/motd/tree/etc/issue.j2:2:1: motd: a tag with no end\n"
    );
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn an_account_s_faults_print_at_their_lines_in_the_base() {
    let rendered = thin(fixtures::USERS);
    assert_eq!(
        String::from_utf8_lossy(&rendered.stdout),
        concat!(
            "hosts/x1/x1.json:2:16: x1-prod: users: not an int\n",
            "hosts/x1/x1.json:3:22: x1-prod: users: undeclared field \"shel\"; the object declares groups, password, shell, uid\n",
            "hosts/x1/x1.json:4:7: x1-prod: users: missing required field uid\n",
        )
    );
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn a_container_on_another_environment_s_target_prints_its_error() {
    let rendered = render(fixtures::CONTAINERS, Form::Text);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["target ", "hosts/"]),
        concat!(
            "target       east-lab   base east   environment lab\n",
            "target       east-prod   base east   environment prod\n",
            "hosts/chat/chat-prod.json:1:43: chat-prod: placement's on names east-lab, a target of lab\n",
        )
    );
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn a_missing_key_in_a_template_prints_its_evaluation_error() {
    let rendered = thin(&[fixtures::MISSING_TEMPLATE_KEY]);
    assert_eq!(rendered.stdout, b"tasks/motd/tree/etc/issue.j2:1:11: x1-prod: motd: a missing key\n");
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn a_missing_file_and_target_print_where_the_base_sets_them() {
    let rendered = thin(fixtures::REFERENCES);
    assert_eq!(
        String::from_utf8_lossy(&rendered.stdout),
        concat!(
            "hosts/x1/x1.json:1:28: x1-prod: net: no file lab-ca/lab.crt at this revision\n",
            "hosts/x1/x1.json:1:54: x1-prod: net: no target x9-prod in prod\n",
        )
    );
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn a_target_past_its_rendered_bytes_prints_a_limit_error() {
    let files = fixtures::thin_with(fixtures::TEMPLATED);
    let scenario = Scenario { files: &files, ..fixtures::thin() };
    let rendered = protocol::with_limits(&scenario, fixtures::SIMULATION_SEED, &fixtures::target_output_limit());
    assert_eq!(rendered.stdout, b"tasks/motd/tree/etc/issue.j2:1:39: x1-prod: motd: a limit exceeded\n");
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn a_false_guard_prints_its_message_at_its_key_and_leaves_its_target_out() {
    let rendered = thin(&[fixtures::GUARDED]);
    assert_eq!(
        rendered.stdout,
        b"tasks/motd/task.json:3:5: x1-prod: motd: requires greeting: set a greeting of your own\n"
    );
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn a_target_with_a_schema_fault_and_a_false_guard_prints_the_schema_fault_only() {
    let rendered = thin(&[fixtures::GUARDED, fixtures::WRONG_GREETING]);
    assert_eq!(rendered.stdout, b"hosts/x1/x1.json:1:21: x1-prod: motd: not a string\n");
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn a_for_each_making_a_key_twice_prints_a_collision_at_its_entry() {
    let rendered = thin(&[fixtures::TWICE]);
    assert_eq!(
        rendered.stdout,
        b"tasks/motd/task.json:3:5: x1-prod: motd: the key \"ops\" is made twice (tasks/motd/task.json:3:5)\n"
    );
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn an_undefined_value_beside_text_fails_while_a_lone_tag_leaves_the_field_absent() {
    let rendered = thin(&[fixtures::UNSET]);
    assert_eq!(rendered.stdout, b"tasks/motd/task.json:4:21: x1-prod: motd: an undefined value used\n");
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn a_boolean_in_a_string_field_prints_an_evaluation_error_at_the_field() {
    let rendered = thin(&[fixtures::BOOLEAN_FIELD]);
    assert_eq!(rendered.stdout, b"tasks/motd/task.json:3:21: x1-prod: motd: a value of the wrong type\n");
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn a_typo_in_a_settings_reference_prints_the_declared_names() {
    let rendered = thin(&[fixtures::TYPO]);
    assert_eq!(
        rendered.stdout,
        b"tasks/motd/task.json:3:21: motd: undeclared setting \"greting\"; motd declares greeting\n"
    );
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn a_typo_in_a_branch_no_run_takes_prints_at_its_line() {
    let rendered = thin(fixtures::TYPO_BRANCH);
    assert_eq!(
        rendered.stdout,
        b"tasks/motd/tree/etc/issue.j2:1:27: motd: undeclared setting \"greting\"; motd declares greeting\n"
    );
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn a_debian_the_schema_refuses_prints_the_layer_that_set_it() {
    let rendered = thin(&[fixtures::REFUSED_DEBIAN]);
    assert_eq!(
        String::from_utf8_lossy(&rendered.stdout),
        "hosts/x1/x1.json:1:43: x1-prod: debian.components must keep main on\n"
    );
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn a_secret_without_a_path_prints_its_fault_at_the_entry() {
    let rendered = thin(&[fixtures::SECRET_NO_PATH]);
    assert_eq!(rendered.stdout, b"tasks/motd/task.json:1:14: x1-prod: motd: missing required field path\n");
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn a_component_debian_does_not_enable_prints_the_package_and_the_component() {
    let rendered = render(&fixtures::packages_files(false), Form::Text);
    assert_eq!(rendered.stdout, b"tasks/base/task.json:1:60: x1-prod: base: firmware-linux needs component non-free-firmware, which debian.components does not enable\n");
    assert_eq!(rendered.code, Some(2));
}
