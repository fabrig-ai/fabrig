//! Accessibility roles and owned bytes, with no layout or application types.
//! Nodes are preorder; a subtree occupies `size` consecutive nodes including
//! its root (skein's ui/README.md, section 3). New builds have unassigned ids;
//! the diff assigns ids when the tree becomes visible.

use alloc::boxed::Box;
use skein_lib::{List, Token};

/// A visible node's identity, preserved while it matches across builds.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct NodeId(pub u32);

/// An opaque owner of tokens, keys, styles, regions, pictures and phrases.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Owner {
    pub number: u32,
}

/// A node's meaning, mapped exhaustively by every shell.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Role {
    /// A page node.
    Page,
    /// The banner landmark.
    Banner,
    /// The navigation landmark.
    Navigation,
    /// The main landmark.
    Main,
    /// The complementary landmark.
    Complementary,
    /// The content information landmark.
    ContentInfo,
    /// The form landmark.
    Form,
    /// The search landmark.
    Search,

    /// A region node.
    Region,
    /// An ordinary group of related nodes.
    Group,
    /// A heading at a semantic level.
    Heading { level: u8 },
    /// A paragraph node.
    Paragraph,
    /// A text node.
    Text,
    /// A list node.
    List,
    /// A item node.
    Item,
    /// A table node.
    Table,
    /// A row node.
    Row,
    /// A cell node.
    Cell,
    /// A header naming its column.
    ColumnHeader,
    /// A header naming its row.
    RowHeader,
    /// A named control choosing one option.
    Select,
    /// One option in a select.
    Option,
    /// A term node.
    Term,
    /// A description node.
    Description,
    /// A button node.
    Button,
    /// A link node.
    Link,
    /// A textfield node.
    TextField,
    /// A textarea node.
    TextArea,
    /// A choice node.
    Choice,
    /// A choicegroup node.
    ChoiceGroup,
    /// A check node.
    Check,
    /// A dialog node.
    Dialog,
    /// A status node.
    Status,
    /// A alert node.
    Alert,
    /// A log node.
    Log,
    /// A tree node.
    Tree,
    /// A treeitem node.
    TreeItem,
    /// A tabs node.
    Tabs,
    /// A tab node.
    Tab,
    /// A progress node.
    Progress,
    /// A time node.
    Time,
    /// A code node.
    Code,
    /// A quote node.
    Quote,
    /// A separator node.
    Separator,
    /// A named group of additional acts.
    Menu,
    /// A bound act inside a menu.
    MenuItem,
    /// A region a shell may fold.
    Disclosure,
    /// A named picture from its owner.
    Image,

    /// A figure node.
    Figure,
    /// A slot node.
    Slot,
    /// A drawing node.
    Drawing,
}

const FIELD: u32 = 1 << 0;
const COLLECTION: u32 = 1 << 1;
const POSITIONED: u32 = 1 << 2;
const CELL: u32 = 1 << 3;
const ROW_DATA: u32 = 1 << 4;
const LIST: u32 = 1 << 5;
const LIST_ITEM: u32 = 1 << 6;
const KEYED: u32 = 1 << 7;
const TOGGLE: u32 = 1 << 8;
const SELECTABLE: u32 = 1 << 9;
const PRESSABLE: u32 = 1 << 10;
const RUNS: u32 = 1 << 11;
const REQUIRED: u32 = 1 << 12;
const REGION: u32 = 1 << 13;
const NATIVE_REQ: u32 = 1 << 14;
const NATIVE_DISABLED: u32 = 1 << 15;
const NATIVE_FOCUS: u32 = 1 << 16;
const FOCUS: u32 = 1 << 17;
const WORDS_INSIDE: u32 = 1 << 18;
const DESCRIPTION_INSIDE: u32 = 1 << 19;

