//! Whole genuine owner facts, with no inferred lead, revision, proposal or load.
use jig_ops_client_domain::observability::{documents, values};
use jig_ops_domain_observability as source;
use skein_lib::List;
#[must_use]
pub fn service(value: source::Service) -> documents::Service {
    documents::Service { environment: value.environment, name: value.name }
}
#[must_use]
pub fn fact(value: source::Fact) -> values::Fact {
    values::Fact {
        observed: value.observed,
        healthy_replicas: value.healthy_replicas,
        errors: value.errors,
        error_rate_percent: value.error_rate_percent,
        load_percent: value.load_percent,
        load: value
            .load
            .into_vec()
            .into_iter()
            .map(|value| values::LoadPoint { observed: value.at, load_percent: value.percent })
            .collect::<Vec<_>>()
            .into(),
    }
}
#[must_use]
pub fn state(value: source::IncidentState) -> documents::State {
    match value {
        source::IncidentState::Alerting => documents::State::Alerting,
        source::IncidentState::Resolved => documents::State::Resolved,
    }
}
#[must_use]
pub fn incident(value: source::Incident) -> values::Incident {
    let mut timeline = List::with_capacity(value.timeline.capacity());
    for value in &value.timeline {
        timeline
            .push(values::Point {
                at: value.at,
                health: value.health.map(|health| values::Health {
                    observed: health.observed,
                    healthy_replicas: health.healthy_replicas,
                    errors: health.errors,
                    error_rate_percent: health.error_rate_percent,
                }),
            })
            .expect("source timeline capacity");
    }
    values::Incident {
        project: value.project,
        number: value.number,
        watch: value.watch,
        lead: value.lead,
        alerts: value.alerts,
        raised: value.raised,
        service: service(value.service),
        severity: value.severity,
        state: state(value.state),
        timeline,
        cut: value.cut,
    }
}
#[must_use]
pub fn purpose(value: source::Purpose) -> values::Purpose {
    match value {
        source::Purpose::Call { attempt, completion, position } => {
            values::Purpose::Call { attempt, completion, position }
        }
        source::Purpose::Procedure { purpose } => values::Purpose::Procedure { purpose },
        source::Purpose::Projection { purpose } => values::Purpose::Projection { purpose },
    }
}
#[must_use]
pub fn entry(value: source::Entry) -> values::Entry {
    values::Entry {
        project: value.project,
        number: value.number,
        key: values::Key {
            deployment: value.key.deployment,
            task: value.key.task,
            origin: purpose(value.key.origin),
            purpose: value.key.purpose,
        },
        effect: values::Effect { rule: value.effect.rule, until: value.effect.until },
        attempt: value.attempt,
        deadline: value.deadline,
        phase: match value.phase {
            source::Phase::Kept => values::Phase::Kept,
            source::Phase::Sent => values::Phase::Sent,
            source::Phase::Uncertain => values::Phase::Uncertain,
            source::Phase::Held => values::Phase::Held,
        },
    }
}
#[must_use]
pub fn document(value: source::ClientDocument) -> values::Value {
    match value {
        source::ClientDocument::Service { service: value, fact: current } => {
            values::Value::Service { service: service(value), fact: fact(current) }
        }
        source::ClientDocument::Incident { incident: value, fact: current } => {
            values::Value::Incident { incident: incident(value), fact: current.map(fact) }
        }
        source::ClientDocument::Alert { alert, raised, state: current, fact: health } => values::Value::Alert {
            alert: values::Alert { number: alert.number, service: service(alert.service), severity: alert.severity },
            raised,
            state: state(current),
            fact: health.map(fact),
        },
        source::ClientDocument::Outbox { entry: value } => values::Value::Outbox { entry: entry(value) },
        source::ClientDocument::Project { project, services, incidents } => {
            let mut views = List::with_capacity(services.capacity());
            for value in &services {
                views
                    .push(values::ServiceState {
                        service: service(value.service.clone()),
                        fact: value.fact.clone().map(fact),
                    })
                    .expect("source service capacity");
            }
            let mut history = List::with_capacity(incidents.capacity());
            for value in &incidents {
                history.push(incident(value.clone())).expect("source incident capacity");
            }
            values::Value::Project { project, services: views, incidents: history }
        }
        source::ClientDocument::Deployment { watches, effects, read_limit, incident_limit } => {
            let mut entries = List::with_capacity(effects.capacity());
            for value in &effects {
                entries.push(entry(value.clone())).expect("source outbox capacity");
            }
            values::Value::Deployment { watches, effects: entries, read_limit, incident_limit }
        }
    }
}
