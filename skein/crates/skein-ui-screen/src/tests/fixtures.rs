mod every;
pub use every::every;

use crate::{Arrange, Border, Glyphs, Paint, Place, RegionLook, Rgb, StyleLook, Theme};
use skein_lib::{Queue, Token};
use skein_ui::{Binding, Builder, EntryKind, Limits, NodeId, Owner, Region, Role, Run, State, Style, Tree};

// Room for the twenty paragraphs, their text leaves, and the role fixture.
pub const LIMITS: Limits = Limits {
    nodes: 128,
    depth: 16,
    text_per_node: 256,
    text_bytes: 4096,
    styles_per_node: 2,
    patches: 1024,
    bindings: 32,
    ..Limits::EMPTY
};
pub const OWNER: Owner = Owner { number: 1 };
pub const TOP: Region = Region { owner: OWNER, number: 1 };
pub const BOTTOM: Region = Region { owner: OWNER, number: 2 };
pub const RED: Style = Style { owner: OWNER, token: 1 };
pub const BLUE: Style = Style { owner: OWNER, token: 2 };
pub const RED_PAINT: Paint = Paint { fore: Some(Rgb { red: 255, green: 0, blue: 0 }), bold: true, ..Paint::NONE };
pub const BLUE_PAINT: Paint = Paint { fore: Some(Rgb { red: 0, green: 0, blue: 255 }), italic: true, ..Paint::NONE };
pub fn theme() -> Theme {
    Theme {
        base: Paint::NONE,
        focus: Paint { reverse: true, ..Paint::NONE },
        placeholder: Paint { dim: true, ..Paint::NONE },
        regions: Box::new([
            RegionLook {
                region: TOP,
                place: Place::Top,
                arrange: Arrange::Lines,
                border: Some(Border {
                    corners: ['+', '+', '+', '+'],
                    across: '-',
                    down: '|',
                    paint: Paint { bold: true, ..Paint::NONE },
                    title: true,
                }),
                pad: 0,
                paint: Paint::NONE,
            },
            RegionLook {
                region: BOTTOM,
                place: Place::Bottom,
                arrange: Arrange::Spread,
                border: None,
                pad: 0,
                paint: Paint::NONE,
            },
        ]),
        styles: Box::new([StyleLook { style: RED, paint: RED_PAINT }, StyleLook { style: BLUE, paint: BLUE_PAINT }]),
        glyphs: Glyphs { focus: '>', mask: '●' },
    }
}
pub fn binding() -> Binding {
    Binding { owner: OWNER, token: Token::new(1) }
}
pub fn leaf(b: &mut Builder, role: Role, text: &[u8]) {
    b.open(role);
    b.text(Run::Plain, text);
    b.close();
}
pub fn button(b: &mut Builder, text: &[u8]) {
    b.open(Role::Button);
    b.name(text);
    b.bind(binding());
    b.close();
}
pub fn finish(b: Builder) -> Tree {
    let mut tree = b.finish();
    let mut patches = Queue::with_capacity(LIMITS.patches);
    skein_ui::diff(&Tree::new(LIMITS.nodes), &mut tree, &LIMITS, &mut 1, &mut patches);
    tree
}
pub fn id(tree: &Tree, name: &[u8]) -> NodeId {
    for n in tree.nodes() {
        if n.name.as_deref() == Some(name) {
            return n.id;
        }
    }
    unreachable!("fixture names its focus")
}
pub fn bands() -> Tree {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    b.open(Role::Main);
    leaf(&mut b, Role::Paragraph, b"Body.");
    b.open(Role::List);
    for text in [b"one".as_slice(), b"two", b"three"] {
        leaf(&mut b, Role::Item, text);
    }
    b.close();
    button(&mut b, b"First");
    button(&mut b, b"Second");
    b.close();
    b.open(Role::Banner);
    b.region(TOP);
    leaf(&mut b, Role::Heading { level: 1 }, b"Title");
    leaf(&mut b, Role::Paragraph, b"First paragraph in the top.");
    leaf(&mut b, Role::Paragraph, b"Second paragraph.");
    b.close();
    b.open(Role::ContentInfo);
    b.region(BOTTOM);
    b.open(Role::Button);
    b.name(b"Quit");
    b.bind(binding());
    b.style(RED);
    b.close();
    b.open(Role::Status);
    b.style(BLUE);
    b.text(Run::Plain, b"Ready");
    b.close();
    b.close();
    b.close();
    finish(b)
}
pub fn styled_bands() -> Tree {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    b.open(Role::Banner);
    b.region(TOP);
    b.open(Role::Heading { level: 1 });
    b.style(RED);
    b.text(Run::Plain, b"Title");
    b.close();
    leaf(&mut b, Role::Paragraph, b"Inside");
    b.close();
    b.open(Role::ContentInfo);
    b.region(BOTTOM);
    leaf(&mut b, Role::Status, b"End");
    b.close();
    b.close();
    finish(b)
}
pub fn text() -> Tree {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    leaf(&mut b, Role::Paragraph, b"Alpha beta gamma delta epsilon");
    leaf(&mut b, Role::Paragraph, "abcdefghijklmnopqrstuvw界Z".as_bytes());
    leaf(&mut b, Role::Paragraph, b"safe\x1b\tbad\xff!");
    b.close();
    finish(b)
}
pub fn fields() -> Tree {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    for (name, value, entry) in [
        (b"Name".as_slice(), b"abcdef".as_slice(), EntryKind::Text),
        (b"Password", "séc".as_bytes(), EntryKind::Secret),
        (b"Overflow", b"0123456789012345678901234567890123456789", EntryKind::Text),
    ] {
        b.open(Role::TextField);
        b.name(name);
        b.value(value, 0);
        b.entry(entry);
        b.bind(binding());
        b.close();
    }
    b.open(Role::TextField);
    b.name(b"Hint");
    b.placeholder(b"type here");
    b.bind(binding());
    b.close();
    b.open(Role::TextArea);
    b.name(b"Notes");
    b.bind(binding());
    b.value(b"A text area wraps these words across its region width.", 0);
    b.close();
    b.close();
    finish(b)
}
pub fn scroll() -> Tree {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    for text in [
        b"Paragraph 01",
        b"Paragraph 02",
        b"Paragraph 03",
        b"Paragraph 04",
        b"Paragraph 05",
        b"Paragraph 06",
        b"Paragraph 07",
        b"Paragraph 08",
        b"Paragraph 09",
        b"Paragraph 10",
        b"Paragraph 11",
        b"Paragraph 12",
        b"Paragraph 13",
        b"Paragraph 14",
        b"Paragraph 15",
        b"Paragraph 16",
        b"Paragraph 17",
        b"Paragraph 18",
        b"Paragraph 19",
        b"Paragraph 20",
    ] {
        leaf(&mut b, Role::Paragraph, text);
    }
    button(&mut b, b"End");
    b.close();
    finish(b)
}
pub fn roles() -> Tree {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    for (role, name) in [
        (Role::Banner, b"Banner".as_slice()),
        (Role::Navigation, b"Navigation"),
        (Role::Main, b"Main"),
        (Role::Complementary, b"Complementary"),
        (Role::ContentInfo, b"Content info"),
        (Role::Form, b"Form"),
        (Role::Search, b"Search?"),
        (Role::Region, b"Region"),
    ] {
        b.open(role);
        b.name(name);
        if role == Role::Navigation {
            b.state(State::Expanded);
        }
        b.close();
    }
    leaf(&mut b, Role::Heading { level: 1 }, b"Heading");
    b.open(Role::Group);
    b.name(b"Group is not printed");
    leaf(&mut b, Role::Paragraph, b"Paragraph");
    b.close();
    b.open(Role::Slot);
    b.text(Run::Code, b"Text");
    b.close();
    leaf(&mut b, Role::Time, b"Now");
    leaf(&mut b, Role::Status, b"Ready");
    leaf(&mut b, Role::Alert, b"Attention");
    b.open(Role::Separator);
    b.close();
    b.open(Role::List);
    b.name(b"Steps");
    b.count(8);
    b.ordered(4);
    b.open(Role::Item);
    b.position(3);
    b.state(State::Completed);
    b.text(Run::Plain, b"Do it");
    b.close();
    b.close();
    b.open(Role::Button);
    b.name(b"Push");
    b.bind(binding());
    for state in [State::Disabled, State::Busy, State::Current, State::Invalid, State::Pressed, State::Selected] {
        b.state(state);
    }
    b.close();
    b.open(Role::Link);
    b.external(b"https://example.test/");
    b.text(Run::Plain, b"Docs");
    b.close();
    b.open(Role::TextField);
    b.name(b"Email");
    b.value(b"a@b", 0);
    b.entry(EntryKind::Email);
    b.state(State::Required);
    b.state(State::SubmitsOnEnter);
    b.description(b"Use your address.");
    b.bind(binding());
    b.close();
    b.open(Role::TextArea);
    b.name(b"Notes");
    b.value(b"hi", 0);
    b.bind(binding());
    b.close();
    b.close();
    finish(b)
}

