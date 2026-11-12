use super::{DRAWING, MAIN, button, drawing, end, main, page};
use crate::{
    mapping::Primary,
    page::{Event, Op, Page},
    tests::fixtures::{self, LIMITS},
};
use skein_ui::{Act, Builder, DrawingKind, Modifier, NodeId, Patch, Role, Run, Shortcut, State, View};

// Preorder: page, main, button, link, field, area, check, choice, dialog,
// disabled button, primary button, alt button, plain button, duplicate primary,
// unbound button, external link, menu item, menu.
const EVENT_BUTTON: NodeId = NodeId(3);
const EVENT_LINK: NodeId = NodeId(4);
const EVENT_FIELD: NodeId = NodeId(5);
const EVENT_AREA: NodeId = NodeId(6);
const EVENT_CHECK: NodeId = NodeId(7);
const EVENT_CHOICE: NodeId = NodeId(8);
const EVENT_DIALOG: NodeId = NodeId(9);
const EVENT_DISABLED: NodeId = NodeId(10);
const EVENT_PRIMARY: NodeId = NodeId(11);
const EVENT_ALT: NodeId = NodeId(12);
const EVENT_PLAIN: NodeId = NodeId(13);
const EVENT_UNBOUND: NodeId = NodeId(15);
const EVENT_EXTERNAL: NodeId = NodeId(16);
const EVENT_ITEM: NodeId = NodeId(17);
const EVENT_MENU: NodeId = NodeId(18);

fn events(builder: &mut Builder) {
    main(builder);
    button(builder, b"Press");
    builder.open(Role::Link);
    builder.link(fixtures::binding(2));
    builder.close();
    for role in [Role::TextField, Role::TextArea] {
        builder.open(role);
        builder.bind(fixtures::binding(3));
        builder.state(State::SubmitsOnEnter);
        builder.close();
    }
    builder.open(Role::Check);
    builder.bind(fixtures::binding(4));
    builder.state(State::Pressed);
    builder.close();
    builder.open(Role::Choice);
    builder.bind(fixtures::binding(5));
    builder.state(State::Selected);
    builder.close();
    builder.open(Role::Dialog);
    builder.close();
    builder.open(Role::Button);
    builder.bind(fixtures::binding(6));
    builder.state(State::Disabled);
    builder.close();
    for (key, modifier) in
        [(b'k', Modifier::Primary), (b'k', Modifier::Alt), (b'p', Modifier::Plain), (b'k', Modifier::Primary)]
    {
        builder.open(Role::Button);
        builder.bind(fixtures::binding(7));
        builder.shortcut(Shortcut { key, modifier });
        builder.close();
    }
    builder.open(Role::Button);
    builder.close();
    builder.open(Role::Link);
    builder.external(b"https://example.com/");
    builder.close();
    builder.open(Role::MenuItem);
    builder.bind(fixtures::binding(8));
    builder.close();
    builder.open(Role::Menu);
    builder.close();
    end(builder);
}

#[derive(Clone, Copy, Debug)]
enum KeyCase {
    Primary,
    Other,
    Shift,
    ControlAlt,
    AltComposed,
    Plain,
}
#[derive(Clone, Copy, Debug)]
enum Case {
    Click(NodeId, bool),
    Input(bool),
    CompositionEnd,
    Enter(NodeId, bool, bool),
    Change(NodeId),
    Cancel,
    Key(KeyCase),
}
#[derive(Clone, Copy)]
enum Expected {
    None,
    Press(NodeId),
    Input,
    Submit(NodeId),
    Toggle(NodeId),
}

const EVENTS: &[(Primary, Case, Expected)] = &[
    (Primary::Control, Case::Click(EVENT_BUTTON, false), Expected::Press(EVENT_BUTTON)),
    (Primary::Control, Case::Click(EVENT_LINK, false), Expected::Press(EVENT_LINK)),
    (Primary::Control, Case::Click(EVENT_LINK, true), Expected::None),
    (Primary::Control, Case::Click(EVENT_DISABLED, false), Expected::None),
    (Primary::Control, Case::Click(EVENT_UNBOUND, false), Expected::None),
    (Primary::Control, Case::Click(EVENT_EXTERNAL, false), Expected::None),
    (Primary::Control, Case::Click(EVENT_ITEM, false), Expected::Press(EVENT_ITEM)),
    (Primary::Control, Case::Click(EVENT_MENU, false), Expected::None),
    (Primary::Control, Case::Input(false), Expected::Input),
    (Primary::Control, Case::Input(true), Expected::None),
    (Primary::Control, Case::Enter(EVENT_FIELD, false, false), Expected::Submit(EVENT_FIELD)),
    (Primary::Control, Case::Enter(EVENT_AREA, false, false), Expected::Submit(EVENT_AREA)),
    (Primary::Control, Case::Enter(EVENT_AREA, true, false), Expected::None),
    (Primary::Control, Case::Enter(EVENT_FIELD, false, true), Expected::None),
    (Primary::Control, Case::Change(EVENT_CHECK), Expected::Toggle(EVENT_CHECK)),
    (Primary::Control, Case::Change(EVENT_CHOICE), Expected::Toggle(EVENT_CHOICE)),
    (Primary::Control, Case::Cancel, Expected::None),
    (Primary::Control, Case::Key(KeyCase::Primary), Expected::Press(EVENT_PRIMARY)),
    (Primary::Meta, Case::Key(KeyCase::Primary), Expected::Press(EVENT_PRIMARY)),
    (Primary::Control, Case::Key(KeyCase::Other), Expected::None),
    (Primary::Meta, Case::Key(KeyCase::Other), Expected::None),
    (Primary::Control, Case::Key(KeyCase::Shift), Expected::None),
    (Primary::Meta, Case::Key(KeyCase::Shift), Expected::None),
    (Primary::Control, Case::Key(KeyCase::ControlAlt), Expected::None),
    (Primary::Meta, Case::Key(KeyCase::ControlAlt), Expected::None),
    (Primary::Control, Case::Key(KeyCase::AltComposed), Expected::Press(EVENT_ALT)),
    (Primary::Meta, Case::Key(KeyCase::AltComposed), Expected::Press(EVENT_ALT)),
    (Primary::Control, Case::Key(KeyCase::Plain), Expected::Press(EVENT_PLAIN)),
    (Primary::Control, Case::CompositionEnd, Expected::Input),
];

fn key_event(primary: Primary, case: KeyCase, in_field: bool) -> Event {
    let mut event = Event::Key {
        key: Box::from(b"k".as_slice()),
        code: Box::from(b"KeyK".as_slice()),
        control: false,
        meta: false,
        alt: false,
        shift: false,
        in_field,
        composing: false,
    };
    if let Event::Key { key, code, control, meta, alt, shift, .. } = &mut event {
        match case {
            KeyCase::Primary | KeyCase::Shift => {
                *control = primary == Primary::Control;
                *meta = primary == Primary::Meta;
                *shift = matches!(case, KeyCase::Shift);
            }
            KeyCase::Other => {
                *control = primary == Primary::Meta;
                *meta = primary == Primary::Control;
            }
            KeyCase::ControlAlt => {
                *control = true;
                *alt = true;
            }
            KeyCase::AltComposed => {
                *key = Box::from("˚".as_bytes());
                *alt = true;
            }
            KeyCase::Plain => {
                *key = Box::from(b"p".as_slice());
                *code = Box::from(b"KeyP".as_slice());
            }
        }
    }
    event
}
fn event(case: Case, primary: Primary) -> Event {
    match case {
        Case::Click(node, modifier) => Event::Click { node, modifier },
        Case::Input(composing) => Event::Input { node: EVENT_FIELD, value: Box::from(b"Typed".as_slice()), composing },
        Case::CompositionEnd => Event::CompositionEnd { node: EVENT_FIELD, value: Box::from(b"Typed".as_slice()) },
        Case::Enter(node, shift, composing) => Event::Enter { node, shift, composing },
        Case::Change(node) => Event::Change { node },
        Case::Cancel => Event::Cancel { dialog: EVENT_DIALOG },
        Case::Key(case) => key_event(primary, case, false),
    }
}
fn expected(expected: Expected) -> Option<(NodeId, Act)> {
    match expected {
        Expected::None => None,
        Expected::Press(node) => Some((node, Act::Press)),
        Expected::Input => Some((EVENT_FIELD, Act::Input { value: Box::from(b"Typed".as_slice()) })),
        Expected::Submit(node) => Some((node, Act::Submit)),
        Expected::Toggle(node) => Some((node, Act::Toggle)),
    }
}

#[test]
fn each_event_gives_its_act() {
    let mut view = View::new(&LIMITS);
    let patches = fixtures::patches(&mut view, 0, events);
    for (primary, case, want) in EVENTS {
        let mut page = Page::new(fixtures::look(), &LIMITS, b"/", *primary);
        page.apply(&patches, &mut Vec::new());
        assert_eq!(page.act(event(*case, *primary), &mut Vec::new()), expected(*want), "{primary:?}: {case:?}");
    }
}

