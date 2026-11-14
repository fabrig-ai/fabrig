//! Values crossing the connector top's parent boundary. Durable records are
//! whole rows: the parent saves them in the decision and only then sends
//! `Committed` for a new outbox entry (jig's domain/connectors.md, 4.2).
use alloc::boxed::Box;
use skein_lib::Token;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_issues as issues;
use temper_engine_domain_forge_landing as landing;

/// A pull request whose context the connector gathers for a run brief.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BriefItem {
    /// Its repository in the deployment's adopted set.
    pub repository: u32,
    /// Its provider number.
    pub number: u64,
}

/// A pinned head for one brief read.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BriefCommit(pub [u8; 32]);

/// A charter's document to gather (temper/docs/design/protocol/configuration/settings.md, 6).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct BriefDocument {
    pub host: Box<[u8]>,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    /// The branch's segments joined by `/`.
    pub branch: Box<[u8]>,
    pub path: Box<[u8]>,
    pub heading: Option<Box<[u8]>>,
}

/// Which forge context the connector owns for a brief.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BriefSource {
    /// A document at its branch head.
    Document,
    /// Failed checks and their logs on a pinned head.
    Ci { item: BriefItem, head: BriefCommit },
    /// Reviews and remarks on a pinned head.
    Reviews { item: BriefItem, head: BriefCommit },
    /// A person's request at the reviewed head, pinned to the change's head (domain/forge/change/repairs.md).
    Request { item: BriefItem, head: BriefCommit, reviewed: BriefCommit },
    /// A pull request against its base at a pinned head.
    Pull { item: BriefItem, head: BriefCommit },
    /// Why a task reads its project's adopted home without pushing.
    Home { repository: skein_forge::api::Repository, design: bool },
}

/// The connector's bounded section, or a failed gather.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum BriefRead {
    /// The bytes the connector rendered, and bytes left out at its read bound.
    Got { bytes: Box<[u8]>, left: u64 },
    /// The pinned section could not be read.
    Failed(crate::BriefUnread),
}

/// A resource of one configured forge repository (domain/forge/README.md, section 2).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Name {
    pub forge: u16,
    pub repository: u32,
    pub what: What,
}
/// The kind and identity within a repository.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum What {
    /// The repository itself.
    Repository,
    /// A branch's path segments.
    Branch(Box<[Box<[u8]>]>),
    /// A numbered pull request.
    Pull(u64),
    /// A numbered issue.
    Issue(u64),
}
/// What the project may do on an adopted repository.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Role {
    /// Temper owns the repository.
    Owned,
    /// Temper may contribute and land in it.
    Adopted,
    /// Temper contributes through a fork.
    Fork,
    /// Temper reads but never writes.
    Context,
}
/// Capabilities observed when the repository was adopted, narrowed on refusal.
#[expect(clippy::struct_excessive_bools, reason = "the forge has independent effect kinds")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
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
/// Forge read grants (domain/forge/README.md, section 2).
pub const READ_KINDS: [u16; 1] = [1];

/// Forge grants needed by the change procedure (domain/forge/README.md, section 4 and domain/forge/change/README.md).
pub const CHANGE_KINDS: [u16; 7] = [2, 3, 4, 10, 11, 12, 17];

/// One adopted repository, with provider name and project ownership.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Repository {
    pub project: u32,
    /// Other projects reading this repository (domain/forge/follows.md, 8).
    pub contexts: Box<[u32]>,
    /// This project's selected repository for goal issue projections and chats' saved work.
    pub home: bool,
    pub provider: skein_forge::api::Repository,
    pub host: Box<[u8]>,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub prefix: Box<[u8]>,
    pub role: Role,
    /// Whether this repository runs CI; absence of a status never selects this policy.
    pub ci: bool,
    /// Blocking project check gates used when CI is absent.
    pub checks: Box<[u32]>,
    pub kinds: Kinds,
    pub protection: Protection,
    pub settings: skein_forge::api::Settings,
    /// CI workflow file names (temper/docs/design/domain/forge/adoption.md, 7).
    pub workflows: Box<[Box<[u8]>]>,
}
impl Repository {
    /// This project's adoption role (domain/forge/follows.md, 8).
    #[must_use]
    pub fn role_for(&self, project: u32) -> Option<Role> {
        if self.project == project {
            Some(self.role)
        } else if self.contexts.contains(&project) {
            Some(Role::Context)
        } else {
            None
        }
    }

    /// Project-scoped facts; context never carries a writer's home or effects.
    #[must_use]
    pub fn for_project(&self, project: u32) -> Option<Self> {
        let role = self.role_for(project)?;
        let mut row = self.clone();
        row.project = project;
        row.role = role;
        if role == Role::Context {
            row.home = false;
            row.kinds = Kinds {
                read: self.kinds.read,
                push: false,
                open: false,
                land: false,
                review: false,
                status: false,
                comment: false,
                issue: false,
                branch: false,
            };
        }
        Some(row)
    }
}
/// The git identity a forge's repositories are pushed and fetched with: an
/// application credential numbered by the deployment's configuration
/// (protocol/credentials.md, "each repository to a git identity").
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GitIdentity {
    pub forge: u16,
    pub credential: u32,
}
/// Protection as observed by this writer at adoption.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Protection {
    /// The writer could not read protection.
    Unknown,
    /// No applicable rule exists.
    Absent,
    /// An applicable rule was read.
    Rule(skein_forge::api::Protection),
}
/// The caller's request to adopt an existing repository.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Adoption {
    pub project: u32,
    /// Select this repository as the project's home if it has none yet.
    pub home: bool,
    pub forge: u16,
    pub host: Box<[u8]>,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub prefix: Box<[u8]>,
    pub role: Role,
    pub landing: Box<[u8]>,
    pub ci: bool,
    pub checks: Box<[u32]>,
}
/// A FORGE request admitted by the core (client/engine.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Ask {
    DismissMaking {
        forge: u16,
        org: Box<[u8]>,
        name: Box<[u8]>,
        into: Option<u32>,
    },
    MakeRepository {
        forge: u16,
        org: Box<[u8]>,
        name: Box<[u8]>,
        from: crate::MakeFrom,
        into: Option<crate::MakeInto>,
    },
    Adopt(Adoption),
    SetDraft(crate::SetDraft),
    SetUp {
        forge: Making,
        first: SetUpFirst,
    },
    RemoveForge {
        forge: u16,
    },
    StartPlayground,
    EndPlayground,
    Connect {
        forge: Making,
        how: How,
    },
    Install {
        forge: u16,
        back: Option<Box<[u8]>>,
    },
    Accept {
        forge: u16,
        installation: core::num::NonZeroU64,
    },
    Reachable {
        forge: u16,
        installation: Option<u64>,
        page: u32,
    },
    /// A GitHub App's new key or a connected Forgejo's new tokens, carried (client/forge/records.md).
    Replace {
        forge: u16,
    },
}
/// The first repository requested when connecting a Forgejo (client/engine.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FirstAsk {
    pub org: Box<[u8]>,
    pub repository: Box<[u8]>,
    pub label: Box<[u8]>,
}
/// The public connect's input; credentials remain carried handles.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum How {
    ForgejoAdmin {
        first: Option<FirstAsk>,
    },
    GitHubApp {
        app: core::num::NonZeroU64,
        read_only: bool,
    },
    GitHubManifest {
        back: Option<Box<[u8]>>,
    },
    /// A connected Forgejo's bot's tokens, carried (protocol/setup/README.md, section 4).
    ForgejoTokens,
}
/// The App records' provenance since this engine started (client/forge/records.md).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AppWay {
    Manifest,
    Pasted,
}
/// A retained application answer; adoption and making outcomes are durable.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Answered {
    Follow(Option<Box<[u8]>>),
    Make(MakeAnswer),
    Adopt(ApplicationOutcome),
    SetUp(SetUpAnswer),
    RemoveForge(RemoveAnswer),
    StartPlayground(StartAnswer),
    EndPlayground(EndAnswer),
    Connect(ConnectAnswer),
    Install(InstallAnswer),
    Accept(AcceptAnswer),
    Reachable(ReachableAnswer),
    Dismiss(crate::DismissAnswer),
    Looked(crate::LookedAnswer),
    SetAway(crate::SetAwayAnswer),
    Overturn(crate::OverturnAnswer),
    Device(crate::DeviceAnswer),
    FileFinding(crate::FileFindingAnswer),
    HandFinding(crate::HandFindingAnswer),
    TriageFinding(crate::TriageFindingAnswer),
    SetDraft(crate::SetDraftAnswer),
}
/// Making admission (temper/docs/design/domain/forge/adoption.md, 5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MakeAnswer {
    Kept,
    Refused(crate::MakeRefusal),
}
/// Public forge admission refusals (client/engine.md).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ForgeRefusal {
    Role,
    NoSuchForge,
    NotTaken,
    Connected,
    NotConnected,
    Busy,
    NoKey,
    /// A first repository's organization named as the bot's login: Forgejo's users and organizations
    /// share names (protocol/setup/reconcile.md, 5).
    FirstIsBot,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ConnectAnswer {
    Connecting,
    Link(Box<[u8]>),
    Refused(ForgeRefusal),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum InstallAnswer {
    Link(Box<[u8]>),
    Connecting,
    Refused(ForgeRefusal),
}
/// An existing installation, verified and retained by the setup host.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum AcceptAnswer {
    Accepted { installation: u64 },
    Refused(ForgeRefusal),
    ForgeRefused(AdoptionRefusal),
}
/// A picker answer lives beside the public links, never in the store.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ReachableAnswer {
    Page(Page),
    Refused(ForgeRefusal),
    ForgeRefused(AdoptionRefusal),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Page {
    pub rows: Box<[Reached]>,
    pub more: bool,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Reached {
    pub id: u64,
    pub owner_id: u64,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub private: bool,
}
/// The start is answered once both are done, *running*, or at the first that
/// fails, *start failed*, naming why (protocol/setup/playground.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum StartAnswer {
    Running,
    NotOffered,
    Failed(StartFailure),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum StartFailure {
    Group { why: Box<[u8]> },
    Account,
    Adoption,
    NotConnected,
}
/// The playground's scripted provider and start deadline (temper/docs/design/protocol/setup/playground.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PlaygroundConfig {
    pub provider: u32,
    pub deadline: skein_lib::Duration,
}
/// The reset's acts (temper/docs/design/protocol/setup/playground.md).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ResetAct {
    RemoveProject { project: u32 },
    RemoveAccount { account: u32 },
}
/// The setup request's one answer, independent of its later course.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SetUpAnswer {
    SettingUp,
    Refused(SetUpRefusal),
}
/// The configured forge provider (domain/forge/adoption.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    Forgejo,
    GitHub,
}
/// How the deployment connects to its forge.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Mode {
    Connected,
    Managed,
    Owned,
    Fake,
}
/// Public owned-forge startup details (protocol/setup/owned.md, 5).
/// The password field is an absolute file path, never the password itself.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Owned {
    pub forgejo: Box<[u8]>,
    pub runner: Option<Box<[u8]>>,
    pub password: Box<[u8]>,
}
/// Public startup configuration, without credentials.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Configured {
    /// The records-only managed string form accepts an admin paste.
    pub admin_connect: bool,
    /// The setup entry's bot login; empty for a forge with no setup entry.
    pub bot: Box<[u8]>,
    pub owned: Option<Owned>,
    /// Pull-address base, empty when the configured git base cannot fit the address bound.
    pub git: Box<[u8]>,
    pub web: Option<Box<[u8]>>,
    pub read_only: bool,
    pub number: u16,
    pub kind: Kind,
    pub mode: Mode,
    /// The configuration's `start: asked`; false is the ordinary automatic start.
    pub asked: bool,
}
/// Where a forge's API is reached (temper/docs/design/protocol/forge/engine.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Address {
    pub host: Box<[u8]>,
    pub port: u16,
    pub tls: Option<Box<[u8]>>,
}
/// A forge's API, remote base and pages (temper/docs/design/protocol/forge/engine.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Place {
    pub api: Address,
    pub git: Box<[u8]>,
    pub web: Option<Box<[u8]>>,
}
/// The machine's places (temper/docs/design/protocol/configuration/engine.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Machine {
    /// `application.machine.owned.person`'s login; empty with no owned machine.
    pub owned_person: Box<[u8]>,
    pub owned: Option<Place>,
    pub fake: Option<Place>,
    pub github: Place,
    pub github_git_program: Option<Box<[u8]>>,
}
/// The bot's login and commit author (temper/docs/design/protocol/forge/engine.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Bot {
    pub login: Box<[u8]>,
    pub name: Box<[u8]>,
    pub email: Box<[u8]>,
}
/// The first repository's intent (temper/docs/design/protocol/forge/engine.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct EmptyIntent {
    pub playground: bool,
    pub org: Box<[u8]>,
    pub repository: Box<[u8]>,
    pub label: Box<[u8]>,
}
/// The set up's durable choice (temper/docs/design/protocol/setup/owned.md, 7).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FirstIntent {
    Empty(EmptyIntent),
    Copies { addresses: Box<[Box<[u8]>]> },
    Nothing,
}
/// The set up's admitted input (temper/docs/design/client/forge/records.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SetUpFirst {
    Empty { org: Box<[u8]>, repository: Box<[u8]>, playground: bool },
    Copies { addresses: Box<[Box<[u8]>]> },
    Nothing,
}
/// The row's first-repository choice (temper/docs/design/protocol/setup/owned.md, 7).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum StartsWith {
    Repository,
    Copies { addresses: Box<[Box<[u8]>]> },
    Nothing,
}
/// A forge's link to a box's one person: on this forge, that party is that user
/// (temper/docs/design/protocol/forge/engine.md, 1; temper/docs/design/domain/forge/adoption.md, 6).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Link {
    /// The forge's user, never 0.
    pub user: u64,
    /// Jig's party, never 0.
    pub person: u64,
}

