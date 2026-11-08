use crate::{Command, Key, command};
use skein_lib::{Queue, Token, bytes};
use skein_ui::{Act, Binding, Builder, Limits, Modifier, NodeId, Owner, Role, Shortcut, State, Tree};
use skein_ui_screen::Focus;

// Room for the nested forms, disabled select and nine interactive roles below.
const LIMITS: Limits =
    Limits { nodes: 32, depth: 8, text_per_node: 64, text_bytes: 512, bindings: 24, patches: 192, ..Limits::EMPTY };
fn binding() -> Binding {
    Binding { owner: Owner { number: 1 }, token: Token::new(1) }
}
fn shown(builder: Builder) -> Tree {
    let mut tree = builder.finish();
    let mut patches = Queue::with_capacity(LIMITS.patches);
    skein_ui::diff(&Tree::new(LIMITS.nodes), &mut tree, &LIMITS, &mut 1, &mut patches);
    tree
}
fn control(b: &mut Builder, role: Role, submit: bool, disabled: bool) {
    b.open(role);
    b.name(b"N");
    b.bind(binding());
    if role.is_text_field() {
        b.value("aéz".as_bytes(), 0);
        if submit {
            b.state(State::SubmitsOnEnter);
        }
    }
    if disabled {
        b.state(State::Disabled);
    }
    b.close();
}
fn single(role: Role, submit: bool, disabled: bool) -> Tree {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    control(&mut b, role, submit, disabled);
    b.close();
    shown(b)
}
fn focus(tree: &Tree, at: u32) -> Focus {
    Focus { node: Some(tree.get(at).expect("fixture node").id), caret: 3, scroll: 12, honoured: 7, ..Focus::default() }
}
fn input(value: &[u8], node: NodeId) -> Command {
    Command::Act { node, act: Act::Input { value: bytes::copy_of(value) } }
}
#[test]
fn enter_and_space_take_only_the_focused_roles_act() {
    for (role, enter, space) in [
        (Role::Button, Act::Press, Act::Press),
        (Role::Link, Act::Press, Act::Press),
        (Role::Tab, Act::Press, Act::Press),
        (Role::MenuItem, Act::Press, Act::Press),
        (Role::Option, Act::Press, Act::Press),
        (Role::Check, Act::Toggle, Act::Toggle),
        (Role::Choice, Act::Toggle, Act::Toggle),
        (Role::TextField, Act::Submit, Act::Input { value: bytes::copy_of("aé z".as_bytes()) }),
        (Role::TextArea, Act::Submit, Act::Input { value: bytes::copy_of("aé z".as_bytes()) }),
    ] {
        let tree = single(role, true, false);
        let node = tree.get(1).expect("control").id;
        for (key, act) in [(Key::Enter, enter), (Key::Char(' '), space)] {
            assert_eq!(
                command(&key, &tree, &mut focus(&tree, 1), &LIMITS),
                Command::Act { node, act },
                "{role:?}, {key:?}"
            );
        }
    }
    for role in [Role::TextField, Role::TextArea] {
        let tree = single(role, false, false);
        assert_eq!(command(&Key::Enter, &tree, &mut focus(&tree, 1), &LIMITS), Command::Nothing);
    }
}
#[test]
fn fields_edit_whole_values_and_move_by_characters() {
    for role in [Role::TextField, Role::TextArea] {
        let tree = single(role, false, false);
        let node = tree.get(1).expect("field").id;
        for (key, at, expected, caret) in [
            (Key::Char('🙂'), 3, input("aé🙂z".as_bytes(), node), 7),
            (Key::Paste(bytes::copy_of("🙂".as_bytes())), 3, input("aé🙂z".as_bytes(), node), 7),
            (Key::Backspace, 3, input(b"az", node), 1),
            (Key::Delete, 1, input(b"az", node), 1),
            (Key::Left, 3, Command::Moved, 1),
            (Key::Right, 1, Command::Moved, 3),
            (Key::Home, 3, Command::Moved, 0),
            (Key::End, 0, Command::Moved, 4),
            (Key::Backspace, 0, Command::Nothing, 0),
            (Key::Delete, 4, Command::Nothing, 4),
            (Key::Left, 0, Command::Nothing, 0),
            (Key::Right, 4, Command::Nothing, 4),
            (Key::Home, 0, Command::Nothing, 0),
            (Key::End, 4, Command::Nothing, 4),
        ] {
            let mut focus = focus(&tree, 1);
            focus.caret = at;
            assert_eq!(command(&key, &tree, &mut focus, &LIMITS), expected, "{role:?}, {key:?}, caret {at}");
            assert_eq!(focus.caret, caret, "{role:?}, {key:?}");
        }
    }
}
#[test]
fn field_pastes_flatten_breaks_and_drop_trailing_breaks() {
    for (role, text, expected) in [
        (Role::TextField, b"x\r\ny\nz\r\n\n".as_slice(), Some("aéx y zz".as_bytes())),
        (Role::TextField, b"\r\n", None),
        (Role::TextField, b"x \n", Some("aéx z".as_bytes())),
        (Role::TextArea, b"x\r\ny\n", Some("aéx\r\ny\nz".as_bytes())),
    ] {
        let tree = single(role, false, false);
        let mut focus = focus(&tree, 1);
        let node = focus.node.expect("field focus");
        let wanted = match expected {
            Some(value) => input(value, node),
            None => Command::Nothing,
        };
        assert_eq!(
            command(&Key::Paste(bytes::copy_of(text)), &tree, &mut focus, &LIMITS),
            wanted,
            "{role:?}, {text:?}"
        );
    }
}
#[test]
fn inputs_at_the_nodes_text_bound_hold_and_past_it_do_nothing() {
    let tree = single(Role::TextField, false, false);
    let node = tree.get(1).expect("field").id;
    // Name N is one byte; aéz four; inserting q takes the node to six.
    for (bound, expected, caret) in [(6, input("aéqz".as_bytes(), node), 4), (5, Command::Nothing, 3)] {
        let mut focus = focus(&tree, 1);
        let limits = Limits { text_per_node: bound, ..LIMITS };
        assert_eq!(command(&Key::Char('q'), &tree, &mut focus, &limits), expected);
        assert_eq!(focus.caret, caret);
    }
}
fn focus_tree() -> Tree {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    control(&mut b, Role::Button, false, false);
    control(&mut b, Role::Button, false, true);
    b.open(Role::Button);
    b.name(b"unbound");
    b.close();
    b.open(Role::Select);
    b.state(State::Disabled);
    control(&mut b, Role::Option, false, false);
    b.close();
    control(&mut b, Role::TextField, false, false);
    control(&mut b, Role::Link, false, false);
    control(&mut b, Role::TextArea, false, false);
    control(&mut b, Role::Choice, false, false);
    control(&mut b, Role::Check, false, false);
    control(&mut b, Role::Option, false, false);
    control(&mut b, Role::Tab, false, false);
    control(&mut b, Role::MenuItem, false, false);
    b.close();
    shown(b)
}
#[test]
fn focus_walks_enabled_bound_controls_and_wraps_both_ends() {
    let tree = focus_tree();
    for (key, order) in [
        (Key::Tab, &[1, 6, 7, 8, 9, 10, 11, 12, 13, 1][..]),
        (Key::Down, &[1, 6, 7, 8, 9, 10, 11, 12, 13, 1]),
        (Key::BackTab, &[13, 12, 11, 10, 9, 8, 7, 6, 1, 13]),
        (Key::Up, &[13, 12, 11, 10, 9, 8, 7, 6, 1, 13]),
    ] {
        let mut focus = Focus::default();
        for index in order {
            assert_eq!(command(&key, &tree, &mut focus, &LIMITS), Command::Moved, "{key:?}, index {index}");
            assert_eq!(focus.node, Some(tree.get(*index).expect("ordered fixture control").id));
            let expected_caret = if *index == 6 || *index == 8 { 4 } else { 0 };
            assert_eq!(focus.caret, expected_caret);
        }
    }
}
#[test]
#[expect(clippy::disallowed_types, reason = "the diagnostic reports every paging row")]
#[expect(clippy::disallowed_macros, reason = "failed paging rows share one diagnostic")]
#[expect(clippy::use_debug, reason = "failed rows report the key, command and focus")]
fn pages_scroll_and_control_c_closes_on_every_focus() {
    use core::fmt::Write;
    let mut failures = String::new();
    let tree = focus_tree();
    for (rows, key, start, following, end, follow, wanted) in [
        (0, Key::PageUp, 12, true, 11, false, Command::Moved),
        (0, Key::PageDown, 12, true, 13, true, Command::Moved),
        (1, Key::PageUp, 12, true, 11, false, Command::Moved),
        (1, Key::PageDown, 12, true, 13, true, Command::Moved),
        (10, Key::PageUp, 12, true, 3, false, Command::Moved),
        (10, Key::PageDown, 12, true, 21, true, Command::Moved),
        (10, Key::PageUp, 0, true, 0, false, Command::Moved),
    ] {
        let mut focus = Focus { scroll: start, rows, follow: following, ..Focus::default() };
        let got = command(&key, &tree, &mut focus, &LIMITS);
        if got != wanted || (focus.scroll, focus.follow) != (end, follow) {
            writeln!(failures, "{key:?} rows {rows} start {start}: {got:?} {focus:?}").expect("diagnostic");
        }
    }
    assert!(failures.is_empty(), "{failures}");
    for node in tree.nodes() {
        for (key, scroll, expected) in [
            (Key::PageUp, 3, Command::Moved),
            (Key::PageDown, 21, Command::Moved),
            (Key::Control(b'c'), 12, Command::Close),
        ] {
            let mut focus = Focus { node: Some(node.id), scroll: 12, rows: 10, ..Focus::default() };
            assert_eq!(command(&key, &tree, &mut focus, &LIMITS), expected, "{key:?}, {:?}", node.role);
            assert_eq!(focus.scroll, scroll);
        }
    }
    for (key, start, end, expected) in [
        (Key::PageUp, 4, 0, Command::Moved),
        (Key::PageUp, 0, 0, Command::Nothing),
        (Key::PageDown, u32::MAX, u32::MAX, Command::Nothing),
        (Key::Control(b'c'), 0, 0, Command::Close),
    ] {
        let mut focus = Focus { scroll: start, rows: 10, ..Focus::default() };
        assert_eq!(command(&key, &tree, &mut focus, &LIMITS), expected);
        assert_eq!(focus.scroll, end);
    }
}
fn shortcuts() -> Tree {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    for modifier in [Modifier::Plain, Modifier::Primary, Modifier::Alt] {
        for disabled in [true, false, false] {
            b.open(Role::Button);
            b.bind(binding());
            b.shortcut(Shortcut { key: b'x', modifier });
            if disabled {
                b.state(State::Disabled);
            }
            b.close();
        }
    }
    for role in [Role::TextField, Role::TextArea] {
        control(&mut b, role, false, false);
    }
    b.close();
    shown(b)
}
#[test]
fn shortcuts_choose_first_enabled_and_plain_never_presses_in_fields() {
    let tree = shortcuts();
    for (key, index) in [(Key::Char('x'), 2), (Key::Control(b'x'), 5), (Key::Alt('x'), 8)] {
        for focused in [None, Some(1), Some(10), Some(11)] {
            let mut focus = match focused {
                Some(at) => focus(&tree, at),
                None => Focus::default(),
            };
            let expected = if key == Key::Char('x') && (focused == Some(10) || focused == Some(11)) {
                input("aéxz".as_bytes(), focus.node.expect("field"))
            } else {
                Command::Act { node: tree.get(index).expect("shortcut target").id, act: Act::Press }
            };
            assert_eq!(command(&key, &tree, &mut focus, &LIMITS), expected, "{key:?}, {focused:?}");
        }
    }
    for key in [Key::Char('X'), Key::Char('!'), Key::Char('é'), Key::Control(b'z'), Key::Alt('X')] {
        assert_eq!(command(&key, &tree, &mut Focus::default(), &LIMITS), Command::Nothing, "{key:?}");
    }
}
fn cancel(b: &mut Builder, disabled: bool) {
    b.open(Role::Button);
    b.bind(binding());
    b.shortcut(Shortcut::ESCAPE);
    if disabled {
        b.state(State::Disabled);
    }
    b.close();
}
fn scopes(dialog: bool, inner_cancel: bool) -> Tree {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    b.open(Role::Form);
    cancel(&mut b, false);
    b.open(Role::Form);
    control(&mut b, Role::TextField, false, false);
    if inner_cancel {
        cancel(&mut b, false);
    }
    b.close();
    b.close();
    if dialog {
        b.open(Role::Dialog);
        cancel(&mut b, true);
        cancel(&mut b, false);
        b.close();
    }
    b.close();
    shown(b)
}
#[test]
fn escape_uses_dialog_then_innermost_form_without_falling_back() {
    for (dialog, inner, focused, target) in [
        (false, true, Some(4), Some(5)),
        (false, false, Some(4), None),
        (false, true, Some(2), Some(2)),
        (false, true, Some(0), None),
        (true, true, Some(4), Some(8)),
        (true, true, None, Some(8)),
    ] {
        let tree = scopes(dialog, inner);
        let mut focus = match focused {
            Some(at) => focus(&tree, at),
            None => Focus::default(),
        };
        let expected = match target {
            Some(at) => Command::Act { node: tree.get(at).expect("cancel button").id, act: Act::Press },
            None => Command::Nothing,
        };
        assert_eq!(
            command(&Key::Escape, &tree, &mut focus, &LIMITS),
            expected,
            "dialog {dialog}, inner {inner}, focus {focused:?}"
        );
    }
}
#[test]
fn disabled_and_noninteractive_focus_never_emit_acts() {
    for role in [Role::Button, Role::TextField, Role::TextArea, Role::Check, Role::Choice, Role::Option] {
        let tree = single(role, true, true);
        for key in [
            Key::Enter,
            Key::Char(' '),
            Key::Char('x'),
            Key::Paste(bytes::copy_of(b"x")),
            Key::Backspace,
            Key::Delete,
            Key::Left,
            Key::Right,
            Key::Home,
            Key::End,
        ] {
            assert_eq!(command(&key, &tree, &mut focus(&tree, 1), &LIMITS), Command::Nothing, "{role:?}, {key:?}");
        }
    }
    let tree = focus_tree();
    for at in [0, 3, 5] {
        for key in [
            Key::Enter,
            Key::Char(' '),
            Key::Paste(bytes::copy_of(b"x")),
            Key::Backspace,
            Key::Delete,
            Key::Left,
            Key::Right,
            Key::Home,
            Key::End,
        ] {
            assert_eq!(command(&key, &tree, &mut focus(&tree, at), &LIMITS), Command::Nothing, "{at}, {key:?}");
        }
    }
    let empty = Tree::new(0);
    for key in [
        Key::Tab,
        Key::BackTab,
        Key::Enter,
        Key::Char(' '),
        Key::Paste(bytes::copy_of(b"x")),
        Key::Backspace,
        Key::Left,
        Key::Escape,
    ] {
        assert_eq!(command(&key, &empty, &mut Focus::default(), &LIMITS), Command::Nothing, "{key:?}");
    }
}

