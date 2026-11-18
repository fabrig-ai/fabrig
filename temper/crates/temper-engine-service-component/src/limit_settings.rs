//! The open-store, pre-root seam (configuration/limit-settings.md, 3).
use crate::store::Store;
use alloc::boxed::Box;
use jig_domain_core::{self as core, settings::Override};
use jig_protocol_config::{Path, What};
use temper_engine_domain_forge::{self as forge, settings};
pub use temper_engine_service_startup::startup::{LimitInputs, LimitKind, Resolved, resolve_limits};
use temper_engine_service_startup::startup::{Problems, What as StartWhat};

#[derive(Debug)]
pub struct InForce {
    pub inputs: LimitInputs,
    pub jig: Box<[Override]>,
    pub temper: Box<[Override]>,
    pub resolved: Resolved,
    pub held: bool,
}

impl crate::Engine {
    pub fn take_limit_store(&mut self, store: Store, state: InForce) {
        assert!(self.limits_in_force.is_none(), "limit settings installed once before restore");
        self.replace_store(store);
        self.limits_in_force = Some(state);
    }
    #[must_use]
    pub fn limits_in_force(&self) -> Option<&InForce> {
        self.limits_in_force.as_ref()
    }
}

impl Store {
    /// Only the stock is allowed to size the substrate that opens/replays.
    pub fn for_limits(inputs: &LimitInputs, models: &[core::Model]) -> Result<Self, Problems> {
        let (mut jig, own) = inputs.stock()?;
        let root = temper_engine_service_startup::startup::engine_limits(&mut jig, &own, None)?;
        let store = temper_engine_service_startup::startup::checked_store(&root, &jig.store, None)?;
        Ok(Self::new(&root, &store, models, inputs.seeds.installation))
    }
    /// After Opened and before Restart: read, resolve, then install the codecs.
    /// A held pair replaces both kinds only for resolution, not the retained settings.
    pub fn resolve_settings(
        &mut self,
        inputs: LimitInputs,
        held: Option<(&[Override], &[Override])>,
    ) -> Result<InForce, Problems> {
        let key = settings_key();
        let encoded = temper_protocol_store::keys::encode(&forge::Key::Setting(key), self.host.limits().store.key)
            .ok_or(bad(b"temper-limits"))?;
        let (jig, temper) = self.host.limit_settings(encoded).or(Err(bad(b"limits")))?;
        let jig = jig_list(jig).ok_or(bad(b"limits"))?;
        let (mut stock_jig, stock_own) = inputs.stock()?;
        let stock_root = temper_engine_service_startup::startup::engine_limits(&mut stock_jig, &stock_own, None)?;
        let bounds = temper_protocol_store::bounds(&stock_root.forge, stock_root.tasks.spec_bytes)
            .ok_or(bad(b"temper-limits"))?;
        let temper = temper_list(temper, &bounds).ok_or(bad(b"temper-limits"))?;
        let resolved = match held {
            Some((jig, temper)) => resolve_limits(&inputs, jig, temper)?,
            None => resolve_limits(&inputs, &jig, &temper)?,
        };
        let application = temper_protocol_store::application(&resolved.root.forge, resolved.root.tasks.spec_bytes)
            .ok_or(bad(b"temper-limits"))?;
        if let Err(why) = self.host.install_limits(
            &temper_engine_domain::core_limits(&resolved.root),
            application,
            resolved.root.journal.transcript_bytes.max(resolved.root.loads.transcript_bytes),
        ) {
            return Err(match why {
                jig_protocol_store::InstallRefusal::NotOpening => bad(b"limits"),
                jig_protocol_store::InstallRefusal::Store(why) => {
                    temper_engine_service_startup::startup::checks::store_refusal(crate::store::Refusal::Map(why))
                }
            });
        }
        self.routes = crate::store::route::Routes::new(
            resolved.root.loads.loads,
            &resolved.root,
            temper_protocol_store::bounds(&resolved.root.forge, resolved.root.tasks.spec_bytes)
                .ok_or(bad(b"temper-limits"))?,
        );
        Ok(InForce { inputs, jig, temper, resolved, held: held.is_some() })
    }
    /// Opening's terminal is consumed before building the root. Other terminals
    /// (notably the census) stay queued for the root's normal first pass.
    pub fn opened_limits(&mut self, installation: [u8; 16], pending: bool) {
        self.opened = true;
        self.installation = installation;
        self.pending_generation = pending;
    }
}
pub(crate) fn restore_refusal(state: Option<&InForce>, stopped: &crate::Stop) -> Option<crate::Stop> {
    let state = state?;
    if state.held {
        return None;
    }
    let (name, held, configured) = match stopped {
        crate::Stop::StartRefused { failure, .. } => core_fit(failure)?,
        crate::Stop::ForgeFit { failure, .. } => {
            (forge_fit(failure)?, failure.held.saturating_add(failure.additional), failure.limit)
        }
        crate::Stop::FrameTooLarge { .. }
        | crate::Stop::Journal { .. }
        | crate::Stop::CommitFailed { .. }
        | crate::Stop::Start(_)
        | crate::Stop::LimitRestore { .. }
        | crate::Stop::Room(_)
        | crate::Stop::RestorePage { .. }
        | crate::Stop::ForgeLink { .. }
        | crate::Stop::Binding
        | crate::Stop::Setting(_)
        | crate::Stop::StoreOpen { .. }
        | crate::Stop::StoreRow { .. }
        | crate::Stop::Listener { .. }
        | crate::Stop::Hooks => return None,
    };
    let kind = restore_source(state, name)?;
    Some(crate::Stop::LimitRestore { name, kind, configured, held })
}
fn restore_source(state: &InForce, name: &[u8]) -> Option<LimitKind> {
    for item in &state.jig {
        if item.name.as_ref() == name && jig_protocol_config::names::find(name).is_some() {
            return Some(LimitKind::Jig);
        }
    }
    for item in &state.temper {
        if item.name.as_ref() == name && crate::limits::find(name).is_some() {
            return Some(LimitKind::Temper);
        }
    }
    let (jig, own) = state.inputs.stock().ok()?;
    let changed = match jig_protocol_config::names::find(name) {
        Some(row) => (row.value)(&jig) != (row.value)(&state.resolved.jig),
        None => match crate::limits::find(name) {
            Some(row) => (row.value)(&own) != (row.value)(&state.resolved.temper),
            None => false,
        },
    };
    if !changed {
        return None;
    }
    if [b"tasks.delegates".as_slice(), b"tasks.exemptions", b"people.operations", b"people.inbox_tasks"].contains(&name)
    {
        for item in &state.jig {
            if [b"tasks.tasks".as_slice(), b"tasks.delegates", b"tasks.project_goals"].contains(&item.name.as_ref()) {
                return Some(LimitKind::Jig);
            }
        }
    }
    for item in &state.temper {
        if item.name.as_ref() == b"live_goals" || item.name.as_ref() == b"tool_payload" {
            return Some(LimitKind::Temper);
        }
    }
    None
}
fn core_fit(failure: &core::start::Failure) -> Option<(&'static [u8], u64, u64)> {
    use core::start::{Failure, HeldLimit, LiveSet};
    match failure {
        Failure::HeldPastLimit { which, held, limit } => Some((
            match which {
                HeldLimit::Revisions => Some(b"core.settings.revisions".as_slice()),
                HeldLimit::Versions => Some(b"core.settings.versions".as_slice()),
            }?,
            u64::from(*held),
            u64::from(*limit),
        )),
        Failure::NeedsPastLimit { held, limit } => Some((b"fleet.needs_bytes", *held, *limit)),
        Failure::LiveSet { set, held, limit, .. } => Some((
            match set {
                LiveSet::Projects => Some(b"authority.projects".as_slice()),
                LiveSet::RunProofs | LiveSet::Census => Some(b"tasks.tasks".as_slice()),
                LiveSet::Calls => Some(b"core.call_records".as_slice()),
                LiveSet::Projections => Some(b"tasks.project_goals".as_slice()),
                LiveSet::ReadResults | LiveSet::Conversations => None,
                LiveSet::WorkerClaims => Some(b"fleet.attempts".as_slice()),
                LiveSet::TasksBound(bound) => task_bound(bound),
                LiveSet::PeopleBound(bound) => people_bound(bound),
            }?,
            *held,
            *limit,
        )),
        Failure::Settings { .. }
        | Failure::Application { .. }
        | Failure::ProjectPolicy { .. }
        | Failure::AccountsPastLimit { .. }
        | Failure::Accounting { .. }
        | Failure::LlmChoice { .. }
        | Failure::Link { .. } => None,
    }
}
fn task_bound(bound: &jig_domain_tasks::RestoreBound) -> Option<&'static [u8]> {
    use jig_domain_tasks::RestoreBound as B;
    match bound {
        B::Stubs
        | B::Writers
        | B::PersonProposals
        | B::ResultFields
        | B::HoldPathCapacity
        | B::AccountOriginBytes
        | B::PreparationReasonBytes
        | B::Subagents
        | B::SettingsItems
        | B::RootHoldings
        | B::RefusedRows
        | B::ReminderBytes
        | B::ZeroBytes
        | B::AmendmentSlots
        | B::MetSlots => None,
        B::Tasks => Some(b"tasks.tasks".as_slice()),
        B::Pools => Some(b"tasks.pools".as_slice()),
        B::Funders => Some(b"tasks.funders".as_slice()),
        B::PlacedHolds => Some(b"tasks.placed_holds".as_slice()),
        B::HeldRevisions => Some(b"core.settings.revisions".as_slice()),
        B::ProjectTasks => Some(b"tasks.project_tasks".as_slice()),
        B::ProjectGoals => Some(b"tasks.project_goals".as_slice()),
        B::ProjectPlacedHolds => Some(b"tasks.project_placed_holds".as_slice()),
        B::PartyPlacedHolds => Some(b"tasks.party_placed_holds".as_slice()),
        B::Parameters => Some(b"tasks.parameters".as_slice()),
        B::Inputs => Some(b"tasks.inputs".as_slice()),
        B::SpecBytes => Some(b"tasks.spec_bytes".as_slice()),
        B::InboxBytes => Some(b"tasks.inbox_bytes".as_slice()),
        B::AuthorityGrants => Some(b"tasks.authority_grants".as_slice()),
        B::ExecutorKinds => Some(b"tasks.executor_kinds".as_slice()),
        B::AuthoritySegments => Some(b"tasks.authority_segments".as_slice()),
        B::AuthorityBytes => Some(b"tasks.authority_bytes".as_slice()),
        B::ResultBytes => Some(b"tasks.result_bytes".as_slice()),
        B::ContractChoices => Some(b"tasks.contract_choices".as_slice()),
        B::Batch => Some(b"tasks.batch_bytes".as_slice()),
        B::HoldSegments => Some(b"tasks.hold_segments".as_slice()),
        B::HoldBytes => Some(b"tasks.hold_bytes".as_slice()),
        B::Circle => Some(b"tasks.circle".as_slice()),
        B::Dependencies => Some(b"tasks.dependencies".as_slice()),
        B::Holdings => Some(b"tasks.holdings".as_slice()),
        B::MessageBytes => Some(b"tasks.message_bytes".as_slice()),
        B::Exemptions => Some(b"tasks.exemptions".as_slice()),
        B::Delegates => Some(b"tasks.delegates".as_slice()),
        B::References => Some(b"tasks.references".as_slice()),
        B::Subscriptions => Some(b"tasks.subscriptions".as_slice()),
        B::InboxMessages => Some(b"tasks.inbox_messages".as_slice()),
        B::SavedResources => Some(b"tasks.saved_resources".as_slice()),
    }
}
fn people_bound(bound: &jig_domain_people::RestoreBound) -> Option<&'static [u8]> {
    use jig_domain_people::RestoreBound as B;
    match bound {
        B::Roles
        | B::ReadResults
        | B::ReadConversations
        | B::ModelNames
        | B::ModelNameBytes
        | B::Subagents
        | B::Batch
        | B::NameBytes
        | B::ProgramNameBytes
        | B::Uses
        | B::AddressBytes
        | B::NamedTasks
        | B::KeptPending => None,
        B::IdentityBytes => Some(b"people.identity_bytes".as_slice()),
        B::People => Some(b"people.people".as_slice()),
        B::SignIns => Some(b"people.sign_ins".as_slice()),
        B::Holdings => Some(b"people.holdings".as_slice()),
        B::Requests => Some(b"people.requests".as_slice()),
        B::Operations => Some(b"people.operations".as_slice()),
        B::Words => Some(b"people.words".as_slice()),
        B::AmendmentBytes => Some(b"people.amendment_bytes".as_slice()),

        B::ProgramHosts => Some(b"people.program_hosts".as_slice()),
        B::ProgramHostNameBytes => Some(b"people.program_host_name_bytes".as_slice()),
    }
}
fn forge_fit(failure: &forge::RestoreFit) -> Option<&'static [u8]> {
    use forge::{RecordKind as R, RestoreSet as S};
    match failure.set {
        S::Rows(kind) => match kind {
            R::Entry => Some(b"forge.entries"),
            R::Binding => Some(b"forge.bindings"),
            R::Hold => Some(b"forge.holds"),
            R::Closer => Some(b"forge.tasks"),
            R::Repository => Some(b"forge.repositories"),
            R::Forge
            | R::Call
            | R::ProposedEffect
            | R::Names
            | R::Subscription
            | R::BranchHead
            | R::PullState
            | R::Ci
            | R::Landed
            | R::Client
            | R::Change
            | R::Issue
            | R::Release
            | R::Application
            | R::Writer
            | R::Setting
            | R::Flight
            | R::Landing
            | R::Receipt
            | R::Follow
            | R::Tier
            | R::Verification
            | R::LandingLog
            | R::Making
            | R::Dismissed
            | R::ProjectFacts
            | R::Looked
            | R::Away
            | R::Decided
            | R::Device
            | R::Finding
            | R::LandedDraft
            | R::Red
            | R::RedNumber
            | R::RedPass
            | R::HistoryProject
            | R::Milestone => None,
        },
        S::NamedAnswerBytes
        | S::NamedAnswerRows
        | S::BindingAddressBytes
        | S::BindingSubjectBytes
        | S::SettingGates
        | S::SettingApprovals
        | S::SettingSegments
        | S::ChangeTimers
        | S::ProjectionTimers
        | S::References
        | S::Watches
        | S::FollowQueue
        | S::LandingFlights
        | S::LandingMembers
        | S::LandingsKept
        | S::LandingGates
        | S::LandingAsks
        | S::LandingEntries
        | S::NameBytes
        | S::ContextProjects => None,
    }
}