/// What setup read of the forge's person (temper/docs/design/domain/forge/adoption.md, 6).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PersonAnswer {
    /// The person's user: a Forgejo `Reconcile`'s `Person` id or a manifest-made App's `User` account.
    Is(u64),
    /// The person's Forgejo account is gone: a `Reconcile`'s `Person` with no id.
    Gone,
    /// The App has no matching `AppMade` or its account is not a `User`.
    NotTheirs,
}

/// What happened to the forge's link (temper/docs/design/domain/forge/adoption.md, 6).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LinkNote {
    Gone,
    Writer,
    Replaced,
    Cleared,
}

/// A forge kept in the connector's store (temper/docs/design/protocol/forge/engine.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ForgeRecord {
    pub number: u16,
    pub kind: Kind,
    pub mode: Mode,
    pub state: ForgeState,
    pub place: Place,
    pub git_program: Option<Box<[u8]>>,
    pub read_only: bool,
    pub bot: Bot,
    /// The setup entry's `start: asked` (protocol/setup/init.md).
    pub asked: bool,
    pub first: Option<FirstIntent>,
    /// The forge's link to a box's person, or none (temper/docs/design/protocol/forge/engine.md, 1).
    pub person: Option<Link>,
}
/// Kept, or ended with removal's answers (temper/docs/design/domain/forge/adoption.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ForgeState {
    Kept,
    Ended { secrets_erased: bool, box_removed: bool },
}
/// A kept forge or a new one numbered next (temper/docs/design/protocol/forge/engine.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Making {
    Kept(u16),
    New(ForgeAsk),
}
/// A new forge's input, completed by the machine's places (temper/docs/design/protocol/configuration/engine.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ForgeAsk {
    GitHub,
    Owned,
    Managed {
        place: Place,
        git_program: Option<Box<[u8]>>,
    },
    /// A Forgejo of the person's connected with its bot's tokens (protocol/setup/README.md, section 4).
    Connected {
        place: Place,
        git_program: Option<Box<[u8]>>,
    },
}
/// The box group's kind (temper/docs/design/protocol/setup/box.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BoxKind {
    Owned,
    Fake,
}
/// The playground's row; no row is none (temper/docs/design/protocol/setup/playground.md).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Playground {
    pub state: PlaygroundState,
    pub forge: u16,
    pub project: Option<u32>,
    pub account: Option<u32>,
}
/// The playground's progress (temper/docs/design/protocol/setup/playground.md).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PlaygroundState {
    Starting,
    Running,
    Resetting(ResetStep),
}
/// The reset's ordered acts (temper/docs/design/protocol/setup/playground.md).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ResetStep {
    Project,
    Forge,
    Account,
}
/// Removal's answer (temper/docs/design/domain/forge/adoption.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RemoveAnswer {
    Removed,
    Refused(ForgeRefusal),
}
/// Reset's answer (temper/docs/design/protocol/setup/playground.md).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EndAnswer {
    Reset,
    NotOffered,
}

/// The row's setup offer, also used to admit `SetUp` (domain/forge/adoption.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Offer {
    NotOffered,
    NotUnderTheBox,
    Offered,
}
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum CreateStep {
    Admin,
    Org,
    Repository,
    File,
}
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum FirstState {
    Making,
    Made { id: u64 },
    Failed { step: CreateStep },
}
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum SetupStage {
    Admin,
    Bot,
    Token,
    Org,
    Repository,
    File,
    App,
    Installation,
    State,
}
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum AdminStop {
    NotAdmin,
    Refused { status: u16, stage: SetupStage },
    Unmarked,
    Other,
}
#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub struct First {
    pub playground: bool,
    pub org: Box<[u8]>,
    pub repository: Box<[u8]>,
    pub state: FirstState,
    pub label: Box<[u8]>,
}

/// A repository already adopted on this forge (client/forge/records.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AdoptedRow {
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub project: u32,
}

/// One configured forge, including its connection, offer and first repository.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Row {
    pub read_only: bool,
    pub number: u16,
    /// The host in repository resource names, including an explicit port.
    pub host: Box<[u8]>,
    pub kind: Kind,
    pub mode: Mode,
    pub connection: Connection,
    pub failed: Option<FailedStep>,
    pub fetched: Option<SetUpFetched>,
    pub offer: Offer,
    pub first: Option<First>,
    pub starts_with: StartsWith,
    pub installations: Box<[(u64, Installation)]>,
    pub ended: Option<Box<[u8]>>,
    pub app: Option<AppWay>,
    pub web: Option<Box<[u8]>>,
    pub repositories: Box<[AdoptedRow]>,
    pub makings: Box<[crate::RepositoryMaking]>,
    pub owned: Option<Owned>,
    pub admin: bool,
    pub ended_stop: Option<AdminStop>,
}
/// The deployment's forges, ordered by number (domain/forge/adoption.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Forges {
    pub rows: Box<[Row]>,
}
/// The admitted repository and its observed collaborators.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Adopted {
    pub repository: Repository,
    pub collaborators: Box<[skein_forge::api::Collaborator]>,
}
/// The connector's outcome, kept under the people child's application token.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ApplicationOutcome {
    Follow(Option<Box<[u8]>>),
    Adopted(AdoptedRepository),
    CoreRefused(jig_domain_people::Refusal),
    ForgeRefused(AdoptionRefusal),
    MakeKept,
    MakeRefused(crate::MakeRefusal),
}
/// The repository facts shown by the adoption's terminal (client/forge/records.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AdoptedRepository {
    pub repository: skein_forge::api::Repository,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub role: Role,
    pub home: bool,
    pub kinds: Kinds,
    pub protection: Protection,
}
/// A forge refusal stated in the adoption form's vocabulary (client/engine.md).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AdoptionRefusal {
    NoSuchForge,
    NotFound,
    NoAccess,
    Refused,
    InFlight,
    TooLarge,
    Unreachable,
    NotInstalled,
    InvalidAnswer,
    AnotherProject,
    HomeHeld,
    PrefixOccupied,
}
/// Project a forge call's refusal into the adoption's terminal reason.
#[must_use]
pub const fn reason(error: skein_forge::api::Error) -> AdoptionRefusal {
    match error {
        skein_forge::api::Error::Missing => AdoptionRefusal::NotFound,
        skein_forge::api::Error::Forbidden => AdoptionRefusal::NoAccess,
        skein_forge::api::Error::Refused => AdoptionRefusal::Refused,
        skein_forge::api::Error::Busy => AdoptionRefusal::InFlight,
        skein_forge::api::Error::TooLarge(_) | skein_forge::api::Error::Full => AdoptionRefusal::TooLarge,
        skein_forge::api::Error::NotInstalled => AdoptionRefusal::NotInstalled,
        skein_forge::api::Error::NotConnected
        | skein_forge::api::Error::Unavailable
        | skein_forge::api::Error::Timeout
        | skein_forge::api::Error::RateLimited { .. } => AdoptionRefusal::Unreachable,
        skein_forge::api::Error::MissingJob
        | skein_forge::api::Error::Empty
        | skein_forge::api::Error::Exists
        | skein_forge::api::Error::NothingToMerge
        | skein_forge::api::Error::Closed
        | skein_forge::api::Error::Stale
        | skein_forge::api::Error::Conflict
        | skein_forge::api::Error::Protected
        | skein_forge::api::Error::InvalidAnswer => AdoptionRefusal::InvalidAnswer,
    }
}
/// Exclusive write hold and its current writer.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Hold {
    pub name: Name,
    pub task: u64,
    pub writer: Option<Writer>,
    /// Runs writing design beside the slot (domain/forge/branches.md, 3).
    pub design_writers: Box<[DesignWriter]>,
    /// Last outside change requiring a person to release this hold.
    pub drift: Option<Drift>,
}