#[test]
fn a_dialogs_cancel_keeps_it_open() {
    let (mut page, mut view, mut ops) = (page(), View::new(&LIMITS), Vec::new());
    page.apply(&fixtures::patches(&mut view, 0, events), &mut Vec::new());
    page.act(Event::Cancel { dialog: EVENT_DIALOG }, &mut ops);
    assert_eq!(
        ops,
        vec![Op::Attributes {
            node: EVENT_DIALOG,
            element: crate::dom::Element {
                tag: "dialog".into(),
                attributes: vec![("data-node".into(), "9".into())],
                children: vec![],
            }
        }]
    );
}

#[test]
fn a_shortcut_with_no_modifier_gives_none_while_a_field_has_focus() {
    let (mut page, mut view) = (page(), View::new(&LIMITS));
    page.apply(&fixtures::patches(&mut view, 0, events), &mut Vec::new());
    assert_eq!(page.act(key_event(Primary::Control, KeyCase::Plain, true), &mut Vec::new()), None);
}

#[test]
fn a_composition_end_is_reported_once() {
    let (mut page, mut view) = (page(), View::new(&LIMITS));
    page.apply(&fixtures::patches(&mut view, 0, events), &mut Vec::new());
    page.act(event(Case::CompositionEnd, Primary::Control), &mut Vec::new());
    assert_eq!(page.act(event(Case::Input(false), Primary::Control), &mut Vec::new()), None);
}

#[test]
fn a_check_is_put_back_to_the_tree_after_its_change() {
    let (mut page, mut view, mut ops) = (page(), View::new(&LIMITS), Vec::new());
    page.apply(&fixtures::patches(&mut view, 0, events), &mut Vec::new());
    page.act(Event::Change { node: EVENT_CHECK }, &mut ops);
    assert_eq!(ops, vec![Op::Choice { node: EVENT_CHECK, checked: true }]);
}

// Preorder: page, main, menu, first item, disabled item, last item.
const MENU: NodeId = NodeId(3);
const FIRST_MENU_ITEM: NodeId = NodeId(4);
const LAST_MENU_ITEM: NodeId = NodeId(6);
fn menu(builder: &mut Builder) {
    main(builder);
    builder.open(Role::Menu);
    builder.name(b"Actions");
    for disabled in [false, true, false] {
        builder.open(Role::MenuItem);
        builder.name(b"Item");
        builder.bind(fixtures::binding(1));
        if disabled {
            builder.state(State::Disabled);
        }
        builder.close();
    }
    builder.close();
    end(builder);
}
fn plain_key(key: &[u8]) -> Event {
    Event::Key {
        key: key.into(),
        code: Box::new([]),
        control: false,
        meta: false,
        alt: false,
        shift: false,
        in_field: false,
        composing: false,
    }
}

#[test]
fn a_menu_opens_without_a_step() {
    let (mut page, mut view) = (page(), View::new(&LIMITS));
    page.apply(&fixtures::patches(&mut view, 0, menu), &mut Vec::new());
    page.act(Event::Click { node: MENU, modifier: false }, &mut Vec::new());
    assert_eq!(page.html(), include_str!("../golden/menu-open.html").trim_end());
    page.act(Event::Click { node: MENU, modifier: false }, &mut Vec::new());
    assert_eq!(page.html(), include_str!("../golden/menu-closed.html").trim_end());
}

#[test]
fn escape_closes_an_open_menu() {
    let (mut page, mut view) = (page(), View::new(&LIMITS));
    page.apply(&fixtures::patches(&mut view, 0, menu), &mut Vec::new());
    page.act(Event::Click { node: MENU, modifier: false }, &mut Vec::new());
    page.act(plain_key(b"Escape"), &mut Vec::new());
    assert_eq!(page.html(), include_str!("../golden/menu-closed.html").trim_end());
}

#[test]
fn menu_arrows_skip_disabled_items() {
    let (mut page, mut view, mut ops) = (page(), View::new(&LIMITS), Vec::new());
    page.apply(&fixtures::patches(&mut view, 0, menu), &mut Vec::new());
    page.act(Event::Click { node: MENU, modifier: false }, &mut Vec::new());
    page.focused(FIRST_MENU_ITEM);
    page.act(plain_key(b"ArrowDown"), &mut ops);
    assert_eq!(ops, vec![Op::Focus { node: LAST_MENU_ITEM }]);
}

#[test]
fn pressing_a_menu_item_closes_the_menu() {
    let (mut page, mut view) = (page(), View::new(&LIMITS));
    page.apply(&fixtures::patches(&mut view, 0, menu), &mut Vec::new());
    page.act(Event::Click { node: MENU, modifier: false }, &mut Vec::new());
    assert_eq!(
        page.act(Event::Click { node: FIRST_MENU_ITEM, modifier: false }, &mut Vec::new()),
        Some((FIRST_MENU_ITEM, Act::Press))
    );
    assert_eq!(page.html(), include_str!("../golden/menu-closed.html").trim_end());
}

// Preorder: page, main, outside button, menu, item, drawing.
const OUTSIDE_BUTTON: NodeId = NodeId(3);
const BESIDE_MENU: NodeId = NodeId(4);
fn menu_with_button(builder: &mut Builder) {
    main(builder);
    button(builder, b"Outside");
    builder.open(Role::Menu);
    builder.name(b"Actions");
    builder.open(Role::MenuItem);
    builder.name(b"Item");
    builder.bind(fixtures::binding(1));
    builder.close();
    builder.close();
    builder.drawing(DrawingKind { owner: fixtures::OWNER, kind: 0 }, b"Chart", b"", 128);
    builder.close();
    end(builder);
}

#[test]
fn a_press_outside_closes_an_open_menu_and_returns_the_focus() {
    let (mut page, mut view, mut ops) = (page(), View::new(&LIMITS), Vec::new());
    page.apply(&fixtures::patches(&mut view, 0, menu), &mut Vec::new());
    page.act(Event::Click { node: MENU, modifier: false }, &mut Vec::new());
    page.focused(FIRST_MENU_ITEM);
    assert_eq!(page.act(Event::Click { node: MAIN, modifier: false }, &mut ops), None);
    assert_eq!(page.html(), include_str!("../golden/menu-closed.html").trim_end());
    assert_eq!(ops.last(), Some(&Op::Focus { node: MENU }));
}

#[test]
fn a_press_outside_goes_on_to_what_it_lands_on_and_keeps_its_focus() {
    let (mut page, mut view, mut ops) = (page(), View::new(&LIMITS), Vec::new());
    page.apply(&fixtures::patches(&mut view, 0, menu_with_button), &mut Vec::new());
    let closed = page.html();
    page.act(Event::Click { node: BESIDE_MENU, modifier: false }, &mut Vec::new());
    page.focused(OUTSIDE_BUTTON);
    assert_eq!(
        page.act(Event::Click { node: OUTSIDE_BUTTON, modifier: false }, &mut ops),
        Some((OUTSIDE_BUTTON, Act::Press))
    );
    assert_eq!(page.html(), closed);
    assert!(!ops.iter().any(|op| matches!(op, Op::Focus { .. })));
}

#[test]
fn a_press_past_the_page_closes_an_open_menu() {
    let (mut page, mut view, mut ops) = (page(), View::new(&LIMITS), Vec::new());
    page.apply(&fixtures::patches(&mut view, 0, menu), &mut Vec::new());
    page.act(Event::Click { node: MENU, modifier: false }, &mut Vec::new());
    assert_eq!(page.act(Event::PressOutside { focus_taken: false }, &mut ops), None);
    assert_eq!(page.html(), include_str!("../golden/menu-closed.html").trim_end());
    assert_eq!(ops.last(), Some(&Op::Focus { node: MENU }));
}

#[test]
fn a_press_past_the_page_keeps_the_focus_it_took() {
    let (mut page, mut view, mut ops) = (page(), View::new(&LIMITS), Vec::new());
    page.apply(&fixtures::patches(&mut view, 0, menu), &mut Vec::new());
    page.act(Event::Click { node: MENU, modifier: false }, &mut Vec::new());
    page.focused(FIRST_MENU_ITEM);
    assert_eq!(page.act(Event::PressOutside { focus_taken: true }, &mut ops), None);
    assert_eq!(page.html(), include_str!("../golden/menu-closed.html").trim_end());
    assert!(!ops.iter().any(|op| matches!(op, Op::Focus { .. })));
}

#[test]
fn a_press_with_no_open_menu_moves_nothing() {
    let (mut page, mut view, mut ops) = (page(), View::new(&LIMITS), Vec::new());
    page.apply(&fixtures::patches(&mut view, 0, menu), &mut Vec::new());
    page.focused(MAIN);
    assert_eq!(page.act(Event::Click { node: MAIN, modifier: false }, &mut ops), None);
    assert_eq!(page.act(Event::PressOutside { focus_taken: false }, &mut ops), None);
    assert_eq!(ops.len(), 0);
    assert_eq!(page.html(), include_str!("../golden/menu-closed.html").trim_end());
}

#[test]
fn a_press_on_a_disabled_item_leaves_the_menu_open() {
    const DISABLED_ITEM: NodeId = NodeId(5);
    let (mut page, mut view, mut ops) = (page(), View::new(&LIMITS), Vec::new());
    page.apply(&fixtures::patches(&mut view, 0, menu), &mut Vec::new());
    page.act(Event::Click { node: MENU, modifier: false }, &mut Vec::new());
    assert_eq!(page.act(Event::Click { node: DISABLED_ITEM, modifier: false }, &mut ops), None);
    assert_eq!(ops.len(), 0);
    assert_eq!(page.html(), include_str!("../golden/menu-open.html").trim_end());
}

