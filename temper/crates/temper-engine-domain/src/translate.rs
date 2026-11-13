//! Total vocabulary and configuration translations (jig's domain/root.md, 6).
use crate::boundary::{Call, EscalationChoice, MessageForm, ProposalChoice, ProposedAction, Tool};
use crate::domain::Domain;
use crate::limits::Limits;
use crate::route::{LandingRule, host_route};
use crate::{CallAnswer, Decision};
use alloc::boxed::Box;
use jig_domain_link::JobBound;
use jig_domain_notes as notes;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Map, ReplyTo};
use temper_engine_domain_forge as forge;

/// Startup configuration, supplied once by the shell (domain/root.md, 5):
/// the core's whole, as jig's applications hold it, and temper's own.
#[derive(Debug)]
pub struct Config {
    /// Charter brief-room costs, absent when no agent is configured (temper/docs/design/domain/agent/tools.md).
    pub host_part: Option<crate::room::HostPart>,
    /// Deployment funding span (domain/routing.md, 7).
    pub period_span: skein_lib::Duration,
    /// The core's rules, charters, owners and projects; its one connector is
    /// the forge's; whether the workers run in the engine's process is
    /// configuration too, set by an in-process hosts' listener: standalone,
    /// they do (domain/root.md, 2).
    pub core: jig_domain_core::Config,
    /// Each project's landing way and repository checks.
    pub landings: Map<u32, ProjectLanding>,
    /// The forge's kind defaults at this start (temper/docs/design/domain/routing.md, 6).
    pub kinds_stock: Box<[forge::settings::Stock]>,
    /// The stock ways, in their order (temper/docs/design/protocol/configuration/settings.md, 8).
    pub ways_stock: Box<[temper_engine_domain_ways::StockWay]>,
    /// Forge writer identities and this deployment's branch namespace.
    pub forge: temper_engine_domain_forge::Config,
    /// Each forge's git identity, an application credential (D10;
    /// protocol/credentials.md, "each repository to a git identity").
    pub identities: Box<[forge::GitIdentity]>,
    /// Machine places for new forge records (temper/docs/design/protocol/configuration/engine.md).
    pub machine: Option<forge::Machine>,
    pub forges: Box<[forge::Configured]>,
    pub under_box: bool,
    pub playground: Option<forge::PlaygroundConfig>,
}

/// What the root keeps of its configuration once its children hold theirs.
#[derive(Hash, Debug)]
pub(crate) struct RootConfig {
    pub(crate) ways_stock: Box<[temper_engine_domain_ways::StockWay]>,
    pub(crate) prepare_kind: Option<JobBound>,
    pub(crate) merge_kind: Option<JobBound>,
    pub(crate) close_kind: Option<JobBound>,
    pub(crate) forge_connector: u16,
    pub(crate) landings: Map<u32, ProjectLanding>,
}

/// Landing rules for test worlds' builders alone (domain/root.md, 5).
#[derive(Debug)]
pub struct LandingRules {
    pub deployment: Box<[LandingRule]>,
    pub projects: Map<u32, Box<[LandingRule]>>,
}

/// The goal closer's attempts (temper/docs/design/protocol/configuration/engine.md).
pub const LANDING_ATTEMPTS: u8 = 3;

