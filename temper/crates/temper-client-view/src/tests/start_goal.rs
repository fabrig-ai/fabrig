//! The sheet reached through the domain's acts (client/pages/goal.md, 3).
use super::fixtures as f;

use crate::{frame::Bindings, look, words::Words};

use jig_client::contract as c;

use skein_lib::{Env, List, Queue, Time};

use skein_ui::{Builder, Role, Shortcut, State, Tree};

use temper_client_domain::{Action, Address, Domain, Event, pages::Act};
use crate::work_tests::start_goal::{CHAT, CUSTOM, FIRST, PATH, SECOND, Sheet, TEXT, node};

#[test]
fn a_draft_names_the_sheet_and_its_name() {
    let sheet = Sheet::open();
    for words in [f::words(), f::browser_words()] {
        let (tree, bindings) = sheet.tree(&words);
        let title = b"Start a goal from away-mode.md";
        let dialog = node(&tree, Role::Dialog, title);
        assert!(dialog.styles.as_slice().contains(&look::SHEET));
        node(&tree, Role::Heading { level: 2 }, title);
        let name = node(&tree, Role::TextField, b"Name");
        assert_eq!(name.value.as_ref().unwrap().text.as_ref(), b"away-mode");
        assert!(name.styles.as_slice().contains(&look::GOAL_TEXT));
        assert_eq!(
            bindings.decode(name.binding.unwrap().token, skein_ui::Act::Input { value: b"A name".as_slice().into() }),
            Some(Action::Pages(Act::StartName { words: b"A name".as_slice().into() }))
        );
        assert!(
            skein_lib::bytes::find(
                &f::text(&tree),
                b"The goal takes the draft as it is now. Later edits reach it through Amend."
            )
            .is_some()
        );
        assert!(f::find(&tree, Role::TextField, b"Draft").is_none());
        let mut view = f::view();
        let full = f::build(&mut view, &sheet.root);
        let mut main_end = None;
        let mut dialog_index = None;
        for (index, node) in full.nodes().iter().enumerate() {
            if node.role == Role::Main {
                main_end = Some(index + usize::try_from(node.size).unwrap());
            }
            if node.role == Role::Dialog {
                dialog_index = Some(index);
            }
        }
        assert!(dialog_index.unwrap() >= main_end.unwrap());
    }
}

#[test]
fn one_option_per_way_in_order() {
    let sheet = Sheet::open();
    let (tree, bindings) = sheet.tree(&f::browser_words());
    let group = node(&tree, Role::ChoiceGroup, b"How it runs");
    assert!(group.styles.as_slice().contains(&look::SHEET_FIELDSET));
    let mut choices = List::uncapped();
    let mut names = List::uncapped();
    for node in tree.nodes() {
        if node.role == Role::Choice
            && let Some(Action::Pages(Act::StartWay { .. })) =
                bindings.decode(node.binding.unwrap().token, skein_ui::Act::Toggle)
        {
            choices.push(node).unwrap();
            names.push(node.name.as_deref().unwrap()).unwrap();
        }
    }
    assert_eq!(names.as_slice(), [b"Task by task".as_slice(), b"Plan, tasks, land whole", b"Custom way"]);
    for (choice, number) in choices.iter().zip([FIRST, SECOND, CUSTOM]) {
        assert_eq!(
            bindings.decode(choice.binding.unwrap().token, skein_ui::Act::Toggle),
            Some(Action::Pages(Act::StartWay { template: number }))
        );
    }
    let mut view = f::view();
    f::build(&mut view, &sheet.root);
    let choice = node(view.tree(), Role::Choice, b"Plan, tasks, land whole").id;
    assert_eq!(
        crate::act(&mut view, choice, skein_ui::Act::Toggle),
        Some(Action::Pages(Act::StartWay { template: SECOND }))
    );
}