#[test]
fn a_drawing_press_outside_closes_the_menu() {
    const BESIDE_DRAWING: NodeId = NodeId(6); // page, main, button, menu, item, drawing
    let (mut page, mut view, mut ops) = (page(), View::new(&LIMITS), Vec::new());
    page.apply(&fixtures::patches(&mut view, 0, menu_with_button), &mut Vec::new());
    let closed = page.html();
    page.act(Event::Click { node: BESIDE_MENU, modifier: false }, &mut Vec::new());
    assert_eq!(page.act(Event::Drawing { node: BESIDE_DRAWING, under: Box::new([]), x: 0, y: 0 }, &mut ops), None);
    assert_eq!(page.html(), closed);
    assert_eq!(ops.last(), Some(&Op::Focus { node: BESIDE_MENU }));
}

const LOG: NodeId = NodeId(3); // page, main, log, text

fn reading(b: &mut Builder, bound: bool, continues: bool, request: Option<u64>) {
    main(b);
    b.open(Role::Log);
    if bound {
        b.bind(fixtures::binding(1));
    }
    if continues {
        b.state(State::Continues);
    }
    if let Some(count) = request {
        b.end_request(count);
    }
    b.text(Run::Plain, b"Lines");
    b.close();
    end(b);
}

fn reading_page(bound: bool, continues: bool) -> Page<'static> {
    let mut page = page();
    page.apply(&fixtures::patches(&mut View::new(&LIMITS), 0, |b| reading(b, bound, continues, None)), &mut Vec::new());
    page
}

fn scroll(page: &mut Page<'_>, top: i32, height: i32, viewport: i32) -> Option<(NodeId, Act)> {
    page.act(Event::Scrolled { node: LOG, top, height, viewport }, &mut Vec::new())
}

const NEAR_HEAD: Option<(NodeId, Act)> = Some((LOG, Act::Near { edge: skein_ui::LogEdge::Head }));
const NEAR_FOOT: Option<(NodeId, Act)> = Some((LOG, Act::Near { edge: skein_ui::LogEdge::Foot }));

#[test]
fn a_head_within_a_screen_and_a_half_is_near() {
    let mut page = reading_page(true, false);
    assert_eq!(scroll(&mut page, 0, 1000, 0), None, "not laid out");
    assert_eq!(scroll(&mut page, 300, 1000, 200), None);
    assert_eq!(scroll(&mut page, 299, 1000, 200), NEAR_HEAD);
    assert_eq!(scroll(&mut page, 300, 1000, 200), None);
    assert_eq!(scroll(&mut page, 299, 1000, 200), NEAR_HEAD, "leaving forgets");
    assert_eq!(scroll(&mut page, 0, 1000, 200), NEAR_HEAD, "at the head");
}

#[test]
fn near_comes_again_as_the_height_moves_and_not_on_every_scroll() {
    let mut page = reading_page(true, false);
    assert_eq!(scroll(&mut page, 200, 1000, 200), NEAR_HEAD);
    assert_eq!(scroll(&mut page, 199, 1000, 200), None);
    assert_eq!(scroll(&mut page, 200, 1100, 200), NEAR_HEAD);
    assert_eq!(scroll(&mut page, 200, 1000, 200), NEAR_HEAD);
    assert_eq!(scroll(&mut page, 200, 1000, 200), None);
}

fn log_busy(page: &mut Page<'_>, busy: bool) {
    let mut view = View::new(&LIMITS);
    let _patches = fixtures::patches(&mut view, 0, |b| reading(b, true, true, None));
    let mut attribute = skein_ui::Made::of(view.tree().get(2).expect("fixture log")).attribute;
    if busy {
        attribute.states.push(State::Busy).expect("fixture state room");
    }
    let mut ops = Vec::new();
    page.apply(&[Patch::Attribute { node: LOG, attribute: Box::new(attribute) }], &mut ops);
    assert!(ops.ends_with(&[Op::Measure { node: LOG }]));
}

#[test]
fn near_comes_again_when_busy_leaves() {
    for (top, edge) in [(200, NEAR_HEAD), (600, NEAR_FOOT)] {
        let mut page = reading_page(true, true);
        assert_eq!(scroll(&mut page, top, 1000, 200), edge);
        log_busy(&mut page, true);
        assert_eq!(scroll(&mut page, top, 1000, 200), None);
        log_busy(&mut page, false);
        assert_eq!(scroll(&mut page, top, 1000, 200), edge, "busy left without a height change");
        assert_eq!(scroll(&mut page, top, 1000, 200), None, "reported once");
    }
}

#[test]
fn a_log_kept_busy_says_nothing_until_the_person_moves() {
    for (top, nearer, almost, edge) in [(200, 150, 151, NEAR_HEAD), (600, 650, 649, NEAR_FOOT)] {
        let mut page = reading_page(true, true);
        assert_eq!(scroll(&mut page, top, 1000, 200), edge);
        log_busy(&mut page, true);
        assert_eq!(scroll(&mut page, top, 1000, 200), None, "busy apply");
        assert_eq!(scroll(&mut page, top, 1100, 200), None, "height alone while busy");
        assert_eq!(scroll(&mut page, top, 1000, 200), None);
        assert_eq!(scroll(&mut page, top + 100, 1100, 200), None, "height also moves the view");
        assert_eq!(scroll(&mut page, top, 1000, 200), None);
        log_busy(&mut page, true);
        assert_eq!(scroll(&mut page, top, 1000, 200), None, "failed read keeps busy");
        assert_eq!(scroll(&mut page, almost, 1000, 200), None, "less than a quarter screen");
        assert_eq!(scroll(&mut page, nearer, 1000, 200), edge, "person moves a quarter screen");
        assert_eq!(scroll(&mut page, nearer, 1000, 200), None);
    }
}

#[test]
fn near_says_nothing_if_the_edge_went_far_while_busy() {
    for (top, edge) in [(200, NEAR_HEAD), (600, NEAR_FOOT)] {
        let mut page = reading_page(true, true);
        assert_eq!(scroll(&mut page, top, 1000, 200), edge);
        log_busy(&mut page, true);
        assert_eq!(scroll(&mut page, 400, 1000, 200), None, "both edges far while busy");
        log_busy(&mut page, false);
        assert_eq!(scroll(&mut page, 400, 1000, 200), None, "busy leaving cannot make an edge near");
        assert_eq!(scroll(&mut page, top, 1000, 200), edge, "edge comes near again");
    }
}

#[test]
fn a_quarter_screen_nearer_says_near_again() {
    let mut page = reading_page(true, false);
    assert_eq!(scroll(&mut page, 299, 1000, 200), NEAR_HEAD);
    assert_eq!(scroll(&mut page, 250, 1000, 200), None);
    assert_eq!(scroll(&mut page, 249, 1000, 200), NEAR_HEAD);
    assert_eq!(scroll(&mut page, 275, 1000, 200), None);
    assert_eq!(scroll(&mut page, 200, 1000, 200), None);
    assert_eq!(scroll(&mut page, 199, 1000, 200), NEAR_HEAD);
}

#[test]
fn the_foot_is_near_only_where_the_log_continues() {
    assert_eq!(scroll(&mut reading_page(true, false), 800, 1000, 200), None);
    let mut page = reading_page(true, true);
    assert_eq!(scroll(&mut page, 500, 1000, 200), None);
    assert_eq!(scroll(&mut page, 501, 1000, 200), NEAR_FOOT);
    assert_eq!(scroll(&mut page, 550, 1000, 200), None);
    assert_eq!(scroll(&mut page, 551, 1000, 200), NEAR_FOOT);
    let mut short = reading_page(true, true);
    assert_eq!(scroll(&mut short, 0, 100, 200), NEAR_HEAD);
    assert_eq!(scroll(&mut short, 0, 100, 200), NEAR_FOOT);
    assert_eq!(scroll(&mut short, 0, 100, 200), None);
}

#[test]
fn an_unbound_log_says_nothing() {
    assert_eq!(scroll(&mut reading_page(false, true), 0, 100, 200), None);
}

#[test]
fn an_apply_measures_each_bound_log() {
    let (mut page, mut view, mut ops) = (page(), View::new(&LIMITS), Vec::new());
    let patches = fixtures::patches(&mut view, 0, |b| {
        main(b);
        for bound in [true, false, true] {
            b.open(Role::Log);
            if bound {
                b.bind(fixtures::binding(1));
            }
            b.close();
        }
        end(b);
    });
    page.apply(&patches, &mut ops);
    assert!(ops.ends_with(&[Op::Measure { node: LOG }, Op::Measure { node: NodeId(5) }]));
    ops.clear();
    page.apply(&[Patch::Text { node: LOG, text: Box::new([]) }], &mut ops);
    assert!(ops.ends_with(&[Op::Measure { node: LOG }, Op::Measure { node: NodeId(5) }]));
    ops.clear();
    page.apply(&[], &mut ops);
    assert_eq!(ops.len(), 0);
}

