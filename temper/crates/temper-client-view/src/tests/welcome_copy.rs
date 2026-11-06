//! The welcome’s command Copy (temper/docs/design/client/README.md, 3.5).
use super::{fixtures as f, welcome};
use crate::look;
use skein_ui::{Act, Role};
use temper_client_domain::{self as d, copy::Written};

#[expect(clippy::disallowed_methods, reason = "semantic assertions locate the command’s subtree")]
fn command(tree: &skein_ui::Tree) -> Box<[u8]> {
    let index = tree
        .nodes()
        .iter()
        .position(|n| n.role == Role::Code && n.styles.as_slice().contains(&look::WELCOME_CMD))
        .unwrap();
    f::node_text(tree, index)
}

#[test]
#[expect(clippy::disallowed_methods, reason = "semantic assertions inspect the browser and terminal trees")]
fn only_the_browser_wraps_the_exact_command_with_a_quiet_copy_button() {
    let (mut root, _) = welcome::model_with_forge(d::WayKind::Program);
    super::connect_model::answer_environment(&mut root, super::connect_model::ENVIRONMENT);
    let mut browser = f::browser_page_view(&root);
    let terminal = f::page_view(&root);
    let expected = command(terminal.tree());
    assert!(!expected.ends_with(b"\n"));
    assert_eq!(command(browser.tree()), expected);
    let tree = browser.tree();
    let slot = tree.nodes().iter().find(|n| n.styles.as_slice().contains(&look::WELCOME_CMD_BOX)).unwrap();
    assert_eq!(slot.role, Role::Slot);
    let code = tree.nodes().iter().find(|n| n.role == Role::Code).unwrap();
    let button = tree.nodes().iter().find(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Copy")).unwrap();
    assert_eq!(super::connect_forge::parent(tree, code.id), Some(slot.id));
    assert_eq!(super::connect_forge::parent(tree, button.id), Some(slot.id));
    assert!(button.styles.as_slice().contains(&look::FLOW_QUIET));
    assert!(button.styles.as_slice().contains(&look::WELCOME_COPY));
    let id = button.id;
    assert_eq!(crate::act(&mut browser, id, Act::Press), Some(d::Action::Copy { words: expected }));
    assert!(
        !terminal.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::WELCOME_CMD_BOX)
            || n.styles.as_slice().contains(&look::WELCOME_COPY))
    );
}

#[test]
fn clipboard_outcomes_change_the_button_for_two_seconds_and_leave_the_command_selectable() {
    for (written, label) in
        [(Written::Done, "Copied"), (Written::Refused, "Couldn’t copy"), (Written::Failed, "Couldn’t copy")]
    {
        let (mut root, _) = welcome::model_with_forge(d::WayKind::Program);
        let mut view = f::browser_page_view(&root);
        let expected = command(view.tree());
        let id = f::find(view.tree(), Role::Button, b"Copy").unwrap();
        let event = crate::act_event(&mut view, id, Act::Press).unwrap();
        f::root_event(&mut root, event);
        f::root_event(&mut root, d::Event::Copied { token: 1, written });
        f::build(&mut view, &root);
        assert!(f::find(view.tree(), Role::Button, label.as_bytes()).is_some());
        assert_eq!(command(view.tree()), expected);
        let mut out = skein_lib::Queue::uncapped();
        for (now, name) in [(1_999_999_999, label.as_bytes()), (2_000_000_000, b"Copy")] {
            d::fire(
                &mut root,
                &skein_lib::Env { now: skein_lib::Time::from_nanos(now), wall: f::NOW, limits: f::limits() },
                &mut out,
            );
            f::build(&mut view, &root);
            assert!(f::find(view.tree(), Role::Button, name).is_some());
            assert_eq!(command(view.tree()), expected);
        }
    }
}
