//! The forge connector top's vocabulary: what crosses its edge
//! (temper/docs/design/domain/forge/README.md, section 3).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::must_use_candidate, reason = "the vocabulary exposes the forge top's existing value operations")]
#![expect(clippy::disallowed_macros, reason = "kind 11 requires Default: both flags false with none")]
extern crate alloc;
use alloc::boxed::Box;

pub mod address;
pub mod approvals;
pub mod away;
pub mod boundary;
pub mod brief;
pub mod brief_words;
pub mod closer;
pub mod commit;
pub mod devices;
pub mod dismissed;
pub mod domain;
pub mod drafts;
pub mod effect_words;
pub mod findings;
pub mod follow_check;
pub mod follows;
pub mod forge_user;
pub mod gates;
pub mod held;
pub mod items;
pub mod judge;
pub mod landing;
pub mod landing_log;
pub mod limits;
pub mod make;
pub mod making;
pub mod panel;
pub mod preparation;
pub mod public;
pub mod purpose;
pub mod reconcile;
pub mod red;
pub mod reds;
pub mod refusal;
pub mod resources;
pub mod restore;
pub mod result;
pub mod settings;
pub mod status;
#[cfg(test)]
mod tests;
pub mod text;
pub mod tiers;
pub mod verification;
pub mod views;
pub mod workflows;

pub use address::{PULL_ADDRESS_BYTES, host, pull_address};
pub use approvals::{ApprovalWait, ShortApproval};
pub use away::{
    AWAY_SUBJECT, AskedView, Away, AwaysView, DECIDED_DAY_NS, DECIDED_KEPT_NS, DECIDED_OPTION_BYTES, DECIDED_OPTIONS,
    DECIDED_QUESTION_BYTES, DECIDED_WHY_BYTES, DECISIONS_SHOWN, Decided, DecidedCall, DecisionView, DecisionsView,
    OverturnAnswer, SetAwayAnswer, decided_subject,
};
pub use brief_words::{BriefFate, BriefUnread};
pub use closer::CloserRow;
pub use devices::{
    AddDevice, DEVICES, DEVICES_SUBJECT, Device, DeviceAnswer, DeviceRefusal, DeviceView, DevicesView, Push, PushAnswer,
};
pub use dismissed::{DISMISSED_SUBJECT, DismissAnswer, PersonFact};
pub use drafts::{
    DRAFTS, DraftChange, DraftRefusal, DraftRepository, DraftView, DraftsView, LandedDraft, SetDraft, SetDraftAnswer,
    draft_path, drafts_subject,
};
pub use findings::{
    AreaRow, FileFinding, FileFindingAnswer, Filer, Finding, FindingAnswer, FindingDone, FindingKind, FindingRefusal,
    FindingReply, FindingRow, FindingState, FindingWrite, Findings, HandFinding, HandFindingAnswer, Holder, Severity,
    Triage, TriageFinding, TriageFindingAnswer, findings_subject, owner,
};
pub use follows::{
    FollowAct, FollowCommit, FollowEntry, FollowLag, FollowParity, FollowQuestion, FollowRefusal, FollowRow,
    FollowSkip, FollowState, FollowVerdict,
};
pub use forge_user::{FORGE_PROVIDER, key_user, user_key};
pub use landing_log::LOGGED_TITLE_BYTES;
pub use make::{BOT_EMAIL, BOT_LOGIN};
pub use making::{
    MakeFailure, MakeFrom, MakeInto, MakeRefusal, MakeStep, MakingFailed, MakingKey, MakingState, RepositoryMaking,
};
pub use public::{PublicAsk, PublicMade, PublicTask};
pub use status::{
    BRIEF_LEDE_BYTES, BRIEF_SENTENCE_BYTES, Brief, Looked, LookedAnswer, ProjectFacts, SITTING, StatusRunning,
    StatusView, status_paragraphs, status_subject,
};
pub use workflows::{WORKFLOW_BYTES, WORKFLOWS};

pub use boundary::*;
pub use brief::{HOME_SECTION_BYTES, hex, home_access_reason};
pub use commit::commit_text;
pub use domain::{DEFAULT_CHANGE_TITLE, tag, untag};
pub use gates::{GateCandidate, GateLast, GatePattern, Requires, Touched, covers, covers_branch, touched};
pub use judge::{Criterion, Freshness as JudgeFreshness, Reviewer, Verdict as JudgeVerdict};
pub use limits::{
    BOT_LOGIN_SUFFIX, CONNECT_WHY_BYTES, FIRST_NAME_BYTES, Limits, STOPPED_TEXT_BYTES, TierLimits, valid,
};
pub use panel::{
    MARKED, PANEL_AHEAD, PANEL_GATES, PANEL_NAME_BYTES, Panel, PanelApproval, PanelGate, PanelQuestion, PanelQuestions,
    PanelStands, PanelStep, PanelUsed, ReviewFacts, ReviewRead, UnmappedReview,
};
pub use reconcile::{MissingLink, Need, RecordKind};
pub use red::{JobLog, RedNews, RedOn, red_news_words};
pub use reds::{RecordRed, Recorder, Red, RedOutcome, RedRecorded, RedRefusal, RedReply};
pub use refusal::{RefusalCause, bound_words};
pub use restore::{RestoreFit, RestoreSet};
pub use result::{ChangeResult, ResultText};
pub use settings::ForgeCharters;
pub use temper_engine_domain_forge_landing::CHECK_NAME_BYTES;
pub use temper_engine_domain_forge_landing::{
    Combine as LandingCombine, Hold as LandingHold, Leave as LandingLeave, Phase as LandingPhase,
    Round as LandingRound, Search as LandingSearch,
};
pub use verification::LandingRed;
pub use views::{
    APPROVALS_SUBJECT, ChangesLanded, ChangesQueue, ChangesQueued, ChangesView, DependencyView, Document,
    FollowLagEntry, FollowView, FollowsView, GateProof, LandingGate, LandingLine, LandingMember, LandingStep,
    LandingTrain, LandingView, LandingsView, PrefixJudgement, TrainLanding, changes_subject, follows_subject,
    landings_subject,
};