#[test]
fn a_log_that_continues_is_away_and_is_not_followed() {
    let mut page = reading_page(true, true);
    assert!(page.html().contains("data-log-away"));
    scroll(&mut page, 800, 1000, 200);
    assert!(page.html().contains("data-log-away"));
    let mut ops = Vec::new();
    page.apply(&[Patch::Text { node: LOG, text: Box::new([]) }], &mut ops);
    assert!(!ops.contains(&Op::ScrollEnd { node: LOG }));
}

#[test]
fn an_end_request_follows_through_continues_until_the_person_scrolls_up() {
    let (mut page, mut view) = (page(), View::new(&LIMITS));
    page.apply(&fixtures::patches(&mut view, 0, |b| reading(b, true, true, Some(0))), &mut Vec::new());
    scroll(&mut page, 400, 1000, 200);
    let mut ops = Vec::new();
    page.apply(&fixtures::patches(&mut view, 1, |b| reading(b, true, true, Some(1))), &mut ops);
    assert!(ops.contains(&Op::ScrollEnd { node: LOG }));
    assert!(!page.html().contains("data-log-away"));
    scroll(&mut page, 800, 1000, 200);
    scroll(&mut page, 800, 2000, 200);
    ops.clear();
    page.apply(&[Patch::Text { node: LOG, text: Box::new([]) }], &mut ops);
    assert!(ops.contains(&Op::ScrollEnd { node: LOG }));
    assert!(!page.html().contains("data-log-away"));
    scroll(&mut page, 799, 2000, 200);
    assert!(page.html().contains("data-log-away"));
    ops.clear();
    page.apply(&[Patch::Text { node: LOG, text: Box::new([]) }], &mut ops);
    assert!(!ops.contains(&Op::ScrollEnd { node: LOG }));
    // Removing the request doesn't request the end again.
    ops.clear();
    page.apply(&fixtures::patches(&mut view, 2, |b| reading(b, true, true, None)), &mut ops);
    assert!(!ops.contains(&Op::ScrollEnd { node: LOG }));
    page.apply(&fixtures::patches(&mut view, 3, |b| reading(b, true, false, Some(2))), &mut Vec::new());
    scroll(&mut page, 1800, 2000, 200);
    assert!(!page.html().contains("data-log-away"));
    scroll(&mut page, 1800, 2100, 200);
    assert!(page.html().contains("data-log-away"), "reaching the list's end releases the request");
}

#[test]
fn an_away_log_holds_its_view_across_an_apply() {
    let mut page = reading_page(true, false);
    scroll(&mut page, 400, 1000, 200);
    let mut ops = Vec::new();
    page.apply(&[Patch::Text { node: LOG, text: Box::new([]) }], &mut ops);
    assert_eq!(ops.first(), Some(&Op::HoldView { node: LOG }));
    assert!(ops.ends_with(&[Op::PutView { node: LOG }, Op::Measure { node: LOG }]));
    scroll(&mut page, 800, 1000, 200);
    ops.clear();
    page.apply(&[Patch::Text { node: LOG, text: Box::new([]) }], &mut ops);
    assert!(!ops.contains(&Op::HoldView { node: LOG }));
    assert!(!ops.contains(&Op::PutView { node: LOG }));
}

fn page_key(down: bool, in_field: bool) -> Event {
    let key = if down { b"PageDown".as_slice() } else { b"PageUp".as_slice() };
    Event::Key {
        key: key.into(),
        code: key.into(),
        control: false,
        meta: false,
        alt: false,
        shift: false,
        in_field,
        composing: false,
    }
}

#[test]
fn a_page_key_measures_its_log_in_the_same_apply() {
    let mut page = reading_page(false, false);
    scroll(&mut page, 800, 1000, 200);
    for down in [false, true] {
        let mut ops = Vec::new();
        assert_eq!(page.act(page_key(down, true), &mut ops), None);
        assert_eq!(ops, vec![Op::ScrollScreen { node: LOG, down }, Op::Measure { node: LOG }]);
    }
}

#[test]
fn a_page_up_while_following_stops_following() {
    let mut page = reading_page(false, false);
    scroll(&mut page, 800, 1000, 200);
    assert!(!page.html().contains("data-log-away"));
    page.act(page_key(false, true), &mut Vec::new());
    scroll(&mut page, 600, 1000, 200);
    assert!(page.html().contains("data-log-away"));
}

#[test]
fn a_delivery_after_a_page_up_leaves_the_position() {
    let mut page = reading_page(true, false);
    scroll(&mut page, 800, 1000, 200);
    page.act(page_key(false, true), &mut Vec::new());
    scroll(&mut page, 600, 1000, 200);
    let mut ops = Vec::new();
    page.apply(&[Patch::Text { node: LOG, text: Box::new([]) }], &mut ops);
    assert_eq!(ops.first(), Some(&Op::HoldView { node: LOG }));
    assert!(ops.ends_with(&[Op::PutView { node: LOG }, Op::Measure { node: LOG }]));
    assert_eq!(scroll_ends(&ops).len(), 0);
}

#[test]
fn a_page_down_to_the_foot_follows_again() {
    let mut page = reading_page(false, false);
    scroll(&mut page, 800, 1000, 200);
    page.act(page_key(false, true), &mut Vec::new());
    scroll(&mut page, 600, 1000, 200);
    assert!(page.html().contains("data-log-away"));
    page.act(page_key(true, true), &mut Vec::new());
    scroll(&mut page, 800, 1000, 200);
    assert!(!page.html().contains("data-log-away"));
    let mut ops = Vec::new();
    page.apply(&[Patch::Text { node: LOG, text: Box::new([]) }], &mut ops);
    assert_eq!(scroll_ends(&ops), vec![LOG]);
    assert!(!ops.contains(&Op::HoldView { node: LOG }));
}

#[test]
fn the_browsers_scroll_event_after_a_page_key_changes_nothing() {
    let mut page = reading_page(true, false);
    scroll(&mut page, 400, 600, 200);
    page.act(page_key(false, true), &mut Vec::new());
    assert_eq!(scroll(&mut page, 200, 600, 200), NEAR_HEAD);
    let mut ops = Vec::new();
    assert_eq!(page.act(Event::Scrolled { node: LOG, top: 200, height: 600, viewport: 200 }, &mut ops), None);
    assert_eq!(ops.len(), 0);
}

#[test]
fn a_press_to_log_end_keeps_the_focus() {
    let (page, _) = log_page(false);
    assert!(page.press_keeps_focus(LOG_END));
    assert!(!page.press_keeps_focus(LOG));
    let mut plain = super::page();
    plain.apply(&fixtures::patches(&mut View::new(&LIMITS), 0, events), &mut Vec::new());
    assert!(!plain.press_keeps_focus(EVENT_BUTTON));
}

#[test]
fn pgup_and_pgdn_scroll_the_first_log_from_a_field() {
    let mut page = page();
    page.apply(
        &fixtures::patches(&mut View::new(&LIMITS), 0, |b| {
            main(b);
            for _ in 0..2 {
                b.open(Role::Log);
                b.close();
            }
            end(b);
        }),
        &mut Vec::new(),
    );
    let mut ops = Vec::new();
    page.act(page_key(false, true), &mut ops);
    assert!(ops.is_empty(), "unmeasured log");
    scroll(&mut page, 0, 200, 200);
    page.act(page_key(false, true), &mut ops);
    assert!(ops.is_empty(), "nothing to scroll");
    scroll(&mut page, 400, 1000, 200);
    for (in_field, composing) in [(false, false), (true, false), (true, true)] {
        for down in [false, true] {
            ops.clear();
            let mut event = page_key(down, in_field);
            if let Event::Key { composing: held, .. } = &mut event {
                *held = composing;
            }
            assert_eq!(page.act(event, &mut ops), None);
            assert_eq!(ops, vec![Op::ScrollScreen { node: LOG, down }, Op::Measure { node: LOG }]);
        }
    }
    for modifier in 0..4 {
        let mut event = page_key(false, true);
        if let Event::Key { control, meta, alt, shift, .. } = &mut event {
            *control = modifier == 0;
            *meta = modifier == 1;
            *alt = modifier == 2;
            *shift = modifier == 3;
        }
        ops.clear();
        assert_eq!(page.act(event, &mut ops), None);
        assert_eq!(ops.len(), 0);
    }
    page.act(Event::Scrolled { node: NodeId(4), top: 400, height: 1000, viewport: 200 }, &mut Vec::new());
    page.apply(&[Patch::Move { node: NodeId(4), parent: MAIN, before: Some(LOG) }], &mut Vec::new());
    ops.clear();
    assert_eq!(page.act(page_key(false, true), &mut ops), None);
    assert_eq!(
        ops,
        vec![Op::ScrollScreen { node: NodeId(4), down: false }, Op::Measure { node: NodeId(4) }],
        "tree order, not node order"
    );
}

#[test]
fn a_menu_or_a_dialog_keeps_them() {
    for dialog in [false, true] {
        let mut page = page();
        page.apply(
            &fixtures::patches(&mut View::new(&LIMITS), 0, |b| {
                main(b);
                b.open(Role::Log);
                b.close();
                b.open(if dialog { Role::Dialog } else { Role::Menu });
                b.close();
                end(b);
            }),
            &mut Vec::new(),
        );
        scroll(&mut page, 400, 1000, 200);
        if !dialog {
            page.act(Event::Click { node: NodeId(4), modifier: false }, &mut Vec::new());
        }
        let mut ops = Vec::new();
        for down in [false, true] {
            assert_eq!(page.act(page_key(down, true), &mut ops), None);
            assert_eq!(ops.len(), 0);
        }
    }
}

