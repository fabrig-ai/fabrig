use hearth::Form;
use hearth_builder_world::{
    fixtures::{self, additions, lines_starting},
    protocol,
};

#[test]
fn hearth_render_lists_subuid_and_subgid() {
    let rendered = super::render(additions::SUBORDINATE);
    assert_eq!(
        lines_starting(&rendered.stdout, &["  /etc/sub"]),
        concat!(
            "  /etc/subgid              file 0644 root:root a1c563a310c6\n",
            "  /etc/subuid              file 0644 root:root a1c563a310c6\n"
        )
    );
}

#[test]
fn the_initramfs_configuration_is_cached_and_printed_by_digest() {
    let files = fixtures::thin_with(fixtures::PRESEEDED);
    let scenario = fixtures::Scenario { files: &files, form: Form::JsonLines, ..fixtures::thin() };
    let rendered = protocol::run(&scenario, fixtures::SIMULATION_SEED);
    let digest = additions::INITRAMFS_SHA256_HEX;
    let path = [fixtures::STORE, b"/cache/", digest.as_bytes()].concat();
    assert_eq!(
        rendered.shared.contents(&path).expect("cached initramfs"),
        b"DEBUG=no\nAUTO_MODULES=no\nMICROCODE=generic\nINCLUDE_MODULES=btrfs,nvme\n"
    );
    let facts = rendered.shared.borrow().behind.machine().linux().facts(&path).expect("cached initramfs");
    assert_eq!(facts.mode & 0o7777, 0o644);

    let stdout = std::str::from_utf8(&rendered.stdout).expect("JSON text");
    let line = stdout.lines().find(|line| line.contains(r#""name":"packages""#)).expect("packages part");
    assert!(line.contains(&format!(r#""initramfs_file":"{digest}","packages":{{"#)));
}
