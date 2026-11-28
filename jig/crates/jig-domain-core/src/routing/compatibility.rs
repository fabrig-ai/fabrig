//! The connector answers still accepted at the core entrance
//! (jig/docs/design/domain/engine/contracts.md, 3).

use super::{Event, Work};
use crate::effects::connector::Event as EffectEvent;
use jig_domain_contract::{
    Up, effects, lifecycle, parts, procedures, projections, requests, resources, settings, topics,
};
use jig_domain_tasks as tasks;
use jig_domain_views as views;

#[expect(clippy::wildcard_enum_match_arm, reason = "all remaining core events keep their core routes")]
pub(super) fn connector(event: Event) -> Work {
    match event {
        Event::EffectConnector(event) => effect(event),
        Event::ProjectReleased { project, connector } => {
            Work::Connector(Some(connector), Up::Lifecycle(lifecycle::Up::Released { project }))
        }
        Event::Holdings { request, holdings, refused, connector } => {
            Work::Connector(Some(connector), Up::Resources(resources::Up::Holdings { request, holdings, refused }))
        }
        Event::DelegateHoldings { request, holdings, refused, connector } => Work::Connector(
            Some(connector),
            Up::Resources(resources::Up::DelegateHoldings { request, holdings, refused }),
        ),
        Event::ProcedureHoldings { task, step, holdings, refused, connector } => Work::Connector(
            Some(connector),
            Up::Resources(resources::Up::ProcedureHoldings { task, step, holdings, refused }),
        ),
        Event::Tasks(tasks::Event::ReadAfresh { resource }) => {
            Work::Connector(Some(resource.connector), Up::Resources(resources::Up::ReadAfresh { resource }))
        }
        Event::Tasks(tasks::Event::Settled { task }) => {
            Work::Connector(None, Up::Effects(effects::Up::Released { task }))
        }
        Event::Tasks(tasks::Event::WakeProcedure { task }) => {
            Work::Connector(None, Up::Procedures(procedures::Up::Wake { task }))
        }
        Event::EffectStart {
            owner,
            connector,
            origin: origin @ (effects::EffectOrigin::Procedure { .. } | effects::EffectOrigin::Projection { .. }),
        } => Work::Connector(Some(connector), Up::Effects(effects::Up::Start { owner, origin })),
        Event::View(views::Event::Made { watcher, outcome }) => {
            Work::Connector(None, Up::Parts(parts::Up::Made { watcher, outcome }))
        }
        Event::Parts { token, parts, connector } => {
            Work::Connector(Some(connector), Up::Parts(parts::Up::Lines { token, parts }))
        }
        Event::PublicAnswered { reply, answer } => {
            Work::Connector(None, Up::Parts(parts::Up::Public { reply, answer }))
        }
        Event::PartsMarked { tasks, connector } => {
            Work::Connector(Some(connector), Up::Parts(parts::Up::Marked { tasks }))
        }
        Event::ProcedureStep { task, step, code, decision, connector } => {
            Work::Connector(Some(connector), Up::Procedures(procedures::Up::Step { task, step, code, decision }))
        }
        Event::Waits { task, waits, connector } => {
            Work::Connector(Some(connector), Up::Procedures(procedures::Up::Waits { task, waits }))
        }
        Event::ProjectionSettled { goal, connector } => {
            Work::Connector(Some(connector), Up::Projections(projections::Up::Settled { goal }))
        }
        Event::ApplicationFinished { request, outcome } => {
            Work::Connector(None, Up::Requests(requests::Up::Decided { request, outcome }))
        }
        Event::Marked { subjects, connector } => {
            Work::Connector(Some(connector), Up::Requests(requests::Up::Marked { subjects }))
        }
        other => Work::Event(other),
    }
}

fn effect(event: EffectEvent) -> Work {
    let (connector, up) = match event {
        EffectEvent::Adopted { project, name, role, home } => {
            (Some(name.connector), Up::Lifecycle(lifecycle::Up::Adopted { project, name, role, home }))
        }
        EffectEvent::Resource { project, name, role, hold } => {
            (Some(name.connector), Up::Resources(resources::Up::Resource { project, name, role, hold }))
        }
        EffectEvent::PoolSlots { name, slots } => {
            (Some(name.connector), Up::Resources(resources::Up::PoolSlots { name, slots }))
        }
        EffectEvent::Drift { task, resource } => {
            (Some(resource.connector), Up::Resources(resources::Up::Drift { task, resource }))
        }
        EffectEvent::Described { owner, description } => {
            (Some(description.connector), Up::Effects(effects::Up::Described { owner, description }))
        }
        EffectEvent::DescribeRefused { owner } => (None, Up::Effects(effects::Up::Refused { owner })),
        EffectEvent::DescribeBusy { owner } => (None, Up::Effects(effects::Up::Busy { owner })),
        EffectEvent::Verdict { owner, judge, verdict, at, guarded, state } => {
            (Some(judge.connector), Up::Effects(effects::Up::Verdict { owner, judge, verdict, at, guarded, state }))
        }
        EffectEvent::Outbox { entry, task, outcome, connector } => {
            (Some(connector), Up::Effects(effects::Up::Outbox { entry, task, outcome }))
        }
        EffectEvent::Closed { task, connector } => (Some(connector), Up::Effects(effects::Up::Closed { task })),
        EffectEvent::News { task, subscription, topic, class, words } => {
            (None, Up::Topics(topics::Up::News { task, subscription, topic, class, words }))
        }
        EffectEvent::ProjectionOutbox { entry, outcome, connector } => {
            (Some(connector), Up::Projections(projections::Up::Outbox { entry, outcome }))
        }
        EffectEvent::SettingsJudged { owner, judged } => (None, Up::Settings(settings::Up::Judged { owner, judged })),
        EffectEvent::SettingsRestored { owner, judged } => {
            (None, Up::Settings(settings::Up::Restored { owner, judged }))
        }
        EffectEvent::SettingsDefaults { connector, defaults, dropped } => {
            (Some(connector), Up::Settings(settings::Up::Defaults { defaults, dropped }))
        }
        // The old event has no source. Roots report the source by calling catch_up_done.
        EffectEvent::RestartDone { stage } => (None, Up::Lifecycle(lifecycle::Up::Done { stage })),
        EffectEvent::Procedure { .. } | EffectEvent::SectionReady { .. } | EffectEvent::WorkspaceReady { .. } => {
            return Work::Event(Event::EffectConnector(event));
        }
    };
    Work::Connector(connector, up)
}