/// An outside change and its observation time, retained with the affected hold.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Drift {
    pub at: skein_lib::Wall,
    pub change: DriftChange,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum DriftChange {
    Branch { expected: skein_forge::api::Commit, observed: Option<skein_forge::api::Commit> },
    PullClosed { number: u64 },
    Retargeted { number: u64, before: Box<[u8]>, after: Box<[u8]> },
}
/// Durable progress while a closing task releases its connector resources.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ReleaseRow {
    pub task: u64,
    pub root: u64,
    pub ending: ReleaseEnding,
    pub close_pull: Option<u64>,
    /// The cancel's reason, still to be posted on its pull before it closes.
    pub comment: Option<Box<[u8]>>,
    pub pending: Option<(u64, Option<Name>)>,
    pub failed: bool,
    /// All effects issued before cleanup have settled.
    pub effects_settled: bool,
    /// The task child has authorized cleanup after effect settlement.
    pub releasing: bool,
}
/// Terminal task class that determines forge cleanup.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ReleaseEnding {
    /// The task finished successfully.
    Done,
    /// The task failed and its branch remains until the root closes.
    Failed,
    /// The task was cancelled, so its pull closes before its branch is deleted.
    Cancelled,
}
/// The last head pushed in one repository by a hosted attempt.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Pushed {
    pub tag: u64,
    pub commit: skein_forge::api::Commit,
    pub checked: bool,
}
/// A goal's design-writing attempt, retained with its hold (domain/forge/branches.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DesignWriter {
    pub task: u64,
    pub attempt: u64,
    /// The answer or loss has requested the branch's fresh read.
    pub rereading: bool,
}

/// A run or connector effect occupying one writer slot.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Writer {
    /// A worker attempt, fenced by the root's activation.
    Run { task: u64, attempt: u64 },
    /// An ended attempt whose fresh branch read must release its writer.
    Lost { task: u64, attempt: u64 },
    /// An outbox entry making a change to the resource.
    Entry(u64),
}
/// A subject published by the forge (domain/forge/README.md, section 5).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Topic {
    /// A branch's tip moved.
    Landings {
        repository: skein_forge::api::Repository,
        branch: Box<[u8]>,
    },
    /// CI changed on one commit.
    Ci {
        repository: skein_forge::api::Repository,
        head: skein_forge::api::Commit,
    },
    /// A pull's state changed.
    Pull {
        repository: skein_forge::api::Repository,
        number: u64,
    },
    /// A participant wrote on an object.
    Participation {
        repository: skein_forge::api::Repository,
        number: u64,
    },
    Findings {
        project: u32,
    },
}
/// A subscriber's classification of observed news.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Class {
    /// Start or resume this task when its inbox permits it.
    Wakes,
    /// Retain for the next run.
    Kept,
    /// The task has no need to hear this event.
    Dropped,
}
/// News delivered through a topic to task inboxes.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum News {
    /// A branch moved; overlap is decided from bounded file facts.
    Landing {
        before: skein_forge::api::Commit,
        after: skein_forge::api::Commit,
        files: Option<Box<[Box<[u8]>]>>,
    },
    /// The combined CI state changed at its head.
    Ci {
        head: skein_forge::api::Commit,
        status: skein_forge::api::Ci,
    },
    /// A pull or participating object's state changed.
    Changed {
        number: u64,
    },
    Red(crate::RedNews),
}
/// A live subscription and its known paths for landing overlap.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Subscriber {
    pub task: u64,
    /// Root-issued task subscription, retained across restarts for news delivery.
    pub number: u64,
    pub topic: Topic,
    pub own_change: Option<u64>,
    /// Whole goal subtree for automatic topics; absent for an agent subscription.
    pub goal_tasks: Option<Box<[u64]>>,
    pub paths: Box<[Box<[u8]>]>,
}
/// Last observed branch tip, used to publish each landing once.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct BranchHead {
    pub name: Name,
    pub commit: skein_forge::api::Commit,
    /// The store has confirmed this branch was left by our writer.
    pub owned: bool,
}
/// Last published pull state and CI verdict.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PullState {
    pub name: Name,
    pub head: skein_forge::api::Commit,
    pub ci: skein_forge::api::Ci,
    pub state: skein_forge::api::State,
}
/// Last CI verdict published to subscribers of one commit.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CiState {
    pub repository: skein_forge::api::Repository,
    pub head: skein_forge::api::Commit,
    pub status: skein_forge::api::Ci,
}
/// A goal's issue projection, including an in-flight write and created issue.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct IssueRow {
    pub goal: u64,
    pub repository: skein_forge::api::Repository,
    pub state: issues::Projected,
    pub number: Option<u64>,
    pub pending: Option<u64>,
    pub before: Option<issues::Projected>,
    pub failed: bool,
    /// Latest root-authorized goal snapshot, including an ending while a write is pending.
    pub desired: Option<issues::GoalView>,
}
/// Maximum retained operation, target and outcome text.
pub const EFFECT_ACCOUNT_BYTES: u32 = 512;
/// A settled effect's explanation, retained after its outbox entry is retired.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct EffectAccount {
    pub entry: u64,
    pub words: Box<[u8]>,
}
/// A change's terminal account (temper/docs/design/domain/forge/branches.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Ending {
    Landed,
    NoChange,
    Cancelled,
    Failed { line: Box<[u8]> },
}
/// The source landings a change follows (temper/docs/design/domain/forge/follows.md, 4).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ChangeFollows {
    pub follow: u32,
    pub commits: Box<[skein_forge::api::Commit]>,
}
/// The change's durable exact-head workflow progress (domain/forge/follows.md, 6).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FollowCheck {
    pub follow: u32,
    pub source: skein_forge::api::Commit,
    pub head: skein_forge::api::Commit,
    pub definition: Box<[u8]>,
    pub key: Option<[u8; 32]>,
    pub flight: Option<TierFlight>,
    pub page: u32,
    pub faults: u32,
    pub held: Option<Box<[u8]>>,
}
/// A change task's parameters and progress (temper/docs/design/domain/forge/change/README.md, 1, and temper/docs/design/domain/forge/branches.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ChangeRow {
    pub job: Option<crate::history::JobRecord>,
    pub signed: Box<[SignOff]>,
    /// The tracked goal served at admission.
    pub goal: Option<u64>,
    pub ending: Option<Ending>,
    /// Whole-life delegate account; a release resets retry counts alone.
    pub repair_tasks: Box<[u64]>,
    pub resolution_tasks: Box<[u64]>,
    /// Bounded, durable observations for the landed result.
    pub result: crate::ChangeResult,
    pub task: u64,
    pub repository: skein_forge::api::Repository,
    pub branch: Box<[u8]>,
    pub base: Box<[u8]>,
    /// Explicit pull title; the production brief stays in the task spec.
    pub title: Box<[u8]>,
    /// Whole pull body, fitted only when the open effect is made.
    pub body: Box<[u8]>,
    pub priority: i32,
    pub change: change::Change,
    /// Last exact-head CI observation retained when the change lands or ends (domain/forge/change/README.md, 5).
    pub ci: Option<change::Ci>,
    pub pull: Option<u64>,
    pub pending: Option<u64>,
    pub effect: change::EffectResult,
    /// The current child and its latest standing; a held close job stays named while live.
    pub delegate: Option<(u64, change::Delegate)>,
    pub delegate_status: change::Status,
    /// Bounded current verdict per gate, retained through clean updates.
    pub verdicts: Box<[change::GateReport]>,
    /// Bounded explanations of gate verdicts at their named heads.
    pub gate_remarks: Box<[GateRemark]>,
    pub drift: Option<change::Hold>,
    /// A deployment-owned repair of a landing branch, exempt from that branch's broken CI.
    pub base_repair: bool,
    /// The base repair started while this ordinary change owned the queue.
    pub queue_repair: Option<u64>,
    /// Repairs attempted for this landing queue while this change owned it.
    pub queue_repairs: u32,
    /// Charter numbers captured when the change is made (protocol/configuration/settings.md, 5).
    pub producing_charter: u32,
    pub review_charter: u32,
    /// The LLM choices by role it was made with (domain/forge/change/README.md, 1, and domain/forge/change/gates.md, 3).
    pub producing_llm: Option<jig_domain_tasks::LlmChoice>,
    pub review_llm: Option<jig_domain_tasks::LlmChoice>,
    pub effect_account: Option<EffectAccount>,
    /// Words of the hold its delegate's end made, within the hold account bound (domain/forge/change/repairs.md).
    pub producer_failure: Option<Box<[u8]>>,
    /// Its producing task's last successful result words (domain/forge/change/README.md, 2 step 7)
    pub report: Box<[u8]>,
    /// Its agents' questions and their relay (domain/forge/change/questions.md, 1).
    pub questions: change::Questions,
    /// The red landing this change repairs (temper/docs/design/domain/agent/tools.md, 1).
    pub repairs: Option<skein_forge::api::Commit>,
    pub follows: Option<ChangeFollows>,
    pub follow_check: Option<FollowCheck>,
    /// The commit this catch-up merges (temper/docs/design/domain/forge/change/README.md, 2).
    pub catch_up: Option<skein_forge::api::Commit>,
    /// The close request (temper/docs/design/domain/forge/change/README.md, 1).
    pub close: Option<Box<[u8]>>,
    /// The last merge job's result, at most 512 bytes (temper/docs/design/domain/forge/change/repairs.md).
    pub merge_result: Option<Box<[u8]>>,
    /// Open intervals and the last event (landing-history.md, 3.2).
    pub history: Option<crate::history::ChangeHistory>,
}
impl ChangeRow {
    /// The pull a judgement delegate reads; every other state keeps the change's pull.
    #[must_use]
    pub fn review_target(&self) -> Option<(u64, &[u8], &[u8], skein_forge::api::Commit)> {
        if let change::State::Judging { target, .. } = &self.change.state {
            return Some((target.pull, &target.branch, &target.base, target.prefix));
        }
        Some((self.pull?, &self.branch, &self.base, self.change.last_head?))
    }