/// A project's landing way and repository checks (protocol/configuration/engine.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProjectLanding {
    pub gate_retries: u8,
    pub gate_timeout: skein_lib::Duration,
    pub check_timeout: Option<skein_lib::Duration>,
    pub by: Way,
    pub combine_main: temper_engine_domain_forge_landing::Combine,
    pub combine_other: temper_engine_domain_forge_landing::Combine,
    pub merge_job: Option<Box<[u8]>>,
    pub close_job: Option<Box<[u8]>>,
    pub landing_attempts: u8,
    pub repositories: Box<[RepositoryChecks]>,
    pub after: AfterLandingCheck,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Way {
    Queue,
    Turn,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RepositoryChecks {
    pub repository: skein_forge::api::Repository,
    pub gates: Box<[GateCheck]>,
    pub tiers: Box<[TierCheck]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GateCheck {
    pub base: bool,
    pub name: Box<[u8]>,
    pub workflow: Box<[u8]>,
    pub ignored: Box<[Box<[u8]>]>,
    pub blocking: bool,
}
/// A repository's tier and the branch it follows (domain/after-landing.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TierCheck {
    pub name: Box<[u8]>,
    pub workflow: Box<[u8]>,
    pub ignored: Box<[Box<[u8]>]>,
    pub artifact: Box<[u8]>,
    pub format: temper_engine_domain_forge_tiers::Format,
    pub follows: forge::Follows,
    pub span_ms: u64,
    pub description: Box<[u8]>,
}
/// A project's verification settings (protocol/configuration/engine.md).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AfterLandingCheck {
    pub tier_retries: u8,
    pub verification_bound_ms: u64,
}
pub(crate) fn project_landing(forge: &mut forge::Domain, project: u32, landing: &ProjectLanding) -> bool {
    let mut repositories =
        skein_lib::List::with_capacity(u32::try_from(landing.repositories.len()).expect("repositories"));
    for repository in &landing.repositories {
        let mut gates = skein_lib::List::with_capacity(u32::try_from(repository.gates.len()).expect("gates"));
        for gate in &repository.gates {
            gates
                .push(temper_engine_domain_forge_landing::Gate {
                    name: gate.name.clone(),
                    workflow: gate.workflow.clone(),
                    ignored: gate.ignored.clone(),
                    blocking: gate.blocking,
                    base: gate.base,
                })
                .expect("counted gates");
        }
        let mut tiers = skein_lib::List::with_capacity(u32::try_from(repository.tiers.len()).expect("tiers"));
        for tier in &repository.tiers {
            tiers
                .push(forge::Tier {
                    name: tier.name.clone(),
                    workflow: tier.workflow.clone(),
                    ignored: tier.ignored.clone(),
                    artifact: tier.artifact.clone(),
                    format: tier.format,
                    follows: tier.follows.clone(),
                    span: match tier.span_ms {
                        0 => None,
                        millis => Some(skein_lib::Duration::from_millis(millis)),
                    },
                    description: tier.description.clone(),
                })
                .expect("counted tiers");
        }
        repositories
            .push(forge::RepositoryGates {
                repository: repository.repository,
                gates: gates.into_boxed(),
                tiers: tiers.into_boxed(),
            })
            .expect("counted repositories");
    }
    if !forge.project_landing(
        project,
        match landing.by {
            Way::Queue => temper_engine_domain_forge_change::Way::Queue,
            Way::Turn => temper_engine_domain_forge_change::Way::Turn,
        },
        repositories.into_boxed(),
        temper_engine_domain_forge_landing::Project {
            gate_retries: u32::from(landing.gate_retries),
            gate_timeout: landing.gate_timeout,
        },
        forge::AfterLanding {
            verdict: temper_engine_domain_forge_tiers::Settings {
                retries: u32::from(landing.after.tier_retries),
                suspects: temper_engine_domain_forge_tiers::Suspects::Bisect,
            },
            bound: skein_lib::Duration::from_millis(landing.after.verification_bound_ms),
        },
    ) {
        return false;
    }
    if let Some(command) = &landing.merge_job
        && !forge.project_merge_job(project, command.clone())
    {
        return false;
    }
    match &landing.close_job {
        Some(command) => forge.project_close_job(project, command.clone()),
        None => true,
    }
}

pub(crate) fn environment_forge(env: &Env<Limits>) -> Env<forge::Limits> {
    env.with(env.limits.forge)
}

pub(crate) fn checked_call(mut body: Call, limits: &Limits) -> Call {
    let note_refusal = note_shape(&body.tool, limits);
    if let Some(why) = note_refusal {
        body.tool = Tool::RejectedNote(why);
    }
    if let Some(why) = call_shape(&body.tool, &limits.tasks) {
        body.tool = match body.tool {
            Tool::Message { .. } => Tool::RejectedMessage(why),
            Tool::Amend { .. } | Tool::Cancel { .. } | Tool::Release { .. } | Tool::DecideEscalation { .. } => {
                Tool::RejectedControl(why)
            }
            Tool::Propose { .. } | Tool::Decide { .. } | Tool::Withdraw { .. } => Tool::RejectedProposal(why),
            Tool::Note { .. } | Tool::NoteLines { .. } | Tool::Recall { .. } => {
                Tool::RejectedNote(jig_domain_notes::Refusal::Oversized)
            }
            Tool::Finding(_)
            | Tool::Decided(_)
            | Tool::Goal { .. }
            | Tool::ReadTask { .. }
            | Tool::See { .. }
            | Tool::Document { .. }
            | Tool::Remember { .. }
            | Tool::Settings { .. }
            | Tool::ProposeSettings { .. }
            | Tool::ProposeSettingsNote { .. }
            | Tool::Change { .. }
            | Tool::Delegate { .. }
            | Tool::Introduce { .. }
            | Tool::Subscribe { .. }
            | Tool::SubscribeForge { .. }
            | Tool::SubscribeNamedForge { .. }
            | Tool::ReadNamed { .. }
            | Tool::ReadForge { .. }
            | Tool::EffectForge { .. }
            | Tool::Unsubscribe { .. }
            | Tool::Rejected(_)
            | Tool::RejectedMessage(_)
            | Tool::RejectedControl(_)
            | Tool::RejectedProposal(_)
            | Tool::RejectedNote(_)
            | Tool::Unavailable => Tool::Rejected(why),
        };
    }
    body
}

