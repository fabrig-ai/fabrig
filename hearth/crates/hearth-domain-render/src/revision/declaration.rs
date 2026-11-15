//! Declaration maps (hearth/docs/design/protocol/format/tasks.md, 1).
pub use hearth_domain_render_vocabulary::revision::declaration::{Fragment, Map, Name};

use super::{Position, Value};
use alloc::boxed::Box;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Text {
    pub at: Position,
    pub text: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Number {
    pub at: Position,
    pub value: i64,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Declaration {
    pub at: Position,
    pub types: Box<[Type]>,
    pub settings: Box<[Spec]>,
    pub reads: Box<[Read]>,
    pub requires: Box<[Guard]>,
    pub scripts: Box<[Script]>,
    pub exports: Box<[Export]>,
    pub entries: Box<[Entries]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Script {
    pub fragment: Fragment,
    pub at: Position,
    pub order: u16,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Read {
    pub task: Text,
    pub at: Position,
    pub settings: Box<[Text]>,
    pub exports: bool,
    pub optional: bool,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Type {
    pub at: Position,
    pub form: Form,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Form {
    Bool,
    Int,
    String,
    Path,
    Mode,
    Duration,
    Secret,
    Target,
    File,
    Enum(Box<[Text]>),
    List(u32),
    Map(u32),
    Object(Box<[Spec]>),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Spec {
    pub name: Text,
    pub at: Position,
    pub ty: u32,
    pub default: Option<Value>,
    pub optional: bool,
    pub min: Option<Number>,
    pub max: Option<Number>,
    pub max_length: Option<Number>,
}

/// A declaration expression at its JSON string (hearth/docs/design/protocol/format/tasks.md, 9).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Condition {
    pub at: Position,
    pub expression: crate::evaluate::Expression,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Guard {
    pub name: Text,
    pub at: Position,
    pub when: Condition,
    pub message: Text,
}

/// A compiled entry key at its opening quote (hearth/docs/design/protocol/format/tasks.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Interpolated {
    Plain { at: Position, text: Box<[u8]> },
    Parts { at: Position, parts: Box<[crate::evaluate::template::Instruction]> },
}
impl Interpolated {
    #[must_use]
    pub const fn at(&self) -> Position {
        match self {
            Self::Plain { at, text: _ } | Self::Parts { at, parts: _ } => *at,
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Entries {
    pub map: Map,
    pub at: Position,
    pub entries: Box<[Entry]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Entry {
    pub key: Interpolated,
    pub at: Position,
    pub when: Option<Condition>,
    pub for_each: Option<Condition>,
    pub fields: Box<[Field]>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Field {
    pub name: Name,
    pub at: Position,
    pub value: Literal,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Literal {
    Text(Interpolated),
    Plain { at: Position, text: Box<[u8]> },
    Typed(Interpolated),
    Int(Number),
    Bool { at: Position, value: bool },
    Strings { at: Position, each: Box<[Interpolated]> },
    Range { at: Position, fields: Box<[Field]> },
}

/// One typed public value (hearth/docs/design/protocol/format/README.md, 6.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Export {
    pub name: Text,
    pub at: Position,
    pub ty: u32,
    pub optional: bool,
    pub value: Interpolated,
}
