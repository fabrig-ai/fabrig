use hearth_builder_world::fixtures::{files, lines_starting};

#[test]
fn hearth_render_lists_a_link_a_directory_and_a_files_new_mode() {
    let rendered = super::render(files::LINKS);
    assert_eq!(
        lines_starting(&rendered.stdout, &["  /"]),
        concat!(
            "  /etc/motd                file 0600 root:adm 775324fcea7f\n",
            "  /etc/mtab                link root:root /proc/self/mounts\n",
            "  /usr/local/bin/motd      file 0755 root:root f487088ec8b1\n",
            "  /var/lib/motd            directory 0750 root:root\n"
        )
    );
}

#[test]
fn hearth_render_lists_rendered_and_copied_files_by_their_sha256() {
    let rendered = super::render(files::FILES);
    assert_eq!(
        lines_starting(&rendered.stdout, &["  /"]),
        concat!(
            "  /etc/motd                file 0644 root:root 775324fcea7f\n",
            "  /etc/motd.d/dev          file 0640 root:root b0b0dd8b10a8\n",
            "  /etc/motd.d/ops          file 0640 root:root e55838f39d99\n",
            "  /etc/ssl/lab.crt         file 0644 root:root 4cb3bd448645\n",
            "  /usr/local/bin/motd      file 0755 root:root f487088ec8b1\n"
        )
    );
}