fn note_shape(tool: &Tool, limits: &Limits) -> Option<notes::Refusal> {
    match tool {
        Tool::Note { entry, .. } | Tool::NoteLines { entry, .. } => {
            if notes::valid_new(entry, &limits.notes) {
                None
            } else {
                Some(notes::new_refusal(entry, &limits.notes))
            }
        }
        Tool::Recall { by, .. } => (!notes::valid_recall(by, &limits.notes)).then_some(notes::Refusal::Oversized),
        Tool::Finding(_)
        | Tool::Decided(_)
        | Tool::Goal { .. }
        | Tool::ReadTask { .. }
        | Tool::See { .. }
        | Tool::Document { .. }
        | Tool::Remember { .. }
        | Tool::Settings { .. }
        | Tool::ProposeSettings { .. }
        | Tool::ProposeSettingsNote { .. }
        | Tool::Change { .. }
        | Tool::Delegate { .. }
        | Tool::Message { .. }
        | Tool::Amend { .. }
        | Tool::Cancel { .. }
        | Tool::Release { .. }
        | Tool::Decide { .. }
        | Tool::DecideEscalation { .. }
        | Tool::Withdraw { .. }
        | Tool::Propose { .. }
        | Tool::Introduce { .. }
        | Tool::Subscribe { .. }
        | Tool::SubscribeForge { .. }
        | Tool::SubscribeNamedForge { .. }
        | Tool::ReadNamed { .. }
        | Tool::ReadForge { .. }
        | Tool::EffectForge { .. }
        | Tool::Unsubscribe { .. }
        | Tool::Unavailable
        | Tool::Rejected(_)
        | Tool::RejectedMessage(_)
        | Tool::RejectedControl(_)
        | Tool::RejectedProposal(_)
        | Tool::RejectedNote(_) => None,
    }
}

pub(crate) fn relay_call(
    domain: &mut Domain,
    limits: &Limits,
    decision: &mut Decision,
    to: ReplyTo,
    answer: CallAnswer,
) {
    host_route::render(domain, limits, decision, to, answer);
}

pub(crate) fn call_needs_input(tool: &Tool) -> bool {
    match tool {
        Tool::Goal { agree, .. } => *agree,
        Tool::Finding(_)
        | Tool::Decided(_)
        | Tool::ReadTask { .. }
        | Tool::See { .. }
        | Tool::Document { .. }
        | Tool::Remember { .. }
        | Tool::Settings { .. }
        | Tool::ProposeSettings { .. }
        | Tool::ProposeSettingsNote { .. }
        | Tool::Change { .. }
        | Tool::Unavailable
        | Tool::Rejected(_)
        | Tool::RejectedMessage(_)
        | Tool::RejectedControl(_)
        | Tool::RejectedProposal(_)
        | Tool::RejectedNote(_)
        | Tool::Note { .. }
        | Tool::NoteLines { .. }
        | Tool::Recall { .. }
        | Tool::Message { .. }
        | Tool::Introduce { .. }
        | Tool::Subscribe { .. }
        | Tool::SubscribeForge { .. }
        | Tool::SubscribeNamedForge { .. }
        | Tool::ReadNamed { .. }
        | Tool::ReadForge { .. }
        | Tool::EffectForge { .. }
        | Tool::Unsubscribe { .. }
        | Tool::Cancel { .. }
        | Tool::Release { .. }
        | Tool::Amend { .. }
        | Tool::Propose { .. }
        | Tool::Decide { .. }
        | Tool::Withdraw { .. }
        | Tool::DecideEscalation { .. } => false,
        Tool::Delegate { batch } => {
            for member in batch {
                if !member.spec.inputs.is_empty() {
                    return true;
                }
            }
            false
        }
    }
}

