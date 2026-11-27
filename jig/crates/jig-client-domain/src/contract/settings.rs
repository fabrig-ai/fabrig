//! Whole settings values and retained revision facts (client/domain/contract/requests.md and client/domain/contract/answers.md, 1 and client/domain/contract/documents.md and client/domain/contract/reads.md, 1, and client/domain/contract/watches.md, 1; domain/engine/settings.md).
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
    Limits,
    Model(SettingNumber),
    Worker(SettingNumber),
    Provider(SettingNumber),
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
    pub speeds: List<SettingSpeed>,
    pub speed: Option<Text>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SettingSpeed {
    pub name: Text,
    pub also: List<Text>,
    pub input_price: u64,
    pub cached_price: u64,
    pub cache_write_price: Option<u64>,
    pub output_price: u64,
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
/// A limit override carried without resolving its name (domain/engine/settings.md, 1).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Override {
    pub name: Box<[u8]>,
    pub value: u64,
    pub reason: Box<[u8]>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SettingValue {
    Charter(Box<Charter>),
    Overlay(Box<Overlay>),
    Template(Box<Template>),
    Policy(Box<SettingPolicy>),
    Rules(Box<Rules>),
    Limits(Box<[Override]>),
    Model(Box<CatalogueEntry>),
    Worker(Box<WorkerSetting>),
    Provider(Box<ProviderSetting>),
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
/// The last charter, template, policy or model number; mirrors jig-domain-people's settings vocabulary.
pub const NUMBERS: u32 = 64;
/// The stock's last number; higher numbers through `NUMBERS` are a deployment's own.
pub const STOCK_NUMBERS: u32 = 32;
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
    Linked,
    Accounts,
    Range,
    Name,
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
    pub shown: Option<DocumentText>,
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

/// A whole model setting (jig/docs/design/domain/engine/settings-catalogue.md, 1).
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

/// A model act uses the newest catalogue (jig/docs/design/domain/engine/settings-catalogue.md, 1).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ModelAct {
    Picker { number: u32, on: bool },
    Add { provider: u32, name: Text },
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChangeModel {
    pub act: ModelAct,
}

/// Worker value (jig/docs/design/domain/engine/workers.md, 1).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WorkerSetting {
    pub number: u32,
    pub name: Box<[u8]>,
    pub places: u32,
    pub digest: Option<[u8; 32]>,
    pub withdrawn: bool,
}
/// A stable name's number, or the lowest unused number (workers.md, 2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WorkerNumber {
    Live(u32),
    Withdrawn(u32),
    Free(u32),
}
#[must_use]
pub fn worker_number(workers: &[super::WorkerState], name: &[u8]) -> WorkerNumber {
    for worker in workers {
        if worker.name.as_ref() == name {
            return if worker.withdrawn {
                WorkerNumber::Withdrawn(worker.number)
            } else {
                WorkerNumber::Live(worker.number)
            };
        }
    }
    let mut number = 1_u32;
    // At most one more than the number of rows can be the first hole.
    for _ in 0..=workers.len() {
        let mut held = false;
        for worker in workers {
            held |= worker.number == number;
        }
        if !held {
            return WorkerNumber::Free(number);
        }
        number = number.checked_add(1).expect("worker count leaves a free number");
    }
    unreachable!("one more candidate than held numbers")
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum EndpointAddress {
    Socket(core::net::SocketAddr),
    Host { name: Box<[u8]>, port: u16 },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProviderSetting {
    pub provider: Provider,
    pub endpoints: Box<[Endpoint]>,
    pub withdrawn: bool,
}
/// One configured LLM provider.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Provider {
    /// Its name for showing.
    pub name: Box<[u8]>,
    pub number: u32,
    /// The endpoint its added models take (protocol/configuration-engine.md, 1).
    pub endpoint: Option<Box<[u8]>>,
    /// The program-free endpoint its sign-ins and keys take (protocol/configuration-engine.md, 1).
    pub direct: Option<Box<[u8]>>,
    pub list: Option<ProviderList>,
    pub signed_in_plan: bool,
    pub manage_usage: Option<Box<[u8]>>,
    /// The sign-in it offers, with its public registration.
    pub sign_in: Option<Registration>,
    /// Whether it offers a key.
    pub keys: bool,
    /// The programs it offers, such as `claude_code`.
    pub programs: Box<[Box<[u8]>]>,
}

/// A provider's public registration for its sign-in: a client secret is refused.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Registration {
    pub issuer: Box<[u8]>,
    pub client: Box<[u8]>,
    pub scopes: Box<[Box<[u8]>]>,
    pub name_hint: Option<Box<[u8]>>,
    pub authorization: Box<[u8]>,
    pub token: Box<[u8]>,
    pub revocation: Option<Box<[u8]>>,
    pub keys: Box<[u8]>,
    pub endpoint_transport: Transport,
    pub must_grant: Box<[Box<[u8]>]>,
    pub resource: Option<Box<[u8]>>,
    pub redirect_path: Box<[u8]>,
    pub redirect_port: Option<u16>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Endpoint {
    pub name: Box<[u8]>,
    pub number: u32,
    pub dialect: u32,
    pub provider: Box<[u8]>,
    pub program: Option<Box<[u8]>>,
    pub address: EndpointAddress,
    pub transport: Transport,
    pub headers: Box<[Header]>,
    pub connect: Duration,
    pub handshake: Duration,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Header {
    pub name: Box<[u8]>,
    pub value: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Transport {
    Tls { server_name: Box<[u8]>, trust: Option<Box<[u8]>> },
    Plaintext,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProviderList {
    pub keep: Box<[Box<[u8]>]>,
    pub drop: Box<[Box<[u8]>]>,
}

#[cfg(test)]
mod worker_tests {
    use super::*;
    fn row(number: u32, name: &[u8], withdrawn: bool) -> super::super::WorkerState {
        super::super::WorkerState {
            number,
            name: name.into(),
            places: 1,
            digest: None,
            withdrawn,
            linked: false,
            since: None,
            pause: super::super::WorkerPause::Taking,
        }
    }
    #[test]
    fn worker_number_keeps_withdrawn_names_and_takes_the_lowest_hole() {
        let rows = [row(1, b"local", false), row(2, b"w1", true)];
        assert_eq!(worker_number(&rows, b"local"), WorkerNumber::Live(1));
        assert_eq!(worker_number(&rows, b"w1"), WorkerNumber::Withdrawn(2));
        assert_eq!(worker_number(&rows, b"w2"), WorkerNumber::Free(3));
        assert_eq!(worker_number(&[row(2, b"w1", false)], b"w9"), WorkerNumber::Free(1));
    }
}
