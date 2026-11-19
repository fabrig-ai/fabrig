use super::{Bounds, Undecodable, api, names, w};
use alloc::boxed::Box;
use skein_lib::{Time, Wall};
use temper_engine_domain_forge_vocabulary as top;

fn encode_condition(value: &skein_forge_client::Condition) -> w::Condition {
    match value {
        skein_forge_client::Condition::None => w::Condition::None,
        skein_forge_client::Condition::Merge { base } => w::Condition::Merge(w::MergeCondition { base: base.clone() }),
        skein_forge_client::Condition::Update { head, base } => {
            w::Condition::Update(w::UpdateCondition { head: *head, base: *base })
        }
        skein_forge_client::Condition::Review { head } => w::Condition::Review(w::Commit { commit: *head }),
    }
}
fn decode_condition(value: w::Condition) -> skein_forge_client::Condition {
    match value {
        w::Condition::None => skein_forge_client::Condition::None,
        w::Condition::Merge(value) => skein_forge_client::Condition::Merge { base: value.base },
        w::Condition::Update(value) => {
            let w::UpdateCondition { head, base } = value;
            skein_forge_client::Condition::Update { head, base }
        }
        w::Condition::Review(value) => skein_forge_client::Condition::Review { head: value.commit },
    }
}
fn encode_effect(value: &skein_forge_client::Effect) -> w::Effect {
    let skein_forge_client::Effect { write, condition } = value;
    w::Effect { write: api::encode_write(write), condition: encode_condition(condition) }
}
fn decode_effect(value: w::Effect) -> skein_forge_client::Effect {
    let w::Effect { write, condition } = value;
    skein_forge_client::Effect { write: api::decode_write(write), condition: decode_condition(condition) }
}
fn encode_entry(value: &skein_forge_client::Entry) -> w::Entry {
    let skein_forge_client::Entry { number, task, repository, effect, start, attempt, failures, queued } = value;
    w::Entry {
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
    }
}
fn decode_entry(value: w::Entry) -> skein_forge_client::Entry {
    let w::Entry { number, task, repository, effect, start, attempt, failures, queued } = value;
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
    w::Position { at: at.as_nanos(), comment, review, review_page }
}
pub(super) fn decode_position(value: w::Position) -> skein_forge_client::Position {
    let w::Position { at, comment, review, review_page } = value;
    skein_forge_client::Position { at: Time::from_nanos(at), comment, review, review_page }
}
fn encode_attempt(value: skein_forge_client::Attempt) -> w::Attempt {
    let skein_forge_client::Attempt { sent, deadline, wall, expires } = value;
    w::Attempt {
        sent: sent.as_nanos(),
        deadline: deadline.as_nanos(),
        wall: wall.as_nanos(),
        expires: expires.as_nanos(),
    }
}
fn decode_attempt(value: w::Attempt) -> skein_forge_client::Attempt {
    let w::Attempt { sent, deadline, wall, expires } = value;
    skein_forge_client::Attempt {
        sent: Time::from_nanos(sent),
        deadline: Time::from_nanos(deadline),
        wall: Wall::from_nanos(wall),
        expires: Wall::from_nanos(expires),
    }
}
pub(super) fn encode_entry_row(entry: &skein_forge_client::Entry, held: bool) -> w::EntryRow {
    w::EntryRow { entry: encode_entry(entry), held }
}
pub(super) fn decode_entry_row(value: w::EntryRow) -> top::Stored {
    let w::EntryRow { entry, held } = value;
    top::Stored::Entry { entry: decode_entry(entry), held }
}
pub(super) fn encode_proposed(value: &top::ProposedEffect) -> w::ProposedEffect {
    let top::ProposedEffect { number, resource, entry } = value;
    w::ProposedEffect { number: *number, resource: names::encode_what(resource), entry: encode_entry(entry) }
}
pub(super) fn decode_proposed(value: w::ProposedEffect, bounds: &Bounds) -> Result<top::ProposedEffect, Undecodable> {
    let w::ProposedEffect { number, resource, entry } = value;
    Ok(top::ProposedEffect { number, resource: names::decode_what(resource, bounds)?, entry: decode_entry(entry) })
}