    /// A landed or ended change gathers no more brief or change facts.
    #[must_use]
    pub fn terminal(&self) -> bool {
        match self.change.state {
            change::State::Landed { .. }
            | change::State::Red { .. }
            | change::State::Cleared { .. }
            | change::State::Uncleared { .. }
            | change::State::NoChange { .. }
            | change::State::Ended => true,
            change::State::Producing { .. }
            | change::State::Opening { .. }
            | change::State::Recreating { .. }
            | change::State::Reopening { .. }
            | change::State::Checking { .. }
            | change::State::Judging { .. }
            | change::State::Gating { .. }
            | change::State::CancellingGates { .. }
            | change::State::Ready { .. }
            | change::State::Preparing { .. }
            | change::State::Queued { .. }
            | change::State::First { .. }
            | change::State::Updating { .. }
            | change::State::WaitingProducer { .. }
            | change::State::Resolving { .. }
            | change::State::Repairing { .. }
            | change::State::Landing { .. }
            | change::State::Held { .. } => false,
        }
    }
}
/// A project's test gates in configuration order (domain/landing.md, 13.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RepositoryGates {
    pub repository: skein_forge::api::Repository,
    pub gates: Box<[landing::Gate]>,
    pub tiers: Box<[Tier]>,
}
/// A tier description's byte bound (temper/docs/design/protocol/configuration/engine.md).
pub const DESCRIPTION_BYTES: usize = 120;

/// A repository's tier and followed branch (temper/docs/design/domain/after-landing.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Tier {
    pub name: Box<[u8]>,
    pub workflow: Box<[u8]>,
    pub ignored: Box<[Box<[u8]>]>,
    pub artifact: Box<[u8]>,
    pub format: temper_engine_domain_forge_tiers::Format,
    pub follows: Follows,
    pub span: Option<skein_lib::Duration>,
    pub description: Box<[u8]>,
}
/// What a tier follows (temper/docs/design/domain/after-landing.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Follows {
    /// The repository's landing branch, its adoption's default.
    Landing,
    /// One branch by its name on the forge.
    Branch(Box<[u8]>),
    /// The goals' own branches of the repository; judges no landing.
    Goals,
}
/// A project's verification settings (temper/docs/design/domain/after-landing.md, 11).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AfterLanding {
    pub verdict: temper_engine_domain_forge_tiers::Settings,
    pub bound: skein_lib::Duration,
}
/// Why a landing cannot be made (domain/landing.md, 12).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LandingRefusal {
    Train,
    Landings,
    Members,
    Repository,
    Way,
}
/// Family 23, committed together with its task's decision (domain/landing.md, 12).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LandingRow {
    pub job: Option<crate::history::JobRecord>,
    pub landing: landing::Landing,
    pub asks: Box<[Option<change::Ask>]>,
    pub entries: Box<[LandingEntry]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct LandingEntry {
    /// Exact round and base survive a cut while a dispatched run is followed.
    /// Absent on entries patched from a single-prefix landing.
    pub preparation: Option<u16>,
    pub base: Option<landing::Commit>,
    pub entry: u64,
    pub member: u16,
    pub gate: u16,
    pub operation: Operation,
    pub run: Option<LandingRun>,
}
/// Observation of a dispatch's current run (domain/landing-history.md, 3.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct LandingRun {
    pub id: u64,
    pub attempt: u32,
    pub ended: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Operation {
    Dispatch,
    Rerun,
    Pending,
    Verdict,
    Merge,
    Open,
    Close,
    Delete,
    Record,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LandingHeard {
    pub job: Option<crate::history::JobStep>,
    pub outcomes: Box<[crate::preparation::Prepared]>,
    pub dependencies: Box<[ChangeDependencies]>,
    pub held: Box<[u64]>,
    pub failure: Box<[u8]>,
    pub preparation: landing::Preparation,
    /// Live landings waiting prepared on a member of another live landing.
    pub kept: Box<[u64]>,
    pub released: bool,
    pub cancelled: bool,
}
/// Family 24, a result of a run dispatched by temper (domain/landing.md, 6.2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Receipt {
    /// Failed workflow jobs, captured before this receipt is published (domain/landing.md, 7.3).
    pub logs: Box<[u8]>,
    pub failures: Option<temper_engine_domain_forge_tiers::Failures>,
    pub passed: bool,
    pub run: ReceiptRun,
    pub candidate: skein_forge::api::Commit,
    pub ended: skein_lib::Wall,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ReceiptRun {
    pub id: u64,
    pub number: u64,
    pub url: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ReceiptKey {
    pub project: u32,
    pub repository: skein_forge::api::Repository,
    pub check: Check,
    pub key: [u8; 32],
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Check {
    Gate(Box<[u8]>),
    Tier(Box<[u8]>),
}
/// One gate's explanation, retained with its verdict for a repair brief.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GateRemark {
    pub number: u64,
    pub head: skein_forge::api::Commit,
    pub words: Box<[u8]>,
}
/// Coherent provider facts used by the root to check a change effect before
/// releasing its durable outbox entry.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ChangeEvidence {
    pub touched: crate::Touched,
    /// State to restore if the root does not authorize this effect.
    pub prior: change::Change,
    pub head: Option<skein_forge::api::Commit>,
    pub base_tip: skein_forge::api::Commit,
    pub contains_base: change::Status,
    pub ci: change::Status,
    pub gates: Box<[change::GateReport]>,
    pub reviews: Box<[skein_forge::api::Review]>,
    pub reviews_complete: bool,
    pub size: Option<skein_forge::api::Size>,
}
/// Coherent merge facts for a landing member, read in the landing's step.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LandingEvidence {
    pub change: u64,
    pub facts: ChangeEvidence,
}
/// Identity supplied by the named-call owner, interpreted only for lookup.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct NamedCall {
    pub task: u64,
    pub attempt: u64,
    pub completion: u32,
    pub position: u32,
}

/// The connector's part of a named answer; core decisions remain outside it.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum CallPayload {
    Follow { follow: u32, refusal: Option<Box<[u8]>> },
    Finding(Box<crate::FindingReply>),
    Decided { took: Option<Box<[u8]>>, reach: Option<temper_protocol_reaches::Reach> },
    Effect { entry: u64, deadline: skein_lib::Wall, outcome: Option<skein_forge_client::Outcome> },
    Refused(skein_forge::api::Error),
    Read(Box<Result<skein_forge::api::Answer, skein_forge::api::Error>>),
    Red(Box<crate::RedReply>),
}

/// The connector's durable records, wrapped by the root store.
#[expect(clippy::large_enum_variant, reason = "the durable change row keeps role choices and merge-job state inline")]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Stored {
    /// Identity only: a stored setting the codec refused, never its retired value.
    RefusedSetting(crate::settings::SettingKey),
    /// The connector part of a live named call.
    Call {
        key: NamedCall,
        answer: CallPayload,
    },
    /// Connector-owned payload retained under an explicit proposal's number.
    ProposedEffect(ProposedEffect),
    /// One adopted repository.
    Repository(Repository),
    /// One held resource.
    Hold(Hold),
    /// A live task's resources, or a kept task's repositories (temper/docs/design/domain/forge/adoption.md, 1).
    Names {
        task: u64,
        resources: Box<[Name]>,
    },
    /// One topic subscription.
    Subscription(Subscriber),
    /// One branch's last observed tip.
    BranchHead(BranchHead),
    /// One pull's last published state.
    PullState(PullState),
    Ci(CiState),
    /// A merge made by this deployment, for own-change news classification.
    Landed {
        commit: skein_forge::api::Commit,
        task: u64,
    },
    /// An unsettled outbox entry. A held one is out of the client's outbox,
    /// keeping its room until a person makes it again or it is settled not
    /// made (domain/forge/README.md, 4; jig's `domain/connectors.md`, 4.3).
    Entry {
        entry: skein_forge_client::Entry,
        held: bool,
    },
    /// A client working-set row.
    Client(skein_forge_client::Stored),
    /// A goal closer's committed state.
    Closer(crate::CloserRow),
    /// A change procedure's committed state.
    Change(Box<ChangeRow>),
    /// A goal issue's committed projection.
    Issue(IssueRow),
    Release(ReleaseRow),
    /// A keyed adoption outcome retained until its wall-clock end.
    Application {
        token: u64,
        until: skein_lib::Wall,
        outcome: ApplicationOutcome,
    },
    /// A forge's writer, kept before any restored effect (domain/forge/adoption.md, 1).
    Writer(skein_forge_client::Writer),
    /// First Google subject for a member address (protocol/sign-in.md, 5).
    Binding {
        address: Box<[u8]>,
        subject: Box<[u8]>,
    },
    /// The forge's kinds (temper/docs/design/protocol/store.md, 2; temper/docs/design/domain/forge/change/gates.md, 3).
    Setting(crate::settings::SettingRow),
    /// A sent write not yet settled (domain/forge/README.md, 4; protocol/store.md, 2).
    Flight {
        task: u64,
        entry: u64,
    },
    Landing(LandingRow),
    Receipt {
        key: ReceiptKey,
        receipt: Receipt,
    },
    /// A forge's durable setup and removal state (temper/docs/design/protocol/forge/engine.md, 1; temper/docs/design/domain/forge/adoption.md, 1).
    Forge(ForgeRecord),
    /// The last forge number given (temper/docs/design/protocol/forge/engine.md, 1).
    ForgeNumber {
        last: u16,
    },
    /// The playground's durable progress (temper/docs/design/protocol/setup/playground.md).
    Playground(Playground),
    Follow(crate::FollowRow),
    Tier(TierRow),
    Verification(Verification),
    LandingLog(LandingLog),
    Making(crate::RepositoryMaking),
    /// A person's dismissals (temper/docs/design/domain/forge/attention.md, 1).
    Dismissed {
        party: u64,
        what: crate::PersonFact,
    },
    ProjectFacts(crate::ProjectFacts),
    Looked(crate::Looked),
    Away(crate::Away),
    Decided(crate::Decided),
    Device(crate::Device),
    Finding(crate::Finding),
    LandedDraft(crate::LandedDraft),
    Red(crate::Red),
    RedPass(crate::RedPass),
    RedNumber {
        last: u64,
    },
    HistoryProject(crate::history::Project),
    HistoryEvent(crate::history::Event),
    HistoryChange(crate::history::ChangeIndex),
    HistoryLanding(crate::history::LandingIndex),
    HistoryLast(crate::history::Last),
}
/// A bounded connector payload awaiting an authority holder's decision.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProposedEffect {
    pub number: u64,
    pub resource: What,
    pub entry: skein_forge_client::Entry,
}

