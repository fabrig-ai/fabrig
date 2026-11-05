//! The sheet reached through the domain's acts (client/README.md, 5.14).
use super::fixtures as f;
use crate::{frame::Bindings, look, words::Words};
use jig_client::contract as c;
use skein_lib::{Env, List, Queue, Time};
use skein_ui::{Builder, Role, Shortcut, State, Tree};
use temper_client_domain::{Action, Address, Domain, Event, pages::Act};

const PATH: &[u8] = b"docs/design/draft/away-mode.md";
const CHAT: u64 = 4;
const FIRST: u32 = 2;
const SECOND: u32 = 4;
const CUSTOM: u32 = 9;
const TEXT: u32 = 256;

struct Sheet {
    root: Domain,
    limits: temper_client_domain::Limits,
}
impl Sheet {
    fn new() -> Self {
        let mut limits = f::domain_limits();
        limits.client.domain.text = TEXT;
        limits.client.domain.turn_text = TEXT;
        let (mut root, watch, _) = f::root_at(Address::Home { project: f::PROJECT }, &limits);
        let source = f::playground_root(Address::Home { project: f::PROJECT }, f::PROJECT, false);
        let mut me = source.pages().me(source.client()).unwrap().clone();
        let member = me.projects.get_mut(0).unwrap();
        let stock = super::goals::template(FIRST, b"goal", true);
        member.templates = List::with_capacity(4);
        for (number, name, tracked) in [
            (SECOND, b"goal for a draft".as_slice(), true),
            (7, b"chat", false),
            (FIRST, b"goal", true),
            (CUSTOM, b"Custom way", true),
        ] {
            let mut template = stock.clone();
            template.number = number;
            template.name = c::DocumentText::complete(c::Text::new(name.into(), TEXT).unwrap());
            template.tracked = if tracked { Some(1) } else { None };
            member.templates.push(template).unwrap();
        }
        f::deliver_me(&mut root, watch, me);
        Self { root, limits }
    }
    fn act(&mut self, act: Act) {
        temper_client_domain::step(
            &mut self.root,
            &Env { now: Time::ZERO, wall: f::NOW, limits: self.limits },
            Event::Act { action: Action::Pages(act) },
            &mut Queue::uncapped(),
        );
    }
    fn open() -> Self {
        let mut sheet = Self::new();
        sheet.act(Act::OpenStart { draft: Some(PATH.into()), chat: Some(CHAT) });
        sheet
    }
    fn tree(&self, words: &Words) -> (Tree, Bindings) {
        let mut b = Builder::new(&f::tree_limits());
        let mut bindings = Bindings::new(f::tree_limits().bindings);
        if let Some(start) = self.root.pages().start() {
            crate::pages::start_goal::sheet(&mut b, &self.root, start, words, &mut bindings);
        }
        (b.finish(), bindings)
    }
}
fn node<'a>(tree: &'a Tree, role: Role, name: &[u8]) -> &'a skein_ui::Node {
    for node in tree.nodes() {
        if node.role == role && node.name.as_deref() == Some(name) {
            return node;
        }
    }
    panic!("named node: {role:?} {name:?}")
}
fn press(tree: &Tree, bindings: &Bindings, role: Role, name: &[u8]) -> Action {
    bindings.decode(node(tree, role, name).binding.unwrap().token, skein_ui::Act::Press).unwrap()
}

