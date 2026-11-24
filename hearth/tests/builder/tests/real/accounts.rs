use hearth::Form;
use hearth_builder_world::fixtures::{self, Repository, accounts};
#[test]
fn hearth_render_lists_groups_users_and_members() {
    let repository = Repository::make("accounts", &fixtures::thin_with(&[accounts::ACCOUNTS]));
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["  groups ", "  users ", "  members "]),
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
