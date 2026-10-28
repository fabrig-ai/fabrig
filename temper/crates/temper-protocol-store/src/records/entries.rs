use super::{Bounds, FITS, Undecodable, api, names, w};
use alloc::boxed::Box;
use skein_lib::{Time, Wall};
use temper_engine_domain_forge as top;

fn encode_condition(value: &skein_forge_client::Condition) -> w::Condition {
    match value {
        skein_forge_client::Condition::None => w::Condition::None,
        skein_forge_client::Condition::Merge { base } => w::Condition::Merge(
            w::MergeCondition::new(&w::CEILINGS, w::MergeConditionParts { base: base.clone() }).expect(FITS),
        ),
        skein_forge_client::Condition::Update { head, base } => w::Condition::Update(
            w::UpdateCondition::new(&w::CEILINGS, w::UpdateConditionParts { head: *head, base: *base }).expect(FITS),
        ),
        skein_forge_client::Condition::Review { head } => {
            w::Condition::Review(w::Commit::new(&w::CEILINGS, w::CommitParts { commit: *head }).expect(FITS))
        }
    }
}
fn decode_condition(value: w::Condition) -> skein_forge_client::Condition {
    match value {
        w::Condition::None => skein_forge_client::Condition::None,
        w::Condition::Merge(value) => skein_forge_client::Condition::Merge { base: value.into_parts().base },
        w::Condition::Update(value) => {
            let w::UpdateConditionParts { head, base } = value.into_parts();
            skein_forge_client::Condition::Update { head, base }
        }
        w::Condition::Review(value) => skein_forge_client::Condition::Review { head: *value.commit() },
    }
}
fn encode_effect(value: &skein_forge_client::Effect) -> w::Effect {
    let skein_forge_client::Effect { write, condition } = value;
    w::Effect::new(
        &w::CEILINGS,
        w::EffectParts { write: api::encode_write(write), condition: encode_condition(condition) },
    )
    .expect(FITS)
}
fn decode_effect(value: w::Effect) -> skein_forge_client::Effect {
    let w::EffectParts { write, condition } = value.into_parts();
    skein_forge_client::Effect { write: api::decode_write(write), condition: decode_condition(condition) }
}
#[expect(clippy::manual_map, reason = "step translations use explicit matches without closures")]
fn encode_entry(value: &skein_forge_client::Entry) -> w::Entry {
    let skein_forge_client::Entry { number, task, repository, effect, start, attempt, failures, queued } = value;
    w::Entry::new(
        &w::CEILINGS,
        w::EntryParts {
            number: *number,
            task: *task,
            repository: names::encode_repository(*repository),
            effect: encode_effect(effect),
            start: match start {
                Some(value) => Some(encode_position(*value)),
                None => None,
            },
            attempt: match attempt {
                Some(value) => Some(encode_attempt(*value)),
                None => None,
            },
            failures: *failures,
            queued: *queued,
        },
    )
    .expect(FITS)
}
#[expect(clippy::manual_map, reason = "step translations use explicit matches without closures")]
fn decode_entry(value: w::Entry) -> skein_forge_client::Entry {
    let w::EntryParts { number, task, repository, effect, start, attempt, failures, queued } = value.into_parts();
    skein_forge_client::Entry {
        number,
        task,
        repository: names::decode_repository(repository),
        effect: decode_effect(effect),
        start: match start {
            Some(value) => Some(decode_position(value)),
            None => None,
        },
        attempt: match attempt {
            Some(value) => Some(decode_attempt(value)),
            None => None,
        },
        failures,
        queued,
    }
}
pub(super) fn encode_position(value: skein_forge_client::Position) -> w::Position {
    let skein_forge_client::Position { at, comment, review, review_page } = value;
    w::Position::new(&w::CEILINGS, w::PositionParts { at: at.as_nanos(), comment, review, review_page }).expect(FITS)
}
pub(super) fn decode_position(value: w::Position) -> skein_forge_client::Position {
    let w::PositionParts { at, comment, review, review_page } = value.into_parts();
    skein_forge_client::Position { at: Time::from_nanos(at), comment, review, review_page }
}
fn encode_attempt(value: skein_forge_client::Attempt) -> w::Attempt {
    let skein_forge_client::Attempt { sent, deadline, wall, expires } = value;
    w::Attempt::new(
        &w::CEILINGS,
        w::AttemptParts {
            sent: sent.as_nanos(),
            deadline: deadline.as_nanos(),
            wall: wall.as_nanos(),
            expires: expires.as_nanos(),
        },
    )
    .expect(FITS)
}
fn decode_attempt(value: w::Attempt) -> skein_forge_client::Attempt {
    let w::AttemptParts { sent, deadline, wall, expires } = value.into_parts();
    skein_forge_client::Attempt {
        sent: Time::from_nanos(sent),
        deadline: Time::from_nanos(deadline),
        wall: Wall::from_nanos(wall),
        expires: Wall::from_nanos(expires),
    }
}
pub(super) fn encode_entry_row(entry: &skein_forge_client::Entry, held: bool) -> w::EntryRow {
    w::EntryRow::new(&w::CEILINGS, w::EntryRowParts { entry: encode_entry(entry), held }).expect(FITS)
}
pub(super) fn decode_entry_row(value: w::EntryRow) -> top::Stored {
    let w::EntryRowParts { entry, held } = value.into_parts();
    top::Stored::Entry { entry: decode_entry(entry), held }
}
pub(super) fn encode_proposed(value: &top::ProposedEffect) -> w::ProposedEffect {
    let top::ProposedEffect { number, resource, entry } = value;
    w::ProposedEffect::new(
        &w::CEILINGS,
        w::ProposedEffectParts { number: *number, resource: names::encode_what(resource), entry: encode_entry(entry) },
    )
    .expect(FITS)
}
pub(super) fn decode_proposed(value: w::ProposedEffect, bounds: &Bounds) -> Result<top::ProposedEffect, Undecodable> {
    let w::ProposedEffectParts { number, resource, entry } = value.into_parts();
    Ok(top::ProposedEffect { number, resource: names::decode_what(resource, bounds)?, entry: decode_entry(entry) })
}