/// Stable store key of a connector record.
/// The settings family is temper/docs/design/protocol/store.md, 2; temper/docs/design/domain/forge/change/gates.md, 3.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Key {
    Call(NamedCall),
    ProposedEffect(u64),
    Repository(skein_forge::api::Repository),
    Hold(Name),
    Names(u64),
    Subscription { task: u64, topic: Topic },
    BranchHead(Name),
    PullState(Name),
    Ci { repository: skein_forge::api::Repository, head: skein_forge::api::Commit },
    Landed(skein_forge::api::Commit),
    Entry(u64),
    Client(skein_forge_client::Key),
    Change(u64),
    Issue(u64),
    Release(u64),
    Application(u64),
    Writer(u16),
    Binding(Box<[u8]>),
    Setting(crate::settings::SettingKey),
    Flight { task: u64, entry: u64 },
    Landing(u64),
    Receipt(ReceiptKey),
    Forge(u16),
    ForgeNumber,
    Playground,
    Tier { repository: skein_forge::api::Repository, tier: Box<[u8]> },
    Verification { repository: skein_forge::api::Repository, commit: skein_forge::api::Commit },
    LandingLog { repository: skein_forge::api::Repository },
    Finding { project: u32, number: u32 },
    Dismissed { party: u64, what: crate::PersonFact },
    Making(crate::MakingKey),
    ProjectFacts { project: u32 },
    Looked { party: u64, project: u32 },
    Away { party: u64, project: u32 },
    Decided { project: u32, number: u32 },
    Closer(u64),
    Device { party: u64, number: u32 },
    LandedDraft { repository: skein_forge::api::Repository, path: Box<[u8]> },
    Follow { project: u32, follow: u32 },
    Red { goal: u64, key: Box<[u8]> },
    RedNumber,
    RedPass { change: u64, key: Box<[u8]> },
    HistoryProject { project: u32 },
    HistoryEvent { project: u32, number: u64 },
    HistoryChange { project: u32, change: u64, number: u64 },
    HistoryLanding { project: u32, landing: u64, number: u64 },
    HistoryLast { project: u32, number: u64, change: u64 },
}
/// A completed phase of the connector's restart contract.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RestartStage {
    /// Durable connector records and entries are restored.
    Restored,
    /// Every live resource has had its first fresh read.
    ReadAfresh,
    /// Every uncertain restored entry has been looked for.
    Settled,
}
/// Volatile connection information, rebuilt from routing at every start (domain/forge/adoption.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Connection {
    NotConnected,
    /// An owned forge the box starts when asked, its files absent (protocol/forge/README.md, 7.1).
    NotSetUp,
    /// The box runs the forge's group, at the step it last named (protocol/forge/engine.md, 8).
    SettingUp {
        step: Option<SetUpStep>,
    },
    /// The box's `failed`, or its pipe closed during the ask, with why (protocol/forge/engine.md, 8).
    SetUpFailed {
        why: Box<[u8]>,
    },
    Connecting,
    Connected {
        id: u64,
        login: Box<[u8]>,
    },
    Stopped {
        reason: Box<[u8]>,
    },
}
/// The box's word for the step it runs (protocol/setup/box.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SetUpStep {
    Fetch,
    Prepare,
    Run,
    Source,
}
/// The binary named by the box's fetch line (protocol/forge/engine.md, 8).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SetUpBinary {
    Forgejo,
    Runner,
}
/// The latest fetch line, kept in memory alone (protocol/forge/engine.md, 8).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SetUpFetched {
    pub binary: SetUpBinary,
    pub done: u64,
    pub total: Option<u64>,
}
/// The held connection's step when setup failed (domain/forge/adoption.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FailedStep {
    Fetch,
    Prepare,
    Run,
    Connect,
}
/// Why a `SetUp` is refused (protocol/forge/engine.md, 8; domain/forge/adoption.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SetUpRefusal {
    NotOffered,
    NotUnderTheBox,
    Busy,
    Invalid,
}
/// Installation information for the deployment's page (protocol/forge/engine.md, 4).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Installation {
    pub account: Option<(u64, Box<[u8]>)>,
    pub stray: bool,
    pub suspended: bool,
}
/// Pending task dependencies supplied by the root, never retained by the connector
/// (jig's domain/tasks/lifecycle.md, 8; domain/landing.md, 4.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ChangeDependencies {
    pub change: u64,
    pub pending: Box<[u64]>,
}

