//! Conversion at the engine's connectors' port (jig/docs/design/domain/engine/contracts.md, 1 and 3).

use alloc::boxed::Box;
use jig_domain_contract::{
    self as contract, Connector, Released, effects, lifecycle, parts, procedures, projections, requests, resources,
    settings, topics,
};

use crate::{Ask, Event, Held, Request, effects::connector};
use jig_domain_tasks as tasks;
use jig_domain_views as views;

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
            resources::Ask::Lost { task, attempt } => Ask::Lost { task, attempt },
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
            effects::Ask::Keep { owner, entry, task, key } => {
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
            topics::Ask::Subscribed { request, key, subscription } => {
                Ask::SubscriptionDone { request, key, subscription }
            }
            topics::Ask::Unsubscribed { request, key } => Ask::UnsubscriptionDone { request, key },
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
            procedures::Ask::Start { context, step } => Ask::StartProcedure { context, step },
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

#[expect(clippy::result_large_err, reason = "uncased asks pass through by value")]
pub(super) fn ask(ask: Ask) -> Result<contract::Ask, Ask> {
    let port = match ask {
        Ask::Adopt { request, project, adoption } => {
            contract::Ask::Lifecycle(lifecycle::Ask::Adopt { request, project, adoption })
        }
        Ask::ReleaseProject { project } => contract::Ask::Lifecycle(lifecycle::Ask::ReleaseProject { project }),
        Ask::Hold { task, resource, entry } => contract::Ask::Resources(resources::Ask::Hold { task, resource, entry }),
        Ask::Lost { task, attempt } => contract::Ask::Resources(resources::Ask::Lost { task, attempt }),
        Ask::TaskHoldings { request, project, root, number, executor, spec, tracked, symbols } => {
            contract::Ask::Resources(resources::Ask::Holdings {
                request,
                project,
                root,
                number,
                executor,
                spec,
                tracked,
                symbols,
            })
        }
        Ask::DelegateHoldings { request, from, members } => {
            contract::Ask::Resources(resources::Ask::DelegateHoldings { request, from, members })
        }
        Ask::ProcedureHoldings { task, step, members } => {
            contract::Ask::Resources(resources::Ask::ProcedureHoldings { task, step, members })
        }
        Ask::RepairRefused { repair, number, why } => {
            contract::Ask::Resources(resources::Ask::RepairRefused { repair, number, why })
        }
        Ask::DelegateRefused { task } => contract::Ask::Resources(resources::Ask::DelegateRefused { task }),
        Ask::Close { task, root, ending } => contract::Ask::Effects(effects::Ask::Close { task, root, ending }),
        Ask::Release { task, root, ending, entry } => {
            contract::Ask::Effects(effects::Ask::Release { task, root, ending, entry })
        }
        Ask::SubscriptionDone { request, key, subscription } => {
            contract::Ask::Topics(topics::Ask::Subscribed { request, key, subscription })
        }
        Ask::UnsubscriptionDone { request, key } => contract::Ask::Topics(topics::Ask::Unsubscribed { request, key }),
        Ask::DropSubscription { request } => contract::Ask::Topics(topics::Ask::Refused { request }),
        Ask::EndTopic { task, subscription } => contract::Ask::Topics(topics::Ask::Ended { task, subscription }),
        Ask::Document { watcher, scope, subject, party, window } => {
            contract::Ask::Parts(parts::Ask::Document { watcher, scope, subject, party, window })
        }
        Ask::DropDocument { subject } => contract::Ask::Parts(parts::Ask::DropDocument { subject }),
        Ask::Read { reply, scope, read, party, window } => {
            contract::Ask::Parts(parts::Ask::Read { reply, scope, read, party, window })
        }
        Ask::Parts(parts) => contract::Ask::Parts(parts::Ask::Lines(parts)),
        Ask::Public { reply, kind, version, project, task, public } => {
            contract::Ask::Parts(parts::Ask::Public { reply, kind, version, project, task, public })
        }
        Ask::StartProcedure { context, step } => contract::Ask::Procedures(procedures::Ask::Start { context, step }),
        Ask::StepMade { task, step, first, revisions } => {
            contract::Ask::Procedures(procedures::Ask::StepMade { task, step, first, revisions })
        }
        Ask::StepRefused { task, step, why } => {
            contract::Ask::Procedures(procedures::Ask::StepRefused { task, step, why })
        }
        Ask::WaitsRefused { task, why } => contract::Ask::Procedures(procedures::Ask::WaitsRefused { task, why }),
        Ask::ProjectGoal { feed } => contract::Ask::Projections(projections::Ask::Project { feed }),
        Ask::ForgetProjection { goal } => contract::Ask::Projections(projections::Ask::Forget { goal }),
        Ask::Application { request, token, owner, person, scope, role, retention } => {
            contract::Ask::Requests(requests::Ask::Decide { request, token, owner, person, scope, role, retention })
        }
        Ask::DropApplication { owner } => contract::Ask::Requests(requests::Ask::Drop { owner }),
        Ask::ApplicationPartAnswered { request, outcome } => {
            contract::Ask::Requests(requests::Ask::PartAnswered { request, outcome })
        }
        Ask::Effect(ask) => match effect_ask(ask) {
            Ok(ask) => ask,
            Err(ask) => return Err(Ask::Effect(ask)),
        },
        other @ (Ask::Workers(_)
        | Ask::Providers(_)
        | Ask::Catalogue { .. }
        | Ask::Gather { .. }
        | Ask::CutTo { .. }
        | Ask::Drop { .. }
        | Ask::ClaimDone { .. }
        | Ask::JobClaimed { .. }) => return Err(other),
    };
    Ok(port)
}

fn effect_ask(ask: connector::Ask) -> Result<contract::Ask, connector::Ask> {
    let port = match ask {
        connector::Ask::CatchUp => contract::Ask::Lifecycle(lifecycle::Ask::CatchUp),
        connector::Ask::Unname { task } => contract::Ask::Resources(resources::Ask::Unname { task }),
        connector::Ask::Describe { owner } => contract::Ask::Effects(effects::Ask::Describe { owner }),
        connector::Ask::DescribeProposal { owner, proposal } => {
            contract::Ask::Effects(effects::Ask::DescribeProposal { owner, proposal })
        }
        connector::Ask::Judge { owner, judge, state, resources } => {
            contract::Ask::Effects(effects::Ask::Judge { owner, judge, state, resources })
        }
        connector::Ask::Keep { owner, entry, task, key } => {
            contract::Ask::Effects(effects::Ask::Keep { owner, entry, task, key })
        }
        connector::Ask::KeepProposal { owner, proposal, task, project } => {
            contract::Ask::Effects(effects::Ask::KeepProposal { owner, proposal, task, project })
        }
        connector::Ask::Drop { owner, answer, findings, why } => {
            contract::Ask::Effects(effects::Ask::Drop { owner, answer, findings, why })
        }
        connector::Ask::DropProposal { proposal } => contract::Ask::Effects(effects::Ask::DropProposal { proposal }),
        connector::Ask::ResolveUncertain { entry, again } => {
            contract::Ask::Effects(effects::Ask::Resolve { entry, again })
        }
        connector::Ask::JudgeSettings { owner, revision, proposal, items } => {
            contract::Ask::Settings(settings::Ask::Judge { owner, revision, proposal, items })
        }
        connector::Ask::RestoreSettings { owner, proposal, items } => {
            contract::Ask::Settings(settings::Ask::Restore { owner, proposal, items })
        }
        connector::Ask::KeepSettings { revision, proposal, items } => {
            contract::Ask::Settings(settings::Ask::Keep { revision, proposal, items })
        }
        connector::Ask::KeepProposalSettings { proposal, items } => {
            contract::Ask::Settings(settings::Ask::KeepProposal { proposal, items })
        }
        connector::Ask::DropProposalSettings { proposal } => {
            contract::Ask::Settings(settings::Ask::DropProposal { proposal })
        }
        connector::Ask::ForgetReplaced { before } => contract::Ask::Settings(settings::Ask::ForgetReplaced { before }),
        connector::Ask::SaveSettingsDefaults { revision } => {
            contract::Ask::Settings(settings::Ask::SaveDefaults { revision })
        }
        other @ (connector::Ask::Names { .. }
        | connector::Ask::Hold { .. }
        | connector::Ask::ReleaseHold { .. }
        | connector::Ask::Writer { .. }
        | connector::Ask::Make { .. }
        | connector::Ask::ProcedureMade { .. }
        | connector::Ask::ProcedureActivate { .. }
        | connector::Ask::ProcedureMessage { .. }
        | connector::Ask::ProcedureClose { .. }
        | connector::Ask::Project { .. }
        | connector::Ask::Release { .. }
        | connector::Ask::Subscribe { .. }
        | connector::Ask::Unsubscribe { .. }
        | connector::Ask::Read { .. }
        | connector::Ask::Gather { .. }
        | connector::Ask::CutTo { .. }
        | connector::Ask::Take { .. }
        | connector::Ask::Prepare { .. }
        | connector::Ask::Left { .. }
        | connector::Ask::Adopt { .. }
        | connector::Ask::Restore { .. }) => return Err(other),
    };
    Ok(port)
}

/// Connector-specific entrances alongside the cases that use the core's step.
#[expect(clippy::large_enum_variant, reason = "move the core event straight to its step without allocating")]
pub(super) enum Input {
    Event(Event),
    CatchUp(lifecycle::RestartStage),
    Act { key: [u8; 16], act: Box<contract::ops::Act>, carried: Option<contract::ops::Carried> },
}

pub(super) fn input(Connector(connector): Connector, up: contract::Up) -> Input {
    let event = match up {
        contract::Up::Lifecycle(up) => match up {
            lifecycle::Up::Done { stage } => return Input::CatchUp(stage),
            lifecycle::Up::Released { project } => Event::ProjectReleased { project, connector },
            lifecycle::Up::Adopted { project, name, role, home } => {
                Event::EffectConnector(connector::Event::Adopted { project, name, role, home })
            }
        },
        contract::Up::Resources(up) => match up {
            resources::Up::Holdings { request, holdings, refused } => {
                Event::Holdings { request, holdings, refused, connector }
            }
            resources::Up::DelegateHoldings { request, holdings, refused } => {
                Event::DelegateHoldings { request, holdings, refused, connector }
            }
            resources::Up::ProcedureHoldings { task, step, holdings, refused } => {
                Event::ProcedureHoldings { task, step, holdings, refused, connector }
            }
            resources::Up::Resource { project, name, role, hold } => {
                Event::EffectConnector(connector::Event::Resource { project, name, role, hold })
            }
            resources::Up::PoolSlots { name, slots } => {
                Event::EffectConnector(connector::Event::PoolSlots { name, slots })
            }
            resources::Up::Drift { task, resource } => {
                Event::EffectConnector(connector::Event::Drift { task, resource })
            }
            resources::Up::ReadAfresh { resource } => Event::Tasks(tasks::Event::ReadAfresh { resource }),
        },
        contract::Up::Effects(up) => match up {
            effects::Up::Described { owner, description } => {
                Event::EffectConnector(connector::Event::Described { owner, description })
            }
            effects::Up::Refused { owner } => Event::EffectConnector(connector::Event::DescribeRefused { owner }),
            effects::Up::Busy { owner } => Event::EffectConnector(connector::Event::DescribeBusy { owner }),
            effects::Up::Verdict { owner, judge, verdict, at, guarded, state } => {
                Event::EffectConnector(connector::Event::Verdict { owner, judge, verdict, at, guarded, state })
            }
            effects::Up::Outbox { entry, task, outcome } => {
                Event::EffectConnector(connector::Event::Outbox { entry, task, outcome, connector })
            }
            effects::Up::Closed { task } => Event::EffectConnector(connector::Event::Closed { task, connector }),
            effects::Up::Released { task } => Event::Tasks(tasks::Event::Settled { task }),
            effects::Up::Start { owner, origin } => Event::EffectStart { owner, origin, connector },
        },
        contract::Up::Topics(up) => match up {
            topics::Up::News { task, subscription, topic, class, words } => {
                Event::EffectConnector(connector::Event::News { task, subscription, topic, class, words })
            }
        },
        contract::Up::Parts(up) => match up {
            parts::Up::Made { watcher, outcome } => Event::View(views::Event::Made { watcher, outcome }),
            parts::Up::Lines { token, parts } => Event::Parts { token, parts, connector },
            parts::Up::Public { reply, answer } => Event::PublicAnswered { reply, answer },
            parts::Up::Marked { tasks } => Event::PartsMarked { tasks, connector },
        },
        contract::Up::Procedures(up) => match up {
            procedures::Up::Step { task, step, code, decision } => {
                Event::ProcedureStep { task, step, code, decision, connector }
            }
            procedures::Up::Waits { task, waits } => Event::Waits { task, waits, connector },
            procedures::Up::Wake { task } => Event::Tasks(tasks::Event::WakeProcedure { task }),
        },
        contract::Up::Projections(up) => match up {
            projections::Up::Settled { goal } => Event::ProjectionSettled { goal, connector },
            projections::Up::Outbox { entry, outcome } => {
                Event::EffectConnector(connector::Event::ProjectionOutbox { entry, outcome, connector })
            }
        },
        contract::Up::Settings(up) => match up {
            settings::Up::Judged { owner, judged } => {
                Event::EffectConnector(connector::Event::SettingsJudged { owner, judged })
            }
            settings::Up::Restored { owner, judged } => {
                Event::EffectConnector(connector::Event::SettingsRestored { owner, judged })
            }
            settings::Up::Defaults { defaults, dropped } => {
                Event::EffectConnector(connector::Event::SettingsDefaults { defaults, dropped, connector })
            }
        },
        contract::Up::Requests(up) => match up {
            requests::Up::Decided { request, outcome } => Event::ApplicationFinished { request, outcome },
            requests::Up::Marked { subjects } => Event::Marked { subjects, connector },
        },
        contract::Up::Act { key, act, carried } => return Input::Act { key, act, carried },
    };
    Input::Event(event)
}