#[test]
fn scrolling_a_log_away_stops_its_following() {
    let (mut page, mut view, mut ops) = (page(), View::new(&LIMITS), Vec::new());
    page.apply(
        &fixtures::patches(&mut view, 0, |b| {
            main(b);
            b.open(Role::Log);
            b.text(Run::Plain, b"Old");
            b.close();
            end(b);
        }),
        &mut Vec::new(),
    );
    page.act(Event::Scrolled { node: LOG, top: 0, height: 100, viewport: 20 }, &mut Vec::new());
    page.apply(
        &fixtures::patches(&mut view, 1, |b| {
            main(b);
            b.open(Role::Log);
            b.text(Run::Plain, b"New");
            b.close();
            end(b);
        }),
        &mut ops,
    );
    assert_eq!(scroll_ends(&ops), vec![]);
}

// Preorder: page, main, log, text, log, text, button.
const OTHER_LOG: NodeId = NodeId(5);
const LOG_END: NodeId = NodeId(7);
fn logs(builder: &mut Builder, bound: bool, continues: bool, words: &[u8]) {
    main(builder);
    for name in [b"First".as_slice(), b"Second".as_slice()] {
        builder.open(Role::Log);
        builder.name(name);
        if continues {
            builder.state(State::Continues);
        }
        builder.description(words);
        builder.text(Run::Plain, words);
        builder.close();
    }
    builder.open(Role::Button);
    builder.name(b"Latest");
    builder.state(State::ToLogEnd);
    if bound {
        builder.bind(fixtures::binding(1));
        builder.shortcut(Shortcut { key: b'l', modifier: Modifier::Plain });
    }
    builder.close();
    end(builder);
}
fn log_page(bound: bool) -> (Page<'static>, View) {
    let (mut page, mut view) = (page(), View::new(&LIMITS));
    page.apply(&fixtures::patches(&mut view, 0, |b| logs(b, bound, false, b"Old")), &mut Vec::new());
    (page, view)
}
fn scroll_log(page: &mut Page<'_>, node: NodeId, top: i32, ops: &mut Vec<Op>) {
    page.act(Event::Scrolled { node, top, height: 100, viewport: 20 }, ops);
}
fn log_attribute(ops: &[Op], node: NodeId) -> &crate::dom::Element {
    ops.iter()
        .find_map(|op| match op {
            Op::Attributes { node: changed, element } if *changed == node => Some(element),
            Op::Insert { .. }
            | Op::Remove { .. }
            | Op::Move { .. }
            | Op::Replace { .. }
            | Op::Attributes { .. }
            | Op::Text { .. }
            | Op::Value { .. }
            | Op::Choice { .. }
            | Op::Options { .. }
            | Op::Href { .. }
            | Op::Focus { .. }
            | Op::Reveal { .. }
            | Op::Component { .. }
            | Op::ScrollEnd { .. }
            | Op::HoldView { .. }
            | Op::PutView { .. }
            | Op::Measure { .. }
            | Op::ScrollScreen { .. } => None,
        })
        .expect("log attributes emitted")
}
fn scroll_ends(ops: &[Op]) -> Vec<NodeId> {
    ops.iter()
        .filter_map(|op| match op {
            Op::ScrollEnd { node } => Some(*node),
            Op::Insert { .. }
            | Op::Remove { .. }
            | Op::Move { .. }
            | Op::Replace { .. }
            | Op::Attributes { .. }
            | Op::Text { .. }
            | Op::Value { .. }
            | Op::Choice { .. }
            | Op::Options { .. }
            | Op::Href { .. }
            | Op::Focus { .. }
            | Op::Reveal { .. }
            | Op::HoldView { .. }
            | Op::PutView { .. }
            | Op::Measure { .. }
            | Op::ScrollScreen { .. }
            | Op::Component { .. } => None,
        })
        .collect()
}

#[test]
fn a_log_scrolled_away_says_so_until_it_is_back() {
    let (mut page, _) = log_page(false);
    let mut ops = Vec::new();
    scroll_log(&mut page, LOG, 0, &mut ops);
    assert!(crate::dom::print(log_attribute(&ops, LOG)).contains("data-log-away"));
    ops.clear();
    scroll_log(&mut page, LOG, 10, &mut ops);
    assert!(ops.is_empty(), "no attributes when following does not change");
    scroll_log(&mut page, LOG, 80, &mut ops);
    assert!(!crate::dom::print(log_attribute(&ops, LOG)).contains("data-log-away"));
    assert!(!page.html().contains("data-log-away"));
}

#[test]
fn new_lines_while_away_keep_the_attribute_and_the_view() {
    let (mut page, mut view) = log_page(false);
    scroll_log(&mut page, LOG, 0, &mut Vec::new());
    let mut ops = Vec::new();
    page.apply(
        &fixtures::patches(&mut view, 1, |b| {
            logs(b, false, false, b"New");
        }),
        &mut ops,
    );
    assert!(crate::dom::print(log_attribute(&ops, LOG)).contains("data-log-away"));
    assert!(page.html().contains("data-log-away"));
    assert_eq!(scroll_ends(&ops), vec![OTHER_LOG]);
}

#[test]
fn a_press_to_log_end_takes_every_log_to_its_end() {
    for shortcut in [false, true] {
        let (mut page, mut view) = log_page(true);
        for log in [LOG, OTHER_LOG] {
            scroll_log(&mut page, log, 0, &mut Vec::new());
        }
        let mut ops = Vec::new();
        let event = if shortcut { plain_key(b"l") } else { Event::Click { node: LOG_END, modifier: false } };
        page.act(event, &mut ops);
        assert_eq!(scroll_ends(&ops), vec![LOG, OTHER_LOG]);
        for log in [LOG, OTHER_LOG] {
            assert!(!crate::dom::print(log_attribute(&ops, log)).contains("data-log-away"));
        }
        ops.clear();
        page.apply(&fixtures::patches(&mut view, 1, |b| logs(b, true, false, b"New")), &mut ops);
        assert_eq!(scroll_ends(&ops), vec![LOG, OTHER_LOG]);
        assert!(!page.html().contains("data-log-away"));
    }
}

#[test]
fn a_press_to_log_end_follows_through_continues_until_the_person_scrolls_up() {
    let (mut page, mut view) = (page(), View::new(&LIMITS));
    page.apply(&fixtures::patches(&mut view, 0, |b| logs(b, true, true, b"Old")), &mut Vec::new());
    scroll(&mut page, 40, 100, 20);
    let mut ops = Vec::new();
    assert_eq!(page.act(Event::Click { node: LOG_END, modifier: false }, &mut ops), Some((LOG_END, Act::Press)));
    assert_eq!(scroll_ends(&ops), vec![LOG, OTHER_LOG]);
    scroll(&mut page, 80, 100, 20);
    scroll(&mut page, 80, 200, 20);
    ops.clear();
    page.apply(&fixtures::patches(&mut view, 1, |b| logs(b, true, true, b"New")), &mut ops);
    assert_eq!(scroll_ends(&ops), vec![LOG, OTHER_LOG]);
    assert!(!page.html().contains("data-log-away"));
    scroll(&mut page, 79, 200, 20);
    ops.clear();
    page.apply(&[Patch::Text { node: LOG, text: Box::new([]) }], &mut ops);
    assert_eq!(scroll_ends(&ops), vec![OTHER_LOG]);
    assert!(page.html().contains("data-log-away"));
}

#[test]
fn an_unbound_press_to_log_end_reaches_no_owner_and_a_bound_one_does() {
    for (bound, expected) in [(false, None), (true, Some((LOG_END, Act::Press)))] {
        let (mut page, _) = log_page(bound);
        scroll_log(&mut page, LOG, 0, &mut Vec::new());
        let mut ops = Vec::new();
        assert_eq!(page.act(Event::Click { node: LOG_END, modifier: false }, &mut ops), expected);
        assert_eq!(scroll_ends(&ops), vec![LOG, OTHER_LOG]);
    }
}

#[test]
fn a_choice_is_put_back_to_the_tree_after_its_change() {
    let (mut page, mut view, mut ops) = (page(), View::new(&LIMITS), Vec::new());
    page.apply(&fixtures::patches(&mut view, 0, events), &mut Vec::new());
    page.act(Event::Change { node: EVENT_CHOICE }, &mut ops);
    assert_eq!(ops, vec![Op::Choice { node: EVENT_CHOICE, checked: true }]);
}

#[test]
fn a_dialog_closed_by_the_browser_is_shown_again() {
    let (mut page, mut view, mut ops) = (page(), View::new(&LIMITS), Vec::new());
    page.apply(&fixtures::patches(&mut view, 0, events), &mut Vec::new());
    page.act(Event::Closed { dialog: EVENT_DIALOG }, &mut ops);
    assert_eq!(
        ops,
        vec![Op::Attributes {
            node: EVENT_DIALOG,
            element: crate::dom::Element {
                tag: "dialog".into(),
                attributes: vec![("data-node".into(), "9".into())],
                children: vec![],
            }
        }]
    );
}