/// The prepare job (temper/docs/design/domain/worker/jobs.md).
pub const PREPARE_KIND: u32 = 1;

/// The merge job (temper/docs/design/domain/worker/jobs.md).
pub const MERGE_KIND: u32 = 3;
/// The close job (temper/docs/design/domain/worker/jobs.md).
pub const CLOSE_KIND: u32 = 4;
/// Connector configuration (temper/docs/design/protocol/configuration/engine.md, Config).
#[derive(Clone, Debug)]
pub struct Config {
    pub namespace: Box<[u8]>,
    pub writers: Box<[skein_forge_client::Writer]>,
    /// Allowed push service origins, none when push is off.
    pub push: Option<Box<[Box<[u8]>]>>,
}
/// Connector-owned access to one exact effect on an adopted resource.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Access {
    Owned,
    Participant,
    Context,
    Unavailable,
}

/// What a forge write does to its named target.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EffectForm {
    Creation,
    Transition,
    Set,
}

/// Recovery promise of one forge effect kind.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Recovery {
    Keyed,
    Conditional,
    Idempotent,
    Unrecoverable,
}

/// Effect form and recovery class declared by the forge connector.
///
/// The kinds follow domain/forge/README.md, section 4, one kind per row:
/// 3 opens pulls, 10 edits their title/body, 11 updates from the base,
/// 4 merges, 12 closes/reopens, 5 posts reviews, 7 posts comments,
/// 6 sets statuses, 8 creates issues, 13 edits/closes/reopens issues,
/// 14 sets reviewers, 9 creates branches, and 15 deletes branches.
/// Kind 1 reads and kind 2 grants workspace pushes. The order has no
/// implications between distinct kinds: each intended write needs a grant.
#[must_use]
pub const fn effect_shape(kind: u16) -> Option<(EffectForm, Recovery)> {
    match kind {
        2 | 17 => Some((EffectForm::Transition, Recovery::Unrecoverable)),
        11 => Some((EffectForm::Transition, Recovery::Idempotent)),
        4 => Some((EffectForm::Transition, Recovery::Conditional)),
        3 => Some((EffectForm::Creation, Recovery::Conditional)),
        5 | 7 | 8 | 9 | 16 => Some((EffectForm::Creation, Recovery::Unrecoverable)),
        6 | 10 | 12..=15 => Some((EffectForm::Set, Recovery::Idempotent)),
        0 | 1 | 18..=u16::MAX => None,
    }
}

/// A kept change's landing waits for dependencies (domain/forge/change/README.md, 2 item 7).
#[must_use]
pub const fn waits_for_dependencies(kind: u16) -> bool {
    kind == 4
}

/// Connector-owned request and the facts jig may inspect before admitting it.
#[derive(Debug)]
pub struct AgentEffect {
    pub effect: skein_forge_client::Effect,
    pub kind: u16,
    pub state: [u8; 32],
    pub form: EffectForm,
    pub recovery: Recovery,
}

