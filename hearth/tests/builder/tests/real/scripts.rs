use hearth::Form;
use hearth_builder_world::fixtures::{self, Repository, scripts};
#[test]
fn hearth_render_lists_presets_tmpfiles_and_scripts() {
    let repository = Repository::make("scripts", &fixtures::thin_with(scripts::PRESETS));
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["  presets ", "  tmpfiles ", "  scripts "]),
        concat!(
            "  presets    getty@tty1.service disable\n",
            "  presets    ssh.service enable\n",
            "  tmpfiles   /var/log/journal d 2755 root systemd-journal\n",
            "  scripts    motd 300 b40d08aa8c5c\n",
        )
    );
}
