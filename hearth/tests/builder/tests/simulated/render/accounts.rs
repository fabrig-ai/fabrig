use hearth_builder_world::fixtures::{accounts, lines_starting};

#[test]
fn hearth_render_lists_groups_users_and_members() {
    let rendered = super::render(&[accounts::ACCOUNTS]);
    assert_eq!(
        lines_starting(&rendered.stdout, &["  groups ", "  users ", "  members "]),
        concat!(
            "  groups     mail package\n",
            "  groups     ops 1500\n",
            "  users      alice 1000 alice /home/alice /bin/bash\n",
            "  users      ci - ci / /usr/sbin/nologin\n",
            "  users      postfix package\n",
            "  members    alice ops\n",
            "  members    alice sudo\n"
        )
    );
}