/// Classify and bound an agent write in the connector's own vocabulary.
pub fn describe_agent_effect(
    repository: &Repository,
    resource: &What,
    write: skein_forge::api::Write,
    key: Box<[u8]>,
    op_bytes: u32,
) -> Result<AgentEffect, skein_forge::api::Error> {
    let (write, kind, state, form, valid_resource) = match write {
        skein_forge::api::Write::CreateIssue { title, body, .. } => (
            skein_forge::api::Write::CreateIssue { key, title, body },
            8,
            [0; 32],
            EffectForm::Creation,
            *resource == What::Repository,
        ),
        skein_forge::api::Write::Post { number, body, .. } => (
            skein_forge::api::Write::Post { number, key, body },
            7,
            [0; 32],
            EffectForm::Creation,
            *resource == What::Issue(number) || *resource == What::Pull(number),
        ),
        skein_forge::api::Write::Status { commit, context, check, description, link } => (
            skein_forge::api::Write::Status { commit, context, check, description, link },
            6,
            commit,
            EffectForm::Set,
            *resource == What::Repository,
        ),
        skein_forge::api::Write::OpenPull { .. }
        | skein_forge::api::Write::Review { .. }
        | skein_forge::api::Write::Edit { .. }
        | skein_forge::api::Write::SetReviewers { .. }
        | skein_forge::api::Write::Close { .. }
        | skein_forge::api::Write::Reopen { .. }
        | skein_forge::api::Write::Merge { .. }
        | skein_forge::api::Write::Update { .. }
        | skein_forge::api::Write::CreateBranch { .. }
        | skein_forge::api::Write::Dispatch { .. }
        | skein_forge::api::Write::Rerun { .. }
        | skein_forge::api::Write::DeleteBranch { .. } => return Err(skein_forge::api::Error::Forbidden),
    };
    let access = effect_access(repository, resource, kind);
    let available = match access {
        Access::Owned | Access::Participant => true,
        Access::Context | Access::Unavailable => false,
    };
    if !valid_resource || !available {
        return Err(skein_forge::api::Error::Forbidden);
    }
    let effect = skein_forge_client::Effect { write, condition: skein_forge_client::Condition::None };
    let recovery = match skein_forge_client::recovery(&effect.write) {
        skein_forge_client::Recovery::Keyed => Recovery::Keyed,
        skein_forge_client::Recovery::Conditional => Recovery::Conditional,
        skein_forge_client::Recovery::Idempotent => Recovery::Idempotent,
        skein_forge_client::Recovery::Unrecoverable => Recovery::Unrecoverable,
    };
    match skein_forge_client::measure::effect(&effect) {
        Some(bytes) if bytes <= u64::from(op_bytes) => {}
        Some(_) | None => return Err(skein_forge::api::Error::Forbidden),
    }
    Ok(AgentEffect { effect, kind, state, form, recovery })
}

