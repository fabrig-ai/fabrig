//! Bridge for roots that do not yet speak the contract (jig/docs/design/domain/engine/contracts.md, 1 and 3).

use alloc::boxed::Box;
use jig_domain_contract::{
    self as contract, Connector, Released, effects, lifecycle, parts, procedures, projections, requests, resources,
    settings, topics,
};

use crate::{Ask, Held, Request, effects::connector};

/// Route a connector ask through a root that still speaks the core vocabulary.
#[must_use]
pub fn bridge_ask(Connector(connector): Connector, ask: contract::Ask) -> Request {
    let ask = match ask {
        contract::Ask::Lifecycle(ask) => match ask {
            lifecycle::Ask::CatchUp => Ask::Effect(connector::Ask::CatchUp),
            lifecycle::Ask::Adopt { request, project, adoption } => Ask::Adopt { request, project, adoption },
            lifecycle::Ask::ReleaseProject { project } => Ask::ReleaseProject { project },
        },
        contract::Ask::Resources(ask) => match ask {
            resources::Ask::Unname { task } => Ask::Effect(connector::Ask::Unname { task }),
            resources::Ask::Hold { task, resource, entry } => Ask::Hold { task, resource, entry },
            resources::Ask::Lost { task, attempt, resources: _ } => Ask::Lost { task, attempt },
            resources::Ask::Holdings { request, project, root, number, executor, spec, tracked, symbols } => {
                Ask::TaskHoldings { request, project, root, number, executor, spec, tracked, symbols }
            }
            resources::Ask::DelegateHoldings { request, from, members } => {
                Ask::DelegateHoldings { request, from, members }
            }
            resources::Ask::ProcedureHoldings { task, step, members } => Ask::ProcedureHoldings { task, step, members },
            resources::Ask::RepairRefused { repair, number, why } => Ask::RepairRefused { repair, number, why },
            resources::Ask::DelegateRefused { task } => Ask::DelegateRefused { task },
        },
        contract::Ask::Effects(ask) => match ask {
            effects::Ask::Describe { owner } => Ask::Effect(connector::Ask::Describe { owner }),
            effects::Ask::DescribeProposal { owner, proposal } => {
                Ask::Effect(connector::Ask::DescribeProposal { owner, proposal })
            }
            effects::Ask::Judge { owner, judge, state, resources } => {
                Ask::Effect(connector::Ask::Judge { owner, judge, state, resources })
            }
            effects::Ask::Keep { owner, entry, task, key, project: _ } => {
                Ask::Effect(connector::Ask::Keep { owner, entry, task, key })
            }
            effects::Ask::KeepProposal { owner, proposal, task, project } => {
                Ask::Effect(connector::Ask::KeepProposal { owner, proposal, task, project })
            }
            effects::Ask::Drop { owner, answer, findings, why } => {
                Ask::Effect(connector::Ask::Drop { owner, answer, findings, why })
            }
            effects::Ask::DropProposal { proposal } => Ask::Effect(connector::Ask::DropProposal { proposal }),
            effects::Ask::Resolve { entry, again } => Ask::Effect(connector::Ask::ResolveUncertain { entry, again }),
            effects::Ask::Close { task, root, ending } => Ask::Close { task, root, ending },
            effects::Ask::Release { task, root, ending, entry } => Ask::Release { task, root, ending, entry },
        },
        contract::Ask::Topics(ask) => match ask {
            topics::Ask::Subscribed { request, key, subscription, .. } => {
                Ask::SubscriptionDone { request, key, subscription }
            }
            topics::Ask::Unsubscribed { request, key, .. } => Ask::UnsubscriptionDone { request, key },
            topics::Ask::Refused { request } => Ask::DropSubscription { request },
            topics::Ask::Ended { task, subscription } => Ask::EndTopic { task, subscription },
        },
        contract::Ask::Parts(ask) => match ask {
            parts::Ask::Document { watcher, scope, subject, party, window } => {
                Ask::Document { watcher, scope, subject, party, window }
            }
            parts::Ask::DropDocument { subject } => Ask::DropDocument { subject },
            parts::Ask::Read { reply, scope, read, party, window } => Ask::Read { reply, scope, read, party, window },
            parts::Ask::Lines(parts) => Ask::Parts(parts),
            parts::Ask::Public { reply, kind, version, project, task, public } => {
                Ask::Public { reply, kind, version, project, task, public }
            }
        },
        contract::Ask::Procedures(ask) => match ask {
            procedures::Ask::Start { context, step, chat: _ } => Ask::StartProcedure { context, step },
            procedures::Ask::StepMade { task, step, first, revisions } => {
                Ask::StepMade { task, step, first, revisions }
            }
            procedures::Ask::StepRefused { task, step, why } => Ask::StepRefused { task, step, why },
            procedures::Ask::WaitsRefused { task, why } => Ask::WaitsRefused { task, why },
        },
        contract::Ask::Projections(ask) => match ask {
            projections::Ask::Project { feed } => Ask::ProjectGoal { feed },
            projections::Ask::Forget { goal } => Ask::ForgetProjection { goal },
        },
        contract::Ask::Settings(ask) => match ask {
            settings::Ask::Judge { owner, revision, proposal, items } => {
                Ask::Effect(connector::Ask::JudgeSettings { owner, revision, proposal, items })
            }
            settings::Ask::Restore { owner, proposal, items } => {
                Ask::Effect(connector::Ask::RestoreSettings { owner, proposal, items })
            }
            settings::Ask::Keep { revision, proposal, items } => {
                Ask::Effect(connector::Ask::KeepSettings { revision, proposal, items })
            }
            settings::Ask::KeepProposal { proposal, items } => {
                Ask::Effect(connector::Ask::KeepProposalSettings { proposal, items })
            }
            settings::Ask::DropProposal { proposal } => Ask::Effect(connector::Ask::DropProposalSettings { proposal }),
            settings::Ask::ForgetReplaced { before } => Ask::Effect(connector::Ask::ForgetReplaced { before }),
            settings::Ask::SaveDefaults { revision } => Ask::Effect(connector::Ask::SaveSettingsDefaults { revision }),
        },
        contract::Ask::Requests(ask) => match ask {
            requests::Ask::Decide { request, token, owner, person, scope, role, retention } => {
                Ask::Application { request, token, owner, person, scope, role, retention }
            }
            requests::Ask::Drop { owner } => Ask::DropApplication { owner },
            requests::Ask::PartAnswered { request, outcome } => Ask::ApplicationPartAnswered { request, outcome },
        },
    };
    Request::Ask { connector, ask }
}

/// Route a released output through a root that still speaks the core vocabulary.
#[must_use]
pub fn bridge_released(Connector(connector): Connector, released: Released) -> Request {
    let held = match released {
        Released::Make { entry } => Held::MakeEffect { connector, entry },
        Released::Answered { key, outcome } => Held::ConnectorActed { connector, key, outcome },
    };
    Request::Held(Box::new(held))
}
