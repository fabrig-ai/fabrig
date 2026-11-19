//! Total translations of the test connector records (protocol/store/README.md, 7).
use crate::{Undecodable, v2 as w};
use alloc::boxed::Box;
use jig_test_connector as d;
use skein_lib::{List, Reader, Token, Wall};
use w::Record as _;
mod facts;
use facts::{decode_refusal, encode_refusal};
/// Encode one record admitted under the connector's bounds.
#[must_use]
pub fn encode(record: &d::Record) -> Box<[u8]> {
    let row = w::Row { value: encode_record(record) };
    row.encode(&w::CEILINGS).expect("admitted record fits schema")
}
/// Decode one record within this connector's configured bounds.
pub fn decode(bytes: &[u8], limits: &d::Limits) -> Result<d::Record, Undecodable> {
    let bounds = crate::bounds(limits).ok_or(Undecodable)?;
    let Ok(row) = w::Row::decode(&bounds, &mut Reader::new(bytes)) else {
        return Err(Undecodable);
    };
    decode_record(row.value, limits)
}

fn encode_key(value: &d::Key) -> w::Key {
    let origin = match value.origin {
        d::Purpose::Procedure => w::Purpose::Procedure,
        d::Purpose::Projection => w::Purpose::Projection,
        d::Purpose::Call { attempt, completion, position } => {
            w::Purpose::Call(w::CallPurpose { attempt, completion, position })
        }
    };
    w::Key { origin, deployment: value.deployment, task: value.task, purpose: value.purpose }
}
fn decode_key(value: w::Key) -> d::Key {
    let w::Key { origin, deployment, task, purpose } = value;
    let origin = match origin {
        w::Purpose::Procedure => d::Purpose::Procedure,
        w::Purpose::Projection => d::Purpose::Projection,
        w::Purpose::Call(call) => {
            let w::CallPurpose { attempt, completion, position } = call;
            d::Purpose::Call { attempt, completion, position }
        }
    };
    d::Key { origin, deployment, task, purpose }
}
fn encode_effect(value: &d::Effect) -> w::Effect {
    let d::Effect { kind, resources, purpose, condition, target, state } = value;
    w::Effect {
        kind: *kind,
        resources: encode_paths(resources),
        purpose: *purpose,
        condition: *condition,
        target: *target,
        state: *state,
    }
}
fn decode_effect(value: w::Effect, limits: &d::Limits) -> Result<d::Effect, Undecodable> {
    let w::Effect { kind, resources, purpose, condition, target, state } = value;
    Ok(d::Effect { kind, resources: decode_paths(resources, limits)?, purpose, condition, target, state })
}
fn encode_attempt(value: &d::Attempt) -> w::Attempt {
    let d::Attempt { number, sent, deadline } = value;
    w::Attempt { number: *number, sent: sent.as_nanos(), deadline: deadline.as_nanos() }
}
fn decode_attempt(value: w::Attempt) -> d::Attempt {
    let w::Attempt { number, sent, deadline } = value;
    d::Attempt { number, sent: Wall::from_nanos(sent), deadline: Wall::from_nanos(deadline) }
}
fn encode_outbox_entry(value: &d::OutboxEntry) -> w::OutboxEntry {
    let d::OutboxEntry { number, task, key, effect, attempt, phase } = value;
    w::OutboxEntry {
        number: *number,
        task: *task,
        key: encode_key(key),
        effect: encode_effect(effect),
        attempt: match attempt {
            Some(value) => Some(encode_attempt(value)),
            None => None,
        },
        phase: encode_effect_phase(*phase),
    }
}
fn decode_outbox_entry(value: w::OutboxEntry, limits: &d::Limits) -> Result<d::OutboxEntry, Undecodable> {
    let w::OutboxEntry { number, task, key, effect, attempt, phase } = value;
    Ok(d::OutboxEntry {
        number,
        task,
        key: decode_key(key),
        effect: decode_effect(effect, limits)?,
        attempt: match attempt {
            Some(value) => Some(decode_attempt(value)),
            None => None,
        },
        phase: decode_effect_phase(phase),
    })
}
fn encode_procedure_state(value: &d::ProcedureState) -> w::ProcedureState {
    let d::ProcedureState { project, task, number, resource, index, steps, awaiting, deadline, delegate, revision } =
        value;
    w::ProcedureState {
        project: *project,
        task: *task,
        number: *number,
        resource: encode_path(resource),
        index: *index,
        steps: *steps,
        awaiting: *awaiting,
        delegate: *delegate,
        revision: *revision,
        deadline: match deadline {
            Some(value) => Some(value.as_nanos()),
            None => None,
        },
    }
}
fn decode_procedure_state(value: w::ProcedureState, limits: &d::Limits) -> Result<d::ProcedureState, Undecodable> {
    let w::ProcedureState { project, task, number, resource, index, steps, awaiting, deadline, delegate, revision } =
        value;
    Ok(d::ProcedureState {
        project,
        task,
        number,
        resource: decode_path(resource, limits)?,
        index,
        steps,
        awaiting,
        delegate,
        revision,
        deadline: match deadline {
            Some(value) => Some(Wall::from_nanos(value)),
            None => None,
        },
    })
}
fn encode_application_object(value: &d::ApplicationObject) -> w::ApplicationObject {
    let d::ApplicationObject { revision, value } = value;
    w::ApplicationObject { revision: *revision, value: *value }
}
fn decode_application_object(value: w::ApplicationObject) -> d::ApplicationObject {
    let w::ApplicationObject { revision, value } = value;
    d::ApplicationObject { revision, value }
}
fn encode_effect_phase(value: d::EffectPhase) -> w::EffectPhase {
    match value {
        d::EffectPhase::Kept => w::EffectPhase::Kept,
        d::EffectPhase::Sent => w::EffectPhase::Sent,
        d::EffectPhase::Uncertain => w::EffectPhase::Uncertain,
        d::EffectPhase::Retry => w::EffectPhase::Retry,
        d::EffectPhase::Repeat => w::EffectPhase::Repeat,
        d::EffectPhase::Held => w::EffectPhase::Held,
    }
}
fn decode_effect_phase(value: w::EffectPhase) -> d::EffectPhase {
    match value {
        w::EffectPhase::Kept => d::EffectPhase::Kept,
        w::EffectPhase::Sent => d::EffectPhase::Sent,
        w::EffectPhase::Uncertain => d::EffectPhase::Uncertain,
        w::EffectPhase::Retry => d::EffectPhase::Retry,
        w::EffectPhase::Repeat => d::EffectPhase::Repeat,
        w::EffectPhase::Held => d::EffectPhase::Held,
    }
}
fn encode_resource_role(value: d::ResourceRole) -> w::ResourceRole {
    match value {
        d::ResourceRole::Owned => w::ResourceRole::Owned,
        d::ResourceRole::Participating => w::ResourceRole::Participating,
        d::ResourceRole::Context => w::ResourceRole::Context,
        d::ResourceRole::Unavailable => w::ResourceRole::Unavailable,
    }
}
fn decode_resource_role(value: w::ResourceRole) -> d::ResourceRole {
    match value {
        w::ResourceRole::Owned => d::ResourceRole::Owned,
        w::ResourceRole::Participating => d::ResourceRole::Participating,
        w::ResourceRole::Context => d::ResourceRole::Context,
        w::ResourceRole::Unavailable => d::ResourceRole::Unavailable,
    }
}
fn encode_application_outcome(value: &d::ApplicationOutcome) -> w::ApplicationOutcome {
    match value {
        d::ApplicationOutcome::Changed { object, now } => {
            w::ApplicationOutcome::Changed(w::ApplicationOutcomeChanged {
                object: *object,
                now: encode_application_object(now),
            })
        }
        d::ApplicationOutcome::Moved { object, now } => w::ApplicationOutcome::Moved(w::ApplicationOutcomeMoved {
            object: *object,
            now: encode_application_object(now),
        }),
        d::ApplicationOutcome::Role { held } => w::ApplicationOutcome::Role(w::ApplicationOutcomeRole { held: *held }),
        d::ApplicationOutcome::Unknown => w::ApplicationOutcome::Unknown,
        d::ApplicationOutcome::Exhausted => w::ApplicationOutcome::Exhausted,
        d::ApplicationOutcome::CoreRefused { why } => {
            w::ApplicationOutcome::CoreRefused(w::ApplicationOutcomeCoreRefused { why: encode_refusal(why) })
        }
    }
}
fn decode_application_outcome(value: w::ApplicationOutcome) -> d::ApplicationOutcome {
    match value {
        w::ApplicationOutcome::Changed(value) => {
            let w::ApplicationOutcomeChanged { object, now } = value;
            d::ApplicationOutcome::Changed { object, now: decode_application_object(now) }
        }
        w::ApplicationOutcome::Moved(value) => {
            let w::ApplicationOutcomeMoved { object, now } = value;
            d::ApplicationOutcome::Moved { object, now: decode_application_object(now) }
        }
        w::ApplicationOutcome::Role(value) => {
            let w::ApplicationOutcomeRole { held } = value;
            d::ApplicationOutcome::Role { held }
        }
        w::ApplicationOutcome::Unknown => d::ApplicationOutcome::Unknown,
        w::ApplicationOutcome::Exhausted => d::ApplicationOutcome::Exhausted,
        w::ApplicationOutcome::CoreRefused(value) => {
            let w::ApplicationOutcomeCoreRefused { why } = value;
            d::ApplicationOutcome::CoreRefused { why: decode_refusal(why) }
        }
    }
}
fn encode_record(value: &d::Record) -> w::Value {
    match value {
        d::Record::SettingReplaced { kind, scope, number, revision, value } => {
            w::Value::SettingReplaced(w::RecordSettingReplaced {
                kind: *kind,
                scope: *scope,
                number: *number,
                revision: *revision,
                value: value.clone(),
            })
        }
        d::Record::SettingDefault { kind, scope, number, value } => w::Value::SettingDefault(w::RecordSettingDefault {
            kind: *kind,
            scope: *scope,
            number: *number,
            value: value.clone(),
        }),
        d::Record::SettingChange { kind, scope, number, value } => w::Value::SettingChange(w::RecordSettingChange {
            kind: *kind,
            scope: *scope,
            number: *number,
            value: value.clone(),
        }),
        d::Record::SettingProposal { proposal, kind, scope, number, edit } => {
            w::Value::SettingProposal(w::RecordSettingProposal {
                proposal: *proposal,
                kind: *kind,
                scope: *scope,
                number: *number,
                edit: encode_setting_edit(edit),
            })
        }
        d::Record::ApplicationObject { object, now } => w::Value::ApplicationObject(w::RecordApplicationObject {
            object: *object,
            now: encode_application_object(now),
        }),
        d::Record::ApplicationAnswer { token, until, outcome } => {
            w::Value::ApplicationAnswer(w::RecordApplicationAnswer {
                token: token.raw(),
                until: until.as_nanos(),
                outcome: encode_application_outcome(outcome),
            })
        }
        d::Record::Proposal { number, task, project, effect } => w::Value::Proposal(w::RecordProposal {
            number: *number,
            task: *task,
            project: *project,
            effect: encode_effect(effect),
        }),
        d::Record::Procedure(value) => w::Value::Procedure(w::RecordProcedure { value: encode_procedure_state(value) }),
        d::Record::ProcedureDelegateOutcome { task, step, child, revisions } => {
            encode_step(*task, *step, *child, revisions)
        }
        d::Record::Result { task, code } => w::Value::Result(w::RecordResult { task: *task, code: *code }),
        d::Record::Task { task, project, resources } => {
            w::Value::Task(w::RecordTask { task: *task, project: *project, resources: encode_paths(resources) })
        }
        d::Record::Adoption { project, path, role } => w::Value::Adoption(w::RecordAdoption {
            project: *project,
            path: encode_path(path),
            role: encode_resource_role(*role),
        }),
        d::Record::Subscription { topic, task, wake_at, keep_at } => w::Value::Subscription(w::RecordSubscription {
            topic: *topic,
            task: *task,
            wake_at: *wake_at,
            keep_at: *keep_at,
        }),
        d::Record::Pool { path, slots } => w::Value::Pool(w::RecordPool { path: encode_path(path), slots: *slots }),
        d::Record::Outbox(value) => w::Value::Outbox(w::RecordOutbox { value: encode_outbox_entry(value) }),
        d::Record::Made { key, resources, state } => {
            w::Value::Made(w::RecordMade { key: encode_key(key), resources: encode_paths(resources), state: *state })
        }
    }
}
fn decode_record(value: w::Value, limits: &d::Limits) -> Result<d::Record, Undecodable> {
    Ok(match value {
        w::Value::SettingReplaced(value) => {
            let w::RecordSettingReplaced { kind, scope, number, revision, value } = value;
            d::Record::SettingReplaced { kind, scope, number, revision, value }
        }
        w::Value::SettingDefault(value) => {
            let w::RecordSettingDefault { kind, scope, number, value } = value;
            d::Record::SettingDefault { kind, scope, number, value }
        }
        w::Value::SettingChange(value) => {
            let w::RecordSettingChange { kind, scope, number, value } = value;
            d::Record::SettingChange { kind, scope, number, value }
        }
        w::Value::SettingProposal(value) => {
            let w::RecordSettingProposal { proposal, kind, scope, number, edit } = value;
            d::Record::SettingProposal { proposal, kind, scope, number, edit: decode_setting_edit(edit) }
        }
        w::Value::ApplicationObject(value) => {
            let w::RecordApplicationObject { object, now } = value;
            d::Record::ApplicationObject { object, now: decode_application_object(now) }
        }
        w::Value::ApplicationAnswer(value) => {
            let w::RecordApplicationAnswer { token, until, outcome } = value;
            d::Record::ApplicationAnswer {
                token: Token::new(token),
                until: Wall::from_nanos(until),
                outcome: decode_application_outcome(outcome),
            }
        }
        w::Value::Proposal(value) => {
            let w::RecordProposal { number, task, project, effect } = value;
            d::Record::Proposal { number, task, project, effect: decode_effect(effect, limits)? }
        }
        w::Value::Procedure(value) => {
            let w::RecordProcedure { value } = value;
            d::Record::Procedure(decode_procedure_state(value, limits)?)
        }
        w::Value::ProcedureDelegateOutcome(value) => {
            let w::RecordProcedureDelegateOutcome { task, step, child, revisions } = value;
            if revisions.len() > limits.tasks {
                return Err(Undecodable);
            }
            let mut decoded = List::with_capacity(revisions.len());
            for value in revisions.into_boxed() {
                let w::SpecRevision { task, revision } = value;
                decoded.push((task, revision)).expect("one revision per entry");
            }
            let revisions = decoded.into_boxed();
            d::Record::ProcedureDelegateOutcome { task, step, child, revisions }
        }
        w::Value::Result(value) => {
            let w::RecordResult { task, code } = value;
            d::Record::Result { task, code }
        }
        w::Value::Task(value) => {
            let w::RecordTask { task, project, resources } = value;
            d::Record::Task { task, project, resources: decode_paths(resources, limits)? }
        }
        w::Value::Adoption(value) => {
            let w::RecordAdoption { project, path, role } = value;
            d::Record::Adoption { project, path: decode_path(path, limits)?, role: decode_resource_role(role) }
        }
        w::Value::Subscription(value) => {
            let w::RecordSubscription { topic, task, wake_at, keep_at } = value;
            d::Record::Subscription { topic, task, wake_at, keep_at }
        }
        w::Value::Pool(value) => {
            let w::RecordPool { path, slots } = value;
            d::Record::Pool { path: decode_path(path, limits)?, slots }
        }
        w::Value::Outbox(value) => {
            let w::RecordOutbox { value } = value;
            d::Record::Outbox(decode_outbox_entry(value, limits)?)
        }
        w::Value::Made(value) => {
            let w::RecordMade { key, resources, state } = value;
            d::Record::Made { key: decode_key(key), resources: decode_paths(resources, limits)?, state }
        }
    })
}