/// A field followed by a paragraph makes accidental extra field rows visible.
pub fn field_tail(role: Role, value: &[u8], entry: EntryKind) -> Tree {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    b.open(role);
    b.name(b"F");
    b.value(value, 0);
    if role == Role::TextField {
        b.entry(entry);
    }
    b.placeholder(b"hint");
    b.bind(binding());
    b.close();
    leaf(&mut b, Role::Paragraph, b"Tail");
    b.close();
    finish(b)
}

pub fn role_control(role: Role, state: State, disabled: bool, bound: bool) -> Tree {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    if role == Role::Option {
        b.open(Role::Select);
        b.name(b"Select");
        if disabled {
            b.state(State::Disabled);
        }
    }
    b.open(role);
    b.name(b"Choose");
    b.state(state);
    if disabled && role != Role::Option {
        b.state(State::Disabled);
    }
    if bound {
        b.bind(binding());
    }
    b.close();
    if role == Role::Option {
        b.close();
    }
    b.close();
    finish(b)
}

pub fn selected_check(tree: &Tree) -> NodeId {
    for node in tree.nodes() {
        if node.role == Role::Check && node.states.as_slice().contains(&State::Selected) {
            return node.id;
        }
    }
    unreachable!("every-role tree has a selected check")
}

pub fn disclosures() -> Tree {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    b.open(Role::Disclosure);
    b.description(b"unnamed description");
    leaf(&mut b, Role::Paragraph, b"child");
    b.close();
    b.open(Role::Disclosure);
    b.name(b"More?");
    b.description(b"first\nnext");
    b.state(State::Expanded);
    leaf(&mut b, Role::Paragraph, b"child");
    b.close();
    b.close();
    finish(b)
}

