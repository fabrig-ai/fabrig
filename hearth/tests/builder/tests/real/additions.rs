use hearth::Form;
use hearth_builder_world::fixtures::{self, Repository, additions, data};
#[test]
fn hearth_render_lists_subuid_and_subgid() {
    let repository = Repository::make("subordinate", &fixtures::thin_with(additions::SUBORDINATE));
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["  /etc/sub"]),
        concat!(
            "  /etc/subgid              file 0644 root:root a1c563a310c6\n",
            "  /etc/subuid              file 0644 root:root a1c563a310c6\n"
        )
    );
}
#[test]
fn hearth_render_lists_a_d_line_for_each_data_entry() {
    let repository = Repository::make("data-lines", &fixtures::thin_with(&[data::DATA]));
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["  tmpfiles "]),
        concat!(
            "  tmpfiles   /var/lib/iwd d 0700 root root\n",
            "  tmpfiles   /var/log/journal d 2755 root systemd-journal\n"
        )
    );
}
#[test]
fn the_initramfs_configuration_enters_the_cache() {
    let (_repository, path) = additions::cached_initramfs();
    assert_eq!(
        std::fs::read(path).expect("cached initramfs"),
        b"DEBUG=no\nAUTO_MODULES=no\nMICROCODE=generic\nINCLUDE_MODULES=btrfs,nvme\n"
    );
}
#[test]
fn the_cached_initramfs_has_mode_0644() {
    use std::os::unix::fs::PermissionsExt;
    let (_repository, path) = additions::cached_initramfs();
    assert_eq!(std::fs::metadata(path).expect("cached initramfs").permissions().mode() & 0o7777, 0o644);
}
#[test]
fn json_gives_the_initramfs_digest_beside_the_packages_digest() {
    let repository = additions::preseeded();
    let rendered = fixtures::render(&repository, &[], Form::JsonLines);
    let stdout = std::str::from_utf8(&rendered.stdout).expect("JSON text");
    let line = stdout.lines().find(|line| line.contains(r#""name":"packages""#)).expect("packages part");
    assert!(line.contains(
        r#""initramfs_file":"6cb7c9647dfa51a50f2cb619e3644f84e0f908044fb1aeb9187232a98528caa8","packages":{"#
    ));
}