fn encode_payload(value: &top::CallPayload) -> w::CallPayload {
    match value {
        top::CallPayload::Follow { follow, refusal } => {
            w::CallPayload::Follow(w::CallFollow { follow: *follow, refusal: refusal.clone() })
        }
        top::CallPayload::Red(reply) => w::CallPayload::Red(Box::new(super::reds::encode_reply(reply))),
        top::CallPayload::Finding(reply) => w::CallPayload::Finding(Box::new(super::findings::encode_reply(reply))),
        top::CallPayload::Decided { took, reach } => {
            let reach = match reach {
                Some(reach) => Some(super::away::encode_reach(*reach)),
                None => None,
            };
            w::CallPayload::Decided(w::CallDecided { took: took.clone(), reach })
        }
        top::CallPayload::Effect { entry, deadline, outcome } => w::CallPayload::Effect(w::CallEffect {
            entry: *entry,
            deadline: deadline.as_nanos(),
            outcome: match outcome {
                Some(value) => Some(api::encode_outcome(*value)),
                None => None,
            },
        }),
        top::CallPayload::Refused(error) => w::CallPayload::Refused(w::ErrorValue { error: api::encode_error(*error) }),
        top::CallPayload::Read(value) => {
            let result = match value.as_ref() {
                Ok(value) => w::ReadResult::Answer(Box::new(w::AnswerValue { answer: super::answers::encode(value) })),
                Err(error) => w::ReadResult::Error(w::ErrorValue { error: api::encode_error(*error) }),
            };
            w::CallPayload::Read(w::CallRead { result })
        }
    }
}
fn decode_payload(value: w::CallPayload, bounds: &Bounds) -> Result<top::CallPayload, Undecodable> {
    Ok(match value {
        w::CallPayload::Follow(value) => {
            let w::CallFollow { follow, refusal } = value;
            top::CallPayload::Follow { follow, refusal }
        }
        w::CallPayload::Red(reply) => top::CallPayload::Red(Box::new(super::reds::decode_reply(*reply, bounds)?)),
        w::CallPayload::Finding(reply) => top::CallPayload::Finding(Box::new(super::findings::decode_reply(*reply))),
        w::CallPayload::Decided(value) => {
            let w::CallDecided { took, reach } = value;
            let reach = match reach {
                Some(reach) => Some(super::away::decode_reach(reach)?),
                None => None,
            };
            top::CallPayload::Decided { took, reach }
        }
        w::CallPayload::Effect(value) => {
            let w::CallEffect { entry, deadline, outcome } = value;
            top::CallPayload::Effect {
                entry,
                deadline: Wall::from_nanos(deadline),
                outcome: match outcome {
                    Some(value) => Some(api::decode_outcome(value)),
                    None => None,
                },
            }
        }
        w::CallPayload::Refused(value) => top::CallPayload::Refused(api::decode_error(value.error)),
        w::CallPayload::Read(value) => top::CallPayload::Read(Box::new(match value.result {
            w::ReadResult::Answer(value) => Ok(super::answers::decode(value.answer, bounds)?),
            w::ReadResult::Error(value) => Err(api::decode_error(value.error)),
        })),
    })
}
pub(super) fn encode_call(key: &top::NamedCall, answer: &top::CallPayload) -> w::Call {
    let top::NamedCall { task, attempt, completion, position } = *key;
    let key = w::NamedCall { task, attempt, completion, position };
    w::Call { key, answer: encode_payload(answer) }
}
pub(super) fn decode_call(value: w::Call, bounds: &Bounds) -> Result<top::Stored, Undecodable> {
    let w::Call { key, answer } = value;
    let w::NamedCall { task, attempt, completion, position } = key;
    Ok(top::Stored::Call {
        key: top::NamedCall { task, attempt, completion, position },
        answer: decode_payload(answer, bounds)?,
    })
}
