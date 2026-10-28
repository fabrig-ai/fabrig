//! Root admission by variant and backlog (skein/docs/foundation/programming-model.md, 7.1–7.2).
use crate::{Domain, Event, Limits, step};
use jig_domain_host as host;
use jig_ops_domain_infrastructure as infrastructure;
use jig_ops_domain_observability as observability;
use skein_lib::Env;

/// How the root takes an event (jig/docs/design/domain/root.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Class {
    /// Finishes or advances admitted work, taken at any backlog.
    Admitted,
    /// New work, handed back unchanged at the watermark.
    New,
    /// Its admitted part is stepped; its new part waits at the watermark.
    Mixed,
}

/// Class an event by its variant and payload alone.
#[must_use]
pub fn class(event: &Event) -> Class {
    match event {
        Event::Core(event) => {
            if event.new_work() {
                Class::New
            } else {
                Class::Admitted
            }
        }
        Event::Timer(timer) => {
            if timer.new_work() {
                Class::New
            } else {
                Class::Admitted
            }
        }
        Event::Observability(event) => observability_class(event),
        Event::Infrastructure(event) => infrastructure_class(event),
        Event::Host(event) => host_class(event),
        Event::Agent(event) => match event {
            host::FromAgent::Called { .. } => Class::New,
            host::FromAgent::Withdrawn { .. }
            | host::FromAgent::Turn { .. }
            | host::FromAgent::Piece { .. }
            | host::FromAgent::Activity { .. }
            | host::FromAgent::Unauthorized { .. }
            | host::FromAgent::Limited { .. }
            | host::FromAgent::AccountRefused { .. }
            | host::FromAgent::Finished { .. }
            | host::FromAgent::Started { .. }
            | host::FromAgent::StartRefused { .. }
            | host::FromAgent::Refused { .. }
            | host::FromAgent::Yielded { .. }
            | host::FromAgent::Faulted { .. }
            | host::FromAgent::Gone { .. } => Class::Admitted,
        },
        Event::WatchApplication { .. }
        | Event::ReadApplication { .. }
        | Event::Application { .. }
        | Event::Restart { .. }
        | Event::Effect { .. } => Class::New,
        Event::AgentContent { .. }
        | Event::Store(..)
        | Event::Released(..)
        | Event::ObservabilityTimer
        | Event::InfrastructureTimer
        | Event::TurnTimer
        | Event::ResultRead { .. }
        | Event::ConnectorRead { .. }
        | Event::CallAnswered { .. } => Class::Admitted,
    }
}

fn host_class(event: &host::Event) -> Class {
    match event {
        host::Event::Called { .. } | host::Event::Opened { .. } => Class::New,
        host::Event::Down(event) => match event {
            host::LinkDown::Sources { .. }
            | host::LinkDown::Check { .. }
            | host::LinkDown::SignOut { .. }
            | host::LinkDown::Assign { .. }
            | host::LinkDown::Message { .. } => Class::New,
            host::LinkDown::Relayed { .. }
            | host::LinkDown::TurnKept { .. }
            | host::LinkDown::Acknowledge { .. }
            | host::LinkDown::Cancel { .. }
            | host::LinkDown::Grant { .. } => Class::Admitted,
        },
        host::Event::SourceAnswered { .. }
        | host::Event::SourceLost
        | host::Event::Turn { .. }
        | host::Event::Withdrawn { .. }
        | host::Event::Piece { .. }
        | host::Event::Activity { .. }
        | host::Event::Unauthorized { .. }
        | host::Event::Limited { .. }
        | host::Event::AccountRefused { .. }
        | host::Event::Finished { .. }
        | host::Event::RelayCancelled { .. }
        | host::Event::CancelAll { .. }
        | host::Event::Prepared { .. }
        | host::Event::Unprepared { .. }
        | host::Event::Started { .. }
        | host::Event::StartRefused { .. }
        | host::Event::Refused { .. }
        | host::Event::Yielded { .. }
        | host::Event::Faulted { .. }
        | host::Event::Gone { .. }
        | host::Event::Delivered { .. }
        | host::Event::Saved { .. } => Class::Admitted,
    }
}

fn observability_class(event: &observability::Event) -> Class {
    use observability::{Event as O, SystemEvent as S};
    match event {
        O::System(event) => match event {
            S::Fact { .. } => Class::Mixed,
            S::HealthChanged { .. } | S::Alert { own: false, .. } => Class::New,
            S::FactSettle { .. }
            | S::Alert { own: true, .. }
            | S::ReadDone { .. }
            | S::Applied { .. }
            | S::Found { .. } => Class::Admitted,
        },
        O::Subscribe { .. } | O::Describe { .. } => Class::New,
        O::DocumentPrepare { .. }
        | O::DocumentForget { .. }
        | O::Document { .. }
        | O::ReadDocument { .. }
        | O::Close { .. }
        | O::Release { .. }
        | O::Judge { .. }
        | O::Read { .. }
        | O::LinkWatch { .. }
        | O::Unsubscribe { .. }
        | O::StartWatch { .. }
        | O::StopWatch { .. }
        | O::WakeWatch { .. }
        | O::WatchStep { .. }
        | O::ProcedureDelegateOutcome { .. }
        | O::Keep { .. }
        | O::Drop { .. }
        | O::Make { .. }
        | O::MakeEntry { .. }
        | O::Restore { .. }
        | O::ReadAfresh
        | O::Restart => Class::Admitted,
    }
}

