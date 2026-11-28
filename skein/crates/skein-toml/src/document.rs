//! Compact owned text and records (skein/docs/design/data/toml.md, 3–5).

use alloc::boxed::Box;
use skein_lib::{List, usize_of};

use crate::Limits;

/// A table in the document it came from.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Table(u32);

/// An array (static or of tables) in the document it came from.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Array(u32);

/// How a table was defined. Implicit parents from headers are `Header` tables.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TableKind {
    Root,
    Header,
    Dotted,
    Inline,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DateTimeKind {
    OffsetDateTime,
    LocalDateTime,
    LocalDate,
    LocalTime,
}

/// A slice of checked UTF-8 in the document it came from.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Text {
    start: u32,
    end: u32,
}

/// One decoded key segment and its value. `line` is where the key first appeared.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Entry {
    pub key: Text,
    pub line: u64,
    pub value: Value,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Value {
    pub line: u64,
    pub kind: ValueKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ValueKind {
    String(Text),
    Integer(i64),
    Float(Text),
    Boolean(bool),
    DateTime { kind: DateTimeKind, text: Text },
    Array(Array),
    Table(Table),
    ArrayOfTables(Array),
}

/// An owned TOML tree. Its root is always a table. Use handles only with the
/// document that supplied them; navigation never allocates.
#[derive(Debug)]
pub struct Document {
    text: Box<[u8]>,
    root: Record,
    records: Box<[Record]>,
    children: Box<[u32]>,
}

impl Document {
    #[must_use]
    pub const fn root(&self) -> Table {
        Table(0)
    }

    #[must_use]
    pub fn text(&self, text: Text) -> Option<&[u8]> {
        self.text.get(usize_of(text.start)..usize_of(text.end))
    }

    #[must_use]
    pub fn table_kind(&self, table: Table) -> Option<TableKind> {
        Some(self.table(table)?.public())
    }

    #[must_use]
    pub fn table_len(&self, table: Table) -> Option<u32> {
        self.table(table)?;
        Some(self.record(table.0)?.len)
    }

    /// Entry by zero-based index in first-appearance order.
    #[must_use]
    pub fn entry(&self, table: Table, index: u32) -> Option<Entry> {
        self.table(table)?;
        self.child_entry(self.record(table.0)?, index)
    }

    /// Entry by decoded key segment (a dot is just a byte, not a path).
    #[must_use]
    pub fn get(&self, table: Table, key: &[u8]) -> Option<Entry> {
        let id = self.find(table, key)?;
        self.entry_at(id)
    }

    /// Radix of an integer entry as written (2, 8, 10 or 16), or none if not an integer.
    #[must_use]
    pub fn integer_radix(&self, table: Table, key: &[u8]) -> Option<u8> {
        let id = self.find(table, key)?;
        match self.record(id)?.kind {
            Stored::Integer { radix, .. } => Some(radix),
            Stored::Pending
            | Stored::String(_)
            | Stored::Float(_)
            | Stored::Boolean(_)
            | Stored::DateTime { .. }
            | Stored::Array
            | Stored::Table(_)
            | Stored::ArrayOfTables => None,
        }
    }

    fn find(&self, table: Table, key: &[u8]) -> Option<u32> {
        self.table(table)?;
        let parent = self.record(table.0)?;
        for index in 0..parent.len {
            let id = self.child(parent, index)?;
            if self.text(self.record(id)?.key)? == key {
                return Some(id);
            }
        }
        None
    }

    #[must_use]
    pub fn array_len(&self, array: Array) -> Option<u32> {
        Some(self.array(array)?.len)
    }

    /// Element by zero-based index, for both kinds of array.
    #[must_use]
    pub fn element(&self, array: Array, index: u32) -> Option<Value> {
        let id = self.child(self.array(array)?, index)?;
        self.record(id)?.value(id)
    }

    fn record(&self, id: u32) -> Option<&Record> {
        if id == 0 { Some(&self.root) } else { self.records.get(usize_of(id.checked_sub(1)?)) }
    }

    fn table(&self, table: Table) -> Option<TableState> {
        self.record(table.0)?.kind.table()
    }

    fn array(&self, array: Array) -> Option<&Record> {
        let record = self.record(array.0)?;
        match record.kind {
            Stored::Array | Stored::ArrayOfTables => Some(record),
            Stored::Pending
            | Stored::Table(_)
            | Stored::String(_)
            | Stored::Integer { .. }
            | Stored::Float(_)
            | Stored::Boolean(_)
            | Stored::DateTime { .. } => None,
        }
    }

    fn child(&self, parent: &Record, index: u32) -> Option<u32> {
        if index >= parent.len {
            return None;
        }
        self.children.get(usize_of(parent.start.checked_add(index)?)).copied()
    }

    fn child_entry(&self, parent: &Record, index: u32) -> Option<Entry> {
        let id = self.child(parent, index)?;
        self.entry_at(id)
    }

    fn entry_at(&self, id: u32) -> Option<Entry> {
        let record = self.record(id)?;
        Some(Entry { key: record.key, line: record.key_line, value: record.value(id)? })
    }
}

/// Heap retained by a document under `limits`, in bytes; excludes parsing scratch.
#[must_use]
pub fn document_bound(limits: &Limits) -> Option<u64> {
    let records = List::<Record>::price(limits.entries)?;
    let children = List::<u32>::price(limits.entries)?;
    records.checked_add(children)?.checked_add(u64::from(limits.input_bytes))
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TableState {
    Root,
    Implicit,
    Header,
    Dotted,
    Inline,
}

impl TableState {
    const fn public(self) -> TableKind {
        match self {
            Self::Root => TableKind::Root,
            Self::Implicit | Self::Header => TableKind::Header,
            Self::Dotted => TableKind::Dotted,
            Self::Inline => TableKind::Inline,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Stored {
    Pending,
    String(Text),
    Integer { number: i64, radix: u8 },
    Float(Text),
    Boolean(bool),
    DateTime { kind: DateTimeKind, text: Text },
    Array,
    Table(TableState),
    ArrayOfTables,
}

impl Stored {
    pub(crate) const fn table(self) -> Option<TableState> {
        match self {
            Self::Table(state) => Some(state),
            Self::Pending
            | Self::String(_)
            | Self::Integer { .. }
            | Self::Float(_)
            | Self::Boolean(_)
            | Self::DateTime { .. }
            | Self::Array
            | Self::ArrayOfTables => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Record {
    key: Text,
    key_line: u64,
    kind: Stored,
    value_line: u64,
    start: u32,
    len: u32,
}

impl Record {
    fn value(&self, id: u32) -> Option<Value> {
        let kind = match self.kind {
            Stored::Pending => return None,
            Stored::String(text) => ValueKind::String(text),
            Stored::Integer { number, .. } => ValueKind::Integer(number),
            Stored::Float(text) => ValueKind::Float(text),
            Stored::Boolean(flag) => ValueKind::Boolean(flag),
            Stored::DateTime { kind, text } => ValueKind::DateTime { kind, text },
            Stored::Array => ValueKind::Array(Array(id)),
            Stored::Table(_) => ValueKind::Table(Table(id)),
            Stored::ArrayOfTables => ValueKind::ArrayOfTables(Array(id)),
        };
        Some(Value { kind, line: self.value_line })
    }
}

/// During parsing a parent's children can grow at any time. Each one has an
/// ID in insertion order and a link to its next sibling. Packing walks those
/// links once to give each parent a contiguous range of child IDs.
#[derive(Debug)]
struct Node {
    record: Record,
    first: u32,
    last: u32,
    next: u32,
}

impl Node {
    const fn new(key: Text, line: u64, kind: Stored) -> Node {
        Node {
            record: Record { key, key_line: line, kind, value_line: line, start: 0, len: 0 },
            first: 0,
            last: 0,
            next: 0,
        }
    }
}

pub(crate) struct Build {
    pub(crate) text: List<u8>,
    root: Node,
    nodes: List<Node>,
}

impl Text {
    pub(crate) const EMPTY: Text = Text { start: 0, end: 0 };

    pub(crate) fn since(start: u32, buffer: &List<u8>) -> Text {
        Text { start, end: buffer.len() }
    }

    fn in_buffer(self, buffer: &List<u8>) -> &[u8] {
        buffer.as_slice().get(usize_of(self.start)..usize_of(self.end)).expect("decoded text is in its buffer")
    }
}

impl Build {
    pub(crate) fn new(limits: &Limits) -> Build {
        Build {
            root: Node::new(Text::EMPTY, 1, Stored::Table(TableState::Root)),
            text: List::with_capacity(limits.input_bytes),
            nodes: List::with_capacity(limits.entries),
        }
    }

    pub(crate) fn append(&mut self, parent: u32, key: Text, line: u64, kind: Stored) -> Option<u32> {
        self.nodes.push(Node::new(key, line, kind)).ok()?;
        let id = self.nodes.len();
        let last = self.last(parent);
        if last != 0 {
            self.node_mut(last).next = id;
        }
        let parent = self.node_mut(parent);
        if last == 0 {
            parent.first = id;
        }
        parent.last = id;
        parent.record.len = parent.record.len.checked_add(1).expect("child count is within the admitted node count");
        Some(id)
    }

    pub(crate) fn find(&self, parent: u32, key: Text) -> Option<u32> {
        let key = key.in_buffer(&self.text);
        let parent = self.node(parent);
        let mut id = parent.first;
        for _ in 0..parent.record.len {
            let node = self.node(id);
            if node.record.key.in_buffer(&self.text) == key {
                return Some(id);
            }
            id = node.next;
        }
        None
    }

    pub(crate) fn kind(&self, id: u32) -> Stored {
        self.node(id).record.kind
    }

    pub(crate) fn set_kind(&mut self, id: u32, kind: Stored) {
        self.node_mut(id).record.kind = kind;
    }

    pub(crate) fn set_line(&mut self, id: u32, line: u64) {
        self.node_mut(id).record.value_line = line;
    }

    pub(crate) fn last(&self, id: u32) -> u32 {
        self.node(id).last
    }

    fn node(&self, id: u32) -> &Node {
        if id == 0 { &self.root } else { self.nodes.get(id.saturating_sub(1)).expect("assigned node ID") }
    }

    fn node_mut(&mut self, id: u32) -> &mut Node {
        if id == 0 { &mut self.root } else { self.nodes.get_mut(id.saturating_sub(1)).expect("assigned node ID") }
    }

    pub(crate) fn finish(self) -> Document {
        let mut children = List::allocated(self.nodes.len());
        let mut records = List::allocated(self.nodes.len());
        let root = self.pack(&self.root, &mut children);
        for node in &self.nodes {
            let record = self.pack(node, &mut children);
            records.push(record).expect("one record per admitted node");
        }
        Document { root, text: self.text.into_boxed(), records: records.into_boxed(), children: children.into_boxed() }
    }

    fn pack(&self, parent: &Node, children: &mut List<u32>) -> Record {
        let mut record = parent.record;
        record.start = children.len();
        let mut next = parent.first;
        for _ in 0..record.len {
            children.push(next).expect("one child index per admitted node");
            next = self.node(next).next;
        }
        record
    }
}