impl Role {
    /// The named sets this role is in; a new role breaks the build here.
    const fn sets(self) -> u32 {
        use Role::{Banner, Complementary, ContentInfo, Form, Log, Navigation, Region, Search, Tabs};

        match self {
            Self::Page
            | Self::Group
            | Self::Heading { .. }
            | Self::Paragraph
            | Self::Term
            | Self::Description
            | Self::Status
            | Self::Alert
            | Self::Time
            | Self::Code
            | Self::Quote
            | Self::Separator
            | Self::Menu
            | Self::Image
            | Self::Figure
            | Self::Drawing => 0,
            Banner | Navigation | Complementary | ContentInfo | Form | Search | Region | Log | Tabs => REGION,
            Self::Main => REGION | FOCUS,
            Self::Text | Self::Progress => RUNS,
            Self::List | Self::Tree => COLLECTION | LIST,
            Self::Item => POSITIONED | LIST_ITEM | KEYED | WORDS_INSIDE | DESCRIPTION_INSIDE,
            Self::Table => COLLECTION,
            Self::Row => POSITIONED,
            Self::Cell | Self::RowHeader => CELL | ROW_DATA | WORDS_INSIDE | DESCRIPTION_INSIDE,
            Self::ColumnHeader => CELL | WORDS_INSIDE | DESCRIPTION_INSIDE,
            Self::Select => REQUIRED | NATIVE_REQ | NATIVE_DISABLED | NATIVE_FOCUS | FOCUS,
            Self::Option => SELECTABLE | NATIVE_DISABLED,
            Self::Button => PRESSABLE | NATIVE_DISABLED | NATIVE_FOCUS | FOCUS,
            Self::Link | Self::Tab => PRESSABLE | NATIVE_FOCUS | FOCUS,
            Self::TextField | Self::TextArea => FIELD | REQUIRED | NATIVE_REQ | NATIVE_DISABLED | NATIVE_FOCUS | FOCUS,
            Self::Choice => TOGGLE | SELECTABLE | NATIVE_DISABLED | NATIVE_FOCUS | FOCUS,
            Self::ChoiceGroup => REQUIRED,
            Self::Check => TOGGLE | REQUIRED | NATIVE_REQ | NATIVE_DISABLED | NATIVE_FOCUS | FOCUS,
            Self::Dialog => REGION | NATIVE_FOCUS | FOCUS,
            Self::TreeItem => POSITIONED | LIST_ITEM | KEYED,
            Self::MenuItem => PRESSABLE | WORDS_INSIDE,
            Self::Disclosure => NATIVE_FOCUS | FOCUS,
            Self::Slot => KEYED,
        }
    }

    /// A field a person types in.
    #[must_use]
    pub const fn is_text_field(self) -> bool {
        self.sets() & FIELD != 0
    }
    /// A collection with a whole count.
    #[must_use]
    pub const fn is_collection(self) -> bool {
        self.sets() & COLLECTION != 0
    }
    /// An item or row with a position in its collection.
    #[must_use]
    pub const fn is_positioned(self) -> bool {
        self.sets() & POSITIONED != 0
    }
    /// A table cell, including its headers.
    #[must_use]
    pub const fn is_cell(self) -> bool {
        self.sets() & CELL != 0
    }
    /// A cell carrying row data.
    #[must_use]
    pub const fn is_row_data(self) -> bool {
        self.sets() & ROW_DATA != 0
    }
    /// A list or tree of items.
    #[must_use]
    pub const fn is_list(self) -> bool {
        self.sets() & LIST != 0
    }
    /// An item in a list or tree.
    #[must_use]
    pub const fn is_list_item(self) -> bool {
        self.sets() & LIST_ITEM != 0
    }
    /// A node identified by its object.
    #[must_use]
    pub const fn is_keyed(self) -> bool {
        self.sets() & KEYED != 0
    }
    /// A control a person toggles.
    #[must_use]
    pub const fn is_toggle(self) -> bool {
        self.sets() & TOGGLE != 0
    }
    /// A choice or option with native selection.
    #[must_use]
    pub const fn is_selectable(self) -> bool {
        self.sets() & SELECTABLE != 0
    }
    /// A node a person presses.
    #[must_use]
    pub const fn is_pressable(self) -> bool {
        self.sets() & PRESSABLE != 0
    }
    /// A node that holds text runs.
    #[must_use]
    pub const fn takes_runs(self) -> bool {
        self.sets() & RUNS != 0
    }
    /// A field or group that may be required.
    #[must_use]
    pub const fn takes_required(self) -> bool {
        self.sets() & REQUIRED != 0
    }
    /// A node that takes a region name.
    #[must_use]
    pub const fn takes_region(self) -> bool {
        self.sets() & REGION != 0
    }
    /// A control with a native required attribute.
    #[must_use]
    pub const fn has_native_required(self) -> bool {
        self.sets() & NATIVE_REQ != 0
    }
    /// A control with a native disabled attribute.
    #[must_use]
    pub const fn has_native_disabled(self) -> bool {
        self.sets() & NATIVE_DISABLED != 0
    }
    /// A node with native web focus handling.
    #[must_use]
    pub const fn has_native_focus(self) -> bool {
        self.sets() & NATIVE_FOCUS != 0
    }
    /// A node the web page may focus.
    #[must_use]
    pub const fn is_focusable(self) -> bool {
        self.sets() & FOCUS != 0
    }
    /// A node whose words stay inside its web element.
    #[must_use]
    pub const fn holds_words_inside(self) -> bool {
        self.sets() & WORDS_INSIDE != 0
    }
    /// A node whose description stays inside its web element.
    #[must_use]
    pub const fn describes_inside(self) -> bool {
        self.sets() & DESCRIPTION_INSIDE != 0
    }
}

