//! The link's shared values and worker membership (jig/docs/design/domain/root.md, 2; domain/hosts/hub.md, 1).
//! The fleet, hub and accounts share these values; the crate holds no domain state.
//! Account lists follow jig/docs/design/protocol/channel/records.md, 1.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::boxed::Box;
use skein_lib::{Duration, Token, Wall};

/// How a root takes an event (jig/docs/design/domain/root.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Class {
    /// Finishes or advances admitted work, taken at any backlog.
    Admitted,
    /// New work, handed back unchanged at the watermark.
    New,
    /// Its admitted part is stepped; its new part waits at the watermark.
    Mixed,
}

impl Class {
    /// A child's `new_work`, as a class.
    #[must_use]
    pub const fn of(new_work: bool) -> Self {
        if new_work { Self::New } else { Self::Admitted }
    }
}

/// The engine's pause ask (jig/docs/design/protocol/channel/pause.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PauseMode {
    Resume,
    Pause,
    Drain,
}

/// The worker's pause state (jig/docs/design/protocol/channel/pause.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PauseState {
    Taking,
    Paused,
    Draining,
    Drained,
}

/// A configuration-directory name (domain/accounts-sources.md, 1), never an account number.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SourceName(skein_lib::Name<32>);

impl SourceName {
    #[must_use]
    pub fn new(value: &[u8]) -> Option<Self> {
        Some(Self(skein_lib::Name::new(value, skein_lib::Grammar::LowercaseDigitsHyphen)?))
    }
    /// Reconstruct stored bytes without judging their spelling (domain/engine/restart.md).
    #[must_use]
    pub fn from_stored(value: &[u8]) -> Option<Self> {
        Some(Self(skein_lib::Name::from_checked(value)?))
    }
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// A position admitted from the current worker settings.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Worker {
    position: u32,
}
impl Worker {
    #[must_use]
    pub const fn number(self) -> u32 {
        self.position
    }
}

/// A startup value exceeds the priced configured membership.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Problem {
    Workers { limit: u32 },
    WorkerName { worker: u32, limit: u32 },
}

