//! Neutral whole settings values and the four settings read views.
use alloc::boxed::Box;
use skein_lib::{Duration, List, Wall};
pub const AUTHORITY_MODELS: u32 = 64;
pub const AUTHORITY_MODEL_BYTES: u32 = 4_096;
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AuthorityModels {
    All,
    Named(List<Box<[u8]>>),
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChangeSettings {
    pub against: u32,
    pub items: List<SettingItem>,
    pub running: Running,
}
/// A charter and its parts (temper/docs/design/client/ctl/tools.md, 3).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Charter {
    pub rest: bool,
    /// Resume, inspect, modify, shell and agents, in that order.
    pub switches: [bool; 5],
    pub number: u32,
    pub instructions: Text,
    pub waiting: Duration,
    pub turns: u32,
    pub time: Duration,
    pub reserve_turns: u32,
    pub reserve_time: Duration,
    pub model: SettingModel,
    pub alternatives: List<SettingModel>,
    pub call_timeout: Duration,
    pub tools: u64,
    pub hosts: SettingHosts,
    pub spend: u64,
    pub wake: WakePolicy,
    pub documents: List<CharterDocument>,
    pub parts: List<CharterPart>,
}
/// Literal text and optional replacements (client/ctl/overlays.md, 1).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Overlay {
    pub rest: Option<bool>,
    pub model: Option<SettingModel>,
    pub alternatives: Option<List<SettingModel>>,
    pub inspect: Option<bool>,
    pub modify: Option<bool>,
    pub shell: Option<bool>,
    pub agents: Option<bool>,
    pub tools: Option<u64>,
    pub hosts: Option<SettingHosts>,
    pub waiting: Option<Duration>,
    pub resume: Option<bool>,
    pub turns: Option<u32>,
    pub time: Option<Duration>,
    pub reserve_turns: Option<u32>,
    pub reserve_time: Option<Duration>,
    pub call_timeout: Option<Duration>,
    pub spend: Option<u64>,
    pub documents: Option<List<CharterDocument>>,
    pub instructions: Text,
    pub parts: List<CharterPart>,
    /// Unresolved directory names; empty after assembly.
    pub charters: List<Box<[u8]>>,
}
/// The store names in bit order (temper/docs/design/client/ctl/tools.md, 3).
pub const REASON_NAMES: [&[u8]; 14] = [
    b"first",
    b"question",
    b"question_answered",
    b"question_ended",
    b"question_owed",
    b"escalation",
    b"proposal",
    b"person_stalled",
    b"result",
    b"notice",
    b"nothing_awaited",
    b"news",
    b"timer",
    b"words",
];
/// The kinds every run is offered, which choose no part (temper/docs/design/client/ctl/tools.md, 3).
pub const ALWAYS_OFFERED_NAMES: [&[u8]; 7] = [
    b"answer",
    b"proposal_decision",
    b"settings_decision",
    b"effect_not_made",
    b"met",
    b"amendment",
    b"finish_requested",
];
/// A charter's part (temper/docs/design/client/ctl/tools.md, 3).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CharterPart {
    /// Bits of `REASON_NAMES`, as jig's client `CharterPart.on`.
    pub on: u32,
    pub instructions: Text,
}
/// A charter document (temper/docs/design/protocol/configuration/settings.md, 6).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CharterDocument {
    pub connector: u16,
    pub name: List<Box<[u8]>>,
    pub path: Box<[u8]>,
    pub heading: Option<Box<[u8]>>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DocumentText {
    pub bytes: Text,
    pub cut: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EndedBy {
    /// Its run's judged result ends it.
    Result,
    /// Only its requester ends it; a run's raw finish parks it (5.8).
    Requester,
    /// A named procedure closes an agent task (jig/docs/design/domain/tasks/lifecycle.md, 11).
    Closer(Closer),
    Standing,
}

/// The procedure named to close a task (jig/docs/design/domain/tasks/lifecycle.md, 11).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Closer {
    pub connector: u16,
    pub code: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FieldRule {
    pub name: Box<[u8]>,
    pub max: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HeldRevision {
    pub head: RevisionHead,
    pub holders: u32,
}
/// A template's model choice (temper/docs/design/client/ctl/tools.md, 3).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LlmChoice {
    /// Creator preference, separate from the task's last successfully claimed account.
    pub account: Option<u32>,
    pub main: ModelEffort,
    pub subagents: List<ModelEffort>,
    pub later: Option<LlmLater>,
}
/// The choice after a task's first activation (temper/docs/design/client/ctl/tools.md, 3).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LlmLater {
    pub main: ModelEffort,
    pub subagents: List<ModelEffort>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModelEffort {
    pub model: Text,
    /// One of the efforts the model declares; `None` is its LLM provider's default.
    pub effort: Option<Text>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Authority {
    pub models: Option<AuthorityModels>,
    pub tools: u64,
    pub grants: List<Grant>,
    pub delegation: Delegation,
    pub spend: u64,
    pub deadline: Option<Wall>,
    pub notes: u8,
    pub note_resources: List<ResourceScope>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AuthorityExecutor {
    Charter(u32),
    Procedure(u32),
    Role(u32),
    Job(u32),
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Delegation {
    pub kinds: List<AuthorityExecutor>,
    pub tasks: u32,
    pub depth: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Grant {
    pub connector: u16,
    pub kind: u16,
    pub pattern: Pattern,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KindImplication {
    pub connector: u16,
    pub kind: u16,
    pub implies: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Part {
    pub connector: u16,
    pub kind: PartKind,
    pub key: PartKey,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PartKey {
    Path(List<Box<[u8]>>),
    Number(u64),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PartKind {
    Resource,
    Pool,
    Result,
    Procedure,
    Effect,
    Outbox,
    /// The supplied connector account for the numbered held task.
    Hold,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Pattern {
    pub segments: List<Box<[u8]>>,
    pub last: PatternEnd,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PatternEnd {
    Exact(Box<[u8]>),
    Open(Box<[u8]>),
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceScope {
    pub connector: u16,
    pub pattern: Pattern,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PolicyEditRole {
    pub number: u32,
    pub authority: Authority,
    pub period_spend: u64,
    pub requests: u16,
    pub decides: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PolicyGuard {
    Guarded,
    Observed { freshness: Duration },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PolicyJudge {
    pub connector: u16,
    pub requirement: u16,
    pub parameters: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PolicyPermission {
    pub connector: u16,
    pub permission: u16,
    pub role: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PolicyRequirement {
    pub connector: u16,
    pub kind: u16,
    pub pattern: Pattern,
    pub judge: PolicyJudge,
    pub guard: PolicyGuard,
    pub must_be_guarded: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProposalKey {
    pub proposer: Proposer,
    pub number: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Proposer {
    /// A task's proposal store.
    Task(u64),
    /// A party's proposal store.
    Party(u64),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Retention {
    Kept,
    ForgottenOnceRead,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RevisionAnswer {
    Revision(RevisionDocument),
    Forgotten(RevisionForgotten),
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RevisionDecider {
    FirstStart,
    Release,
    Party(RevisionParty),
    List { provider: u32 },
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RevisionDocument {
    pub head: RevisionHead,
    pub items: List<RevisionItem>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RevisionEdit {
    Application(SizedEdit),
    Set(SettingSet),
    TooLarge(Size),
    Removed,
    Reset,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RevisionForgotten {
    pub oldest: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RevisionHead {
    pub number: u32,
    pub against: u32,
    pub at: Wall,
    pub by: RevisionDecider,
    pub proposal: Option<ProposalKey>,
    pub moved: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RevisionItem {
    pub setting: Setting,
    pub edit: RevisionEdit,
    pub replaced: RevisionReplaced,
}
/// The state before an item changed it (jig/docs/design/domain/engine/settings.md, 10).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RevisionReplaced {
    Value(SettingSet),
    TooLarge(Size),
    Application { size: u32 },
    Default,
    Removed,
    NotOffered,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RevisionItemEdit {
    Set,
    Removed,
    Reset,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RevisionItemHead {
    pub setting: Setting,
    pub edit: RevisionItemEdit,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RevisionParty {
    pub number: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RevisionRow {
    pub head: RevisionHead,
    pub items: List<RevisionItemHead>,
    pub more: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RevisionsPage {
    pub rows: List<RevisionRow>,
    pub next: Option<u32>,
    pub forgotten: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Rules {
    pub ceiling: Authority,
    pub period_spend: u64,
    pub minimum_run_spend: u64,
    pub maximum_run_spend: u64,
    pub implies: List<KindImplication>,
    pub requirements: List<PolicyRequirement>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Running {
    Stays,
    Moves,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Setting {
    Model(SettingNumber),
    Application {
        kind: u32,
        scope: u32,
        number: u32,
    },
    Charter(SettingNumber),
    Template(SettingNumber),
    Policy(SettingNumber),
    Rules,
    /// A project's own policy (jig/docs/design/domain/engine/settings.md, 2).
    ProjectPolicy {
        project: u32,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SettingDocument {
    pub overlay: Option<Box<Overlay>>,
    pub setting: Setting,
    pub revision: u32,
    pub value: Option<SettingValue>,
    pub application: Option<ApplicationState>,
    pub proposed: Option<SizedEdit>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SettingEdit {
    Set(SettingSet),
    Removed,
    Reset,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SettingFault {
    Judged,
    Shape,
    Count,
    Reference,
    Rules,
    Budget,
    Grace,
    Owner,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SettingFollows {
    Overlay,
    Default,
    Change,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SettingHosts {
    Workers,
    Engine,
    Both,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SettingItem {
    pub setting: Setting,
    pub edit: SettingEdit,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SettingLine {
    pub setting: Setting,
    pub name: Option<DocumentText>,
    pub follows: SettingFollows,
    pub last: Option<u32>,
    pub application: Option<ApplicationState>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SettingModel {
    pub provider: u32,
    pub dialect: u32,
    pub endpoint: u32,
    pub program: Option<Text>,
    pub name: Text,
    pub window: u32,
    pub output: u32,
    pub input_price: u64,
    pub cached_price: u64,
    pub cache_write_price: Option<u64>,
    pub output_price: u64,
    pub price_unit: u32,
    pub efforts: List<Text>,
    pub effort: Option<Text>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SettingNumber {
    pub number: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SettingPolicy {
    pub escalation_role: Option<u32>,
    pub ceiling: Authority,
    pub projections: List<Grant>,
    pub period_spend: u64,
    pub roles: List<PolicyEditRole>,
    pub requirements: List<PolicyRequirement>,
    pub permissions: List<PolicyPermission>,
    pub templates: List<TemplateOverride>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SettingSet {
    pub value: SettingValue,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SettingValue {
    Overlay(Box<Overlay>),
    Model(Box<CatalogueEntry>),
    Application(Box<[u8]>),
    Charter(Box<Charter>),
    Template(Box<Template>),
    Policy(Box<SettingPolicy>),
    Rules(Box<Rules>),
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SettingsDocument {
    pub newest: u32,
    pub settings: List<SettingLine>,
    pub held: List<HeldRevision>,
    pub pending: List<SettingsProposal>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SettingsProposal {
    pub proposer: Proposer,
    pub number: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Size {
    pub bytes: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TaskContractDocument {
    Approval { state: Box<[u8]>, part: Part },
    Report { words: u32 },
    Verdict { choices: List<VerdictChoice>, cut: bool },
    Change { connector: u16, kind: u16, words: u32 },
    Job { bytes: u32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TaskFlags {
    pub conversation: bool,
    pub listed: bool,
    pub retention: Retention,
    pub ended_by: EndedBy,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Template {
    /// Connector symbols resolved to resources when the task is created.
    pub resources: List<Symbol>,
    pub number: u32,
    pub name: DocumentText,
    pub executor: TemplateExecutor,
    pub contract: TaskContractDocument,
    pub shallower: u32,
    pub budget: TemplateBudget,
    pub deadline: TemplateDeadline,
    pub tracked: Option<u32>,
    pub flags: TaskFlags,
    pub label: u32,
    pub grace: Option<Duration>,
    /// The LLM choice the template gives its task; `None` is its charter's default.
    pub llm: Option<LlmChoice>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TemplateBudget {
    Amount(u64),
    AsAllowed { cap: Option<u64> },
    Given,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TemplateDeadline {
    None,
    Span(Duration),
    Given,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TemplateExecutor {
    Charter(u32),
    Chosen,
}
/// A connector's resource named after the task (jig/docs/design/protocol/configuration.md, 3.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Symbol {
    /// The connector that resolves the symbol.
    pub connector: u16,
    /// The connector's resource symbol.
    pub symbol: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TemplateOverride {
    pub template: u32,
    pub executor: Option<TemplateExecutor>,
    pub budget: Option<TemplateBudget>,
    pub deadline: Option<TemplateDeadline>,
    pub priority: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Text {
    bytes: Box<[u8]>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VerdictChoice {
    pub code: u32,
    pub words: u32,
    pub followups: u32,
    pub fields: List<FieldRule>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadValue {
    Settings(Box<SettingsDocument>),
    Setting(Box<SettingDocument>),
    Revision(Box<RevisionAnswer>),
    Revisions(Box<RevisionsPage>),
}
impl Text {
    pub fn new(bytes: Box<[u8]>, limit: u32) -> Result<Self, Box<[u8]>> {
        let bounds = skein_json::writer::Limits { depth: 1, length: limit.saturating_mul(6).saturating_add(2) };
        let mut text = skein_json::writer::Encoder::measure(&bounds);
        text.string(&bytes);
        if bytes.len() > usize::try_from(limit).expect("text bound") || text.measured().is_err() {
            return Err(bytes);
        }
        Ok(Self { bytes })
    }
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    #[must_use]
    pub fn into_bytes(self) -> Box<[u8]> {
        self.bytes
    }
}
impl DocumentText {
    #[must_use]
    pub fn complete(bytes: Text) -> Self {
        Self { bytes, cut: false }
    }
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.bytes.bytes()
    }
    #[must_use]
    pub fn whole(&self) -> Option<Text> {
        if self.cut { None } else { Some(self.bytes.clone()) }
    }
}
impl Charter {
    #[must_use]
    pub fn resume(&self) -> bool {
        *self.switches.first().expect("charter switch")
    }
    #[must_use]
    pub fn inspect(&self) -> bool {
        *self.switches.get(1).expect("charter switch")
    }
    #[must_use]
    pub fn modify(&self) -> bool {
        *self.switches.get(2).expect("charter switch")
    }
    #[must_use]
    pub fn shell(&self) -> bool {
        *self.switches.get(3).expect("charter switch")
    }
    #[must_use]
    pub fn agents(&self) -> bool {
        *self.switches.get(4).expect("charter switch")
    }
}

/// A charter's default activation policy (jig/docs/design/domain/tasks/messages.md, 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WakePolicy {
    pub words: WakeRule,
    pub notices: WakeRule,
    pub news: WakeRule,
    pub results: ResultsWake,
    pub questions: bool,
    pub answers: bool,
    pub timers: bool,
}
impl WakePolicy {
    pub const DEFAULT: Self = Self {
        words: WakeRule::Immediate,
        notices: WakeRule::Immediate,
        news: WakeRule::Immediate,
        results: ResultsWake::LastOrFailure,
        questions: true,
        answers: true,
        timers: true,
    };
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WakeRule {
    Never,
    Immediate,
    Batch { count: u32, age: Duration },
    Paced { span: Duration },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResultsWake {
    Never,
    Each,
    LastOrFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ApplicationFollows {
    Default,
    Changed { revision: u32 },
    Removed { revision: u32 },
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ApplicationState {
    pub follows: ApplicationFollows,
    pub size: u32,
    pub charters: List<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SizedEdit {
    Set { size: u32 },
    Remove,
    Reset,
}

/// The charter member rejected by the settings check (jig/docs/design/domain/engine/settings.md, 9).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SettingMember {
    /// The charter's document count.
    Documents,
    /// A document resource name's segment count.
    NameSegments,
    /// One segment's byte length in a document resource name.
    NameSegmentBytes,
    /// The document path's byte length.
    Path,
    /// The optional heading's byte length.
    Heading,
    /// The document connector's registration.
    Connector,
    /// The charter's stored policy and parts byte length.
    StoredPolicy,
}
/// The failed setting rule; scalar bounds are carried beside it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SettingRule {
    /// The supplied count or length exceeds the configured maximum.
    Maximum,
    /// The supplied length falls below the required minimum.
    Minimum,
    /// The supplied connector number is not registered.
    Registered,
}
/// Bounded refusal facts, without copying any of the rejected charter's bytes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SettingDetail {
    /// The member relative to the charter.
    pub member: SettingMember,
    /// Document index; zero for documents or stored policy.
    pub document: u32,
    /// Name segment index; zero for members other than `NameSegmentBytes`.
    pub segment: u32,
    /// The failed check, interpreted with the member and bound.
    pub rule: SettingRule,
    /// The supplied count, byte length or connector number.
    pub supplied: u32,
    /// The minimum or maximum; zero for `Registered`.
    pub bound: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CatalogueEntry {
    pub number: u32,
    pub model: SettingModel,
    pub shown: Option<Text>,
    pub description: Option<Text>,
    pub picker: bool,
    pub withdrawn: bool,
    pub source: ModelSource,
    pub estimated: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModelSource {
    Stock,
    List,
    Person,
}