/// A semantic state, never a visual style (ui/README.md, 3.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum State {
    /// The node is disabled.
    Disabled,
    /// The node is expanded.
    Expanded,
    /// The node is busy.
    Busy,
    /// The node is current.
    Current,
    /// The node is invalid.
    Invalid,
    /// The node is pressed.
    Pressed,
    /// The node is selected.
    Selected,
    /// The form needs this field filled.
    Required,
    /// The item is done: a step finished, a checklist entry met.
    Completed,
    /// The node is submitsonenter.
    SubmitsOnEnter,
    /// A disclosure folds on an outside press or Escape.
    FoldsAway,
    /// The log's last entry is not its list's last.
    Continues,
    /// A button takes the page's logs to their ends before its press reaches its owner where bound.
    ToLogEnd,
    /// A menu offering the choices of the nearest text field or area before it in its form.
    Options,
}

/// A token from an owner's closed style set.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Style {
    pub owner: Owner,
    pub token: u16,
}

/// A region's language-free name from its owner's closed set (ui/README.md, 8).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Region {
    pub owner: Owner,
    pub number: u16,
}
/// One of an owner's pictures, supplied when a shell is built (ui/README.md, 3.5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Picture {
    pub owner: Owner,
    pub number: u16,
}
/// A field's keyboard or control (ui/README.md, 3.4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EntryKind {
    Text,
    Number,
    Email,
    Telephone,
    Url,
    Date,
    Time,
    Secret,
}
/// A column header's order (ui/README.md, 3.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Sort {
    Ascending,
    Descending,
}
/// The rows and columns a cell spans, each at least one (ui/README.md, 3.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Spans {
    pub rows: u16,
    pub columns: u16,
}
/// Done of total, in its owner's numbers (ui/README.md, 3.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Fraction {
    pub done: u64,
    pub total: u64,
}
/// An image's owner picture or checked address (ui/README.md, 3.5).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Source {
    Picture(Picture),
    Address(Box<[u8]>),
}
/// A key a bound node answers to as a press (ui/README.md, 6.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Shortcut {
    pub key: u8,
    pub modifier: Modifier,
}
impl Shortcut {
    /// Escape is named only inside a dialog or a form (ui/README.md, 6.1 and 12).
    pub const ESCAPE: Shortcut = Shortcut { key: 0x1b, modifier: Modifier::Plain };
}
/// The modifier pressed with a shortcut's key.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Modifier {
    Plain,
    Primary,
    Alt,
}

/// The meaning of a span of text.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Run {
    /// Ordinary words.
    Plain,
    /// Emphasized words.
    Emphasis,
    /// Strongly emphasized words.
    Strong,
    /// Literal code.
    Code,
    /// A target interpreted by the text's owner.
    Link { to: Token },
}

/// One owned span on a text node.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TextRun {
    pub run: Run,
    pub text: Box<[u8]>,
}

/// A field's value and count of the owner's own writes.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Value {
    pub text: Box<[u8]>,
    pub written: u64,
}

/// The object an item represents, independent of its sibling position.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Key {
    pub owner: Owner,
    pub kind: u16,
    pub a: u64,
    pub b: u64,
}

/// A fragment requested from its owner's view.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SlotRef {
    pub owner: Owner,
    pub token: Token,
}

/// An owner's drawing vocabulary number.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DrawingKind {
    pub owner: Owner,
    pub kind: u16,
}

/// Opaque drawing data; ordinary child nodes provide its readable fallback.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Drawing {
    pub kind: DrawingKind,
    pub data: Box<[u8]>,
}