fn options(role: Role, text: &[u8], disabled: bool) -> Tree {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    b.open(Role::Form);
    b.open(role);
    b.name(b"Field");
    b.value(text, 0);
    b.bind(binding());
    b.state(State::SubmitsOnEnter);
    b.close();
    b.open(Role::Menu);
    b.state(State::Options);
    for (name, disabled) in [(b"/model".as_slice(), disabled), (b"/new", true), (b"/help", disabled)] {
        b.open(Role::MenuItem);
        b.name(name);
        b.bind(binding());
        if disabled {
            b.state(State::Disabled);
        }
        b.close();
    }
    b.close();
    cancel(&mut b, false);
    b.close();
    b.close();
    shown(b)
}

#[test]
#[expect(clippy::disallowed_types, reason = "the diagnostic reports every options key row")]
#[expect(clippy::disallowed_macros, reason = "failed options rows share one diagnostic")]
#[expect(clippy::use_debug, reason = "failed rows report the key, command and focus")]
fn fields_options_move_press_complete_and_keep_escape() {
    use core::fmt::Write;
    let mut failures = String::new();
    for role in [Role::TextField, Role::TextArea] {
        let tree = options(role, b"/", false);
        let field = tree.get(2).expect("field").id;
        let first = tree.get(4).expect("first item").id;
        let last = tree.get(6).expect("last item").id;
        let cancel = tree.get(7).expect("cancel").id;
        for (key, active, wanted, next, caret) in [
            (Key::Down, first, Command::Moved, last, 1),
            (Key::Down, last, Command::Nothing, last, 1),
            (Key::Up, last, Command::Moved, first, 1),
            (Key::Up, first, Command::Nothing, first, 1),
            (Key::Enter, last, Command::Act { node: last, act: Act::Press }, last, 1),
            (Key::Tab, last, input(b"/help", field), last, 5),
            (Key::Escape, last, Command::Act { node: cancel, act: Act::Press }, last, 1),
        ] {
            let mut focus = Focus { node: Some(field), caret: 1, active: Some(active), ..Focus::default() };
            let got = command(&key, &tree, &mut focus, &LIMITS);
            if got != wanted || (focus.node, focus.active, focus.caret) != (Some(field), Some(next), caret) {
                writeln!(failures, "{role:?} {key:?} active {active:?}: {got:?} {focus:?}").expect("diagnostic");
            }
        }
        // Field's five name bytes plus /help's five bytes take ten bytes.
        for (bound, wanted, caret) in [(10, input(b"/help", field), 5), (9, Command::Nothing, 1)] {
            let mut focus = Focus { node: Some(field), caret: 1, active: Some(last), ..Focus::default() };
            let limits = Limits { text_per_node: bound, ..LIMITS };
            let got = command(&Key::Tab, &tree, &mut focus, &limits);
            if got != wanted || focus.caret != caret || focus.node != Some(field) {
                writeln!(failures, "{role:?} completion bound {bound}: {got:?} {focus:?}").expect("diagnostic");
            }
        }
        let tree = options(role, b"different", false);
        let mut focus = Focus {
            node: Some(tree.get(2).expect("field").id),
            active: Some(tree.get(6).expect("item").id),
            ..Focus::default()
        };
        if command(&Key::Tab, &tree, &mut focus, &LIMITS) != Command::Moved
            || focus.node != Some(tree.get(4).expect("first item").id)
            || focus.active.is_some()
        {
            writeln!(failures, "{role:?} unmatched Tab: {focus:?}").expect("diagnostic");
        }
        let tree = options(role, b"/", true);
        let field = tree.get(2).expect("field").id;
        let cancel = tree.get(7).expect("cancel").id;
        for (key, wanted, next) in [
            (Key::Down, Command::Moved, cancel),
            (Key::Up, Command::Moved, cancel),
            (Key::Tab, Command::Moved, cancel),
            (Key::Enter, Command::Act { node: field, act: Act::Submit }, field),
            (Key::Escape, Command::Act { node: cancel, act: Act::Press }, field),
        ] {
            let mut focus = Focus { node: Some(field), ..Focus::default() };
            let got = command(&key, &tree, &mut focus, &LIMITS);
            if got != wanted || focus.node != Some(next) || focus.active.is_some() {
                writeln!(failures, "{role:?} all disabled {key:?}: {got:?} {focus:?}").expect("diagnostic");
            }
        }
    }
    let tree = options_scopes();
    for (field, target) in
        [(b"Outer".as_slice(), b"Outer item".as_slice()), (b"Inner", b"Inner item"), (b"Nearest", b"Nearest item")]
    {
        let field = named(&tree, field);
        let target = named(&tree, target);
        let mut focus = Focus { node: Some(field), ..Focus::default() };
        let got = command(&Key::Enter, &tree, &mut focus, &LIMITS);
        if got != (Command::Act { node: target, act: Act::Press }) || focus.node != Some(field) {
            writeln!(failures, "scoped field {field:?}: {got:?} {focus:?}").expect("diagnostic");
        }
    }
    assert!(failures.is_empty(), "{failures}");
}

fn named(tree: &Tree, name: &[u8]) -> NodeId {
    for node in tree.nodes() {
        if node.name.as_deref() == Some(name) {
            return node.id;
        }
    }
    unreachable!("named fixture node exists")
}

fn options_scopes() -> Tree {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    b.open(Role::Form);
    scoped_field(&mut b, b"Outer");
    b.open(Role::Form);
    scoped_field(&mut b, b"Inner");
    scoped_menu(&mut b, b"Inner item");
    b.close();
    // The closed inner form's field does not belong to this menu.
    scoped_menu(&mut b, b"Outer item");
    scoped_field(&mut b, b"Nearest");
    scoped_menu(&mut b, b"Nearest item");
    b.close();
    b.close();
    shown(b)
}
fn scoped_field(b: &mut Builder, name: &[u8]) {
    b.open(Role::TextField);
    b.name(name);
    b.bind(binding());
    b.close();
}
fn scoped_menu(b: &mut Builder, name: &[u8]) {
    b.open(Role::Menu);
    b.state(State::Options);
    b.open(Role::MenuItem);
    b.name(name);
    b.bind(binding());
    b.close();
    b.close();
}