#[expect(clippy::too_many_lines, reason = "one exhaustive tool-shape match keeps bounded admission together")]
pub(crate) fn call_shape(tool: &Tool, limits: &tasks::Limits) -> Option<tasks::Refusal> {
    match tool {
        Tool::Propose { action, reason, .. } => proposal_shape(action, reason, limits),
        Tool::Decide { decision, .. } => match decision {
            ProposalChoice::Reject { reason } if reason.len() > skein_lib::usize_of(limits.message_bytes) => {
                Some(tasks::Refusal::Read)
            }
            ProposalChoice::Accept | ProposalChoice::Reject { .. } | ProposalChoice::Pass => None,
        },
        Tool::DecideEscalation { decision, .. } => match decision {
            EscalationChoice::Reject { reason } if reason.len() > skein_lib::usize_of(limits.result_bytes) => {
                Some(tasks::Refusal::Read)
            }
            EscalationChoice::Release | EscalationChoice::Reject { .. } | EscalationChoice::Pass => None,
        },
        Tool::Finding(_)
        | Tool::Decided(_)
        | Tool::Goal { .. }
        | Tool::ReadTask { .. }
        | Tool::See { .. }
        | Tool::Document { .. }
        | Tool::Remember { .. }
        | Tool::Settings { .. }
        | Tool::ProposeSettings { .. }
        | Tool::ProposeSettingsNote { .. }
        | Tool::Change { .. }
        | Tool::Withdraw { .. }
        | Tool::Unavailable
        | Tool::Rejected(_)
        | Tool::RejectedMessage(_)
        | Tool::RejectedControl(_)
        | Tool::RejectedProposal(_)
        | Tool::RejectedNote(_)
        | Tool::Note { .. }
        | Tool::NoteLines { .. }
        | Tool::Recall { .. }
        | Tool::Introduce { .. }
        | Tool::Subscribe { .. }
        | Tool::SubscribeForge { .. }
        | Tool::SubscribeNamedForge { .. }
        | Tool::ReadNamed { .. }
        | Tool::ReadForge { .. }
        | Tool::EffectForge { .. }
        | Tool::Unsubscribe { .. }
        | Tool::Release { .. } => None,
        Tool::Cancel { reason, .. } => {
            if reason.len() > skein_lib::usize_of(limits.result_bytes) {
                Some(tasks::Refusal::Read)
            } else {
                None
            }
        }
        Tool::Amend { amendment, .. } => {
            if amendment.reason.len() > skein_lib::usize_of(limits.message_bytes) {
                return Some(tasks::Refusal::Read);
            }
            if let Some(spec) = &amendment.spec
                && (!tasks::valid_spec(limits, spec) || !spec.inputs.is_empty())
            {
                return Some(tasks::Refusal::Spec);
            }
            if let Some(authority) = &amendment.authority
                && !tasks::valid_authority(limits, authority)
            {
                return Some(tasks::Refusal::AuthorityShape);
            }
            if let Some(dependencies) = &amendment.dependencies
                && dependencies.len() > skein_lib::usize_of(limits.dependencies)
            {
                return Some(tasks::Refusal::Dependencies);
            }
            None
        }
        Tool::Message { form, words, .. } => {
            if words.is_empty() || words.len() > skein_lib::usize_of(limits.message_bytes) {
                return Some(tasks::Refusal::Read);
            }
            match form {
                MessageForm::Answer { question } if *question == 0 => return Some(tasks::Refusal::Read),
                MessageForm::Words | MessageForm::Question | MessageForm::Answer { .. } => {}
            }
            None
        }
        Tool::Delegate { batch } => {
            if batch.len() > skein_lib::usize_of(limits.batch) {
                return Some(tasks::Refusal::Batch);
            }
            for member in batch {
                if !tasks::valid_spec(limits, &member.spec) {
                    return Some(tasks::Refusal::Spec);
                }
                if !tasks::valid_contract(limits, &member.contract) {
                    return Some(tasks::Refusal::Contract);
                }
                if !tasks::valid_authority(limits, &member.authority) {
                    return Some(tasks::Refusal::AuthorityShape);
                }
                if member.dependencies.len() > skein_lib::usize_of(limits.dependencies) {
                    return Some(tasks::Refusal::Dependencies);
                }
            }
            None
        }
    }
}

