//! Family envelopes and subjects (jig/docs/design/examples.md, 8).
use crate::infrastructure::documents as d;
use crate::wire::{Box, Invalid, w};
pub(crate) fn scope_encode(value: d::Scope) -> Result<w::InfrastructureScope, Invalid> {
    Ok(match value {
        d::Scope::Project(value) => {
            let parts = w::InfrastructureScopeProjectParts { value };
            let Ok(value) = w::InfrastructureScopeProject::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureScope::Project(value)
        }
        d::Scope::Deployment => w::InfrastructureScope::Deployment,
    })
}

pub(crate) fn scope_decode(value: w::InfrastructureScope) -> d::Scope {
    match value {
        w::InfrastructureScope::Project(value) => {
            let value = value.into_parts();
            d::Scope::Project(value.value)
        }
        w::InfrastructureScope::Deployment => d::Scope::Deployment,
    }
}

pub(crate) fn kind_encode(value: &d::Kind) -> Result<w::InfrastructureKind, Invalid> {
    Ok(match value {
        d::Kind::Environment(value) => {
            let parts = w::InfrastructureKindEnvironmentParts {
                value: crate::wire::infrastructure_data::environment_encode(value)?,
            };
            let Ok(value) = w::InfrastructureKindEnvironment::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureKind::Environment(value)
        }
        d::Kind::Environments { pool } => {
            let parts = w::InfrastructureKindEnvironmentsParts {
                pool: match pool {
                    Some(item) => Some(crate::wire::infrastructure_data::pool_encode(item)?),
                    None => None,
                },
            };
            let Ok(value) = w::InfrastructureKindEnvironments::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureKind::Environments(value)
        }
        d::Kind::Panel(value) => {
            let parts = w::InfrastructureKindPanelParts { value: (*value) };
            let Ok(value) = w::InfrastructureKindPanel::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureKind::Panel(value)
        }
        d::Kind::Result(value) => {
            let parts = w::InfrastructureKindResultParts { value: (*value) };
            let Ok(value) = w::InfrastructureKindResult::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureKind::Result(value)
        }
        d::Kind::Effect(value) => {
            let parts = w::InfrastructureKindEffectParts { value: crate::wire::proposal_encode(value)? };
            let Ok(value) = w::InfrastructureKindEffect::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureKind::Effect(value)
        }
        d::Kind::OwnerEffect(value) => {
            let parts = w::InfrastructureKindOwnerEffectParts { value: (*value) };
            let Ok(value) = w::InfrastructureKindOwnerEffect::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureKind::OwnerEffect(value)
        }
        d::Kind::Outbox(value) => {
            let parts = w::InfrastructureKindOutboxParts { value: (*value) };
            let Ok(value) = w::InfrastructureKindOutbox::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureKind::Outbox(value)
        }
        d::Kind::Hold(value) => {
            let parts = w::InfrastructureKindHoldParts { value: (*value) };
            let Ok(value) = w::InfrastructureKindHold::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureKind::Hold(value)
        }
        d::Kind::Project(value) => {
            let parts = w::InfrastructureKindProjectParts { value: (*value) };
            let Ok(value) = w::InfrastructureKindProject::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureKind::Project(value)
        }
        d::Kind::Deployment => w::InfrastructureKind::Deployment,
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn kind_decode(value: w::InfrastructureKind) -> d::Kind {
    match value {
        w::InfrastructureKind::Environment(value) => {
            let value = value.into_parts();
            d::Kind::Environment(crate::wire::infrastructure_data::environment_decode(value.value))
        }
        w::InfrastructureKind::Environments(value) => {
            let value = value.into_parts();
            d::Kind::Environments {
                pool: match value.pool {
                    Some(item) => Some(crate::wire::infrastructure_data::pool_decode(item)),
                    None => None,
                },
            }
        }
        w::InfrastructureKind::Panel(value) => {
            let value = value.into_parts();
            d::Kind::Panel(value.value)
        }
        w::InfrastructureKind::Result(value) => {
            let value = value.into_parts();
            d::Kind::Result(value.value)
        }
        w::InfrastructureKind::Effect(value) => {
            let value = value.into_parts();
            d::Kind::Effect(crate::wire::proposal_decode(value.value))
        }
        w::InfrastructureKind::OwnerEffect(value) => {
            let value = value.into_parts();
            d::Kind::OwnerEffect(value.value)
        }
        w::InfrastructureKind::Outbox(value) => {
            let value = value.into_parts();
            d::Kind::Outbox(value.value)
        }
        w::InfrastructureKind::Hold(value) => {
            let value = value.into_parts();
            d::Kind::Hold(value.value)
        }
        w::InfrastructureKind::Project(value) => {
            let value = value.into_parts();
            d::Kind::Project(value.value)
        }
        w::InfrastructureKind::Deployment => d::Kind::Deployment,
    }
}

pub(crate) fn subject_encode(value: &d::Subject) -> Result<w::InfrastructureSubject, Invalid> {
    let parts = w::InfrastructureSubjectParts { scope: scope_encode(value.scope)?, kind: kind_encode(&(value.kind))? };
    let Ok(value) = w::InfrastructureSubject::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}

pub(crate) fn subject_decode(value: w::InfrastructureSubject) -> d::Subject {
    let value = value.into_parts();
    d::Subject { scope: scope_decode(value.scope), kind: kind_decode(value.kind) }
}

pub(crate) fn document_encode(value: &d::Document) -> Result<w::InfrastructureDocument, Invalid> {
    Ok(match value {
        d::Document::Value(value) => {
            let parts = w::InfrastructureDocumentValueParts {
                value: crate::wire::infrastructure_data::document_encode(value)?,
            };
            let Ok(value) = w::InfrastructureDocumentValue::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureDocument::Value(value)
        }
    })
}

pub(crate) fn document_decode(value: w::InfrastructureDocument) -> d::Document {
    match value {
        w::InfrastructureDocument::Value(value) => {
            let value = value.into_parts();
            d::Document::Value(Box::new(crate::wire::infrastructure_data::document_decode(value.value)))
        }
    }
}

pub(crate) fn ask_encode(value: &d::Ask) -> Result<w::InfrastructureAsk, Invalid> {
    Ok(match value {
        d::Ask::MoveEnd { object, project, environment, revision, until } => {
            let parts = w::InfrastructureAskMoveEndParts {
                object: (*object).raw(),
                project: (*project),
                environment: crate::wire::infrastructure_data::environment_encode(environment)?,
                revision: revision.value,
                until: (*until),
            };
            let Ok(value) = w::InfrastructureAskMoveEnd::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureAsk::MoveEnd(value)
        }
    })
}

pub(crate) fn ask_decode(value: w::InfrastructureAsk) -> d::Ask {
    match value {
        w::InfrastructureAsk::MoveEnd(value) => {
            let value = value.into_parts();
            d::Ask::MoveEnd {
                object: skein_lib::Token::new(value.object),
                project: value.project,
                environment: crate::wire::infrastructure_data::environment_decode(value.environment),
                revision: jig_client::contract::Revision { value: value.revision },
                until: value.until,
            }
        }
    }
}

pub(crate) fn refusal_encode(value: d::Refusal) -> Result<w::InfrastructureRefusal, Invalid> {
    Ok(match value {
        d::Refusal::Role { held } => {
            let parts = w::InfrastructureRefusalRoleParts { held };
            let Ok(value) = w::InfrastructureRefusalRole::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureRefusal::Role(value)
        }
        d::Refusal::Past => w::InfrastructureRefusal::Past,
        d::Refusal::Exhausted => w::InfrastructureRefusal::Exhausted,
        d::Refusal::SignedOut => w::InfrastructureRefusal::SignedOut,
        d::Refusal::NotAllowed => w::InfrastructureRefusal::NotAllowed,
        d::Refusal::Conflict => w::InfrastructureRefusal::Conflict,
        d::Refusal::Invalid => w::InfrastructureRefusal::Invalid,
    })
}

pub(crate) fn refusal_decode(value: w::InfrastructureRefusal) -> d::Refusal {
    match value {
        w::InfrastructureRefusal::Role(value) => {
            let value = value.into_parts();
            d::Refusal::Role { held: value.held }
        }
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
            let parts = w::InfrastructureAnswerMovedEndParts {
                environment: crate::wire::infrastructure_data::environment_encode(environment)?,
                revision: revision.value,
                until: (*until),
            };
            let Ok(value) = w::InfrastructureAnswerMovedEnd::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureAnswer::MovedEnd(value)
        }
        d::Answer::Moved { environment, revision, until } => {
            let parts = w::InfrastructureAnswerMovedParts {
                environment: crate::wire::infrastructure_data::environment_encode(environment)?,
                revision: revision.value,
                until: (*until),
            };
            let Ok(value) = w::InfrastructureAnswerMoved::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureAnswer::Moved(value)
        }
        d::Answer::Gone => w::InfrastructureAnswer::Gone,
        d::Answer::Refused { why } => {
            let parts = w::InfrastructureAnswerRefusedParts { why: refusal_encode(*why)? };
            let Ok(value) = w::InfrastructureAnswerRefused::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureAnswer::Refused(value)
        }
        d::Answer::EntranceRefused(_) => return Err(Invalid),
    })
}

pub(crate) fn answer_decode(value: w::InfrastructureAnswer) -> d::Answer {
    match value {
        w::InfrastructureAnswer::MovedEnd(value) => {
            let value = value.into_parts();
            d::Answer::MovedEnd {
                environment: crate::wire::infrastructure_data::environment_decode(value.environment),
                revision: jig_client::contract::Revision { value: value.revision },
                until: value.until,
            }
        }
        w::InfrastructureAnswer::Moved(value) => {
            let value = value.into_parts();
            d::Answer::Moved {
                environment: crate::wire::infrastructure_data::environment_decode(value.environment),
                revision: jig_client::contract::Revision { value: value.revision },
                until: value.until,
            }
        }
        w::InfrastructureAnswer::Gone => d::Answer::Gone,
        w::InfrastructureAnswer::Refused(value) => {
            let value = value.into_parts();
            d::Answer::Refused { why: refusal_decode(value.why) }
        }
    }
}
