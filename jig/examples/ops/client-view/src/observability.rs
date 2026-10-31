//! Keep: family pages, owner keys and jig object slots (root.md, 6–9).
//! Replace: incidents, health and observability'service supplied facts.
use crate::{
    bindings::{Bindings, Meaning},
    drawing,
    words::{Word, Words},
};
use jig_client::contract::{ObjectKey, ApplicationPart, Proposer};
use jig_client_view::Owner;
use jig_ops_client_domain::{
    Action,
    observability::{
        self, Value,
        documents::{Document, Kind, Scope, State, Subject},
    },
};
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Key, Role, Run, SlotRef};

use skein_lib::Token;
pub const NAV: SlotRef = SlotRef {
    owner: (Owner::Application {
        number: jig_ops_domain_connectors::number(jig_ops_domain_connectors::Connector::Observability),
    })
    .opaque(),
    token: Token::new(0),
};
pub fn navigation(builder: &mut Builder, project: u32, words: &Words, bindings: &mut Bindings) {
    link(builder, Word::Incidents, &[], observability::Address::Incidents { project, open: true }, words, bindings);
}
fn link(
    builder: &mut Builder,
    word: Word,
    holes: &[Hole],
    address: observability::Address,
    words: &Words,
    bindings: &mut Bindings,
) {
    builder.open(Role::Link);
    builder.name(&words.say(word, holes));
    words.text(builder, word, holes);
    let binding = bindings.insert(Meaning::Action(Action::Observability(observability::Act::Go { address })));
    builder.bind(binding);
    builder.link(binding);
    builder.close();
}
pub fn page(builder: &mut Builder, side: &observability::Side, words: &Words, bindings: &mut Bindings) {
    builder.open(Role::Heading { level: 1 });
    words.text(builder, Word::Incidents, &[]);
    builder.close();
    let Some(id) = side.page() else {
        words.status(builder, Word::Unknown, &[]);
        return;
    };
    let object = side.object(id).expect("page is named");
    builder.open(Role::Item);
    builder.key(key(&object.subject));
    match &object.value {
        Value::Loading => words.status(builder, Word::Loading, &[]),
        Value::Unknown => words.status(builder, Word::Unknown, &[]),
        Value::Known(document) => document_view(builder, document, words, bindings),
    }
    builder.close();
}
fn plan_slot(builder: &mut Builder, task: u64, words: &Words, bindings: &mut Bindings) {
    let section = jig_client_view::page::Section::Task { task, section: jig_client_view::address::TaskSection::Plan };
    let binding = bindings.insert(Meaning::Section(section));
    builder.slot(SlotRef { owner: Owner::Jig.opaque(), token: binding.token }, &words.say(Word::Loading, &[]));
}
fn jig_slot(builder: &mut Builder, key: ObjectKey, words: &Words, bindings: &mut Bindings) {
    let binding = bindings.insert(Meaning::Object(key));
    builder.slot(SlotRef { owner: Owner::Jig.opaque(), token: binding.token }, &words.say(Word::Loading, &[]));
}
fn show_service(builder: &mut Builder, service: &observability::documents::Service, words: &Words) {
    words.text(builder, Word::Service, &[Hole::Name(service.name.clone())]);
    words.text(builder, Word::InEnvironment, &[Hole::Name(service.environment.clone())]);
}
fn document_view(builder: &mut Builder, document: &Document, words: &Words, bindings: &mut Bindings) {
    let Document::Value(document) = document;
    show_document(builder, document, words, bindings);
}
pub fn part(builder: &mut Builder, side: &observability::Side, part: &ApplicationPart, words: &Words, bindings: &mut Bindings) {
    if !side.contains(part) {
        words.status(builder, Word::Unshowable, &[]);
        return;
    }
    match part {
        ApplicationPart::Connector { .. } => words.text(
            builder,
            Word::Connector,
            &[Hole::Number(u64::from(jig_ops_domain_connectors::number(
                jig_ops_domain_connectors::Connector::Observability,
            )))],
        ),
        ApplicationPart::Resource { path, .. } => {
            words.text(builder, Word::Resource, &[Hole::Name(skein_lib::bytes::copy_of(b""))]);
            for segment in path {
                jig_client_view::literal_segment::render(builder, segment);
            }
        }
        ApplicationPart::Kind { kind, .. } => words.text(builder, Word::Kind, &[Hole::Number(u64::from(*kind))]),
        ApplicationPart::Panel { .. }
        | ApplicationPart::Result { .. }
        | ApplicationPart::OwnerEffect { .. }
        | ApplicationPart::Effect { .. }
        | ApplicationPart::Outbox { .. }
        | ApplicationPart::Hold { .. }
        | ApplicationPart::Project { .. }
        | ApplicationPart::Deployment { .. } => match side.part(part) {
            Some(object) => match &object.value {
                Value::Loading => words.status(builder, Word::Loading, &[]),
                Value::Unknown => words.status(builder, Word::Unknown, &[]),
                Value::Known(document) => document_view(builder, document, words, bindings),
            },
            None => words.status(builder, Word::Unshowable, &[]),
        },
    }
}
/// Public family identity; routing tokens are never used as display identity.
#[must_use]
pub fn key(service: &Subject) -> Key {
    let value = match service.scope {
        Scope::Project(part) => u64::from(part),
        // Project identifiers are u32; this sentinel preserves every supplied project, including zero.
        Scope::Deployment => u64::MAX,
    };
    let (kind, builder) = match service.kind {
        Kind::Incidents { open } => (1, u64::from(open)),
        Kind::Incident(incident) => (2, incident),
        Kind::Panel(t) => (3, t),
        Kind::Result(t) => (4, t),
        Kind::Effect(part) => {
            // Keep the full maker identity; equal task and party numbers remain different effects.
            let (kind, maker) = match part.proposer {
                Proposer::Task(task) => (5, task),
                Proposer::Party(party) => (10, party),
            };
            return Key { owner: NAV.owner, kind, a: maker, b: part.number };
        }
        Kind::Outbox(entry) => (6, entry),
        Kind::Hold(t) => (7, t),
        Kind::Project(part) => (8, u64::from(part)),
        Kind::Deployment => (9, 0),
    };
    Key { owner: NAV.owner, kind, a: value, b: builder }
}