/// One node and its attributes; `size` includes its descendants.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Node {
    pub id: NodeId,
    pub role: Role,
    pub size: u32,
    pub name: Option<Box<[u8]>>,
    pub states: List<State>,
    pub styles: List<Style>,
    pub region: Option<Region>,
    pub shortcut: Option<Shortcut>,
    pub source: Option<Source>,
    pub description: Option<Box<[u8]>>,
    pub placeholder: Option<Box<[u8]>>,
    /// Words a shell with a pointer shows on hover, never needed to decide (ui/README.md, 3.2).
    pub hint: Option<Box<[u8]>>,
    pub entry: Option<EntryKind>,
    pub focus: Option<u64>,
    pub end_request: Option<u64>,
    pub count: Option<u32>,
    /// A list's order: its items are numbered, and this is the first one's number (ui/README.md, 3.2).
    pub ordered: Option<u32>,
    pub position: Option<u32>,
    pub spans: Option<Spans>,
    pub sort: Option<Sort>,
    pub fraction: Option<Fraction>,
    pub text: List<TextRun>,
    pub value: Option<Value>,
    pub link: Option<crate::Binding>,
    /// A checked external Markdown target, independent of application addresses.
    pub external: Option<Box<[u8]>>,
    pub key: Option<Key>,
    pub binding: Option<crate::Binding>,
    pub slot: Option<SlotRef>,
    pub drawing: Option<Drawing>,
}

impl Node {
    pub(crate) fn vacant() -> Node {
        Node {
            id: NodeId(0),
            role: Role::Text,
            size: 1,
            name: None,
            states: List::with_capacity(0),
            styles: List::with_capacity(0),
            region: None,
            shortcut: None,
            source: None,
            description: None,
            placeholder: None,
            hint: None,
            entry: None,
            focus: None,
            end_request: None,
            count: None,
            ordered: None,
            position: None,
            spans: None,
            sort: None,
            fraction: None,
            text: List::with_capacity(0),
            value: None,
            link: None,
            external: None,
            key: None,
            binding: None,
            slot: None,
            drawing: None,
        }
    }

    pub(crate) fn new(role: Role, limits: &crate::Limits, runs: u32) -> Node {
        Node {
            id: NodeId(0),
            role,
            size: 1,
            name: None,
            states: List::with_capacity(10),
            styles: List::with_capacity(limits.styles_per_node),
            region: None,
            shortcut: None,
            source: None,
            description: None,
            placeholder: None,
            hint: None,
            entry: None,
            focus: None,
            end_request: None,
            count: None,
            ordered: None,
            position: None,
            spans: None,
            sort: None,
            fraction: None,
            text: List::with_capacity(runs),
            value: None,
            link: None,
            external: None,
            key: None,
            binding: None,
            slot: None,
            drawing: None,
        }
    }

    /// The bytes the builder charges this node against `Limits::text_per_node`
    /// (ui/README.md, section 13).
    #[must_use]
    pub fn text_bytes(&self) -> u32 {
        let mut bytes = 0_u32;
        if let Some(name) = &self.name {
            bytes = length(name);
        }
        if let Some(description) = &self.description {
            bytes = bytes.checked_add(length(description)).expect("node bytes fit u32");
        }
        if let Some(placeholder) = &self.placeholder {
            bytes = bytes.checked_add(length(placeholder)).expect("node bytes fit u32");
        }
        if let Some(hint) = &self.hint {
            bytes = bytes.checked_add(length(hint)).expect("node bytes fit u32");
        }
        if let Some(source) = &self.source {
            match source {
                Source::Picture(_) => {}
                Source::Address(address) => bytes = bytes.checked_add(length(address)).expect("node bytes fit u32"),
            }
        }
        if let Some(value) = &self.value {
            bytes = bytes.checked_add(length(&value.text)).expect("node bytes fit u32");
        }
        if let Some(external) = &self.external {
            bytes = bytes.checked_add(length(external)).expect("node bytes fit u32");
        }
        for run in &self.text {
            bytes = bytes.checked_add(length(&run.text)).expect("node bytes fit u32");
        }
        bytes
    }
}

pub(crate) fn length(bytes: &[u8]) -> u32 {
    u32::try_from(bytes.len()).expect("view bytes fit u32")
}

/// A single tree in preorder, sized before building.
#[derive(Debug)]
pub struct Tree {
    pub(crate) nodes: List<Node>,
}

