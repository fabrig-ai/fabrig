//! The example's live tree fixture (skein/docs/design/ui/web/testing.md, 3).
use skein_lib::{Decimal, Queue, Token, Writer};
use skein_ui::{Act, Binding, Builder, Key, Limits, NodeId, Owner, Patch, Role, Run, View};

const OWNER: Owner = Owner { number: 1 };
const NEXT: Binding = Binding { owner: OWNER, token: Token::new(1) };
const REVISION: &[u8] = b"Tree revision ";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Case {
    First,
    Middle,
    Move,
    Rename,
    Last,
    Mixed,
    Form,
    Summary,
    ShownFolded,
}

impl Case {
    #[must_use]
    pub fn from_path(path: &str) -> Option<Self> {
        match path.strip_prefix("/test/tree/")? {
            "first" => Some(Self::First),
            "middle" => Some(Self::Middle),
            "move" => Some(Self::Move),
            "rename" => Some(Self::Rename),
            "last" => Some(Self::Last),
            "mixed" => Some(Self::Mixed),
            "form" => Some(Self::Form),
            "summary" => Some(Self::Summary),
            "shown-folded" => Some(Self::ShownFolded),
            _ => None,
        }
    }
}

#[must_use]
pub fn limits() -> Limits {
    let mut limits = super::limits::limits();
    limits.depth = 7; // Page, main, tree, three item levels, and their text.
    limits
}

#[derive(Debug)]
pub struct Tree {
    case: Case,
    view: View,
    revision: u64,
}

impl Tree {
    #[must_use]
    pub fn new(case: Case) -> Self {
        Self { case, view: View::new(&limits()), revision: 0 }
    }

    pub fn act(&mut self, node: NodeId, act: Act) {
        if let Some(bound) = self.view.act(node, act)
            && bound.binding == NEXT
            && bound.act == Act::Press
        {
            self.revision = self.revision.checked_add(1).expect("fixture revision fits");
        }
    }

    /// Emit one pending delivery, or return false if the current view is drawn.
    #[must_use]
    pub fn patch(&mut self, out: &mut Queue<Patch>) -> bool {
        if !self.view.stale(self.revision) {
            return false;
        }
        let mut builder = self.view.builder();
        build(&mut builder, self.case, self.revision);
        self.view.finish(builder, self.revision, out);
        // The focus case removes the form before renaming its summary, in two deliveries in one turn.
        if self.case == Case::Summary && self.revision == 1 {
            self.revision = 2;
        }
        true
    }
}

fn item(builder: &mut Builder, key: u64, name: &[u8]) {
    builder.open(Role::TreeItem);
    builder.key(Key { owner: OWNER, kind: 0, a: key, b: 0 });
    builder.name(name);
    builder.text(Run::Plain, name);
}

fn build(builder: &mut Builder, case: Case, revision: u64) {
    builder.open(Role::Page);
    builder.open(Role::Main);
    builder.open(Role::Button);
    builder.name(b"Patch tree");
    builder.bind(NEXT);
    builder.close();
    builder.open(Role::Status);
    let digits = Decimal::of(revision);
    let length = REVISION.len().checked_add(digits.as_bytes().len()).expect("revision caption length fits");
    let mut text = Writer::new(length);
    text.put(REVISION).expect("caption reserves the prefix");
    text.put(digits.as_bytes()).expect("caption reserves the digits");
    let text = text.finish();
    builder.name(&text);
    builder.text(Run::Plain, &text);
    builder.close();
    match case {
        Case::ShownFolded => account(builder, revision),
        Case::Form | Case::Summary => form(builder, revision, case == Case::Summary),
        Case::First | Case::Middle | Case::Move | Case::Rename | Case::Last | Case::Mixed => {
            keyed_tree(builder, case, revision);
        }
    }
    builder.close();
    builder.close();
}

fn keyed_tree(builder: &mut Builder, case: Case, revision: u64) {
    builder.open(Role::Tree);
    builder.name(b"Patch tree");
    item(builder, 100, b"Parent");
    let changed = revision != 0;
    let children: &[u64] = match case {
        Case::First if !changed => &[],
        Case::Last if changed => &[],
        Case::First | Case::Last => &[1],
        Case::Middle if !changed => &[1, 3],
        Case::Move if changed => &[3, 1, 2],
        Case::Middle | Case::Move | Case::Rename => &[1, 2, 3],
        Case::Mixed if revision == 0 => &[1, 3],
        Case::Mixed if revision == 1 => &[1, 0, 3],
        Case::Mixed if revision == 2 => &[1, 2, 0, 3, 4],
        Case::Mixed => &[1, 2, 3, 4],
        Case::Form | Case::Summary | Case::ShownFolded => unreachable!("this case draws a disclosure"),
    };
    for key in children {
        if *key == 0 {
            builder.open(Role::Paragraph);
            builder.name(b"Between runs");
            builder.text(Run::Plain, b"Between runs");
        } else {
            let name: &[u8] = match key {
                1 => b"One",
                2 if case == Case::Rename && changed => b"Renamed two",
                2 => b"Two",
                3 => b"Three",
                4 => b"Four",
                _ => unreachable!("declared child keys"),
            };
            item(builder, *key, name);
            // The keyed item keeps its own nested group and descendant.
            if *key == 1 {
                item(builder, 10, b"Grandchild");
                builder.close();
            }
        }
        builder.close();
    }
    builder.close();
    item(builder, 200, b"Stable");
    item(builder, 201, b"Stable leaf");
    builder.close();
    builder.close();
    builder.close();
}

// A parent's name can change before its form is removed in a later delivery.
fn form(builder: &mut Builder, revision: u64, closes_first: bool) {
    builder.open(Role::Disclosure);
    builder.name(if revision == 0 || (closes_first && revision == 1) { b"Project" } else { b"Renamed project" });
    builder.state(skein_ui::State::Expanded);
    if revision < if closes_first { 1 } else { 2 } {
        builder.open(Role::Form);
        builder.name(if revision == 0 { b"Rename project" } else { b"Rename renamed project" });
        builder.open(Role::TextField);
        builder.name(b"Name");
        builder.focus(0);
        builder.value(b"Project", 0);
        builder.bind(NEXT);
        builder.close();
        builder.open(Role::Button);
        builder.name(b"Save");
        builder.bind(NEXT);
        builder.close();
        builder.close();
    }
    builder.close();
}

fn account(builder: &mut Builder, revision: u64) {
    builder.open(Role::Disclosure);
    builder.name(b"Claude subscription");
    builder.description(if revision < 2 { b"Ready" } else { b"Updated" });
    builder.open(Role::Group);
    if revision != 4 {
        builder.state(skein_ui::State::ShownFolded);
    }
    builder.text(Run::Plain, if revision < 2 { b"Week" } else { b"Month" });
    builder.open(Role::Meter);
    builder.name(b"Usage");
    builder.fraction(skein_ui::Fraction { done: 2900, total: 10000 });
    builder.run(Run::Plain, b"29% used");
    builder.close();
    builder.close();
    if revision == 1 || revision == 2 {
        builder.open(Role::Text);
        builder.state(skein_ui::State::ShownFolded);
        builder.run(Run::Plain, b"Extra");
        builder.close();
    }
    builder.open(Role::Paragraph);
    builder.text(Run::Plain, b"Sources");
    builder.close();
    builder.close();
}