fn proposal_shape(action: &ProposedAction, reason: &[u8], limits: &tasks::Limits) -> Option<tasks::Refusal> {
    if reason.len() > skein_lib::usize_of(limits.message_bytes) {
        return Some(tasks::Refusal::Read);
    }
    match action {
        ProposedAction::Batch(batch) => {
            if batch.is_empty() || batch.len() > skein_lib::usize_of(limits.batch) {
                return Some(tasks::Refusal::Batch);
            }
            for member in batch {
                if !tasks::valid_spec(limits, &member.spec) || !member.spec.inputs.is_empty() {
                    return Some(tasks::Refusal::Spec);
                }
                if !tasks::valid_contract(limits, &member.contract)
                    || !tasks::valid_authority(limits, &member.authority)
                    || member.dependencies.len() > skein_lib::usize_of(limits.dependencies)
                {
                    return Some(tasks::Refusal::AuthorityShape);
                }
            }
            None
        }
        ProposedAction::Amend { amendment, .. } => {
            if amendment.reason.len() > skein_lib::usize_of(limits.message_bytes) {
                Some(tasks::Refusal::Read)
            } else if match &amendment.authority {
                Some(authority) => !tasks::valid_authority(limits, authority),
                None => false,
            } {
                Some(tasks::Refusal::AuthorityShape)
            } else {
                None
            }
        }
        ProposedAction::Widen { authority, .. } => {
            if tasks::valid_authority(limits, authority) {
                None
            } else {
                Some(tasks::Refusal::AuthorityShape)
            }
        }
        ProposedAction::Effect { .. } | ProposedAction::Release { .. } => None,
    }
}

pub(crate) fn tool_kind(tool: &Tool) -> Option<jig_domain_core::ToolKind> {
    Some(match tool {
        Tool::See { .. } | Tool::Document { .. } | Tool::Remember { .. } => jig_domain_core::ToolKind::Ask,
        Tool::Settings { .. } | Tool::ProposeSettings { .. } | Tool::ProposeSettingsNote { .. } => {
            jig_domain_core::ToolKind::Settings
        }
        Tool::Finding(_) | Tool::Goal { .. } | Tool::Change { .. } | Tool::Delegate { .. } => {
            jig_domain_core::ToolKind::Delegate
        }
        Tool::Message { target, .. } => jig_domain_core::ToolKind::Message { target: *target },
        Tool::Introduce { left, .. } => jig_domain_core::ToolKind::Message { target: *left },
        Tool::Decided(_) | Tool::Amend { .. } | Tool::Cancel { .. } | Tool::Release { .. } => {
            jig_domain_core::ToolKind::Control
        }
        Tool::Decide { .. } | Tool::Withdraw { .. } | Tool::DecideEscalation { .. } => {
            jig_domain_core::ToolKind::Decide
        }
        Tool::Propose { .. } => jig_domain_core::ToolKind::Propose,
        Tool::Subscribe { .. }
        | Tool::SubscribeForge { .. }
        | Tool::SubscribeNamedForge { .. }
        | Tool::Unsubscribe { .. } => jig_domain_core::ToolKind::Subscribe,
        Tool::EffectForge { .. } => jig_domain_core::ToolKind::Effect,
        Tool::Note { entry, .. } | Tool::NoteLines { entry, .. } => {
            jig_domain_core::ToolKind::Note { scope: entry.scope.clone() }
        }
        Tool::Recall { by, .. } => jig_domain_core::ToolKind::Recall { by: by.clone() },
        Tool::ReadTask { .. } | Tool::ReadNamed { .. } | Tool::ReadForge { .. } => jig_domain_core::ToolKind::Read,
        Tool::Unavailable
        | Tool::Rejected(_)
        | Tool::RejectedMessage(_)
        | Tool::RejectedControl(_)
        | Tool::RejectedProposal(_)
        | Tool::RejectedNote(_) => return None,
    })
}

pub(crate) const fn named_call(key: crate::CallKey) -> forge::NamedCall {
    forge::NamedCall { task: key.task, attempt: key.attempt, completion: key.completion, position: key.position }
}

pub(crate) const fn core_call(key: forge::NamedCall) -> crate::CallKey {
    crate::CallKey { task: key.task, attempt: key.attempt, completion: key.completion, position: key.position }
}