/// A parent or child input to the connector top.
#[derive(PartialEq, Eq, Hash, Debug)]
#[expect(clippy::large_enum_variant, reason = "change admission carries a full row across the root procedure boundary")]
pub enum Event {
    HistoryLoaded {
        load: u64,
        rows: Box<[Stored]>,
        next: Option<Key>,
    },
    HistoryUnloaded {
        load: u64,
    },
    ReadHistory {
        read: u64,
        query: crate::history::Query,
    },
    /// Keep the merge judgement in the decision admitting its effect.
    Signed {
        task: u64,
        signed: Box<[SignOff]>,
    },
    /// The service's terminal for a push (attention.md, 7.2).
    Pushed {
        party: u64,
        device: u32,
        endpoint: Box<[u8]>,
        answer: crate::PushAnswer,
    },
    SetDraft {
        request: Token,
        token: Token,
        party: u64,
        call: crate::SetDraft,
    },
    SetAway {
        request: Token,
        token: Token,
        party: u64,
        project: u32,
        on: bool,
        until: Option<skein_lib::Wall>,
    },
    Overturn {
        request: Token,
        token: Token,
        party: u64,
        project: u32,
        number: u32,
    },
    Decided {
        to: skein_lib::ReplyTo,
        key: NamedCall,
        party: u64,
        project: u32,
        task: u64,
        always: bool,
        call: crate::DecidedCall,
    },
    MakingStarted {
        key: crate::MakingKey,
    },
    MakingCopying {
        key: crate::MakingKey,
    },
    MakingMade {
        key: crate::MakingKey,
        id: u64,
        branch: Box<[u8]>,
    },
    MakingStopped {
        key: crate::MakingKey,
        failed: crate::MakingFailed,
    },
    MakingAdopt {
        key: crate::MakingKey,
    },
    MakingAsked {
        request: Token,
        adoption: Adoption,
        repository: skein_forge::api::Repository,
    },
    MakingActed {
        key: crate::MakingKey,
        outcome: Option<crate::MakingFailed>,
    },
    MakingAgain {
        key: crate::MakingKey,
    },
    /// A due step saved these specification words on its own task.
    TierWords {
        task: u64,
        words: Box<[u8]>,
    },
    StatusTask {
        project: u32,
        task: u64,
    },
    StatusRefused {
        project: u32,
    },
    StatusHeard {
        task: u64,
        report: Option<Box<[u8]>>,
    },
    FollowChangeAmended {
        task: u64,
        follows: ChangeFollows,
        words: Box<[u8]>,
    },
    FollowAct {
        request: Token,
        token: Token,
        retention: skein_lib::Duration,
        project: u32,
        follow: u32,
        act: crate::follows::FollowAct,
        authorized: bool,
    },
    Follow {
        row: crate::FollowRow,
    },
    TierTask {
        repository: skein_forge::api::Repository,
        tier: Box<[u8]>,
        number: u64,
        task: u64,
    },
    TierRefused {
        repository: skein_forge::api::Repository,
        tier: Box<[u8]>,
        number: u64,
    },
    StepTier {
        task: u64,
        entry: u64,
        heard: TierHeard,
    },
    VetoTier {
        task: u64,
        entry: u64,
    },
    /// Release adoptions after the project's tasks are erased
    /// (jig/docs/design/domain/people/acts.md, 2; temper/docs/design/domain/forge/adoption.md, 1).
    ReleaseProject {
        project: u32,
        request: Token,
    },
    /// Removal completions (temper/docs/design/domain/forge/adoption.md, 1).
    SecretsErased {
        forge: u16,
    },
    BoxRemoved {
        forge: u16,
    },
    OrderLanding {
        task: u64,
        changes: Box<[u64]>,
    },
    JoinLanding {
        task: u64,
        member: landing::Member,
        after: Box<[u64]>,
    },
    Landing {
        row: LandingRow,
    },
    VetoLanding {
        task: u64,
        entry: u64,
    },
    StepLanding {
        task: u64,
        entry: u64,
        heard: LandingHeard,
    },
    /// The engine stops: finish sent entries without starting another write (domain/forge/README.md, 4).
    Settle,
    /// A successful conversion kept its App records.
    AppRecorded {
        forge: u16,
        app: AppWay,
    },
    /// Bind the folded member address before minting a sign-in.
    Bind {
        request: Token,
        address: Box<[u8]>,
        subject: Box<[u8]>,
    },
    PlaygroundMade {
        project: u32,
        account: u32,
    },
    PlaygroundFailed {
        why: StartFailure,
    },
    ResetActAnswered {
        key: [u8; 16],
        keep: bool,
    },
    /// The box ended its pipe (domain/forge/adoption.md, 1).
    BoxClosed,
    /// The routing's answer to a core-admitted connect request.
    ConnectAnswered {
        forge: u16,
        request: Token,
        answer: ConnectAnswer,
    },
    AccessAnswered {
        forge: u16,
        request: Token,
        answer: Result<(), AdoptionRefusal>,
    },
    ReachableAnswered {
        forge: u16,
        request: Token,
        answer: ReachableAnswer,
    },
    AcceptAnswered {
        forge: u16,
        request: Token,
        answer: AcceptAnswer,
        stopped: Option<Box<[u8]>>,
    },
    InstallAnswered {
        forge: u16,
        request: Token,
        answer: InstallAnswer,
        stopped: Option<Box<[u8]>>,
    },
    ConnectEnded {
        stop: Option<AdminStop>,
        forge: u16,
        request: Token,
        connected: bool,
        why: Option<Box<[u8]>>,
    },
    /// The routing's answer to a core-admitted setup request.
    SetUpAnswered {
        forge: u16,
        request: Token,
        answer: SetUpAnswer,
    },
    FirstMade {
        forge: u16,
        id: u64,
    },
    FirstFailed {
        forge: u16,
        step: CreateStep,
    },
    Connection {
        forge: u16,
        state: Connection,
    },
    Fetched {
        forge: u16,
        fetched: SetUpFetched,
    },
    Installation {
        forge: u16,
        installation: u64,
        state: Option<Installation>,
    },
    /// Decide one core-admitted keyed adoption and keep its outcome.
    Application {
        request: Token,
        token: Token,
        retention: skein_lib::Duration,
        ask: Ask,
        role: Option<jig_domain_people::Role>,
        carried: Option<jig_domain_core_vocabulary::Carried>,
    },
    Looked {
        request: Token,
        token: Token,
        party: u64,
        project: u32,
    },
    /// A core-admitted dismissal (temper/docs/design/domain/forge/attention.md, 1).
    Dismiss {
        request: Token,
        token: Token,
        party: u64,
        what: crate::PersonFact,
    },
    /// The core's answer to the application's adoption part.
    ApplicationAnswered {
        request: Token,
        outcome: jig_domain_people::Outcome,
    },
    /// Drop a projection only after the core has settled its closing feed.
    ForgetProjection {
        goal: u64,
    },
    /// Plan a charter document as a connector-owned section.
    PlanDocument {
        section: Token,
        document: Box<BriefDocument>,
    },
    /// Keep a connector-owned section source while the core budgets its brief.
    PlanBrief {
        section: Token,
        source: BriefSource,
        partner: Option<Token>,
    },
    /// Gather a planned section with the core's allotted budget.
    GatherPlanned {
        section: Token,
        task: u64,
        parts: u32,
        bytes: u32,
        ci_budget: u32,
        deadline: skein_lib::Time,
    },
    /// Gather one pinned, connector-owned section for a run brief.
    GatherBrief {
        owner: Token,
        source: BriefSource,
        parts: u32,
        bytes: u32,
        max_job_bytes: u32,
    },
    /// Gather a section and retain its rendered bytes until the root takes or drops its token.
    GatherBriefHeld {
        section: Token,
        source: BriefSource,
        parts: u32,
        bytes: u32,
        max_job_bytes: u32,
    },
    /// Cut retained words to the brief's allotment; CI keeps whole check heads and log tails.
    CutBrief {
        section: Token,
        bytes: u32,
    },
    /// Transfer retained words to the root for a completed assignment.
    TakeBrief {
        section: Token,
    },
    /// Release a section, including one still gathering.
    DropBrief {
        section: Token,
    },
    /// Read current permissions, settings, protection and collaborators.
    Adopt {
        request: Token,
    },
    /// Withdraw a just-adopted repository when its parent cannot commit the role seed.
    ForgetAdoption {
        repository: skein_forge::api::Repository,
        restore: Option<Repository>,
    },
    /// Replace the resources this live task names.
    Names {
        task: u64,
        resources: Box<[Name]>,
    },
    /// Replace a live task's resources; a released task's kept row stays as it is.
    Rename {
        task: u64,
        resources: Box<[Name]>,
    },
    /// The task is erased: drop what the connector keeps of its names.
    Unnamed {
        task: u64,
    },
    /// Take a hold or hand one down from its current holder.
    Hold {
        entry: u64,
        task: u64,
        resource: Name,
        from: Option<u64>,
    },
    /// Claim writer slots for one task's attempt.
    Claim {
        task: u64,
        attempt: u64,
        writes: Box<[Name]>,
        designs: Box<[Name]>,
        holders: Box<[u64]>,
    },
    /// Release slots and confirm each branch, head and checked-delivery flag.
    Answered {
        task: u64,
        attempt: u64,
        pushed: Box<[(Name, skein_forge::api::Commit, bool)]>,
    },
    /// Read a lost writer's resource afresh before making its slot available.
    Lost {
        task: u64,
        attempt: u64,
    },
    /// Retain an explicitly proposed agent effect without making it.
    KeepProposedEffect {
        row: ProposedEffect,
    },
    /// Remove the payload of a terminal proposal.
    DropProposedEffect {
        number: u64,
    },
    /// Enqueue a checked effect in the same decision as its cause.
    Enqueue {
        entry: skein_forge_client::Entry,
    },
    /// Keep a projection candidate after core admission.
    KeepProjection {
        row: IssueRow,
        entry: skein_forge_client::Entry,
    },
    /// Describe the next write for a goal projection.
    Project {
        entry: u64,
        repository: skein_forge::api::Repository,
        view: issues::GoalView,
    },
    /// The root could not translate or authorize a goal projection.
    FailProjection {
        goal: u64,
        repository: skein_forge::api::Repository,
        closing: bool,
    },
    /// Continue the latest authorized snapshot after an outbox outcome or interval.
    ProjectDesired {
        entry: u64,
        goal: u64,
    },
    /// Give a full table an unpinned row's room, settled first (domain/forge/change/README.md, 7).
    DropChange {
        task: u64,
    },
    /// Register one activated change task with its held branch.
    Change {
        row: ChangeRow,
    },
    /// A change's goal's priority moved (domain/forge/queues.md).
    Priority {
        task: u64,
        priority: i32,
    },
    /// Commit the deployment repair identity beside the task creation decision.
    QueueRepairStarted {
        owner: u64,
        repair: u64,
    },
    /// A job attempt was claimed or its result consumed in the root decision.
    JobHistory {
        task: u64,
        job: crate::history::JobStep,
    },
    /// Remember a committed procedure child and its role in this change.
    Delegated {
        task: u64,
        child: u64,
        kind: change::Delegate,
    },
    /// The whole procedure batch was refused before a child number was admitted.
    DelegateAdmissionRefused {
        task: u64,
        kind: change::Delegate,
    },
    /// A proposed child was refused by tasks before creation; keep a releasable state.
    DelegateRefused {
        task: u64,
        child: u64,
    },
    /// Keep the successful producer's title and report before its result wakes the change.
    ProducedTitle {
        task: u64,
        child: u64,
        title: Box<[u8]>,
        report: Box<[u8]>,
    },
    /// Resolve the named child's terminal result or close job's failure hold before stepping again.
    DelegateResult {
        task: u64,
        child: u64,
        status: change::Status,
        /// The child explicitly concluded that no change applies.
        no_change: bool,
        /// The child failed to produce a result, rather than returning a negative verdict.
        failed: bool,
        /// The delegate was cancelled; it supplied no verdict and used no review retry.
        cancelled: bool,
        words: Box<[u8]>,
    },
    /// The last pending dependency ended done; refresh producing and end its unrequested stall hold.
    DependenciesMet {
        task: u64,
    },
    /// Save the relay admitted with a change's procedure step.
    Relayed {
        task: u64,
        questions: change::Questions,
    },
    /// A relay question refused by jig holds with its words and stays unasked.
    RelayRefused {
        task: u64,
        questions: change::Questions,
        words: Box<[u8]>,
    },
    /// The live producer's preparation could not merge a dependency head (domain/forge/change/questions.md, 1).
    StartConflicted {
        child: u64,
        tag: u64,
        head: skein_forge::api::Commit,
    },
    /// A counted person's request refused the change's merge (domain/forge/change/gates.md, 1).
    ReviewRequested {
        task: u64,
        head: skein_forge::api::Commit,
    },
    /// Re-derive the queue after the root keeps changed review facts (domain/forge/queues.md).
    ReviewsChanged {
        task: u64,
    },
    /// Gather fresh forge facts and step one registered change.
    StepChange {
        /// The change's current authority delegates to the merge kind (domain/forge/change/repairs.md).
        merge_job: bool,
        task: u64,
        entry: u64,
        heard: change::Heard,
        gates: Box<[change::GateReport]>,
        queue_repair_active: bool,
        kept: bool,
        pending: Box<[change::Pending]>,
    },
    /// A goal closer's progress joins its procedure step.
    Closer {
        row: crate::CloserRow,
    },
    /// Commit the admitted relay without replacing concurrent step progress.
    CloserRelayed {
        task: u64,
        relay: temper_engine_domain_forge_closer::relay::Relay,
    },
    /// Read the next pending CI selection for the closer report.
    ReadCloserSelection {
        task: u64,
        repository: skein_forge::api::Repository,
    },
    /// Read fresh containment for the named closer step.
    CheckCloser {
        task: u64,
        step: u64,
        repository: skein_forge::api::Repository,
        branch: Box<[u8]>,
        base: Box<[u8]>,
    },
    /// A change's task ending before its resources finish releasing.
    ChangeEnding {
        task: u64,
        ending: Ending,
    },
    /// A requester or person released a held change.
    ReleaseChange {
        task: u64,
    },
    /// A held goal is active again after its drift was released (domain/forge/branches.md, 3).
    GoalReleased {
        task: u64,
    },
    /// Wait for a closing task's prior effects before cleanup begins.
    SettleEffects {
        task: u64,
        root: u64,
        ending: ReleaseEnding,
    },
    /// A closing task's saved work: each branch becomes its hold before the
    /// release that follows (domain/forge/branches.md).
    SavedWork {
        task: u64,
        branches: Box<[Name]>,
    },
    /// Release a closing task's holds after prior effects settled.
    Release {
        task: u64,
        root: u64,
        ending: ReleaseEnding,
        reason: Option<Box<[u8]>>,
        entry: u64,
    },
    /// Continue a release after one of its outbox effects settles.
    ContinueRelease {
        task: u64,
        entry: u64,
    },
    /// Supply a fresh entry for a missing goal branch (domain/forge/branches.md, 1).
    ContinueGoalBranch {
        task: u64,
        entry: u64,
    },
    /// Read and make a goal's branch after its making hold is released.
    RetryGoalBranch {
        task: u64,
    },
    /// Lift a held projection after a person resolves its permanent failure.
    ReleaseProjection {
        goal: u64,
    },
    /// The root has durably committed this entry.
    Committed {
        entry: u64,
    },
    /// A person settled a held entry: make it again, or record it not made
    /// (domain/forge/README.md, 4; jig's `domain/connectors.md`, 4.3).
    Resolve {
        entry: u64,
        again: bool,
    },
    /// Withdraw an entry whose write has not gone out.
    Withdraw {
        entry: u64,
    },
    /// Root authority vetoed a new, unsent change effect in this decision.
    VetoChange {
        task: u64,
        entry: u64,
        prior: change::Change,
        /// Holds, refusals and pending dependencies end the turn; an approval wait does not.
        leaves_queue: bool,
    },
    /// Subscribe a live task to a topic.
    Subscribe {
        subscription: Subscriber,
    },
    /// Unsubscribe a task from a topic.
    Unsubscribe {
        task: u64,
        topic: Topic,
    },
    /// One provider hint passed from the protocol.
    Hint {
        hint: skein_forge::api::Hint,
    },
    /// A client event returned after an earlier connector request.
    Client(skein_forge_client::Event),
    /// Restore one durable connector record.
    Restore {
        record: Stored,
    },
    /// Finish restoration and reconcile loaded entries.
    Restored {
        clock: skein_forge_client::RecoveryClock,
    },
    /// Read the system's live resources before procedures step.
    ReadAfresh,
    /// Look for each restored uncertain entry before procedures step.
    SettleOutbox,
    /// A connected Forgejo's Check found the user its tokens answer as: the connector keeps its
    /// login as the record's bot (protocol/forge/engine.md, 2).
    BotLogin {
        forge: u16,
        login: Box<[u8]>,
    },
    /// The deployment's sole owner, handed by the root at each start (temper/docs/design/domain/root.md, 8.1).
    Owner {
        party: Option<u64>,
    },
    /// What a setup run read of the forge's person (temper/docs/design/domain/forge/adoption.md, 6).
    PersonOf {
        forge: u16,
        answer: PersonAnswer,
    },
    AddDevice {
        request: Token,
        token: Token,
        party: u64,
        person: bool,
        call: crate::AddDevice,
    },
    RemoveDevice {
        request: Token,
        token: Token,
        party: u64,
        number: u32,
    },
}
/// The owner of the root's fresh read `serial`: bit 61 alone of the top five
/// bits, the serial below 2^59; `None` past that (temper/docs/design/domain/forge/keeping-up.md, 3).
#[must_use]
pub fn root_read(serial: u64) -> Option<Token> {
    if serial < 1_u64.checked_shl(59).expect("read serial bound fits u64") {
        Some(Token::new(serial | 1_u64.checked_shl(61).expect("root read bit fits u64")))
    } else {
        None
    }
}