fn infrastructure_class(event: &infrastructure::Event) -> Class {
    use infrastructure::{Event as I, SystemEvent as S};
    match event {
        I::System(event) => match event {
            S::Service { other_hand: true, .. } | S::Environment { other_hand: true, fact: None, .. } => Class::Mixed,
            S::Drift { .. } | S::ServiceDrift { .. } | S::EnvironmentDrift { .. } => Class::New,
            S::Service { other_hand: false, .. }
            | S::Environment { other_hand: false, .. }
            | S::Environment { other_hand: true, fact: Some(_), .. }
            | S::ServiceSettle { .. }
            | S::EnvironmentSettle { .. }
            | S::Pool { .. }
            | S::Applied { .. }
            | S::Looked { .. } => Class::Admitted,
        },
        I::ApplicationPrepare { .. } | I::Describe { .. } => Class::New,
        I::Closed { .. }
        | I::SendCloses
        | I::ProcedureDelegateOutcome { .. }
        | I::DocumentPrepare { .. }
        | I::DocumentForget { .. }
        | I::Document { .. }
        | I::ReadDocument { .. }
        | I::ApplicationDrop { .. }
        | I::Application { .. }
        | I::Close { .. }
        | I::Release { .. }
        | I::Names { .. }
        | I::Unnamed { .. }
        | I::StartProcedure { .. }
        | I::Procedure { .. }
        | I::DescribeProposal { .. }
        | I::KeepProposal { .. }
        | I::DropProposal { .. }
        | I::Keep { .. }
        | I::Drop { .. }
        | I::Make { .. }
        | I::MakeEntry { .. }
        | I::ResolveEntry { .. }
        | I::Restore { .. }
        | I::ReadAfresh
        | I::Restart => Class::Admitted,
    }
}

/// Take admitted work at any backlog; return only new work at the watermark.
#[expect(clippy::result_large_err, reason = "the admission boundary returns the owned event")]
pub fn try_step(domain: &mut Domain, env: &Env<Limits>, event: Event) -> Result<(), Event> {
    match class(&event) {
        Class::Admitted => {
            step(domain, env, event);
            Ok(())
        }
        Class::New => {
            if !admits(domain, &env.limits) {
                return Err(event);
            }
            step(domain, env, event);
            Ok(())
        }
        Class::Mixed => {
            if admits(domain, &env.limits) {
                step(domain, env, event);
                return Ok(());
            }
            let (admitted, rest) = split(domain, env, event);
            step(domain, env, admitted);
            match rest {
                Some(rest) => Err(rest),
                None => Ok(()),
            }
        }
    }
}

#[expect(clippy::manual_map, reason = "step code wraps owned remainders without closures")]
fn split(domain: &Domain, env: &Env<Limits>, event: Event) -> (Event, Option<Event>) {
    match event {
        Event::Observability(observability::Event::System(system)) => {
            let (settle, rest) = observability::split(
                &domain.observability,
                &Env { now: env.now, wall: env.wall, limits: env.limits.observability },
                system,
            )
            .expect("only a mixed system fact is split");
            let rest = match rest {
                Some(rest) => Some(Event::Observability(observability::Event::System(rest))),
                None => None,
            };
            (Event::Observability(observability::Event::System(settle)), rest)
        }
        Event::Infrastructure(infrastructure::Event::System(system)) => {
            let (settle, rest) = infrastructure::split(
                &domain.infrastructure,
                &Env { now: env.now, wall: env.wall, limits: env.limits.infrastructure },
                system,
            )
            .expect("only a mixed system fact is split");
            let rest = match rest {
                Some(rest) => Some(Event::Infrastructure(infrastructure::Event::System(rest))),
                None => None,
            };
            (Event::Infrastructure(infrastructure::Event::System(settle)), rest)
        }
        Event::WatchApplication { .. }
        | Event::ReadApplication { .. }
        | Event::Application { .. }
        | Event::Core(..)
        | Event::Observability(..)
        | Event::Infrastructure(..)
        | Event::Host(..)
        | Event::Agent(..)
        | Event::AgentContent { .. }
        | Event::Store(..)
        | Event::Released(..)
        | Event::Restart { .. }
        | Event::Timer(..)
        | Event::ObservabilityTimer
        | Event::InfrastructureTimer
        | Event::TurnTimer
        | Event::ResultRead { .. }
        | Event::ConnectorRead { .. }
        | Event::CallAnswered { .. }
        | Event::Effect { .. } => unreachable!("only a mixed fact is split"),
    }
}

/// What the root holds of admitted work, in entries (dynamic-memory.md, 4.1).
pub(crate) fn backlog(domain: &Domain) -> u64 {
    domain.journal.backlog()
}

/// New work enters while the journal runs and the backlog is under the watermark.
pub(crate) fn admits(domain: &Domain, limits: &Limits) -> bool {
    !domain.journal.stopped() && backlog(domain) < u64::from(limits.watermark)
}
