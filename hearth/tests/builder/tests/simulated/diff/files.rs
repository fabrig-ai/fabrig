use hearth_builder_world::{
    fixtures::{self, files},
    protocol,
};

#[test]
fn hearth_diff_shows_a_moved_link_and_a_directory_s_new_mode() {
    let before = fixtures::thin_with(files::LINKS);
    let later = fixtures::edited(&before, files::LINKS_MOVED, &[]);
    let scenario = fixtures::diffing(&before, &later);
    let run = protocol::run(&scenario, fixtures::SIMULATION_SEED);
    assert_eq!(
        fixtures::lines_starting(&run.stdout, &["  config "]),
        concat!(
            "  config   ~ /etc/mtab                              link root:root /proc/mounts\n",
            "  config   ~ /var/lib/motd                          directory 0755 root:root\n"
        )
    );
    assert_eq!(run.code, Some(1));
}

#[test]
fn hearth_diff_shows_each_element_s_changed_output() {
    let before = fixtures::thin_with(files::FILES);
    let later = fixtures::edited(&before, files::FILES_HI, &[]);
    let scenario = fixtures::diffing(&before, &later);
    let run = protocol::run(&scenario, fixtures::SIMULATION_SEED);
    assert_eq!(
        fixtures::lines_starting(&run.stdout, &["  config "]),
        concat!(
            "  config   ~ /etc/motd.d/dev                        0640 root:root\n",
            "  config   ~ /etc/motd.d/ops                        0640 root:root\n"
        )
    );
    assert_eq!(run.code, Some(1));
}
