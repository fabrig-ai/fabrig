//! Decode errors retain their revision places (hearth/docs/design/domain/render/errors.md, 1).
use crate::revision::{self, Oid, Position};
pub use crate::vocabulary::Class;
use alloc::boxed::Box;
pub mod syntax;
pub use syntax::Syntax;
mod reserved;
pub use reserved::{Reserved, Rule};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Error {
    pub side: Option<crate::diff::Side>,
    pub kind: Kind,
    pub class: Class,
    pub at: Option<Place>,
    pub also: Option<Place>,
    pub target: Option<Box<[u8]>>,
    pub task: Option<Box<[u8]>>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Place {
    pub path: Box<[u8]>,
    pub position: Option<Position>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    Reserved(Reserved),
    Exported(Exported),
    Ids(Ids),

    Resolution(Resolution),
    Guard(Guard),
    Collision(Collision),
    Store(StoreError),
    Evaluation(crate::evaluate::FaultKind),
    Schema(Schema),
    Decode(Decode),
    Repository(Repository),
    Tool(Tool),
    Composition(Composition),
}
impl Kind {
    #[must_use]
    pub fn tool(&self) -> Option<&Tool> {
        match self {
            Self::Tool(tool) => Some(tool),
            Self::Reserved(_)
            | Self::Exported(_)
            | Self::Ids(_)
            | Self::Resolution(_)
            | Self::Guard(_)
            | Self::Collision(_)
            | Self::Store(_)
            | Self::Evaluation(_)
            | Self::Schema(_)
            | Self::Decode(_)
            | Self::Repository(_)
            | Self::Composition(_) => None,
        }
    }
    pub fn store_mut(&mut self) -> Option<&mut StoreError> {
        match self {
            Self::Store(error) => Some(error),
            Self::Reserved(_)
            | Self::Exported(_)
            | Self::Ids(_)
            | Self::Resolution(_)
            | Self::Guard(_)
            | Self::Collision(_)
            | Self::Evaluation(_)
            | Self::Schema(_)
            | Self::Decode(_)
            | Self::Repository(_)
            | Self::Tool(_)
            | Self::Composition(_) => None,
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Decode {
    Syntax(Syntax),
    Declaration(Declared),
    ListNull,
    Unexpected,
    Trailing,
    TooDeep,
    StringTooLong,
    NumberTooLong,
    Number,
    Escape,
    Surrogate,
    Utf8,
    Control,
    Truncated,
    NotObject,
    NotInteger,
    Range,
    Nul,
    Keys,
    Note,
    Name,
    Placement,
    Members,
    Member,
    Duplicate { key: Box<[u8]> },
    Settings { task: Box<[u8]> },
    Unknown { field: Box<[u8]> },
    Wrong { field: Field },
    Missing { field: Field },
    Misplaced { field: Field },
    TooLarge { limit: Limit, max: u64 },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Field {
    Kind,
    Firmware,
    Hardware,
    On,
    Mutable,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Limit {
    Record,
    Closure,
    Template,
    Document,
    Decoded,
    Static,
    Entries,
    Path,
    Paths,
    Tasks,
    Targets,
}
/// Why a decoded task.json is no declaration (hearth/docs/design/protocol/format/tasks.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Declared {
    Unknown { of: Object },
    Later,
    Wrong { expected: Expected },
    Missing { key: Required },
    Name,
    Doc,
    Order,
    Optional,
    Bound,
    Above,
    Enum,
    Settings,
    Secret,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Object {
    Declaration,
    Read,
    Guard,
    Script,
    Bless,
    Entry(revision::declaration::Map),
    Range,
    Spec,
    Export,
    Type,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Expected {
    Object,
    String,
    Strings,
    Bool,
    Int,
    Field(Takes),
    True,
    Type,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Required {
    If,
    Message,
    Type,
    Doc,
    Value,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Takes {
    Text,
    Int,
    Bool,
    Strings,
    Range,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Schema {
    Source { first: revision::declaration::Name, second: revision::declaration::Name },
    Export { fault: ExportFault },
    Unlisted { reader: Box<[u8]>, root: Root, task: Box<[u8]>, setting: Option<Box<[u8]>> },
    Dynamic { root: Root },
    Task { task: Box<[u8]> },
    Unread { task: Box<[u8]>, setting: Box<[u8]>, declared: Box<[Box<[u8]>]> },
    Wrong { expected: TypeName },
    Enum { value: Box<[u8]>, listed: Box<[Box<[u8]>]> },
    Beneath { allowed: Box<[Box<[u8]>]> },
    Below { min: i64 },
    Above { max: i64 },
    Length { max_length: i64 },
    Null,
    UndeclaredField { field: Box<[u8]>, declared: Box<[Box<[u8]>]> },
    MissingField { field: Box<[u8]> },
    Undeclared { setting: Box<[u8]>, declared: Box<[Box<[u8]>]> },
    Missing { setting: Box<[u8]> },
    Absent { task: Box<[u8]> },
    Malformed { format: Format },
    Target { name: Box<[u8]>, environment: Box<[u8]> },
    File { path: Box<[u8]> },
    Large { path: Box<[u8]>, max: u64 },
    Main,
    Security,
    Component { package: Box<[u8]>, component: Box<[u8]> },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Format {
    DataName,
    Sha256,
    Link,
    Path,
    Mode,
    Duration,
    Secret,
    File,
    Key,
    Timestamp,
    Debian,
    Preseed,
    Module,
    Param,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TypeName {
    Bool,
    Int,
    String,
    Enum,
    List,
    Map,
    Object,
    Path,
    Mode,
    Duration,
    Secret,
    Target,
    File,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Repository {
    NoTree,
    NoTemplate,
    NoScript,
    Unused,
    LinkTarget,
    Stray,
    Environment { env: Box<[u8]> },
    Name,
    Declaration,
    Static,
    Destination,
    Submodule,
    Link,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Tool {
    pub command: Command,
    pub ended: Ended,
    pub tail: Box<[Box<[u8]>]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Command {
    Mmdebstrap,
    RevParse,
    LsTree,
    CatFile,
    Diff,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Ended {
    Timeout,
    Start,
    Exit(u8),
    Signal(u32),
    Output,
    Missing { oid: Oid },
    Stream,
}
/// A store failure in an answer (hearth/docs/design/domain/render/errors.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct StoreError {
    pub store: Box<[u8]>,
    pub fault: StoreFault,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum StoreFault {
    Full { limit: StoreLimit, most: u32 },
    Failed { step: Step },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum StoreLimit {
    Resolutions,
    CacheFiles,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Step {
    Write,
    List,
    Remove,
    Load,
    Empty,
    Keep,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Composition {
    Duplicate,
    Long,
    Placement,
    Members,
    Unplaced,
    On { on: Box<[u8]>, host: Host },
    Null,
    Task,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Host {
    Unplaced,
    Unknown,
    Elsewhere { environment: Box<[u8]> },
    Container,
}
/// A false guard's name and message (hearth/docs/design/protocol/format/tasks.md, 9).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Guard {
    pub name: Box<[u8]>,
    pub message: Box<[u8]>,
}
/// A key produced twice by one task's expanded maps (hearth/docs/design/protocol/format/tasks.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Collision {
    Key { key: Box<[u8]> },
    Given { subject: Subject, name: Box<[u8]>, first: Box<[u8]>, second: Box<[u8]> },
    Differs { subject: Subject, name: Box<[u8]>, first: Box<[u8]>, second: Box<[u8]> },
    Shared { id: Id, value: u32, first: Box<[u8]>, second: Box<[u8]> },
    Hidden { path: Box<[u8]>, data: Box<[u8]>, first: Box<[u8]>, second: Box<[u8]> },
    Overlap { subject: Subject, first: Box<[u8]>, second: Box<[u8]> },
}
/// The namespace of a collision (hearth/docs/design/domain/render/README.md, 3.7).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Subject {
    Tree,
    DataName,
    DataPath,
    Secret,
    Identity,
    Download,
    User,
    Group,
    Preset,
    Tmpfiles,
    Preseed,
    Parameter,
    Range,
}
/// The account id namespace (hearth/docs/design/protocol/format/tasks.md, 6).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Id {
    Uid,
    Gid,
}
/// Reference roots restricted to literal names (hearth/docs/design/protocol/format/README.md, 5.4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Root {
    Debian,
    Tasks,
    Exports,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ExportFault {
    Form,
    Root { root: Root },
}
/// Another target's exports failed (hearth/docs/design/domain/render/README.md, 3.4).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Exported {
    pub exporter: Box<[u8]>,
    pub task: Box<[u8]>,
    pub cause: Cause,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Cause {
    Decode(Decode),
    Repository(Repository),
    Composition(Composition),
    Schema(Schema),
    Evaluation(crate::evaluate::FaultKind),
}
/// Apt's configuration verdict and the bounded closure (hearth/docs/design/domain/render/spec.md, 2.5, hearth/docs/design/domain/render/errors.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Resolution {
    TooLarge { max: u64 },
    Unknown { package: Box<[u8]> },
    Unsatisfiable { package: Option<Box<[u8]>> },
    Absent { uri: Box<[u8]>, suite: Box<[u8]> },
    Missing { package: Box<[u8]> },
    Unmatched { name: Box<[u8]>, version: Box<[u8]>, architecture: Box<[u8]> },
    Disagree { name: Box<[u8]>, version: Box<[u8]>, architecture: Box<[u8]> },
    TooMany { max: u32 },
}
/// A fixed id is missing for an entry's owner (hearth/docs/design/domain/render/errors.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Ids {
    pub owner: Box<[u8]>,
    pub of: Owns,
    pub entry: Box<[u8]>,
    pub why: Why,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Owns {
    Data,
    Secret,
    Identity,
    Download,
    User,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Why {
    NoUser,
    NoGroup,
    NoUid,
    NoGid,
    OneId,
    NotPackageUser,
    NotPackageGroup,
}