/// Whether a client owner is in the root's read range: bits 63 to 59 are
/// 0, 0, 1, 0, 0 (temper/docs/design/domain/forge/keeping-up.md, 3).
#[must_use]
pub fn is_root_read(owner: Token) -> bool {
    owner.raw().checked_shr(59).expect("read range shift fits u64") == 0b0_0100
}

/// The connector's output to the root, including child API calls.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Request {
    LoadHistory {
        load: u64,
        range: crate::history::HistoryRange,
        after: Option<Key>,
        most: u32,
    },
    HistoryPage {
        read: u64,
        answer: crate::history::Reply,
    },
    Decided {
        to: skein_lib::ReplyTo,
        key: NamedCall,
        took: Option<Box<[u8]>>,
        reach: Option<temper_protocol_reaches::Reach>,
    },
    MakingCopied {
        key: crate::MakingKey,
    },
    MakingAct {
        key: crate::MakingKey,
        act: [u8; 16],
        adoption: Adoption,
        repository: skein_forge::api::Repository,
    },
    /// Make Home's brief task (temper/docs/design/domain/forge/attention.md, 3).
    StatusRun {
        project: u32,
        words: Box<[u8]>,
        signers: Box<[u64]>,
    },
    /// Make the tier run's task; the root answers `TierTask` or `TierRefused`.
    FollowRun {
        project: u32,
        follow: u32,
    },
    CancelFollow {
        project: u32,
        task: u64,
        reason: Box<[u8]>,
    },
    TierRun {
        repository: skein_forge::api::Repository,
        tier: Box<[u8]>,
        number: u64,
        tip: skein_forge::api::Commit,
    },
    /// File a red of no goal's in the decision keeping its verdict.
    FileRedFinding {
        task: u64,
        /// The landing's first kept follow, numbered within the finding's project.
        follow: Option<u32>,
        finding: crate::FileFinding,
    },
    TierDecision {
        task: u64,
        decision: TierDecision,
        /// Execution facts retained when a terminal decision clears the tier row.
        projection: Option<Box<[u8]>>,
        entry: Option<u64>,
    },
    /// The project's adoption erasures precede this post-commit reply.
    ProjectReleased {
        request: Token,
    },
    /// Make or refresh the runtime (temper/docs/design/protocol/forge/engine.md, 2).
    ForgeKept(ForgeRecord),
    /// Drop the runtime and erase secrets (temper/docs/design/protocol/forge/engine.md, 3).
    ForgeGone {
        forge: u16,
    },
    /// Start a box group by kind (temper/docs/design/protocol/setup/box.md, 1).
    BoxGroup {
        forge: u16,
        kind: BoxKind,
        request: Token,
    },
    /// Remove the box group (temper/docs/design/protocol/setup/box.md, 1).
    BoxRemove {
        forge: u16,
    },
    LandingNews {
        task: u64,
        member: u16,
        step: LandingNews,
    },
    LandingDecision {
        task: u64,
        decision: landing::Decision,
        entry: Option<u64>,
        evidence: Option<Box<[LandingEvidence]>>,
    },
    /// The client finished sent entries or reached its bound; those without an outcome.
    Settled {
        unfinished: Box<[Unfinished]>,
    },
    /// The root holds this answer behind the decision's durable commit.
    Bound {
        request: Token,
        answer: BindAnswer,
    },
    /// Make the playground's ordinary records (temper/docs/design/protocol/setup/playground.md).
    PlaygroundActs {
        request: Token,
        forge: u16,
    },
    /// Send the reset's kept act (temper/docs/design/protocol/setup/playground.md).
    ResetAct {
        key: [u8; 16],
        act: ResetAct,
    },
    /// Ask the box link to set up the forge (protocol/forge/engine.md, 8).
    SetUp {
        forge: u16,
        request: Token,
        first: SetUpFirst,
    },
    Connect {
        forge: u16,
        request: Token,
        how: How,
        carried: jig_domain_core_vocabulary::Carried,
    },
    Manifest {
        forge: u16,
        request: Token,
        back: Option<Box<[u8]>>,
    },
    Accept {
        forge: u16,
        request: Token,
        installation: core::num::NonZeroU64,
    },
    Install {
        forge: u16,
        request: Token,
        back: Option<Box<[u8]>>,
    },
    AdoptAccess {
        forge: u16,
        request: Token,
        owner: Box<[u8]>,
        name: Box<[u8]>,
    },
    Reachable {
        forge: u16,
        request: Token,
        installation: Option<u64>,
        page: u32,
    },
    CarriedUnused {
        number: u64,
    },
    EraseCarried {
        number: u64,
    },
    /// The deployment's forges document changed.
    MarkedForges,
    /// Changed panel subjects, bounded by `MARKED` (domain/root.md, 9).
    Marked {
        tasks: Box<[u64]>,
    },
    ApplicationPart {
        request: Token,
        adoption: Adoption,
        repository: skein_forge::api::Repository,
    },
    ApplicationDone {
        request: Token,
        token: Token,
    },
    ApplicationAgain {
        request: Token,
        why: jig_domain_people::Again,
    },
    /// A pushed or first-read change head can enable its dependents (domain/forge/change/questions.md, 1).
    StepDependents {
        task: u64,
    },
    /// A closer's fresh containment read can resume its due step.
    CloserChecked {
        task: u64,
    },
    /// The connector has fresh news or a retry deadline for its procedure.
    WakeProcedure {
        task: u64,
    },
    /// Current role and hold admission for one named resource.
    Resource {
        project: u32,
        name: Name,
        role: crate::Access,
        hold: crate::resources::HoldKind,
    },

    /// Drop a goal's core CI subscription at restart (domain/forge/README.md, 5.2).
    GoalUntopic {
        goal: u64,
        topic: Topic,
    },
    /// Feed the next bounded read of a forge section to this connector's client.
    BriefClient {
        event: skein_forge_client::Event,
    },
    /// One connector-owned section is ready for the root to route to the brief.
    BriefReady {
        owner: Token,
        read: BriefRead,
    },
    /// A held section has this rendered size, or its gather/cut failed.
    BriefSized {
        section: Token,
        size: Result<u32, crate::BriefUnread>,
    },
    /// A section read failed, whether or not its paired brief goes on (domain/forge/change/repairs.md).
    BriefReadRefused {
        section: Token,
        stage: &'static str,
        reason: &'static str,
        heads: Option<(skein_forge::api::Commit, skein_forge::api::Commit)>,
        error: Option<skein_forge::api::Error>,
    },
    /// A held section transferred its rendered bytes to the root.
    BriefTaken {
        section: Token,
        bytes: Option<Box<[u8]>>,
    },
    /// Terminal result of a repository adoption, for the root's person route.
    Adopted {
        reply_to: Token,
        result: Result<Adopted, skein_forge::api::Error>,
    },
    /// Save one record atomically with the current decision, carrying the
    /// owner's post-commit entry and read-afresh handoffs without inspecting it.
    Save {
        record: Stored,
        release: Option<u64>,
        read_afresh: Option<Name>,
    },
    /// Erase one record atomically with the current decision.
    Erase {
        key: Key,
    },
    /// A resource is held or a writer slot is taken.
    Taken {
        task: u64,
        resource: Name,
        by: u64,
    },
    /// A request cannot proceed for a cause without its own typed refusal.
    Refused {
        task: u64,
    },
    /// A named operation failed with its resource and admission or backend cause.
    ResourceRefused {
        task: u64,
        operation: &'static [u8],
        resource: Option<Name>,
        cause: crate::RefusalCause,
    },
    /// A new head did not fit (temper/docs/design/domain/forge/keeping-up.md, 1).
    HeadRefused {
        task: Option<u64>,
        resource: Name,
        configured: u64,
        needed: u64,
    },
    /// A goal's own branch could not be made (domain/forge/branches.md, 1).
    MakeFailed {
        task: u64,
        resource: Name,
        reason: Box<[u8]>,
    },
    /// A run ended, but its branch could not be read afresh.
    LostReadFailed {
        task: u64,
        attempt: u64,
        resource: Name,
        reason: Box<[u8]>,
    },
    /// An entry settled or remains uncertain; root routes the task's news.
    Outcome {
        entry: u64,
        task: u64,
        /// The connector's account survives retirement of its outbox entry.
        account: Option<Box<[u8]>>,
        outcome: skein_forge_client::Outcome,
    },
    /// Projection recovery belongs to the connector and settles no task or call.
    ProjectionOutcome {
        entry: u64,
        outcome: skein_forge_client::Outcome,
    },
    /// A held entry settled as not made, by a person or its task's close.
    NotMade {
        entry: u64,
        task: u64,
    },
    /// A prior effect or release write settled; advance the closing task.
    ContinueRelease {
        task: u64,
    },
    /// A missing goal branch needs a fresh outbox number (domain/forge/branches.md, 1).
    ContinueGoalBranch {
        task: u64,
    },
    /// All effects issued before resource cleanup have settled.
    EffectsSettled {
        task: u64,
    },
    /// All connector resources of a closing task are released.
    Released {
        task: u64,
    },
    /// A failed task's resource remains held by its tree root until that root closes.
    Retained {
        task: u64,
        root: u64,
        resource: Name,
    },
    /// A cleanup effect failed and the closing task needs a person's decision.
    ReleaseFailed {
        task: u64,
        account: Option<EffectAccount>,
    },
    /// A connector-described projection candidate, awaiting core admission.
    ProjectionEffect {
        row: IssueRow,
        entry: skein_forge_client::Entry,
        description: crate::ProjectionEffect,
    },
    /// A projection has to wait until its interval or prior effect settles.
    ProjectAfter {
        goal: u64,
        when: Option<skein_lib::Wall>,
    },
    /// The closed projection has no remaining desired write.
    ProjectionSettled {
        goal: u64,
    },
    /// A projection could not continue under current facts or limits.
    ProjectionFailed {
        goal: u64,
    },
    /// A ready change's fresh review evidence for the root to judge (domain/forge/change/gates.md, 2).
    Reviewed {
        task: u64,
        evidence: ChangeEvidence,
    },
    /// A change policy step and its checked effect entry, when any.
    ChangeDecision {
        task: u64,
        decision: change::Decision,
        entry: Option<u64>,
        evidence: Option<ChangeEvidence>,
    },
    /// Task-specific news with a connector classification.
    News {
        task: u64,
        subscription: u64,
        topic: Topic,
        news: News,
        class: Class,
    },
    /// Drift holds this task until a person releases it.
    Drift {
        task: u64,
        resource: Name,
    },
    /// A bounded provider call, released after preceding progress commits.
    Call {
        call: Token,
        repository: skein_forge::api::Repository,
        op: skein_forge::api::Op,
    },
    /// One bounded fresh connector read for a root worker tool.
    Read {
        owner: Token,
        result: Result<skein_forge::api::Answer, skein_forge::api::Error>,
    },
    /// The requested restart phase finished; the root may request the next.
    RestartDone {
        stage: RestartStage,
    },
    /// Run a Replace: Check on a connected Forgejo, Paste with the kept App id on GitHub
    /// (protocol/forge/engine.md, 2).
    Replace {
        forge: u16,
        request: Token,
        carried: jig_domain_core_vocabulary::Carried,
    },
    /// A forge's link changed or was refused (temper/docs/design/domain/forge/adoption.md, 6).
    LinkNoted {
        forge: u16,
        note: LinkNote,
    },
}

