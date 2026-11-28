//! Total vocabulary and configuration translations (jig's domain/root.md, 6).
use crate::Call;
use crate::domain::Domain;
use crate::limits::Limits;
use crate::route::{LandingRule, host_route};
use crate::{CallAnswer, Decision};
use alloc::boxed::Box;
use jig_domain_link::JobBound;
use skein_lib::{Env, Map, ReplyTo};
use temper_engine_domain_forge as forge;
pub(crate) use temper_engine_domain_translate::{call_needs_input, is_core_read, tool_kind};

/// Startup configuration, supplied once by the shell (domain/root-translation.md, 1):
/// the core's whole, as jig's applications hold it, and temper's own.
#[derive(Debug)]
pub struct Config {
    /// Measured charter costs, including without an agent (temper/docs/design/domain/agent/room.md, 4.4).
    pub host_part: crate::room::HostPart,
    /// Deployment funding span (domain/routing.md, 7).
    pub period_span: skein_lib::Duration,
    /// The core's rules, charters, owners and projects; its one connector is
    /// the forge's; whether the workers run in the engine's process is
    /// configuration too, set by an in-process hosts' listener: standalone,
    /// they do (domain/root.md, 2).
    pub core: jig_domain_core::Config,
    /// Operator hold taken once at start (domain/landing-settings.md, 1.3).
    pub landing_held: bool,
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
    pub(crate) prepare_capacity: (u32, u32),
    pub(crate) turn_capacity: Option<&'static [u8]>,
    pub(crate) merge_kind: Option<JobBound>,
    pub(crate) close_kind: Option<JobBound>,
    pub(crate) forge_connector: u16,
}

/// Landing rules for test worlds' builders alone (domain/root-translation.md, 1).
#[derive(Debug)]
pub struct LandingRules {
    pub deployment: Box<[LandingRule]>,
    pub projects: Map<u32, Box<[LandingRule]>>,
}

/// The goal closer's attempts (temper/docs/design/protocol/configuration/engine.md).
pub const LANDING_ATTEMPTS: u8 = 3;

pub(crate) fn environment_forge(env: &Env<Limits>) -> Env<forge::Limits> {
    env.with(forge_limits(&env.limits))
}

pub(crate) const fn forge_limits(limits: &Limits) -> forge::Limits {
    forge::Limits { waits: limits.waits, ..limits.forge }
}

pub(crate) fn checked_call(body: Call, limits: &Limits) -> Call {
    temper_engine_domain_translate::checked_call(body, &limits.tasks, &limits.notes)
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
        forge::CallPayload::Follow { follow, refusal } => {
            CallAnswer::Follow(Box::new(crate::FollowToolAnswer::Write { follow, refusal }))
        }
        forge::CallPayload::Red(reply) => CallAnswer::Red(Box::new(crate::RedToolAnswer::Write(*reply))),
        forge::CallPayload::Finding(reply) => CallAnswer::Finding(Box::new(crate::FindingToolAnswer::Write(*reply))),
        forge::CallPayload::Decided { took, reach } => CallAnswer::Decided { took, reach },
        forge::CallPayload::Effect { entry, deadline, outcome } => CallAnswer::ForgeEffect { entry, deadline, outcome },
        forge::CallPayload::Refused(error) => CallAnswer::ForgeEffectRefused(error),
        forge::CallPayload::Read(answer) => CallAnswer::ForgeRead(answer),
    }
}

