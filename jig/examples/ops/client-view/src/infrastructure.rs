//! Keep: supplied owner facts and revision-bound forms (root.md, 6–9).
//! Replace: environments, procedure panels, effects and drift.
use crate::{
    bindings::{Bindings, Meaning},
    words::{Word, Words},
};
use jig_client::contract::{Part, Proposer};
use jig_client_view::Owner;
use jig_ops_client_domain::{
    Action,
    infrastructure::{
        self, Card, Value,
        documents::{Answer, Document, Kind, Scope, Subject},
    },
};
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Key, Role, SlotRef, State};

use skein_lib::{Decimal, Id, Token};
pub const NAV: SlotRef = SlotRef {
    owner: (Owner::Application {
        number: jig_ops_domain_connectors::number(jig_ops_domain_connectors::Connector::Infrastructure),
    })
    .opaque(),
    token: Token::new(0),
};
pub fn navigation(builder: &mut Builder, project: u32, words: &Words, bindings: &mut Bindings) {
    link(
        builder,
        Word::Environments,
        &[],
        infrastructure::Address::NativeEnvironments { project, pool: None },
        words,
        bindings,
    );
}
fn link(
    builder: &mut Builder,
    word: Word,
    holes: &[Hole],
    address: infrastructure::Address,
    words: &Words,
    bindings: &mut Bindings,
) {
    builder.open(Role::Link);
    builder.name(&words.say(word, holes));
    words.text(builder, word, holes);
    let binding = bindings.insert(Meaning::Action(Action::Infrastructure(infrastructure::Act::Go { address })));
    builder.link(binding);
    builder.bind(binding);
    builder.close();
}
pub fn page(
    builder: &mut Builder,
    side: &infrastructure::Side,
    words: &Words,
    jig_words: &jig_client_view::words::Words,
    bindings: &mut Bindings,
) {
    builder.open(Role::Heading { level: 1 });
    words.text(builder, Word::Environments, &[]);
    builder.close();
    let Some(id) = side.page() else {
        words.status(builder, Word::Unknown, &[]);
        return;
    };
    show_object(builder, side, id, words, jig_words, bindings);
}
fn show_object(
    builder: &mut Builder,
    side: &infrastructure::Side,
    id: Id<infrastructure::Object>,
    words: &Words,
    jig_words: &jig_client_view::words::Words,
    bindings: &mut Bindings,
) {
    let object = side.object(id).expect("named object");
    builder.open(Role::Item);
    builder.key(key(&object.subject).unwrap_or(Key { owner: NAV.owner, kind: 12, a: 0, b: id.token().raw() }));
    match &object.value {
        Value::Loading => words.status(builder, Word::Loading, &[]),
        Value::Unknown => words.status(builder, Word::Unknown, &[]),
        Value::Known(document) => {
            document_view(builder, document, words, bindings);
            match document.as_ref() {
                Document::Native(document) => match &document.value {
                    infrastructure::native::Value::Environment { end_revision, end, fact, .. } => {
                        let until = match end {
                            Some(end) => end.until,
                            None => fact.until,
                        };
                        native_form(
                            builder,
                            id,
                            jig_client::contract::Revision { value: *end_revision },
                            until,
                            object,
                            words,
                            bindings,
                        );
                    }
                    infrastructure::native::Value::Project { .. }
                    | infrastructure::native::Value::Panel { .. }
                    | infrastructure::native::Value::Result { .. }
                    | infrastructure::native::Value::Effect { .. }
                    | infrastructure::native::Value::Outbox { .. }
                    | infrastructure::native::Value::Hold { .. }
                    | infrastructure::native::Value::Deployment { .. } => {}
                },
            }
        }
    }
    show_card(builder, &object.card, words, jig_words, bindings.text_limit());
    builder.close();
}
fn native_form(
    builder: &mut Builder,
    id: Id<infrastructure::Object>,
    revision: jig_client::contract::Revision,
    until: u64,
    object: &infrastructure::Object,
    words: &Words,
    bindings: &mut Bindings,
) {
    let disabled = match object.card {
        Card::Deciding { .. } | Card::Unknown => true,
        Card::Open | Card::Refused { .. } | Card::Done { .. } => false,
    };
    builder.open(Role::TextField);
    builder.name(&words.say(Word::Until, &[]));
    builder.value(Decimal::of(object.native_draft.unwrap_or(until)).as_bytes(), revision.value);
    if disabled {
        builder.state(State::Disabled);
    } else {
        builder.bind(bindings.insert(Meaning::NativeUntil { object: id.token(), revision }));
    }
    builder.close();
    builder.open(Role::Button);
    builder.name(&words.say(Word::MoveEnd, &[]));
    if disabled {
        builder.state(State::Disabled);
    } else {
        builder.bind(bindings.insert(Meaning::Action(Action::Infrastructure(infrastructure::Act::NativeMoveEnd {
            object: id.token(),
            revision,
        }))));
    }
    builder.close();
}
fn show_card(
    builder: &mut Builder,
    card: &Card,
    words: &Words,
    jig_words: &jig_client_view::words::Words,
    text_limit: u32,
) {
    match card {
        Card::Open => {}
        Card::Unknown => words.status(builder, Word::Unknown, &[]),
        Card::Deciding { progress, .. } => words.status(
            builder,
            match progress {
                skein_lib::RequestProgress::InFlight => Word::Deciding,
                skein_lib::RequestProgress::Retrying => Word::Retrying,
                skein_lib::RequestProgress::Parked => Word::Parked,
                skein_lib::RequestProgress::Unknown => Word::Unknown,
            },
            &[],
        ),
        Card::Done { answer } | Card::Refused { answer } => match answer {
            Answer::NativeMovedEnd { revision, until, .. } => {
                words.status(builder, Word::Done, &[]);
                words.text(builder, Word::Revision, &[Hole::Number(revision.value)]);
                words.text(builder, Word::EndSeconds, &[Hole::Number(*until)]);
            }
            Answer::NativeMoved { revision, .. } => {
                words.status(builder, Word::Moved, &[Hole::Number(revision.value)]);
            }
            Answer::Gone => words.status(builder, Word::Gone, &[]),
            Answer::EntranceRefused(refusal) => {
                jig_client_view::cards::refusal(builder, refusal.refusal(), jig_words, text_limit);
            }
            Answer::Refused { why } => {
                words.status(builder, Word::Refused, &[]);
                match why {
                    infrastructure::documents::Refusal::Role { held } => {
                        words.text(builder, Word::HeldRole, &[Hole::Number(u64::from(*held))]);
                    }
                    infrastructure::documents::Refusal::Past => words.status(builder, Word::Past, &[]),
                    infrastructure::documents::Refusal::Exhausted => words.status(builder, Word::Exhausted, &[]),
                    infrastructure::documents::Refusal::SignedOut => words.status(builder, Word::SignedOut, &[]),
                    infrastructure::documents::Refusal::NotAllowed => words.status(builder, Word::NotAllowed, &[]),
                    infrastructure::documents::Refusal::Conflict => words.status(builder, Word::Conflict, &[]),
                    infrastructure::documents::Refusal::Invalid => words.status(builder, Word::Invalid, &[]),
                }
            }
        },
    }
}
fn deployment_diagnostics(builder: &mut Builder, subject: Subject, words: &Words) {
    match subject.kind {
        Kind::Deployment => {
            words.text(builder, Word::SystemRequestBudgetNotKept, &[]);
            words.text(builder, Word::LastSystemResponseNotKept, &[]);
        }
        Kind::NativeEnvironment(_)
        | Kind::NativeEnvironments { .. }
        | Kind::Panel(_)
        | Kind::Result(_)
        | Kind::Effect(_)
        | Kind::OwnerEffect(_)
        | Kind::Outbox(_)
        | Kind::Hold(_)
        | Kind::Project(_) => {}
    }
}
fn document_view(builder: &mut Builder, document: &Document, words: &Words, bindings: &mut Bindings) {
    let Document::Native(document) = document;
    native_document(builder, document, words, bindings);
}
pub fn part(
    builder: &mut Builder,
    side: &infrastructure::Side,
    part: &Part,
    words: &Words,
    jig_words: &jig_client_view::words::Words,
    bindings: &mut Bindings,
) {
    if !side.contains(part) {
        words.status(builder, Word::Unshowable, &[]);
        return;
    }
    match part {
        Part::Connector { .. } => words.text(
            builder,
            Word::Connector,
            &[Hole::Number(u64::from(jig_ops_domain_connectors::number(
                jig_ops_domain_connectors::Connector::Infrastructure,
            )))],
        ),
        Part::Resource { path, .. } => {
            words.text(builder, Word::Resource, &[Hole::Name(skein_lib::bytes::copy_of(b""))]);
            for segment in path {
                jig_client_view::literal_segment::render(builder, segment);
            }
        }
        Part::Kind { kind, .. } => words.text(builder, Word::Kind, &[Hole::Number(u64::from(*kind))]),
        Part::Panel { .. }
        | Part::Result { .. }
        | Part::OwnerEffect { .. }
        | Part::Effect { .. }
        | Part::Outbox { .. }
        | Part::Hold { .. }
        | Part::Project { .. }
        | Part::Deployment { .. } => match side.part(part) {
            Some(object) => {
                let id = side.find(object.subject.clone()).expect("part is named");
                show_object(builder, side, id, words, jig_words, bindings);
            }
            None => words.status(builder, Word::Unshowable, &[]),
        },
    }
}
#[must_use]
pub fn key(service: &Subject) -> Option<Key> {
    let value = match service.scope {
        Scope::Project(part) => u64::from(part),
        // Project identifiers are u32; this sentinel preserves every supplied project, including zero.
        Scope::Deployment => u64::MAX,
    };
    let (kind, builder) = match &service.kind {
        Kind::Panel(t) => (3, *t),
        Kind::Result(t) => (4, *t),
        Kind::Effect(part) => {
            // Keep the full maker identity; equal task and party numbers remain different effects.
            let (kind, maker) = match part.proposer {
                Proposer::Task(task) => (5, task),
                Proposer::Party(party) => (10, party),
            };
            return Some(Key { owner: NAV.owner, kind, a: maker, b: part.number });
        }
        Kind::Outbox(entry) => (6, *entry),
        Kind::OwnerEffect(proposal) => (11, *proposal),
        Kind::Hold(t) => (7, *t),
        Kind::Project(part) => (8, u64::from(*part)),
        Kind::Deployment => (9, 0),
        Kind::NativeEnvironment(_) | Kind::NativeEnvironments { .. } => return None,
    };
    Some(Key { owner: NAV.owner, kind, a: value, b: builder })
}
fn native_service(builder: &mut Builder, service: &infrastructure::native::Service, words: &Words) {
    words.text(builder, Word::Service, &[Hole::Name(service.name.clone())]);
    words.text(builder, Word::InEnvironment, &[Hole::Name(service.environment.clone())]);
}
fn native_environment(builder: &mut Builder, environment: &infrastructure::native::Environment, words: &Words) {
    words.text(builder, Word::Name, &[Hole::Name(environment.name.clone())]);
    words.text(builder, Word::Name, &[Hole::Name(environment.pool.clone())]);
}
fn native_resource(builder: &mut Builder, resource: &infrastructure::native::Resource, words: &Words) {
    use infrastructure::native::Resource as R;
    match resource {
        R::Service(service) => native_service(builder, service, words),
        R::Environment(environment) => native_environment(builder, environment, words),
        R::Pool(pool) | R::Quota(pool) => words.text(builder, Word::Name, &[Hole::Name(pool.0.clone())]),
    }
}
fn native_effect(builder: &mut Builder, effect: &infrastructure::native::Effect, words: &Words) {
    use infrastructure::native::Effect as E;
    match effect {
        E::Restart { service, operation } => {
            words.status(builder, Word::Restart, &[]);
            native_service(builder, service, words);
            words.text(builder, Word::Operation, &[Hole::Number(*operation)]);
        }
        E::Scale { service, from, to } => {
            words.status(builder, Word::Scale, &[]);
            native_service(builder, service, words);
            words.text(builder, Word::Replicas, &[Hole::Number(u64::from(*from))]);
            words.text(builder, Word::Replicas, &[Hole::Number(u64::from(*to))]);
        }
        E::Rollback { service, from, to } => {
            words.status(builder, Word::Rollback, &[]);
            native_service(builder, service, words);
            words.text(builder, Word::From, &[Hole::Name(from.clone())]);
            words.text(builder, Word::To, &[Hole::Name(to.clone())]);
        }
        E::CreateEnvironment { environment, until, price } => {
            words.status(builder, Word::Provision, &[]);
            native_environment(builder, environment, words);
            words.text(builder, Word::EndSeconds, &[Hole::Number(*until)]);
            words.text(builder, Word::Cost, &[Hole::Number(*price)]);
        }
        E::TearDown { environment, .. } => {
            words.status(builder, Word::TearDown, &[]);
            native_environment(builder, environment, words);
        }
    }
}
fn native_state(builder: &mut Builder, state: &infrastructure::native::ProcedureState, words: &Words) {
    use infrastructure::native::{Procedure as P, ProcedurePhase as Phase};
    words.text(builder, Word::Task, &[Hole::Number(state.task)]);
    match &state.procedure {
        P::Remediate { service, operation, deadline } => {
            words.status(builder, Word::Remediate, &[]);
            native_service(builder, service, words);
            words.text(builder, Word::Operation, &[Hole::Number(*operation)]);
            words.text(builder, Word::DeadlineSeconds, &[Hole::Number(*deadline)]);
        }
        P::Scale { service, replicas, deadline } => {
            words.status(builder, Word::Scale, &[]);
            native_service(builder, service, words);
            words.text(builder, Word::Replicas, &[Hole::Number(u64::from(*replicas))]);
            words.text(builder, Word::DeadlineSeconds, &[Hole::Number(*deadline)]);
        }
        P::Provision { environment, until, price, deadline } => {
            words.status(builder, Word::Provision, &[]);
            native_environment(builder, environment, words);
            words.text(builder, Word::EndSeconds, &[Hole::Number(*until)]);
            words.text(builder, Word::Cost, &[Hole::Number(*price)]);
            words.text(builder, Word::DeadlineSeconds, &[Hole::Number(*deadline)]);
        }
        P::TearDown { environment, deadline, .. } => {
            words.status(builder, Word::TearDown, &[]);
            native_environment(builder, environment, words);
            words.text(builder, Word::DeadlineSeconds, &[Hole::Number(*deadline)]);
        }
    }
    words.status(
        builder,
        match state.phase {
            Phase::Active => Word::Active,
            Phase::ReadingFacts => Word::ReadingFacts,
            Phase::FactsReady => Word::FactsReady,
            Phase::EffectOutstanding => match state.procedure {
                P::Remediate { .. } => Word::Restarting,
                P::Scale { .. } | P::Provision { .. } | P::TearDown { .. } => Word::EffectOutstanding,
            },
            Phase::WaitingCondition => match state.procedure {
                P::Remediate { .. } => Word::WaitingHealthy,
                P::Scale { .. } | P::Provision { .. } | P::TearDown { .. } => Word::WaitingCondition,
            },
            Phase::Holding => Word::Holding,
            Phase::Done => Word::Finished,
            Phase::Held => Word::Held,
        },
        &[],
    );
}
fn native_entry(builder: &mut Builder, entry: &infrastructure::native::Entry, words: &Words) {
    use infrastructure::native::Phase;
    words.text(builder, Word::Entry, &[Hole::Number(entry.number)]);
    words.text(builder, Word::Task, &[Hole::Number(entry.key.task)]);
    native_effect(builder, &entry.effect, words);
    words.status(
        builder,
        match entry.phase {
            Phase::Kept => Word::Kept,
            Phase::Sent => Word::Sent,
            Phase::Uncertain => Word::Uncertain,
            Phase::Held => Word::Held,
        },
        &[],
    );
    words.text(builder, Word::Attempt, &[Hole::Number(u64::from(entry.attempt))]);
    words.text(builder, Word::DeadlineSeconds, &[Hole::Number(entry.deadline)]);
}
fn native_resources(
    builder: &mut Builder,
    resources: &skein_lib::List<infrastructure::native::ClientResource>,
    words: &Words,
) {
    use infrastructure::native::{ResourceFact as R, ResourceRole};
    for resource in resources {
        native_resource(builder, &resource.named.resource, words);
        words.status(
            builder,
            match resource.named.role {
                ResourceRole::Owned => Word::Owned,
                ResourceRole::Participant => Word::Participating,
                ResourceRole::Context => Word::Context,
                ResourceRole::Unavailable => Word::ResourceUnavailable,
            },
            &[],
        );
        match &resource.fact {
            R::Service(fact) => {
                if let Some(fact) = fact {
                    words.text(builder, Word::Version, &[Hole::Name(fact.version.clone())]);
                    words.text(builder, Word::Replicas, &[Hole::Number(u64::from(fact.replicas))]);
                    words.status(
                        builder,
                        if fact.healthy { Word::ServiceHealthy } else { Word::ServiceUnhealthy },
                        &[],
                    );
                    words.text(builder, Word::Revision, &[Hole::Number(fact.revision)]);
                    words.text(builder, Word::ObservedSeconds, &[Hole::Number(fact.observed)]);
                }
            }
            R::Environment { fact, end } => {
                if let Some(fact) = fact {
                    words.text(builder, Word::ReadyAtSeconds, &[Hole::Number(fact.ready_at)]);
                    words.text(builder, Word::Cost, &[Hole::Number(fact.price)]);
                    words.text(builder, Word::ObservedEndSeconds, &[Hole::Number(fact.until)]);
                }
                if let Some(end) = end {
                    words.text(builder, Word::EndSeconds, &[Hole::Number(end.until)]);
                    words.text(builder, Word::Revision, &[Hole::Number(end.revision)]);
                }
            }
            R::Pool(pool) | R::Quota(pool) => {
                if let Some(pool) = pool {
                    words.text(builder, Word::Used, &[Hole::Number(u64::from(pool.used))]);
                    words.text(builder, Word::Quota, &[Hole::Number(u64::from(pool.quota))]);
                }
            }
        }
    }
}
fn native_environment_rows(
    builder: &mut Builder,
    document: &infrastructure::native::Document,
    project: u32,
    environments: &skein_lib::List<infrastructure::native::ClientEnvironment>,
    words: &Words,
    bindings: &mut Bindings,
) {
    for environment in environments {
        let visible = match &document.subject.kind {
            Kind::NativeEnvironments { pool } => match pool {
                Some(pool) => pool.0 == environment.environment.pool,
                None => true,
            },
            Kind::NativeEnvironment(_)
            | Kind::Panel(_)
            | Kind::Result(_)
            | Kind::Effect(_)
            | Kind::OwnerEffect(_)
            | Kind::Outbox(_)
            | Kind::Hold(_)
            | Kind::Project(_)
            | Kind::Deployment => true,
        };
        if visible {
            link(
                builder,
                Word::Name,
                &[Hole::Name(environment.environment.name.clone())],
                infrastructure::Address::NativeEnvironment { project, environment: environment.environment.clone() },
                words,
                bindings,
            );
            native_environment(builder, &environment.environment, words);
        }
    }
}
fn native_document(
    builder: &mut Builder,
    document: &infrastructure::native::Document,
    words: &Words,
    bindings: &mut Bindings,
) {
    use infrastructure::native::Value as V;
    deployment_diagnostics(builder, document.subject.clone(), words);
    match &document.value {
        V::Environment { environment, fact, end, pool, end_revision } => {
            native_environment(builder, environment, words);
            words.text(builder, Word::ReadyAtSeconds, &[Hole::Number(fact.ready_at)]);
            words.text(builder, Word::Cost, &[Hole::Number(fact.price)]);
            words.text(builder, Word::ObservedEndSeconds, &[Hole::Number(fact.until)]);
            words.text(builder, Word::Revision, &[Hole::Number(*end_revision)]);
            if let Some(end) = end {
                words.text(builder, Word::EndSeconds, &[Hole::Number(end.until)]);
            }
            if let Some(pool) = pool {
                words.text(builder, Word::Quota, &[Hole::Number(u64::from(pool.quota))]);
            }
        }
        V::Panel { state, resources, effects, .. } => {
            native_state(builder, state, words);
            native_resources(builder, resources, words);
            for entry in effects {
                native_entry(builder, entry, words);
            }
        }
        V::Result { state, resources, .. } => {
            native_state(builder, state, words);
            native_resources(builder, resources, words);
        }
        V::Effect { proposal, task, effect, description } => {
            words.text(builder, Word::Task, &[Hole::Number(*task)]);
            words.text(builder, Word::NativeProposal, &[Hole::Number(*proposal)]);
            words.text(builder, Word::State, &[Hole::Number(description.state)]);
            native_effect(builder, effect, words);
            words.text(builder, Word::Kind, &[Hole::Number(u64::from(description.kind))]);
            if let Some(price) = description.price {
                words.text(builder, Word::MaximumPrice, &[Hole::Number(price)]);
            }
        }
        V::Outbox { entry, description } => {
            native_entry(builder, entry, words);
            words.text(builder, Word::Kind, &[Hole::Number(u64::from(description.kind))]);
        }
        V::Hold { task, resources, drift, effects } => {
            words.text(builder, Word::Task, &[Hole::Number(*task)]);
            native_resources(builder, resources, words);
            for account in drift {
                native_resource(builder, &account.resource, words);
                words.text(builder, Word::At, &[Hole::Wall(account.at)]);
            }
            for entry in effects {
                native_entry(builder, entry, words);
            }
        }
        V::Project { project, resources, environments } => {
            native_resources(builder, resources, words);
            native_environment_rows(builder, document, *project, environments, words, bindings);
        }
        V::Deployment { backend, effects, made, proposal_limit, staged_limit } => {
            words.status(
                builder,
                match backend {
                    infrastructure::native::Backend::OperationIds => Word::BackendIds,
                    infrastructure::native::Backend::NoOperationIds => Word::BackendNoIds,
                },
                &[],
            );
            words.text(builder, Word::Made, &[Hole::Number(u64::from(*made))]);
            words.text(builder, Word::ProposalLimit, &[Hole::Number(u64::from(*proposal_limit))]);
            words.text(builder, Word::StagedLimit, &[Hole::Number(u64::from(*staged_limit))]);
            for entry in effects {
                native_entry(builder, entry, words);
            }
        }
    }
}
