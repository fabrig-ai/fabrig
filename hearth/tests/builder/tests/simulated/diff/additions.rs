use hearth::Form;
use hearth_builder_world::{
    fixtures::{self, additions, data},
    protocol,
};

fn mode_diff(form: Form) -> protocol::Run {
    let files = fixtures::thin_with(&[data::DATA]);
    let later = fixtures::edited(&files, &[additions::DATA_MODE], &[]);
    let scenario = fixtures::Scenario { form, ..fixtures::diffing(&files, &later) };
    protocol::run(&scenario, fixtures::SIMULATION_SEED)
}

#[test]
fn hearth_diff_shows_a_data_entrys_new_mode_as_a_tmpfiles_change() {
    let run = mode_diff(Form::Text);
    assert_eq!(
        fixtures::lines_starting(&run.stdout, &["  config "]),
        "  config   ~ tmpfiles /var/log/journal              d 0755 root systemd-journal\n"
    );
    assert_eq!(run.code, Some(1));
}

#[test]
fn a_data_mode_changes_the_config_rebuild_key() {
    let run = mode_diff(Form::JsonLines);
    assert!(std::str::from_utf8(&run.stdout).expect("JSON text").contains(r#""kind":"from_config""#));
}