pub(crate) fn connector_answer(domain: &Domain, key: crate::CallKey) -> Option<CallAnswer> {
    let payload = domain.forge.named_answer(named_call(key))?;
    Some(call_payload_answer(payload))
}

pub(crate) fn call_payload_answer(payload: forge::CallPayload) -> CallAnswer {
    match payload {
        forge::CallPayload::Finding(reply) => CallAnswer::Finding(Box::new(crate::FindingToolAnswer::Write(*reply))),
        forge::CallPayload::Decided { took, reach } => CallAnswer::Decided { took, reach },
        forge::CallPayload::Effect { entry, deadline, outcome } => CallAnswer::ForgeEffect { entry, deadline, outcome },
        forge::CallPayload::Refused(error) => CallAnswer::ForgeEffectRefused(error),
        forge::CallPayload::Read(answer) => CallAnswer::ForgeRead(answer),
    }
}

#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(clippy::result_large_err, reason = "a refused answer returns to its caller whole")
)]
pub(crate) fn keep_connector_answer(
    domain: &mut Domain,
    key: crate::CallKey,
    answer: CallAnswer,
) -> Result<Option<CallAnswer>, CallAnswer> {
    let payload = match answer {
        CallAnswer::Finding(answer) => match *answer {
            crate::FindingToolAnswer::Write(reply) => forge::CallPayload::Finding(Box::new(reply)),
            crate::FindingToolAnswer::List(_) | crate::FindingToolAnswer::Show { .. } => {
                unreachable!("finding reads are not recorded")
            }
        },
        CallAnswer::Decided { took, reach } => forge::CallPayload::Decided { took, reach },
        CallAnswer::ForgeEffect { entry, deadline, outcome } => forge::CallPayload::Effect { entry, deadline, outcome },
        CallAnswer::ForgeEffectRefused(error) => forge::CallPayload::Refused(error),
        CallAnswer::ForgeRead(answer) => forge::CallPayload::Read(answer),
        answer @ (CallAnswer::TaskRead(_)
        | CallAnswer::ForgeEffectDenied { .. }
        | CallAnswer::ToolDenied { .. }
        | CallAnswer::EscalationDecided { .. }
        | CallAnswer::EscalationRefused(_)
        | CallAnswer::Proposed { .. }
        | CallAnswer::ProposalDecided { .. }
        | CallAnswer::ProposalRefused(_)
        | CallAnswer::Again { .. }
        | CallAnswer::PassedUp { .. }
        | CallAnswer::Revised { .. }
        | CallAnswer::Amended { .. }
        | CallAnswer::Moved { .. }
        | CallAnswer::Controlled
        | CallAnswer::EffectRefused(_)
        | CallAnswer::ControlRefused(_)
        | CallAnswer::ControlDenied { .. }
        | CallAnswer::Sent { .. }
        | CallAnswer::Introduced
        | CallAnswer::MessageRefused(_)
        | CallAnswer::Subscribed { .. }
        | CallAnswer::Unsubscribed
        | CallAnswer::ForgeSubscriptionRefused { .. }
        | CallAnswer::SubscriptionRefused(_)
        | CallAnswer::Delegated(_)
        | CallAnswer::DelegationDenied { .. }
        | CallAnswer::ReadRepositoryRefused { .. }
        | CallAnswer::SubscriptionRepositoryRefused { .. }
        | CallAnswer::ChangeRefused { .. }
        | CallAnswer::DelegationRefused(_)
        | CallAnswer::NoteWritten { .. }
        | CallAnswer::NoteDeleted { .. }
        | CallAnswer::NoteRecalled { .. }
        | CallAnswer::Parts(_)
        | CallAnswer::Document(_)
        | CallAnswer::Remember(_)
        | CallAnswer::NoteRefused(_)
        | CallAnswer::SettingsRefused(_)
        | CallAnswer::HoldOwner { .. }
        | CallAnswer::Unavailable) => return Err(answer),
    };
    match domain.forge.keep_named_answer(&domain.limits.forge, named_call(key), payload) {
        Ok(Some(payload)) => Ok(Some(call_payload_answer(payload))),
        Ok(None) => Ok(None),
        Err(payload) => Err(call_payload_answer(payload)),
    }
}