#[test]
fn a_log_follows_its_end() {
    let (mut page, mut view, mut ops) = (page(), View::new(&LIMITS), Vec::new());
    page.apply(
        &fixtures::patches(&mut view, 0, |b| {
            main(b);
            b.open(Role::Log);
            b.close();
            end(b);
        }),
        &mut ops,
    );
    let following: Vec<_> = ops
        .iter()
        .filter_map(|op| match op {
            Op::ScrollEnd { node } => Some(*node),
            Op::Insert { .. }
            | Op::Remove { .. }
            | Op::Move { .. }
            | Op::Replace { .. }
            | Op::Attributes { .. }
            | Op::Text { .. }
            | Op::Value { .. }
            | Op::Choice { .. }
            | Op::Options { .. }
            | Op::Href { .. }
            | Op::Focus { .. }
            | Op::Reveal { .. }
            | Op::HoldView { .. }
            | Op::PutView { .. }
            | Op::Measure { .. }
            | Op::ScrollScreen { .. }
            | Op::Component { .. } => None,
        })
        .collect();
    assert_eq!(following, vec![LOG]);
}

fn act_scene(_: DrawingKind, data: &[u8], width: u16, height: u16, scene: &mut skein_ui::scene::Scene) -> bool {
    use skein_ui::scene::Point;
    scene.rect(Point { x: 0, y: 0 }, Point { x: width, y: height }, 0, &[]);
    if data == b"undrawn" {
        scene.scene_act(b"bg");
        return false;
    }
    if data == b"background" { scene.scene_act(b"bg") } else { scene.act(b"bar") }
}
fn drawing_page(data: &[u8], look: crate::Look<'static>) -> Page<'static> {
    let (mut page, mut view) = (Page::new(look, &LIMITS, b"/", Primary::Control), View::new(&LIMITS));
    page.apply(&fixtures::patches(&mut view, 0, |b| drawing(b, data)), &mut Vec::new());
    let mut attribute = skein_ui::Made::of(view.tree().get(2).expect("fixture drawing")).attribute;
    attribute.bound = true;
    page.apply(&[Patch::Attribute { node: DRAWING, attribute: Box::new(attribute) }], &mut Vec::new());
    page.resized(DRAWING, 100, 50, &mut Vec::new());
    page
}

#[test]
fn a_mark_press_reports_its_bytes() {
    let mut page = drawing_page(b"mark", crate::Look { draw: act_scene, ..fixtures::look() });
    assert_eq!(
        page.act(Event::Drawing { node: DRAWING, under: Box::new([0]), x: 7, y: 9 }, &mut Vec::new()),
        Some((DRAWING, Act::Drawing { data: Box::from(b"bar".as_slice()) }))
    );
}

#[test]
fn a_background_press_appends_its_point() {
    for (look, width, x, y, scene_x, scene_y) in [
        (crate::Look { draw: act_scene, ..fixtures::look() }, 100, 7, 9, 7_u16, 9_u16),
        (fixtures::scene_size_look(), 360, 90, 10, 180, 20),
        (fixtures::scene_size_look(), 1000, 91, 11, 65, 7),
        (fixtures::scene_size_look(), 1, u16::MAX, u16::MAX, u16::MAX, u16::MAX),
        (fixtures::scene_size_look(), 0, 90, 10, 90, 10),
    ] {
        let mut page = drawing_page(b"background", look);
        page.resized(DRAWING, width, 50, &mut Vec::new());
        let [x_lo, x_hi] = scene_x.to_le_bytes();
        let [y_lo, y_hi] = scene_y.to_le_bytes();
        assert_eq!(
            page.act(Event::Drawing { node: DRAWING, under: Box::new([]), x, y }, &mut Vec::new()),
            Some((DRAWING, Act::Drawing { data: Box::new([b'b', b'g', x_lo, x_hi, y_lo, y_hi]) })),
            "box width {width}, point ({x}, {y})"
        );
    }
}

#[test]
fn an_undrawn_scene_does_not_act() {
    let mut page = drawing_page(b"undrawn", crate::Look { draw: act_scene, ..fixtures::look() });
    assert_eq!(page.act(Event::Drawing { node: DRAWING, under: Box::new([]), x: 7, y: 9 }, &mut Vec::new()), None);
}

#[test]
fn an_enter_keeps_its_input_after_a_composition() {
    let (mut page, mut view) = (page(), View::new(&LIMITS));
    page.apply(&fixtures::patches(&mut view, 0, events), &mut Vec::new());
    page.act(event(Case::CompositionEnd, Primary::Control), &mut Vec::new());
    page.act(Event::Enter { node: EVENT_FIELD, shift: false, composing: false }, &mut Vec::new());
    assert_eq!(
        page.act(event(Case::Input(false), Primary::Control), &mut Vec::new()),
        Some((EVENT_FIELD, Act::Input { value: Box::from(b"Typed".as_slice()) }))
    );
}

const ESCAPE_DIALOG: NodeId = NodeId(3); // page, main, dialog
const ESCAPE_BUTTON: NodeId = NodeId(5); // page, main, dialog, disabled escape button, escape button
fn dialog_escape(b: &mut Builder) {
    main(b);
    b.open(Role::Dialog);
    for disabled in [true, false] {
        b.open(Role::Button);
        b.bind(fixtures::binding(1));
        b.shortcut(Shortcut::ESCAPE);
        if disabled {
            b.state(State::Disabled);
        }
        b.close();
    }
    b.close();
    end(b);
}

#[test]
fn escape_presses_the_dialogs_escape_button() {
    let (mut page, mut view) = (page(), View::new(&LIMITS));
    page.apply(&fixtures::patches(&mut view, 0, dialog_escape), &mut Vec::new());
    assert_eq!(page.act(Event::Cancel { dialog: ESCAPE_DIALOG }, &mut Vec::new()), Some((ESCAPE_BUTTON, Act::Press)));
}

// Preorder: page, main, form, field, disabled button, first enabled button,
// second enabled button, button outside the form.
const FORM_FIELD: NodeId = NodeId(4);
const FORM_ESCAPE: NodeId = NodeId(6);
const OUTSIDE_FORM: NodeId = NodeId(8);
fn form_escape(b: &mut Builder, names_escape: bool) {
    main(b);
    b.open(Role::Form);
    b.open(Role::TextField);
    b.bind(fixtures::binding(1));
    b.close();
    for disabled in [true, false, false] {
        b.open(Role::Button);
        b.bind(fixtures::binding(1));
        if names_escape {
            b.shortcut(Shortcut::ESCAPE);
        }
        if disabled {
            b.state(State::Disabled);
        }
        b.close();
    }
    b.close();
    b.open(Role::Button);
    b.bind(fixtures::binding(1));
    b.close();
    end(b);
}
fn form_escape_page(names_escape: bool) -> Page<'static> {
    let (mut page, mut view) = (page(), View::new(&LIMITS));
    page.apply(&fixtures::patches(&mut view, 0, |b| form_escape(b, names_escape)), &mut Vec::new());
    page
}

#[test]
fn escape_in_a_form_presses_its_escape_button() {
    for (focus, in_field) in [(FORM_FIELD, true), (FORM_ESCAPE, false)] {
        let mut page = form_escape_page(true);
        page.focused(focus);
        let mut key = plain_key(b"Escape");
        if let Event::Key { in_field: field, .. } = &mut key {
            *field = in_field;
        }
        assert_eq!(page.act(key, &mut Vec::new()), Some((FORM_ESCAPE, Act::Press)));
    }
}

#[test]
fn escape_in_a_form_naming_none_does_nothing() {
    let mut page = form_escape_page(false);
    page.focused(FORM_FIELD);
    assert_eq!(page.act(plain_key(b"Escape"), &mut Vec::new()), None);
}

#[test]
fn escape_outside_a_form_presses_nothing() {
    let mut page = form_escape_page(true);
    page.focused(OUTSIDE_FORM);
    assert_eq!(page.act(plain_key(b"Escape"), &mut Vec::new()), None);
}

// Preorder: page, main, outer form, outer button, inner form, field, inner button.
const INNER_FORM_FIELD: NodeId = NodeId(6);
const INNER_FORM_ESCAPE: NodeId = NodeId(7);
fn nested_form_escape_page(inner_escape: bool) -> Page<'static> {
    let (mut page, mut view) = (page(), View::new(&LIMITS));
    page.apply(
        &fixtures::patches(&mut view, 0, |b| {
            main(b);
            b.open(Role::Form);
            b.open(Role::Button);
            b.bind(fixtures::binding(1));
            b.shortcut(Shortcut::ESCAPE);
            b.close();
            b.open(Role::Form);
            b.open(Role::TextField);
            b.close();
            b.open(Role::Button);
            b.bind(fixtures::binding(1));
            if inner_escape {
                b.shortcut(Shortcut::ESCAPE);
            }
            b.close();
            b.close();
            b.close();
            end(b);
        }),
        &mut Vec::new(),
    );
    page.focused(INNER_FORM_FIELD);
    page
}

