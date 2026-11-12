//! Whole settings values and retained revision facts (client/domain/contract/requests.md and client/domain/contract/README.md, 3 and client/domain/contract/documents.md and client/domain/contract/README.md, 4–5; domain/engine/settings.md).
use super::{
    Authority, DocumentText, Grant, KindImplication, PolicyEditRole, PolicyPermission, PolicyRequirement, ProposalKey,
    Proposer, Template, TemplateOverride, Text, WakePolicy,
};
use alloc::boxed::Box;
use skein_lib::{Duration, List, Wall};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SettingNumber {
    pub number: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Setting {
    Charter(SettingNumber),
    Template(SettingNumber),
    Policy(SettingNumber),
    Rules,
    Model(SettingNumber),
    Application {
        kind: u32,
        scope: u32,
        number: u32,
    },
    /// A project's own policy, an item of its scope (domain/engine/settings.md, 2).
    ProjectPolicy {
        project: u32,
    },
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
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SettingHosts {
    Workers,
    Engine,
    Both,
}
/// A charter's document (jig/docs/design/domain/engine/settings.md, 9).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CharterDocument {
    pub connector: u16,
    pub name: List<Box<[u8]>>,
    pub path: Box<[u8]>,
    pub heading: Option<Box<[u8]>>,
}
/// A charter's instructions for runs whose reasons meet `on`
/// (jig/docs/design/client/domain/contract/requests.md, 2).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CharterPart {
    pub on: u32,
    pub instructions: Text,
}
/// The reasons in bit order (jig/docs/design/client/domain/contract/requests.md, 2).
pub const REASON_NAMES: [&str; 14] = [
    "first",
    "question",
    "question_answered",
    "question_ended",
    "question_owed",
    "escalation",
    "proposal",
    "person_stalled",
    "result",
    "notice",
    "nothing_awaited",
    "news",
    "timer",
    "words",
];
/// A charter whole (jig/docs/design/client/domain/contract/requests.md, 2).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[expect(clippy::struct_excessive_bools, reason = "the whole charter carries independent tool grants and resume")]
pub struct Charter {
    pub rest: bool,
    pub documents: List<CharterDocument>,
    pub parts: List<CharterPart>,
    pub number: u32,
    pub instructions: Text,
    pub waiting: Duration,
    pub resume: bool,
    pub turns: u32,
    pub time: Duration,
    pub reserve_turns: u32,
    pub reserve_time: Duration,
    pub model: SettingModel,
    pub alternatives: List<SettingModel>,
    pub inspect: bool,
    pub modify: bool,
    pub shell: bool,
    pub agents: bool,
    pub call_timeout: Duration,
    pub tools: u64,
    pub hosts: SettingHosts,
    pub spend: u64,
    pub wake: WakePolicy,
}
/// Fields replaced and text appended over a charter's default (domain/engine/overlays.md, 1).
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
pub struct Rules {
    pub ceiling: Authority,
    pub period_spend: u64,
    pub minimum_run_spend: u64,
    pub maximum_run_spend: u64,
    pub implies: List<KindImplication>,
    pub requirements: List<PolicyRequirement>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SettingValue {
    Charter(Box<Charter>),
    Overlay(Box<Overlay>),
    Template(Box<Template>),
    Policy(Box<SettingPolicy>),
    Rules(Box<Rules>),
    Model(Box<CatalogueEntry>),
    Application(Box<[u8]>),
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SettingSet {
    pub value: SettingValue,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SettingEdit {
    Set(SettingSet),
    Removed,
    Reset,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SettingItem {
    pub setting: Setting,
    pub edit: SettingEdit,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Running {
    Stays,
    Moves,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChangeSettings {
    pub against: u32,
    pub items: List<SettingItem>,
    pub running: Running,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OutcomeSettingsChanged {
    pub revision: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RefusalStale {
    pub newest: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RefusalFull {
    pub revision: u32,
    pub tasks: List<u64>,
    pub more: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SettingFault {
    Shape,
    Count,
    Reference,
    Rules,
    Budget,
    Grace,
    Owner,
    Judged,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MalformedSettings {
    pub setting: Setting,
    pub fault: SettingFault,
    pub detail: Option<SettingDetail>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ReadSetting {
    pub setting: Setting,
    pub revision: u32,
    pub proposal: Option<ProposalKey>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ReadRevision {
    pub number: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ReadRevisions {
    pub before: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SettingFollows {
    Overlay,
    Default,
    Change,
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
pub struct SettingLine {
    pub setting: Setting,
    pub name: Option<DocumentText>,
    pub follows: SettingFollows,
    pub last: Option<u32>,
    pub application: Option<ApplicationState>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RevisionParty {
    pub number: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RevisionDecider {
    FirstStart,
    Release,
    Party(RevisionParty),
    List { provider: u32 },
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
pub struct HeldRevision {
    pub head: RevisionHead,
    pub holders: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SettingsProposal {
    pub proposer: Proposer,
    pub number: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SettingsDocument {
    pub newest: u32,
    pub settings: List<SettingLine>,
    pub held: List<HeldRevision>,
    pub pending: List<SettingsProposal>,
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
pub struct Size {
    pub bytes: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SizedEdit {
    Set { size: u32 },
    Remove,
    Reset,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RevisionEdit {
    Set(SettingSet),
    TooLarge(Size),
    Application(SizedEdit),
    Removed,
    Reset,
}
/// What a revision item replaced (jig/docs/design/domain/engine/settings.md, 10).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RevisionReplaced {
    Value(SettingSet),
    TooLarge(Size),
    Default,
    Removed,
    NotOffered,
    Application { size: u32 },
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RevisionItem {
    pub setting: Setting,
    pub edit: RevisionEdit,
    pub replaced: RevisionReplaced,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RevisionDocument {
    pub head: RevisionHead,
    pub items: List<RevisionItem>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RevisionForgotten {
    pub oldest: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RevisionAnswer {
    Revision(RevisionDocument),
    Forgotten(RevisionForgotten),
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ReadRevisionAnswer {
    pub value: RevisionAnswer,
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

/// A deployment-scoped proposal; its accepter chooses whether running work moves.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProposeSettings {
    pub against: u32,
    pub items: List<SettingItem>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OriginRevision {
    pub revision: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SettingOrigin {
    Default(OriginRevision),
    Change(OriginRevision),
    NotOffered,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ReviewValue {
    Value(SettingSet),
    TooLarge(Size),
    None,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ApplicationReview {
    pub edit: SizedEdit,
    pub now: Option<ApplicationState>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ReviewItem {
    pub setting: Setting,
    pub edit: RevisionItemEdit,
    pub now: ReviewValue,
    pub origin: SettingOrigin,
    pub would_be: ReviewValue,
    pub application: Option<ApplicationReview>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SettingsReview {
    pub key: Option<u64>,
    pub newest: u32,
    pub items: List<ReviewItem>,
    pub cut: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProposalActionSettings {
    pub against: u32,
    pub items: List<RevisionItemHead>,
    pub review: Option<SettingsReview>,
    pub made: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RetainedActionSettings {
    pub projects: List<u32>,
    pub against: u32,
    pub items: List<RevisionItemHead>,
    pub made: Option<u32>,
}

/// An owner's move of live agent tasks (domain/engine/settings.md, 4; client/domain/contract/requests.md, 2).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MoveToNewest {
    pub revision: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OutcomeMovedToNewest {
    pub newest: u32,
    pub moved: u32,
    pub stayed: u32,
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

/// A whole model setting (jig/docs/design/domain/engine/settings.md, 11).
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

/// A model act uses the newest catalogue (jig/docs/design/domain/engine/settings.md, 11).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ModelAct {
    Picker { number: u32, on: bool },
    Add { provider: u32, name: Text },
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChangeModel {
    pub act: ModelAct,
}