fn show_document(
    builder: &mut Builder,
    document: &observability::values::Document,
    words: &Words,
    bindings: &mut Bindings,
) {
    match &document.value {
        observability::values::Value::Service { service, fact } => {
            show_service(builder, service, words);
            show_fact(builder, fact, words);
        }
        observability::values::Value::Alert { alert, raised, state, fact } => {
            show_service(builder, &alert.service, words);
            words.text(builder, Word::Severity, &[Hole::Number(u64::from(alert.severity))]);
            words.text(builder, Word::Raised, &[Hole::Wall(*raised)]);
            words.status(
                builder,
                match state {
                    State::Alerting => Word::Open,
                    State::Resolved => Word::Resolved,
                },
                &[],
            );
            if let Some(fact) = fact {
                show_fact(builder, fact, words);
            }
        }
        observability::values::Value::Outbox { entry } => {
            words.text(builder, Word::Entry, &[Hole::Number(entry.number)]);
            words.text(builder, Word::Task, &[Hole::Number(entry.key.task)]);
            words.text(builder, Word::Name, &[Hole::Name(entry.effect.rule.clone())]);
            words.text(
                builder,
                Word::End,
                &[Hole::Wall(skein_lib::Wall::from_nanos(entry.effect.until.saturating_mul(1_000_000_000)))],
            );
        }
        observability::values::Value::Deployment { watches, effects, read_limit, incident_limit } => {
            words.text(builder, Word::Watches, &[Hole::Number(u64::from(*watches))]);
            words.text(builder, Word::ReadLimit, &[Hole::Number(u64::from(*read_limit))]);
            words.text(builder, Word::IncidentLimit, &[Hole::Number(u64::from(*incident_limit))]);
            for entry in effects {
                words.text(builder, Word::Entry, &[Hole::Number(entry.number)]);
                words.text(builder, Word::Name, &[Hole::Name(entry.effect.rule.clone())]);
            }
        }
        observability::values::Value::Incident { incident, fact } => {
            show_incident(builder, incident, words, bindings, true);
            if let Some(fact) = fact {
                drawing::rate(builder, words, incident, fact);
                show_fact(builder, fact, words);
            }
        }
        observability::values::Value::Project { incidents, services, .. } => {
            builder.open(Role::List);
            for incident in incidents {
                let visible = match document.subject.kind {
                    Kind::Incidents { open: true } => incident.state == State::Alerting,
                    Kind::Incidents { open: false } | Kind::Project(_) => true,
                    Kind::Incident(_)
                    | Kind::Panel(_)
                    | Kind::Result(_)
                    | Kind::Effect(_)
                    | Kind::Outbox(_)
                    | Kind::Hold(_)
                    | Kind::Deployment => false,
                };
                if visible {
                    builder.open(Role::Item);
                    link(
                        builder,
                        Word::Incident,
                        &[Hole::Number(incident.number)],
                        observability::Address::Incident { project: incident.project, incident: incident.number },
                        words,
                        bindings,
                    );
                    show_incident(builder, incident, words, bindings, false);
                    builder.close();
                }
            }
            builder.close();
            match document.subject.kind {
                Kind::Project(_) => {
                    for service in services {
                        show_service(builder, &service.service, words);
                        if let Some(fact) = &service.fact {
                            show_fact(builder, fact, words);
                        }
                    }
                }
                Kind::Incidents { .. }
                | Kind::Incident(_)
                | Kind::Panel(_)
                | Kind::Result(_)
                | Kind::Effect(_)
                | Kind::Outbox(_)
                | Kind::Hold(_)
                | Kind::Deployment => {}
            }
        }
    }
}
fn show_fact(builder: &mut Builder, fact: &observability::values::Fact, words: &Words) {
    words.text(builder, Word::Healthy, &[Hole::Number(u64::from(fact.healthy_replicas))]);
    words.text(builder, Word::Errors, &[Hole::Number(u64::from(fact.errors))]);
    words.text(builder, Word::Rate, &[Hole::Number(u64::from(fact.error_rate_percent))]);
    builder.text(Run::Plain, b" %");
    words.text(
        builder,
        Word::Observed,
        &[Hole::Wall(skein_lib::Wall::from_nanos(fact.observed.saturating_mul(1_000_000_000)))],
    );
}
fn show_incident(
    builder: &mut Builder,
    incident: &observability::values::Incident,
    words: &Words,
    bindings: &mut Bindings,
    slots: bool,
) {
    builder.open(Role::Region);
    builder.name(&words.say(Word::Incident, &[Hole::Number(incident.number)]));
    builder.open(Role::Heading { level: 2 });
    words.text(builder, Word::Incident, &[Hole::Number(incident.number)]);
    builder.close();
    show_service(builder, &incident.service, words);
    words.status(
        builder,
        match incident.state {
            State::Alerting => Word::Open,
            State::Resolved => Word::Resolved,
        },
        &[],
    );
    words.text(builder, Word::Severity, &[Hole::Number(u64::from(incident.severity))]);
    words.text(builder, Word::Raised, &[Hole::Wall(incident.raised)]);
    match incident.lead {
        Some(lead) => {
            words.text(builder, Word::Lead, &[Hole::Number(lead)]);
            if slots {
                jig_slot(builder, ObjectKey::Task(lead), words, bindings);
                plan_slot(builder, lead, words, bindings);
            }
        }
        None => words.status(builder, Word::NoLead, &[]),
    }
    builder.open(Role::List);
    builder.name(&words.say(Word::Timeline, &[]));
    for point in &incident.timeline {
        builder.open(Role::Item);
        words.text(builder, Word::At, &[Hole::Wall(point.at)]);
        if let Some(health) = point.health {
            words.text(builder, Word::Healthy, &[Hole::Number(u64::from(health.healthy_replicas))]);
            words.text(builder, Word::Errors, &[Hole::Number(u64::from(health.errors))]);
            words.text(builder, Word::Rate, &[Hole::Number(u64::from(health.error_rate_percent))]);
            words.text(
                builder,
                Word::Observed,
                &[Hole::Wall(skein_lib::Wall::from_nanos(health.observed.saturating_mul(1_000_000_000)))],
            );
        }
        builder.close();
    }
    if incident.cut {
        words.status(builder, Word::Cut, &[]);
    }
    builder.close();
    builder.close();
}
