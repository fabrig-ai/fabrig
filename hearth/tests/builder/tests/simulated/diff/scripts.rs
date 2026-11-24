use hearth_builder_world::{
    fixtures::{self, scripts},
    protocol,
};

#[test]
fn hearth_diff_names_a_preset_a_tmpfiles_line_and_a_script() {
    let files = fixtures::thin_with(scripts::PRESETS);
    let later = fixtures::edited(&files, scripts::PRESETS_MOVED, &[]);
    let run = protocol::run(&fixtures::diffing(&files, &later), fixtures::SIMULATION_SEED);
    assert_eq!(
        fixtures::lines_starting(&run.stdout, &["  config "]),
        concat!(
            "  config   ~ presets ssh.service                    disable\n",
            "  config   ~ tmpfiles /var/log/journal              d 0755 root systemd-journal\n",
            "  config   ~ scripts                                motd 300 4b1f03482366\n",
        )
    );
    assert_eq!(run.code, Some(1));
}