/// Whether this member address belongs to the presented Google subject.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BindAnswer {
    Bound,
    Another,
    /// Every binding row is taken: rows outlive their members (protocol/sign-in.md, 5).
    Full,
}

/// A sent entry still without its outcome when the client’s stop bound passed.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Unfinished {
    pub entry: u64,
    pub task: u64,
}

/// A tier task's step (temper/docs/design/domain/after-landing.md, 10).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TierDecision {
    Wait { until: Option<skein_lib::Wall> },
    Held { words: Box<[u8]> },
    Done { words: Box<[u8]> },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TierHeard {
    pub released: bool,
    /// The root's `tasks.message_bytes`, for the red finding's news body.
    pub message_bytes: u32,
}
/// A goal branch a tier following goal branches follows (temper/docs/design/domain/after-landing.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TierGoal {
    /// The goal task holding the branch.
    pub goal: u64,
    /// `temper/<root>/g<goal>`, as the hold names it.
    pub branch: Box<[u8]>,
    /// The branch's newest tip seen since the goal's hold made it.
    pub tip: Option<skein_forge::api::Commit>,
    /// The tip its last run tested.
    pub ran: Option<skein_forge::api::Commit>,
    /// What its last run failed again after its rerun; empty when green.
    pub failing: temper_engine_domain_forge_tiers::Failures,
    /// When its last run started.
    pub started: Option<skein_lib::Time>,
}
/// Family 28's policy and task (temper/docs/design/domain/after-landing.md, 10).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TierRow {
    /// Observed execution identities awaiting or replaying the task projection across restart.
    pub observations: Box<[u8]>,
    pub repository: skein_forge::api::Repository,
    pub tier: Box<[u8]>,
    pub record: temper_engine_domain_forge_tiers::Record,
    pub task: Option<u64>,
    pub requested: bool,
    pub cancelled: bool,
    pub purpose: Option<crate::purpose::EffectPurpose>,
    pub pending: Box<[TierLanding]>,
    pub baseline: Option<TierLanding>,
    pub branch: TierBranch,
    /// The current or completed probe, retained until its branch is released.
    pub probe: Option<TierProbeBranch>,
    pub flight: Option<TierFlight>,
    pub held: Option<landing::Fault>,
    /// The final part observation or cached receipt awaiting release and judgement.
    pub result: Option<Receipt>,
    pub received: bool,
    pub goals: Box<[TierGoal]>,
    pub goal: Option<u64>,
    pub started: Option<skein_lib::Time>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TierLanding {
    pub commit: skein_forge::api::Commit,
    pub key: Option<[u8; 32]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TierBranch {
    Absent,
    Pending { entry: u64 },
    Made,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TierProbeBranch {
    pub landing: u32,
    pub branch: TierBranch,
}
/// Durable follower progress; trees and workflow details are read again after restart.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TierFlight {
    pub entry: u64,
    pub start: Option<skein_lib::Time>,
    pub since: Option<skein_lib::Time>,
    pub waiting: bool,
    pub next: Option<skein_lib::Time>,
    pub run: Option<u64>,
    pub pending: Box<[skein_forge::api::JobAttempt]>,
    pub fault: Option<landing::Fault>,
    /// A failed part awaiting its artifact, read again after restart.
    pub report: Option<ReceiptRun>,
}
/// Family 29's landing verdicts (temper/docs/design/domain/after-landing.md, 10).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Verification {
    pub repository: skein_forge::api::Repository,
    pub commit: skein_forge::api::Commit,
    pub changes: Box<[u64]>,
    pub tiers: Box<[TierVerification]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TierVerification {
    pub tier: Box<[u8]>,
    pub verdict: Option<TierVerdict>,
    pub repaired: Option<TierRepair>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TierVerdict {
    pub run: u64,
    pub blame: temper_engine_domain_forge_tiers::Blame,
}

/// The run that passed a landing's blamed failures (client/forge/README.md, 2.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TierRepair {
    pub run: u64,
    pub tip: skein_forge::api::Commit,
    pub by: Option<skein_forge::api::Commit>,
}

/// Family 30's ring, oldest first (temper/docs/design/domain/after-landing.md, 10).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LandingLog {
    pub repository: skein_forge::api::Repository,
    pub landings: Box<[LoggedLanding]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LoggedLanding {
    pub commit: skein_forge::api::Commit,
    pub branch: Box<[u8]>,
    pub at: skein_lib::Wall,
    pub changes: Box<[LoggedChange]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LoggedChange {
    pub signed: Box<[SignOff]>,
    pub task: u64,
    pub pull: Option<u64>,
    pub title: Box<[u8]>,
}

/// The five landing transitions reported to its member's goal (domain/landing.md, 4.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LandingNews {
    Probing,
    Joined,
    Proving,
    Landed,
    Left,
    Held,
}
/// Approval evidence retained at admission (temper/docs/design/domain/forge/change/gates.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SignOff {
    pub person: u64,
    pub head: skein_forge::api::Commit,
    pub at: skein_lib::Wall,
}

/// Maximum distinct people retained with one change.
pub const SIGN_OFFS: usize = 8;