#[expect(clippy::result_large_err, reason = "a refused answer returns to its caller whole")]
pub(crate) fn keep_connector_answer(
    domain: &mut Domain,
    key: crate::CallKey,
    answer: CallAnswer,
) -> Result<Option<CallAnswer>, CallAnswer> {
    let payload = match answer {
        CallAnswer::Follow(answer) => match *answer {
            crate::FollowToolAnswer::Write { follow, refusal } => forge::CallPayload::Follow { follow, refusal },
            crate::FollowToolAnswer::List(_) | crate::FollowToolAnswer::Show { .. } => {
                unreachable!("follow reads are not recorded")
            }
        },
        CallAnswer::Red(answer) => match *answer {
            crate::RedToolAnswer::Write(reply) => forge::CallPayload::Red(Box::new(reply)),
            crate::RedToolAnswer::List { .. } | crate::RedToolAnswer::NoGoal { .. } => {
                unreachable!("red reads are not recorded")
            }
        },
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
        | CallAnswer::WordsRecorded { .. }
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

/// Translate test worlds' landing rules to forge stock (temper/docs/design/domain/root-translation.md, 1).
#[must_use]
pub fn landing_stock(
    deployment: &[LandingRule],
    projects: &Map<u32, Box<[LandingRule]>>,
    change: u32,
    review: u32,
) -> Box<[forge::settings::Stock]> {
    crate::route::policy_translate::landing_stock(deployment, projects, change, review)
}

/// The forge's connector capabilities, in the engine's vocabulary.
pub(crate) fn forge_declaration(connector: u16) -> jig_domain_contract::Declaration {
    use jig_domain_contract::{
        Connector, Declaration,
        effects::{EffectForm, EffectKind, Recovery},
        resources::{ResourceKind, Rule, Taken},
    };
    use temper_engine_domain_forge as forge;
    let hold = match forge::resources::BRANCH_HOLD {
        forge::resources::HoldKind::Exclusive { wait } => {
            Rule::Exclusive { taken: if wait { Taken::Waits } else { Taken::Refuses } }
        }
        forge::resources::HoldKind::Shared => unreachable!("private branch kind takes exclusive holds"),
    };
    let count = u32::try_from(forge::EFFECT_KINDS.len()).expect("bounded forge kinds");
    let mut effects = skein_lib::List::with_capacity(count);
    for row in forge::EFFECT_KINDS {
        let form = match row.form {
            forge::EffectForm::Creation => EffectForm::Creation,
            forge::EffectForm::Transition => EffectForm::Transition,
            forge::EffectForm::Set => EffectForm::Set,
        };
        let recovery = match row.recovery {
            forge::Recovery::Keyed => Recovery::Keyed,
            forge::Recovery::Conditional => Recovery::Conditional,
            forge::Recovery::Idempotent => Recovery::Idempotent,
            forge::Recovery::Unrecoverable => Recovery::Unrecoverable,
        };
        effects
            .push(EffectKind { kind: row.kind, form, recovery, waits_for_dependencies: row.waits_for_dependencies })
            .expect("one declaration per forge write kind");
    }
    Declaration {
        connector: Connector(connector),
        resources: Box::new([ResourceKind { kind: 1, hold }]),
        effects: effects.into_boxed(),
        projects_goals: true,
        sections: false,
    }
}

#[cfg(test)]
mod proposal_tests {
    use super::checked_call;
    use crate::{Call, ProposedAction, Tool};
    use alloc::boxed::Box;
    use jig_domain_tasks as tasks;

    fn proposal_refusal(action: ProposedAction, reason: &[u8], limits: &crate::Limits) -> Option<tasks::Refusal> {
        let body = Call {
            completion: 1,
            position: 0,
            tool: Tool::Propose { action, reason: reason.into(), as_holder: false },
        };
        let tool = checked_call(body, limits).tool;
        if let Tool::RejectedProposal(why) = tool {
            return Some(why);
        }
        let Tool::Propose { .. } = tool else { panic!("unexpected proposal tool: {tool:?}") };
        None
    }

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
        let limits = crate::tests::resume_fixture::limits();
        assert_eq!(proposal_refusal(amendment(None, b"new plan"), b"please amend", &limits), None);
    }

    #[test]
    fn an_amend_proposal_rejects_malformed_named_authority() {
        let limits = crate::tests::resume_fixture::limits();
        let mut authority = tasks::Authority {
            tools: tasks::Tools(0),
            models: tasks::Models::NONE,
            grants: Box::new([]),
            delegation: tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
            budget: tasks::Budget { spend: 0, deadline: None },
            notes: tasks::Scopes(0),
            note_resources: Box::new([]),
        };
        assert!(tasks::valid_authority(&limits.tasks, &authority));
        assert_eq!(proposal_refusal(amendment(Some(authority.clone()), b"plan"), b"ask", &limits), None);
        authority.notes = tasks::Scopes(8);
        assert!(!tasks::valid_authority(&limits.tasks, &authority));
        assert_eq!(
            proposal_refusal(amendment(Some(authority), b"plan"), b"ask", &limits),
            Some(tasks::Refusal::AuthorityShape)
        );
    }

    #[test]
    fn an_amend_proposal_bounds_both_reasons() {
        let mut limits = crate::tests::resume_fixture::limits();
        limits.tasks.message_bytes = 4;
        assert_eq!(proposal_refusal(amendment(None, b"plan"), b"ask!", &limits), None);
        assert_eq!(proposal_refusal(amendment(None, b"plan!"), b"ask!", &limits), Some(tasks::Refusal::Read));
        assert_eq!(proposal_refusal(amendment(None, b"plan"), b"ask!!", &limits), Some(tasks::Refusal::Read));
    }
}