#[must_use]
pub fn settings_key() -> settings::SettingKey {
    settings::SettingKey {
        kind: settings::Kind::Limits.number(),
        scope: 0,
        number: 1,
        slot: settings::SettingSlot::Change,
    }
}
fn bad(name: &[u8]) -> Problems {
    let mut problems = Problems::empty();
    problems.push(
        Path::root().member(name),
        StartWhat::Jig(What::Because(b"cannot read the stored limit setting before restore")),
    );
    problems
}
fn jig_list(row: Option<jig_protocol_store::Row>) -> Option<Box<[Override]>> {
    let Some(row) = row else {
        return Some(Box::new([]));
    };
    let row = match row {
        jig_protocol_store::Row::Jig(row) => row,
        jig_protocol_store::Row::Application { .. } => return None,
    };
    let row = match row {
        core::Record::Core(row) => row,
        core::Record::Tasks(_) | core::Record::People(_) | core::Record::Notes(_) | core::Record::Accounts(_) => {
            return None;
        }
    };
    let row = match row {
        core::CoreRecord::Settings(row) => row,
        core::CoreRecord::Projection(_)
        | core::CoreRecord::Deployment(_)
        | core::CoreRecord::Call(_)
        | core::CoreRecord::RunProof(_)
        | core::CoreRecord::Turn(_)
        | core::CoreRecord::Terminal(_)
        | core::CoreRecord::ProposalDecision(_)
        | core::CoreRecord::EscalationDecision(_) => return None,
    };
    let changed = match row {
        core::settings::Record::Changed { changed, .. } => changed,
        core::settings::Record::Default { .. }
        | core::settings::Record::Application { .. }
        | core::settings::Record::Revision { .. }
        | core::settings::Record::Extent { .. }
        | core::settings::Record::Charter { .. }
        | core::settings::Record::Proposal { .. } => return None,
    };
    let value = changed.value?;
    Some(Box::from(value.as_limits()?))
}
fn temper_list(
    row: Option<jig_protocol_store::Row>,
    bounds: &temper_protocol_store::Bounds,
) -> Option<Box<[Override]>> {
    let Some(row) = row else {
        return Some(Box::new([]));
    };
    let value = match row {
        jig_protocol_store::Row::Application { key, value } => (key, value),
        jig_protocol_store::Row::Jig(_) => return None,
    };
    let row = match temper_protocol_store::records::row(&value.0, &value.1, bounds).ok()? {
        forge::Stored::Setting(row) => row,
        forge::Stored::RefusedSetting(_)
        | forge::Stored::Call { .. }
        | forge::Stored::ProposedEffect(_)
        | forge::Stored::Repository(_)
        | forge::Stored::LimitHold(_)
        | forge::Stored::Hold(_)
        | forge::Stored::Names { .. }
        | forge::Stored::Subscription(_)
        | forge::Stored::BranchHead(_)
        | forge::Stored::PullState(_)
        | forge::Stored::Ci(_)
        | forge::Stored::Landed { .. }
        | forge::Stored::Entry { .. }
        | forge::Stored::Client(_)
        | forge::Stored::Closer(_)
        | forge::Stored::Change(_)
        | forge::Stored::Issue(_)
        | forge::Stored::Release(_)
        | forge::Stored::Application { .. }
        | forge::Stored::Writer(_)
        | forge::Stored::Binding { .. }
        | forge::Stored::Flight { .. }
        | forge::Stored::Landing(_)
        | forge::Stored::Receipt { .. }
        | forge::Stored::Forge(_)
        | forge::Stored::ForgeNumber { .. }
        | forge::Stored::Playground(_)
        | forge::Stored::Follow(_)
        | forge::Stored::Tier(_)
        | forge::Stored::Verification(_)
        | forge::Stored::LandingLog(_)
        | forge::Stored::Milestone(_)
        | forge::Stored::Making(_)
        | forge::Stored::Dismissed { .. }
        | forge::Stored::ProjectFacts(_)
        | forge::Stored::Looked(_)
        | forge::Stored::Away(_)
        | forge::Stored::Decided(_)
        | forge::Stored::Device(_)
        | forge::Stored::Finding(_)
        | forge::Stored::LandedDraft(_)
        | forge::Stored::Red(_)
        | forge::Stored::RedPass(_)
        | forge::Stored::RedNumber { .. }
        | forge::Stored::ChangeCold(_)
        | forge::Stored::ChangeHot(_)
        | forge::Stored::HistoryProject(_)
        | forge::Stored::HistoryEvent(_)
        | forge::Stored::HistoryChange(_)
        | forge::Stored::HistoryLanding(_)
        | forge::Stored::HistoryLast(_) => return None,
    };
    let value = match row.edit {
        settings::Edit::Set(value) => value.value,
        settings::Edit::Remove | settings::Edit::Reset => return Some(Box::new([])),
    };
    match value {
        settings::Value::Limits(items) => Some(items),
        settings::Value::Landing(_)
        | settings::Value::Gates(_)
        | settings::Value::Welcome(_)
        | settings::Value::Rule(_)
        | settings::Value::Charter(_)
        | settings::Value::Questions(_)
        | settings::Value::Ways(_)
        | settings::Value::Classes(_)
        | settings::Value::CharterWords(_)
        | settings::Value::ProjectWords(_)
        | settings::Value::Goal(_)
        | settings::Value::Follow(_)
        | settings::Value::Dependency(_)
        | settings::Value::ProjectAreas(_)
        | settings::Value::ProjectTriager(_) => None,
    }
}