/// Describe current access from the forge's adopted role and effect kinds.
#[must_use]
pub fn effect_access(repository: &Repository, what: &What, kind: u16) -> Access {
    let permitted = match kind {
        1 => repository.kinds.read,
        2 => repository.kinds.push,
        3 | 10..=12 => repository.kinds.open,
        4 => repository.kinds.land,
        5 | 14 => repository.kinds.review,
        6 => repository.kinds.status,
        7 => repository.kinds.comment,
        8 | 13 => repository.kinds.issue,
        9 | 15..=17 => repository.kinds.branch,
        _ => false,
    };
    let shared = match what {
        What::Issue(_) | What::Pull(_) => true,
        What::Repository | What::Branch(_) => false,
    };
    match repository.role {
        Role::Context => Access::Context,
        Role::Owned | Role::Adopted | Role::Fork if !permitted => Access::Unavailable,
        Role::Owned | Role::Adopted | Role::Fork if shared => Access::Participant,
        Role::Adopted => Access::Participant,
        Role::Fork if kind == 4 => Access::Participant,
        Role::Owned | Role::Fork => Access::Owned,
    }
}
/// Stable key of one durable connector row.
#[must_use]
pub fn stored_key(row: &Stored) -> Key {
    match row {
        Stored::Forge(row) => Key::Forge(row.number),
        Stored::ForgeNumber { .. } => Key::ForgeNumber,
        Stored::Red(row) => Key::Red { goal: row.goal, key: row.key.clone() },
        Stored::RedNumber { .. } => Key::RedNumber,
        Stored::Playground(_) => Key::Playground,
        Stored::Follow(row) => Key::Follow { project: row.project, follow: row.follow },
        Stored::Tier(row) => Key::Tier { repository: row.repository, tier: row.tier.clone() },
        Stored::Verification(row) => Key::Verification { repository: row.repository, commit: row.commit },
        Stored::LandingLog(row) => Key::LandingLog { repository: row.repository },
        Stored::ProjectFacts(row) => Key::ProjectFacts { project: row.project },
        Stored::Device(row) => Key::Device { party: row.party, number: row.number },
        Stored::LandedDraft(row) => Key::LandedDraft { repository: row.repository, path: row.path.clone() },
        Stored::Away(row) => Key::Away { party: row.party, project: row.project },
        Stored::Decided(row) => Key::Decided { project: row.project, number: row.number },
        Stored::Finding(row) => Key::Finding { project: row.project, number: row.number },
        Stored::Looked(row) => Key::Looked { party: row.party, project: row.project },
        Stored::Landing(row) => Key::Landing(row.landing.task),
        Stored::Receipt { key, .. } => Key::Receipt(key.clone()),
        Stored::Call { key, .. } => Key::Call(*key),
        Stored::Repository(row) => Key::Repository(row.provider),
        Stored::Hold(row) => Key::Hold(row.name.clone()),
        Stored::Names { task, .. } => Key::Names(*task),
        Stored::Subscription(row) => Key::Subscription { task: row.task, topic: row.topic.clone() },
        Stored::BranchHead(row) => Key::BranchHead(row.name.clone()),
        Stored::PullState(row) => Key::PullState(row.name.clone()),
        Stored::Ci(row) => Key::Ci { repository: row.repository, head: row.head },
        Stored::Landed { commit, .. } => Key::Landed(*commit),
        Stored::ProposedEffect(row) => Key::ProposedEffect(row.number),
        Stored::Entry { entry, .. } => Key::Entry(entry.number),
        Stored::Client(row) => Key::Client(match row {
            skein_forge_client::Stored::Live(row) => skein_forge_client::Key::Live(row.watch.resource.clone()),
            skein_forge_client::Stored::Repository(row) => skein_forge_client::Key::Repository(row.repository),
            skein_forge_client::Stored::Writer(_) => {
                unreachable!("the connector keeps the client's writer as Stored::Writer")
            }
        }),
        Stored::Closer(row) => Key::Closer(row.task),
        Stored::Change(row) => Key::Change(row.task),
        Stored::Issue(row) => Key::Issue(row.goal),
        Stored::Release(row) => Key::Release(row.task),
        Stored::Application { token, .. } => Key::Application(*token),
        Stored::Writer(row) => Key::Writer(row.forge),
        Stored::RefusedSetting(key) => Key::Setting(*key),
        Stored::Setting(row) => Key::Setting(row.key),
        Stored::Binding { address, .. } => Key::Binding(address.clone()),
        Stored::Making(row) => Key::Making(row.key.clone()),
        Stored::Dismissed { party, what } => Key::Dismissed { party: *party, what: *what },
        Stored::Flight { task, entry } => Key::Flight { task: *task, entry: *entry },
    }
}
/// Checked payload bytes of one connector record for its parent's store page.
#[must_use]
#[expect(clippy::too_many_lines, reason = "one exhaustive connector record payload sum")]
pub fn stored_bytes(record: &Stored) -> Option<u64> {
    use alloc::boxed::Box;
    use core::mem::size_of;
    use temper_engine_domain_forge_change as change;
    fn bytes(value: &[u8]) -> u64 {
        skein_lib::len_u64(value.len())
    }
    fn remark_bytes(remarks: &[GateRemark]) -> Option<u64> {
        let mut total = 0_u64;
        for remark in remarks {
            total = total.checked_add(bytes(&remark.words))?;
        }
        Some(total)
    }
    fn name(value: &Name) -> Option<u64> {
        match &value.what {
            What::Branch(parts) => {
                let mut held =
                    skein_lib::len_u64(parts.len()).checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?;
                for part in parts {
                    held = held.checked_add(bytes(part))?;
                }
                Some(held)
            }
            What::Repository | What::Pull(_) | What::Issue(_) => Some(0),
        }
    }
    fn topic(value: &Topic) -> u64 {
        match value {
            Topic::Landings { branch, .. } => bytes(branch),
            Topic::Ci { .. } | Topic::Pull { .. } | Topic::Participation { .. } | Topic::Findings { .. } => 0,
        }
    }
    match record {
        Stored::Making(row) => {
            let mut held = bytes(&row.key.org).checked_add(bytes(&row.key.name))?;
            match &row.from {
                MakeFrom::Empty => {}
                MakeFrom::Copy { address } => {
                    held = held.checked_add(bytes(address))?;
                }
            }
            if let Some(into) = &row.into {
                held = held.checked_add(bytes(&into.host))?;
            }
            match &row.state {
                MakingState::Waiting | MakingState::Making => {}
                MakingState::Made { branch, .. } | MakingState::Adopting { branch, .. } => {
                    held = held.checked_add(bytes(branch))?;
                }
                MakingState::Failed(failed) => {
                    held = held.checked_add(bytes(&failed.words))?;
                }
            }
            Some(held)
        }
        Stored::Forge(row) => {
            let mut held = bytes(&row.place.api.host)
                .checked_add(bytes(&row.place.git))?
                .checked_add(bytes(&row.bot.login))?
                .checked_add(bytes(&row.bot.name))?
                .checked_add(bytes(&row.bot.email))?;
            if let Some(program) = &row.git_program {
                held = held.checked_add(bytes(program))?;
            }
            if let Some(tls) = &row.place.api.tls {
                held = held.checked_add(bytes(tls))?;
            }
            if let Some(web) = &row.place.web {
                held = held.checked_add(bytes(web))?;
            }
            match &row.first {
                Some(FirstIntent::Empty(first)) => {
                    held = held
                        .checked_add(bytes(&first.org))?
                        .checked_add(bytes(&first.repository))?
                        .checked_add(bytes(&first.label))?;
                }
                Some(FirstIntent::Copies { addresses }) => {
                    held = held.checked_add(
                        skein_lib::len_u64(addresses.len()).checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?,
                    )?;
                    for address in addresses {
                        held = held.checked_add(bytes(address))?;
                    }
                }
                Some(FirstIntent::Nothing) | None => {}
            }
            Some(held)
        }
        Stored::Setting(row) => match &row.edit {
            settings::Edit::Set(settings::SizedValue { value: settings::Value::Rule(rule), .. }) => {
                let mut held = skein_lib::len_u64(rule.pattern.segments.len())
                    .checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?;
                for segment in &rule.pattern.segments {
                    held = held.checked_add(bytes(segment))?;
                }
                let last = match &rule.pattern.last {
                    GateLast::Exact(last) | GateLast::Open(last) => last,
                };
                held.checked_add(bytes(last))?
                    .checked_add(
                        skein_lib::len_u64(rule.gates.len())
                            .checked_mul(u64::try_from(size_of::<settings::RuleGate>()).ok()?)?,
                    )?
                    .checked_add(
                        skein_lib::len_u64(rule.approvals.len())
                            .checked_mul(u64::try_from(size_of::<settings::RuleApproval>()).ok()?)?,
                    )?
                    .checked_add(bytes(&rule.sets))
            }
            settings::Edit::Set(settings::SizedValue { value: settings::Value::Classes(classes), .. }) => {
                let mut held = skein_lib::len_u64(classes.items.len())
                    .checked_mul(u64::try_from(size_of::<temper_protocol_reaches::Item>()).ok()?)?
                    .checked_add(
                        skein_lib::len_u64(classes.sets.len())
                            .checked_mul(u64::try_from(size_of::<temper_protocol_reaches::PathSet>()).ok()?)?,
                    )?;
                for item in &classes.items {
                    held = held.checked_add(bytes(&item.words))?;
                }
                for set in &classes.sets {
                    held = held.checked_add(bytes(&set.name))?.checked_add(
                        skein_lib::len_u64(set.patterns.len())
                            .checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?,
                    )?;
                    for pattern in &set.patterns {
                        held = held.checked_add(bytes(pattern))?;
                    }
                }
                Some(held)
            }
            settings::Edit::Set(settings::SizedValue { value: settings::Value::ProjectAreas(value), .. }) => {
                let mut held = skein_lib::len_u64(value.areas.len())
                    .checked_mul(u64::try_from(size_of::<settings::Area>()).ok()?)?;
                for area in &value.areas {
                    held = held.checked_add(bytes(&area.name))?.checked_add(bytes(&area.words))?;
                }
                Some(held)
            }
            settings::Edit::Set(settings::SizedValue { value: settings::Value::Follow(follow), .. }) => {
                let mut held = bytes(&follow.source.branch)
                    .checked_add(bytes(&follow.target.branch))?
                    .checked_add(bytes(&follow.check.workflow))?
                    .checked_add(bytes(&follow.check.artifact))?
                    .checked_add(
                        skein_lib::len_u64(follow.source.paths.len())
                            .checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?,
                    )?;
                for path in &follow.source.paths {
                    held = held.checked_add(bytes(path))?;
                }
                if let Some(producer) = &follow.producer {
                    held = held.checked_add(bytes(producer))?;
                }
                Some(held)
            }
            settings::Edit::Set(settings::SizedValue { value: settings::Value::CharterWords(line), .. }) => {
                Some(bytes(line))
            }
            settings::Edit::Set(settings::SizedValue { value: settings::Value::ProjectWords(words), .. }) => {
                bytes(&words.headline).checked_add(bytes(&words.lede))
            }
            settings::Edit::Set(settings::SizedValue {
                value:
                    settings::Value::Dependency(_)
                    | settings::Value::Charter(_)
                    | settings::Value::Questions(_)
                    | settings::Value::Ways(_)
                    | settings::Value::ProjectTriager(_)
                    | settings::Value::Goal(_),
                ..
            })
            | settings::Edit::Remove
            | settings::Edit::Reset => Some(0),
        },
        Stored::Follow(row) => follows::row_bytes(row),
        Stored::Tier(row) => tiers::row_bytes(row),
        Stored::Verification(row) => verification::row_bytes(row),
        Stored::LandingLog(row) => landing_log::row_bytes(row),
        Stored::Red(row) => skein_lib::len_u64(row.key.len())
            .checked_add(skein_lib::len_u64(row.head.len()))?
            .checked_add(skein_lib::len_u64(row.why.len())),
        Stored::Finding(row) => {
            let mut bytes = skein_lib::len_u64(row.title.len()).checked_add(skein_lib::len_u64(row.body.len()))?;
            for field in [&row.area, &row.why, &row.reason].into_iter().flatten() {
                bytes = bytes.checked_add(skein_lib::len_u64(field.len()))?;
            }
            Some(bytes)
        }
        Stored::Decided(row) => {
            let mut bytes = skein_lib::len_u64(row.question.len()).checked_add(skein_lib::len_u64(row.why.len()))?;
            for option in &row.options {
                bytes = bytes.checked_add(skein_lib::len_u64(option.len()))?;
            }
            Some(bytes)
        }
        Stored::ProjectFacts(row) => Some(match &row.brief {
            Some(brief) => {
                skein_lib::len_u64(brief.sentence.len()).checked_add(skein_lib::len_u64(brief.lede.len()))?
            }
            None => 0,
        }),
        Stored::Landing(row) => landing::row_bytes(row),
        Stored::Receipt { key, receipt } => landing::receipt_bytes(key, receipt),
        Stored::Call { answer, .. } => match answer {
            CallPayload::Follow { refusal, .. } => Some(match refusal {
                Some(words) => skein_lib::len_u64(words.len()),
                None => 0,
            }),
            CallPayload::Red(reply) => {
                u64::try_from(size_of_val(reply.as_ref())).ok()?.checked_add(reds::reply_bytes(reply)?)
            }
            CallPayload::Finding(reply) => {
                u64::try_from(size_of_val(reply.as_ref())).ok()?.checked_add(findings::reply_bytes(reply)?)
            }
            CallPayload::Decided { took, .. } => Some(match took {
                Some(words) => skein_lib::len_u64(words.len()),
                None => 0,
            }),
            CallPayload::Effect { .. } | CallPayload::Refused(_) => Some(0),
            CallPayload::Read(answer) => match answer.as_ref() {
                Ok(answer) => skein_forge::measure::answer(answer),
                Err(_) => Some(0),
            },
        },
        Stored::Repository(row) => {
            let mut held = skein_lib::len_u64(row.contexts.len())
                .checked_mul(u64::try_from(size_of::<u32>()).ok()?)?
                .checked_add(bytes(&row.host))?
                .checked_add(bytes(&row.owner))?
                .checked_add(bytes(&row.name))?
                .checked_add(bytes(&row.prefix))?
                .checked_add(bytes(&row.settings.default_branch))?;
            held = held.checked_add(
                skein_lib::len_u64(row.workflows.len()).checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?,
            )?;
            for workflow in &row.workflows {
                held = held.checked_add(bytes(workflow))?;
            }
            match &row.protection {
                Protection::Rule(rule) => {
                    held = held.checked_add(bytes(&rule.branch))?.checked_add(
                        skein_lib::len_u64(rule.contexts.len())
                            .checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?,
                    )?;
                    for context in &rule.contexts {
                        held = held.checked_add(bytes(context))?;
                    }
                }
                Protection::Unknown | Protection::Absent => {}
            }
            Some(held)
        }
        Stored::Application { outcome, .. } => match outcome {
            ApplicationOutcome::Follow(words) => Some(match words {
                Some(words) => bytes(words),
                None => 0,
            }),
            ApplicationOutcome::CoreRefused(_)
            | ApplicationOutcome::ForgeRefused(_)
            | ApplicationOutcome::MakeKept
            | ApplicationOutcome::MakeRefused(_) => Some(0),
            ApplicationOutcome::Adopted(row) => {
                let mut held = bytes(&row.owner).checked_add(bytes(&row.name))?;
                match &row.protection {
                    Protection::Unknown | Protection::Absent => {}
                    Protection::Rule(rule) => {
                        held = held.checked_add(bytes(&rule.branch))?.checked_add(
                            skein_lib::len_u64(rule.contexts.len())
                                .checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?,
                        )?;
                        for context in &rule.contexts {
                            held = held.checked_add(bytes(context))?;
                        }
                    }
                }
                Some(held)
            }
        },
        Stored::Hold(row) => {
            let mut bytes = name(&row.name)?;
            if let Some(drift) = &row.drift {
                match &drift.change {
                    DriftChange::Retargeted { before, after, .. } => {
                        bytes = bytes
                            .checked_add(skein_lib::len_u64(before.len()))?
                            .checked_add(skein_lib::len_u64(after.len()))?;
                    }
                    DriftChange::Branch { .. } | DriftChange::PullClosed { .. } => {}
                }
            }
            Some(bytes)
        }
        Stored::Names { resources, .. } => {
            let mut held = skein_lib::len_u64(resources.len()).checked_mul(u64::try_from(size_of::<Name>()).ok()?)?;
            for resource in resources {
                held = held.checked_add(name(resource)?)?;
            }
            Some(held)
        }
        Stored::Subscription(row) => {
            let tasks = match &row.goal_tasks {
                Some(tasks) => skein_lib::len_u64(tasks.len()).checked_mul(8)?,
                None => 0,
            };
            let mut held = topic(&row.topic).checked_add(tasks)?.checked_add(
                skein_lib::len_u64(row.paths.len()).checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?,
            )?;
            for path in &row.paths {
                held = held.checked_add(bytes(path))?;
            }
            Some(held)
        }
        Stored::Device(row) => bytes(&row.endpoint).checked_add(bytes(&row.label)),
        Stored::Binding { address, subject } => bytes(address).checked_add(bytes(subject)),
        Stored::BranchHead(row) => name(&row.name),
        Stored::PullState(row) => name(&row.name),
        Stored::LandedDraft(row) => bytes(&row.path).checked_add(bytes(&row.branch))?.checked_add(bytes(&row.title)),
        Stored::RefusedSetting(_)
        | Stored::RedNumber { .. }
        | Stored::Away(_)
        | Stored::Looked(_)
        | Stored::Dismissed { .. }
        | Stored::ForgeNumber { .. }
        | Stored::Playground(_)
        | Stored::Ci(_)
        | Stored::Landed { .. }
        | Stored::Writer(_)
        | Stored::Flight { .. } => Some(0),
        Stored::ProposedEffect(row) => {
            let resource = match &row.resource {
                What::Branch(branch) => {
                    let mut total = 0_u64;
                    for segment in branch {
                        total = total.checked_add(bytes(segment))?;
                    }
                    total
                }
                What::Repository | What::Pull(_) | What::Issue(_) => 0,
            };
            skein_forge_client::measure::effect(&row.entry.effect)?.checked_add(resource)
        }
        Stored::Entry { entry, .. } => skein_forge_client::measure::effect(&entry.effect),
        Stored::Client(row) => skein_forge_client::measure::stored(row),
        Stored::Closer(row) => {
            let closed = match &row.closer.closed {
                Some(tasks) => skein_lib::len_u64(tasks.len()),
                None => 0,
            };
            let mut total = closed
                .checked_add(skein_lib::len_u64(row.close_changes.len()))?
                .checked_add(skein_lib::len_u64(row.landing_changes.len()))?
                .checked_mul(8)?
                .checked_add(skein_lib::len_u64(row.closer.selections.len()).checked_mul(
                    u64::try_from(size_of::<temper_engine_domain_forge_closer::PullSelection>()).ok()?,
                )?)?;
            for selection in &row.closer.selections {
                let held = match &selection.progress {
                    temper_engine_domain_forge_closer::SelectionProgress::Jobs { attempts, .. } => {
                        skein_lib::len_u64(attempts.len())
                            .checked_mul(u64::try_from(size_of::<skein_forge::api::JobAttempt>()).ok()?)?
                    }
                    temper_engine_domain_forge_closer::SelectionProgress::Done { line: Some(line) } => bytes(line),
                    temper_engine_domain_forge_closer::SelectionProgress::Done { line: None }
                    | temper_engine_domain_forge_closer::SelectionProgress::Pull
                    | temper_engine_domain_forge_closer::SelectionProgress::Checks => 0,
                };
                total = total.checked_add(held)?;
            }
            total.checked_add(row.relay.held_bytes()?)
        }
        Stored::Change(row) => skein_lib::len_u64(row.signed.len())
            .checked_mul(u64::try_from(size_of::<SignOff>()).ok()?)?
            .checked_add(skein_lib::len_u64(row.repair_tasks.len()).checked_mul(8)?)?
            .checked_add(u64::try_from(size_of::<ChangeRow>()).ok()?)?
            .checked_add(skein_lib::len_u64(row.resolution_tasks.len()).checked_mul(8)?)?
            .checked_add(match &row.ending {
                Some(Ending::Failed { line }) => bytes(line),
                Some(Ending::Landed | Ending::NoChange | Ending::Cancelled) | None => 0,
            })?
            .checked_add(bytes(&row.branch))?
            .checked_add(bytes(&row.base))?
            .checked_add(bytes(&row.title))?
            .checked_add(bytes(&row.body))?
            .checked_add(bytes(&row.report))?
            .checked_add(match &row.follows {
                Some(follows) => skein_lib::len_u64(follows.commits.len()).checked_mul(32)?,
                None => 0,
            })?
            .checked_add(match &row.follow_check {
                Some(check) => follow_check::stored_bytes(check)?,
                None => 0,
            })?
            .checked_add(
                skein_lib::len_u64(row.change.gates.len())
                    .checked_mul(u64::try_from(size_of::<change::Gate>()).ok()?)?,
            )?
            .checked_add(skein_lib::len_u64(row.change.clean.len()).checked_mul(32)?)?
            .checked_add(
                skein_lib::len_u64(row.change.gate_failures.len())
                    .checked_mul(u64::try_from(size_of::<change::GateFailure>()).ok()?)?,
            )?
            .checked_add(
                skein_lib::len_u64(row.verdicts.len())
                    .checked_mul(u64::try_from(size_of::<change::GateReport>()).ok()?)?,
            )?
            .checked_add(
                skein_lib::len_u64(row.gate_remarks.len()).checked_mul(u64::try_from(size_of::<GateRemark>()).ok()?)?,
            )?
            .checked_add(remark_bytes(&row.gate_remarks)?)?
            .checked_add(bytes(&row.result.checks.bytes))?
            .checked_add(bytes(&row.result.failed.bytes))?
            .checked_add(bytes(&row.result.repairs.bytes))?
            .checked_add(bytes(&row.result.reviews.bytes))?
            .checked_add(match &row.effect_account {
                Some(account) => bytes(&account.words),
                None => 0,
            })?
            .checked_add(match &row.producer_failure {
                Some(words) => bytes(words),
                None => 0,
            })?
            .checked_add(change_state_bytes(&row.change.state, 0)?),
        Stored::Issue(row) => {
            let before = match &row.before {
                Some(before) => before.milestones.len(),
                None => 0,
            };
            let mut held = u64::try_from(row.state.milestones.len().checked_add(before)?)
                .ok()?
                .checked_mul(u64::try_from(size_of::<temper_engine_domain_forge_issues::MilestoneKey>()).ok()?)?;
            if let Some(view) = &row.desired {
                held =
                    held.checked_add(bytes(view.title.as_bytes()))?
                        .checked_add(bytes(view.goal_text.as_bytes()))?
                        .checked_add(skein_lib::len_u64(view.plan.len()).checked_mul(
                            u64::try_from(size_of::<temper_engine_domain_forge_issues::PlanItem>()).ok()?,
                        )?)?
                        .checked_add(skein_lib::len_u64(view.milestones.len()).checked_mul(
                            u64::try_from(size_of::<temper_engine_domain_forge_issues::Milestone>()).ok()?,
                        )?)?;
                for item in &view.plan {
                    held = held.checked_add(bytes(item.text.as_bytes()))?;
                }
                for milestone in &view.milestones {
                    held = held.checked_add(bytes(milestone.text.as_bytes()))?;
                }
                if let Some(finished) = &view.finished {
                    held = held.checked_add(bytes(finished.as_bytes()))?;
                }
            }
            Some(held)
        }
        Stored::Release(row) => {
            let comment = match &row.comment {
                Some(comment) => bytes(comment),
                None => 0,
            };
            let pending = match &row.pending {
                Some((_, Some(resource))) => name(resource)?,
                Some((_, None)) | None => 0,
            };
            comment.checked_add(pending)
        }
    }
}