pub(crate) fn is_core_read(tool: &Tool) -> bool {
    match tool {
        Tool::Finding(call) => match call {
            crate::FindingCall::List { .. } | crate::FindingCall::Show { .. } => true,
            crate::FindingCall::File { .. } | crate::FindingCall::Hand { .. } | crate::FindingCall::Triage { .. } => {
                false
            }
        },
        Tool::ReadTask { .. } | Tool::See { .. } | Tool::Settings { .. } => true,
        Tool::Document { .. }
        | Tool::Remember { .. }
        | Tool::ProposeSettings { .. }
        | Tool::ProposeSettingsNote { .. }
        | Tool::Decided(_)
        | Tool::Goal { .. }
        | Tool::Note { .. }
        | Tool::NoteLines { .. }
        | Tool::Recall { .. }
        | Tool::EffectForge { .. }
        | Tool::ReadNamed { .. }
        | Tool::ReadForge { .. }
        | Tool::Propose { .. }
        | Tool::Decide { .. }
        | Tool::Withdraw { .. }
        | Tool::DecideEscalation { .. }
        | Tool::Change { .. }
        | Tool::Delegate { .. }
        | Tool::Amend { .. }
        | Tool::Cancel { .. }
        | Tool::Release { .. }
        | Tool::Message { .. }
        | Tool::Introduce { .. }
        | Tool::Subscribe { .. }
        | Tool::SubscribeForge { .. }
        | Tool::SubscribeNamedForge { .. }
        | Tool::Unsubscribe { .. }
        | Tool::Rejected(_)
        | Tool::RejectedMessage(_)
        | Tool::RejectedControl(_)
        | Tool::RejectedProposal(_)
        | Tool::RejectedNote(_)
        | Tool::Unavailable => false,
    }
}

/// Translate test worlds' landing rules to forge stock (temper/docs/design/domain/root.md, 5).
#[must_use]
pub fn landing_stock(
    deployment: &[LandingRule],
    projects: &Map<u32, Box<[LandingRule]>>,
    change: u32,
    review: u32,
) -> Box<[forge::settings::Stock]> {
    crate::route::policy_translate::landing_stock(deployment, projects, change, review)
}

#[cfg(test)]
mod proposal_tests {
    use super::{ProposedAction, proposal_shape, tasks};
    use alloc::boxed::Box;

    fn amendment(authority: Option<tasks::Authority>, reason: &[u8]) -> ProposedAction {
        ProposedAction::Amend {
            task: 1,
            revision: 0,
            amendment: tasks::Amendment {
                tracked: None,
                addressed: None,
                instructions: Some(b"revised instructions".as_slice().into()),
                release: None,
                spec: None,
                wake: None,
                dependencies: None,
                authority,
                llm: None,
                reason: reason.into(),
            },
        }
    }

    #[test]
    fn an_amend_proposal_may_omit_authority() {
        let limits = crate::tests::resume_fixture::limits().tasks;
        assert_eq!(proposal_shape(&amendment(None, b"new plan"), b"please amend", &limits), None);
    }

    #[test]
    fn an_amend_proposal_rejects_malformed_named_authority() {
        let limits = crate::tests::resume_fixture::limits().tasks;
        let mut authority = tasks::Authority {
            tools: tasks::Tools(0),
            models: tasks::Models::NONE,
            grants: Box::new([]),
            delegation: tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
            budget: tasks::Budget { spend: 0, deadline: None },
            notes: tasks::Scopes(0),
            note_resources: Box::new([]),
        };
        assert!(tasks::valid_authority(&limits, &authority));
        assert_eq!(proposal_shape(&amendment(Some(authority.clone()), b"plan"), b"ask", &limits), None);
        authority.notes = tasks::Scopes(8);
        assert!(!tasks::valid_authority(&limits, &authority));
        assert_eq!(
            proposal_shape(&amendment(Some(authority), b"plan"), b"ask", &limits),
            Some(tasks::Refusal::AuthorityShape)
        );
    }

    #[test]
    fn an_amend_proposal_bounds_both_reasons() {
        let mut limits = crate::tests::resume_fixture::limits().tasks;
        limits.message_bytes = 4;
        assert_eq!(proposal_shape(&amendment(None, b"plan"), b"ask!", &limits), None);
        assert_eq!(proposal_shape(&amendment(None, b"plan!"), b"ask!", &limits), Some(tasks::Refusal::Read));
        assert_eq!(proposal_shape(&amendment(None, b"plan"), b"ask!!", &limits), Some(tasks::Refusal::Read));
    }
}
