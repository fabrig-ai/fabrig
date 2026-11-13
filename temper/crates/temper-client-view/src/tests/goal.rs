use super::fixtures::{self, task as f};


use jig_client::contract as c;


use skein_ui::Role;

use temper_client_domain::pages::Act;


pub(super) use super::fixtures::goal::{
    offers, paused_document, press, with_goal_answer,
};

#[test]
fn an_agents_close_uses_the_same_sheet() {
    let mut task = f::delegate();
    task.acts = offers(&[c::ActKind::Cancel]);
    let mut root = f::agent_root(task, Some(f::tree()), true);
    press(&mut root, Act::Close);
    let view = fixtures::browser_page_view(&root);
    assert!(fixtures::find(view.tree(), Role::Dialog, b"Close this task?").is_some());
    assert!(
        skein_lib::bytes::find(
            &fixtures::text(view.tree()),
            "You can still read it, but it can’t be reopened.".as_bytes()
        )
        .is_some()
    );
    for name in [b"Keep it".as_slice(), b"Close task"] {
        assert!(fixtures::find(view.tree(), Role::Button, name).is_some());
    }
}