fn encode_paths(paths: &[d::Path]) -> List<w::ResourcePath> {
    let mut result = List::with_capacity(u32::try_from(paths.len()).expect("bounded paths"));
    for path in paths {
        result.push(encode_path(path)).expect("same count");
    }
    result
}
fn decode_paths(paths: List<w::ResourcePath>, limits: &d::Limits) -> Result<Box<[d::Path]>, Undecodable> {
    let mut result = List::with_capacity(paths.len());
    for path in paths.into_boxed() {
        result.push(decode_path(path, limits)?).expect("same count");
    }
    Ok(result.into_boxed())
}
fn encode_path(path: &d::Path) -> w::ResourcePath {
    let mut segments = List::with_capacity(u32::try_from(path.segments().len()).expect("bounded path"));
    for bytes in path.segments() {
        segments.push(w::Segment { bytes: bytes.clone() }).expect("same count");
    }
    w::ResourcePath { segments }
}
fn decode_path(path: w::ResourcePath, limits: &d::Limits) -> Result<d::Path, Undecodable> {
    let mut segments = List::with_capacity(path.segments.len());
    for segment in path.segments.into_boxed() {
        segments.push(segment.bytes).expect("same count");
    }
    d::Path::new(segments, limits.segment_bytes).ok_or(Undecodable)
}

fn encode_step(task: u64, step: u64, child: Option<u64>, revisions: &[(u64, u64)]) -> w::Value {
    let mut encoded = List::with_capacity(u32::try_from(revisions.len()).expect("bounded revisions"));
    for (task, revision) in revisions {
        encoded.push(w::SpecRevision { task: *task, revision: *revision }).expect("one revision per entry");
    }
    w::Value::ProcedureDelegateOutcome(w::RecordProcedureDelegateOutcome { task, step, child, revisions: encoded })
}

fn encode_setting_edit(edit: &d::SettingEdit) -> w::SettingEdit {
    match edit {
        d::SettingEdit::Set(value) => w::SettingEdit::Set(w::SettingSet { value: value.clone() }),
        d::SettingEdit::Remove => w::SettingEdit::Remove,
        d::SettingEdit::Reset => w::SettingEdit::Reset,
    }
}
fn decode_setting_edit(edit: w::SettingEdit) -> d::SettingEdit {
    match edit {
        w::SettingEdit::Set(value) => d::SettingEdit::Set(value.value),
        w::SettingEdit::Remove => d::SettingEdit::Remove,
        w::SettingEdit::Reset => d::SettingEdit::Reset,
    }
}