/// Connector-owned description of one goal projection write.
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct ProjectionEffect {
    pub resource: What,
    pub kind: u16,
    pub form: EffectForm,
    pub recovery: Recovery,
    pub access: Access,
}

/// Describe the write selected by the issue projection mechanism.
#[must_use]
pub fn describe_projection_effect(
    repository: &Repository,
    write: &skein_forge::api::Write,
) -> Option<ProjectionEffect> {
    let (resource, kind, form) = match write {
        skein_forge::api::Write::CreateIssue { .. } => (What::Repository, 8, EffectForm::Creation),
        skein_forge::api::Write::Edit { number, .. } | skein_forge::api::Write::Close { number } => {
            (What::Issue(*number), 13, EffectForm::Set)
        }
        skein_forge::api::Write::Post { number, .. } => (What::Issue(*number), 7, EffectForm::Creation),
        skein_forge::api::Write::Status { .. }
        | skein_forge::api::Write::OpenPull { .. }
        | skein_forge::api::Write::Review { .. }
        | skein_forge::api::Write::SetReviewers { .. }
        | skein_forge::api::Write::Reopen { .. }
        | skein_forge::api::Write::Merge { .. }
        | skein_forge::api::Write::Update { .. }
        | skein_forge::api::Write::CreateBranch { .. }
        | skein_forge::api::Write::Dispatch { .. }
        | skein_forge::api::Write::Rerun { .. }
        | skein_forge::api::Write::DeleteBranch { .. } => return None,
    };
    let recovery = match skein_forge_client::recovery(write) {
        skein_forge_client::Recovery::Keyed => Recovery::Keyed,
        skein_forge_client::Recovery::Conditional => Recovery::Conditional,
        skein_forge_client::Recovery::Idempotent => Recovery::Idempotent,
        skein_forge_client::Recovery::Unrecoverable => Recovery::Unrecoverable,
    };
    Some(ProjectionEffect { access: effect_access(repository, &resource, kind), resource, kind, form, recovery })
}

