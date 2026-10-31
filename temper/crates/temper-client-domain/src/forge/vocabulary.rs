//! The forge family in domain words, independent of its wire (client/README.md, 4.6).
use alloc::boxed::Box;
use jig_client::contract::{EntranceRefusal, Refusal};
/// The forge's one address: the adoption page for a project (client/README.md,
/// 4.5); reached from the navigation's link.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Address {
    Adopt { project: u32 },
}

/// A repository's role, as the forge connector's `Role`, in its order;
/// the form offers `Owned`, `Fork` and `Context`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    Owned,
    Adopted,
    Fork,
    Context,
}

/// A person's keyed adoption (03.2's `Adopt`, field for field): sent by
/// the form through the request table; ended by one `Answer`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Adoption {
    pub project: u32,
    pub home: bool,
    pub forge: u16,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub prefix: Box<[u8]>,
    pub role: Role,
    pub landing: Box<[u8]>,
    pub ci: bool,
    pub checks: Box<[u32]>,
}

/// What temper may do in an adopted repository (03.2's `Kinds`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[expect(clippy::struct_excessive_bools, reason = "each flag is one kind of the forge's, as the wire's record has it")]
pub struct Kinds {
    pub read: bool,
    pub push: bool,
    pub open: bool,
    pub land: bool,
    pub review: bool,
    pub status: bool,
    pub comment: bool,
    pub issue: bool,
    pub branch: bool,
}

/// A branch protection rule as the forge connector read it (03.2's `Rule`).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Rule {
    pub branch: Box<[u8]>,
    pub contexts: Box<[Box<[u8]>]>,
    pub approvals: u32,
    pub dismiss_stale: bool,
}

/// Whether branch protection was read (03.2's `Protection`).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Protection {
    Unknown,
    Absent,
    Rule(Rule),
}

/// The repository as the forge connector recorded it (03.2's `Adopted`).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Adopted {
    pub forge: u16,
    pub repository: u32,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub role: Role,
    pub home: bool,
    pub kinds: Kinds,
    pub protection: Protection,
}

/// The forge connector's reasons (03.2's `ForgeReason`; client/README.md, 4.7).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reason {
    NoSuchForge,
    NotFound,
    NoAccess,
    Refused,
    InFlight,
    TooLarge,
    Unreachable,
    InvalidAnswer,
    AnotherProject,
    HomeHeld,
    PrefixOccupied,
}

/// The adoption's terminal, as the routing decoded it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Answer {
    /// The forge connector adopted it.
    Adopted(Adopted),
    /// The core refused the adoption the root made on the party's behalf.
    CoreRefused(Refusal),
    /// The forge connector refused it.
    ForgeRefused(Reason),
    /// jig refused the request at admission; it never reached the connector.
    EntranceRefused(EntranceRefusal),
    /// The outcome's retention ended: whether it was adopted is unknown
    /// (03.2's `unknown`, sent for 02.2's `Delivery::Application` with none).
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Served {
    Unknown,
    Yes,
    OutOfDate,
}

/// The group's reason rendered as text: each invalid byte may need a UTF-8 replacement.
pub const PLAYGROUND_WHY_BYTES: u32 = temper_client_protocol_wire::SET_UP_WHY_BYTES.checked_mul(3).expect("reason text fits");

/// The first-password file path rendered as text: each invalid byte may need a UTF-8 replacement.
pub const OWNED_PASSWORD_BYTES: u32 =
    temper_client_protocol_wire::v1::CEILINGS.owned_forge_password.checked_mul(3).expect("password path text fits");

/// The playground as the client reads it (temper/docs/design/draft/playground-reset.md, 6.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Playground {
    pub state: PlaygroundState,
    pub forge: u16,
    pub project: Option<u32>,
    pub provider: u32,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub cues: Box<[Box<[u8]>]>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaygroundState {
    Starting,
    Running,
    Resetting,
}

/// The wire bounds on playground repository names.
pub const PLAYGROUND_FIELD_BYTES: u32 = temper_client_protocol_wire::v1::CEILINGS.playground_answer_owner;