#[test]
fn no_draft_opens_no_sheet() {
    let mut sheet = Sheet::new();
    sheet.act(Act::OpenStart { draft: None, chat: None });
    assert!(sheet.root.pages().start().is_none());
    assert!(sheet.tree(&f::browser_words()).0.nodes().is_empty());
}
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
        if node.role == Role::Choice {
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
#[test]
fn the_default_is_chosen_and_says_so() {
    let mut sheet = Sheet::open();
    assert_eq!(sheet.root.pages().start().unwrap().way, Some(FIRST));
    for chosen in [FIRST, SECOND] {
        sheet.act(Act::StartWay { template: chosen });
        let (tree, bindings) = sheet.tree(&f::browser_words());
        let mut selected = 0_u32;
        for choice in tree.nodes() {
            if choice.role != Role::Choice {
                continue;
            }
            let act = bindings.decode(choice.binding.unwrap().token, skein_ui::Act::Toggle).unwrap();
            if choice.states.as_slice().contains(&State::Selected) {
                selected += 1;
                assert_eq!(act, Action::Pages(Act::StartWay { template: chosen }));
            }
        }
        assert_eq!(selected, 1);
        assert_eq!(skein_lib::bytes::count(&f::text(&tree), "Shop’s default.".as_bytes(), 2), 1);
        let mut ordinal = 0_u32;
        for (index, node) in tree.nodes().iter().enumerate() {
            if node.role == Role::Slot && node.styles.as_slice().contains(&look::SHEET_OPTION) {
                let detail = f::node_text(&tree, index);
                assert_eq!(detail.ends_with("Shop’s default.".as_bytes()), ordinal == 0);
                ordinal += 1;
            }
        }
    }
}
#[test]
fn no_budget_and_no_asks() {
    let sheet = Sheet::open();
    let (tree, _) = sheet.tree(&f::browser_words());
    for forbidden in [b"Budget".as_slice(), b"When it needs you", b"Draft"] {
        assert!(skein_lib::bytes::find(&f::text(&tree), forbidden).is_none());
        for node in tree.nodes() {
            assert_ne!(node.name.as_deref(), Some(forbidden));
        }
    }
}
#[test]
fn start_is_disabled_while_empty_or_out() {
    let mut sheet = Sheet::open();
    for name in [b"".as_slice(), b"   ", b"A goal"] {
        sheet.act(Act::StartName { words: name.into() });
        let (tree, bindings) = sheet.tree(&f::browser_words());
        assert_eq!(
            node(&tree, Role::Button, b"Start goal").states.as_slice().contains(&State::Disabled),
            name.trim_ascii().is_empty()
        );
        assert_eq!(press(&tree, &bindings, Role::Button, b"Start goal"), Action::Pages(Act::StartGoal));
    }
    sheet.act(Act::StartGoal);
    assert!(sheet.root.pages().start().unwrap().pending.is_some());
    let (tree, _) = sheet.tree(&f::browser_words());
    assert!(node(&tree, Role::Button, b"Start goal").states.as_slice().contains(&State::Disabled));
    assert!(node(&tree, Role::TextField, b"Name").states.as_slice().contains(&State::Disabled));
}
#[test]
fn cancel_carries_escape() {
    let mut sheet = Sheet::open();
    let (tree, bindings) = sheet.tree(&f::browser_words());
    assert_eq!(node(&tree, Role::Button, b"Cancel").shortcut, Some(Shortcut::ESCAPE));
    assert_eq!(press(&tree, &bindings, Role::Button, b"Cancel"), Action::Pages(Act::StartCancel));
    sheet.act(Act::StartCancel);
    assert!(sheet.root.pages().start().is_none());
}
#[test]
fn the_error_is_jigs_words() {
    let mut sheet = Sheet::open();
    // Removing the creating offer makes the chosen template NotOffered.
    let (mut root, watch, _) = f::root_at(Address::Home { project: f::PROJECT }, &sheet.limits);
    let mut me = sheet.root.pages().me(sheet.root.client()).unwrap().clone();
    me.projects.get_mut(0).unwrap().creating = None;
    f::deliver_me(&mut root, watch, me);
    sheet.root = root;
    sheet.act(Act::OpenStart { draft: Some(PATH.into()), chat: Some(CHAT) });
    sheet.act(Act::StartGoal);
    assert!(sheet.root.pages().start().unwrap().error.is_some());
    let words = f::browser_words();
    let (tree, _) = sheet.tree(&words);
    let mut has_error = false;
    for node in tree.nodes() {
        has_error |= node.styles.as_slice().contains(&look::SHEET_ERROR);
    }
    assert!(has_error);
    let mut expected = Builder::new(&f::tree_limits());
    words.refusal(&mut expected, &c::Refusal::NotOffered, TEXT);
    let expected = f::text(&expected.finish());
    assert!(skein_lib::bytes::find(&f::text(&tree), &expected).is_some());
}
#[test]
fn its_settings_link() {
    let sheet = Sheet::open();
    let (tree, bindings) = sheet.tree(&f::browser_words());
    let name = "The project’s ways of working".as_bytes();
    let link = node(&tree, Role::Link, name);
    assert!(link.styles.as_slice().contains(&look::SHEET_LINK));
    assert_eq!(
        press(&tree, &bindings, Role::Link, name),
        Action::Pages(Act::Go { address: Address::Settings { project: f::PROJECT } })
    );
}