/// A canonical member address, bounded as protocol/sign-in.md, 3.1 requires.
#[must_use]
pub fn binding_address(address: &[u8]) -> bool {
    if !(3..=254).contains(&address.len()) || address.first() == Some(&b'@') || address.last() == Some(&b'@') {
        return false;
    }
    let mut at = false;
    for byte in address {
        if !byte.is_ascii_graphic() || byte.is_ascii_uppercase() {
            return false;
        }
        if *byte == b'@' {
            if at {
                return false;
            }
            at = true;
        }
    }
    at
}

fn change_state_bytes(value: &temper_engine_domain_forge_change::State, depth: u32) -> Option<u64> {
    use core::mem::size_of;
    use temper_engine_domain_forge_change as change;
    if depth > 16 {
        return None;
    }
    match value {
        change::State::Judging { target, asked, .. } => skein_lib::len_u64(target.branch.len())
            .checked_add(skein_lib::len_u64(target.base.len()))?
            .checked_add(skein_lib::len_u64(asked.len()).checked_mul(8)?),
        change::State::NoChange { words } => Some(skein_lib::len_u64(words.len())),
        change::State::Gating { asked, .. } => {
            skein_lib::len_u64(asked.len()).checked_mul(u64::try_from(size_of::<u64>()).ok()?)
        }
        change::State::Held { was, .. } => {
            u64::try_from(size_of::<change::State>()).ok()?.checked_add(change_state_bytes(was, depth.checked_add(1)?)?)
        }
        change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
        | change::State::CancellingGates { .. }
        | change::State::Queued { .. }
        | change::State::First { .. }
        | change::State::Ready { .. }
        | change::State::Preparing { .. }
        | change::State::Updating { .. }
        | change::State::WaitingProducer { .. }
        | change::State::Resolving { .. }
        | change::State::Repairing { .. }
        | change::State::Landing { .. }
        | change::State::Ended
        | change::State::Landed { .. }
        | change::State::Red { .. }
        | change::State::Cleared { .. }
        | change::State::Uncleared { .. } => Some(0),
    }
}
