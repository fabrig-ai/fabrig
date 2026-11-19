//! Family envelopes and subjects (jig/docs/design/examples.md, 8).
use crate::infrastructure::documents as d;
use crate::wire::{Box, Invalid, w};
pub(crate) fn scope_encode(value: d::Scope) -> w::InfrastructureScope {
    match value {
        d::Scope::Project(value) => {
            let value = w::InfrastructureScopeProject { value };
            w::InfrastructureScope::Project(value)
        }
        d::Scope::Deployment => w::InfrastructureScope::Deployment,
    }
}

pub(crate) fn scope_decode(value: w::InfrastructureScope) -> d::Scope {
    match value {
        w::InfrastructureScope::Project(value) => d::Scope::Project(value.value),
        w::InfrastructureScope::Deployment => d::Scope::Deployment,
    }
}

pub(crate) fn kind_encode(value: &d::Kind) -> w::InfrastructureKind {
    match value {
        d::Kind::Environment(value) => {
            let value =
                w::InfrastructureKindEnvironment { value: crate::wire::infrastructure_data::environment_encode(value) };
            w::InfrastructureKind::Environment(value)
        }
        d::Kind::Environments { pool } => {
            let value = w::InfrastructureKindEnvironments {
                pool: match pool {
                    Some(item) => Some(crate::wire::infrastructure_data::pool_encode(item)),
                    None => None,
                },
            };
            w::InfrastructureKind::Environments(value)
        }
        d::Kind::Panel(value) => {
            let value = w::InfrastructureKindPanel { value: (*value) };
            w::InfrastructureKind::Panel(value)
        }
        d::Kind::Result(value) => {
            let value = w::InfrastructureKindResult { value: (*value) };
            w::InfrastructureKind::Result(value)
        }
        d::Kind::Effect(value) => {
            let value = w::InfrastructureKindEffect { value: crate::wire::proposal_encode(value) };
            w::InfrastructureKind::Effect(value)
        }
        d::Kind::OwnerEffect(value) => {
            let value = w::InfrastructureKindOwnerEffect { value: (*value) };
            w::InfrastructureKind::OwnerEffect(value)
        }
        d::Kind::Outbox(value) => {
            let value = w::InfrastructureKindOutbox { value: (*value) };
            w::InfrastructureKind::Outbox(value)
        }
        d::Kind::Hold(value) => {
            let value = w::InfrastructureKindHold { value: (*value) };
            w::InfrastructureKind::Hold(value)
        }
        d::Kind::Project(value) => {
            let value = w::InfrastructureKindProject { value: (*value) };
            w::InfrastructureKind::Project(value)
        }
        d::Kind::Deployment => w::InfrastructureKind::Deployment,
    }
}
pub(crate) fn kind_decode(value: w::InfrastructureKind) -> d::Kind {
    match value {
        w::InfrastructureKind::Environment(value) => {
            d::Kind::Environment(crate::wire::infrastructure_data::environment_decode(value.value))
        }
        w::InfrastructureKind::Environments(value) => d::Kind::Environments {
            pool: match value.pool {
                Some(item) => Some(crate::wire::infrastructure_data::pool_decode(item)),
                None => None,
            },
        },
        w::InfrastructureKind::Panel(value) => d::Kind::Panel(value.value),
        w::InfrastructureKind::Result(value) => d::Kind::Result(value.value),
        w::InfrastructureKind::Effect(value) => d::Kind::Effect(crate::wire::proposal_decode(value.value)),
        w::InfrastructureKind::OwnerEffect(value) => d::Kind::OwnerEffect(value.value),
        w::InfrastructureKind::Outbox(value) => d::Kind::Outbox(value.value),
        w::InfrastructureKind::Hold(value) => d::Kind::Hold(value.value),
        w::InfrastructureKind::Project(value) => d::Kind::Project(value.value),
        w::InfrastructureKind::Deployment => d::Kind::Deployment,
    }
}

pub(crate) fn subject_encode(value: &d::Subject) -> w::InfrastructureSubject {
    w::InfrastructureSubject { scope: scope_encode(value.scope), kind: kind_encode(&(value.kind)) }
}

pub(crate) fn subject_decode(value: w::InfrastructureSubject) -> d::Subject {
    d::Subject { scope: scope_decode(value.scope), kind: kind_decode(value.kind) }
}

pub(crate) fn document_encode(value: &d::Document) -> Result<w::InfrastructureDocument, Invalid> {
    Ok(match value {
        d::Document::Value(value) => {
            let value =
                w::InfrastructureDocumentValue { value: crate::wire::infrastructure_data::document_encode(value)? };
            w::InfrastructureDocument::Value(value)
        }
    })
}