pub fn table_edges() -> Tree {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    b.open(Role::Table);
    b.description(b"table");
    b.count(9);
    b.open(Role::Row);
    b.description(b"header row");
    b.open(Role::ColumnHeader);
    b.description(b"empty header");
    b.sort(skein_ui::Sort::Ascending);
    b.close();
    b.close();
    b.open(Role::Row);
    leaf(&mut b, Role::Cell, b"");
    leaf(&mut b, Role::Cell, b"");
    b.close();
    b.open(Role::Row);
    b.position(2);
    b.state(State::Selected);
    leaf(&mut b, Role::Cell, b"data\ncontinued");
    b.close();
    b.close();
    b.open(Role::ColumnHeader);
    b.text(Run::Plain, b"Sorted");
    b.sort(skein_ui::Sort::Descending);
    b.close();
    leaf(&mut b, Role::RowHeader, b"outside row");
    b.close();
    finish(b)
}

pub fn wrapped_row() -> Tree {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    b.open(Role::Table);
    b.open(Role::Row);
    leaf(&mut b, Role::Cell, b"alpha beta gamma delta");
    leaf(&mut b, Role::Cell, b"epsilon");
    b.close();
    b.close();
    b.close();
    finish(b)
}

pub fn blocks() -> Tree {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    leaf(&mut b, Role::Code, "123456789\nabcdefg界Z\nok".as_bytes());
    leaf(&mut b, Role::Quote, b"one\ntwo");
    b.open(Role::Progress);
    b.name(b"Work");
    b.close();
    b.drawing(skein_ui::DrawingKind { owner: OWNER, kind: 1 }, b"", b"", 0);
    leaf(&mut b, Role::Paragraph, b"fallback");
    b.close();
    b.close();
    finish(b)
}

pub fn log_menu_tree() -> Tree {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    b.open(Role::Log);
    b.name(b"Log");
    b.bind(binding());
    b.state(State::Continues);
    b.end_request(1);
    leaf(&mut b, Role::Paragraph, b"oldest");
    leaf(&mut b, Role::Paragraph, b"newest");
    b.close();
    b.open(Role::Menu);
    b.name(b"Menu");
    b.state(State::Options);
    b.open(Role::MenuItem);
    b.name(b"Run");
    b.bind(binding());
    b.close();
    b.close();
    b.open(Role::Tree);
    b.count(3);
    b.open(Role::TreeItem);
    b.position(2);
    b.state(State::Expanded);
    b.description(b"item description");
    leaf(&mut b, Role::Paragraph, b"branch\ncontinued");
    b.open(Role::Tree);
    b.open(Role::TreeItem);
    b.position(1);
    b.text(Run::Plain, b"inner");
    b.close();
    b.close();
    b.close();
    b.open(Role::TreeItem);
    b.position(3);
    b.state(State::Expanded);
    b.open(Role::Disclosure);
    b.name(b"Details");
    leaf(&mut b, Role::Paragraph, b"detail");
    b.close();
    b.close();
    b.close();
    b.close();
    finish(b)
}