/// A fleet member's stable identity and admission places (domain/engine/workers.md, 4).
#[derive(Hash, Debug)]
pub struct Member {
    pub number: u32,
    pub name: Box<[u8]>,
    pub places: u32,
    pub withdrawn: bool,
}
/// Current worker settings, including withdrawn identities for link observations.
#[derive(Hash, Debug)]
pub struct Membership {
    members: Box<[Member]>,
    extent: u32,
}
impl Membership {
    #[must_use]
    pub fn empty() -> Self {
        Self { members: Box::new([]), extent: 0 }
    }
    /// # Errors
    /// A worker number or name exceeds its priced bound.
    pub fn new(members: Box<[Member]>, workers: u32, name_bytes: u32) -> Result<Self, Problem> {
        let mut extent = 0;
        for member in &members {
            if member.number == 0 || member.number > workers {
                return Err(Problem::Workers { limit: workers });
            }
            let worker = member.number.checked_sub(1).expect("positive worker number");
            if member.name.len() > skein_lib::usize_of(name_bytes) {
                return Err(Problem::WorkerName { worker, limit: name_bytes });
            }
            extent = extent.max(member.number);
        }
        Ok(Self { members, extent })
    }
    #[must_use]
    pub const fn len(&self) -> u32 {
        self.extent
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
    #[must_use]
    pub fn worker(&self, position: u32) -> Option<Worker> {
        self.places(position)?;
        Some(Worker { position })
    }
    #[expect(clippy::manual_find, reason = "step code uses an explicit bounded scan, not a closure")]
    fn member(&self, position: u32) -> Option<&Member> {
        let number = position.checked_add(1)?;
        for member in &self.members {
            if member.number == number {
                return Some(member);
            }
        }
        None
    }
    #[must_use]
    pub fn places(&self, position: u32) -> Option<u32> {
        let member = self.member(position)?;
        if member.withdrawn { None } else { Some(member.places) }
    }
    #[must_use]
    pub fn get(&self, position: u32) -> Option<&[u8]> {
        Some(&self.member(position)?.name)
    }
    #[must_use]
    pub fn name(&self, worker: Worker) -> &[u8] {
        self.get(worker.position).expect("admitted worker identity stays in membership")
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod membership_tests;

/// Maximum endpoint identities admitted in one host hello (domain/hosts/charters.md, 1).
/// Independent of the number of model declarations.
pub const PROGRAM_ENDPOINTS: u32 = 64;

/// Reject an oversized or ambiguous endpoint declaration before retaining it.
/// A quadratic prefix scan, bounded by 64 endpoints: at most 2,016 comparisons.
#[must_use]
pub fn valid_program_endpoints(endpoints: &[u32]) -> bool {
    if endpoints.len() > skein_lib::length::usize_of(PROGRAM_ENDPOINTS) {
        return false;
    }
    for (at, endpoint) in endpoints.iter().enumerate() {
        if endpoints.get(..at).expect("prefix").contains(endpoint) {
            return false;
        }
    }
    true
}

/// The program's bounded, printable payment method, without login contents.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct LoginMethod(skein_lib::Name<32>);

impl LoginMethod {
    #[must_use]
    pub fn new(value: &[u8]) -> Option<Self> {
        Some(Self(skein_lib::Name::new(value, skein_lib::Grammar::PrintableAscii)?))
    }
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SourceFailure {
    Missing,
    Deadline,
    Unreadable,
    Exited,
    Unconfigured,
    Unknown,
    Closed,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum LoginState {
    Usable,
    NotSignedIn,
    SignedInOtherwise(LoginMethod),
    Failed { reason: SourceFailure, detail: Box<[u8]> },
}
impl LoginState {
    /// A failed check does not prove absence, except when the program is unconfigured.
    #[must_use]
    pub fn kept(&self) -> bool {
        match self {
            Self::NotSignedIn | Self::Failed { reason: SourceFailure::Unconfigured, .. } => false,
            Self::Usable | Self::SignedInOtherwise(_) | Self::Failed { .. } => true,
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Login {
    pub name: SourceName,
    pub state: LoginState,
    pub email: Option<Box<[u8]>>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SignOutResult {
    SignedOut,
    Failed { reason: SourceFailure, detail: Box<[u8]> },
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct EndpointBound {
    pub endpoint: u32,
    pub window: u32,
    pub output: u32,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Bounds {
    pub program_endpoints: Box<[u32]>,
    pub endpoints: Box<[EndpointBound]>,
    pub turns: u32,
    pub spend: u64,
    pub time: Duration,
    pub waiting: Duration,
    pub checks: Duration,
    pub message_bytes: u32,
    pub inbox: u32,
    pub unacknowledged: u32,
    /// The longest host answer the agent takes, in bytes, if the application names it
    /// (jig/docs/design/domain/hosts/README.md, section 2).
    pub reply_bytes: Option<u32>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Grant {
    pub form: Form,
    pub account: u32,
    pub generation: u64,
    pub valid: Duration,
}

/// The form in which a grant lends its value (protocol/channel/README.md, section 7).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Form {
    /// A credential with the scheme its provider takes.
    Credential(Scheme),
    /// An application's credential, held in its own table.
    Application,
    /// A program source lent by its configuration-directory name.
    Program(SourceName),
}

/// A credential's scheme (protocol/channel/README.md, section 7).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Scheme {
    /// A bearer credential.
    Bearer,
    /// A key credential.
    Key,
}

impl Form {
    /// The protocol table that holds this form's value.
    #[must_use]
    pub const fn table(self) -> GrantTable {
        match self {
            Self::Credential(_) => GrantTable::Accounts,
            Self::Application => GrantTable::Application,
            Self::Program(_) => GrantTable::Name,
        }
    }
}

/// The protocol layer's table that holds a credential (protocol/channel/README.md, section 7).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GrantTable {
    Accounts,
    Application,
    /// The value is the form's name; no table entry is held.
    Name,
}

/// What a worker says first on every channel (domain/engine/fleet.md).
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Hello {
    /// Job kinds this host runs, ascending and unique (jig/docs/design/domain/hosts/hub.md, 9).
    pub jobs: Box<[JobBound]>,
    /// Whether the agreed channel carries usage reads (protocol/channel/records.md, 1).
    pub reads_usage: bool,
    pub bounds: Bounds,
    /// The worker's longest stop time after losing contact. It must be
    /// strictly below the engine's grace.
    pub stop_bound: Duration,
    /// How many runs it hosts at once: none once it is shutting down.
    pub slots: u32,
    pub workstreams: Box<[u64]>,
    /// The runs it hosts, and those whose answers it holds.
    pub hosting: Box<[Hosting]>,
    /// No hello of this worker process has yet been admitted (domain/engine/executors.md, 6.5).
    pub fresh: bool,
}

/// A run a worker hosts, or whose answer it holds.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Hosting {
    pub run: Token,
    pub attempt: Token,
    pub phase: Phase,
}

/// Where a hosted run is in its lifecycle (jig/docs/design/domain/hosts/hub.md,
/// 2), as a hello says.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Phase {
    Preparing,
    Starting,
    /// Its agent is at work.
    Active,
    /// It yielded, and waits for its next inbound event.
    Waiting,
    /// How it ends is decided, and it answers next.
    Ending,
    /// It answered while there was no channel: the answer follows the hello.
    Answered,
}

#[cfg(test)]
mod tests {
    use super::{Class, Form, GrantTable, Scheme};

    #[test]
    fn a_childs_new_work_names_its_class() {
        assert_eq!(Class::of(true), Class::New);
        assert_eq!(Class::of(false), Class::Admitted);
    }

    #[test]
    fn a_form_names_its_table() {
        for (form, table) in [
            (Form::Credential(Scheme::Bearer), GrantTable::Accounts),
            (Form::Credential(Scheme::Key), GrantTable::Accounts),
            (Form::Application, GrantTable::Application),
            (Form::Program(crate::SourceName::new(b"claude").expect("source name")), GrantTable::Name),
        ] {
            assert_eq!(form.table(), table);
        }
    }

    #[test]
    fn a_form_takes_34_bytes() {
        // tag 1 + length 1 + 32 name bytes.
        assert_eq!(size_of::<Form>(), 34);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Operation {
    Checks,
    Completion,
    Command,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CountKind {
    Admitted,
    Ended,
    LlmStarted,
    LlmRetried,
    LlmFinished,
    ToolStarted,
    ToolFinished,
    CheckStarted,
    CheckFinished,
    TextArrived,
}

#[cfg(test)]
mod grammar_tests {
    use super::{LoginMethod, SourceName};
    #[test]
    fn source_names_and_methods_have_separate_grammars() {
        for value in [b"a".as_slice(), b"source-123", &[b'z'; 32]] {
            assert_eq!(SourceName::new(value).expect("source name").as_bytes(), value);
        }
        for value in [b"".as_slice(), b"A", b"../a", b"a_b", &[b'a'; 33], b"\x80"] {
            assert!(SourceName::new(value).is_none(), "{value:?}");
        }
        for value in [b"!~".as_slice(), b"claude.ai", &[b'X'; 32]] {
            assert_eq!(LoginMethod::new(value).expect("printable method").as_bytes(), value);
        }
        for value in [b"".as_slice(), b"two words", b"\x7f", b"\n", &[b'x'; 33]] {
            assert!(LoginMethod::new(value).is_none(), "{value:?}");
        }
    }
}

/// An application's job kind (jig/docs/design/domain/hosts/hub.md, 9).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct JobKind(pub u32);

/// The bounds of a declared job kind (jig/docs/design/domain/hosts/hub.md, 9).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct JobBound {
    pub kind: JobKind,
    pub spec: u32,
    pub result: u32,
    pub deadline: Duration,
}

/// Account list bounds and values (jig/docs/design/protocol/channel/records.md, 1).
pub const LIST_ROWS: u32 = 256;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ListedModel {
    pub name: Box<[u8]>,
    pub shown: Option<Box<[u8]>>,
    pub window: Option<u32>,
    pub output: Option<u32>,
    pub efforts: Box<[Box<[u8]>]>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ListFailure {
    Refused { status: u16 },
    Unreachable,
    Unreadable,
    Deadline,
    Agent,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ListOutcome {
    Listed(Box<[ListedModel]>),
    Failed(ListFailure),
}

/// An application's read-only answer (jig/docs/design/domain/hosts/hub.md, 10).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum AskAnswer {
    Bytes(Box<[u8]>),
    Refused,
}

/// A program's usage window (jig/docs/design/domain/accounts-lists.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FactWindow {
    pub length: Option<Duration>,
    pub used: Option<u16>,
    pub resets: Option<Wall>,
    pub state: u16,
}
pub const WINDOW_OPEN: u16 = 1;
pub const WINDOW_WARNED: u16 = 2;
pub const WINDOW_LIMITED: u16 = 3;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct UsageWindow {
    pub label: Box<[u8]>,
    pub window: FactWindow,
}
pub const USAGE_WINDOWS: u32 = 8;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum UsageOutcome {
    Read(Box<[UsageWindow]>),
    NotSignedIn,
    Spent,
    Failed { reason: SourceFailure, detail: Box<[u8]> },
}
