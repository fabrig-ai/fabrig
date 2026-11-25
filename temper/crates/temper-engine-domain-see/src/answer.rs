//! Answers for the host renderer (temper/docs/design/domain/agent/ask-temper.md, 1).
use alloc::boxed::Box;
use jig_domain_core::reads::Answer;
use temper_engine_domain_store::CallAnswer;

/// What a `see` call read, for its renderer (domain/agent/ask-temper.md, 1).
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum SeeAnswer {
    /// The project's ways (temper/docs/design/protocol/configuration/settings.md, 8).
    Ways(Box<temper_engine_domain_ways::WaysDocument>),
    /// A `settings` family read's answer, with an inventory's context.
    Settings(Box<SeeSettings>),
    /// An address read as the chat's person (domain/agent/ask-temper.md, 1).
    Person(Box<SeePerson>),
    /// An address read as the person whose first read was refused.
    NotRead(SeeNotRead),
    /// A read's own answer as the tool `tool` renders it: the settings family's refusal
    /// (`b"settings"`); a branch or file read's answer or refusal (`b"read_branch"`,
    /// `b"read_file"`); a repository the project has none of (`b"read_file"`).
    Call { tool: &'static [u8], answer: Box<CallAnswer> },
    /// `help` (the page `index`) and `help NAME`.
    Page { name: Box<[u8]> },
    /// `help search WORDS`, the words joined by single spaces.
    Search { words: Box<[u8]> },
    /// `settings landing` (filled by `see-forge`).
    Landing(Box<SeeLanding>),
    /// `settings gates OWNER/NAME` (filled by `see-forge`).
    Gates(Box<SeeRepository>),
    /// `proposal tN/K`, `proposal pN/K` (filled by `see-proposal`).
    Proposal(Box<SeeProposal>),
    /// No address.
    Unknown,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SeeNotRead {
    /// Answered unknown: "nothing you may read in this project is there."
    Unknown,
    /// Answered busy: "temper is busy; read it again."
    Busy,
    /// The chat's requester is a task or the deployment: "this chat has no person to read as."
    NoPerson,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct SeePerson {
    pub read: SeeRead,
    pub names: SeeNames,
}

/// Each read's answer as the core gave it, `jig_domain_core::reads::Answer`
/// (`Answer` below), in the order 8.4.1's *Over* names them. A read after the
/// first may be `Answer::None(_)`: its part says it was not read.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum SeeRead {
    /// `Read::Project`, then `Read::Inbox { group: Needs, filter: { project, goal: None }, from: None }`,
    /// then `Read::Question` for each question in that page, in order.
    Me { person: u64, project: Answer, needs: Answer, words: Box<[SeeWords]> },
    /// `Read::Project`, then `Read::Tasks` of the project's held tasks
    /// (`phase: Some(Held)`, `order: Number`), live group; repositories as the inventory names them.
    Project { project: Answer, held: Answer, repositories: Box<[RepositoryIdentity]> },
    /// `Read::Tasks` of the project's tracked tasks (`tracked: Some(true)`), live by
    /// `Priority`, or ended (`phase: Some(Ended)`, `order: Ended`); then `Read::Notes { project }`.
    Goals { ended: bool, goals: Answer, reports: Answer },
    /// `Read::Tasks` of the project, `order: Activity`, live group.
    Tasks { tasks: Answer },
    /// `Read::Task { task }`; then, for a tracked task, `Read::ScopedNotes` of its goal scope.
    Task { number: u64, task: Answer, report: Option<Answer> },
    /// `Read::Tasks` with `parent: Some(number)`, `order: Number`: its live group, then its ended group.
    Tree { number: u64, live: Answer, ended: Answer },
    /// `Read::Activity { task, before: None }`.
    Activity { number: u64, activity: Answer },
    /// `Read::Turn { task, attempt, turn }`.
    Turn { number: u64, attempt: u64, turn: u32, read: Answer },
    /// The forge's panel of change `number`, `Document::Change`.
    Change { number: u64, panel: Box<temper_engine_domain_forge::Document> },
    /// The forge's `changes_subject(project)` and `landings_subject(project, false)` documents;
    /// `None` where the forge answered unknown.
    Changes {
        changes: Option<Box<temper_engine_domain_forge::Document>>,
        landings: Option<Box<temper_engine_domain_forge::Document>>,
    },
    /// `Read::Inbox { group: Needs, filter: { project, goal: None }, from: None }`,
    /// then `Read::Question` for each question in that page, in order.
    Inbox { needs: Answer, words: Box<[SeeWords]> },
}

/// A needs-page question's words as its person read answered (domain/agent/ask-temper.md, 1).
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct SeeWords {
    pub task: u64,
    pub message: u64,
    pub words: Answer,
}

/// Names the answer's numbers need (8.4.1, *How an answer reads*).
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct SeeNames {
    /// The chat's project, by number and name.
    pub project: SeeName,
    /// The project's people, each by number and name, their login where no name is kept.
    pub people: Box<[SeeName]>,
    /// The charters at the newest revision, each by number and name.
    pub charters: Box<[SeeName]>,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct SeeName {
    pub number: u64,
    pub name: Box<[u8]>,
}

/// What `SeeAnswer::Settings` carries (temper/docs/design/domain/agent/ask-temper.md, 1).
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct SeeSettings {
    pub answer: jig_domain_core::SettingsAnswer,
    pub repositories: Box<[RepositoryIdentity]>,
    pub people: Box<[ProjectPerson]>,
    pub forge_people: Box<[ForgePerson]>,
    pub offer: Box<[OfferedModel]>,
    pub application: Option<temper_engine_domain_forge::settings::SizedValue>,
    pub requested: Option<u64>,
    /// Section 1, *A deployment landing rule the project holds*: `Some(N)` only
    /// for a `see` asking `Setting::Application { kind: 1, scope: 0, number: N }`
    /// answered `SettingsAnswer::Unknown`, whose chat's project holds N;
    /// `None` for every other answer.
    pub project_landing_rule: Option<u32>,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct SeeLanding {
    pub by: temper_engine_domain_forge_change::Way,
    pub merge_job: Option<Box<[u8]>>,
    /// The project's after-landing settings, where its configuration gives them.
    pub after: Option<temper_engine_domain_forge::AfterLanding>,
    /// Each of the project's readable repositories, in the inventory's order.
    pub repositories: Box<[SeeRepository]>,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct SeeRepository {
    /// `OWNER/NAME`, as the inventory names it.
    pub name: Box<[u8]>,
    /// Its gates and tiers; both empty where the configuration gives it none.
    pub checks: temper_engine_domain_forge::RepositoryGates,
}

/// A proposal of settings as jig's run read answers it (9.4).
#[derive(PartialEq, Eq, Hash, Debug)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(clippy::large_enum_variant, reason = "the proposal contract carries documents inline, boxed by SeeAnswer")
)]
pub enum SeeProposal {
    Waiting(jig_domain_core::documents::Proposal),
    Decided(jig_domain_core::reads::Ended),
    Unknown,
}

/// An adopted repository's canonical resource identity for settings discovery
/// (temper/docs/design/domain/agent/agents.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RepositoryIdentity {
    /// The host in resource names, including any port.
    pub host: Box<[u8]>,
    /// The repository's forge owner.
    pub owner: Box<[u8]>,
    /// The repository's name within that owner.
    pub name: Box<[u8]>,
    /// The branch its adoption names for landing.
    pub landing: Box<[u8]>,
}

/// One person of the caller's project, for an authorized inventory
/// (temper/docs/design/domain/agent/agents.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProjectPerson {
    pub person: u64,
    pub role: u32,
    /// Empty for someone adoption seeded who has not signed in.
    pub login: Box<[u8]>,
    /// Their account on each forge of the project's repositories, each forge once,
    /// in the order the repositories first name it (domain/agent/agents.md, 1).
    pub forges: Box<[PersonForge]>,
    /// The person this chat's task was requested by.
    pub you: bool,
}
/// What the lookup answers for one person on one forge (domain/agent/agents.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PersonForge {
    /// The forge's host, as the inventory's repository lines name it.
    pub host: Box<[u8]>,
    /// Their user there, unknown as `None`.
    pub user: Option<u64>,
    /// That user is temper's retained writer on this forge, who never counts as a person.
    pub writer: bool,
}
/// One model of the offer, for an authorized inventory (temper/docs/design/domain/agent/agents.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct OfferedModel {
    pub name: Box<[u8]>,
    pub provider: Box<[u8]>,
    pub efforts: Box<[Box<[u8]>]>,
    pub effort: Option<Box<[u8]>>,
}

/// An owned forge's person, the login made for whoever runs the box.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ForgePerson {
    pub host: Box<[u8]>,
    pub login: Box<[u8]>,
}