pub(crate) fn document_decode(value: w::InfrastructureDocument) -> d::Document {
    match value {
        w::InfrastructureDocument::Value(value) => {
            d::Document::Value(Box::new(crate::wire::infrastructure_data::document_decode(value.value)))
        }
    }
}

pub(crate) fn ask_encode(value: &d::Ask) -> w::InfrastructureAsk {
    match value {
        d::Ask::MoveEnd { object, project, environment, revision, until } => {
            let value = w::InfrastructureAskMoveEnd {
                object: (*object).raw(),
                project: (*project),
                environment: crate::wire::infrastructure_data::environment_encode(environment),
                revision: revision.value,
                until: (*until),
            };
            w::InfrastructureAsk::MoveEnd(value)
        }
    }
}

pub(crate) fn ask_decode(value: w::InfrastructureAsk) -> d::Ask {
    match value {
        w::InfrastructureAsk::MoveEnd(value) => d::Ask::MoveEnd {
            object: skein_lib::Token::new(value.object),
            project: value.project,
            environment: crate::wire::infrastructure_data::environment_decode(value.environment),
            revision: jig_client::contract::Revision { value: value.revision },
            until: value.until,
        },
    }
}

pub(crate) fn refusal_encode(value: d::Refusal) -> w::InfrastructureRefusal {
    match value {
        d::Refusal::Role { held } => {
            let value = w::InfrastructureRefusalRole { held };
            w::InfrastructureRefusal::Role(value)
        }
        d::Refusal::Past => w::InfrastructureRefusal::Past,
        d::Refusal::Exhausted => w::InfrastructureRefusal::Exhausted,
        d::Refusal::SignedOut => w::InfrastructureRefusal::SignedOut,
        d::Refusal::NotAllowed => w::InfrastructureRefusal::NotAllowed,
        d::Refusal::Conflict => w::InfrastructureRefusal::Conflict,
        d::Refusal::Invalid => w::InfrastructureRefusal::Invalid,
    }
}

pub(crate) fn refusal_decode(value: w::InfrastructureRefusal) -> d::Refusal {
    match value {
        w::InfrastructureRefusal::Role(value) => d::Refusal::Role { held: value.held },
        w::InfrastructureRefusal::Past => d::Refusal::Past,
        w::InfrastructureRefusal::Exhausted => d::Refusal::Exhausted,
        w::InfrastructureRefusal::SignedOut => d::Refusal::SignedOut,
        w::InfrastructureRefusal::NotAllowed => d::Refusal::NotAllowed,
        w::InfrastructureRefusal::Conflict => d::Refusal::Conflict,
        w::InfrastructureRefusal::Invalid => d::Refusal::Invalid,
    }
}

pub(crate) fn answer_encode(value: &d::Answer) -> Result<w::InfrastructureAnswer, Invalid> {
    Ok(match value {
        d::Answer::MovedEnd { environment, revision, until } => {
            let value = w::InfrastructureAnswerMovedEnd {
                environment: crate::wire::infrastructure_data::environment_encode(environment),
                revision: revision.value,
                until: (*until),
            };
            w::InfrastructureAnswer::MovedEnd(value)
        }
        d::Answer::Moved { environment, revision, until } => {
            let value = w::InfrastructureAnswerMoved {
                environment: crate::wire::infrastructure_data::environment_encode(environment),
                revision: revision.value,
                until: (*until),
            };
            w::InfrastructureAnswer::Moved(value)
        }
        d::Answer::Gone => w::InfrastructureAnswer::Gone,
        d::Answer::Refused { why } => {
            let value = w::InfrastructureAnswerRefused { why: refusal_encode(*why) };
            w::InfrastructureAnswer::Refused(value)
        }
        d::Answer::EntranceRefused(_) => return Err(Invalid),
    })
}

pub(crate) fn answer_decode(value: w::InfrastructureAnswer) -> d::Answer {
    match value {
        w::InfrastructureAnswer::MovedEnd(value) => d::Answer::MovedEnd {
            environment: crate::wire::infrastructure_data::environment_decode(value.environment),
            revision: jig_client::contract::Revision { value: value.revision },
            until: value.until,
        },
        w::InfrastructureAnswer::Moved(value) => d::Answer::Moved {
            environment: crate::wire::infrastructure_data::environment_decode(value.environment),
            revision: jig_client::contract::Revision { value: value.revision },
            until: value.until,
        },
        w::InfrastructureAnswer::Gone => d::Answer::Gone,
        w::InfrastructureAnswer::Refused(value) => d::Answer::Refused { why: refusal_decode(value.why) },
    }
}