#[expect(clippy::manual_map, reason = "step translations use explicit matches without closures")]
fn encode_payload(value: &top::CallPayload) -> w::CallPayload {
    match value {
        top::CallPayload::Effect { entry, deadline, outcome } => w::CallPayload::Effect(
            w::CallEffect::new(
                &w::CEILINGS,
                w::CallEffectParts {
                    entry: *entry,
                    deadline: deadline.as_nanos(),
                    outcome: match outcome {
                        Some(value) => Some(api::encode_outcome(*value)),
                        None => None,
                    },
                },
            )
            .expect(FITS),
        ),
        top::CallPayload::Refused(error) => w::CallPayload::Refused(
            w::ErrorValue::new(&w::CEILINGS, w::ErrorValueParts { error: api::encode_error(*error) }).expect(FITS),
        ),
        top::CallPayload::Read(value) => {
            let result = match value.as_ref() {
                Ok(value) => w::ReadResult::Answer(Box::new(
                    w::AnswerValue::new(&w::CEILINGS, w::AnswerValueParts { answer: super::answers::encode(value) })
                        .expect(FITS),
                )),
                Err(error) => w::ReadResult::Error(
                    w::ErrorValue::new(&w::CEILINGS, w::ErrorValueParts { error: api::encode_error(*error) })
                        .expect(FITS),
                ),
            };
            w::CallPayload::Read(w::CallRead::new(&w::CEILINGS, w::CallReadParts { result }).expect(FITS))
        }
    }
}
#[expect(clippy::manual_map, reason = "step translations use explicit matches without closures")]
fn decode_payload(value: w::CallPayload, bounds: &Bounds) -> Result<top::CallPayload, Undecodable> {
    Ok(match value {
        w::CallPayload::Effect(value) => {
            let w::CallEffectParts { entry, deadline, outcome } = value.into_parts();
            top::CallPayload::Effect {
                entry,
                deadline: Wall::from_nanos(deadline),
                outcome: match outcome {
                    Some(value) => Some(api::decode_outcome(value)),
                    None => None,
                },
            }
        }
        w::CallPayload::Refused(value) => top::CallPayload::Refused(api::decode_error(value.into_parts().error)),
        w::CallPayload::Read(value) => top::CallPayload::Read(Box::new(match value.into_parts().result {
            w::ReadResult::Answer(value) => Ok(super::answers::decode(value.into_parts().answer, bounds)?),
            w::ReadResult::Error(value) => Err(api::decode_error(value.into_parts().error)),
        })),
    })
}
pub(super) fn encode_call(key: &top::NamedCall, answer: &top::CallPayload) -> w::Call {
    let top::NamedCall { task, attempt, completion, position } = *key;
    let key = w::NamedCall::new(&w::CEILINGS, w::NamedCallParts { task, attempt, completion, position }).expect(FITS);
    w::Call::new(&w::CEILINGS, w::CallParts { key, answer: encode_payload(answer) }).expect(FITS)
}
pub(super) fn decode_call(value: w::Call, bounds: &Bounds) -> Result<top::Stored, Undecodable> {
    let w::CallParts { key, answer } = value.into_parts();
    let w::NamedCallParts { task, attempt, completion, position } = key.into_parts();
    Ok(top::Stored::Call {
        key: top::NamedCall { task, attempt, completion, position },
        answer: decode_payload(answer, bounds)?,
    })
}
