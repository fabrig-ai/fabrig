use hearth_builder_world::fixtures::{data, lines_starting};

#[test]
fn hearth_render_lists_the_data_entries_and_their_tmpfiles_lines() {
    let rendered = super::render(&[data::DATA]);
    assert_eq!(
        lines_starting(&rendered.stdout, &["  iwd ", "  journal "]),
        "  iwd        /var/lib/iwd\n  journal    /var/log/journal\n"
    );
    assert_eq!(
        lines_starting(&rendered.stdout, &["  tmpfiles "]),
        concat!(
            "  tmpfiles   /var/lib/iwd d 0700 root root\n",
            "  tmpfiles   /var/log/journal d 2755 root systemd-journal\n"
        )
    );
}
