use hearth::Form;
use hearth_builder_world::fixtures::{self, Repository, accounts};
use skein_io::kernel::Exit;
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
#[test]
fn hearth_diff_names_each_account_group_and_member_moved() {
    assert_eq!(
        fixtures::lines_starting(&accounts::diff_accounts().stdout, &["  config "]),
        concat!(
            "  config   - groups ops                             1500\n",
            "  config   ~ users alice                            1001 alice /home/alice /bin/bash\n",
            "  config   - members alice ops\n",
            "  config   + members alice video\n"
        )
    );
}
#[test]
fn moved_accounts_exit_with_a_difference() {
    assert_eq!(accounts::diff_accounts().exit, Exit::Code(1));
}
