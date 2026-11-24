use hearth_builder_world::fixtures::{lines_starting, software};

#[test]
fn hearth_render_lists_downloads_and_the_software_script() {
    let rendered = super::render(software::SOFTWARE);
    assert_eq!(
        lines_starting(&rendered.stdout, &["  download ", "  scripts ", "  fetch "]),
        concat!(
            "  download   /opt/forgejo/bin/forgejo file 0755 root:root 7b36e8a38247\n",
            "  download   /opt/ripgrep tar 1 root:root 33165223a0a3\n",
            "  scripts    motd 500 1632d8753eb0\n",
            "  fetch      33165223a0a3 https://github.com/BurntSushi/ripgrep/releases/download/15.1.0/ripgrep-15.1.0-x86_64-unknown-linux-musl.tar.gz\n",
            "  fetch      7b36e8a38247 https://codeberg.org/forgejo/forgejo/releases/download/v15.0.0/forgejo-15.0.0-linux-amd64\n"
        )
    );
}

#[test]
fn a_path_outside_opt_prints_its_fault_at_the_path_key_and_exits_2() {
    let rendered = super::render(&[software::SOFTWARE_OUT]);
    assert_eq!(
        String::from_utf8(rendered.stdout).expect("text"),
        "tasks/motd/task.json:1:220: x1-prod: motd: path must be beneath /opt, /usr/local\n"
    );
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn one_task_keeps_its_software_and_customization_scripts_in_their_parts() {
    let rendered = super::render(software::BOTH_SCRIPTS);
    assert_eq!(
        lines_starting(&rendered.stdout, &["  scripts "]),
        concat!("  scripts    motd 500 1632d8753eb0\n", "  scripts    motd 300 b40d08aa8c5c\n",)
    );
}
