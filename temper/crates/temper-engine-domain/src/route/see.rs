//! `see`: Ask temper's read by address (temper/docs/design/domain/agent/ask-temper.md, 1).
pub(super) mod forge;
pub(crate) mod person;
pub(super) mod proposal;

use super::host_route::{FlightRender, ForgePerson, OfferedModel, ProjectPerson, RepositoryIdentity};
use super::{Decision, Delivery, Domain, Env, Limits, ReplyTo, Work, emit};
use alloc::boxed::Box;
use jig_domain_core::{self as core, reads::Answer, settings::Setting};
use skein_lib::{List, Writer};

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
    Call { tool: &'static [u8], answer: Box<crate::CallAnswer> },
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

/// What `HostDelivery::Settings` carries beside its reply and offset.
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct SeeSettings {
    pub answer: jig_domain_core::SettingsAnswer,
    pub repositories: Box<[RepositoryIdentity]>,
    pub people: Box<[ProjectPerson]>,
    pub forge_people: Box<[ForgePerson]>,
    pub offer: Box<[OfferedModel]>,
    pub application: Option<temper_engine_domain_forge::settings::SizedValue>,
    pub requested: Option<u64>,
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

/// A `see` read in flight (domain/agent/ask-temper.md, 1).
#[derive(Hash, Debug)]
pub(crate) struct SeeFlight {
    pub line: Box<[u8]>,
    pub reading: SeeReading,
}
#[derive(Hash, Debug)]
pub(crate) enum SeeReading {
    Settings,
    Head,
    File,
    Proposal,
    Person(Box<person::Gathered>),
}

/// One address, as its line parses in the chat's project (domain/agent/ask-temper.md, 1).
#[derive(PartialEq, Eq, Debug)]
pub(crate) enum Address {
    Ways,
    Inventory,
    Setting(Setting),
    Landing,
    Gates { repository: Box<[u8]> },
    Revisions,
    Revision { number: u32 },
    Proposal { proposer: jig_domain_people::Proposer, number: u64 },
    Page { name: Box<[u8]> },
    Search { words: Box<[u8]> },
    File { repository: Box<[u8]>, path: Box<[u8]>, branch: Option<Box<[u8]>> },
    Me,
    Project,
    Goals { ended: bool },
    Tasks,
    Task { number: u64 },
    Tree { number: u64 },
    Activity { number: u64 },
    Turn { number: u64, attempt: u64, turn: u32 },
    Change { number: u64 },
    Changes,
    Inbox,
    Unknown,
}

/// ASCII words; positive decimal numbers and bounded page and repository names.
#[expect(clippy::too_many_lines, reason = "one grammar table names every see address")]
pub(crate) fn address(line: &[u8], project: u32) -> Address {
    let mut words = List::with_capacity(u32::try_from(line.len()).expect("address bound"));
    let mut from = 0_usize;
    for (at, byte) in line.iter().enumerate() {
        if byte.is_ascii_whitespace() {
            if from < at {
                words.push(line.get(from..at).expect("address word")).expect("admitted address words");
            }
            from = at.checked_add(1).expect("address offset");
        }
    }
    if from < line.len() {
        words.push(line.get(from..).expect("address word")).expect("admitted address words");
    }
    match words.as_slice() {
        [b"ways"] => Address::Ways,
        [b"settings"] => Address::Inventory,
        [b"settings", b"rules"] => Address::Setting(Setting::Rules),
        [b"settings", b"landing"] => Address::Landing,
        [b"settings", b"gates", repository] if repository_name(repository) => {
            Address::Gates { repository: (*repository).into() }
        }
        [b"settings", b"project-areas"] => {
            Address::Setting(Setting::Application { kind: 14, scope: project, number: 1 })
        }
        [b"settings", b"project-triager"] => {
            Address::Setting(Setting::Application { kind: 15, scope: project, number: 1 })
        }
        [b"settings", b"project-words"] => {
            Address::Setting(Setting::Application { kind: 10, scope: project, number: 1 })
        }
        [b"settings", b"project-ways"] => Address::Setting(Setting::Application { kind: 7, scope: project, number: 1 }),
        [b"settings", b"project-questions"] => {
            Address::Setting(Setting::Application { kind: 6, scope: project, number: 1 })
        }
        [b"settings", b"project-classes"] => {
            Address::Setting(Setting::Application { kind: 8, scope: project, number: 1 })
        }
        [b"settings", b"dependency", number] => match small(number) {
            Some(number) => Address::Setting(Setting::Application { kind: 13, scope: project, number }),
            None => Address::Unknown,
        },
        [b"settings", b"follow", number] => match small(number) {
            Some(number) => Address::Setting(Setting::Application { kind: 12, scope: project, number }),
            None => Address::Unknown,
        },
        [b"settings", b"charter", number] => setting_number(b"charter", number),
        [b"settings", b"template", number] => setting_number(b"template", number),
        [b"settings", b"policy", number] => setting_number(b"policy", number),
        [b"settings", b"model", number] => setting_number(b"model", number),
        [b"settings", b"charter-words", number] => setting_number(b"charter-words", number),
        [b"settings", b"forge-charter", number] => setting_number(b"forge-charter", number),
        [b"settings", b"template-goal", number] => setting_number(b"template-goal", number),
        [b"settings", b"landing-rule", number] => setting_number(b"landing-rule", number),
        [b"settings", b"landing-rule", number, b"project"] => match small(number) {
            Some(number) => Address::Setting(Setting::Application { kind: 2, scope: project, number }),
            None => Address::Unknown,
        },
        [b"revisions"] => Address::Revisions,
        [b"revision", number] => match small(number) {
            Some(number) => Address::Revision { number },
            None => Address::Unknown,
        },
        [b"proposal", name] => proposal_name(name),
        [b"help"] => Address::Page { name: b"index".as_slice().into() },
        [b"help", b"search", rest @ ..] if !rest.is_empty() => {
            let mut length = 0_usize;
            for word in rest {
                if length != 0 {
                    length = length.checked_add(1).expect("address bound");
                }
                length = length.checked_add(word.len()).expect("address bound");
            }
            let mut joined = Writer::new(length);
            let mut first = true;
            for word in rest {
                if !first {
                    joined.put(b" ").expect("address bound");
                }
                joined.put(word).expect("address bound");
                first = false;
            }
            Address::Search { words: joined.finish() }
        }
        [b"help", name] if *name != b"search" && page_name(name) => Address::Page { name: (*name).into() },
        [b"file", repository, path] if repository_name(repository) => {
            Address::File { repository: (*repository).into(), path: (*path).into(), branch: None }
        }
        [b"file", repository, path, branch] if repository_name(repository) => {
            Address::File { repository: (*repository).into(), path: (*path).into(), branch: Some((*branch).into()) }
        }
        [b"me"] => Address::Me,
        [b"project"] => Address::Project,
        [b"goals"] => Address::Goals { ended: false },
        [b"goals", b"ended"] => Address::Goals { ended: true },
        [b"tasks"] => Address::Tasks,
        [b"task", number] => match natural(number) {
            Some(number) => Address::Task { number },
            None => Address::Unknown,
        },
        [b"task", number, b"tree"] => match natural(number) {
            Some(number) => Address::Tree { number },
            None => Address::Unknown,
        },
        [b"task", number, b"activity"] => match natural(number) {
            Some(number) => Address::Activity { number },
            None => Address::Unknown,
        },
        [b"task", number, b"turn", turn] => {
            let Some(number) = natural(number) else { return Address::Unknown };
            match turn_number(turn) {
                Some((attempt, turn)) => Address::Turn { number, attempt, turn },
                None => Address::Unknown,
            }
        }
        [b"change", number] => match natural(number) {
            Some(number) => Address::Change { number },
            None => Address::Unknown,
        },
        [b"changes"] => Address::Changes,
        [b"inbox"] => Address::Inbox,
        _ => Address::Unknown,
    }
}

fn natural(word: &[u8]) -> Option<u64> {
    if word.is_empty() {
        return None;
    }
    let mut number = 0_u64;
    for byte in word {
        if !byte.is_ascii_digit() {
            return None;
        }
        number = number.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
    }
    if number == 0 { None } else { Some(number) }
}
fn small(word: &[u8]) -> Option<u32> {
    u32::try_from(natural(word)?).ok()
}
fn setting_number(kind: &[u8], word: &[u8]) -> Address {
    let Some(number) = small(word) else { return Address::Unknown };
    let setting = match kind {
        b"charter" => Setting::Charter(number),
        b"template" => Setting::Template(number),
        b"policy" => Setting::Policy(number),
        b"model" => Setting::Model(number),
        b"charter-words" => Setting::Application { kind: 9, scope: 0, number },
        b"forge-charter" => Setting::Application { kind: 3, scope: 0, number },
        b"template-goal" => Setting::Application { kind: 11, scope: 0, number },
        b"landing-rule" => Setting::Application { kind: 1, scope: 0, number },
        _ => unreachable!("setting grammar names its kinds"),
    };
    Address::Setting(setting)
}
fn page_name(name: &[u8]) -> bool {
    if name.is_empty() || name.len() > 32 {
        return false;
    }
    for byte in name {
        if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && *byte != b'-' {
            return false;
        }
    }
    true
}
fn pair(word: &[u8], separator: u8) -> Option<(&[u8], &[u8])> {
    let at = skein_lib::bytes::find(word, &[separator])?;
    let after = at.checked_add(1)?;
    let left = word.get(..at)?;
    let right = word.get(after..)?;
    if skein_lib::bytes::find(right, &[separator]).is_some() {
        return None;
    }
    Some((left, right))
}
fn repository_name(name: &[u8]) -> bool {
    match pair(name, b'/') {
        Some((owner, name)) => !owner.is_empty() && !name.is_empty(),
        None => false,
    }
}
fn turn_number(word: &[u8]) -> Option<(u64, u32)> {
    let (attempt, turn) = pair(word, b'.')?;
    Some((natural(attempt)?, small(turn)?))
}
fn proposal_name(name: &[u8]) -> Address {
    let Some((prefix, tail)) = name.split_first() else { return Address::Unknown };
    if let Some((who, number)) = pair(tail, b'/')
        && let (Some(who), Some(number)) = (natural(who), natural(number))
    {
        let proposer = match prefix {
            b't' => jig_domain_people::Proposer::Task(who),
            b'p' => jig_domain_people::Proposer::Party(who),
            _ => return Address::Unknown,
        };
        return Address::Proposal { proposer, number };
    }
    Address::Unknown
}

pub(crate) fn call(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    to: ReplyTo,
    key: crate::CallKey,
    line: Box<[u8]>,
    from: u32,
) {
    let context = domain.core.tasks.delegation(key.task).expect("authorized chat delegation");
    let project = context.project;
    let read = match address(&line, project) {
        Address::Ways => {
            let party = match context.requester {
                jig_domain_tasks::Party::Person(party) => party,
                jig_domain_tasks::Party::Task(_) | jig_domain_tasks::Party::Deployment { .. } => {
                    deliver(env, decision, to, line, from, SeeAnswer::Unknown);
                    return;
                }
            };
            let document = super::ways::document(domain, project, party);
            deliver(env, decision, to, line, from, SeeAnswer::Ways(Box::new(document)));
            return;
        }
        Address::Inventory => core::SettingsRead::Inventory,
        Address::Setting(setting) => core::SettingsRead::Setting { setting, revision: 0 },
        Address::Revisions => core::SettingsRead::Revisions { before: None },
        Address::Revision { number } => core::SettingsRead::Revision { number },
        Address::Landing => {
            forge::landing(domain, env, decision, to, key, line, from);
            return;
        }
        Address::Gates { repository } => {
            forge::gates(domain, env, decision, to, key, line, from, repository);
            return;
        }
        Address::File { repository, path, branch } => {
            forge::file(domain, env, decision, to, key, line, from, repository, path, branch);
            return;
        }
        Address::Proposal { proposer, number } => {
            proposal::read(domain, env, decision, to, key, line, from, proposer, number);
            return;
        }
        Address::Page { name } => {
            deliver(env, decision, to, line, from, SeeAnswer::Page { name });
            return;
        }
        Address::Search { words } => {
            deliver(env, decision, to, line, from, SeeAnswer::Search { words });
            return;
        }
        address @ (Address::Me
        | Address::Project
        | Address::Goals { .. }
        | Address::Tasks
        | Address::Task { .. }
        | Address::Tree { .. }
        | Address::Activity { .. }
        | Address::Turn { .. }
        | Address::Change { .. }
        | Address::Changes
        | Address::Inbox) => {
            person::read(domain, env, decision, to, key, line, from, address);
            return;
        }
        Address::Unknown => {
            deliver(env, decision, to, line, from, SeeAnswer::Unknown);
            return;
        }
    };
    let requested = match &read {
        core::SettingsRead::Revision { number } => Some(u64::from(*number)),
        core::SettingsRead::Inventory | core::SettingsRead::Setting { .. } | core::SettingsRead::Revisions { .. } => {
            None
        }
    };
    let asked = match &read {
        core::SettingsRead::Setting { setting, revision } => Some((*setting, *revision)),
        core::SettingsRead::Inventory | core::SettingsRead::Revision { .. } | core::SettingsRead::Revisions { .. } => {
            None
        }
    };
    let right = to.into_token();
    domain.call_renders.get_mut(&right).expect("decoded see flight").render = FlightRender::Settings {
        see: Some(Box::new(SeeFlight { line, reading: SeeReading::Settings })),
        from,
        requested,
        asked,
    };
    let mut read = read;
    if let core::SettingsRead::Setting { revision, .. } = &mut read {
        *revision = domain.core.current.newest();
    }
    domain.work.push(Work::Core(core::Event::NamedAction {
        to: ReplyTo::new(right),
        key,
        action: core::NamedAction::Settings(read),
    }));
}

pub(super) fn deliver(
    env: &Env<Limits>,
    decision: &mut Decision,
    to: ReplyTo,
    line: Box<[u8]>,
    from: u32,
    answer: SeeAnswer,
) {
    emit(
        decision,
        &env.limits,
        Delivery::Host(Box::new(super::HostDelivery::See { to, line, from, answer: Box::new(answer) })),
    );
}