#[test]
fn escape_presses_the_innermost_forms_button() {
    let mut page = nested_form_escape_page(true);
    assert_eq!(page.act(plain_key(b"Escape"), &mut Vec::new()), Some((INNER_FORM_ESCAPE, Act::Press)));
}

#[test]
fn escape_in_an_inner_form_does_not_press_the_outer_button() {
    let mut page = nested_form_escape_page(false);
    assert_eq!(page.act(plain_key(b"Escape"), &mut Vec::new()), None);
}

#[test]
fn escape_in_a_dialogs_form_waits_for_its_cancel_event() {
    const DIALOG_FORM_BUTTON: NodeId = NodeId(5); // page, main, dialog/form, form/dialog, button.
    for roles in [[Role::Dialog, Role::Form], [Role::Form, Role::Dialog]] {
        let (mut page, mut view) = (page(), View::new(&LIMITS));
        page.apply(
            &fixtures::patches(&mut view, 0, |b| {
                main(b);
                for role in roles {
                    b.open(role);
                }
                b.open(Role::Button);
                b.bind(fixtures::binding(1));
                b.shortcut(Shortcut::ESCAPE);
                b.close();
                for _ in roles {
                    b.close();
                }
                end(b);
            }),
            &mut Vec::new(),
        );
        page.focused(DIALOG_FORM_BUTTON);
        assert_eq!(page.act(plain_key(b"Escape"), &mut Vec::new()), None);
    }
}

const SELECT: NodeId = NodeId(3); // page, main, select
const SELECTED_OPTION: NodeId = NodeId(4); // page, main, select, first option
const CHOSEN_OPTION: NodeId = NodeId(5); // page, main, select, first option, second option
fn select(b: &mut Builder) {
    main(b);
    b.open(Role::Select);
    for selected in [true, false] {
        b.open(Role::Option);
        b.bind(fixtures::binding(1));
        if selected {
            b.state(State::Selected);
        }
        b.close();
    }
    b.close();
    end(b);
}

#[test]
fn a_select_change_presses_the_option_and_puts_the_choice_back() {
    let (mut page, mut view, mut ops) = (page(), View::new(&LIMITS), Vec::new());
    page.apply(&fixtures::patches(&mut view, 0, select), &mut Vec::new());
    let act = page.act(Event::Choose { select: SELECT, option: CHOSEN_OPTION }, &mut ops);
    assert_eq!(
        (act, ops),
        (
            Some((CHOSEN_OPTION, Act::Press)),
            vec![
                Op::Choice { node: SELECTED_OPTION, checked: true },
                Op::Choice { node: CHOSEN_OPTION, checked: false },
            ]
        )
    );
}

// Preorder: page, main, button, its words, disabled button, its words, link, its words,
// menu, menu item, its words, loose words.
const HELD_BUTTON: NodeId = NodeId(3);
const HELD_BUTTON_WORDS: NodeId = NodeId(4);
const HELD_DISABLED_WORDS: NodeId = NodeId(6);
const HELD_LINK: NodeId = NodeId(7);
const HELD_LINK_WORDS: NodeId = NodeId(8);
const HELD_MENU: NodeId = NodeId(9);
const HELD_ITEM: NodeId = NodeId(10);
const HELD_ITEM_WORDS: NodeId = NodeId(11);
const HELD_LOOSE: NodeId = NodeId(12);
fn held(builder: &mut Builder) {
    main(builder);
    builder.open(Role::Button);
    builder.bind(fixtures::binding(1));
    builder.text(Run::Plain, b"Clear");
    builder.close();
    builder.open(Role::Button);
    builder.bind(fixtures::binding(2));
    builder.state(State::Disabled);
    builder.text(Run::Plain, b"Locked");
    builder.close();
    builder.open(Role::Link);
    builder.link(fixtures::binding(3));
    builder.text(Run::Plain, b"Home");
    builder.close();
    builder.open(Role::Menu);
    builder.name(b"Actions");
    builder.open(Role::MenuItem);
    builder.bind(fixtures::binding(4));
    builder.text(Run::Plain, b"Rename");
    builder.close();
    builder.close();
    builder.text(Run::Plain, b"Loose");
    end(builder);
}
fn held_page() -> Page<'static> {
    let (mut page, mut view) = (page(), View::new(&LIMITS));
    page.apply(&fixtures::patches(&mut view, 0, held), &mut Vec::new());
    page
}

#[test]
fn a_click_on_the_words_a_button_holds_presses_the_button() {
    let mut page = held_page();
    assert_eq!(
        page.act(Event::Click { node: HELD_BUTTON_WORDS, modifier: false }, &mut Vec::new()),
        Some((HELD_BUTTON, Act::Press))
    );
}

#[test]
fn a_click_on_the_words_a_disabled_button_holds_does_nothing() {
    let mut page = held_page();
    assert_eq!(page.act(Event::Click { node: HELD_DISABLED_WORDS, modifier: false }, &mut Vec::new()), None);
}

#[test]
fn a_click_on_the_words_a_link_holds_presses_it_unless_modified() {
    let mut page = held_page();
    assert_eq!(
        page.act(Event::Click { node: HELD_LINK_WORDS, modifier: false }, &mut Vec::new()),
        Some((HELD_LINK, Act::Press))
    );
    assert_eq!(page.act(Event::Click { node: HELD_LINK_WORDS, modifier: true }, &mut Vec::new()), None);
}

#[test]
fn a_click_on_the_words_a_menu_item_holds_presses_it() {
    let (mut page, mut ops) = (held_page(), Vec::new());
    page.act(Event::Click { node: HELD_MENU, modifier: false }, &mut Vec::new());
    assert_eq!(
        page.act(Event::Click { node: HELD_ITEM_WORDS, modifier: false }, &mut ops),
        Some((HELD_ITEM, Act::Press))
    );
    assert_eq!(ops.last(), Some(&Op::Focus { node: HELD_MENU }));
}

#[test]
fn a_click_on_loose_words_does_nothing() {
    let mut page = held_page();
    assert_eq!(page.act(Event::Click { node: HELD_LOOSE, modifier: false }, &mut Vec::new()), None);
}

#[test]
fn shortcut_key_takes_backslash() {
    let mut view = View::new(&LIMITS);
    let patches = fixtures::patches(&mut view, 0, |builder| {
        main(builder);
        builder.open(Role::Button);
        builder.bind(fixtures::binding(1));
        builder.shortcut(Shortcut { key: b'\\', modifier: Modifier::Primary });
        builder.close();
        end(builder);
    });
    // Preorder: page, main, shortcut button.
    let button = NodeId(3);
    for primary in [Primary::Control, Primary::Meta] {
        let mut page = Page::new(fixtures::look(), &LIMITS, b"/", primary);
        page.apply(&patches, &mut Vec::new());
        for (key, code) in [
            (b"\\".as_slice(), b"".as_slice()),
            (b"|".as_slice(), b"Backslash".as_slice()),
            (b"|".as_slice(), b"IntlBackslash".as_slice()),
        ] {
            let event = Event::Key {
                key: key.into(),
                code: code.into(),
                control: primary == Primary::Control,
                meta: primary == Primary::Meta,
                alt: false,
                shift: false,
                in_field: true,
                composing: false,
            };
            assert_eq!(page.act(event, &mut Vec::new()), Some((button, Act::Press)), "{primary:?} {code:?}");
        }
    }
}

// Preorder: page, main, form, area, options, three items, Escape button.
const OPTIONS_FIELD: NodeId = NodeId(4);
const OPTIONS_MENU: NodeId = NodeId(5);
const OPTION_FIRST: NodeId = NodeId(6);
const OPTION_MIDDLE: NodeId = NodeId(7);
const OPTION_LAST: NodeId = NodeId(8);
const OPTIONS_ESCAPE: NodeId = NodeId(9);

fn field_options(b: &mut Builder, disabled: &[NodeId], selected: Option<NodeId>) {
    main(b);
    b.open(Role::Form);
    b.open(Role::TextArea);
    b.name(b"Command");
    b.bind(fixtures::binding(1));
    b.state(State::SubmitsOnEnter);
    b.close();
    b.open(Role::Menu);
    b.name(b"Commands");
    b.state(State::Options);
    for (node, name) in [(OPTION_FIRST, b"/model".as_slice()), (OPTION_MIDDLE, b"/mode"), (OPTION_LAST, b"/more")] {
        b.open(Role::MenuItem);
        b.name(name);
        b.description(b"Help");
        b.bind(fixtures::binding(2));
        if disabled.contains(&node) {
            b.state(State::Disabled);
        }
        if selected == Some(node) {
            b.state(State::Selected);
        }
        b.close();
    }
    b.close();
    b.open(Role::Button);
    b.bind(fixtures::binding(3));
    b.shortcut(Shortcut::ESCAPE);
    b.close();
    b.close();
    end(b);
}