impl Tree {
    /// Reserve node storage once.
    #[must_use]
    pub fn new(capacity: u32) -> Tree {
        Tree { nodes: List::with_capacity(capacity) }
    }
    /// The tree's preorder nodes.
    #[must_use]
    pub fn nodes(&self) -> &[Node] {
        self.nodes.as_slice()
    }
    /// The finished tree's builder-charged storage over `Limits::EMPTY`
    /// (ui/README.md, section 13). Slots count references still in the tree.
    #[must_use]
    pub fn used(&self) -> crate::Limits {
        let mut used = crate::Limits { nodes: self.len(), ..crate::Limits::EMPTY };
        let mut ends = skein_lib::Stack::with_capacity(self.len());
        let mut index = 0_u32;
        for node in self.nodes() {
            while let Some(end) = ends.top() {
                if index < *end {
                    break;
                }
                ends.pop();
            }
            let end = index.checked_add(node.size).expect("subtree end fits u32");
            ends.push(end).expect("ancestor stack fits node count");
            used.depth = used.depth.max(ends.len());
            let bytes = node.text_bytes();
            used.text_per_node = used.text_per_node.max(bytes);
            used.text_bytes = used.text_bytes.checked_add(bytes).expect("tree bytes fit u32");
            if node.binding.is_some() || node.link.is_some() {
                used.bindings = used.bindings.checked_add(1).expect("binding count fits u32");
            }
            if node.slot.is_some() {
                used.slots = used.slots.checked_add(1).expect("slot count fits u32");
            }
            index = index.checked_add(1).expect("node count fits u32");
        }
        used
    }

    /// A node by its preorder index.
    #[must_use]
    pub fn get(&self, index: u32) -> Option<&Node> {
        self.nodes.get(index)
    }
    /// A node by its current visible identity.
    #[must_use]
    #[expect(clippy::manual_find, reason = "step code uses bounded scans without closures")]
    pub fn find(&self, id: NodeId) -> Option<&Node> {
        for node in &self.nodes {
            if node.id == id {
                return Some(node);
            }
        }
        None
    }
    /// Resolve an act's token in this displayed tree. A removed or unbound node
    /// gives nothing. A link's address binding also names its navigation act.
    #[must_use]
    pub fn bound(&self, id: NodeId) -> Option<crate::Binding> {
        if id == NodeId(0) {
            return None;
        }
        let node = self.find(id)?;
        node.binding.or(node.link)
    }

    /// Resolve the whole shell act; the owner interprets its token and value.
    #[must_use]
    pub fn act(&self, id: NodeId, act: crate::Act) -> Option<crate::BoundAct> {
        let binding = self.bound(id)?;
        let node = self.find(id)?;
        if self.disabled(id) || !crate::takes(node.role, node.states.as_slice().contains(&State::SubmitsOnEnter), &act)
        {
            return None;
        }
        Some(crate::BoundAct { binding, act })
    }

    /// A disabled node, including an option of its nearest disabled select.
    #[must_use]
    pub fn disabled(&self, id: NodeId) -> bool {
        let Some(node) = self.find(id) else { return false };
        if node.states.as_slice().contains(&State::Disabled) {
            return true;
        }
        if node.role != Role::Option {
            return false;
        }
        let mut option = None;
        for index in (0..self.len()).rev() {
            let ancestor = self.get(index).expect("preorder index exists");
            if ancestor.id == id {
                option = Some(index);
            } else if let Some(at) = option {
                let end = index.checked_add(ancestor.size).expect("subtree end fits u32");
                if end > at && ancestor.role == Role::Select {
                    return ancestor.states.as_slice().contains(&State::Disabled);
                }
            }
        }
        false
    }

    /// Ask the view root to translate this link owner's address for the shell.
    #[must_use]
    pub fn address(&self, id: NodeId) -> Option<crate::Binding> {
        if id == NodeId(0) {
            return None;
        }
        self.find(id)?.link
    }

    /// A checked external Markdown link for a shell to show directly.
    #[must_use]
    pub fn external(&self, id: NodeId) -> Option<&[u8]> {
        if id == NodeId(0) {
            return None;
        }
        self.find(id)?.external.as_deref()
    }

    /// The number of nodes, including every fallback child.
    #[must_use]
    pub fn len(&self) -> u32 {
        self.nodes.len()
    }
    /// Whether no nodes were built.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}