#[derive(Clone, Copy, Debug)]
enum OptionsStep {
    Key(&'static [u8]),
    ShiftEnter,
    ComposingKey(&'static [u8]),
    Click(NodeId),
    Keep,
    Remove(NodeId),
    Blur,
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table holds the field's options and their keys (ui/web/README.md, 5)")]
fn a_fields_options_take_its_keys() {
    use OptionsStep::{Blur, Click, ComposingKey, Keep, Key, Remove, ShiftEnter};
    let down = Key(b"ArrowDown");
    let up = Key(b"ArrowUp");
    let tab = Key(b"Tab");
    let press = Some((OPTION_FIRST, Act::Press));
    let complete = Some((OPTIONS_FIELD, Act::Input { value: b"/model".as_slice().into() }));
    // label, disabled items, selected item, keys, field words, composing, act, active after.
    let cases = [
        ("starts first", &[][..], None, &[][..], b"".as_slice(), false, None, Some(OPTION_FIRST)),
        ("starts selected", &[][..], Some(OPTION_LAST), &[][..], b"", false, None, Some(OPTION_LAST)),
        ("selected disabled", &[OPTION_MIDDLE][..], Some(OPTION_MIDDLE), &[][..], b"", false, None, Some(OPTION_FIRST)),
        ("first disabled", &[OPTION_FIRST][..], None, &[][..], b"", false, None, Some(OPTION_MIDDLE)),
        ("down", &[][..], None, &[down][..], b"", false, None, Some(OPTION_MIDDLE)),
        ("down wraps", &[OPTION_MIDDLE][..], None, &[down, down][..], b"", false, None, Some(OPTION_FIRST)),
        ("up skips", &[OPTION_MIDDLE][..], Some(OPTION_LAST), &[up][..], b"", false, None, Some(OPTION_FIRST)),
        ("up wraps", &[OPTION_MIDDLE][..], None, &[up][..], b"", false, None, Some(OPTION_LAST)),
        ("Enter holds Submit", &[][..], None, &[Key(b"Enter")][..], b"", false, press, Some(OPTION_FIRST)),
        ("shift Enter", &[][..], None, &[ShiftEnter][..], b"", false, None, Some(OPTION_FIRST)),
        ("composition Enter", &[][..], None, &[Key(b"Enter")][..], b"/mo", true, None, Some(OPTION_FIRST)),
        ("composition starts", &[][..], None, &[ComposingKey(b"Enter")][..], b"", false, None, Some(OPTION_FIRST)),
        ("composition arrows", &[][..], None, &[down][..], b"/mo", true, None, Some(OPTION_FIRST)),
        ("Tab completes", &[][..], None, &[tab][..], b"/mo", false, complete, Some(OPTION_FIRST)),
        ("Tab empty", &[][..], None, &[tab][..], b"", false, None, Some(OPTION_FIRST)),
        ("Tab equal", &[][..], None, &[tab][..], b"/model", false, None, Some(OPTION_FIRST)),
        ("Tab unrelated", &[][..], None, &[tab][..], b"/exit", false, None, Some(OPTION_FIRST)),
        ("composition Tab", &[][..], None, &[tab][..], b"/mo", true, None, Some(OPTION_FIRST)),
        (
            "Escape",
            &[][..],
            None,
            &[Key(b"Escape")][..],
            b"",
            false,
            Some((OPTIONS_ESCAPE, Act::Press)),
            Some(OPTION_FIRST),
        ),
        (
            "click",
            &[][..],
            None,
            &[Click(OPTION_LAST)][..],
            b"",
            false,
            Some((OPTION_LAST, Act::Press)),
            Some(OPTION_FIRST),
        ),
        (
            "disabled click",
            &[OPTION_MIDDLE][..],
            None,
            &[Click(OPTION_MIDDLE)][..],
            b"",
            false,
            None,
            Some(OPTION_FIRST),
        ),
        ("keep across apply", &[][..], None, &[down, Keep][..], b"", false, None, Some(OPTION_MIDDLE)),
        ("reset removed active", &[][..], None, &[up, Remove(OPTION_LAST)][..], b"", false, None, Some(OPTION_FIRST)),
        (
            "no enabled option",
            &[OPTION_FIRST, OPTION_MIDDLE, OPTION_LAST][..],
            None,
            &[down, tab][..],
            b"/mo",
            false,
            None,
            None,
        ),
        (
            "no option submits",
            &[OPTION_FIRST, OPTION_MIDDLE, OPTION_LAST][..],
            None,
            &[Key(b"Enter")][..],
            b"",
            false,
            Some((OPTIONS_FIELD, Act::Submit)),
            None,
        ),
        (
            "later Enter submits",
            &[][..],
            None,
            &[Remove(OPTIONS_MENU), Key(b"Enter")][..],
            b"",
            false,
            Some((OPTIONS_FIELD, Act::Submit)),
            None,
        ),
        ("remove list", &[][..], None, &[Remove(OPTIONS_MENU)][..], b"", false, None, None),
        ("focus elsewhere", &[][..], None, &[Blur, down][..], b"", false, None, Some(OPTION_FIRST)),
    ];
    for (label, disabled, selected, steps, words, composing, expected, active) in cases {
        let (mut page, mut view) = (page(), View::new(&LIMITS));
        let mut ops = Vec::new();
        page.apply(&fixtures::patches(&mut view, 0, |b| field_options(b, disabled, selected)), &mut ops);
        assert!(
            ops.iter().any(|op| matches!(op, Op::Options { field: OPTIONS_FIELD, menu: Some(OPTIONS_MENU), .. })),
            "{label}: platform links the field"
        );
        page.focused(OPTIONS_FIELD);
        page.act(Event::Input { node: OPTIONS_FIELD, value: words.into(), composing }, &mut Vec::new());
        assert!(page.press_keeps_focus(OPTION_FIRST), "{label}: option pointer keeps focus");
        let mut acts = Vec::new();
        ops.clear();
        for step in steps {
            let act = match step {
                Key(key) | ComposingKey(key) => {
                    let composing = composing || matches!(step, ComposingKey(_));
                    let mut event = plain_key(key);
                    if let Event::Key { in_field, composing: held, .. } = &mut event {
                        *in_field = true;
                        *held = composing;
                    }
                    let act = page.act(event, &mut ops);
                    if *key == b"Enter" {
                        if let Some(act) = act {
                            acts.push(act);
                        }
                        page.act(Event::Enter { node: OPTIONS_FIELD, shift: false, composing }, &mut ops)
                    } else {
                        act
                    }
                }
                ShiftEnter => {
                    let mut event = plain_key(b"Enter");
                    if let Event::Key { in_field, shift, .. } = &mut event {
                        *in_field = true;
                        *shift = true;
                    }
                    assert_eq!(page.act(event, &mut ops), None, "{label}");
                    page.act(Event::Enter { node: OPTIONS_FIELD, shift: true, composing: false }, &mut ops)
                }
                Click(node) => page.act(Event::Click { node: *node, modifier: false }, &mut ops),
                Keep => {
                    let mut a = skein_ui::Made::of(view.tree().get(4).expect("fixture menu")).attribute;
                    a.description = Some(b"Updated help".as_slice().into());
                    page.apply(&[Patch::Attribute { node: OPTIONS_MENU, attribute: Box::new(a) }], &mut ops);
                    None
                }
                Remove(node) => {
                    page.apply(&[Patch::Remove { node: *node }], &mut ops);
                    None
                }
                Blur => {
                    page.focused(OPTIONS_ESCAPE);
                    None
                }
            };
            if let Some(act) = act {
                acts.push(act);
            }
        }
        assert_eq!(acts.as_slice(), expected.as_slice(), "{label}: act");
        let html = page.html();
        if let Some(active) = active {
            assert!(
                html.contains(&format!("aria-activedescendant=\"skein-ui-option-{}\"", active.0)),
                "{label}: active option: {html}"
            );
            assert!(
                html.contains("aria-autocomplete=\"list\" aria-controls=\"skein-ui-options-5\""),
                "{label}: field controls list"
            );
            assert_eq!(html.matches("aria-selected=\"true\"").count(), 1, "{label}: one active option");
            let item = html
                .split('<')
                .find(|tag| tag.split('>').next().unwrap().contains(&format!("id=\"skein-ui-option-{}\"", active.0)))
                .expect("active item id");
            assert!(
                item.split('>').next().unwrap().contains("aria-selected=\"true\""),
                "{label}: selected follows active"
            );
        } else {
            assert!(
                !html.contains("aria-activedescendant") && !html.contains("aria-selected"),
                "{label}: no active option"
            );
            if disabled.len() != 3 {
                assert!(
                    !html.contains("aria-autocomplete") && !html.contains("aria-controls"),
                    "{label}: field unlinked"
                );
            }
        }
        assert!(!ops.iter().any(|op| matches!(op, Op::Focus { .. })), "{label}: focus stays");
        if expected.is_none() && steps.iter().any(|step| matches!(step, Key(b"Tab"))) {
            assert!(ops.is_empty(), "{label}: browser can move focus");
        }
        if active.is_some()
            && !composing
            && !steps.iter().any(|step| matches!(step, Blur))
            && steps.iter().any(|step| matches!(step, Key(b"ArrowDown" | b"ArrowUp")))
        {
            assert!(
                ops.contains(&Op::Options { field: OPTIONS_FIELD, menu: Some(OPTIONS_MENU), active }),
                "{label}: platform follows active option"
            );
        }
        if matches!(expected, Some((_, Act::Input { .. }))) {
            assert!(
                ops.contains(&Op::Value { node: OPTIONS_FIELD, value: b"/model".as_slice().into() }),
                "{label}: completion written"
            );
        }
    }
}
